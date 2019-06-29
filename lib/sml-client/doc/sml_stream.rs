use super::sml_buffer::*;

extern crate byteorder;
use byteorder::{NativeEndian};

// TODO: use byteorder, get rid of bytes, also in SMA
use bytes::{Buf, ByteOrder};
use crc16::*;

pub struct SmlStream
{
    pub version: u32,
    pub data: Vec<u8>,
    pub padding: usize,
    pub crc: u16
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
            let mut crc_state = State::<X_25>::new();
            let mut header: u32;
            let footer: u32;
            // TODO: extract method
            loop
            {
                let start_escape = buffer.bytes().windows(4).position(|x|
                {
                    NativeEndian::read_u32(&x) == 0x1b1b1b1b
                });
                if cfg!(debug_assertions)
                {
                    println!("found start of SML file at {:?}", start_escape);
                }
                match start_escape
                {
                    Some(x) => buffer.advance(x),
                    None => return Err("Did not found start token".to_string())
                }

                header = buffer.get_sml_escape()?;
                if header == 0x01010101
                {
                    crc_state.update(
                        &[0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01]);
                    break;
                }
            }

            // TODO: extract method
            loop
            {
                let next_escape = buffer.bytes().windows(4).position(|x|
                {
                    NativeEndian::read_u32(&x) == 0x1b1b1b1b
                });
                match next_escape
                {
                    Some(x) =>
                    {
                        let mut data_to_append = buffer.get_vector(x);
                        crc_state.update(&data_to_append);
                        data.append(&mut data_to_append);
                    }
                    None => return Err(
                        "Did not found next escape token".to_string())
                }

                let escape = buffer.get_sml_escape()?;
                // TODO: this code sucks
                crc_state.update(&[0x1b, 0x1b, 0x1b, 0x1b]);
                crc_state.update(&[((escape & 0xFF000000) >> 24) as u8]);
                crc_state.update(&[((escape & 0x00FF0000) >> 16) as u8]);
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
                        "Found unexpected escape value {:X?}", escape));
                }
                // TODO: this sucks
                crc_state.update(&[((escape & 0x0000FF00) >> 8) as u8]);
                crc_state.update(&[( escape & 0x000000FF) as u8]);
            }

            let footer_crc = (footer & 0x0000FFFF) as u16;
            let calculated_crc = crc_state.get().swap_bytes();
            if calculated_crc != footer_crc
            {
                return Err(format!(
                    "Found invalid stream checksum {:X}, expected {:X}",
                    footer_crc, calculated_crc));
            }

            streams.push(SmlStream
            {
                version: header,
                data: data,
                padding: ((footer & 0x00FF0000) >> 16) as usize,
                crc: footer_crc
            });
        }
        return Ok(streams);
    }
}