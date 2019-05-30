use super::sml_types::*;
use super::sml_message::*;

use bytes::{Buf};

struct SmlFile
{
    header: u32,
    version: u32,
    messages: Vec<SmlMessage>,
    footer: u32,
    end_crc: u32
}

impl SmlFile
{
    pub fn deserialize(buffer: &mut Buf) -> SmlFile
    {
        // TODO: validate len here
        let header = buffer.get_u32_le();
        let version = buffer.get_u32_le();

        // TODO: handle 1b1b1b1b bitstuffing
        // TODO: deserialize messages

        // TODO: validate len here
        let footer = buffer.get_u32_le();
        let end_crc = buffer.get_u32_le();

        return SmlFile
        {
            header: header,
            version: version,
            messages: Vec::new(),
            footer: footer,
            end_crc: end_crc
        };
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
