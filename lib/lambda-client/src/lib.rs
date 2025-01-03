/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2023 Max Maisel

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

use std::net::SocketAddr;
use tokio_modbus::{
    client::{tcp::connect, Context},
    prelude::{Reader, Writer},
};

fn read_err_msg<S>(reg: u16, e: S) -> String
where
    S: std::fmt::Display,
{
    format!("Could not read register {reg}: {e}")
}

fn write_err_msg<S>(reg: u16, e: S) -> String
where
    S: std::fmt::Display,
{
    format!("Could not write register {reg}: {e}")
}

pub enum LambdaMode {
    Heating,
    Cooling,
    Defrosting,
}

pub struct LambdaContext(Context);

impl LambdaContext {
    pub async fn get_op_mode(&mut self) -> Result<LambdaMode, String> {
        let state = match self
            .0
            .read_holding_registers(1002, 1)
            .await
            .map_err(|e| read_err_msg(1002, e))?
            .map_err(|e| read_err_msg(1002, e))?[0]
        {
            9 => LambdaMode::Cooling,
            10 => LambdaMode::Defrosting,
            _ => LambdaMode::Heating,
        };

        Ok(state)
    }

    pub async fn get_current_power(&mut self) -> Result<i16, String> {
        let data = self
            .0
            .read_holding_registers(103, 1)
            .await
            .map_err(|e| read_err_msg(103, e))?
            .map_err(|e| read_err_msg(103, e))?;

        Ok(data[0] as i16)
    }

    pub async fn get_cop(&mut self) -> Result<i16, String> {
        let data = self
            .0
            .read_holding_registers(1013, 1)
            .await
            .map_err(|e| read_err_msg(1013, e))?
            .map_err(|e| read_err_msg(1013, e))?;

        Ok(data[0] as i16)
    }

    pub async fn get_boiler_temps(
        &mut self,
    ) -> Result<(i16, i16, i16), String> {
        let boiler_data = self
            .0
            .read_holding_registers(2002, 1)
            .await
            .map_err(|e| read_err_msg(2002, e))?
            .map_err(|e| read_err_msg(2002, e))?;
        let buffer_data = self
            .0
            .read_holding_registers(3002, 2)
            .await
            .map_err(|e| read_err_msg(3002, e))?
            .map_err(|e| read_err_msg(3002, e))?;
        Ok((
            boiler_data[0] as i16,
            buffer_data[0] as i16,
            buffer_data[1] as i16,
        ))
    }

    pub async fn set_available_power(
        &mut self,
        power: i16,
    ) -> Result<(), String> {
        self.0
            .write_multiple_registers(102, &[power as u16])
            .await
            .map_err(|e| write_err_msg(102, e))?
            .map_err(|e| write_err_msg(102, e))
    }
}

/// Register addresses are derived from the following pattern: ABCC
/// with A: index (type), B: subindex, CC: decimal register number
/// Here we use "general E-Manager" index 0, subindex 1 and
/// register numbers 2 and 3.
/// Note: The device must be configured as Modbus-TCP and E-Input mode.
#[derive(Debug)]
pub struct LambdaClient {
    addr: SocketAddr,
}

impl LambdaClient {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn open(&self) -> Result<LambdaContext, String> {
        connect(self.addr)
            .await
            .map(LambdaContext)
            .map_err(|e| format!("Could not connect to device: {}", e))
    }
}

#[tokio::test]
async fn test_lambda_client() {
    let mut client = LambdaClient::new("127.0.0.1:1502".parse().unwrap());
    let mut context = client.open().await.unwrap();

    assert_eq!(Ok(()), context.set_available_power(1000).await);
    assert_eq!(Ok(1000), context.get_current_power().await);
}
