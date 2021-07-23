use super::InfluxObject;
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use serde::Deserialize;

#[derive(Deserialize, Debug, InfluxDbWriteable)]
pub struct Solar {
    pub time: DateTime<Utc>,
    pub energy: f64,
    pub power: f64,
}

impl Solar {
    pub fn new(
        time: DateTime<Utc>,
        energy: f64,
        power: f64, // TODO: remove, use derivative query
    ) -> Solar {
        return Solar {
            time: time,
            energy: energy,
            power: power,
        };
    }
}

impl InfluxObject<Solar> for Solar {
    const FIELDS: &'static str = "energy, power";
}
