use super::sml_buffer::*;

#[derive(Debug)]
pub enum SmlTime
{
    SecIndex(u32),
    Timestamp(u32)
}

impl SmlTime
{
    pub fn deserialize(mut buffer: &mut SmlBuf)
        -> Result<Option<SmlTime>, String>
    {
        let tl = buffer.get_sml_tl(); // TODO: validate this
        if tl == 0x01
        {
            return Ok(None);
        }
        return match buffer.get_sml_u8()?
        {
            1 => Ok(Some(SmlTime::SecIndex(buffer.get_sml_u32()?))),
            2 => Ok(Some(SmlTime::Timestamp(buffer.get_sml_u32()?))),
            _ => Err("Invalid SmlTime variant found".to_string())
        }
    }
}

#[derive(Debug)]
pub enum SmlStatus
{
    Status8(u8),
    Status16(u16),
    Status32(u32),
    Status64(u64)
}

#[derive(Debug)]
pub enum SmlValue
{
    Boolean(bool),
    OctetString(Vec<u8>),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64)
}
