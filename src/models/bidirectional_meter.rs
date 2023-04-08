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
    units::{second, watt, watt_hour, Energy, Power, Time},
    InfluxObject,
};
use chrono::{DateTime, Utc};
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawBidirectionalMeter {
    pub time: DateTime<Utc>,
    pub energy_consumed: f64,
    pub energy_produced: f64,
    pub power: f64,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(from = "RawBidirectionalMeter")]
pub struct BidirectionalMeter {
    pub time: Time,
    pub energy_consumed: Energy,
    pub energy_produced: Energy,
    pub power: Power,
}

impl BidirectionalMeter {
    pub fn new(
        time: Time,
        energy_consumed: Energy,
        energy_produced: Energy,
        power: Power, // TODO: remove, use derivative query
    ) -> Self {
        Self {
            time,
            energy_consumed,
            energy_produced,
            power,
        }
    }
}

impl InfluxObject<BidirectionalMeter> for BidirectionalMeter {
    const FIELDS: &'static str = "energy_consumed, energy_produced, power";
}

impl InfluxDbWriteable for BidirectionalMeter {
    fn into_query<T: Into<String>>(self, name: T) -> WriteQuery {
        Timestamp::Seconds(self.time.get::<second>() as u128)
            .into_query(name)
            .add_field(
                "energy_consumed",
                self.energy_consumed.get::<watt_hour>(),
            )
            .add_field(
                "energy_produced",
                self.energy_produced.get::<watt_hour>(),
            )
            .add_field("power", self.power.get::<watt>())
    }
}

impl From<RawBidirectionalMeter> for BidirectionalMeter {
    fn from(other: RawBidirectionalMeter) -> Self {
        Self {
            time: Time::new::<second>(other.time.timestamp() as f64),
            energy_consumed: Energy::new::<watt_hour>(other.energy_consumed),
            energy_produced: Energy::new::<watt_hour>(other.energy_produced),
            power: Power::new::<watt>(other.power),
        }
    }
}
