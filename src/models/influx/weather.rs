/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2025 Max Maisel

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
        celsius, degree, hectopascal, meter_per_second, millimeter, percent,
        ratio, second, Angle, Length, Pressure, Ratio, Temperature, Time,
        Velocity,
    },
    InfluxObject,
};
use chrono::{DateTime, Utc};
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use serde::Deserialize;
use ws6in1_proto::parser::Ws6in1Data;

#[derive(Deserialize)]
pub struct RawWeather {
    pub time: DateTime<Utc>,
    pub temperature_in: f64,
    pub humidity_in: f64,
    pub temperature_out: Option<f64>,
    pub humidity_out: Option<f64>,
    pub rain_day: Option<f64>,
    pub rain_actual: Option<f64>,
    pub wind_actual: Option<f64>,
    pub wind_gust: Option<f64>,
    pub wind_dir: Option<f64>,
    pub baro_sea: f64,
    pub baro_absolute: f64,
    pub uv_index: Option<f64>,
    pub dew_point: Option<f64>,
    pub temperature_x1: Option<f64>,
    pub humidity_x1: Option<f64>,
    pub temperature_x2: Option<f64>,
    pub humidity_x2: Option<f64>,
    pub temperature_x3: Option<f64>,
    pub humidity_x3: Option<f64>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(from = "RawWeather")]
pub struct Weather {
    pub time: Time,
    pub temperature_in: Temperature,
    pub humidity_in: Ratio,
    pub temperature_out: Option<Temperature>,
    pub humidity_out: Option<Ratio>,
    pub rain_day: Option<Length>,
    pub rain_actual: Option<Length>,
    pub wind_actual: Option<Velocity>,
    pub wind_gust: Option<Velocity>,
    pub wind_dir: Option<Angle>,
    pub baro_sea: Pressure,
    pub baro_absolute: Pressure,
    pub uv_index: Option<Ratio>,
    pub dew_point: Option<Temperature>,
    pub temperature_x1: Option<Temperature>,
    pub humidity_x1: Option<Ratio>,
    pub temperature_x2: Option<Temperature>,
    pub humidity_x2: Option<Ratio>,
    pub temperature_x3: Option<Temperature>,
    pub humidity_x3: Option<Ratio>,
}

impl Weather {
    pub fn new(data: Ws6in1Data) -> Self {
        Self {
            time: Time::new::<second>(data.local_timestamp as f64),
            temperature_in: Temperature::new::<celsius>(
                data.indoor.temperature as f64,
            ),
            humidity_in: Ratio::new::<percent>(data.indoor.humidity as f64),
            temperature_out: data
                .outdoor
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            humidity_out: data
                .outdoor
                .as_ref()
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            rain_day: data
                .outdoor
                .as_ref()
                .map(|x| Length::new::<millimeter>(x.rain_day as f64)),
            rain_actual: data
                .outdoor
                .as_ref()
                .map(|x| Length::new::<millimeter>(x.rain_actual as f64)),
            wind_actual: data.outdoor.as_ref().map(|x| {
                Velocity::new::<meter_per_second>(x.wind_actual as f64)
            }),
            wind_gust: data
                .outdoor
                .as_ref()
                .map(|x| Velocity::new::<meter_per_second>(x.wind_gust as f64)),
            wind_dir: data
                .outdoor
                .as_ref()
                .map(|x| Angle::new::<degree>(x.wind_dir as f64)),
            baro_sea: Pressure::new::<hectopascal>(data.indoor.baro_sea as f64),
            baro_absolute: Pressure::new::<hectopascal>(
                data.indoor.baro_absolute as f64,
            ),
            uv_index: data
                .outdoor
                .as_ref()
                .map(|x| Ratio::new::<ratio>(x.uv_index as f64)),
            dew_point: data
                .outdoor
                .map(|x| Temperature::new::<celsius>(x.dew_point as f64)),
            temperature_x1: data.ext[0]
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            humidity_x1: data.ext[0]
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            temperature_x2: data.ext[1]
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            humidity_x2: data.ext[1]
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            temperature_x3: data.ext[2]
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            humidity_x3: data.ext[2]
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
        }
    }
}

