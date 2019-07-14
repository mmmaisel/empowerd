extern crate csv;
extern crate chrono;
extern crate sma_client;

use serde::Deserialize;
use chrono::prelude::*;

use sma_client::TimestampedInt;
use crate::miner::StromMiner;

#[derive(Debug, Deserialize)]
struct CsvSolarRecord
{
    date_time: String,
    energy: String,
    power: Option<String>
}

pub fn import_solar(miner: &StromMiner) -> Result<(), String>
{
    let mut reader = csv::ReaderBuilder::new().
        delimiter(b';').
        has_headers(false).
        from_reader(std::io::stdin());

    let data: Result<Vec<TimestampedInt>, String> =
        reader.deserialize().into_iter().map(|record|
    {
        let csvrecord: CsvSolarRecord = match record
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse csv, {}", e))
        };
        let timestamp: u32 = match
            Utc.datetime_from_str(&csvrecord.date_time, "%d.%m.%Y %H:%M:%S")
        {
            Ok(x) => x.timestamp() as u32,
            Err(e) => return Err(format!("Can't parse timestamp, {}", e))
        };
        let energy: u32 = match csvrecord.energy.replace(",", "").parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse energy, {}", e))
        };

        return Ok(TimestampedInt { timestamp: timestamp, value: energy } );
    }).collect();

    match data
    {
        Ok(x) => miner.save_solar_data(x, None),
        Err(e) => return Err(e)
    }
    return Ok(());
}

#[derive(Debug, Deserialize)]
struct CsvDachsRecord
{
    date_time: String,
    runtime: String,
    energy: String
}

pub fn import_dachs(miner: &StromMiner) -> Result<(), String>
{
    let mut reader = csv::ReaderBuilder::new().
        delimiter(b';').
        has_headers(false).
        from_reader(std::io::stdin());

    for record in reader.deserialize().into_iter()
    {
        let csvrecord: CsvDachsRecord = match record
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse csv, {}", e))
        };
        let timestamp: i64 = match
            Utc.datetime_from_str(&csvrecord.date_time, "%d.%m.%Y %H:%M:%S")
        {
            Ok(x) => x.timestamp() as i64,
            Err(e) => return Err(format!("Can't parse timestamp, {}", e))
        };
        let energy: f64 = match csvrecord.energy.parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse energy, {}", e))
        };
        let runtime: f64 = match csvrecord.runtime.parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse runtime, {}", e))
        };
        miner.save_dachs_data(timestamp, runtime, energy);
    };
    return Ok(());
}

/*#[derive(Debug, Deserialize)]
struct CsvWaterRecord
{
    date_time: String,
    total: String
}

// TODO: this will not connect to production db, also above
pub fn import_water(db_url: String, db_name: String)
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
        let csvrecord: CsvWaterRecord = record.expect("💩️ cant parse");
        let timestamp: i64 =
            Utc.datetime_from_str(&csvrecord.date_time, "%d.%m.%Y %H:%M:%S").
            expect("💩️ cant parse datetime").
            timestamp() as i64;
        let total: f64 = csvrecord.total.replace(",", ".").parse().expect("💩️");

        // TODO: calc delta

        return WaterData::new(timestamp, delta, total);
    }).collect();
    // TODO: calc delta, one after one, no save all
/*    if let Err(e) = SolarData::save_all(&influx_conn, data)
    {
        // TODO: use logger
        println!("Save DachsData failed, {}", e);
    }*/
}*/
