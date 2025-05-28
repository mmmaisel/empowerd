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
            Generator as PgGenerator, SimpleMeter as PgSimpleMeter,
        },
        units::second,
    },
    settings::{Settings, SourceType},
};

mod schema_9000 {
    use chrono::{DateTime, NaiveDateTime};
    use diesel::prelude::{
        AsChangeset, Identifiable, Insertable, Queryable, Selectable,
    };
    use libempowerd::{
        error::Error,
        models::units::{
            celsius, degree, micrometer, millimeter_per_second, pascal,
            percent, ratio, second, watt, watt_hour, Abbreviation, Angle,
            Energy, Length, Power, Pressure, Ratio, Temperature, Time,
            Velocity,
        },
    };
    use std::convert::TryFrom;

    diesel::table! {
        heatpumps (series_id, time) {
            series_id -> Int4,
            time -> Timestamp,
            energy_wh -> Int8,
            power_w -> Int4,
            heat_wh -> Nullable<Int8>,
            cop_pct -> Nullable<Int2>,
            boiler_top_degc_e1 -> Nullable<Int2>,
            boiler_mid_degc_e1 -> Nullable<Int2>,
            boiler_bot_degc_e1 -> Nullable<Int2>,
        }
    }

    #[derive(AsChangeset, Identifiable, Insertable, Queryable, Selectable)]
    #[diesel(table_name = heatpumps)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(primary_key(series_id, time))]
    pub struct RawPgHeatpump {
        pub series_id: i32,
        pub time: NaiveDateTime,
        pub energy_wh: i64,
        pub power_w: i32,
        pub heat_wh: Option<i64>,
        pub cop_pct: Option<i16>,
        pub boiler_top_degc_e1: Option<i16>,
        pub boiler_mid_degc_e1: Option<i16>,
        pub boiler_bot_degc_e1: Option<i16>,
    }

    #[derive(Clone, Debug)]
    pub struct PgHeatpump {
        pub time: Time,
        pub energy: Energy,
        pub power: Power,
        pub heat: Option<Energy>,
        pub cop: Option<Ratio>,
        pub boiler_top: Option<Temperature>,
        pub boiler_mid: Option<Temperature>,
        pub boiler_bot: Option<Temperature>,
    }

    impl PgHeatpump {
        pub async fn insert_bulk(
            data: Vec<Self>,
            conn: &mut diesel_async::AsyncPgConnection,
            series_id: i32,
        ) -> Result<usize, super::Error> {
            use diesel_async::RunQueryDsl;
            let raw = data
                .iter()
                .map(|x| {
                    let mut y = RawPgHeatpump::try_from(x)?;
                    y.series_id = series_id;
                    Ok(y)
                })
                .collect::<Result<Vec<RawPgHeatpump>, super::Error>>()?;

            diesel::insert_into(heatpumps::table)
                .values(&raw)
                .on_conflict_do_nothing()
                .execute(conn)
                .await
                .map_err(|e| {
                    Error::Temporary(format!(
                        "Inserting record into series {series_id} failed: {e}",
                    ))
                })
        }
    }

    impl TryFrom<&PgHeatpump> for RawPgHeatpump {
        type Error = Error;
        fn try_from(input: &PgHeatpump) -> Result<Self, Self::Error> {
            Ok(Self {
                series_id: 0,
                time: DateTime::from_timestamp(
                    input.time.get::<second>() as i64,
                    0,
                )
                .ok_or_else(|| {
                    Error::InvalidInput(format!(
                        "Invalid timestamp: {:?}",
                        input.time.into_format_args(second, Abbreviation),
                    ))
                })?
                .naive_utc(),
                energy_wh: input.energy.get::<watt_hour>().round() as i64,
                power_w: input.power.get::<watt>().round() as i32,
                heat_wh: input
                    .heat
                    .map(|x| x.get::<watt_hour>().round() as i64),
                cop_pct: input.cop.map(|x| x.get::<percent>().round() as i16),
                boiler_top_degc_e1: input
                    .boiler_top
                    .map(|x| (x.get::<celsius>() * 10.0).round() as i16),
                boiler_mid_degc_e1: input
                    .boiler_mid
                    .map(|x| (x.get::<celsius>() * 10.0).round() as i16),
                boiler_bot_degc_e1: input
                    .boiler_bot
                    .map(|x| (x.get::<celsius>() * 10.0).round() as i16),
            })
        }
    }

    diesel::table! {
        weathers (series_id, time) {
            series_id -> Int4,
            time -> Timestamp,
            temp_in_degc_e1 -> Int2,
            hum_in_e3 -> Int2,
            temp_out_degc_e1 -> Nullable<Int2>,
            hum_out_e3 -> Nullable<Int2>,
            rain_day_um ->Int4,
            rain_act_um -> Int4,
            wind_act_mms -> Int4,
            wind_gust_mms -> Int4,
            wind_dir_deg_e1 -> Int2,
            baro_sea_pa -> Int4,
            baro_abs_pa -> Int4,
            uv_index_e1 -> Int2,
            dew_point_degc_e1 -> Nullable<Int2>,
            temp_x1_degc_e1 -> Nullable<Int2>,
            hum_x1_e3 -> Nullable<Int2>,
            temp_x2_degc_e1 -> Nullable<Int2>,
            hum_x2_e3 -> Nullable<Int2>,
            temp_x3_degc_e1 -> Nullable<Int2>,
            hum_x3_e3 -> Nullable<Int2>,
        }
    }

