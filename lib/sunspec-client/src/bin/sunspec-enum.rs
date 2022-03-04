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
use sunspec_client::SunspecClient;

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
            Arg::new("id")
                .short('i')
                .long("id")
                .help("Modbus ID for multiplexed diveces")
                .takes_value(true),
        )
        .get_matches();

    let addr: SocketAddr = match matches.value_of("address") {
        Some(x) => match x.parse() {
            Ok(x) => x,
            Err(e) => panic!("Could not parse address: {}", e),
        },
        None => panic!("Address must be given"),
    };

    let id: Option<u8> = match matches.value_of("id") {
        Some(x) => match x.parse() {
            Ok(x) => Some(x),
            Err(e) => panic!("Could not parse ID: {}", e),
        },
        None => None,
    };

    let mut client = SunspecClient::new(addr, id, None);
    let mut context = client.open().await.unwrap();
    client.introspect(&mut context).await.unwrap();

    for (model, reg) in client.models().iter() {
        println!("Model {} at {}", model, reg);
    }

    let energy = client.get_total_yield(&mut context).await.unwrap();
    println!("Total energy yield is {} Wh", energy);

    return Ok(());
}
