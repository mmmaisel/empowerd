use super::sml_buffer::*;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
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
        else if tl != 0x72
        {
            return Err(format!("Invalid TL value {:X} for SmlTime", tl));
        }

        let ty = match buffer.get_sml_u8()?
        {
            Some(x) => x,
            None => return Err("SmlTime type is required".to_string())
        };

        return match ty
        {
            1 =>
            {
                let value = buffer.get_sml_u32()?;
                return match value
                {
                    Some(x) => Ok(Some(SmlTime::SecIndex(x))),
                    None => Err(
                        "SmlTime secIndex is required".to_string())
                };
            }
            2 =>
            {
                let value = buffer.get_sml_u32()?;
                return match value
                {
                    Some(x) => Ok(Some(SmlTime::Timestamp(x))),
                    None => Err(
                        "SmlTime timestamp is required".to_string())
                };
            }
            _ => Err("Invalid SmlTime variant found".to_string())
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum SmlStatus
{
    Status8(u8),
    Status16(u16),
    Status32(u32),
    Status64(u64)
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
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
