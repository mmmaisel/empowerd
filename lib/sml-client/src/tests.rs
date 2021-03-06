/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
use crate::doc::*;
use crate::SmlClient;

#[test]
fn extract_meter_data() {
    let file = SmlFile {
        version: 0x01010101,
        messages: vec![SmlMessage {
            transaction_id: vec![0x10, 0x10, 0x1b, 0x1b],
            group_no: 0x00,
            abort_on_error: 0x00,
            body: SmlBody::GetListResponse(SmlGetListResponse {
                client_id: None,
                server_id: vec![0xaa],
                list_name: None,
                act_sensor_time: Some(SmlTime::SecIndex(0x01020304)),
                values: vec![
                    SmlListEntry {
                        obj_name: vec![0x81, 0x81, 0xc7, 0x82, 0x03, 0xff],
                        status: None,
                        val_time: None,
                        unit: None,
                        scaler: None,
                        value: SmlValue::OctetString(vec![0x11, 0x22, 0x33]),
                        signature: None,
                    },
                    SmlListEntry {
                        obj_name: vec![1, 0, 1, 8, 0, 255],
                        status: None,
                        val_time: None,
                        unit: None,
                        scaler: Some(-1),
                        value: SmlValue::Int(55),
                        signature: None,
                    },
                    SmlListEntry {
                        obj_name: vec![1, 0, 2, 8, 0, 255],
                        status: None,
                        val_time: None,
                        unit: None,
                        scaler: Some(1),
                        value: SmlValue::UInt(66),
                        signature: None,
                    },
                ],
                signature: None,
                act_gateway_time: None,
            }),
            crc: 0x0000,
        }],
    };

    let data = SmlClient::extract_produced_consumed(file);
    match data {
        Ok(x) => {
            assert_eq!(5.5, x.0, "Incorrectly extracted consumed");
            assert_eq!(660.0, x.1, "Incorrectly extracted produced");
        }
        Err(e) => panic!("Error {} occured", e),
    }
}

#[tokio::test]
async fn read_from_device() {
    let mut client = SmlClient::new("/dev/ttyUSB0".into(), 9600, None);
    let file = match client.get_sml_file().await {
        Ok(x) => x,
        Err(e) => panic!("Reading SMLFile failed: {}", e),
    };
    if let Err(e) = SmlClient::extract_produced_consumed(file) {
        panic!("Received file could not be parsed: {}", e);
    }
}