    #[derive(AsChangeset, Identifiable, Insertable, Queryable, Selectable)]
    #[diesel(table_name = weathers)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(primary_key(series_id, time))]
    pub struct RawPgWeather {
        pub series_id: i32,
        pub time: NaiveDateTime,
        pub temp_in_degc_e1: i16,
        pub hum_in_e3: i16,
        pub temp_out_degc_e1: Option<i16>,
        pub hum_out_e3: Option<i16>,
        pub rain_day_um: i32,
        pub rain_act_um: i32,
        pub wind_act_mms: i32,
        pub wind_gust_mms: i32,
        pub wind_dir_deg_e1: i16,
        pub baro_sea_pa: i32,
        pub baro_abs_pa: i32,
        pub uv_index_e1: i16,
        pub dew_point_degc_e1: Option<i16>,
        pub temp_x1_degc_e1: Option<i16>,
        pub hum_x1_e3: Option<i16>,
        pub temp_x2_degc_e1: Option<i16>,
        pub hum_x2_e3: Option<i16>,
        pub temp_x3_degc_e1: Option<i16>,
        pub hum_x3_e3: Option<i16>,
    }

    #[derive(Clone, Debug)]
    pub struct PgWeather {
        pub time: Time,
        pub temp_in: Temperature,
        pub hum_in: Ratio,
        pub temp_out: Option<Temperature>,
        pub hum_out: Option<Ratio>,
        pub rain_day: Length,
        pub rain_act: Length,
        pub wind_act: Velocity,
        pub wind_gust: Velocity,
        pub wind_dir: Angle,
        pub baro_sea: Pressure,
        pub baro_abs: Pressure,
        pub uv_index: Ratio,
        pub dew_point: Option<Temperature>,
        pub temp_x1: Option<Temperature>,
        pub hum_x1: Option<Ratio>,
        pub temp_x2: Option<Temperature>,
        pub hum_x2: Option<Ratio>,
        pub temp_x3: Option<Temperature>,
        pub hum_x3: Option<Ratio>,
    }

    impl PgWeather {
        pub async fn insert_bulk(
            data: Vec<Self>,
            conn: &mut diesel_async::AsyncPgConnection,
            series_id: i32,
        ) -> Result<usize, super::Error> {
            use diesel_async::RunQueryDsl;
            let raw = data
                .iter()
                .map(|x| {
                    let mut y = RawPgWeather::try_from(x)?;
                    y.series_id = series_id;
                    Ok(y)
                })
                .collect::<Result<Vec<RawPgWeather>, super::Error>>()?;

            diesel::insert_into(weathers::table)
                .values(&raw)
                .on_conflict_do_nothing()
                .execute(conn)
                .await
                .map_err(|e| {
                    Error::Temporary(format!(
                        "Inserting record into series {series_id} failed: {e}",
                    ))
                })
        }
    }

    impl TryFrom<&PgWeather> for RawPgWeather {
        type Error = Error;
        fn try_from(input: &PgWeather) -> Result<Self, Self::Error> {
            Ok(Self {
                series_id: 0,
                time: DateTime::from_timestamp(
                    input.time.get::<second>() as i64,
                    0,
                )
                .ok_or_else(|| {
                    Error::InvalidInput(format!(
                        "Invalid timestamp: {:?}",
                        input.time.into_format_args(second, Abbreviation),
                    ))
                })?
                .naive_utc(),
                temp_in_degc_e1: (input.temp_in.get::<celsius>() * 1e1).round()
                    as i16,
                hum_in_e3: (input.hum_in.get::<percent>() * 1e1).round() as i16,
                temp_out_degc_e1: input
                    .temp_out
                    .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
                hum_out_e3: input
                    .hum_out
                    .map(|x| (x.get::<percent>() * 1e1).round() as i16),
                rain_day_um: input.rain_day.get::<micrometer>().round() as i32,
                rain_act_um: input.rain_act.get::<micrometer>().round() as i32,
                wind_act_mms: input
                    .wind_act
                    .get::<millimeter_per_second>()
                    .round() as i32,
                wind_gust_mms: input
                    .wind_gust
                    .get::<millimeter_per_second>()
                    .round() as i32,
                wind_dir_deg_e1: (input.wind_dir.get::<degree>() * 1e1).round()
                    as i16,
                baro_sea_pa: input.baro_sea.get::<pascal>().round() as i32,
                baro_abs_pa: input.baro_abs.get::<pascal>().round() as i32,
                uv_index_e1: (input.uv_index.get::<ratio>() * 1e1).round()
                    as i16,
                dew_point_degc_e1: input
                    .dew_point
                    .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
                temp_x1_degc_e1: input
                    .temp_x1
                    .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
                hum_x1_e3: input
                    .hum_x1
                    .map(|x| (x.get::<percent>() * 1e1).round() as i16),
                temp_x2_degc_e1: input
                    .temp_x2
                    .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
                hum_x2_e3: input
                    .hum_x2
                    .map(|x| (x.get::<percent>() * 1e1).round() as i16),
                temp_x3_degc_e1: input
                    .temp_x3
                    .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
                hum_x3_e3: input
                    .hum_x3
                    .map(|x| (x.get::<percent>() * 1e1).round() as i16),
            })
        }
    }
}

use schema_9000::{PgHeatpump, PgWeather};

#[derive(Args, Clone, Debug)]
pub struct Mig9000Args {
    /// Continue on error
    #[clap(short, long)]
    ignore_errors: bool,
}

pub async fn mig9000_to_postgres(
    settings: Settings,
    args: Mig9000Args,
) -> Result<(), String> {
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
                    rain_day: record.rain_day.unwrap_or_default(),
                    rain_act: record.rain_actual.unwrap_or_default(),
                    wind_act: record.wind_actual.unwrap_or_default(),
                    wind_gust: record.wind_gust.unwrap_or_default(),
                    wind_dir: record.wind_dir.unwrap_or_default(),
                    baro_sea: record.baro_sea,
                    baro_abs: record.baro_absolute,
                    uv_index: record.uv_index.unwrap_or_default(),
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
