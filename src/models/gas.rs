extern crate influx_db_client;
extern crate serde_json;

use influx_db_client::Points;

use super::*;
use influx_derive::InfluxData;

#[derive(Debug, InfluxData)]
#[influx(measurement_name = "gas")]
pub struct GasData
{
    pub timestamp: i64,
    pub delta: f64,
    pub total: f64
}

impl GasData
{
    pub fn new(timestamp: i64, delta: f64, total: f64) -> GasData
    {
        return GasData
        {
            timestamp: timestamp,
            delta: delta,
            total: total
        };
    }
}
