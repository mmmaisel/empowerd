extern crate bytes;
use bytes::{Buf, BufMut, BytesMut};
use std::time;

use super::*;

pub struct SmaCmdLogin {
    pub pkt_header: SmaPacketHeader,
    pub data_header: SmaDataHeader,
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
        self.data_header.serialize(buffer);
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
                SmaDataHeader::LENGTH + SmaCmdLogin::LENGTH,
            ),
            data_header: SmaDataHeader::new(),
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
            .data_header
            .infer_wordcount(retval.pkt_header.data_len);
        retval.data_header.class = SmaDataHeader::CMD_CLASS_A0;
        retval.data_header.app.ctrl = 1;
        retval.data_header.dst.ctrl = 1;
        return retval;
    }

    pub fn set_password(&mut self, passwd: &String) {
        let encoded_password: Vec<u8> =
            passwd.bytes().into_iter().map(|x| x + 0x88).collect();
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
    pub data_header: SmaDataHeader,
    pub cmd: SmaCmdWord,
    pub payload: SmaPayloadLogin,
    pub end: SmaEndToken,
}

impl SmaResponse for SmaResponseLogin {
    fn extract_data(&self) -> SmaData {
        return SmaData::None();
    }

    fn validate(&self) -> Result<(), String> {
        self.pkt_header.validate()?;
        if self.pkt_header.data_len != SmaResponseLogin::LENGTH_MIN
            && self.pkt_header.data_len != SmaResponseLogin::LENGTH_MAX
        {
            return Err("SmaResponseLogin has invalid length".to_string());
        }
        self.data_header.validate()?;
        self.cmd.validate()?;
        self.payload.validate()?;
        self.end.validate()?;
        return Ok(());
    }

    fn fragment_id(&self) -> u16 {
        return self.data_header.fragment_id;
    }

    fn packet_id(&self) -> u16 {
        return self.data_header.packet_id;
    }

    fn opcode(&self) -> u32 {
        return self.cmd.opcode();
    }
}

impl SmaResponseLogin {
    pub const LENGTH_MAX: u16 = 0x003A;
    pub const LENGTH_MIN: u16 = 0x002E;
}
