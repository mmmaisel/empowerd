use super::sml_message::*;
use super::sml_stream::*;

// TODO: use byteorder, get rid of bytes, also in SMA
use bytes::Buf;

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
    pub fn deserialize_streams(streams: Vec<SmlStream>)
        -> Result<Vec<SmlFile>, String>
    {
        let mut parsed_files: Vec<SmlFile> = Vec::new();
        for stream in streams.into_iter()
        {
            parsed_files.push(SmlFile::deserialize(stream)?);
        }
        return Ok(parsed_files);
    }

    pub fn deserialize(stream: SmlStream)
        -> Result<SmlFile, String>
    {
        // TODO: validate len here
        let padding = ((stream.footer & 0x00FF0000) >> 16) as usize;

        let mut messages: Vec<SmlMessage> = Vec::new();
        let mut buffer = std::io::Cursor::new(stream.data);
        // TODO: calc crc first, before deserializing
        while buffer.remaining() > padding
        {
            messages.push(SmlMessage::deserialize(&mut buffer)?);
        }

        return Ok(SmlFile
        {
            version: stream.header,
            messages: messages,
            end_crc: stream.footer
        });
    }

    fn calc_crc(&self) -> u16
    {
        return 0;
    }
}
