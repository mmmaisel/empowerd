use super::sml_types::*;
use super::sml_buffer::*;
use super::sml_message::*;

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

    pub fn validate(&self) -> Result<(), String>
    {
        return Err("not implemented".to_string());
    }
}
