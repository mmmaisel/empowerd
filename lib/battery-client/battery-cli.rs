use clap::{App, Arg};
use modbus::Client;
use std::time::Duration;

const GRID_SPT_W: u16 = 40801;
const GRID_SPT_FLASH_W: u16 = 44439;

fn watt_to_modbus_s32_fix0(watt: i32) -> Vec<u16> {
    return vec![((watt >> 16) & 0xFFFF) as u16, ((watt) & 0xFFFF) as u16];
}

fn main() {
    let matches = App::new("Battery-CLI")
        .version("0.1")
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("addr")
                .help("Target IP Address")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Destination port")
                .takes_value(true)
                .default_value("502"),
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
    let port = match matches.value_of("port") {
        Some(x) => x.parse::<u16>().unwrap(),
        None => panic!("Port must be given"),
    };
    let power = match matches.value_of("power") {
        Some(x) => x.parse::<i32>().unwrap(),
        None => panic!("Power must be given"),
    };

    let config = modbus::tcp::Config {
        tcp_port: port,
        tcp_connect_timeout: Some(Duration::from_secs(5)),
        tcp_read_timeout: Some(Duration::from_secs(10)),
        tcp_write_timeout: Some(Duration::from_secs(10)),
        modbus_uid: 3,
    };

    let mut client = match modbus::tcp::Transport::new_with_cfg(&addr, config) {
        Ok(x) => x,
        Err(e) => panic!("Setup failed {}", e),
    };

    //    if let Err(e) = client.write_multiple_registers(GRID_SPT_FLASH_W, &watt_to_modbus_s32_fix0(power)) {
    //        panic!("Write failed: {}", e);
    //    }

    println!(
        "Received: {:?}",
        client.read_input_registers(GRID_SPT_W, 2).unwrap()
    );
}
