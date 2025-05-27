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

use std::path::PathBuf;

use clap::{Arg, Command};
use libempowerd::{
    models::{
        influx::{Battery, InfluxObject, InfluxOrder, InfluxSeriesResult},
        units::watt_hour,
    },
    settings::Settings,
};

async fn migrate_batch(
    influx: &influxdb::Client,
    measurement: &str,
    capacity: f64,
    past: u64,
    now: u64,
) {
    println!("Querying from {} to {}", &past, &now);
    let series = match Battery::into_series(
        influx
            .json_query(Battery::query_where(
                measurement,
                &format!(
                    "time >= {} AND time < {}",
                    past * 1_000_000_000,
                    now * 1_000_000_000
                ),
                None,
                InfluxOrder::Asc,
            ))
            .await,
    ) {
        InfluxSeriesResult::Some(x) => x,
        InfluxSeriesResult::None => {
            println!("Query returned no series");
            return;
        }
        InfluxSeriesResult::Err(e) => panic!("Query failed: {}", e),
    };

    let mut records = series.values.len();
    let mut skipped = 0;
    for mut value in &mut series.values.into_iter() {
        if value.charge.get::<watt_hour>() > 1.0 {
            skipped += 1;
            records -= 1;
            continue; // Already migrated
        }
        value.charge *= capacity;

        if let Err(e) = influx.query(&value.save_query(measurement)).await {
            panic!("Save BatteryData failed, {}", e);
        }
    }
    println!("Migrated {} records", records);
    if skipped != 0 {
        println!("Skipped {} records", skipped);
    }
}

#[tokio::main]
async fn main() {
    let matches = Command::new("Empowerd Migration: convert battery charge")
        .version("0.3.2")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("empowerd config file location")
                .default_value("/etc/empowerd/empowerd.conf"),
        )
        .arg(
            Arg::new("from")
                .long("from")
                .help("first timestamp of the data to migrate")
                .required(true)
                .value_parser(clap::value_parser!(u64).range(0..)),
        )
        .arg(
            Arg::new("to")
                .long("to")
                .help("last timestamp of the data to migrate")
                .required(true)
                .value_parser(clap::value_parser!(u64).range(0..)),
        )
        .arg(
            Arg::new("capacity")
                .long("capacity")
                .help("capacity of the battery in Wh")
                .required(true)
                .value_parser(clap::value_parser!(f64)),
        )
        .arg(
            Arg::new("measurement")
                .short('m')
                .long("measurement")
                .help("name of the measurement to migrate")
                .required(true),
        )
        .get_matches();

    let from = *matches.get_one::<u64>("from").unwrap();
    let to = *matches.get_one::<u64>("to").unwrap();
    let capacity = *matches.get_one::<f64>("capacity").unwrap();
    let measurement = matches.get_one::<String>("measurement").unwrap();

    let config_filename = matches.get_one::<PathBuf>("config").unwrap();
    let settings = match Settings::load_from_file(config_filename) {
        Ok(x) => x,
        Err(e) => panic!("Could not read config file: {}", e),
    };

    let influx = influxdb::Client::new(
        format!("http://{}", &settings.database.url),
        &settings.database.name,
    )
    .with_auth(&settings.database.user, &settings.database.password);

    let mut past = from;
    for now in (from..=to).step_by(86400) {
        if past == now {
            continue;
        }
        migrate_batch(&influx, measurement, capacity, past, now).await;
        past = now;
    }
    migrate_batch(&influx, measurement, capacity, past, to).await;
}
