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
use std::collections::BTreeMap;
use std::net::SocketAddr;
use tokio_modbus::{
    client::{tcp::connect_slave, Context},
    prelude::Reader,
};

#[derive(Debug)]
pub struct SunspecClient {
    addr: SocketAddr,
    logger: Option<Logger>,
    models: BTreeMap<u16, u16>,
}

impl SunspecClient {
    const SUNSPEC_START_ADDR: u16 = 40000;

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
            models: BTreeMap::new(),
        });
    }

    pub async fn open(&self) -> Result<Context, String> {
        return connect_slave(self.addr, 126.into())
            .await
            .map_err(|e| format!("Could not connect to device: {}", e));
    }

    pub async fn introspect(
        &mut self,
        context: &mut Context,
    ) -> Result<(), String> {
        let mut reg_addr = Self::SUNSPEC_START_ADDR;
        let model = match context.read_input_registers(reg_addr, 2).await {
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
            let header = match context.read_input_registers(reg_addr, 2).await {
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

    async fn read_register(
        &self,
        context: &mut Context,
        model: u16,
        register: u16,
        size: u16,
    ) -> Result<Vec<u16>, String> {
        let model_base = match self.models.get(&model) {
            Some(x) => x,
            None => return Err(format!("The device does not support model {}", &model)),
        };
        let addr = model_base + register;
        return context.read_input_registers(addr, size).await.map_err(|e| e.to_string());
    }
}

#[tokio::test]
async fn test_sunspec_client() {
    let mut client =
        SunspecClient::new("127.0.0.1".into(), 1502, None).unwrap();
    let mut context = client.open().await.unwrap();
    client.introspect(&mut context).await.unwrap();

    assert_eq!(Some(&40002u16), client.models.get(&1));
    eprintln!("{:?}", &client);
}
