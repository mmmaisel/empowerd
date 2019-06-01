use super::sml_buffer::*;

#[derive(Debug)]
pub struct SmlCloseResponse
{
    signature: Option<Vec<u8>>
}

impl SmlCloseResponse
{
    pub fn deserialize(buffer: &mut SmlBuf)
        -> Result<SmlCloseResponse, String>
    {
        let tl = buffer.get_sml_tl();
        if tl != 0x71
        {
            return Err(format!("Invalid TL value {:X} for CloseResponse", tl));
        }

        let signature = buffer.get_sml_octet_str()?;
        return Ok(SmlCloseResponse
        {
            signature: signature
        });
    }
}
