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

    pub fn get_in_out_charge(&self) -> Result<(u32, u32, u32), String> {
        let mut client =
            modbus::tcp::Transport::new_with_cfg(&self.addr, self.config)
                .map_err(|e| e.to_string())?;
        let raw_wh_in = client
            .read_input_registers(BatteryClient::METERING_WH_IN, 2)
            .map_err(|e| e.to_string())?;
        let raw_wh_out = client
            .read_input_registers(BatteryClient::METERING_WH_OUT, 2)
            .map_err(|e| e.to_string())?;
        let raw_charge = client
            .read_input_registers(BatteryClient::BAT_CHA_STT, 2)
            .map_err(|e| e.to_string())?;

        if let Some(l) = &self.logger {
            trace!(l, "raw_wh_in: {:?}", &raw_wh_in);
            trace!(l, "raw_wh_out: {:?}", &raw_wh_out);
            trace!(l, "raw_charge: {:?}", &raw_charge);
        }

        let wh_in = (raw_wh_in[0] as u32) * 65536 + (raw_wh_in[1] as u32);
        let wh_out = (raw_wh_out[0] as u32) * 65536 + (raw_wh_out[1] as u32);
        let charge = (raw_charge[0] as u32) * 65536 + (raw_charge[1] as u32);

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
