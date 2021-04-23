use super::sml_buffer::*;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlCloseResponse
{
    pub signature: Option<Vec<u8>>
}

impl SmlCloseResponse
{
    pub fn deserialize(buffer: &mut dyn SmlBuf)
        -> Result<SmlCloseResponse, String>
    {
        let tl = buffer.get_sml_tl();
        match tl
        {
            SmlType::Struct(len) =>
            {
                if len != 1
                {
                    return Err(format!(
                        "Invalid length {} for SmlCloseResponse", len));
                }
            }
            _ => return Err(format!(
                "Found {:X?}, expected struct", tl))
        }

        let signature = buffer.get_sml_octet_str()?;
        return Ok(SmlCloseResponse
        {
            signature: signature
        });
    }
}
