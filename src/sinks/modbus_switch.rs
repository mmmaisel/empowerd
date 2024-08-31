/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2022 Max Maisel

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
use crate::SwitchGroup;
use async_trait::async_trait;
use std::{net::SocketAddr, time::Duration};
use tokio::{sync::Mutex, time};
use tokio_modbus::{
    client::tcp::connect_slave,
    prelude::{Reader, Slave, Writer},
};

fn read_err_msg<S>(e: S) -> String
where
    S: std::fmt::Display,
{
    format!("Could not read coils: {e}")
}

fn write_err_msg<S>(e: S) -> String
where
    S: std::fmt::Display,
{
    format!("Could not write coils: {e}")
}

#[derive(Debug)]
pub struct ModbusSwitch {
    addr: SocketAddr,
    slave: Slave,
    // Mutex guards the remote device from concurrent access,
    // at least from within this app.
    lock: Mutex<()>,
}

impl ModbusSwitch {
    pub fn new(addr: SocketAddr, slave: u8) -> Self {
        Self {
            addr,
            slave: slave.into(),
            lock: Mutex::new(()),
        }
    }
}

#[async_trait]
impl SwitchGroup for ModbusSwitch {
    async fn read_val(&self, idx: usize) -> Result<bool, String> {
        let guard = self.lock.lock().await;
        let result = time::timeout(Duration::from_secs(1), async {
            let mut session = connect_slave(self.addr, self.slave)
                .await
                .map_err(|e| format!("Could not connect to device: {e}"))?;
            let coil = session
                .read_coils(idx as u16, 1)
                .await
                .map_err(|e| read_err_msg(e))?
                .map_err(|e| read_err_msg(e))?;
            Ok(coil[0])
        })
        .await
        .map_err(|_e| "Reading modbus coil timed out".to_string())?;

        drop(guard);
        result
    }

    async fn write_val(&self, idx: usize, val: bool) -> Result<(), String> {
        let guard = self.lock.lock().await;
        let result = time::timeout(Duration::from_secs(1), async {
            let mut session = connect_slave(self.addr, self.slave)
                .await
                .map_err(|e| format!("Could not connect to device: {e}"))?;
            session
                .write_single_coil(idx as u16, val)
                .await
                .map_err(|e| write_err_msg(e))?
                .map_err(|e| write_err_msg(e))?;
            Ok(())
        })
        .await
        .map_err(|_e| "Writing modbus coil timed out".to_string())?;

        drop(guard);
        result
    }
}
