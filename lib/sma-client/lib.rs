/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
#![forbid(unsafe_code)]

use bytes::{Buf, BytesMut};
use slog::{trace, Logger};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

mod cmds;

#[cfg(test)]
mod tests;

use crate::cmds::*;
pub use cmds::{SmaData, TimestampedInt};

pub struct SmaClient {
    buffer: BytesMut,
    packet_id: u16,
    dst_addr: SocketAddr,
    dst_susy_id: u16,
    dst_serial: u32,
    logger: Option<Logger>,
}

impl SmaClient {
    const BUFFER_SIZE: usize = 1024;

    pub fn new(logger: Option<Logger>) -> SmaClient {
        let buffer = BytesMut::with_capacity(SmaClient::BUFFER_SIZE);

        return SmaClient {
            buffer: buffer,
            packet_id: 0,
            dst_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            dst_susy_id: 0,
            dst_serial: 0,
            logger: logger,
        };
    }

    pub async fn open(&mut self) -> Result<UdpSocket, String> {
        let multicast_addr = Ipv4Addr::new(238, 12, 255, 254);
        let local_addr = Ipv4Addr::new(0, 0, 0, 0);
        let socket =
            match UdpSocket::bind(SocketAddrV4::new(local_addr, 0)).await {
                Ok(x) => x,
                Err(e) => {
                    return Err(format!("Binding socket failed, error: {}", e))
                }
            };
        // TODO: multicast stuff seems to be unneccessary
        if let Err(e) = socket.set_multicast_loop_v4(false) {
            return Err(format!("Disable multicast loop failed, error: {}", e));
        }
        if let Err(e) = socket.join_multicast_v4(multicast_addr, local_addr) {
            return Err(format!("Join multicast group failed, error: {}", e));
        }
        return Ok(socket);
    }

    // TODO: don't panic
    fn merge_rx_data(&self, data: &mut SmaData, mut new_data: SmaData) {
        match data {
            SmaData::None() => {
                std::mem::swap(data, &mut new_data);
                return;
            }
            _ => (),
        }
        match (data, new_data) {
            (
                SmaData::IntTimeSeries(ref mut x),
                SmaData::IntTimeSeries(ref mut y),
            ) => x.append(y),
            _ => panic!("💩️"),
        }
    }

    async fn issue_command(
        &mut self,
        socket: &UdpSocket,
        cmd: &dyn SmaCmd,
        dst_addr: SocketAddr,
    ) -> Result<SmaData, String> {
        self.buffer.clear();
        cmd.serialize(&mut self.buffer);
        self.write(socket, dst_addr).await?;

        let mut data: SmaData = SmaData::None();
        let mut fragment_count = 1;
        while fragment_count != 0 {
            self.read(socket, dst_addr).await?;
            let mut buf =
                std::io::Cursor::new(&self.buffer[0..self.buffer.len()]);

            let responses = match parse_response(&mut buf, &self.logger) {
                Err(e) => return Err(e),
                Ok(x) => x,
            };
            for response in responses.into_iter() {
                match &self.logger {
                    Some(x) => {
                        trace!(
                            x,
                            "Received packet {}, fragment {}",
                            response.packet_id() & 0x7FFF,
                            response.fragment_id()
                        );
                    }
                    None => (),
                }

                if response.packet_id() & 0x7FFF != self.packet_id - 1 {
                    return Err(format!(
                        "💩️ received packet ID {:X}, expected {:X}",
                        response.packet_id() & 0x7FFF,
                        self.packet_id - 1
                    ));
                }
                if response.opcode() != cmd.opcode() {
                    return Err(format!(
                        "💩️ received opcode {:X}, expected {:X}",
                        response.opcode(),
                        cmd.opcode()
                    ));
                }
                // TODO: handle ordering issues
                fragment_count = response.fragment_id();
                // TODO: reserve capacity for fragments in first vector
                self.merge_rx_data(&mut data, response.extract_data());
            }
        }
        return Ok(data);
    }

    fn init_cmd_data_header(
        &mut self,
        header: &mut SmaDataHeader,
        broadcast: bool,
    ) {
        header.packet_id = self.packet_id | 0x8000;
        self.packet_id += 1;
        if self.packet_id >= 0x8000 {
            self.packet_id = 0;
        }

        if broadcast {
            header.dst.susy_id = 0xFFFF;
            header.dst.serial = 0xFFFFFFFF;
        } else {
            header.dst.susy_id = self.dst_susy_id;
            header.dst.serial = self.dst_serial;
        }
        header.app.susy_id = 0xDEAD;
        header.app.serial = 0xDEADBEEA;
    }

