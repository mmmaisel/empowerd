use super::sml_buffer::*;
use super::sml_types::*;

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
    pub fn deserialize(buffer: &mut SmlBuf) -> Result<SmlOpenResponse, String>
    {
        return Err("not implemented yet".to_string());
        // TODO: implement
/*        return SmlOpenResponse
        {
            codepage: None,
            client_id: None,
            req_file_id: Vec::new(),
            server_id: Vec::new(),
            ref_time: None,
            version: None
        };*/
    }
}
