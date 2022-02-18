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

#[derive(Clone, Deserialize, Debug, InfluxDbWriteable)]
pub struct Generator {
    pub time: DateTime<Utc>,
    pub power: f64,
    pub runtime: f64,
    pub energy: f64,
}

impl Generator {
    pub fn new(
        time: DateTime<Utc>,
        energy: f64,
        power: f64, // TODO: remove, use derivative query
        runtime: f64,
    ) -> Self {
        return Self {
            time: time,
            energy: energy,
            power: power,
            runtime: runtime,
        };
    }
}

impl InfluxObject<Generator> for Generator {
    const FIELDS: &'static str = "energy, power, runtime";
}
