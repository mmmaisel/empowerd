use super::sml_buffer::*;
use super::sml_types::*;

#[derive(Debug)]
pub struct SmlOpenResponse
{
    codepage: Option<Vec<u8>>,
    client_id: Option<Vec<u8>>,
    req_file_id: Vec<u8>,
    server_id: Vec<u8>,
    ref_time: Option<SmlTime>,
    version: Option<u8>
}

impl SmlOpenResponse
{
    pub fn deserialize(mut buffer: &mut SmlBuf)
        -> Result<SmlOpenResponse, String>
    {
        let tl = buffer.get_sml_tl();
        if tl != 0x76
        {
            return Err(
                format!("Invalid TL value {:X} for OpenResponse", tl));
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
