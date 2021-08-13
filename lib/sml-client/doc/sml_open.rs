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
use super::sml_types::*;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlOpenResponse {
    pub codepage: Option<Vec<u8>>,
    pub client_id: Option<Vec<u8>>,
    pub req_file_id: Vec<u8>,
    pub server_id: Vec<u8>,
    pub ref_time: Option<SmlTime>,
    pub version: Option<u8>,
}

impl SmlOpenResponse {
    pub fn deserialize(
        mut buffer: &mut dyn SmlBuf,
    ) -> Result<SmlOpenResponse, String> {
        let tl = buffer.get_sml_tl();
        match tl {
            SmlType::Struct(len) => {
                if len != 6 {
                    return Err(format!(
                        "Invalid length {} for SmlOpenResponse",
                        len
                    ));
                }
            }
            _ => return Err(format!("Found {:X?}, expected struct", tl)),
        }

        let codepage = buffer.get_sml_octet_str()?;
        let client_id = buffer.get_sml_octet_str()?;

        let req_file_id = match buffer.get_sml_octet_str()? {
            Some(x) => x,
            None => {
                return Err("OpenResponse req_file_id is required".to_string())
            }
        };
        let server_id = match buffer.get_sml_octet_str()? {
            Some(x) => x,
            None => {
                return Err("OpenResponse server_id is required".to_string())
            }
        };

        let ref_time = SmlTime::deserialize(&mut buffer)?;
        let version = buffer.get_sml_u8()?;

        return Ok(SmlOpenResponse {
            codepage: codepage,
            client_id: client_id,
            req_file_id: req_file_id,
            server_id: server_id,
            ref_time: ref_time,
            version: version,
        });
    }
}
