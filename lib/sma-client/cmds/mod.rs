extern crate bytes;
use bytes::{BufMut, BytesMut};
use bytes::ByteOrder;
extern crate byteorder;
use byteorder::{LittleEndian};

// TODO: get rid of big endian values

use super::*;

mod identify;
pub use identify::*;

mod login;
pub use login::*;

mod logout;
pub use logout::*;

mod get_day_data;
pub use get_day_data::*;

pub trait SmaCmd
{
    fn serialize(&self, buffer: &mut BytesMut);
    fn opcode(&self) -> u32;
}

#[derive(Debug, PartialEq)]
pub struct TimestampedInt
{
    pub timestamp: u32,
    pub value: u32
}

#[derive(Debug)]
pub enum SmaData
{
    None(),
    Endpoint(SmaEndpoint),
    IntTimeSeries(Vec<TimestampedInt>)
}

pub trait SmaResponse
{
    fn extract_data(&self) -> SmaData;
    fn validate(&self) -> Result<(), String>;
    fn fragment_id(&self) -> u16;
    fn packet_id(&self) -> u16;
    fn opcode(&self) -> u32;
}

pub struct SmaPacketHeader
{
    pub sma_fourcc: u32,
    pub hdr_len: u16,
    pub magic: u16,
    pub group: u32,
    pub data_len: u16,
    pub version: u16,
    pub protocol_id: u16
}

impl SmaPacketHeader
{
    const LENGTH: u16 = 16;
    const SMA_FOURCC: u32 = 0x00414D53; // SMA\0
    const SMA_MAGIC: u16 = 0x02A0;
    const SMA_GROUP: u32 = 1;
    const SMA_PROTOCOL_ID: u16 = 0x6065;
    const SMA_VERSION: u16 = 0x10;

    fn new(len: u16) -> SmaPacketHeader
    {
        return SmaPacketHeader
        {
            sma_fourcc: SmaPacketHeader::SMA_FOURCC,
            hdr_len: SmaPacketHeader::LENGTH / 4,
            magic: SmaPacketHeader::SMA_MAGIC,
            group: SmaPacketHeader::SMA_GROUP,
            data_len: len,
            version: SmaPacketHeader::SMA_VERSION,
            protocol_id: SmaPacketHeader::SMA_PROTOCOL_ID
        };
    }

    fn serialize(&self, buffer: &mut BytesMut)
    {
        buffer.put_u32_le(self.sma_fourcc);
        buffer.put_u16_be(self.hdr_len);
        buffer.put_u16_be(self.magic);
        buffer.put_u32_be(self.group);
        buffer.put_u16_be(self.data_len);
        buffer.put_u16_be(self.version);
        buffer.put_u16_be(self.protocol_id);
    }

    fn deserialize(buffer: &mut Buf) -> SmaPacketHeader
    {
        return SmaPacketHeader
        {
            sma_fourcc: buffer.get_u32_le(),
            hdr_len: buffer.get_u16_be(),
            magic: buffer.get_u16_be(),
            group: buffer.get_u32_be(),
            data_len: buffer.get_u16_be(),
            version: buffer.get_u16_be(),
            protocol_id: buffer.get_u16_be()
        }
    }

    fn validate(&self) -> Result<(), String>
    {
        if self.sma_fourcc != SmaPacketHeader::SMA_FOURCC
        {
            return Err("Invalid packet FOURCC".to_string());
        }
        if self.hdr_len != SmaPacketHeader::LENGTH / 4
        {
            return Err("Invalid header len".to_string());
        }
        if self.magic != SmaPacketHeader::SMA_MAGIC
        {
            return Err("Invalid magic number".to_string());
        }
        if self.group != SmaPacketHeader::SMA_GROUP
        {
            return Err("Invalid group".to_string());
        }
        if self.version != SmaPacketHeader::SMA_VERSION
        {
            return Err("Invalid version".to_string());
        }
        if self.protocol_id != SmaPacketHeader::SMA_PROTOCOL_ID
        {
            return Err("Invalid protocol ID".to_string());
        }
        return Ok(());
    }
}

