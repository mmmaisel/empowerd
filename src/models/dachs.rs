extern crate influx_db_client;
extern crate serde_json;

//use futures::future::Future;
use influx_db_client::Points;

use super::*;
use influx_derive::InfluxData;

#[derive(Debug, InfluxData)]
#[influx(measurement_name = "dachs")]
pub struct DachsData
{
    pub timestamp: i64,
    pub power: f64,
    pub runtime: f64,
    pub energy: f64
}

impl DachsData
{
    pub fn new(timestamp: i64, power: f64, runtime: f64, energy: f64)
        -> DachsData
    {
        return DachsData
        {
            timestamp: timestamp,
            power: power,
            runtime: runtime,
            energy: energy
        };
    }
}
