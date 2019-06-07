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
        0x77, 0x01, 0x02, 0xaa, 0x01, 0x72, 0x62, 0x01,
        0x65, 0x01, 0x02, 0x03, 0x04, 0x71, 0x77, 0x07,
        0x81, 0x81, 0xc7, 0x82, 0x03, 0xff, 0x01, 0x01,
        0x01, 0x01, 0x04, 0x11, 0x22, 0x33, 0x01, 0x01,
        0x01
    ];
    let expected_result = Ok(SmlGetListResponse
    {
        client_id: None,
        server_id: vec![0xaa],
        list_name: None,
        act_sensor_time: Some(SmlTime::SecIndex(0x01020304)),
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
    let stream = SmlStream
    {
        header: 0x01010101,
        data: vec!
        [
            0x76, 0x05, 0x10, 0x10, 0x1b, 0x1b, 0x62, 0x00,
            0x62, 0x00, 0x72, 0x63, 0x02, 0x01, 0x71, 0x01,
            0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ],
        footer: 0x1a030000
    };
    let expected_result = Ok(SmlFile
    {
        version: 0x01010101,
        messages: vec!
        [
            SmlMessage
            {
                transaction_id: vec![0x10, 0x10, 0x1b, 0x1b],
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

    let result = SmlFile::deserialize(stream);
    assert_eq!(expected_result, result, "Incorrectly parsed SmlFile");
}

#[test]
fn decode_multi_byte_tl()
{
    let data = vec!
    [
        0x83, 0x02, 0x67, 0x83, 0x91, 0x2b, 0xd3, 0x8f,
        0x3a, 0xcb, 0x49, 0x94, 0x17, 0x66, 0x00, 0xb4,
        0xc1, 0x5e, 0x24, 0xff, 0xf2, 0x14, 0xf9, 0x80,
        0x15, 0xa6, 0x67, 0xed, 0x37, 0xe3, 0x8b, 0x57,
        0xf1, 0x35, 0xfe, 0x71, 0x80, 0xcb, 0xe1, 0x82,
        0x0a, 0x7d, 0x56, 0x57, 0x2a, 0xe0, 0x5e, 0x67,
        0x47, 0xb8
    ];
    let expected_result = Ok(Some(vec!
    [
        0x67, 0x83, 0x91, 0x2b, 0xd3, 0x8f, 0x3a, 0xcb,
        0x49, 0x94, 0x17, 0x66, 0x00, 0xb4, 0xc1, 0x5e,
        0x24, 0xff, 0xf2, 0x14, 0xf9, 0x80, 0x15, 0xa6,
        0x67, 0xed, 0x37, 0xe3, 0x8b, 0x57, 0xf1, 0x35,
        0xfe, 0x71, 0x80, 0xcb, 0xe1, 0x82, 0x0a, 0x7d,
        0x56, 0x57, 0x2a, 0xe0, 0x5e, 0x67, 0x47, 0xb8
    ]));

    let mut buffer = Cursor::new(data);
    let result = buffer.get_sml_octet_str();
    assert_eq!(expected_result, result,
        "Incorrectly parsed multi-bytes TL octet string");
}

#[test]
fn sync_to_file()
{
    let data = vec!
    [
        0x11, 0x1b, 0x1b, 0x22, 0x37, 0x99,
        0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01,
        0x76, 0x06, 0x1b, 0x1b, 0x1b, 0x1b, 0x1b, 0x1b,
        0x1b, 0x1b, 0x1c, 0x62, 0x00, 0x62, 0x00, 0x72,
        0x63, 0x02, 0x01, 0x71, 0x01, 0x63, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x1b, 0x1b, 0x1b, 0x1b,
        0x1a, 0x03, 0x00, 0x00, 0x1b, 0x1b, 0x1b, 0x1b,
        0x01, 0x01, 0x01, 0x01, 0x1b, 0x1b, 0x1b, 0x1b,
        0x1a, 0x00, 0x00, 0x01
    ];
    let expected_result = Ok(vec!
    [
        SmlFile
        {
            version: 0x01010101,
            messages: vec!
            [
                SmlMessage
                {
                    transaction_id: vec![0x1b, 0x1b, 0x1b, 0x1b, 0x1c],
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
        },
        SmlFile
        {
            version: 0x01010101,
            messages: Vec::new(),
            end_crc: 0x1a000001
        }
    ]);

    let streams = match SmlStream::deserialize(&mut Cursor::new(data))
    {
        Ok(x) => x,
        Err(e) => panic!("Could not deserialize stream, error {}", e)
    };
    let result = SmlFile::deserialize_streams(streams);
    assert_eq!(expected_result, result, "Incorrectly synced to data");
}