#[derive(Debug)]
pub struct SmaEndpoint
{
    pub susy_id: u16,
    pub serial: u32,
    pub ctrl: u16
}

impl SmaEndpoint
{
    fn new() -> SmaEndpoint
    {
        return SmaEndpoint
        {
            susy_id: 0,
            serial: 0,
            ctrl: 0
        };
    }

    fn serialize(&self, buffer: &mut BytesMut)
    {
        buffer.put_u16_be(self.susy_id);
        buffer.put_u32_be(self.serial);
        buffer.put_u16_be(self.ctrl);
    }

    fn deserialize(buffer: &mut Buf) -> SmaEndpoint
    {
        return SmaEndpoint
        {
            susy_id: buffer.get_u16_be(),
            serial: buffer.get_u32_be(),
            ctrl: buffer.get_u16_be()
        }
    }

    fn validate(&self) -> Result<(), String>
    {
        // TODO
        return Ok(());
    }
}

pub struct SmaDataHeader
{
    pub wordcount: u8,
    pub class: u8,
    pub dst: SmaEndpoint,
    pub app: SmaEndpoint,
    pub error_code: u16,
    pub fragment_id: u16,
    pub packet_id: u16
}

impl SmaDataHeader
{
    const LENGTH: u16 = 26;
    const CMD_CLASS_A0: u8 = 0xA0;
    const CMD_CLASS_E0: u8 = 0xE0;

    fn new() -> SmaDataHeader
    {
        return SmaDataHeader
        {
            wordcount: 0,
            class: 0,
            dst: SmaEndpoint::new(),
            app: SmaEndpoint::new(),
            error_code: 0,
            fragment_id: 0,
            packet_id: 0
        };
    }

    fn infer_wordcount(&mut self, packet_len_bytes: u16)
    {
        self.wordcount = (packet_len_bytes / 4) as u8;
    }

    fn serialize(&self, buffer: &mut BytesMut)
    {
        buffer.put_u8(self.wordcount);
        buffer.put_u8(self.class);
        self.dst.serialize(buffer);
        self.app.serialize(buffer);
        buffer.put_u16_be(self.error_code);
        buffer.put_u16_be(self.fragment_id);
        buffer.put_u16_le(self.packet_id);
    }

    fn deserialize(buffer: &mut Buf) -> SmaDataHeader
    {
        return SmaDataHeader
        {
            wordcount: buffer.get_u8(),
            class: buffer.get_u8(),
            dst: SmaEndpoint::deserialize(buffer),
            app: SmaEndpoint::deserialize(buffer),
            error_code: buffer.get_u16_be(),
            fragment_id: buffer.get_u16_le(),
            packet_id: buffer.get_u16_le()
        };
    }

    fn validate(&self) -> Result<(), String>
    {
        // TODO: validate class D0 on RX
        self.dst.validate()?;
        self.app.validate()?;
        if self.error_code != 0
        {
            return Err("Device returned an error".to_string());
        }
        return Ok(());
    }
}

pub struct SmaCmdWord
{
    pub word: u32
}

impl SmaCmdWord
{
    fn new(channel: u8, code: u32) -> SmaCmdWord
    {
        return SmaCmdWord
        {
            word: (channel as u32) | (code << 8)
        };
    }

    fn serialize(&self, buffer: &mut BytesMut)
    {
        buffer.put_u32_le(self.word);
    }

    fn deserialize(buffer: &mut Buf) -> SmaCmdWord
    {
        return SmaCmdWord
        {
            word: buffer.get_u32_le(),
        };
    }

    pub fn opcode(&self) -> u32
    {
        return (self.word & 0xFFFFFF00) >> 8;
    }

    pub fn channel(&self) -> u8
    {
        return self.word as u8;
    }

    fn validate(&self) -> Result<(), String>
    {
        // TODO:
        return Ok(());
    }
}

pub struct SmaEndToken
{
    end: u32
}

impl SmaEndToken
{
//    pub const LENGTH: u16 = 0x0004;

    pub fn new() -> SmaEndToken
    {
        return SmaEndToken
        {
            end: 0
        };
    }

