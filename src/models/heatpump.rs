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
use super::{
    units::{
        celsius, ratio, second, watt, watt_hour, Energy, Power, Ratio,
        Temperature, Time,
    },
    InfluxObject,
};
use chrono::{DateTime, Utc};
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawHeatpump {
    pub time: DateTime<Utc>,
    pub energy: f64,
    pub power: f64,
    pub total_heat: Option<f64>,
    pub cop: Option<f64>,
    pub boiler_top: Option<f64>,
    pub boiler_mid: Option<f64>,
    pub boiler_bot: Option<f64>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(from = "RawHeatpump")]
pub struct Heatpump {
    pub time: Time,
    pub energy: Energy,
    pub power: Power,
    pub total_heat: Option<Energy>,
    pub cop: Option<Ratio>,
    pub boiler_top: Option<Temperature>,
    pub boiler_mid: Option<Temperature>,
    pub boiler_bot: Option<Temperature>,
}

impl Heatpump {
    pub fn new(
        time: Time,
        energy: Energy,
        power: Power, // TODO: remove, use derivative query
        total_heat: Option<Energy>,
        cop: Option<Ratio>,
        boiler_top: Option<Temperature>,
        boiler_mid: Option<Temperature>,
        boiler_bot: Option<Temperature>,
    ) -> Self {
        Self {
            time,
            energy,
            power,
            total_heat,
            cop,
            boiler_top,
            boiler_mid,
            boiler_bot,
        }
    }
}

impl InfluxObject<Heatpump> for Heatpump {
    const FIELDS: &'static str =
        "energy, power, total_heat, cop, boiler_top, boiler_mid, boiler_bot";
}

impl InfluxDbWriteable for Heatpump {
    fn into_query<T: Into<String>>(self, name: T) -> WriteQuery {
        Timestamp::Seconds(self.time.get::<second>() as u128)
            .into_query(name)
            .add_field("energy", self.energy.get::<watt_hour>())
            .add_field("power", self.power.get::<watt>())
            .add_field(
                "total_heat",
                self.total_heat.map(|x| x.get::<watt_hour>()),
            )
            .add_field("cop", self.cop.map(|x| x.get::<ratio>()))
            .add_field(
                "boiler_top",
                self.boiler_top.map(|x| x.get::<celsius>()),
            )
            .add_field(
                "boiler_mid",
                self.boiler_mid.map(|x| x.get::<celsius>()),
            )
            .add_field(
                "boiler_bot",
                self.boiler_bot.map(|x| x.get::<celsius>()),
            )
    }
}

impl From<RawHeatpump> for Heatpump {
    fn from(other: RawHeatpump) -> Self {
        Self {
            time: Time::new::<second>(other.time.timestamp() as f64),
            energy: Energy::new::<watt_hour>(other.energy),
            power: Power::new::<watt>(other.power),
            total_heat: other.total_heat.map(|x| Energy::new::<watt_hour>(x)),
            cop: other.cop.map(|x| Ratio::new::<ratio>(x)),
            boiler_top: other
                .boiler_top
                .map(|x| Temperature::new::<celsius>(x)),
            boiler_mid: other
                .boiler_mid
                .map(|x| Temperature::new::<celsius>(x)),
            boiler_bot: other
                .boiler_bot
                .map(|x| Temperature::new::<celsius>(x)),
        }
    }
}
