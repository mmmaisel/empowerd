use super::InfluxObject;
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use serde::Deserialize;

#[derive(Deserialize, Debug, InfluxDbWriteable)]
pub struct Battery {
    pub time: DateTime<Utc>,
    pub charge: f64,
    pub energy_in: f64,
    pub energy_out: f64,
    pub power: f64,
}

impl Battery {
    pub fn new(
        time: DateTime<Utc>,
        charge: f64,
        energy_in: f64,
        energy_out: f64,
        power: f64, // TODO: remove, use derivative query
    ) -> Battery {
        return Battery {
            time: time,
            charge: charge,
            energy_in: energy_in,
            energy_out: energy_out,
            power: power,
        };
    }
}

impl InfluxObject<Battery> for Battery {
    const FIELDS: &'static str = "charge, energy_in, energy_out, power";
}
