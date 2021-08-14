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
    ) -> Self {
        return Self {
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
