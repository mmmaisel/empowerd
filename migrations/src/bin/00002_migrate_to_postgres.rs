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

use clap::Parser;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use libempowerd::{
    error::Error,
    models::{
        influx::{
            Battery as InfluxBattery, BidirectionalMeter as InfluxBidirMeter,
            Generator as InfluxGenerator, Heatpump as InfluxHeatpump,
            InfluxObject, InfluxOrder, InfluxSeriesResult,
            SimpleMeter as InfluxSimpleMeter, Weather as InfluxWeather,
        },
        postgres::{
            run_migrations, Battery as PgBattery, BidirMeter as PgBidirMeter,
            Generator as PgGenerator, Heatpump as PgHeatpump,
            SimpleMeter as PgSimpleMeter, Weather as PgWeather,
        },
        units::second,
    },
    settings::{Settings, SourceType},
};

#[derive(Parser, Debug)]
#[command(version, long_about = "Empowerd migration to PostgreSQL")]
struct Args {
    /// empowerd config file location
    #[arg(short, long, default_value("/etc/empowerd/empowerd.conf"))]
    config: String,
    /// Continue on error
    #[arg(short, long)]
    ignore_errors: bool,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();
    let settings = Settings::load_from_file(&args.config)
        .map_err(|e| format!("Could not read config file: {}", e))?;

    let influx = match settings.influx {
        Some(influx_settings) => influxdb::Client::new(
            format!("http://{}", &influx_settings.url),
            &influx_settings.name,
        )
        .with_auth(&influx_settings.user, &influx_settings.password),
        None => {
            return Err("Missing 'influx' settings in config file".into());
        }
    };

    let pg_url = format!(
        "postgres://{}:{}@{}/{}",
        settings.database.user,
        settings.database.password,
        settings.database.url,
        settings.database.name,
    );
    tokio::task::block_in_place(|| run_migrations(&pg_url))?;

    let pool_cfg =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new(pg_url);
    let mut pool =
        Pool::builder(pool_cfg).build().map_err(|e| e.to_string())?;