    pub fn sma_sock_addr(addr: String) -> Result<SocketAddr, String> {
        let dst_addr = format!("{}:9522", addr).parse::<SocketAddr>();
        match dst_addr {
            Ok(x) => return Ok(x),
            Err(_) => return Err(format!("💩️ {} is not an IP address", addr)),
        }
    }

    pub fn set_dst(&mut self, addr: SocketAddr, susy_id: u16, serial: u32) {
        self.dst_addr = addr;
        self.dst_susy_id = susy_id;
        self.dst_serial = serial;
    }

    pub async fn identify(
        &mut self,
        socket: &UdpSocket,
        dst_addr: SocketAddr,
    ) -> Result<SmaEndpoint, String> {
        match &self.logger {
            Some(x) => trace!(x, "Identify"),
            None => (),
        }
        let mut cmd = SmaCmdIdentify::new();
        self.init_cmd_data_header(&mut cmd.data_header, true);
        match self.issue_command(socket, &cmd, dst_addr).await {
            Ok(x) => match x {
                SmaData::Endpoint(x) => return Ok(x),
                _ => return Err("💩️ received unexpected data type".to_string()),
            },
            Err(e) => return Err(e),
        }
    }

    pub async fn login(
        &mut self,
        socket: &UdpSocket,
        passwd: &String,
    ) -> Result<(), String> {
        match &self.logger {
            Some(x) => trace!(x, "Login"),
            None => (),
        }
        let mut cmd = SmaCmdLogin::new(&self.logger);
        self.init_cmd_data_header(&mut cmd.data_header, true);
        cmd.set_password(passwd);
        match self.issue_command(socket, &cmd, self.dst_addr).await {
            Ok(x) => match x {
                SmaData::None() => return Ok(()),
                _ => return Err("💩️ received unexpected data type".to_string()),
            },
            Err(e) => return Err(e),
        }
    }

    pub async fn logout(&mut self, socket: &UdpSocket) -> Result<(), String> {
        match &self.logger {
            Some(x) => trace!(x, "Logout"),
            None => (),
        }
        let mut cmd = SmaCmdLogout::new();
        self.init_cmd_data_header(&mut cmd.data_header, true);
        self.buffer.clear();
        cmd.serialize(&mut self.buffer);
        self.write(socket, self.dst_addr).await?;
        return Ok(());
    }

    pub async fn get_day_data(
        &mut self,
        socket: &UdpSocket,
        start_time: u32,
        end_time: u32,
    ) -> Result<Vec<TimestampedInt>, String> {
        match &self.logger {
            Some(x) => trace!(x, "GetDayData"),
            None => (),
        }
        let mut cmd = SmaCmdGetDayData::new();
        self.init_cmd_data_header(&mut cmd.data_header, false);
        cmd.start_time = start_time;
        cmd.end_time = end_time;
        match self.issue_command(socket, &cmd, self.dst_addr).await {
            Ok(x) => match x {
                SmaData::IntTimeSeries(x) => return Ok(x),
                _ => return Err("💩️ received unexpected data type".to_string()),
            },
            Err(e) => return Err(e),
        }
    }

    async fn read(
        &mut self,
        socket: &UdpSocket,
        dst_addr: SocketAddr,
    ) -> Result<usize, String> {
        self.buffer.clear();
        match &self.logger {
            Some(x) => {
                trace!(
                    x,
                    "Cap: {}, Len: {}",
                    self.buffer.capacity(),
                    self.buffer.len()
                );
            }
            None => (),
        }

        self.buffer.resize(SmaClient::BUFFER_SIZE, 0);
        let (num_recv, src_addr) = match timeout(
            Duration::from_secs(5),
            socket.recv_from(&mut self.buffer),
        )
        .await
        {
            Err(_) => return Err("Read timed out".into()),
            Ok(x) => match x {
                Err(e) => {
                    return Err(format!("Reading from socket failed: {}", e))
                }
                Ok((rx, addr)) => (rx, addr),
            },
        };
        self.buffer.resize(num_recv, 0);

        if src_addr != dst_addr {
            return Err(format!(
                "💩️ received data from {}, expected {}",
                src_addr, self.dst_addr
            ));
        }
        return Ok(num_recv);
    }

    async fn write(
        &mut self,
        socket: &UdpSocket,
        dst_addr: SocketAddr,
    ) -> Result<(), String> {
        return match socket.send_to(self.buffer.as_ref(), dst_addr).await {
            // TODO: output socket error
            Err(e) => Err("💩️ send data failed".to_string()),
            Ok(_) => Ok(()),
        };
    }
}
