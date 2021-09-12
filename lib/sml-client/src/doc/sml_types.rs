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
pub enum SmlTime {
    Invalid(u32),
    SecIndex(u32),
    Timestamp(u32),
}

impl SmlTime {
    pub fn deserialize(
        buffer: &mut dyn SmlBuf,
    ) -> Result<Option<SmlTime>, String> {
        let tl = buffer.get_sml_tl();
        let ty = match tl {
            SmlType::None => return Ok(None),
            SmlType::Struct(len) => {
                if len != 2 {
                    return Err(format!("Invalid length {} for SmlTime", len));
                }
                match buffer.get_sml_u8()? {
                    Some(x) => x,
                    None => return Err("SmlTime type is required".to_string()),
                }
            }
            SmlType::UInt(len) => {
                if len != 4 {
                    return Err(format!("Invalid length {} for Unit(4)", len));
                }
                0
            }
            _ => {
                return Err(format!(
                    "SmlTime::deserialize: Found {:X?}, expected struct",
                    tl
                ))
            }
        };

        return match ty {
            0 => {
                let value = buffer.get_u32();
                return Ok(Some(SmlTime::Invalid(value)));
            }
            1 => {
                let value = buffer.get_sml_u32()?;
                return match value {
                    Some(x) => Ok(Some(SmlTime::SecIndex(x))),
                    None => Err("SmlTime secIndex is required".to_string()),
                };
            }
            2 => {
                let value = buffer.get_sml_u32()?;
                return match value {
                    Some(x) => Ok(Some(SmlTime::Timestamp(x))),
                    None => Err("SmlTime timestamp is required".to_string()),
                };
            }
            _ => Err("Invalid SmlTime variant found".to_string()),
        };
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlStatus {
    pub status: u64,
}

impl SmlStatus {
    pub fn deserialize(
        buffer: &mut dyn SmlBuf,
    ) -> Result<Option<SmlStatus>, String> {
        let tl = buffer.get_sml_tl();
        return match tl {
            SmlType::None => Ok(None),
            SmlType::UInt(len) => Ok(Some(SmlStatus {
                status: buffer.get_uint(len),
            })),
            _ => Err(format!(
                "SmlStatus::deserialize: Found {:X?} expected uint",
                tl
            )),
        };
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum SmlValue {
    Boolean(bool),
    OctetString(Vec<u8>),
    Int(i64),
    UInt(u64),
}

impl SmlValue {
    pub fn deserialize(buffer: &mut dyn SmlBuf) -> Result<SmlValue, String> {
        let tl = buffer.get_sml_tl();
        return match tl {
            SmlType::OctetString(len) => {
                Ok(SmlValue::OctetString(buffer.get_vector(len)))
            }
            SmlType::Boolean => Ok(SmlValue::Boolean(buffer.get_u8() != 0)),
            SmlType::Int(len) => Ok(SmlValue::Int(buffer.get_int(len))),
            SmlType::UInt(len) => Ok(SmlValue::UInt(buffer.get_uint(len))),
            _ => Err(format!("Found {:X?} which is not an SmlValue", tl)),
        };
    }
}
