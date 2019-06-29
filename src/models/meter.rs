extern crate influx_db_client;
extern crate serde_json;

use influx_db_client::Points;

use super::*;
use influx_derive::InfluxData;

#[derive(Debug, InfluxData)]
#[influx(measurement_name = "meter")]
pub struct MeterData
{
    pub timestamp: i64,
    pub power: f64,
    pub energy_produced: f64,
    pub energy_consumed: f64
}

impl MeterData
{
    pub fn new(timestamp: i64, power: f64,
        energy_produced: f64, energy_consumed: f64) -> MeterData
    {
        return MeterData
        {
            timestamp: timestamp,
            power: power,
            energy_produced: energy_produced,
            energy_consumed: energy_consumed
        };
    }
}
