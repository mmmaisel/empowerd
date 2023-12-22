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
struct RawBattery {
    pub time: DateTime<Utc>,
    pub charge: f64,
    pub energy_in: f64,
    pub energy_out: f64,
    pub power: f64,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(from = "RawBattery")]
pub struct Battery {
    pub time: Time,
    pub charge: Energy,
    pub energy_in: Energy,
    pub energy_out: Energy,
    pub power: Power,
}

impl Battery {
    pub fn new(
        time: Time,
        charge: Energy,
        energy_in: Energy,
        energy_out: Energy,
        power: Power, // TODO: remove, use derivative query
    ) -> Self {
        Self {
            time,
            charge,
            energy_in,
            energy_out,
            power,
        }
    }

    pub fn calc_power(&self, other: &Self) -> Power {
        if self.time == other.time {
            Power::new::<watt>(0.0)
        } else {
            (self.energy_in
                - other.energy_in
                - (self.energy_out - other.energy_out))
                / (self.time - other.time).abs()
        }
    }
}

impl InfluxObject<Battery> for Battery {
    const FIELDS: &'static str = "charge, energy_in, energy_out, power";
}

impl InfluxDbWriteable for Battery {
    fn into_query<T: Into<String>>(self, name: T) -> WriteQuery {
        Timestamp::Seconds(self.time.get::<second>() as u128)
            .into_query(name)
            .add_field("charge", self.charge.get::<watt_hour>())
            .add_field("energy_in", self.energy_in.get::<watt_hour>())
            .add_field("energy_out", self.energy_out.get::<watt_hour>())
            .add_field("power", self.power.get::<watt>())
    }
}

impl From<RawBattery> for Battery {
    fn from(other: RawBattery) -> Self {
        Self {
            time: Time::new::<second>(other.time.timestamp() as f64),
            charge: Energy::new::<watt_hour>(other.charge),
            energy_in: Energy::new::<watt_hour>(other.energy_in),
            energy_out: Energy::new::<watt_hour>(other.energy_out),
            power: Power::new::<watt>(other.power),
        }
    }
}

#[test]
fn test_power_calculation() {
    let old = Battery::new(
        Time::new::<second>(1680966000.0),
        Energy::new::<watt_hour>(5000.0),
        Energy::new::<watt_hour>(430.0),
        Energy::new::<watt_hour>(497.0),
        Power::new::<watt>(0.0),
    );
    let new = Battery::new(
        Time::new::<second>(1680966300.0),
        Energy::new::<watt_hour>(5000.0),
        Energy::new::<watt_hour>(476.0),
        Energy::new::<watt_hour>(497.0),
        Power::new::<watt>(0.0),
    );

    assert_eq!(552.0, new.calc_power(&old).get::<watt>());
}
