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
#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

use serde::Deserialize;
use slog::{debug, Logger};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::net::UdpSocket;

#[derive(Deserialize, Debug)]
pub struct StatusReport {
    #[serde(rename = "State")]
    pub state: u32,
    #[serde(rename = "Plug")]
    pub plug: u32,
    #[serde(rename = "Error1")]
    pub error1: u32,
    #[serde(rename = "Error2")]
    pub error2: u32,
    #[serde(rename = "Enable user")]
    pub enabled: u32,
    #[serde(rename = "Curr HW")]
    pub max_current_hw: u32,
    #[serde(rename = "Curr user")]
    pub max_current: u32,
}

#[derive(Deserialize, Debug)]
pub struct PowerReport {
    // V
    #[serde(rename = "U1")]
    pub voltage1: u32,
    #[serde(rename = "U2")]
    pub voltage2: u32,
    #[serde(rename = "U3")]
    pub voltage3: u32,
    // mA
    #[serde(rename = "I1")]
    pub current1: u32,
    #[serde(rename = "I2")]
    pub current2: u32,
    #[serde(rename = "I3")]
    pub current3: u32,
    // 0.1 Wh
    #[serde(rename = "E pres")]
    pub e_pres: u64,
    #[serde(rename = "E total")]
    pub e_total: u64,
}

#[derive(Debug)]
pub struct KeContactClient {
    addr: SocketAddr,
    logger: Option<Logger>,
}

impl KeContactClient {
    pub fn new(addr: SocketAddr, logger: Option<Logger>) -> Self {
        Self { addr, logger }
    }

    async fn connect(&self) -> Result<UdpSocket, String> {
        let local_addr = Ipv4Addr::new(0, 0, 0, 0);
        let socket = UdpSocket::bind(SocketAddrV4::new(local_addr, 7090))
            .await
            .map_err(|e| format!("Binding socket failed, error: {}", e))?;
        socket.connect(self.addr).await.map_err(|e| {
            format!("Connecting to {} failed: {}", self.addr, e)
        })?;
        Ok(socket)
    }

    fn check_acknowledge(response: &[u8]) -> Result<(), String> {
        if response != b"TCH-OK :done\n" {
            Err(format!(
                "Received invalid response {}",
                String::from_utf8_lossy(response)
            ))
        } else {
            Ok(())
        }
    }

    pub async fn set_enable(&self, enabled: bool) -> Result<(), String> {
        let mut response = [0; 16];
        let socket = self.connect().await?;
        if enabled {
            socket.send(b"ena 1").await.map_err(|e| e.to_string())?;
        } else {
            socket.send(b"ena 0").await.map_err(|e| e.to_string())?;
        }

        let len = socket
            .recv(&mut response)
            .await
            .map_err(|e| e.to_string())?;
        Self::check_acknowledge(&response[..len])
    }

    pub async fn set_max_current(&self, current: u16) -> Result<(), String> {
        let mut response = [0; 16];
        let socket = self.connect().await?;
        socket
            .send(format!("curr {}", current).as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        let len = socket
            .recv(&mut response)
            .await
            .map_err(|e| e.to_string())?;
        Self::check_acknowledge(&response[..len])
    }

    #[allow(clippy::needless_lifetimes)]
    async fn get_report<'a>(
        &self,
        command: &[u8],
        mut response: &'a mut [u8],
    ) -> Result<&'a str, String> {
        let socket = self.connect().await?;
        socket.send(command).await.map_err(|e| e.to_string())?;
        let len = socket
            .recv(&mut response)
            .await
            .map_err(|e| e.to_string())?;
        match std::str::from_utf8(&response[..len]) {
            Ok(x) => Ok(x),
            Err(e) => {
                if let Some(logger) = &self.logger {
                    debug!(
                        logger,
                        "Received invalid status report. Raw data: {:?}",
                        &response[..len]
                    );
                }
                Err(e.to_string())
            }
        }
    }

    pub async fn status_report(&self) -> Result<StatusReport, String> {
        let mut response = [0; 508];
        let str_data = self.get_report(b"report 2", &mut response).await?;
        serde_json::from_str(str_data).map_err(|e| e.to_string())
    }

    pub async fn power_report(&self) -> Result<PowerReport, String> {
        let mut response = [0; 508];
        let str_data = self.get_report(b"report 3", &mut response).await?;
        serde_json::from_str(str_data).map_err(|e| e.to_string())
    }
}

#[tokio::test]
async fn test_kecontact_client() {
    let client =
        KeContactClient::new("192.168.5.72:7090".parse().unwrap(), None);
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        client.set_charge_current(6000).await.unwrap();
        client.set_enable(true).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        eprintln!("{:?}", client.status_report().await.unwrap());
        eprintln!("{:?}", client.power_report().await.unwrap());
    })
    .await
    .unwrap();
}
