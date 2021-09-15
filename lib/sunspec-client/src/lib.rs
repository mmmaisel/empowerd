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

use slog::{trace, Logger};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use tokio_modbus::{
    client::{
        tcp::{connect, connect_slave},
        Context,
    },
    prelude::Reader,
};

#[derive(Debug)]
pub struct SunspecClient {
    addr: SocketAddr,
    id: Option<u8>,
    logger: Option<Logger>,
    models: BTreeMap<u16, u16>,
}

impl SunspecClient {
    const SUNSPEC_START_ADDR: u16 = 40000;

    const SUNSPEC_INVERTER_1: u16 = 101;
    const SUNSPEC_INVERTER_2: u16 = 102;
    const SUNSPEC_INVERTER_3: u16 = 103;

    const SUNSPEC_INVERTER_YIELD_ACC: u16 = 24;
    const SUNSPEC_INVERTER_YIELD_ACC_SIZE: u16 = 2;
    const SUNSPEC_INVERTER_YIELD_SCALE: u16 = 26;
    const SUNSPEC_INVERTER_YIELD_SCALE_SIZE: u16 = 1;

    pub fn new(
        addr: SocketAddr,
        id: Option<u8>,
        logger: Option<Logger>,
    ) -> Self {
        return Self {
            addr: addr,
            id: id,
            logger: logger,
            models: BTreeMap::new(),
        };
    }

    pub async fn open(&self) -> Result<Context, String> {
        match self.id {
            Some(id) => connect_slave(self.addr, id.into())
                .await
                .map_err(|e| format!("Could not connect to device: {}", e)),
            None => connect(self.addr)
                .await
                .map_err(|e| format!("Could not connect to device: {}", e)),
        }
    }

    pub async fn introspect(
        &mut self,
        context: &mut Context,
    ) -> Result<(), String> {
        let mut reg_addr = Self::SUNSPEC_START_ADDR;
        let model = match context.read_holding_registers(reg_addr, 2).await {
            Ok(x) => x,
            Err(e) => {
                return Err(format!(
                    "Could not read register {}: {}",
                    reg_addr, e
                ))
            }
        };
        if !(model[0] == 0x5375 && model[1] == 0x6e53) {
            return Err("Device does not support sunspec protocol".into());
        }

        reg_addr += 2;
        loop {
            let header = match context.read_holding_registers(reg_addr, 2).await {
                Ok(x) => x,
                Err(e) => {
                    return Err(format!(
                        "Could not read register {}: {}",
                        reg_addr, e
                    ))
                }
            };

            let model_id = header[0];
            let len = header[1];
            if let Some(l) = &self.logger {
                trace!(l, "Found model {} at {}", &model_id, &reg_addr);
            }
            self.models.insert(model_id, reg_addr);

            if model_id == 65535 {
                break;
            }
            reg_addr += len + 2;
        }
        return Ok(());
    }

    pub fn models(&self) -> &BTreeMap<u16, u16> {
        return &self.models;
    }

    pub async fn get_total_yield(
        &self,
        context: &mut Context,
    ) -> Result<f64, String> {
        let model = if self.models.contains_key(&Self::SUNSPEC_INVERTER_1) {
            Self::SUNSPEC_INVERTER_1
        } else if self.models.contains_key(&Self::SUNSPEC_INVERTER_2) {
            Self::SUNSPEC_INVERTER_2
        } else {
            Self::SUNSPEC_INVERTER_3
        };

        let total_energy = Self::validate_result_u32(
            "SUNSPEC_INVERTER_YIELD_ACC",
            self.read_register(
                context,
                model,
                Self::SUNSPEC_INVERTER_YIELD_ACC,
                Self::SUNSPEC_INVERTER_YIELD_ACC_SIZE,
            )
            .await,
        )?;

        let scale = Self::validate_result_i16(
            "SUNSPEC_INVERTER_YIELD_SCALE",
            self.read_register(
                context,
                model,
                Self::SUNSPEC_INVERTER_YIELD_SCALE,
                Self::SUNSPEC_INVERTER_YIELD_SCALE_SIZE,
            )
            .await,
        )?;

        return Ok(total_energy as f64 * 10_f64.powf(scale as f64));
    }

    async fn read_register(
        &self,
        context: &mut Context,
        model: u16,
        register: u16,
        size: u16,
    ) -> Result<Vec<u16>, String> {
        let model_base = match self.models.get(&model) {
            Some(x) => x,
            None => {
                return Err(format!(
                    "The device does not support model {}",
                    &model
                ))
            }
        };
        let addr = model_base + register;
        return context
            .read_holding_registers(addr, size)
            .await
            .map_err(|e| e.to_string());
    }

    fn validate_result_i16(
        which: &str,
        res: Result<Vec<u16>, String>,
    ) -> Result<i16, String> {
        match res {
            Err(e) => return Err(e),
            Ok(data) => {
                if data[0] == 0x8000 {
                    return Err(format!(
                        "Received invalid value for {}",
                        which
                    ));
                }
                return Ok(data[0] as i16);
            }
        };
    }

    fn validate_result_u32(
        which: &str,
        res: Result<Vec<u16>, String>,
    ) -> Result<u32, String> {
        match res {
            Err(e) => return Err(e),
            Ok(data) => {
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
}

#[tokio::test]
async fn test_sunspec_client() {
    let mut client =
        SunspecClient::new("127.0.0.1:1502".parse().unwrap(), Some(126), None);
    let mut context = client.open().await.unwrap();
    client.introspect(&mut context).await.unwrap();

    assert_eq!(Some(&40002u16), client.models.get(&1));
    eprintln!("{:?}", &client);
}
