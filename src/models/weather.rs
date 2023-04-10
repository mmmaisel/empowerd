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
    units::{
        celsius, degree, hectopascal, meter_per_second, millimeter, percent,
        ratio, second, Angle, Length, Pressure, Ratio, Temperature, Time,
        Velocity,
    },
    InfluxObject,
};
use bresser6in1_usb::Data as BresserData;
use chrono::{DateTime, Utc};
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawWeather {
    pub time: DateTime<Utc>,
    pub temperature_in: f64,
    pub humidity_in: f64,
    pub temperature_out: Option<f64>,
    pub humidity_out: Option<f64>,
    pub rain_day: f64,
    pub rain_actual: f64,
    pub wind_actual: f64,
    pub wind_gust: f64,
    pub wind_dir: f64,
    pub baro_sea: f64,
    pub baro_absolute: f64,
    pub uv_index: f64,
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
    pub rain_day: Length,
    pub rain_actual: Length,
    pub wind_actual: Velocity,
    pub wind_gust: Velocity,
    pub wind_dir: Angle,
    pub baro_sea: Pressure,
    pub baro_absolute: Pressure,
    pub uv_index: Ratio,
    pub dew_point: Option<Temperature>,
    pub temperature_x1: Option<Temperature>,
    pub humidity_x1: Option<Ratio>,
    pub temperature_x2: Option<Temperature>,
    pub humidity_x2: Option<Ratio>,
    pub temperature_x3: Option<Temperature>,
    pub humidity_x3: Option<Ratio>,
}

impl Weather {
    pub fn new(data: BresserData) -> Self {
        return Self {
            time: Time::new::<second>(data.timestamp as f64),
            temperature_in: Temperature::new::<celsius>(
                data.temperature_in as f64,
            ),
            humidity_in: Ratio::new::<percent>(data.humidity_in as f64),
            temperature_out: data
                .temperature_out
                .map(|x| Temperature::new::<celsius>(x as f64)),
            humidity_out: data
                .humidity_out
                .map(|x| Ratio::new::<percent>(x as f64)),
            rain_day: Length::new::<millimeter>(data.rain_day as f64),
            rain_actual: Length::new::<millimeter>(data.rain_actual as f64),
            wind_actual: Velocity::new::<meter_per_second>(
                data.wind_actual as f64,
            ),
            wind_gust: Velocity::new::<meter_per_second>(data.wind_gust as f64),
            wind_dir: Angle::new::<degree>(data.wind_dir as f64),
            baro_sea: Pressure::new::<hectopascal>(data.baro_sea as f64),
            baro_absolute: Pressure::new::<hectopascal>(
                data.baro_absolute as f64,
            ),
            uv_index: Ratio::new::<ratio>(data.uv_index as f64),
            dew_point: data
                .dew_point
                .map(|x| Temperature::new::<celsius>(x as f64)),
            temperature_x1: data
                .temperature_x1
                .map(|x| Temperature::new::<celsius>(x as f64)),
            humidity_x1: data
                .humidity_x1
                .map(|x| Ratio::new::<percent>(x as f64)),
            temperature_x2: data
                .temperature_x2
                .map(|x| Temperature::new::<celsius>(x as f64)),
            humidity_x2: data
                .humidity_x2
                .map(|x| Ratio::new::<percent>(x as f64)),
            temperature_x3: data
                .temperature_x3
                .map(|x| Temperature::new::<celsius>(x as f64)),
            humidity_x3: data
                .humidity_x3
                .map(|x| Ratio::new::<percent>(x as f64)),
        };
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
            .add_field("rain_day", self.rain_day.get::<millimeter>())
            .add_field("rain_actual", self.rain_actual.get::<millimeter>())
            .add_field(
                "wind_actual",
                self.wind_actual.get::<meter_per_second>(),
            )
            .add_field("wind_gust", self.wind_gust.get::<meter_per_second>())
            .add_field("wind_dir", self.wind_dir.get::<degree>())
            .add_field("baro_sea", self.baro_sea.get::<hectopascal>())
            .add_field("baro_absolute", self.baro_absolute.get::<hectopascal>())
            .add_field("uv_index", self.uv_index.get::<ratio>())
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
                .map(|x| Temperature::new::<celsius>(x)),
            humidity_out: other.humidity_out.map(|x| Ratio::new::<percent>(x)),
            rain_day: Length::new::<millimeter>(other.rain_day),
            rain_actual: Length::new::<millimeter>(other.rain_actual),
            wind_actual: Velocity::new::<meter_per_second>(other.wind_actual),
            wind_gust: Velocity::new::<meter_per_second>(other.wind_gust),
            wind_dir: Angle::new::<degree>(other.wind_dir),
            baro_sea: Pressure::new::<hectopascal>(other.baro_sea),
            baro_absolute: Pressure::new::<hectopascal>(other.baro_absolute),
            uv_index: Ratio::new::<ratio>(other.uv_index),
            dew_point: other.dew_point.map(|x| Temperature::new::<celsius>(x)),
            temperature_x1: other
                .temperature_x1
                .map(|x| Temperature::new::<celsius>(x)),
            humidity_x1: other.humidity_x1.map(|x| Ratio::new::<percent>(x)),
            temperature_x2: other
                .temperature_x2
                .map(|x| Temperature::new::<celsius>(x)),
            humidity_x2: other.humidity_x2.map(|x| Ratio::new::<percent>(x)),
            temperature_x3: other
                .temperature_x3
                .map(|x| Temperature::new::<celsius>(x)),
            humidity_x3: other.humidity_x3.map(|x| Ratio::new::<percent>(x)),
        }
    }
}
