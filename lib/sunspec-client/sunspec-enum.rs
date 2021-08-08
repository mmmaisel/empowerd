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
