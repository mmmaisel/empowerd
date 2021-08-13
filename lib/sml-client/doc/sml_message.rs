/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
use super::sml_buffer::*;
use super::sml_close::*;
use super::sml_get_list::*;
use super::sml_open::*;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum SmlBody {
    OpenResponse(SmlOpenResponse),
    CloseResponse(SmlCloseResponse),
    GetListResponse(SmlGetListResponse),
    SmlNone, // TODO: remove this
}

impl SmlBody {
    const OPEN_RESPONSE_ID: u16 = 0x0101;
    const CLOSE_RESPONSE_ID: u16 = 0x0201;
    const GET_LIST_RESPONSE_ID: u16 = 0x0701;

    pub fn deserialize(mut buffer: &mut dyn SmlBuf) -> Result<SmlBody, String> {
        let tl = buffer.get_sml_tl();
        match tl {
            SmlType::Struct(len) => {
                if len != 2 {
                    return Err(format!("Invalid length {} for SmlBody", len));
                }
            }
            _ => return Err(format!("Found {:X?}, expected struct", tl)),
        }

        let id = match buffer.get_sml_u16()? {
            Some(x) => x,
            None => return Err("SmlBody type is required".to_string()),
        };

        return match id {
            SmlBody::OPEN_RESPONSE_ID => Ok(SmlBody::OpenResponse(
                SmlOpenResponse::deserialize(&mut buffer)?,
            )),
            SmlBody::CLOSE_RESPONSE_ID => Ok(SmlBody::CloseResponse(
                SmlCloseResponse::deserialize(&mut buffer)?,
            )),
            SmlBody::GET_LIST_RESPONSE_ID => Ok(SmlBody::GetListResponse(
                SmlGetListResponse::deserialize(&mut buffer)?,
            )),
            _ => Err("unsupported body type".to_string()),
        };
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlMessage {
    pub transaction_id: Vec<u8>,
    pub group_no: u8,
    pub abort_on_error: u8,
    pub body: SmlBody,
    pub crc: u16,
}

impl SmlMessage {
    // TODO: dont panic, validate all lengths, best already in SmlFile
    pub fn deserialize(
        mut buffer: &mut dyn SmlBuf,
    ) -> Result<SmlMessage, String> {
        let tl = buffer.get_sml_tl();
        match tl {
            SmlType::Struct(len) => {
                if len != 6 {
                    return Err(format!(
                        "Invalid length {} for SmlMessage",
                        len
                    ));
                }
            }
            _ => return Err(format!("Found {:X?}, expected struct", tl)),
        }

        let transaction_id = match buffer.get_sml_octet_str()? {
            Some(x) => x,
            None => {
                return Err("SmlMessage transaction_id is required".to_string())
            }
        };
        let group_no = match buffer.get_sml_u8()? {
            Some(x) => x,
            None => return Err("SmlMessage group_id is required".to_string()),
        };
        let abort_on_error = match buffer.get_sml_u8()? {
            Some(x) => x,
            None => {
                return Err("SmlMessage abort_on_error is required".to_string())
            }
        };

        let body = SmlBody::deserialize(&mut buffer)?;

        let crc16 = match buffer.get_sml_u16()? {
            Some(x) => x,
            None => return Err("SmlMessage crc16 is required".to_string()),
        };
        buffer.get_sml_end()?;

        return Ok(SmlMessage {
            transaction_id: transaction_id,
            group_no: group_no,
            abort_on_error: abort_on_error,
            body: body,
            crc: crc16,
        });
    }
}
