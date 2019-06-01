use super::sml_buffer::*;
use super::sml_types::*;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlListEntry
{
    pub obj_name: Vec<u8>,
    pub status: Option<SmlStatus>,
    pub val_time: Option<SmlTime>,
    pub unit: Option<u8>,
    pub scaler: Option<i8>,
    pub value: SmlValue,
    pub signature: Option<Vec<u8>>
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SmlGetListResponse
{
    pub client_id: Option<Vec<u8>>,
    pub server_id: Vec<u8>,
    pub list_name: Option<Vec<u8>>,
    pub act_sensor_time: Option<SmlTime>,
    pub values: Vec<SmlListEntry>,
    pub signature: Option<Vec<u8>>,
    pub act_gateway_time: Option<SmlTime>
}

impl SmlGetListResponse
{
    pub fn deserialize(buffer: &mut SmlBuf) -> Result<SmlGetListResponse, String>
    {
        return Err("not implemented yet".to_string());
        // TODO: implement
/*        return SmlGetListResponse
        {
            client_id: None,
            server_id: Vec::new(),
            list_name: None,
            act_sensor_time: None,
            values: Vec::new(),
            signature: None,
            act_gateway_time: None
        };*/
    }
}
