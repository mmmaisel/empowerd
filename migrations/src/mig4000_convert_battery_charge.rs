/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2025 Max Maisel

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
use clap::Args;
use libempowerd::{
    models::{
        influx::{Battery, InfluxObject, InfluxOrder, InfluxSeriesResult},
        units::watt_hour,
    },
    settings::Settings,
};

#[derive(Args, Clone, Debug)]
pub struct Mig4000Args {
    /// First timestamp of the data to migrate
    #[clap(long)]
    from: u64,
    /// Last timestamp of the data to migrate
    #[clap(long)]
    to: u64,
    /// Capacity of the battery in Wh
    #[clap(long)]
    capacity: f64,
    /// Name of the measurement to migrate
    #[clap(short, long)]
    measurement: String,
}

async fn migrate_batch(
    influx: &influxdb::Client,
    measurement: &str,
    capacity: f64,
    past: u64,
    now: u64,
) -> Result<(), String> {
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
            return Ok(());
        }
        InfluxSeriesResult::Err(e) => return Err(format!("Query failed: {e}")),
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
            return Err(format!("Save BatteryData failed, {e}"));
        }
    }
    println!("Migrated {} records", records);
    if skipped != 0 {
        println!("Skipped {} records", skipped);
    }

    Ok(())
}

pub async fn mig4000_convert_battery_charge(
    settings: Settings,
    args: Mig4000Args,
) -> Result<(), String> {
    let influx = influxdb::Client::new(
        format!("http://{}", &settings.database.url),
        &settings.database.name,
    )
    .with_auth(&settings.database.user, &settings.database.password);

    let mut past = args.from;
    for now in (args.from..=args.to).step_by(86400) {
        if past == now {
            continue;
        }
        migrate_batch(&influx, &args.measurement, args.capacity, past, now)
            .await?;
        past = now;
    }
    migrate_batch(&influx, &args.measurement, args.capacity, past, args.to)
        .await?;

    Ok(())
}
