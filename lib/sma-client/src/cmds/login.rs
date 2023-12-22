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
use super::*;
use bytes::{Buf, BufMut, BytesMut};
use std::time;

pub struct SmaCmdLogin {
    pub pkt_header: SmaPacketHeader,
    pub inv_header: SmaInvHeader,
    pub cmd: SmaCmdWord,
    pub user_group: u32,
    pub timeout: u32,
    pub timestamp: u32,
    pub _padding: u32,
    pub password: [u8; 12],
    pub end: SmaEndToken,
}

impl SmaCmd for SmaCmdLogin {
    fn serialize(&self, buffer: &mut BytesMut) {
        self.pkt_header.serialize(buffer);
        self.inv_header.serialize(buffer);
        self.cmd.serialize(buffer);
        buffer.put_u32_le(self.user_group);
        buffer.put_u32_le(self.timeout);
        buffer.put_u32_le(self.timestamp);
        buffer.put_u32_le(self._padding);
        buffer.put_slice(&self.password);
        self.end.serialize(buffer);
    }

    fn opcode(&self) -> u32 {
        return self.cmd.opcode();
    }
}

impl SmaCmdLogin {
    pub const OPCODE: u32 = 0xFFFD04;
    pub const LENGTH: u16 = 32;

    pub fn new(logger: &Option<Logger>) -> SmaCmdLogin {
        let mut retval = SmaCmdLogin {
            pkt_header: SmaPacketHeader::new(
                SmaInvHeader::LENGTH + SmaCmdLogin::LENGTH,
                SmaPacketHeader::SMA_PROTOCOL_INV,
            ),
            inv_header: SmaInvHeader::new(),
            cmd: SmaCmdWord::new(0x0C, SmaCmdLogin::OPCODE),
            user_group: 7,
            timeout: 900, //60,
            timestamp: time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            _padding: 0,
            password: [0x88; 12],
            end: SmaEndToken::new(),
        };
        match &logger {
            Some(x) => trace!(x, "Local timestamp: {}", retval.timestamp),
            None => (),
        }
        retval
            .inv_header
            .infer_wordcount(retval.pkt_header.data_len);
        retval.inv_header.class = SmaInvHeader::CMD_CLASS_A0;
        retval.inv_header.app.ctrl = 1;
        retval.inv_header.dst.ctrl = 1;
        return retval;
    }

    pub fn set_password(&mut self, passwd: &str) {
        let encoded_password: Vec<u8> =
            passwd.bytes().map(|x| x + 0x88).collect();
        for (dst, src) in self.password[0..encoded_password.len()]
            .iter_mut()
            .zip(encoded_password.iter())
        {
            *dst = *src;
        }
    }
}

pub struct SmaPayloadLogin {
    pub user_group: u32,
    pub timeout: u32,
    pub timestamp: u32,
    pub _padding: u32,
    pub password: [u8; 12],
}

impl SmaPayloadLogin {
    pub const LENGTH_MIN: usize = 16;
    #[allow(unused)]
    pub const LENGTH_MAX: usize = 28;

    pub fn deserialize(buffer: &mut dyn Buf) -> SmaPayloadLogin {
        let user_group = buffer.get_u32_le();
        let timeout = buffer.get_u32_le();
        let timestamp = buffer.get_u32_le();
        let padding = buffer.get_u32_le();
        let mut password: [u8; 12] = [0; 12];
        if buffer.remaining() >= 12 {
            buffer.copy_to_slice(&mut password);
        }

        return SmaPayloadLogin {
            user_group: user_group,
            timeout: timeout,
            timestamp: timestamp,
            _padding: padding,
            password: password,
        };
    }

    pub fn validate(&self) -> Result<(), String> {
        // TODO
        return Ok(());
    }
}

pub struct SmaResponseLogin {
    pub pkt_header: SmaPacketHeader,
    pub inv_header: SmaInvHeader,
    pub cmd: SmaCmdWord,
    pub payload: SmaPayloadLogin,
    pub end: SmaEndToken,
}

impl SmaResponse for SmaResponseLogin {
    fn extract_data(&self) -> SmaData {
        SmaData::None()
    }

    fn get_header(&self) -> SmaHeader {
        SmaHeader::Inv(&self.inv_header)
    }

    fn opcode(&self) -> u32 {
        self.cmd.opcode()
    }

    fn validate(&self) -> Result<(), String> {
        self.pkt_header.validate()?;
        if self.pkt_header.data_len != SmaResponseLogin::LENGTH_MIN
            && self.pkt_header.data_len != SmaResponseLogin::LENGTH_MAX
        {
            return Err("SmaResponseLogin has invalid length".to_string());
        }
        self.inv_header.validate()?;
        self.cmd.validate()?;
        self.payload.validate()?;
        self.end.validate()?;
        return Ok(());
    }
}

impl SmaResponseLogin {
    pub const LENGTH_MAX: u16 = 0x003A;
    pub const LENGTH_MIN: u16 = 0x002E;
}
