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
        // TODO: use None if TL == 0x01
        let codepage = buffer.get_sml_octet_str()?;
        let client_id = buffer.get_sml_octet_str()?;
        let req_file_id = buffer.get_sml_octet_str()?;
        let server_id = buffer.get_sml_octet_str()?;
        let ref_time = SmlTime::deserialize(&mut buffer)?;
//        let version = buffer.get_sml_u8()?; // TODO: handle none here
let version = 1u8;

        return Ok(SmlOpenResponse
        {
            codepage: Some(codepage),
            client_id: Some(client_id),
            req_file_id: req_file_id,
            server_id: server_id,
            ref_time: ref_time,
            version: Some(version)
        });
    }
}
