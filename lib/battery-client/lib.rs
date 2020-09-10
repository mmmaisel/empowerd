use modbus::Client;
use slog::{trace, Logger};
use std::time::Duration;

pub struct BatteryClient {
    config: modbus::tcp::Config,
    addr: String,
    logger: Option<Logger>,
}

impl BatteryClient {
    const BAT_CHA_STT: u16 = 30845;
    const METERING_WH_IN: u16 = 30595;
    const METERING_WH_OUT: u16 = 30597;

    pub fn new(addr: String, logger: Option<Logger>) -> BatteryClient {
        let config = modbus::tcp::Config {
            tcp_port: 502,
            tcp_connect_timeout: Some(Duration::from_secs(5)),
            tcp_read_timeout: Some(Duration::from_secs(10)),
            tcp_write_timeout: Some(Duration::from_secs(10)),
            modbus_uid: 3,
        };

        return BatteryClient {
            config: config,
            addr: addr,
            logger: logger,
        };
    }

    fn validate_result(
        &self,
        which: &str,
        res: Result<Vec<u16>, modbus::Error>,
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

    pub fn get_in_out_charge(&self) -> Result<(u32, u32, u32), String> {
        let mut client =
            modbus::tcp::Transport::new_with_cfg(&self.addr, self.config)
                .map_err(|e| e.to_string())?;
        let wh_in = self.validate_result(
            "METERING_WH_IN",
            client.read_input_registers(BatteryClient::METERING_WH_IN, 2),
        )?;
        let wh_out = self.validate_result(
            "METERING_WH_OUT",
            client.read_input_registers(BatteryClient::METERING_WH_OUT, 2),
        )?;
        let charge = self.validate_result(
            "BAT_CHA_STT",
            client.read_input_registers(BatteryClient::BAT_CHA_STT, 2),
        )?;

        if charge == 0 {
            return Err("Received invalid value 0 for charge.".into());
        }

        return Ok((wh_in, wh_out, charge));
    }
}

#[test]
fn test_battery_client() {
    let client = BatteryClient {
        config: modbus::tcp::Config {
            tcp_port: 1502,
            tcp_connect_timeout: Some(Duration::from_secs(5)),
            tcp_read_timeout: Some(Duration::from_secs(10)),
            tcp_write_timeout: Some(Duration::from_secs(10)),
            modbus_uid: 3,
        },
        addr: "127.0.0.1".into(),
        logger: None,
    };

    match client.get_in_out_charge() {
        Ok((wh_in, wh_out, charge)) => {
            println!("in: {}, out: {}, charge: {}", wh_in, wh_out, charge);
        }
        Err(e) => panic!("get_in_out_charge failed: {}", e),
    }
}
