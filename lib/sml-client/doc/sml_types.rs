pub enum SmlTime
{
    SecIndex(u32),
    Timestamp(u32)
}

pub enum SmlStatus
{
    Status8(u8),
    Status16(u16),
    Status32(u32),
    Status64(u64)
}

pub enum SmlValue
{
    Boolean(bool),
    OctetString(Vec<u8>),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64)
}
