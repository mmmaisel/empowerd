use std::fmt;
use std::str::Utf8Error;

pub struct Message
{
    pub msgtype: u8,
    pub fragment: u8,
    pub length: usize,
    pub content: [u8; 54],
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Message")
            .field("msgtype", &self.msgtype)
            .field("fragment", &self.fragment)
            .field("length", &self.length)
            .field("data", &String::from_utf8_lossy(&self.content[0..54]))
            .finish()
    }
}

impl Message {
    pub fn to_string(&self) -> Result<&str, Utf8Error> {
        return std::str::from_utf8(&self.content[0..self.length]);
    }
}
