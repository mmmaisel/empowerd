use super::sml_buffer::*;
use super::sml_open::*;
use super::sml_close::*;
use super::sml_get_list::*;

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

    pub fn deserialize(mut buffer: &mut SmlBuf) -> Result<SmlBody, String>
    {
        let tl = buffer.get_sml_tl();
        let id = buffer.get_sml_u16()?;

        return match id
        {
            SmlBody::OPEN_RESPONSE_ID => Ok(SmlBody::OpenResponse(
                    SmlOpenResponse::deserialize(&mut buffer)?)),
            SmlBody::CLOSE_RESPONSE_ID => Ok(SmlBody::CloseResponse(
                SmlCloseResponse::deserialize(&mut buffer)?)),
            SmlBody::GET_LIST_RESPONSE_ID => Ok(SmlBody::GetListResponse(
                SmlGetListResponse::deserialize(&mut buffer)?)),
            _ => Err("unsupported body type".to_string())
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
    crc: u16
}

impl SmlMessage
{
    // TODO: return Result<SmlMessage, String>
    // TODO: dont panic, validate all lengths, best already in SmlFile
    pub fn deserialize(mut buffer: &mut SmlBuf) -> Result<SmlMessage, String>
    {
        let tl = buffer.get_sml_tl();
        let transaction_id = buffer.get_sml_octet_str()?;
        let group_no = buffer.get_sml_u8()?;
        let abort_on_error = buffer.get_sml_u8()?;

        let body = SmlBody::deserialize(&mut buffer)?;

        let crc16 = buffer.get_sml_u16()?;
        buffer.get_sml_end()?;

        return Ok(SmlMessage
        {
            type_len: tl,
            transaction_id: transaction_id,
            group_no: group_no,
            abort_on_error: abort_on_error,
            body: body,
            crc: crc16
        });
    }

    pub fn validate(&self) -> Result<(), String>
    {
        return Err("not implemented".to_string());
    }
}
