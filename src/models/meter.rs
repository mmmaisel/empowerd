use super::InfluxObject;
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use serde::Deserialize;

#[derive(Deserialize, Debug, InfluxDbWriteable)]
pub struct Meter {
    pub time: DateTime<Utc>,
    pub energy_consumed: f64,
    pub energy_produced: f64,
    pub power: f64,
}

impl Meter {
    pub fn new(
        time: DateTime<Utc>,
        energy_consumed: f64,
        energy_produced: f64,
        power: f64, // TODO: remove, use derivative query
    ) -> Meter {
        return Meter {
            time: time,
            energy_consumed: energy_consumed,
            energy_produced: energy_produced,
            power: power,
        };
    }
}

impl InfluxObject<Meter> for Meter {
    const FIELDS: &'static str = "energy_consumed, energy_produced, power";
}
