extern crate influx_db_client;
extern crate serde_json;

use influx_db_client::Points;

use super::*;
use influx_derive::InfluxData;

#[derive(Debug, InfluxData)]
#[influx(measurement_name = "battery")]
pub struct BatteryData {
    pub timestamp: i64,
    pub energy_in: f64,
    pub energy_out: f64,
    pub charge: f64,
    pub power: f64,
}

impl BatteryData {
    pub fn new(
        timestamp: i64,
        energy_in: f64,
        energy_out: f64,
        charge: f64,
        power: f64,
    ) -> BatteryData {
        return BatteryData {
            timestamp: timestamp,
            energy_in: energy_in,
            energy_out: energy_out,
            charge: charge,
            power: power,
        };
    }
}
