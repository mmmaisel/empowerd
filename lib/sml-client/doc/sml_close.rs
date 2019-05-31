use super::sml_buffer::*;

pub struct SmlCloseResponse
{
    signature: Option<Vec<u8>>
}

impl SmlCloseResponse
{
    pub fn deserialize(buffer: &mut SmlBuf) -> Result<SmlCloseResponse, String>
    {
        /*return SmlCloseResponse
        {
            signature: None // TODO: implement this
        };*/
        return Err("Not implemented yet".to_string());
    }
}
