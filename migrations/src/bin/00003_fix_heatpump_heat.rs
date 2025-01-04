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
#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use clap::Parser;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use libempowerd::{
    models::{
        postgres::{run_migrations, Heatpump},
        units::{ratio, second, watt, watt_hour, Energy, Power, Ratio},
    },
    settings::{Settings, SourceType},
};

#[derive(Parser, Debug)]
#[command(version, long_about = "Empowerd migration to PostgreSQL")]
struct Args {
    /// empowerd config file location
    #[arg(short, long, default_value("/etc/empowerd/empowerd.conf"))]
    config: String,
    /// Timestamp of the first record to be fixed
    #[arg(short, long)]
    start_timestamp: i64,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();
    let settings = Settings::load_from_file(&args.config)
        .map_err(|e| format!("Could not read config file: {}", e))?;

    let pg_url = format!(
        "postgres://{}:{}@{}/{}",
        settings.database.user,
        settings.database.password,
        settings.database.url,
        settings.database.name,
    );
    tokio::task::block_in_place(|| run_migrations(&pg_url)).unwrap();

    let pool_cfg =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new(pg_url);
    let mut pool =
        Pool::builder(pool_cfg).build().map_err(|e| e.to_string())?;

    for source in &settings.sources {
        let result = match &source.variant {
            SourceType::LambdaHeatPump(_) => {
                migrate_heatpump(
                    &mut pool,
                    DateTime::from_timestamp(args.start_timestamp, 0).unwrap(),
                    source.series_id,
                )
                .await
            }
            _ => Ok(()),
        };

        if let Err(e) = result {
            eprintln!(
                "Migration of series '{}' finished with error: {}",
                &source.name, e
            );
        }
    }

    Ok(())
}

async fn migrate_heatpump(
    pool: &mut Pool<AsyncPgConnection>,
    mut now: DateTime<Utc>,
    series_id: i32,
) -> Result<(), String> {
    let mut postgres = pool.get().await.map_err(|e| e.to_string())?;

    let mut last_record =
        Heatpump::batch(&mut postgres, series_id, now.naive_utc(), 1)
            .await?
            .remove(0);
    now = DateTime::from_timestamp(last_record.time.get::<second>() as i64, 0)
        .unwrap();

    eprintln!("Fixing series {series_id}");
    loop {
        let batch =
            Heatpump::batch(&mut postgres, series_id, now.naive_utc(), 1000)
                .await?;
        if batch.is_empty() {
            break;
        }
        eprintln!("Fixing batch starting at {}", now.timestamp());

        let mut last_pre_heat = Energy::new::<watt_hour>(0.0);
        for mut record in batch {
            let delta_t = record.time - last_record.time;

            if record.power > Power::new::<watt>(0.0)
                && (record.heat - last_pre_heat)
                    >= Energy::new::<watt_hour>(-10.0)
            {
                last_pre_heat = record.heat;
                record.heat =
                    last_record.heat + record.power * delta_t * record.cop;
                record.defrost = last_record.defrost;
            } else {
                last_pre_heat = record.heat;
                record.heat = last_record.heat;
                record.defrost = last_record.defrost
                    + record.power.abs()
                        * delta_t
                        * record.cop.max(Ratio::new::<ratio>(1.0));
            }

            record.save_changes(&mut postgres, series_id).await?;
            last_record = record;
        }
        now = DateTime::from_timestamp(
            last_record.time.get::<second>() as i64 + 1,
            0,
        )
        .unwrap();
    }

    Ok(())
}