    for source in &settings.sources {
        let result = match &source.variant {
            SourceType::Debug(_) => Ok(()),
            SourceType::SunnyIsland(_) | SourceType::SunnyBoyStorage(_) => {
                migrate_battery(
                    &influx,
                    &mut pool,
                    &source.name,
                    source.series_id,
                    args.ignore_errors,
                )
                .await
            }
            SourceType::SunspecSolar(_)
            | SourceType::SunnyBoySpeedwire(_)
            | SourceType::KeContact(_) => {
                migrate_simple_meter(
                    &influx,
                    &mut pool,
                    &source.name,
                    source.series_id,
                    args.ignore_errors,
                )
                .await
            }
            SourceType::DachsMsrS(_) => {
                migrate_generator(
                    &influx,
                    &mut pool,
                    &source.name,
                    source.series_id,
                    args.ignore_errors,
                )
                .await
            }
            SourceType::LambdaHeatPump(_) => {
                migrate_heatpump(
                    &influx,
                    &mut pool,
                    &source.name,
                    source.series_id,
                    args.ignore_errors,
                )
                .await
            }
            SourceType::SmaMeter(_) | SourceType::SmlMeter(_) => {
                migrate_bidir_meter(
                    &influx,
                    &mut pool,
                    &source.name,
                    source.series_id,
                    args.ignore_errors,
                )
                .await
            }
            SourceType::Bresser6in1(_) => {
                migrate_weather(
                    &influx,
                    &mut pool,
                    &source.name,
                    source.series_id,
                    args.ignore_errors,
                )
                .await
            }
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

async fn migrate_battery(
    influx: &influxdb::Client,
    pool: &mut Pool<AsyncPgConnection>,
    measurement: &str,
    series_id: i32,
    continue_on_err: bool,
) -> Result<(), String> {
    let mut now = chrono::Utc::now().timestamp() as u64 + 100000;
    let mut postgres = pool.get().await.map_err(|e| e.to_string())?;

    eprintln!("Migrating {} records...", &measurement);
    while let InfluxSeriesResult::Some(series) =
        query_influx_batch::<InfluxBattery>(influx, measurement, now, 1000)
            .await
    {
        let records = series
            .values
            .iter()
            .map(|record| {
                now = record.time.get::<second>() as u64;
                PgBattery {
                    time: record.time,
                    charge: record.charge,
                    energy_in: record.energy_in,
                    energy_out: record.energy_out,
                    power: record.power,
                }
            })
            .collect::<Vec<PgBattery>>();

        let record_count = records.len();
        let result =
            PgBattery::insert_bulk(records, &mut postgres, series_id).await;
        check_result(result, measurement, now, record_count, continue_on_err)?;
    }

    Ok(())
}

async fn migrate_simple_meter(
    influx: &influxdb::Client,
    pool: &mut Pool<AsyncPgConnection>,
    measurement: &str,
    series_id: i32,
    continue_on_err: bool,
) -> Result<(), String> {
    let mut now = chrono::Utc::now().timestamp() as u64 + 100000;
    let mut postgres = pool.get().await.map_err(|e| e.to_string())?;

    eprintln!("Migrating {} records...", &measurement);
    while let InfluxSeriesResult::Some(series) =
        query_influx_batch::<InfluxSimpleMeter>(influx, measurement, now, 1000)
            .await
    {
        let records = series
            .values
            .iter()
            .map(|record| {
                now = record.time.get::<second>() as u64;
                PgSimpleMeter {
                    time: record.time,
                    energy: record.energy,
                    power: record.power,
                }
            })
            .collect::<Vec<PgSimpleMeter>>();

        let record_count = records.len();
        let result =
            PgSimpleMeter::insert_bulk(records, &mut postgres, series_id).await;
        check_result(result, measurement, now, record_count, continue_on_err)?;
    }

    Ok(())
}

async fn migrate_generator(
    influx: &influxdb::Client,
    pool: &mut Pool<AsyncPgConnection>,
    measurement: &str,
    series_id: i32,
    continue_on_err: bool,
) -> Result<(), String> {
    let mut now = chrono::Utc::now().timestamp() as u64 + 100000;
    let mut postgres = pool.get().await.map_err(|e| e.to_string())?;

    eprintln!("Migrating {} records...", &measurement);
    while let InfluxSeriesResult::Some(series) =
        query_influx_batch::<InfluxGenerator>(influx, measurement, now, 1000)
            .await
    {
        let records = series
            .values
            .iter()
            .map(|record| {
                now = record.time.get::<second>() as u64;
                PgGenerator {
                    time: record.time,
                    energy: record.energy,
                    power: record.power,
                    runtime: record.runtime,
                }
            })
            .collect::<Vec<PgGenerator>>();

        let record_count = records.len();
        let result =
            PgGenerator::insert_bulk(records, &mut postgres, series_id).await;
        check_result(result, measurement, now, record_count, continue_on_err)?;
    }

    Ok(())
}

async fn migrate_heatpump(
    influx: &influxdb::Client,
    pool: &mut Pool<AsyncPgConnection>,
    measurement: &str,
    series_id: i32,
    continue_on_err: bool,
) -> Result<(), String> {
    let mut now = chrono::Utc::now().timestamp() as u64 + 100000;
    let mut postgres = pool.get().await.map_err(|e| e.to_string())?;

    eprintln!("Migrating {} records...", &measurement);
    while let InfluxSeriesResult::Some(series) =
        query_influx_batch::<InfluxHeatpump>(influx, measurement, now, 1000)
            .await
    {
        let records = series
            .values
            .iter()
            .map(|record| {
                now = record.time.get::<second>() as u64;
                PgHeatpump {
                    time: record.time,
                    energy: record.energy,
                    power: record.power,
                    heat: record.total_heat,
                    cop: record.cop,
                    boiler_top: record.boiler_top,
                    boiler_mid: record.boiler_mid,
                    boiler_bot: record.boiler_bot,
                }
            })
            .collect::<Vec<PgHeatpump>>();

        let record_count = records.len();
        let result =
            PgHeatpump::insert_bulk(records, &mut postgres, series_id).await;
        check_result(result, measurement, now, record_count, continue_on_err)?;
    }

    Ok(())
}

async fn migrate_bidir_meter(
    influx: &influxdb::Client,
    pool: &mut Pool<AsyncPgConnection>,
    measurement: &str,
    series_id: i32,
    continue_on_err: bool,
) -> Result<(), String> {
    let mut now = chrono::Utc::now().timestamp() as u64 + 100000;
    let mut postgres = pool.get().await.map_err(|e| e.to_string())?;

    eprintln!("Migrating {} records...", &measurement);
    while let InfluxSeriesResult::Some(series) =
        query_influx_batch::<InfluxBidirMeter>(influx, measurement, now, 1000)
            .await
    {
        let records = series
            .values
            .iter()
            .map(|record| {
                now = record.time.get::<second>() as u64;
                PgBidirMeter {
                    time: record.time,
                    energy_in: record.energy_consumed,
                    energy_out: record.energy_produced,
                    power: record.power,
                }
            })
            .collect::<Vec<PgBidirMeter>>();

        let record_count = records.len();
        let result =
            PgBidirMeter::insert_bulk(records, &mut postgres, series_id).await;
        check_result(result, measurement, now, record_count, continue_on_err)?;
    }

    Ok(())
}

async fn migrate_weather(
    influx: &influxdb::Client,
    pool: &mut Pool<AsyncPgConnection>,
    measurement: &str,
    series_id: i32,
    continue_on_err: bool,
) -> Result<(), String> {
    let mut now = chrono::Utc::now().timestamp() as u64 + 100000;
    let mut postgres = pool.get().await.map_err(|e| e.to_string())?;

    eprintln!("Migrating {} records...", &measurement);
    while let InfluxSeriesResult::Some(series) =
        query_influx_batch::<InfluxWeather>(influx, measurement, now, 1000)
            .await
    {
        let records = series
            .values
            .iter()
            .map(|record| {
                now = record.time.get::<second>() as u64;
                PgWeather {
                    time: record.time,
                    temp_in: record.temperature_in,
                    hum_in: record.humidity_in,
                    temp_out: record.temperature_out,
                    hum_out: record.humidity_out,
                    rain_day: record.rain_day,
                    rain_act: record.rain_actual,
                    wind_act: record.wind_actual,
                    wind_gust: record.wind_gust,
                    wind_dir: record.wind_dir,
                    baro_sea: record.baro_sea,
                    baro_abs: record.baro_absolute,
                    uv_index: record.uv_index,
                    dew_point: record.dew_point,
                    temp_x1: record.temperature_x1,
                    hum_x1: record.humidity_x1,
                    temp_x2: record.temperature_x2,
                    hum_x2: record.humidity_x2,
                    temp_x3: record.temperature_x3,
                    hum_x3: record.humidity_x3,
                }
            })
            .collect::<Vec<PgWeather>>();

        let record_count = records.len();
        let result =
            PgWeather::insert_bulk(records, &mut postgres, series_id).await;
        check_result(result, measurement, now, record_count, continue_on_err)?;
    }

    Ok(())
}

async fn query_influx_batch<T>(
    influx: &influxdb::Client,
    measurement: &str,
    now: u64,
    limit: u32,
) -> InfluxSeriesResult<T>
where
    T: 'static + Send + for<'de> serde::Deserialize<'de> + InfluxObject<T>,
{
    println!("Querying {} values starting at {}", &limit, &now);
    T::into_series(
        influx
            .json_query(T::query_where(
                measurement,
                &format!("time < {}", now * 1_000_000_000),
                Some(limit),
                InfluxOrder::Desc,
            ))
            .await,
    )
}

fn check_result(
    res: Result<usize, Error>,
    measurement: &str,
    now: u64,
    count: usize,
    continue_on_err: bool,
) -> Result<(), String> {
    match res {
        Ok(insert_count) => {
            eprintln!("Migrated {insert_count} records");
            if count != insert_count {
                let e = "Could not insert all records";
                if continue_on_err {
                    eprintln!("Skipping {} at {}: {e}!", &measurement, &now);
                } else {
                    return Err(e.to_string());
                }
            }
        }
        Err(e) => {
            if continue_on_err {
                eprintln!("Skipping {} at {}: {e}!", &measurement, &now);
            } else {
                return Err(e.to_string());
            }
        }
    }

    Ok(())
}
