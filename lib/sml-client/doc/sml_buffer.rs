use bytes::Buf;

// TODO: add allow none parameter to all those functions
pub trait SmlBuf: Buf
{
    fn get_sml_u8(&mut self) -> Result<u8, String>
    {
        let result = self.get_sml_uint(1)?;
        return Ok(result as u8);
    }

    fn get_sml_u16(&mut self) -> Result<u16, String>
    {
        let result = self.get_sml_uint(2)?;
        return Ok(result as u16);
    }

    fn get_sml_u32(&mut self) -> Result<u32, String>
    {
        let result = self.get_sml_uint(4)?;
        return Ok(result as u32);
    }

    fn get_sml_u64(&mut self) -> Result<u64, String>
    {
        return self.get_sml_uint(8);
    }

    fn get_sml_uint(&mut self, len: usize) -> Result<u64, String>
    {
        let tl = self.get_sml_tl();
        if tl == (0x61 + (len as u8))
        {
            return Ok(self.get_uint_be(len));
        }
        else
        {
            return Err(format!("Invalid TL value {:X} for u{}", tl, len*8));
        };
    }

    fn get_sml_i8(&mut self) -> Result<i8, String>
    {
        let result = self.get_sml_int(1)?;
        return Ok(result as i8);
    }

    fn get_sml_i16(&mut self) -> Result<i16, String>
    {
        let result = self.get_sml_int(2)?;
        return Ok(result as i16);
    }

    fn get_sml_i32(&mut self) -> Result<i32, String>
    {
        let result = self.get_sml_int(4)?;
        return Ok(result as i32);
    }

    fn get_sml_i64(&mut self) -> Result<i64, String>
    {
        return self.get_sml_int(8);
    }

    fn get_sml_int(&mut self, len: usize) -> Result<i64, String>
    {
        let tl = self.get_sml_tl();
        if tl == (0x51 + (len as u8))
        {
            return Ok(self.get_int_be(len));
        }
        else
        {
            return Err(format!("Invalid TL value {:X} for i{}", tl, len*8));
        };
    }

    fn get_sml_bool(&mut self) -> Result<bool, String>
    {
        let tl = self.get_sml_tl();
        if tl == 0x42
        {
            return Ok(self.get_u8() != 0);
        }
        else
        {
            return Err(format!("Invalid TL value {:X} for bool", tl));
        };
    }

    fn get_sml_tl(&mut self) -> u8
    {
        // TODO: handle multi-byte tl values, return (type, length)
        return self.get_u8();
    }

    fn get_sml_octet_str(&mut self) -> Result<Vec<u8>, String>
    {
        let tl = self.get_sml_tl();
        if tl < 1 || tl > 15
        {
            return Err(format!("Invalid TL value {:X} for octet string", tl));
        }

        let mut oct_str: Vec<u8> = Vec::new();
        for _ in 1..tl
        {
            // TODO: try again with take, no loop
            oct_str.push(self.get_u8());
        }
        return Ok(oct_str);
    }

    fn get_sml_escape(&mut self) -> Result<u32, String>
    {
        let escape = self.get_u32_be();
        if escape != 0x1b1b1b1b
        {
            return Err("No escape sequence found".to_string());
        }
        return Ok(self.get_u32_be());
    }

    fn get_sml_end(&mut self) -> Result<(), String>
    {
        let tl = self.get_sml_tl();
        if tl == 0
        {
            return Ok(());
        }
        else
        {
            return Err(format!("Invalid TL value {:X} found for end", tl));
        }
    }
}

impl<'a, T: SmlBuf + ?Sized> SmlBuf for &'a mut T {}
impl<T: AsRef<[u8]>> SmlBuf for std::io::Cursor<T> {}
