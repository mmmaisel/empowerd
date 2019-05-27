mod cmds;

extern crate bytes;
use bytes::{BytesMut, Buf};

use std::net;

use crate::cmds::*;
pub use cmds::SmaData;
pub use cmds::SmaData::*;
pub use cmds::TimestampedInt;

pub struct SmaClient
{
    socket: net::UdpSocket,
    buffer: BytesMut,
    packet_id: u16,
    dst_addr: net::SocketAddr,
    dst_susy_id: u16,
    dst_serial: u32
    // TODO: use feature to add slog logging here
}

impl SmaClient
{
    const BUFFER_SIZE: usize = 1024;

    pub fn new() -> SmaClient
    {
        let multicast_addr = net::SocketAddrV4::new(
            net::Ipv4Addr::new(238,12,255,254), 0);
        let local_addr = net::SocketAddrV4::new(
            net::Ipv4Addr::new(0,0,0,0), 0);
        let socket = net::UdpSocket::bind(local_addr).
            expect("💩️ Binding socket failed");
        socket.set_read_timeout(Some(<std::time::Duration>::new(5, 0))).
            expect("💩️ Set socket timeout failed");
        // TODO: multicast stuff seems to be unneccessary
        socket.set_multicast_loop_v4(false).
            expect("💩️ Disable multicast loop failed");
        socket.join_multicast_v4(&multicast_addr.ip(), &local_addr.ip()).
            expect("💩️ Join multicast group failed");
        let buffer = BytesMut::with_capacity(SmaClient::BUFFER_SIZE);
        return SmaClient
        {
            socket: socket,
            buffer: buffer,
            packet_id: 0,
            dst_addr: net::SocketAddr::new(net::IpAddr::V4(
                net::Ipv4Addr::new(0,0,0,0)), 0),
            dst_susy_id: 0,
            dst_serial: 0
        };
    }

    // TODO: don't panic
    fn merge_rx_data(&self, data: &mut SmaData, mut new_data: SmaData)
    {
        match data
        {
            None() =>
            {
                std::mem::swap(data, &mut new_data);
                return;
            }
            _ => ()
        }
        match (data, new_data)
        {
            (SmaData::IntTimeSeries(ref mut x),
             SmaData::IntTimeSeries(ref mut y)) => x.append(y),
            _ => panic!("💩️")
        }
    }

    fn issue_command(&mut self, cmd: &SmaCmd, dst_addr: net::SocketAddr)
        -> Result<SmaData, String>
    {
        self.buffer.clear();
        cmd.serialize(&mut self.buffer);
        self.write(dst_addr)?;
        let mut data: SmaData = SmaData::None();
        let mut fragment_count = 1;
        while fragment_count != 0
        {
            self.read(dst_addr)?;
            let mut buf = std::io::Cursor::new(
                &self.buffer[0..self.buffer.len()]);

            let responses = match parse_response(&mut buf)
            {
                Err(e) => return Err(e),
                Ok(x) => x
            };
            for response in responses.into_iter()
            {
                if cfg!(debug_assertions)
                {
                    println!("Received packet {}, fragment {}",
                        response.packet_id() & 0x7FFF, response.fragment_id());
                }

                if response.packet_id() & 0x7FFF != self.packet_id - 1
                {
                    return Err(format!("💩️ received packet ID {:X}, expected {:X}",
                        response.packet_id() & 0x7FFF, self.packet_id - 1));
                }
                if response.opcode() != cmd.opcode()
                {
                    return Err(format!("💩️ received opcode {:X}, expected {:X}",
                        response.opcode(), cmd.opcode()));
                }
                // TODO: handle ordering issues
                fragment_count = response.fragment_id();
                // TODO: reserve capacity for fragments in first vector
                self.merge_rx_data(&mut data, response.extract_data());
            }
        }
        return Ok(data);
    }

    fn init_cmd_data_header(&mut self, header: &mut SmaDataHeader,
        broadcast: bool)
    {
        header.packet_id = self.packet_id | 0x8000;
        self.packet_id += 1;
        if broadcast
        {
            header.dst.susy_id = 0xFFFF;
            header.dst.serial = 0xFFFFFFFF;
        }
        else
        {
            header.dst.susy_id = self.dst_susy_id;
            header.dst.serial = self.dst_serial;
        }
        header.app.susy_id = 0xDEAD;
        header.app.serial = 0xDEADBEEA;
    }

