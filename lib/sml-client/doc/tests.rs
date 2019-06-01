use std::io::Cursor;

use super::*;

#[test]
fn decode_sml_open_response()
{
    let data = vec!
    [
        0x76, 0x01, 0x01, 0x07, 0x00, 0x0e, 0x05, 0xe5,
        0xe1, 0x9e, 0x0b, 0x09, 0x01, 0x45, 0x4d, 0x48,
        0x00, 0x00, 0x50, 0xda, 0xfa, 0x01, 0x01
    ];
    let expected_result = Ok(SmlOpenResponse
    {
        codepage: None,
        client_id: None,
        req_file_id: vec![0x00, 0x0e, 0x05, 0xe5, 0xe1, 0x9e],
        server_id: vec![0x09, 0x01, 0x45, 0x4d, 0x48, 0x00, 0x00, 0x50,
            0xda, 0xfa],
        ref_time: None,
        version: None
    });

    let result = SmlOpenResponse::deserialize(&mut Cursor::new(data));
    assert_eq!(expected_result, result, "Incrrectly parsed OpenResponse");
}

#[test]
fn decode_sml_close_response()
{
    let data = vec!
    [
        0x71, 0x01
    ];
    let expected_result = Ok(SmlCloseResponse
    {
        signature: None
    });

    let result = SmlCloseResponse::deserialize(&mut Cursor::new(data));
    assert_eq!(expected_result, result, "Incrrectly parsed CloseResponse");
}

#[test]
fn decode_sml_get_list_response()
{
    let data = vec!
    [
        0x77, 0x01, 0x02, 0xaa, 0x01, 0x01, 0x71, 0x77,
        0x07, 0x81, 0x81, 0xc7, 0x82, 0x03, 0xff, 0x01,
        0x01, 0x01, 0x01, 0x04, 0x11, 0x22, 0x33, 0x01,
        0x01, 0x01
    ];
    let expected_result = Ok(SmlGetListResponse
    {
        client_id: None,
        server_id: vec![0xaa],
        list_name: None,
        act_sensor_time: None,
        values: vec!
        [
            SmlListEntry
            {
                obj_name: vec![0x81, 0x81, 0xc7, 0x82, 0x03, 0xff],
                status: None,
                val_time: None,
                unit: None,
                scaler: None,
                value: SmlValue::OctetString(vec![0x11, 0x22, 0x33]),
                signature: None
            }
        ],
        signature: None,
        act_gateway_time: None
    });

    let result = SmlGetListResponse::deserialize(&mut Cursor::new(data));
    assert_eq!(expected_result, result, "Incorrectly parsed GetListResponse");
}

#[test]
fn decode_sml_file()
{
    let data = vec!
    [
        // TODO: add and check correct crc vales
        0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01,
        0x76, 0x05, 0x1b, 0x1b, 0x1b, 0x1b, 0x1b, 0x1b,
        0x1b, 0x1b, 0x62, 0x00, 0x62, 0x00, 0x72, 0x63,
        0x02, 0x01, 0x71, 0x01, 0x63, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x1b, 0x1b, 0x1b, 0x1b, 0x1a,
        0x03, 0x00, 0x00
    ];
    let expected_result = Ok(SmlFile
    {
        version: 0x01010101,
        messages: vec!
        [
            SmlMessage
            {
                transaction_id: vec![0x1b, 0x1b, 0x1b, 0x1b],
                group_no: 0x00,
                abort_on_error: 0x00,
                body: SmlBody::CloseResponse(SmlCloseResponse
                {
                    signature: None
                }),
                crc: 0x0000
            }
        ],
        end_crc: 0x1a030000
    });

    let result = SmlFile::deserialize(&mut Cursor::new(data));
    assert_eq!(expected_result, result, "Incorrectly parsed SmlFile");
}
