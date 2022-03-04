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

use clap::{Arg, Command};
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
    let matches = Command::new("Battery-CLI")
        .version("0.1")
        .arg(
            Arg::new("address")
                .short('a')
                .long("addr")
                .help("Target IP address and port")
                .takes_value(true),
        )
        .arg(
            Arg::new("power")
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
