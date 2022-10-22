/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2022 Max Maisel

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
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

use bytes::{Buf, BytesMut};
use slog::{trace, Logger};
use socket2::{Domain, Socket, Type};
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

mod cmds;

#[cfg(test)]
mod tests;

use crate::cmds::{
    parse_response, SmaCmd, SmaCmdGetDayData, SmaCmdIdentify, SmaCmdLogin,
    SmaCmdLogout, SmaEmHeader, SmaEmMessage, SmaEndpoint, SmaInvHeader,
    SmaResponse,
};

pub use cmds::{SmaData, SmaHeader, TimestampedInt};

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

    pub fn open(
        &mut self,
        addr: SocketAddr,
        mc_bind_addr: Option<Ipv4Addr>,
    ) -> Result<UdpSocket, String> {
        self.dst_addr = addr;
        let local_port = if mc_bind_addr.is_some() { 9522 } else { 0 };
        let any_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), local_port);

        let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)
            .map_err(|e| format!("Opening socket failed: {}", e))?;
        socket
            .bind(&any_addr.into())
            .map_err(|e| format!("Binding socket failed: {}", e))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| format!("Setting nonblocking mode failed: {}", e))?;

        if let Some(local_addr) = mc_bind_addr {
            let multicast_addr = Ipv4Addr::new(239, 12, 255, 254);
            socket
                .set_multicast_loop_v4(false)
                .map_err(|e| format!("Disable multicast loop failed: {}", e))?;
            socket.set_multicast_if_v4(&local_addr).map_err(|e| {
                format!("Setting multicast interface failed: {}", e)
            })?;
            socket
                .join_multicast_v4(&multicast_addr, &local_addr)
                .map_err(|e| format!("Join multicast group failed: {}", e))?;
        }

        UdpSocket::from_std(socket.into())
            .map_err(|e| format!("Creating tokio socket failed: {}", e))
    }

    // TODO: don't panic
    fn merge_rx_data(&self, data: &mut SmaData, mut new_data: SmaData) {
        if let SmaData::None() = data {
            std::mem::swap(data, &mut new_data);
            return;
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
    ) -> Result<SmaData, String> {
        self.buffer.clear();
        cmd.serialize(&mut self.buffer);
        self.write(socket, self.dst_addr).await?;

        let mut data: SmaData = SmaData::None();
        let mut fragment_count = 1;
        while fragment_count != 0 {
            self.read(socket, self.dst_addr).await?;
            let mut buf =
                std::io::Cursor::new(&self.buffer[0..self.buffer.len()]);

            let responses = match parse_response(&mut buf, &self.logger) {
                Err(e) => return Err(e),
                Ok(x) => x,
            };
            for response in responses.into_iter() {
                let header = match response.get_header() {
                    SmaHeader::Inv(header) => header,
                    _ => return Err("Received invalid header type!".into()),
                };
                if let Some(logger) = &self.logger {
                    trace!(
                        logger,
                        "Received packet {}, fragment {}",
                        header.packet_id & 0x7FFF,
                        header.fragment_id,
                    );
                }

                if header.packet_id & 0x7FFF != self.packet_id - 1 {
                    return Err(format!(
                        "Received packet ID {:X}, expected {:X}",
                        header.packet_id & 0x7FFF,
                        self.packet_id - 1
                    ));
                }
                if response.opcode() != cmd.opcode() {
                    return Err(format!(
                        "Received opcode {:X}, expected {:X}",
                        response.opcode(),
                        cmd.opcode()
                    ));
                }
                // TODO: handle ordering issues
                fragment_count = header.fragment_id;
                // TODO: reserve capacity for fragments in first vector
                self.merge_rx_data(&mut data, response.extract_data());
            }
        }
        Ok(data)
    }

    fn init_cmd_inv_header(
        &mut self,
        header: &mut SmaInvHeader,
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
        header.app.serial = 0xDEADBEEF;
    }

    pub fn sma_sock_addr(addr: String) -> Result<SocketAddr, String> {
        let dst_addr = format!("{}:9522", addr).parse::<SocketAddr>();
        match dst_addr {
            Ok(x) => return Ok(x),
            Err(_) => return Err(format!("{} is not an IP address", addr)),
        }
    }

    pub fn set_dst(&mut self, susy_id: u16, serial: u32) {
        self.dst_susy_id = susy_id;
        self.dst_serial = serial;
    }

    pub async fn identify(
        &mut self,
        socket: &UdpSocket,
    ) -> Result<SmaEndpoint, String> {
        match &self.logger {
            Some(x) => trace!(x, "Identify"),
            None => (),
        }
        let mut cmd = SmaCmdIdentify::new();
        self.init_cmd_inv_header(&mut cmd.inv_header, true);
        match self.issue_command(socket, &cmd).await {
            Ok(x) => match x {
                SmaData::Endpoint(x) => return Ok(x),
                _ => {
                    return Err("Received type is not SmaData::Endpoint".into())
                }
            },
            Err(e) => return Err(e),
        }
    }

    pub async fn login(
        &mut self,
        socket: &UdpSocket,
        passwd: &str,
    ) -> Result<(), String> {
        match &self.logger {
            Some(x) => trace!(x, "Login"),
            None => (),
        }
        let mut cmd = SmaCmdLogin::new(&self.logger);
        self.init_cmd_inv_header(&mut cmd.inv_header, true);
        cmd.set_password(passwd);
        match self.issue_command(socket, &cmd).await {
            Ok(x) => match x {
                SmaData::None() => return Ok(()),
                _ => return Err("Received type is not SmaData::None".into()),
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
        self.init_cmd_inv_header(&mut cmd.inv_header, true);
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
        self.init_cmd_inv_header(&mut cmd.inv_header, false);
        cmd.start_time = start_time;
        cmd.end_time = end_time;
        match self.issue_command(socket, &cmd).await {
            Ok(x) => match x {
                SmaData::IntTimeSeries(x) => return Ok(x),
                _ => {
                    return Err(
                        "Received type is not SmaData::IntTimeSeries".into()
                    )
                }
            },
            Err(e) => return Err(e),
        }
    }

    pub async fn fetch_em_data(
        &mut self,
        socket: &UdpSocket,
    ) -> Result<(SmaEmHeader, BTreeMap<u32, u64>), String> {
        self.read(socket, self.dst_addr).await?;
        let mut buf = std::io::Cursor::new(&self.buffer[0..self.buffer.len()]);

        let responses = parse_response(&mut buf, &self.logger)?;
        if responses.len() != 1 {
            return Err("Received multiple or none EM messages.".into());
        }

        let response = responses.into_iter().last().unwrap();
        let header = match response.get_header() {
            SmaHeader::Em(x) => x,
            _ => return Err("Received incorrect SMA header".into()),
        };

        let payload = match response.extract_data() {
            SmaData::EmPayload(x) => x,
            _ => return Err("Received invalid EM response type".into()),
        };

        Ok((header.to_owned(), payload))
    }

    pub async fn broadcast_em_data(
        &mut self,
        socket: &UdpSocket,
        susy_id: u16,
        serial: u32,
        timestamp_ms: u32,
        payload: BTreeMap<u32, u64>,
    ) -> Result<(), String> {
        let mut msg = SmaEmMessage::new();
        msg.em_header.susy_id = susy_id;
        msg.em_header.serial = serial;
        msg.em_header.timestamp_ms = timestamp_ms;
        msg.payload.0 = payload;
        msg.update_len();
        msg.validate()?;

        self.buffer.clear();
        msg.serialize(&mut self.buffer);
        let multicast_dst =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(239, 12, 255, 254)), 9522);
        self.write(socket, multicast_dst).await
    }

    async fn read(
        &mut self,
        socket: &UdpSocket,
        dst_addr: SocketAddr,
    ) -> Result<usize, String> {
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

        let num_recv = match timeout(Duration::from_secs(5), async {
            loop {
                self.buffer.clear();
                self.buffer.resize(SmaClient::BUFFER_SIZE, 0);
                let (num_recv, src_addr) =
                    match socket.recv_from(&mut self.buffer).await {
                        Err(e) => {
                            return Err(format!(
                                "Reading from socket failed: {}",
                                e
                            ))
                        }
                        Ok((rx, addr)) => (rx, addr),
                    };

                if src_addr.ip() == dst_addr.ip() {
                    return Ok(num_recv);
                }
            }
        })
        .await
        {
            Err(_) => return Err("Read timed out".into()),
            Ok(x) => x,
        }?;

        self.buffer.resize(num_recv, 0);
        return Ok(num_recv);
    }

    async fn write(
        &mut self,
        socket: &UdpSocket,
        dst_addr: SocketAddr,
    ) -> Result<(), String> {
        return match socket.send_to(self.buffer.as_ref(), dst_addr).await {
            Err(e) => Err(format!("Write data to SMA device failed: {}", e)),
            Ok(_) => Ok(()),
        };
    }
}
