use bytes::Buf;

pub enum WH1080MessageType
{
    Dummy
}

pub trait WH1080Buf: Buf
{

    fn get_message_type(&mut self) -> Result<WH1080MessageType, String>
    {
        return Err("not implemented".to_string());
    }
}

impl<'a, T: WH1080Buf + ?Sized> WH1080Buf for &'a mut T {}
impl<T: AsRef<[u8]>> WH1080Buf for std::io::Cursor<T> {}
