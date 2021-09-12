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
use bytes::Buf;

#[derive(Debug)]
pub enum SmlType {
    End,
    None,
    OctetString(usize),
    Boolean,
    Int(usize),
    UInt(usize),
    Struct(usize),
    Invalid(u8),
}

// TODO: add allow none parameter to all those functions
pub trait SmlBuf: Buf {
    fn get_sml_u8(&mut self) -> Result<Option<u8>, String> {
        let result = self.get_sml_uint(1)?;
        return match result {
            Some(x) => Ok(Some(x as u8)),
            None => Ok(None),
        };
    }

    fn get_sml_u16(&mut self) -> Result<Option<u16>, String> {
        let result = self.get_sml_uint(2)?;
        return match result {
            Some(x) => Ok(Some(x as u16)),
            None => Ok(None),
        };
    }

    fn get_sml_u32(&mut self) -> Result<Option<u32>, String> {
        let result = self.get_sml_uint(4)?;
        return match result {
            Some(x) => Ok(Some(x as u32)),
            None => Ok(None),
        };
    }

    fn get_sml_u64(&mut self) -> Result<Option<u64>, String> {
        return self.get_sml_uint(8);
    }

    fn get_sml_uint(&mut self, len: usize) -> Result<Option<u64>, String> {
        let tl = self.get_sml_tl();
        return match tl {
            SmlType::None => Ok(None),
            SmlType::UInt(len) => Ok(Some(self.get_uint(len))),
            _ => Err(format!(
                "get_sml_unit: Found {:X?}, expected u{}",
                tl,
                len * 8
            )),
        };
    }

    fn get_sml_i8(&mut self) -> Result<Option<i8>, String> {
        let result = self.get_sml_int(1)?;
        return match result {
            Some(x) => Ok(Some(x as i8)),
            None => Ok(None),
        };
    }

    fn get_sml_i16(&mut self) -> Result<Option<i16>, String> {
        let result = self.get_sml_int(2)?;
        return match result {
            Some(x) => Ok(Some(x as i16)),
            None => Ok(None),
        };
    }

    fn get_sml_i32(&mut self) -> Result<Option<i32>, String> {
        let result = self.get_sml_int(4)?;
        return match result {
            Some(x) => Ok(Some(x as i32)),
            None => Ok(None),
        };
    }

    fn get_sml_i64(&mut self) -> Result<Option<i64>, String> {
        return self.get_sml_int(8);
    }

    fn get_sml_int(&mut self, len: usize) -> Result<Option<i64>, String> {
        let tl = self.get_sml_tl();
        return match tl {
            SmlType::None => Ok(None),
            SmlType::Int(len) => Ok(Some(self.get_int(len))),
            _ => Err(format!(
                "get_sml_int: Found {:X?}, expected i{}",
                tl,
                len * 8
            )),
        };
    }

    fn get_sml_bool(&mut self) -> Result<Option<bool>, String> {
        let tl = self.get_sml_tl();
        return match tl {
            SmlType::None => Ok(None),
            SmlType::Boolean => Ok(Some(self.get_u8() != 0)),
            _ => Err(format!("get_sml_bool: Found {:X?}, expected bool", tl)),
        };
    }

    fn get_sml_tl(&mut self) -> SmlType {
        let tl = self.get_u8();
        match tl {
            0x00 => return SmlType::End,
            0x01 => return SmlType::None,
            0x10 | 0x20 | 0x30 => return SmlType::Invalid(tl),
            _ => (),
        }

        let mut len = (tl & 0x0F) as usize;
        let mut tl2 = tl;
        let mut tl_bcount = 1;
        while tl2 & 0x80 != 0 {
            tl2 = self.get_u8();
            if tl2 & 0x70 != 0 {
                return SmlType::Invalid(tl2);
            }
            len = (len << 4) | ((tl2 & 0x0F) as usize);
            tl_bcount += 1;
        }

        return match tl & 0x70 {
            0x00 => SmlType::OctetString(len - tl_bcount),
            0x40 => SmlType::Boolean,
            0x50 => SmlType::Int(len - tl_bcount),
            0x60 => SmlType::UInt(len - tl_bcount),
            0x70 => SmlType::Struct(len),
            _ => SmlType::Invalid(tl),
        };
    }

    fn get_vector(&mut self, len: usize) -> Vec<u8> {
        let mut oct_str: Vec<u8> = Vec::new();
        for _ in 0..len {
            // TODO: try again with take, no loop
            oct_str.push(self.get_u8());
        }
        return oct_str;
    }

    fn get_sml_octet_str(&mut self) -> Result<Option<Vec<u8>>, String> {
        let tl = self.get_sml_tl();
        return match tl {
            SmlType::None => Ok(None),
            SmlType::OctetString(len) => Ok(Some(self.get_vector(len))),
            _ => Err(format!(
                "get_sml_octet_str: Found {:X?}, expected octet string",
                tl
            )),
        };
    }

    // TODO: add get implicit choice value here
    // TODO: add get implicit choice status here

    fn get_sml_escape(&mut self) -> Result<u32, String> {
        if self.remaining() < 8 {
            return Err("Less than 8 chars remaining".to_string());
        }

        let escape = self.get_u32();
        if escape != 0x1b1b1b1b {
            return Err("No escape sequence found".to_string());
        }
        return Ok(self.get_u32());
    }

    fn get_sml_end(&mut self) -> Result<(), String> {
        let tl = self.get_sml_tl();
        return match tl {
            SmlType::End => Ok(()),
            _ => Err(format!("get_sml_end: Found {:X?}, expected end", tl)),
        };
    }

    fn skip_get_sml_escape(&mut self) -> Result<u32, String> {
        let mut escape_count = 0;
        while self.has_remaining() {
            if self.get_u8() == 0x1b {
                escape_count += 1;
            } else {
                escape_count = 0;
            }

            if escape_count == 4 {
                if self.remaining() < 4 {
                    return Err("Found incomplete escape sequence".into());
                }
                return Ok(self.get_u32());
            }
        }
        return Err("No escape sequence found in buffer".into());
    }

    #[allow(clippy::same_item_push)]
    fn copy_get_sml_escape(&mut self) -> Result<(Vec<u8>, u32), String> {
        let mut oct_str: Vec<u8> = Vec::new();
        let mut escape_count = 0;
        while self.has_remaining() {
            let val = self.get_u8();
            if val == 0x1b {
                escape_count += 1;
            } else {
                for _ in 0..escape_count {
                    oct_str.push(0x1b);
                }
                escape_count = 0;
                oct_str.push(val);
            }

            if escape_count == 4 {
                if self.remaining() < 4 {
                    return Err("Found incomplete escape sequence".into());
                }
                return Ok((oct_str, self.get_u32()));
            }
        }
        return Err("No escape sequence found in buffer".into());
    }
}

impl<'a, T: SmlBuf + ?Sized> SmlBuf for &'a mut T {}
impl<T: AsRef<[u8]>> SmlBuf for std::io::Cursor<T> {}
