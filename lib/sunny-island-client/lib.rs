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

    pub async fn get_in_out_charge(&self) -> Result<(u32, u32, u32), String> {
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

        if charge == 0 {
            return Err("Received invalid value 0 for charge.".into());
        }

        return Ok((wh_in, wh_out, charge));
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
