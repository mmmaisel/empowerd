use bytes::Buf;

pub enum Bresser6in1MessageType
{
    Dummy
}

pub trait Bresser6in1Buf: Buf
{

    fn get_message_type(&mut self) -> Result<Bresser6in1MessageType, String>
    {
        return Err("not implemented".to_string());
    }
}

impl<'a, T: Bresser6in1Buf + ?Sized> Bresser6in1Buf for &'a mut T {}
impl<T: AsRef<[u8]>> Bresser6in1Buf for std::io::Cursor<T> {}
