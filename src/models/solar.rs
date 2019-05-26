extern crate influx_db_client;
extern crate serde_json;

use influx_db_client::Points;

use super::*;
use influx_derive::InfluxData;

#[derive(Debug, InfluxData)]
#[influx(measurement_name = "solar")]
pub struct SolarData
{
    pub timestamp: i64,
    pub power: f64,
    pub energy: f64
}

impl SolarData
{
    pub fn new(timestamp: i64, power: f64, energy: f64) -> SolarData
    {
        return SolarData
        {
            timestamp: timestamp,
            power: power,
            energy: energy
        };
    }
}
