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
#![forbid(unsafe_code)]

use slog::{trace, Logger};
use std::io::Error;
use std::net::SocketAddr;
use tokio_modbus::client::tcp::connect_slave;
use tokio_modbus::prelude::Reader;

pub struct SunnyIslandClient {
    addr: SocketAddr,
    logger: Option<Logger>,
}

impl SunnyIslandClient {
    const BAT_CHA_STT: u16 = 30845;
    const METERING_WH_IN: u16 = 30595;
    const METERING_WH_OUT: u16 = 30597;
    const BAT_CAPAC_RTG_WH: u16 = 40187;

    pub fn new(
        addr: String,
        port: u16,
        logger: Option<Logger>,
    ) -> Result<Self, String> {
        let addr: SocketAddr = match format!("{}:{}", addr, port).parse() {
            Ok(x) => x,
            Err(e) => return Err(e.to_string()),
        };

        return Ok(Self {
            addr: addr,
            logger: logger,
        });
    }

    fn validate_result(
        &self,
        which: &str,
        res: Result<Vec<u16>, Error>,
    ) -> Result<u32, String> {
        match res {
            Err(e) => return Err(e.to_string()),
            Ok(data) => {
                if let Some(l) = &self.logger {
                    trace!(l, "RAW {}: {:?}", &which, &data);
                }
                if data.iter().all(|x| *x == 0xFFFF) {
                    return Err(format!(
                        "Received invalid value for {}",
                        which
                    ));
                }
                return Ok((data[0] as u32) * 65536 + (data[1] as u32));
            }
        };
    }

    pub async fn get_in_out_charge(&self) -> Result<(u32, u32, f64), String> {
        let mut client = connect_slave(self.addr, 3.into())
            .await
            .map_err(|e| format!("Coult not connect to sunny island: {}", e))?;
        let wh_in = self.validate_result(
            "METERING_WH_IN",
            client.read_input_registers(Self::METERING_WH_IN, 2).await,
        )?;
        let wh_out = self.validate_result(
            "METERING_WH_OUT",
            client.read_input_registers(Self::METERING_WH_OUT, 2).await,
        )?;
        let charge = self.validate_result(
            "BAT_CHA_STT",
            client.read_input_registers(Self::BAT_CHA_STT, 2).await,
        )?;
        let capacity = self.validate_result(
            "BAT_CAPAC_RTG_WH",
            client.read_input_registers(Self::BAT_CAPAC_RTG_WH, 2).await,
        )?;

        if charge == 0 {
            return Err("Received invalid value 0 for charge.".into());
        }

        return Ok((wh_in, wh_out, (charge as f64) * (capacity as f64) / 100.0));
    }
}

#[tokio::test]
async fn test_sunny_island_client() {
    let client =
        SunnyIslandClient::new("127.0.0.1".into(), 1502, None).unwrap();

    match client.get_in_out_charge().await {
        Ok((wh_in, wh_out, charge)) => {
            println!("in: {}, out: {}, charge: {}", wh_in, wh_out, charge);
        }
        Err(e) => panic!("get_in_out_charge failed: {}", e),
    }
}
