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

// TODO: deduplicate all those functions
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
            Err(e) => return Err(format!("Can't parse timestamp {}, {}",
                csvrecord.date_time, e))
        };
        let energy: u32 = match csvrecord.energy.replace(",", "").parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse energy {}, {}",
                csvrecord.energy, e))
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

// TODO: deduplicate all those functions
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
            Err(e) => return Err(format!("Can't parse timestamp {}, {}",
                csvrecord.date_time, e))
        };
        let energy: f64 = match csvrecord.energy.parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse energy {}, {}",
                csvrecord.energy, e))
        };
        let runtime: f64 = match csvrecord.runtime.parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse runtime {}, {}",
                csvrecord.runtime, e))
        };
        miner.save_dachs_data(timestamp, runtime, energy);
    };
    return Ok(());
}

#[derive(Debug, Deserialize)]
struct CsvMeterRecord
{
    date_time: String,
    consumed: String,
    produced: String
}

// TODO: deduplicate all those functions
pub fn import_meter(miner: &StromMiner) -> Result<(), String>
{
    let mut reader = csv::ReaderBuilder::new().
        delimiter(b';').
        has_headers(false).
        from_reader(std::io::stdin());

    for record in reader.deserialize().into_iter()
    {
        let csvrecord: CsvMeterRecord = match record
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse csv, {}", e))
        };
        let timestamp: i64 = match
            Utc.datetime_from_str(&csvrecord.date_time, "%d.%m.%Y %H:%M:%S")
        {
            Ok(x) => x.timestamp() as i64,
            Err(e) => return Err(format!("Can't parse timestamp {}, {}",
                csvrecord.date_time, e))
        };
        let consumed: f64 = match csvrecord.consumed.parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse consumed {}, {}",
                csvrecord.consumed, e))
        };
        let produced: f64 = match csvrecord.produced.parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse produced {}, {}",
                csvrecord.produced, e))
        };
        miner.save_meter_data(timestamp, produced, consumed);
    };
    return Ok(());
}

#[derive(Debug, Deserialize)]
struct CsvWaterRecord
{
    date_time: String,
    current: String,
    reset: String
}

// TODO: deduplicate all those functions
pub fn import_water(miner: &StromMiner) -> Result<(), String>
{
    let mut reader = csv::ReaderBuilder::new().
        delimiter(b';').
        has_headers(false).
        from_reader(std::io::stdin());

    for record in reader.deserialize().into_iter()
    {
        let csvrecord: CsvWaterRecord = match record
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse csv, {}", e))
        };
        let timestamp: i64 = match
            Utc.datetime_from_str(&csvrecord.date_time, "%d.%m.%Y %H:%M:%S")
        {
            Ok(x) => x.timestamp() as i64,
            Err(e) => return Err(format!("Can't parse timestamp {}, {}",
                csvrecord.date_time, e))
        };
        let current: f64 = match csvrecord.current.parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse current {}, {}",
                csvrecord.current, e))
        };
        let reset = csvrecord.reset == "true";
        // TODO: ALL: allow batch jobs here, only read last once
        miner.save_water_data(timestamp, current, reset);
    };
    return Ok(());
}

#[derive(Debug, Deserialize)]
struct CsvGasRecord
{
    date_time: String,
    current: String,
    reset: String
}

// TODO: deduplicate all those functions
pub fn import_gas(miner: &StromMiner) -> Result<(), String>
{
    let mut reader = csv::ReaderBuilder::new().
        delimiter(b';').
        has_headers(false).
        from_reader(std::io::stdin());

    for record in reader.deserialize().into_iter()
    {
        let csvrecord: CsvGasRecord = match record
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse csv, {}", e))
        };
        let timestamp: i64 = match
            Utc.datetime_from_str(&csvrecord.date_time, "%d.%m.%Y %H:%M:%S")
        {
            Ok(x) => x.timestamp() as i64,
            Err(e) => return Err(format!("Can't parse timestamp {}, {}",
                csvrecord.date_time, e))
        };
        let current: f64 = match csvrecord.current.parse()
        {
            Ok(x) => x,
            Err(e) => return Err(format!("Can't parse current {}, {}",
                csvrecord.current, e))
        };
        let reset = csvrecord.reset == "true";
        miner.save_gas_data(timestamp, current, reset);
    };
    return Ok(());
}