    pub fn serialize(&self, buffer: &mut BytesMut)
    {
        buffer.put_u32_le(self.end);
    }

    pub fn deserialize(buffer: &mut Buf) -> SmaEndToken
    {
        return SmaEndToken
        {
            end: buffer.get_u32_le()
        };
    }

    pub fn validate(&self) -> Result<(), String>
    {
        if self.end != 0
        {
            return Err("Invalid value at end of packet".to_string());
        }
        return Ok(());
    }
}

pub fn parse_response(mut buffer: &mut Buf)
    -> Result<Vec<Box<SmaResponse>>, String>
{
    let mut data: Vec<Box<SmaResponse>> = Vec::new();

    while buffer.has_remaining()
    {
        if buffer.remaining() >= 4
        {
            let word = LittleEndian::read_u32(&buffer.bytes()[0..4]);
            if word == 0x00000000
            {
                // discard superflous end tokens
                buffer.advance(4);
            }
            else if word == SmaPacketHeader::SMA_FOURCC
            {
                if buffer.remaining() >=
                    (SmaPacketHeader::LENGTH + SmaDataHeader::LENGTH + 4)
                    as usize
                {
                    let packet = parse_command(&mut buffer);
                    match packet
                    {
                        Ok(packet) => data.push(packet),
                        Err(e) => return Err(e)
                    }
                }
                else
                {
                    return Err("💩️ Garbage at end of buffer".to_string());
                }
            }
        }
        else
        {
            //buffer.advance(buffer.remaining());
            return Err("💩️ Garbage at end of buffer".to_string());
        }
    }
    return Ok(data);
}

fn parse_command(buffer: &mut Buf) -> Result<Box<SmaResponse>, String>
{
    let pkt_header = SmaPacketHeader::deserialize(buffer);
    let data_header = SmaDataHeader::deserialize(buffer);

    let cmd_word = SmaCmdWord::deserialize(buffer);
    if cfg!(debug_assertions)
    {
        println!("Received cmd_word: {:X}", cmd_word.word);
    }

    let packet: Box<SmaResponse>;
    match cmd_word.opcode()
    {
        SmaCmdIdentify::OPCODE =>
        {
            if buffer.remaining() < SmaPayloadIdentify::LENGTH
            {
                return Err("💩️ Received incomplete SmaIdentify packet".
                    to_string());
            }
            let payload = SmaPayloadIdentify::deserialize(buffer);
            let end = SmaEndToken::deserialize(buffer);
            packet = Box::new(SmaResponseIdentify
            {
                pkt_header: pkt_header,
                data_header: data_header,
                cmd: cmd_word,
                payload: payload,
                end: end
            });
        }
        SmaCmdLogin::OPCODE =>
        {
            if buffer.remaining() < SmaPayloadLogin::LENGTH
            {
                return Err("💩️ Received incomplete SmaLogin packet".
                    to_string());
            }
            let payload = SmaPayloadLogin::deserialize(buffer);
            let end = SmaEndToken::deserialize(buffer);
            packet = Box::new(SmaResponseLogin
            {
                pkt_header: pkt_header,
                data_header: data_header,
                cmd: cmd_word,
                payload: payload,
                end: end
            });
        }
        SmaCmdGetDayData::OPCODE =>
        {
            if buffer.remaining() < SmaPayloadGetDayData::MIN_LENGTH
            {
                return Err("💩️ Received packet is too small".to_string());
            }
            let payload = SmaPayloadGetDayData::deserialize(buffer,
                pkt_header.data_len);
            let end = SmaEndToken::deserialize(buffer);
            // TODO: use macro to de-duplicate this
            packet = Box::new(SmaResponseGetDayData
            {
                pkt_header: pkt_header,
                data_header: data_header,
                cmd: cmd_word,
                payload: payload,
                end: end
            });
        }
        _ => return Err("💩️ Unsupported SMA packet received".to_string())
    }
    match packet.validate()
    {
        Err(e) => return Err(format!("💩️ Packet validation failed {}", e)),
        Ok(_) => ()
    };
    return Ok(packet);
}

#[cfg(test)]
mod tests;
