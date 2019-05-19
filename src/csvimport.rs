extern crate csv;
extern crate chrono;
extern crate influx_db_client;

use serde::Deserialize;
use chrono::prelude::*;
use influx_db_client::Client;

// TODO: fix those names
use crate::models::dachs as dachs_model;
use crate::models::solar as solar_model;

#[derive(Debug, Deserialize)]
struct CsvSolarRecord
{
    date_time: String,
    total: String,
    power: String
}

pub fn import_solar(db_url: String, db_name: String)
{
    // TODO: add DB password
    // TODO: correct error handling
    let influx_conn = Client::new(format!("http://{}", db_url), db_name);
    let mut reader = csv::ReaderBuilder::new().
        delimiter(b';').
        has_headers(false).
        from_reader(std::io::stdin());

    let data = reader.deserialize().into_iter().map(|record|
    {
        let csvrecord: CsvSolarRecord = record.expect("💩️ cant parse");
        let timestamp: i64 =
            Utc.datetime_from_str(&csvrecord.date_time, "%d.%m.%Y %H:%M:%S").
            expect("💩️ cant parse datetime").
            timestamp() as i64;
        let total: f64 = csvrecord.total.replace(",", ".").parse().expect("💩️");
        let power: f64 = csvrecord.power.replace(",", ".").parse().expect("💩️");

        return solar_model::SolarData::new(timestamp, power, total);
    }).collect();
    solar_model::SolarData::save_all(&influx_conn, data);
}

#[derive(Debug, Deserialize)]
struct CsvDachsRecord
{
    date_time: String,
    power: String,
    runtime: String,
    total: String
}

pub fn import_dachs(db_url: String, db_name: String)
{
    // TODO: add DB password
    // TODO: correct error handling
    let influx_conn = Client::new(format!("http://{}", db_url), db_name);
    let mut reader = csv::ReaderBuilder::new().
        delimiter(b';').
        has_headers(false).
        from_reader(std::io::stdin());

    let data = reader.deserialize().into_iter().map(|record|
    {
        let csvrecord: CsvDachsRecord = record.expect("💩️ cant parse");
        let timestamp: i64 =
            Utc.datetime_from_str(&csvrecord.date_time, "%d.%m.%Y %H:%M:%S").
            expect("💩️ cant parse datetime").
            timestamp() as i64;
        let power: f64 = csvrecord.power.replace(",", ".").parse().expect("💩️");
        let runtime: f64 = csvrecord.runtime.replace(",", ".").parse().expect("💩️");
        let total: f64 = csvrecord.total.replace(",", ".").parse().expect("💩️");

        return dachs_model::DachsData::new(timestamp, power, runtime, total);
    }).collect();
    dachs_model::DachsData::save_all(&influx_conn, data);
}