extern crate bytes;
extern crate crc16;
extern crate serialport;

use std::io::Cursor;

mod doc;
use doc::SmlFile;
use doc::SmlBuf;

pub struct SmlClient
{
}

impl SmlClient
{
    pub fn decode(data: Vec<u8>) -> Result<SmlFile, String>
    {
        let mut buf = Cursor::new(data);
        return SmlFile::deserialize(&mut buf);
    }
}
