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
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

use async_trait::async_trait;
use slog::{trace, Logger};
use std::io::Error;
use std::net::SocketAddr;
use tokio_modbus::client::tcp::connect_slave;
use tokio_modbus::prelude::Reader;

macro_rules! impl_client {
    ($name:ident, $registers:expr) => {
        pub struct $name {
            addr: SocketAddr,
            logger: Option<Logger>,
        }

        impl $name {
            pub fn new(
                addr: SocketAddr,
                logger: Option<Logger>,
            ) -> Result<Self, String> {
                return Ok(Self { addr, logger });
            }
        }

        #[async_trait]
        impl SunnyStorageClient for $name {
            async fn get_in_out_charge(
                &self,
            ) -> Result<(u32, u32, f64), String> {
                return get_in_out_charge(&self.addr, &self.logger, $registers)
                    .await;
            }
        }
    };
}

#[derive(Debug)]
#[allow(non_snake_case)]
struct RegisterMap {
    BAT_CHA_STT: u16,
    METERING_WH_IN: u16,
    METERING_WH_OUT: u16,
    BAT_CAPAC_RTG_WH: u16,
}

const SUNNY_ISLAND_REGISTERS: RegisterMap = RegisterMap {
    BAT_CHA_STT: 30845,
    METERING_WH_IN: 30595,
    METERING_WH_OUT: 30597,
    BAT_CAPAC_RTG_WH: 40187,
};

const SUNNY_BOY_STORAGE_REGISTERS: RegisterMap = RegisterMap {
    BAT_CHA_STT: 30845,
    METERING_WH_IN: 31397,
    METERING_WH_OUT: 31401,
    BAT_CAPAC_RTG_WH: 40187,
};

impl_client!(SunnyIslandClient, SUNNY_ISLAND_REGISTERS);
impl_client!(SunnyBoyStorageClient, SUNNY_BOY_STORAGE_REGISTERS);

#[async_trait]
pub trait SunnyStorageClient: Send + Sync {
    async fn get_in_out_charge(&self) -> Result<(u32, u32, f64), String>;
}

fn validate_result(
    which: &str,
    res: Result<Vec<u16>, Error>,
    logger: &Option<Logger>,
) -> Result<u32, String> {
    match res {
        Err(e) => return Err(e.to_string()),
        Ok(data) => {
            if let Some(l) = &logger {
                trace!(l, "RAW {}: {:?}", &which, &data);
            }
            if data.iter().all(|x| *x == 0xFFFF) {
                return Err(format!("Received invalid value for {}", which));
            }
            return Ok((data[0] as u32) * 65536 + (data[1] as u32));
        }
    };
}

async fn get_in_out_charge(
    addr: &SocketAddr,
    logger: &Option<Logger>,
    registers: RegisterMap,
) -> Result<(u32, u32, f64), String> {
    let mut client = connect_slave(*addr, 3.into())
        .await
        .map_err(|e| format!("Could not connect to sunny storage: {}", e))?;
    let wh_in = validate_result(
        "METERING_WH_IN",
        client
            .read_input_registers(registers.METERING_WH_IN, 2)
            .await,
        logger,
    )?;
    let wh_out = validate_result(
        "METERING_WH_OUT",
        client
            .read_input_registers(registers.METERING_WH_OUT, 2)
            .await,
        logger,
    )?;
    let charge = validate_result(
        "BAT_CHA_STT",
        client.read_input_registers(registers.BAT_CHA_STT, 2).await,
        logger,
    )?;
    let capacity = validate_result(
        "BAT_CAPAC_RTG_WH",
        client
            .read_input_registers(registers.BAT_CAPAC_RTG_WH, 2)
            .await,
        logger,
    )?;

    if charge == 0 {
        return Err("Received invalid value 0 for charge.".into());
    }

    return Ok((wh_in, wh_out, (charge as f64) * (capacity as f64) / 100.0));
}

#[tokio::test]
async fn test_sunny_island_client() {
    let client =
        SunnyIslandClient::new("127.0.0.1:1502".parse().unwrap(), None)
            .unwrap();

    match client.get_in_out_charge().await {
        Ok((wh_in, wh_out, charge)) => {
            println!("in: {}, out: {}, charge: {}", wh_in, wh_out, charge);
        }
        Err(e) => panic!("get_in_out_charge failed: {}", e),
    }
}
