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

impl SmlListEntry
{
    pub fn deserialize(mut buffer: &mut SmlBuf)
        -> Result<SmlListEntry, String>
    {
        let tl = buffer.get_sml_tl();
        match tl
        {
            SmlType::Struct(len) =>
            {
                if len != 7
                {
                    return Err(format!(
                        "Invalid length {} for SmlListEntry", len));
                }
            }
            _ => return Err(format!(
                "Found {:X?}, expected struct", tl))
        }

        let obj_name = match buffer.get_sml_octet_str()?
        {
            Some(x) => x,
            None => return Err(
                "SmlListEntry obj_name is required".to_string())
        };
        let status = SmlStatus::deserialize(&mut buffer)?;
        let val_time = SmlTime::deserialize(&mut buffer)?;
        let unit = buffer.get_sml_u8()?;
        let scaler = buffer.get_sml_i8()?;
        let value = SmlValue::deserialize(&mut buffer)?;
        let signature = buffer.get_sml_octet_str()?;

        return Ok(SmlListEntry
        {
            obj_name: obj_name,
            status: status,
            val_time: val_time,
            unit: unit,
            scaler: scaler,
            value: value,
            signature: signature
        });
    }
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
    pub fn deserialize(mut buffer: &mut SmlBuf)
        -> Result<SmlGetListResponse, String>
    {
        let tl = buffer.get_sml_tl();
        match tl
        {
            SmlType::Struct(len) =>
            {
                if len != 7
                {
                    return Err(format!(
                        "Invalid length {} for SmlGetListResponse", len));
                }
            }
            _ => return Err(format!(
                "Found {:X?}, expected struct", tl))
        }

        let client_id = buffer.get_sml_octet_str()?;
        let server_id = match buffer.get_sml_octet_str()?
        {
            Some(x) => x,
            None => return Err(
                "GetListResponse server_id is required".to_string())
        };
        let list_name = buffer.get_sml_octet_str()?;
        let act_sensor_time = SmlTime::deserialize(&mut buffer)?;

        let entry_count_tl = buffer.get_sml_tl();
        let entry_count = match entry_count_tl
        {
            SmlType::Struct(len) => len,
            _ => return Err(format!(
                "Found {:X?}, expected struct", entry_count_tl))
        };

        let mut values: Vec<SmlListEntry> = Vec::with_capacity(entry_count);
        for _ in 0..entry_count
        {
            values.push(SmlListEntry::deserialize(&mut buffer)?);
        }

        let signature = buffer.get_sml_octet_str()?;
        let act_gateway_time = SmlTime::deserialize(&mut buffer)?;

        return Ok(SmlGetListResponse
        {
            client_id: client_id,
            server_id: server_id,
            list_name: list_name,
            act_sensor_time: act_sensor_time,
            values: values,
            signature: signature,
            act_gateway_time: act_gateway_time
        });
    }
}