    pub fn sma_sock_addr(addr: String) -> Result<net::SocketAddr, String>
    {
        let dst_addr = format!("{}:9522", addr).parse::<net::SocketAddr>();
        match dst_addr
        {
            Ok(x) => return Ok(x),
            Err(_) => return Err(format!("💩️ {} is not an IP address", addr))
        }
    }

    pub fn set_dst(&mut self, addr: net::SocketAddr, susy_id: u16, serial: u32)
    {
        self.dst_addr = addr;
        self.dst_susy_id = susy_id;
        self.dst_serial = serial;
    }

    pub fn identify(&mut self, dst_addr: net::SocketAddr)
        -> Result<SmaEndpoint, String>
    {
        if cfg!(debug_assertions)
        {
            println!("Identify");
        }
        let mut cmd = SmaCmdIdentify::new();
        self.init_cmd_data_header(&mut cmd.data_header, true);
        match self.issue_command(&cmd, dst_addr)
        {
            Ok(x) =>
            {
                match x
                {
                    Endpoint(x) => return Ok(x),
                    _ => return Err("💩️ received unexpected data type".to_string())
                }
            }
            Err(e) => return Err(e)
        }
    }

    pub fn login(&mut self, passwd: &String) -> Result<(), String>
    {
        if cfg!(debug_assertions)
        {
            println!("login");
        }
        let mut cmd = SmaCmdLogin::new();
        self.init_cmd_data_header(&mut cmd.data_header, true);
        cmd.set_password(passwd);
        match self.issue_command(&cmd, self.dst_addr)
        {
            Ok(x) =>
            {
                match x
                {
                    None() => return Ok(()),
                    _ => return Err("💩️ received unexpected data type".to_string())
                }
            }
            Err(e) => return Err(e)
        }
    }

    pub fn logout(&mut self) -> Result<(), String>
    {
        if cfg!(debug_assertions)
        {
            println!("logout");
        }
        let mut cmd = SmaCmdLogout::new();
        self.init_cmd_data_header(&mut cmd.data_header, true);
        self.buffer.clear();
        cmd.serialize(&mut self.buffer);
        self.write(self.dst_addr)?;
        return Ok(());
    }

    pub fn get_day_data(&mut self, start_time: u32, end_time: u32)
        -> Result<Vec<TimestampedInt>, String>
    {
        if cfg!(debug_assertions)
        {
            println!("get_day_data");
        }
        let mut cmd = SmaCmdGetDayData::new();
        self.init_cmd_data_header(&mut cmd.data_header, false);
        cmd.start_time = start_time;
        cmd.end_time = end_time;
        match self.issue_command(&cmd, self.dst_addr)
        {
            Ok(x) =>
            {
                match x
                {
                    IntTimeSeries(x) => return Ok(x),
                    _ => return Err("💩️ received unexpected data type".to_string())
                }
            }
            Err(e) => return Err(e)
        }
    }

    fn read(&mut self, dst_addr: net::SocketAddr)
        -> Result<usize, String>
    {
        self.buffer.clear();
        if cfg!(debug_assertions)
        {
            println!("Cap: {}, Len: {}",
                self.buffer.capacity(), self.buffer.len());
        }
        unsafe { self.buffer.set_len(SmaClient::BUFFER_SIZE); }
        let (num_recv, src_addr) = match
            self.socket.recv_from(&mut self.buffer)
        {
            // TODO: output socket error
            Err(e) => return Err("💩️ Nothing received".to_string()),
            Ok(x) => x
        };
        unsafe { self.buffer.set_len(num_recv); }
        if src_addr != dst_addr
        {
            return Err(format!("💩️ received data from {}, expected {}",
                src_addr, self.dst_addr));
        }
        return Ok(num_recv);
    }

    fn write(&mut self, dst_addr: net::SocketAddr) -> Result<(), String>
    {
        return match self.socket.send_to(self.buffer.as_ref(), dst_addr)
        {
            // TODO: output socket error
            Err(e) => Err("💩️ send data failed".to_string()),
            Ok(_) => Ok(())
        };
    }
}
