use super::sml_buffer::*;
use super::sml_message::*;

extern crate byteorder;
use byteorder::{LittleEndian};
use bytes::ByteOrder;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlFile
{
    pub version: u32,
    pub messages: Vec<SmlMessage>,
    pub end_crc: u32
}

impl SmlFile
{
    pub fn deserialize_stream(mut buffer: &mut SmlBuf)
        -> Result<Vec<SmlFile>, String>
    {
        let mut files: Vec<SmlFile> = Vec::new();
        while buffer.has_remaining()
        {
            let sync_pattern = LittleEndian::read_u32(&buffer.bytes()[0..4]);
            if sync_pattern == 0x1b1b1b1b
            {
                files.push(SmlFile::deserialize(&mut buffer)?);
            }
            else
            {
                buffer.advance(1);
            }
        }
        return Ok(files);
    }

    pub fn deserialize(mut buffer: &mut SmlBuf)
        -> Result<SmlFile, String>
    {
        // TODO: validate len here
        let version = buffer.get_sml_escape()?;

        // TODO: handle 1b1b1b1b bitstuffing
        // TODO: deserialize messages

        let mut messages: Vec<SmlMessage> = Vec::new();
        // TODO: stop condition does not work
        while buffer.remaining() > 8
        {
            messages.push(SmlMessage::deserialize(&mut buffer)?);
        }

        // TODO: skip padding

        // TODO: validate len here
        let end_crc = buffer.get_sml_escape()?;

        return Ok(SmlFile
        {
            version: version,
            messages: messages,
            end_crc: end_crc
        });
    }

    fn calc_crc(&self) -> u16
    {
        return 0;
    }
}
