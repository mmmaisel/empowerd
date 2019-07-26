extern crate influx_db_client;
extern crate serde_json;

use influx_db_client::Points;

use super::*;
use influx_derive::InfluxData;

#[derive(Debug, InfluxData)]
#[influx(measurement_name = "water")]
pub struct WaterData
{
    pub timestamp: i64,
    pub rate: f64,
    pub total: f64,
    pub current: f64
}

impl WaterData
{
    pub fn new(timestamp: i64, rate: f64, total: f64, current: f64)
        -> WaterData
    {
        return WaterData
        {
            timestamp: timestamp,
            rate: rate,
            total: total,
            current: current
        };
    }
}
