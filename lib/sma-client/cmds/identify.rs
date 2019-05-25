extern crate bytes;
use bytes::{BytesMut, Buf, BufMut};

use super::*;

pub struct SmaCmdIdentify
{
    pub pkt_header: SmaPacketHeader,
    pub data_header: SmaDataHeader,
    pub cmd: SmaCmdWord,
    pub _padding: [u8; 8],
    pub end: SmaEndToken
}

impl SmaCmd for SmaCmdIdentify
{
    fn serialize(&self, buffer: &mut BytesMut)
    {
        self.pkt_header.serialize(buffer);
        self.data_header.serialize(buffer);
        self.cmd.serialize(buffer);
        buffer.put_slice(&self._padding);
        self.end.serialize(buffer);
    }

    fn opcode(&self) -> u32
    {
        return self.cmd.opcode();
    }
}

impl SmaCmdIdentify
{
    pub const OPCODE: u32 = 0x000002;
    pub const LENGTH: u16 = 12;

    pub fn new() -> SmaCmdIdentify
    {
        let mut retval = SmaCmdIdentify
        {
            pkt_header: SmaPacketHeader::new(SmaDataHeader::LENGTH +
                SmaCmdIdentify::LENGTH),
            data_header: SmaDataHeader::new(),
            cmd: SmaCmdWord::new(0, SmaCmdIdentify::OPCODE),
            _padding: [0; 8],
            end: SmaEndToken::new()
        };
        retval.data_header.infer_wordcount(retval.pkt_header.data_len);
        retval.data_header.class = SmaDataHeader::CMD_CLASS_A0;
        return retval;
    }
}

pub struct SmaPayloadIdentify
{
    _unknown: [u8; 48]
}

impl SmaPayloadIdentify
{
    pub const LENGTH: usize = 48;

    pub fn deserialize(buffer: &mut Buf) -> SmaPayloadIdentify
    {
        let mut unknown: [u8; 48] = [0; 48];
        buffer.copy_to_slice(&mut unknown);

        return SmaPayloadIdentify
        {
            _unknown: unknown
        };
    }

    pub fn validate(&self) -> Result<(), String>
    {
        // TODO
        return Ok(());
    }
}

pub struct SmaResponseIdentify
{
    pub pkt_header: SmaPacketHeader,
    pub data_header: SmaDataHeader,
    pub cmd: SmaCmdWord,
    pub payload: SmaPayloadIdentify,
    pub end: SmaEndToken
}

impl SmaResponse for SmaResponseIdentify
{
    fn extract_data(&self) -> SmaData
    {
        return SmaData::Endpoint(SmaEndpoint
        {
            susy_id: self.data_header.app.susy_id,
            serial: self.data_header.app.serial,
            ctrl: self.data_header.app.ctrl
        });
    }

    fn validate(&self) -> Result<(), String>
    {
        self.pkt_header.validate()?;
        if self.pkt_header.data_len != SmaResponseIdentify::LENGTH
        {
            return Err("SmaResponseIdentify has invalid length".to_string());
        }
        self.data_header.validate()?;
        self.cmd.validate()?;
        self.payload.validate()?;
        self.end.validate()?;
        return Ok(());
    }

    fn fragment_id(&self) -> u16
    {
        return self.data_header.fragment_id;
    }

    fn packet_id(&self) -> u16
    {
        return self.data_header.packet_id;
    }

    fn opcode(&self) -> u32
    {
        return self.cmd.opcode();
    }
}

impl SmaResponseIdentify
{
    pub const LENGTH: u16 = 0x004E;
}
