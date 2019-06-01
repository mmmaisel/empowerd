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
    pub fn deserialize(buffer: &mut SmlBuf)
        -> Result<Option<SmlTime>, String>
    {
        let tl = buffer.get_sml_tl();
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

impl SmlStatus
{
    pub fn deserialize(buffer: &mut SmlBuf)
        -> Result<Option<SmlStatus>, String>
    {
        let tl = buffer.get_sml_tl();
        return match tl
        {
            // TODO: this is ugly
            0x01 => Ok(None),
            0x62 => Ok(Some(SmlStatus::Status8(buffer.get_u8()))),
            0x63 => Ok(Some(SmlStatus::Status16(buffer.get_u16_be()))),
            0x64 => Ok(Some(SmlStatus::Status32(buffer.get_u32_be()))),
            0x65 => Ok(Some(SmlStatus::Status64(buffer.get_u64_be()))),
            _ => Err(format!("Invalid TL value {:X} for SmlStatus", tl))
        };
    }
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

impl SmlValue
{
    pub fn deserialize(buffer: &mut SmlBuf)
        -> Result<SmlValue, String>
    {
        let tl = buffer.get_sml_tl();
        if tl >= 2 && tl <= 15
        {
            return Ok(SmlValue::OctetString(buffer.
                get_vector(tl as usize)));
        }

        return match tl
        {
            // TODO: this is ugly
            0x42 => Ok(SmlValue::Boolean(buffer.get_u8() != 0)),
            0x52 => Ok(SmlValue::Int8(buffer.get_i8())),
            0x53 => Ok(SmlValue::Int16(buffer.get_i16_be())),
            0x54 => Ok(SmlValue::Int32(buffer.get_i32_be())),
            0x55 => Ok(SmlValue::Int64(buffer.get_i64_be())),
            0x62 => Ok(SmlValue::UInt8(buffer.get_u8())),
            0x63 => Ok(SmlValue::UInt16(buffer.get_u16_be())),
            0x64 => Ok(SmlValue::UInt32(buffer.get_u32_be())),
            0x65 => Ok(SmlValue::UInt64(buffer.get_u64_be())),
            _ => Err(format!("Invalid TL value {:X} for SmlValue", tl))
        };
    }
}
