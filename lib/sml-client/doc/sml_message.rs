use super::sml_open::*;
use super::sml_close::*;
use super::sml_get_list::*;

pub struct SmlMessage
{
    transaction_id: Vec<u8>,
    group_no: u8,
    abort_on_error: u8,
    body: SmlBody,
    crc: u16,
    end: u8
}

pub enum SmlBody
{
    OpenResponse(SmlOpenResponse),
    CloseResponse(SmlCloseResponse),
    GetListResponse(SmlGetListResponse)
}

