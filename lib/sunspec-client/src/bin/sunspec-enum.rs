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
                .required(true)
                .help("Target IP address and port"),
        )
        .arg(
            Arg::new("id")
                .short('i')
                .long("id")
                .help("Modbus ID for multiplexed diveces")
                .value_parser(clap::value_parser!(u8)),
        )
        .get_matches();

    let addr: SocketAddr =
        match matches.get_one::<String>("address").unwrap().parse() {
            Ok(x) => x,
            Err(e) => panic!("Could not parse address: {}", e),
        };

    let id: Option<u8> = matches.get_one::<u8>("id").map(|x| *x);

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
