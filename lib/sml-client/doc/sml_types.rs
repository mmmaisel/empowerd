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
    pub fn deserialize(buffer: &mut dyn SmlBuf)
        -> Result<Option<SmlTime>, String>
    {
        let tl = buffer.get_sml_tl();
        match tl
        {
            SmlType::None => return Ok(None),
            SmlType::Struct(len) =>
            {
                if len != 2
                {
                    return Err(format!(
                        "Invalid length {} for SmlTime", len));
                }
            }
            _ => return Err(format!(
                "Found {:X?}, expected struct", tl))
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
pub struct SmlStatus
{
    status: u64
}

impl SmlStatus
{
    pub fn deserialize(buffer: &mut dyn SmlBuf)
        -> Result<Option<SmlStatus>, String>
    {
        let tl = buffer.get_sml_tl();
        return match tl
        {
            SmlType::None => Ok(None),
            SmlType::UInt(len) =>
                Ok(Some(SmlStatus { status: buffer.get_uint_be(len) } )),
            _ => Err(format!("Found {:X?} expected uint", tl))
        };
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum SmlValue
{
    Boolean(bool),
    OctetString(Vec<u8>),
    Int(i64),
    UInt(u64)
}

impl SmlValue
{
    pub fn deserialize(buffer: &mut dyn SmlBuf)
        -> Result<SmlValue, String>
    {
        let tl = buffer.get_sml_tl();
        return match tl
        {
            SmlType::OctetString(len) =>
                Ok(SmlValue::OctetString(buffer.get_vector(len))),
            SmlType::Boolean =>
                Ok(SmlValue::Boolean(buffer.get_u8() != 0)),
            SmlType::Int(len) =>
                Ok(SmlValue::Int(buffer.get_int_be(len))),
            SmlType::UInt(len) =>
                Ok(SmlValue::UInt(buffer.get_uint_be(len))),
            _ => Err(format!("Found {:X?} which is not an SmlValue", tl))
        };
    }
}
