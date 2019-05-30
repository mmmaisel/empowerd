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

