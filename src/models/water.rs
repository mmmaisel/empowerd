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
    pub delta: f64,
    pub total: f64
}

impl WaterData
{
    pub fn new(timestamp: i64, delta: f64, total: f64) -> WaterData
    {
        return WaterData
        {
            timestamp: timestamp,
            delta: delta,
            total: total
        };
    }
}
