use super::InfluxObject;
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use serde::Deserialize;

#[derive(Deserialize, Debug, InfluxDbWriteable)]
pub struct Dachs {
    pub time: DateTime<Utc>,
    pub power: f64,
    pub runtime: f64,
    pub energy: f64,
}

impl Dachs {
    pub fn new(
        time: DateTime<Utc>,
        energy: f64,
        power: f64, // TODO: remove, use derivative query
        runtime: f64,
    ) -> Dachs {
        return Dachs {
            time: time,
            energy: energy,
            power: power,
            runtime: runtime,
        };
    }
}

impl InfluxObject<Dachs> for Dachs {
    const FIELDS: &'static str = "energy, power, runtime";
    const MEASUREMENT: &'static str = "dachs";
}
