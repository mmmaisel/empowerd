use super::sml_buffer::*;
use super::sml_types::*;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlOpenResponse
{
    pub codepage: Option<Vec<u8>>,
    pub client_id: Option<Vec<u8>>,
    pub req_file_id: Vec<u8>,
    pub server_id: Vec<u8>,
    pub ref_time: Option<SmlTime>,
    pub version: Option<u8>
}

impl SmlOpenResponse
{
    pub fn deserialize(mut buffer: &mut dyn SmlBuf)
        -> Result<SmlOpenResponse, String>
    {
        let tl = buffer.get_sml_tl();
        match tl
        {
            SmlType::Struct(len) =>
            {
                if len != 6
                {
                    return Err(format!(
                        "Invalid length {} for SmlOpenResponse", len));
                }
            }
            _ => return Err(format!(
                "Found {:X?}, expected struct", tl))
        }

        let codepage = buffer.get_sml_octet_str()?;
        let client_id = buffer.get_sml_octet_str()?;

        let req_file_id = match buffer.get_sml_octet_str()?
        {
            Some(x) => x,
            None => return Err(
                "OpenResponse req_file_id is required".to_string())
        };
        let server_id = match buffer.get_sml_octet_str()?
        {
            Some(x) => x,
            None => return Err(
                "OpenResponse server_id is required".to_string())
        };

        let ref_time = SmlTime::deserialize(&mut buffer)?;
        let version = buffer.get_sml_u8()?;

        return Ok(SmlOpenResponse
        {
            codepage: codepage,
            client_id: client_id,
            req_file_id: req_file_id,
            server_id: server_id,
            ref_time: ref_time,
            version: version
        });
    }
}
