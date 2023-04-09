/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
use super::{
    units::{kilowatt_hour, second, watt, Energy, Power, Time},
    InfluxObject,
};
use chrono::{DateTime, Utc};
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawGenerator {
    pub time: DateTime<Utc>,
    pub power: f64,
    pub runtime: f64,
    pub energy: f64,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(from = "RawGenerator")]
pub struct Generator {
    pub time: Time,
    pub power: Power,
    pub runtime: Time,
    pub energy: Energy,
}

impl Generator {
    pub fn new(
        time: Time,
        energy: Energy,
        power: Power, // TODO: remove, use derivative query
        runtime: Time,
    ) -> Self {
        Self {
            time,
            power,
            runtime,
            energy,
        }
    }
}

impl InfluxObject<Generator> for Generator {
    const FIELDS: &'static str = "energy, power, runtime";
}

impl InfluxDbWriteable for Generator {
    fn into_query<T: Into<String>>(self, name: T) -> WriteQuery {
        Timestamp::Seconds(self.time.get::<second>() as u128)
            .into_query(name)
            .add_field("power", self.power.get::<watt>())
            .add_field("runtime", self.runtime.get::<second>())
            .add_field("energy", self.energy.get::<kilowatt_hour>())
    }
}

impl From<RawGenerator> for Generator {
    fn from(other: RawGenerator) -> Self {
        Self {
            time: Time::new::<second>(other.time.timestamp() as f64),
            power: Power::new::<watt>(other.power),
            energy: Energy::new::<kilowatt_hour>(other.energy),
            runtime: Time::new::<second>(other.runtime),
        }
    }
}
