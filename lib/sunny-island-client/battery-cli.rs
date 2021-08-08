#![forbid(unsafe_code)]

use clap::{App, Arg};
use std::net::SocketAddr;
use tokio_modbus::client::tcp::connect_slave;
use tokio_modbus::prelude::{Reader, Writer};

const GRID_SPT_W: u16 = 40801;
const GRID_SPT_FLASH_W: u16 = 44439;

fn watt_to_modbus_s32_fix0(watt: i32) -> Vec<u16> {
    return vec![((watt >> 16) & 0xFFFF) as u16, ((watt) & 0xFFFF) as u16];
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let matches = App::new("Battery-CLI")
        .version("0.1")
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("addr")
                .help("Target IP address and port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("power")
                .long("power")
                .help("Target net power")
                .takes_value(true),
        )
        .get_matches();

    let addr = match matches.value_of("address") {
        Some(x) => x,
        None => panic!("Address must be given"),
    };
    let power = match matches.value_of("power") {
        Some(x) => x.parse::<i32>().unwrap(),
        None => panic!("Power must be given"),
    };

    let addr: SocketAddr = match addr.parse() {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };

    let mut client = match connect_slave(addr, 3.into()).await {
        Ok(x) => x,
        Err(e) => panic!("Could not connect to battery: {}", e),
    };

    if let Err(e) = client
        .write_multiple_registers(
            GRID_SPT_FLASH_W,
            &watt_to_modbus_s32_fix0(power),
        )
        .await
    {
        panic!("Write failed: {}", e);
    }

    println!(
        "Received: {:?}",
        client.read_input_registers(GRID_SPT_W, 2).await.unwrap()
    );
    return Ok(());
}
