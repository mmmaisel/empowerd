/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2023 Max Maisel

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
use bytes::{BufMut, BytesMut};
use std::collections::BTreeMap;

// TODO: get rid of big endian values

mod em_message;
mod get_day_data;
mod identify;
mod login;
mod logout;

pub use em_message::{SmaEmMessage, SmaEmPayload};
pub use get_day_data::{
    SmaCmdGetDayData, SmaPayloadGetDayData, SmaResponseGetDayData,
};
pub use identify::{SmaCmdIdentify, SmaPayloadIdentify, SmaResponseIdentify};
pub use login::{SmaCmdLogin, SmaPayloadLogin, SmaResponseLogin};
pub use logout::SmaCmdLogout;

pub trait SmaCmd: Sync {
    fn serialize(&self, buffer: &mut BytesMut);
    fn opcode(&self) -> u32;
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimestampedInt {
    pub timestamp: u32,
    pub value: u32,
}

#[derive(Debug)]
pub enum SmaData {
    None(),
    Endpoint(SmaEndpoint),
    IntTimeSeries(Vec<TimestampedInt>),
    EmPayload(BTreeMap<u32, u64>),
}

#[derive(Debug)]
pub enum SmaHeader<'a> {
    Inv(&'a SmaInvHeader),
    Em(&'a SmaEmHeader),
}

pub trait SmaResponse {
    fn extract_data(&self) -> SmaData;
    fn get_header(&self) -> SmaHeader;
    fn opcode(&self) -> u32;
    fn validate(&self) -> Result<(), String>;
}

#[derive(Clone, Debug)]
pub struct SmaPacketHeader {
    //pub sma_fourcc: u32,
    pub hdr_len: u16,
    pub magic: u16,
    pub group: u32,
    pub data_len: u16,
    pub version: u16,
    pub protocol_id: u16,
}

impl SmaPacketHeader {
    const LENGTH: u16 = 16;
    const SMA_FOURCC: u32 = 0x00414D53; // SMA\0
    const SMA_MAGIC: u16 = 0x02A0;
    const SMA_GROUP: u32 = 1;
    const SMA_PROTOCOL_INV: u16 = 0x6065;
    const SMA_PROTOCOL_EM: u16 = 0x6069;
    const SMA_VERSION: u16 = 0x10;

    fn new(len: u16, protocol_id: u16) -> SmaPacketHeader {
        return SmaPacketHeader {
            hdr_len: SmaPacketHeader::LENGTH / 4,
            magic: SmaPacketHeader::SMA_MAGIC,
            group: SmaPacketHeader::SMA_GROUP,
            data_len: len,
            version: SmaPacketHeader::SMA_VERSION,
            protocol_id,
        };
    }

    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u32_le(SmaPacketHeader::SMA_FOURCC);
        buffer.put_u16(self.hdr_len);
        buffer.put_u16(self.magic);
        buffer.put_u32(self.group);
        buffer.put_u16(self.data_len);
        buffer.put_u16(self.version);
        buffer.put_u16(self.protocol_id);
    }

    fn deserialize(buffer: &mut dyn Buf) -> SmaPacketHeader {
        return SmaPacketHeader {
            //sma_fourcc: buffer.get_u32_le(),
            hdr_len: buffer.get_u16(),
            magic: buffer.get_u16(),
            group: buffer.get_u32(),
            data_len: buffer.get_u16(),
            version: buffer.get_u16(),
            protocol_id: buffer.get_u16(),
        };
    }

    fn validate(&self) -> Result<(), String> {
        /*if self.sma_fourcc != SmaPacketHeader::SMA_FOURCC {
            return Err("Invalid packet FOURCC".into());
        }*/
        if self.hdr_len != SmaPacketHeader::LENGTH / 4 {
            return Err("Invalid header len".into());
        }
        if self.magic != SmaPacketHeader::SMA_MAGIC {
            return Err("Invalid magic number".into());
        }
        if self.group != SmaPacketHeader::SMA_GROUP {
            return Err("Invalid group".into());
        }
        if self.version != SmaPacketHeader::SMA_VERSION {
            return Err("Invalid version".into());
        }
        if self.protocol_id != SmaPacketHeader::SMA_PROTOCOL_INV
            && self.protocol_id != SmaPacketHeader::SMA_PROTOCOL_EM
        {
            return Err(format!("Invalid protocol ID {}", self.protocol_id));
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SmaEndpoint {
    pub susy_id: u16,
    pub serial: u32,
    pub ctrl: u16,
}

impl SmaEndpoint {
    fn new() -> SmaEndpoint {
        return SmaEndpoint {
            susy_id: 0,
            serial: 0,
            ctrl: 0,
        };
    }

    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u16(self.susy_id);
        buffer.put_u32(self.serial);
        buffer.put_u16(self.ctrl);
    }

    fn deserialize(buffer: &mut dyn Buf) -> SmaEndpoint {
        return SmaEndpoint {
            susy_id: buffer.get_u16(),
            serial: buffer.get_u32(),
            ctrl: buffer.get_u16(),
        };
    }

    fn validate(&self) -> Result<(), String> {
        // TODO
        return Ok(());
    }
}

#[derive(Clone, Debug)]
pub struct SmaInvHeader {
    pub wordcount: u8,
    pub class: u8,
    pub dst: SmaEndpoint,
    pub app: SmaEndpoint,
    pub error_code: u16,
    pub fragment_id: u16,
    pub packet_id: u16,
}

impl SmaInvHeader {
    const LENGTH: u16 = 26;
    const CMD_CLASS_A0: u8 = 0xA0;
    const CMD_CLASS_E0: u8 = 0xE0;

    fn new() -> Self {
        Self {
            wordcount: 0,
            class: 0,
            dst: SmaEndpoint::new(),
            app: SmaEndpoint::new(),
            error_code: 0,
            fragment_id: 0,
            packet_id: 0,
        }
    }

    fn infer_wordcount(&mut self, packet_len_bytes: u16) {
        self.wordcount = (packet_len_bytes / 4) as u8;
    }

    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u8(self.wordcount);
        buffer.put_u8(self.class);
        self.dst.serialize(buffer);
        self.app.serialize(buffer);
        buffer.put_u16(self.error_code);
        buffer.put_u16(self.fragment_id);
        buffer.put_u16_le(self.packet_id);
    }

    fn deserialize(buffer: &mut dyn Buf) -> Self {
        Self {
            wordcount: buffer.get_u8(),
            class: buffer.get_u8(),
            dst: SmaEndpoint::deserialize(buffer),
            app: SmaEndpoint::deserialize(buffer),
            error_code: buffer.get_u16(),
            fragment_id: buffer.get_u16_le(),
            packet_id: buffer.get_u16_le(),
        }
    }

    fn validate(&self) -> Result<(), String> {
        // TODO: validate class D0 on RX
        self.dst.validate()?;
        self.app.validate()?;
        if self.error_code != 0 {
            return Err("Device returned an error".into());
        }
        return Ok(());
    }
}

pub struct SmaCmdWord {
    pub word: u32,
}

impl SmaCmdWord {
    fn new(channel: u8, code: u32) -> SmaCmdWord {
        return SmaCmdWord {
            word: (channel as u32) | (code << 8),
        };
    }

    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u32_le(self.word);
    }

    fn deserialize(buffer: &mut dyn Buf) -> SmaCmdWord {
        return SmaCmdWord {
            word: buffer.get_u32_le(),
        };
    }

    pub fn opcode(&self) -> u32 {
        return (self.word & 0xFFFFFF00) >> 8;
    }

    #[allow(unused)]
    pub fn channel(&self) -> u8 {
        return self.word as u8;
    }

    fn validate(&self) -> Result<(), String> {
        // TODO:
        return Ok(());
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmaEmHeader {
    pub susy_id: u16,
    pub serial: u32,
    pub timestamp_ms: u32,
}

impl SmaEmHeader {
    const LENGTH: u16 = 10;

    fn new() -> Self {
        Self {
            susy_id: 0,
            serial: 0,
            timestamp_ms: 0,
        }
    }

    fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u16(self.susy_id);
        buffer.put_u32(self.serial);
        buffer.put_u32(self.timestamp_ms);
    }

    fn deserialize(buffer: &mut dyn Buf) -> Self {
        let mut header = Self::new();
        header.susy_id = buffer.get_u16();
        header.serial = buffer.get_u32();
        header.timestamp_ms = buffer.get_u32();
        header
    }

    fn validate(&self) -> Result<(), String> {
        // TODO: implement
        Ok(())
    }
}

pub struct SmaEndToken {
    end: u32,
}

impl SmaEndToken {
    //    pub const LENGTH: u16 = 0x0004;

    pub fn new() -> SmaEndToken {
        return SmaEndToken { end: 0 };
    }

    pub fn serialize(&self, buffer: &mut BytesMut) {
        buffer.put_u32_le(self.end);
    }

    pub fn deserialize(buffer: &mut dyn Buf) -> SmaEndToken {
        if buffer.remaining() >= 4 {
            let _end = buffer.get_u32_le();
        } else {
            let _end = buffer.get_u16_le();
        }

        return SmaEndToken { end: 0 };
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.end != 0 {
            return Err(format!(
                "Invalid value '{}' at end of packet",
                self.end
            ));
        }
        Ok(())
    }
}

pub fn parse_response(
    mut buffer: &mut dyn Buf,
    logger: &Option<Logger>,
) -> Result<Vec<Box<dyn SmaResponse>>, String> {
    let mut data: Vec<Box<dyn SmaResponse>> = Vec::new();

    while buffer.has_remaining() {
        if buffer.remaining() >= 4 {
            let word = buffer.get_u32_le();
            if word == 0x00000000 {
                // discard superflous end tokens
                buffer.advance(4);
            } else if word == SmaPacketHeader::SMA_FOURCC {
                if buffer.remaining()
                    >= (SmaPacketHeader::LENGTH + SmaInvHeader::LENGTH + 4)
                        as usize
                {
                    let packet = parse_command(&mut buffer, logger);
                    match packet {
                        Ok(packet) => data.push(packet),
                        Err(e) => return Err(e),
                    }
                } else {
                    return Err("Garbage at end of buffer".into());
                }
            }
        } else {
            //buffer.advance(buffer.remaining());
            return Err("Garbage at end of buffer".into());
        }
    }
    return Ok(data);
}

fn parse_command(
    mut buffer: &mut dyn Buf,
    logger: &Option<Logger>,
) -> Result<Box<dyn SmaResponse>, String> {
    let pkt_header = SmaPacketHeader::deserialize(buffer);

    if pkt_header.protocol_id == SmaPacketHeader::SMA_PROTOCOL_INV {
        parse_inv_command(&mut buffer, logger, pkt_header)
    } else if pkt_header.protocol_id == SmaPacketHeader::SMA_PROTOCOL_EM {
        parse_em_command(&mut buffer, logger, pkt_header)
    } else {
        Err(format!(
            "Received invalid protocol ID {}",
            pkt_header.protocol_id
        ))
    }
}

fn parse_inv_command(
    buffer: &mut dyn Buf,
    logger: &Option<Logger>,
    pkt_header: SmaPacketHeader,
) -> Result<Box<dyn SmaResponse>, String> {
    let inv_header = SmaInvHeader::deserialize(buffer);

    let cmd_word = SmaCmdWord::deserialize(buffer);
    match &logger {
        Some(x) => trace!(x, "Received cmd_word: {:X}", cmd_word.word),
        None => (),
    }

    let packet: Box<dyn SmaResponse> = match cmd_word.opcode() {
        SmaCmdIdentify::OPCODE => {
            if buffer.remaining() < SmaPayloadIdentify::LENGTH {
                return Err("Received incomplete SmaIdentify packet".into());
            }
            let payload = SmaPayloadIdentify::deserialize(buffer);
            let end = SmaEndToken::deserialize(buffer);
            Box::new(SmaResponseIdentify {
                pkt_header: pkt_header,
                inv_header: inv_header,
                cmd: cmd_word,
                payload: payload,
                end: end,
            })
        }
        SmaCmdLogin::OPCODE => {
            if buffer.remaining() < SmaPayloadLogin::LENGTH_MIN {
                return Err("Received incomplete SmaLogin packet".into());
            }
            let payload = SmaPayloadLogin::deserialize(buffer);
            let end = SmaEndToken::deserialize(buffer);
            Box::new(SmaResponseLogin {
                pkt_header: pkt_header,
                inv_header: inv_header,
                cmd: cmd_word,
                payload: payload,
                end: end,
            })
        }
        SmaCmdGetDayData::OPCODE => {
            if buffer.remaining() < SmaPayloadGetDayData::MIN_LENGTH {
                return Err("Received packet is too small".into());
            }
            let payload =
                SmaPayloadGetDayData::deserialize(buffer, pkt_header.data_len);
            let end = SmaEndToken::deserialize(buffer);
            // TODO: use macro to de-duplicate this
            Box::new(SmaResponseGetDayData {
                pkt_header: pkt_header,
                inv_header: inv_header,
                cmd: cmd_word,
                payload: payload,
                end: end,
            })
        }
        _ => return Err("Unsupported SMA packet received".into()),
    };
    if let Err(e) = packet.validate() {
        return Err(format!("Packet validation failed {}", e));
    }
    return Ok(packet);
}

fn parse_em_command(
    buffer: &mut dyn Buf,
    _logger: &Option<Logger>,
    pkt_header: SmaPacketHeader,
) -> Result<Box<dyn SmaResponse>, String> {
    let em_header = SmaEmHeader::deserialize(buffer);
    let payload_len = pkt_header.data_len - 2 - SmaEmHeader::LENGTH;

    if buffer.remaining() < payload_len.into() {
        return Err(format!(
            "Remaining buffer length {} is less than payload length {}",
            buffer.remaining(),
            payload_len
        ));
    }

    let payload = SmaEmPayload::deserialize(buffer, payload_len);
    let end = SmaEndToken::deserialize(buffer);
    let packet = Box::new(SmaEmMessage {
        pkt_header,
        em_header,
        payload,
        end,
    });

    match packet.validate() {
        Ok(()) => Ok(packet),
        Err(e) => Err(format!("Packet validation failed {}", e)),
    }
}

#[cfg(test)]
mod tests;
