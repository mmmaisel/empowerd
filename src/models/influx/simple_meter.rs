/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2022 Max Maisel

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
    units::{second, watt, watt_hour, Energy, Power, Time},
    InfluxObject,
};
use chrono::{DateTime, Utc};
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawSimpleMeter {
    pub time: DateTime<Utc>,
    pub energy: f64,
    pub power: f64,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(from = "RawSimpleMeter")]
pub struct SimpleMeter {
    pub time: Time,
    pub energy: Energy,
    pub power: Power,
}

impl SimpleMeter {
    pub fn new(
        time: Time,
        energy: Energy,
        power: Power, // TODO: remove, use derivative query
    ) -> Self {
        Self {
            time,
            energy,
            power,
        }
    }

    pub fn calc_power(&self, other: &Self) -> Power {
        if self.time == other.time {
            Power::new::<watt>(0.0)
        } else {
            (self.energy - other.energy) / (self.time - other.time).abs()
        }
    }
}

impl InfluxObject<SimpleMeter> for SimpleMeter {
    const FIELDS: &'static str = "energy, power";
}

impl InfluxDbWriteable for SimpleMeter {
    fn into_query<T: Into<String>>(self, name: T) -> WriteQuery {
        Timestamp::Seconds(self.time.get::<second>() as u128)
            .into_query(name)
            .add_field("energy", self.energy.get::<watt_hour>())
            .add_field("power", self.power.get::<watt>())
    }
}

impl From<RawSimpleMeter> for SimpleMeter {
    fn from(other: RawSimpleMeter) -> Self {
        Self {
            time: Time::new::<second>(other.time.timestamp() as f64),
            energy: Energy::new::<watt_hour>(other.energy),
            power: Power::new::<watt>(other.power),
        }
    }
}