impl InfluxObject<Weather> for Weather {
    const FIELDS: &'static str = "temperature_in, humidity_in, \
        temperature_out, humidity_out, rain_day, rain_actual, \
        wind_actual, wind_gust, wind_dir, baro_sea, baro_absolute, \
        uv_index, dew_point, temperature_x1, humidity_x1, \
        temperature_x2, humidity_x2, temperature_x3, humidity_x3";
}

impl InfluxDbWriteable for Weather {
    fn into_query<T: Into<String>>(self, name: T) -> WriteQuery {
        Timestamp::Seconds(self.time.get::<second>() as u128)
            .into_query(name)
            .add_field("temperature_in", self.temperature_in.get::<celsius>())
            .add_field("humidity_in", self.humidity_in.get::<percent>())
            .add_field(
                "temperature_out",
                self.temperature_out.map(|x| x.get::<celsius>()),
            )
            .add_field(
                "humidity_out",
                self.humidity_out.map(|x| x.get::<percent>()),
            )
            .add_field("rain_day", self.rain_day.map(|x| x.get::<millimeter>()))
            .add_field(
                "rain_actual",
                self.rain_actual.map(|x| x.get::<millimeter>()),
            )
            .add_field(
                "wind_actual",
                self.wind_actual.map(|x| x.get::<meter_per_second>()),
            )
            .add_field(
                "wind_gust",
                self.wind_gust.map(|x| x.get::<meter_per_second>()),
            )
            .add_field("wind_dir", self.wind_dir.map(|x| x.get::<degree>()))
            .add_field("baro_sea", self.baro_sea.get::<hectopascal>())
            .add_field("baro_absolute", self.baro_absolute.get::<hectopascal>())
            .add_field("uv_index", self.uv_index.map(|x| x.get::<ratio>()))
            .add_field("dew_point", self.dew_point.map(|x| x.get::<celsius>()))
            .add_field(
                "temperature_x1",
                self.temperature_x1.map(|x| x.get::<celsius>()),
            )
            .add_field(
                "humidity_x1",
                self.humidity_x1.map(|x| x.get::<percent>()),
            )
            .add_field(
                "temperature_x2",
                self.temperature_x2.map(|x| x.get::<celsius>()),
            )
            .add_field(
                "humidity_x2",
                self.humidity_x2.map(|x| x.get::<percent>()),
            )
            .add_field(
                "temperature_x3",
                self.temperature_x3.map(|x| x.get::<celsius>()),
            )
            .add_field(
                "humidity_x3",
                self.humidity_x3.map(|x| x.get::<percent>()),
            )
    }
}

impl From<RawWeather> for Weather {
    fn from(other: RawWeather) -> Self {
        Self {
            time: Time::new::<second>(other.time.timestamp() as f64),
            temperature_in: Temperature::new::<celsius>(other.temperature_in),
            humidity_in: Ratio::new::<percent>(other.humidity_in),
            temperature_out: other
                .temperature_out
                .map(Temperature::new::<celsius>),
            humidity_out: other.humidity_out.map(Ratio::new::<percent>),
            rain_day: other.rain_day.map(Length::new::<millimeter>),
            rain_actual: other.rain_actual.map(Length::new::<millimeter>),
            wind_actual: other
                .wind_actual
                .map(Velocity::new::<meter_per_second>),
            wind_gust: other.wind_gust.map(Velocity::new::<meter_per_second>),
            wind_dir: other.wind_dir.map(Angle::new::<degree>),
            baro_sea: Pressure::new::<hectopascal>(other.baro_sea),
            baro_absolute: Pressure::new::<hectopascal>(other.baro_absolute),
            uv_index: other.uv_index.map(Ratio::new::<ratio>),
            dew_point: other.dew_point.map(Temperature::new::<celsius>),
            temperature_x1: other
                .temperature_x1
                .map(Temperature::new::<celsius>),
            humidity_x1: other.humidity_x1.map(Ratio::new::<percent>),
            temperature_x2: other
                .temperature_x2
                .map(Temperature::new::<celsius>),
            humidity_x2: other.humidity_x2.map(Ratio::new::<percent>),
            temperature_x3: other
                .temperature_x3
                .map(Temperature::new::<celsius>),
            humidity_x3: other.humidity_x3.map(Ratio::new::<percent>),
        }
    }
}
