use bytes::Buf;

pub struct SmlCloseResponse
{
    signature: Option<Vec<u8>>
}

impl SmlCloseResponse
{
    pub fn deserialize(buffer: &mut Buf) -> SmlCloseResponse
    {
        return SmlCloseResponse
        {
            signature: None // TODO: implement this
        };
    }
}
