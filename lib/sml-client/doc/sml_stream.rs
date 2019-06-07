use super::sml_buffer::*;

extern crate byteorder;
use byteorder::{NativeEndian};

// TODO: use byteorder, get rid of bytes, also in SMA
use bytes::{Buf, ByteOrder};

pub struct SmlStream
{
    pub header: u32,
    pub data: Vec<u8>,
    pub footer: u32
}

impl SmlStream
{
    pub fn deserialize(buffer: &mut SmlBuf)
        -> Result<Vec<SmlStream>, String>
    {
        let mut streams: Vec<SmlStream> = Vec::new();

        while buffer.has_remaining()
        {
            let mut data: Vec<u8> = Vec::new();
            let start_escape = buffer.bytes().windows(4).position(|x|
            {
                NativeEndian::read_u32(&x) == 0x1b1b1b1b
            });
            match start_escape
            {
                Some(x) => buffer.advance(x),
                None => return Err("Did not found start token".to_string())
            }

            let header = buffer.get_sml_escape()?;
            let footer: u32;

            loop
            {
                let next_escape = buffer.bytes().windows(4).position(|x|
                {
                    NativeEndian::read_u32(&x) == 0x1b1b1b1b
                });
                match next_escape
                {
                    Some(x) => data.append(&mut buffer.get_vector(x)),
                    None => return Err(
                        "Did not found next escape token".to_string())
                }

                let escape = buffer.get_sml_escape()?;
                if escape & 0xFFF00000 == 0x1a000000
                {
                    footer = escape;
                    break;
                }
                else if escape == 0x1b1b1b1b
                {
                    data.push(0x1b);
                    data.push(0x1b);
                    data.push(0x1b);
                    data.push(0x1b);
                }
                else
                {
                    return Err(format!(
                        "Found unexpected escape value {}", escape));
                }
            }

            streams.push(SmlStream
            {
                header: header,
                data: data,
                footer: footer
            });
        }
        return Ok(streams);
    }
}
