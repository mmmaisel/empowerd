/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2023 Max Maisel

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
pub struct Heatpump {
    pub time: DateTime<Utc>,
    pub energy: f64,
    pub power: f64,
    pub cop: Option<f64>,
    pub boiler_top: Option<f64>,
    pub boiler_mid: Option<f64>,
    pub boiler_bot: Option<f64>,
}

impl Heatpump {
    pub fn new(
        time: DateTime<Utc>,
        energy: f64,
        power: f64, // TODO: remove, use derivative query
        cop: Option<f64>,
        boiler_top: Option<f64>,
        boiler_mid: Option<f64>,
        boiler_bot: Option<f64>,
    ) -> Self {
        Self {
            time,
            energy,
            power,
            cop,
            boiler_top,
            boiler_mid,
            boiler_bot,
        }
    }
}

impl InfluxObject<Heatpump> for Heatpump {
    const FIELDS: &'static str =
        "energy, power, cop, boiler_top, boiler_mid, boiler_bot";
}
