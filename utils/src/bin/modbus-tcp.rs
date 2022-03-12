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

use clap::{ArgEnum, ArgGroup, CommandFactory, ErrorKind, Parser};
use std::net::SocketAddr;
use std::process;
use tokio_modbus::client::tcp::{connect, connect_slave};
use tokio_modbus::prelude::{Reader, Writer};

#[derive(ArgEnum, Clone, Debug, PartialEq)]
enum RegisterType {
    #[clap(alias("c"))]
    Coil,
    #[clap(alias("d"))]
    Discrete,
    #[clap(alias("i"))]
    Input,
    #[clap(alias("h"))]
    Holding,
}

#[derive(Debug, Parser)]
#[clap(group(ArgGroup::new("rw").required(true).args(&["read", "write"])))]
#[clap(name="modbus-tcp", version="0.1")]
struct Args {
    /// Device IP address and port
    #[clap(short, long)]
    device: SocketAddr,
    /// Modbus device unit ID
    #[clap(short, long)]
    unit: Option<u8>,
    /// Register address
    #[clap(short, long)]
    address: u16,
    /// Read register count
    #[clap(short, long)]
    read: Option<u16>,
    /// Write register values
    #[clap(short, long)]
    write: Option<Vec<u16>>,
    /// Register type
    #[clap(arg_enum, short, long, default_value_t=RegisterType::Holding)]
    r#type: RegisterType,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();

    if args.write.is_some()
        && (args.r#type == RegisterType::Discrete
            || args.r#type == RegisterType::Input)
    {
        Args::command()
            .error(
                ErrorKind::ArgumentConflict,
                "Can't write to input or discrete input registers.",
            )
            .exit();
    }

    let mut client = match args.unit {
        None => match connect(args.device).await {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Could not connect to device: {}", e);
                process::exit(1);
            }
        },
        Some(uid) => match connect_slave(args.device, uid.into()).await {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Could not connect to device: {}", e);
                process::exit(1);
            }
        },
    };

    if let Some(count) = args.read {
        match args.r#type {
            RegisterType::Coil => {
                match client.read_coils(args.address, count).await {
                    Ok(x) => println!("Coil {}: {:?}", args.address, x),
                    Err(e) => {
                        eprintln!("Reading coils failed: {}", e);
                        process::exit(3);
                    }
                }
            }
            RegisterType::Discrete => {
                match client.read_discrete_inputs(args.address, count).await {
                    Ok(x) => println!("Discrete {}: {:?}", args.address, x),
                    Err(e) => {
                        eprintln!("Reading discrete inputs failed: {}", e);
                        process::exit(3);
                    }
                }
            }
            RegisterType::Input => {
                match client.read_input_registers(args.address, count).await {
                    Ok(x) => println!("Input {}: {:?}", args.address, x),
                    Err(e) => {
                        eprintln!("Reading input registers failed: {}", e);
                        process::exit(3);
                    }
                }
            }
            RegisterType::Holding => {
                match client.read_holding_registers(args.address, count).await {
                    Ok(x) => println!("Holding {}: {:?}", args.address, x),
                    Err(e) => {
                        eprintln!("Reading holding registers failed: {}", e);
                        process::exit(3);
                    }
                }
            }
        }
    } else if let Some(write) = args.write {
        match args.r#type {
            RegisterType::Coil => {
                if write.len() == 1 {
                    let value = *write.first().unwrap() != 0;
                    if let Err(e) =
                        client.write_single_coil(args.address, value).await
                    {
                        eprintln!("Writing coil failed: {}", e);
                        process::exit(3);
                    } else {
                        println!("Written Coil: {}: {:?}", args.address, value);
                    }
                } else {
                    let values =
                        write.iter().map(|x| *x != 0).collect::<Vec<bool>>();
                    if let Err(e) =
                        client.write_multiple_coils(args.address, &values).await
                    {
                        eprintln!("Writing coils failed: {}", e);
                        process::exit(3);
                    } else {
                        println!(
                            "Written Coils: {}: {:?}",
                            args.address, &values
                        );
                    }
                }
            }
            RegisterType::Holding => {
                if write.len() == 1 {
                    let value = write.first().unwrap();
                    if let Err(e) =
                        client.write_single_register(args.address, *value).await
                    {
                        eprintln!("Writing register failed: {}", e);
                        process::exit(3);
                    } else {
                        println!(
                            "Written register: {}: {:?}",
                            args.address, value
                        );
                    }
                } else {
                    if let Err(e) = client
                        .write_multiple_registers(args.address, &write)
                        .await
                    {
                        eprintln!("Writing registers failed: {}", e);
                        process::exit(3);
                    } else {
                        println!(
                            "Written registers: {}: {:?}",
                            args.address, &write
                        );
                    }
                }
            }
            _ => (),
        }
    }

    Ok(())
}
