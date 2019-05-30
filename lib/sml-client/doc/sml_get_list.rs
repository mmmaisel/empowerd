use super::sml_types::*;

use bytes::Buf;

pub struct SmlListEntry
{
    obj_name: Vec<u8>,
    status: Option<SmlStatus>,
    val_time: Option<SmlTime>,
    unit: Option<u8>,
    scaler: Option<i8>,
    value: SmlValue,
    signature: Option<Vec<u8>>
}

pub struct SmlGetListResponse
{
    client_id: Option<Vec<u8>>,
    server_id: Vec<u8>,
    list_name: Option<Vec<u8>>,
    act_sensor_time: Option<SmlTime>,
    values: Vec<SmlListEntry>,
    signature: Option<Vec<u8>>,
    act_gateway_time: Option<SmlTime>
}

impl SmlGetListResponse
{
    pub fn deserialize(buffer: &mut Buf) -> SmlGetListResponse
    {
        // TODO: implement
        return SmlGetListResponse
        {
            client_id: None,
            server_id: Vec::new(),
            list_name: None,
            act_sensor_time: None,
            values: Vec::new(),
            signature: None,
            act_gateway_time: None
        };
    }
}
