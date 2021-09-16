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
use empowerd::{
    models::Battery, settings::Settings, InfluxObject, InfluxSeriesResult,
};

#[tokio::main]
async fn main() -> () {
    let matches = App::new("Empowerd Migration: convert battery charge")
        .version("0.3.2")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("empowerd config file location")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("from")
                .long("from")
                .help("first timestamp of the data to migrate")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("to")
                .long("to")
                .help("last timestamp of the data to migrate")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("capacity")
                .long("capacity")
                .help("capacity of the battery in Wh")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("measurement")
                .short("m")
                .long("measurement")
                .help("name of the measurement to migrate")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let from = match matches.value_of("from").unwrap().parse::<u64>() {
        Ok(x) => x,
        Err(e) => panic!("Could not parse argument 'from' as int: {}", e),
    };
    let to = match matches.value_of("to").unwrap().parse::<u64>() {
        Ok(x) => x,
        Err(e) => panic!("Could not parse argument 'to' as int: {}", e),
    };
    let capacity = match matches.value_of("capacity").unwrap().parse::<f64>() {
        Ok(x) => x,
        Err(e) => panic!("Could not parse argument 'capacity' as float: {}", e),
    };
    let measurement = matches.value_of("measurement").unwrap();

    let config_filename = matches
        .value_of("config")
        .or(Some("/etc/empowerd/empowerd.conf"))
        .unwrap();
    let settings = match Settings::load_from_file(config_filename.into()) {
        Ok(x) => x,
        Err(e) => panic!("Could not read config file: {}", e),
    };

    let influx = influxdb::Client::new(
        format!("http://{}", &settings.database.url),
        &settings.database.name,
    )
    .with_auth(&settings.database.user, &settings.database.password);

    let mut past = from;
    for now in (from..to).step_by(86400) {
        if past == now {
            continue;
        }

        println!("Querying from {} to {}", &past, &now);
        let series = match Battery::into_series(
            influx
                .json_query(Battery::query_where(
                    measurement,
                    &format!(
                        "time >= {} AND time < {}",
                        past * 1000_000_000,
                        now * 1000_000_000
                    ),
                ))
                .await,
        ) {
            InfluxSeriesResult::Some(x) => x,
            InfluxSeriesResult::None => {
                println!("Query returned no series");
                continue;
            }
            InfluxSeriesResult::Err(e) => panic!("Query failed: {}", e),
        };

        let mut records = series.values.len();
        let mut skipped = 0;
        for mut value in &mut series.values.into_iter() {
            if value.charge > 1.0 {
                skipped += 1;
                records -= 1;
                continue; // Already migrated
            }
            value.charge = value.charge * capacity;

            if let Err(e) = influx.query(&value.save_query(&measurement)).await
            {
                panic!("Save BatteryData failed, {}", e);
            }
        }
        println!("Migrated {} records", records);
        if skipped != 0 {
            println!("Skipped {} records", skipped);
        }
        past = now;
    }
}
