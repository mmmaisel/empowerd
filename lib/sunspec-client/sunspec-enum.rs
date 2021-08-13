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

use clap::{App, Arg};
use std::net::SocketAddr;
use sunspec_client::SunspecClient;

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
        .get_matches();

    let addr = match matches.value_of("address") {
        Some(x) => x,
        None => panic!("Address must be given"),
    };

    let addr: SocketAddr = match addr.parse() {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };

    let mut client =
        SunspecClient::new(addr.ip().to_string(), addr.port(), None).unwrap();
    let mut context = client.open().await.unwrap();
    client.introspect(&mut context).await.unwrap();

    for (model, reg) in client.models().iter() {
        println!("Model {} at {}", model, reg);
    }
    return Ok(());
}
