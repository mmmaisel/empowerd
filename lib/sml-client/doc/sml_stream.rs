use super::sml_buffer::*;

// TODO: use byteorder, get rid of bytes, also in SMA
use bytes::Buf;
use crc16::*;

pub struct SmlStream {
    pub version: u32,
    pub data: Vec<u8>,
    pub padding: usize,
    pub crc: u16,
}

impl SmlStream {
    pub fn deserialize(
        buffer: &mut dyn SmlBuf,
    ) -> Result<Vec<SmlStream>, String> {
        let mut streams: Vec<SmlStream> = Vec::new();

        while buffer.has_remaining() {
            let mut data: Vec<u8> = Vec::new();
            let mut crc_state = State::<X_25>::new();
            let mut header: u32;
            let footer: u32;

            loop {
                // Next escape might be incomplete.
                header = match buffer.skip_get_sml_escape() {
                    Ok(x) => x,
                    Err(_) => return Ok(streams),
                };
                if header == 0x01010101 {
                    crc_state.update(&[
                        0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01,
                    ]);
                    break;
                }
            }

            loop {
                let escape = match buffer.copy_get_sml_escape() {
                    Ok((mut data_to_append, escape)) => {
                        crc_state.update(&data_to_append);
                        data.append(&mut data_to_append);
                        escape
                    }
                    // If no next_escape is found then an incomplete
                    // file was read.
                    // TODO: handle incomplete files and write correct test for it
                    Err(_) => return Ok(streams),
                };

                // TODO: this code sucks
                crc_state.update(&[0x1b, 0x1b, 0x1b, 0x1b]);
                crc_state.update(&[((escape & 0xFF000000) >> 24) as u8]);
                crc_state.update(&[((escape & 0x00FF0000) >> 16) as u8]);
                if escape & 0xFFF00000 == 0x1a000000 {
                    footer = escape;
                    break;
                } else if escape == 0x1b1b1b1b {
                    data.push(0x1b);
                    data.push(0x1b);
                    data.push(0x1b);
                    data.push(0x1b);
                } else {
                    return Err(format!(
                        "Found unexpected escape value {:X?}",
                        escape
                    ));
                }
                // TODO: this sucks
                crc_state.update(&[((escape & 0x0000FF00) >> 8) as u8]);
                crc_state.update(&[(escape & 0x000000FF) as u8]);
            }

            let footer_crc = (footer & 0x0000FFFF) as u16;
            let calculated_crc = crc_state.get().swap_bytes();
            if calculated_crc != footer_crc {
                return Err(format!(
                    "Found invalid stream checksum {:X}, expected {:X}",
                    footer_crc, calculated_crc
                ));
            }

            streams.push(SmlStream {
                version: header,
                data: data,
                padding: ((footer & 0x00FF0000) >> 16) as usize,
                crc: footer_crc,
            });
        }
        return Ok(streams);
    }
}
