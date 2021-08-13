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

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlCloseResponse {
    pub signature: Option<Vec<u8>>,
}

impl SmlCloseResponse {
    pub fn deserialize(
        buffer: &mut dyn SmlBuf,
    ) -> Result<SmlCloseResponse, String> {
        let tl = buffer.get_sml_tl();
        match tl {
            SmlType::Struct(len) => {
                if len != 1 {
                    return Err(format!(
                        "Invalid length {} for SmlCloseResponse",
                        len
                    ));
                }
            }
            _ => return Err(format!("Found {:X?}, expected struct", tl)),
        }

        let signature = buffer.get_sml_octet_str()?;
        return Ok(SmlCloseResponse {
            signature: signature,
        });
    }
}
