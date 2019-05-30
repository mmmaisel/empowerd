use super::sml_open::*;
use super::sml_close::*;
use super::sml_get_list::*;

use bytes::{Buf, BufMut};

pub enum SmlBody
{
    OpenResponse(SmlOpenResponse),
    CloseResponse(SmlCloseResponse),
    GetListResponse(SmlGetListResponse),
    SmlNone // TODO: remove this
}

impl SmlBody
{
    const OPEN_RESPONSE_ID: u16 = 0x0101;
    const CLOSE_RESPONSE_ID: u16 = 0x0201;
    const GET_LIST_RESPONSE_ID: u16 = 0x0701;

    pub fn deserialize(mut buffer: &mut Buf) -> SmlBody
    {
        let tl = buffer.get_u8();
        let id_tl = buffer.get_u8();
        let id = buffer.get_u16_le();

        return match id
        {
            SmlBody::OPEN_RESPONSE_ID => SmlBody::OpenResponse(
                    SmlOpenResponse::deserialize(&mut buffer)),
            SmlBody::CLOSE_RESPONSE_ID => SmlBody::CloseResponse(
                SmlCloseResponse::deserialize(&mut buffer)),
            SmlBody::GET_LIST_RESPONSE_ID => SmlBody::GetListResponse(
                SmlGetListResponse::deserialize(&mut buffer)),
            _ => panic!("unsupported body type")
        }
    }
}

pub struct SmlMessage
{
    type_len: u8, // TODO: remove this
    transaction_id: Vec<u8>,
    group_no: u8,
    abort_on_error: u8,
    body: SmlBody,
    crc: u16,
    end: u8
}

impl SmlMessage
{
    // TODO: return Result<SmlMessage, String>
    // TODO: dont panic, validate all lengths, best already in SmlFile
    pub fn deserialize(mut buffer: &mut Buf) -> SmlMessage
    {
        let tl = buffer.get_u8();
        let tid_tl = buffer.get_u8();
        // TODO: abort if tid_tl not an Vec<u8>
        let mut transaction_id: Vec<u8> = Vec::new();
        let mut tid_buf = buffer.take(tid_tl as usize);
        transaction_id.put(&mut tid_buf);
        buffer = tid_buf.into_inner();

        // TODO: check all tl fields
        let group_no_tl = buffer.get_u8(); // TODO: validate
        let group_no = buffer.get_u8();

        let abort_on_error_tl = buffer.get_u8();
        let abort_on_error = buffer.get_u8();

        let body = SmlBody::deserialize(&mut buffer);

        let crc16_tl = buffer.get_u8();
        let crc16 = buffer.get_u16_le();
        let end = buffer.get_u8();

        return SmlMessage
        {
            type_len: tl,
            transaction_id: transaction_id,
            group_no: group_no,
            abort_on_error: abort_on_error,
            body: body,
            crc: crc16,
            end: end
        };
    }

    pub fn validate(&self) -> Result<(), String>
    {
        return Err("not implemented".to_string());
    }
}
