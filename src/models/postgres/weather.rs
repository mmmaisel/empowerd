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
    impl_timeseries, schema,
    units::{
        celsius, degree, hectopascal, meter_per_second, micrometer, millimeter,
        millimeter_per_second, pascal, percent, ratio, second, Abbreviation,
        Angle, Length, Pressure, Ratio, Temperature, Time, Velocity,
    },
};
use crate::Error;
use bresser6in1_usb::Data as BresserData;
use chrono::{DateTime, NaiveDateTime};
use diesel::prelude::{
    AsChangeset, ExpressionMethods, Identifiable, Insertable, Queryable,
    Selectable,
};

#[derive(AsChangeset, Identifiable, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::weathers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(series_id, time))]
pub struct RawWeather {
    pub series_id: i32,
    pub time: NaiveDateTime,
    pub temp_in_degc_e1: i16,
    pub hum_in_e3: i16,
    pub temp_out_degc_e1: Option<i16>,
    pub hum_out_e3: Option<i16>,
    pub rain_day_um: i32,
    pub rain_act_um: i32,
    pub wind_act_mms: i32,
    pub wind_gust_mms: i32,
    pub wind_dir_deg_e1: i16,
    pub baro_sea_pa: i32,
    pub baro_abs_pa: i32,
    pub uv_index_e1: i16,
    pub dew_point_degc_e1: Option<i16>,
    pub temp_x1_degc_e1: Option<i16>,
    pub hum_x1_e3: Option<i16>,
    pub temp_x2_degc_e1: Option<i16>,
    pub hum_x2_e3: Option<i16>,
    pub temp_x3_degc_e1: Option<i16>,
    pub hum_x3_e3: Option<i16>,
}

#[derive(Clone, Debug)]
pub struct Weather {
    pub time: Time,
    pub temp_in: Temperature,
    pub hum_in: Ratio,
    pub temp_out: Option<Temperature>,
    pub hum_out: Option<Ratio>,
    pub rain_day: Length,
    pub rain_act: Length,
    pub wind_act: Velocity,
    pub wind_gust: Velocity,
    pub wind_dir: Angle,
    pub baro_sea: Pressure,
    pub baro_abs: Pressure,
    pub uv_index: Ratio,
    pub dew_point: Option<Temperature>,
    pub temp_x1: Option<Temperature>,
    pub hum_x1: Option<Ratio>,
    pub temp_x2: Option<Temperature>,
    pub hum_x2: Option<Ratio>,
    pub temp_x3: Option<Temperature>,
    pub hum_x3: Option<Ratio>,
}

impl_timeseries!(RawWeather, Weather, weathers);

impl Weather {
    pub fn new(data: BresserData) -> Self {
        return Self {
            time: Time::new::<second>(data.timestamp as f64),
            temp_in: Temperature::new::<celsius>(data.temperature_in as f64),
            hum_in: Ratio::new::<percent>(data.humidity_in as f64),
            temp_out: data
                .temperature_out
                .map(|x| Temperature::new::<celsius>(x as f64)),
            hum_out: data.humidity_out.map(|x| Ratio::new::<percent>(x as f64)),
            rain_day: Length::new::<millimeter>(data.rain_day as f64),
            rain_act: Length::new::<millimeter>(data.rain_actual as f64),
            wind_act: Velocity::new::<meter_per_second>(
                data.wind_actual as f64,
            ),
            wind_gust: Velocity::new::<meter_per_second>(data.wind_gust as f64),
            wind_dir: Angle::new::<degree>(data.wind_dir as f64),
            baro_sea: Pressure::new::<hectopascal>(data.baro_sea as f64),
            baro_abs: Pressure::new::<hectopascal>(data.baro_absolute as f64),
            uv_index: Ratio::new::<ratio>(data.uv_index as f64),
            dew_point: data
                .dew_point
                .map(|x| Temperature::new::<celsius>(x as f64)),
            temp_x1: data
                .temperature_x1
                .map(|x| Temperature::new::<celsius>(x as f64)),
            hum_x1: data.humidity_x1.map(|x| Ratio::new::<percent>(x as f64)),
            temp_x2: data
                .temperature_x2
                .map(|x| Temperature::new::<celsius>(x as f64)),
            hum_x2: data.humidity_x2.map(|x| Ratio::new::<percent>(x as f64)),
            temp_x3: data
                .temperature_x3
                .map(|x| Temperature::new::<celsius>(x as f64)),
            hum_x3: data.humidity_x3.map(|x| Ratio::new::<percent>(x as f64)),
        };
    }
}

impl From<RawWeather> for Weather {
    fn from(input: RawWeather) -> Self {
        Self {
            time: Time::new::<second>(input.time.and_utc().timestamp() as f64),
            temp_in: Temperature::new::<celsius>(
                (input.temp_in_degc_e1 as f64) / 1e1,
            ),
            hum_in: Ratio::new::<percent>((input.hum_in_e3 as f64) / 1e1),
            temp_out: input
                .temp_out_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            hum_out: input
                .hum_out_e3
                .map(|x| Ratio::new::<percent>((x as f64) / 1e1)),
            rain_day: Length::new::<micrometer>(input.rain_day_um as f64),
            rain_act: Length::new::<micrometer>(input.rain_act_um as f64),
            wind_act: Velocity::new::<millimeter_per_second>(
                input.wind_act_mms as f64,
            ),
            wind_gust: Velocity::new::<millimeter_per_second>(
                input.wind_gust_mms as f64,
            ),
            wind_dir: Angle::new::<degree>(
                (input.wind_dir_deg_e1 as f64) / 1e1,
            ),
            baro_sea: Pressure::new::<pascal>(input.baro_sea_pa as f64),
            baro_abs: Pressure::new::<pascal>(input.baro_abs_pa as f64),
            uv_index: Ratio::new::<ratio>((input.uv_index_e1 as f64) / 1e1),
            dew_point: input
                .dew_point_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            temp_x1: input
                .temp_x1_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            hum_x1: input
                .hum_x1_e3
                .map(|x| Ratio::new::<percent>((x as f64) / 1e1)),
            temp_x2: input
                .temp_x2_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            hum_x2: input
                .hum_x2_e3
                .map(|x| Ratio::new::<percent>((x as f64) / 1e1)),
            temp_x3: input
                .temp_x3_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            hum_x3: input
                .hum_x3_e3
                .map(|x| Ratio::new::<percent>((x as f64) / 1e1)),
        }
    }
}

impl TryFrom<&Weather> for RawWeather {
    type Error = Error;
    fn try_from(input: &Weather) -> Result<Self, Self::Error> {
        Ok(Self {
            series_id: 0,
            time: DateTime::from_timestamp(
                input.time.get::<second>() as i64,
                0,
            )
            .ok_or_else(|| {
                Error::InvalidInput(format!(
                    "Invalid timestamp: {:?}",
                    input.time.into_format_args(second, Abbreviation),
                ))
            })?
            .naive_utc(),
            temp_in_degc_e1: (input.temp_in.get::<celsius>() * 1e1).round()
                as i16,
            hum_in_e3: (input.hum_in.get::<percent>() * 1e1).round() as i16,
            temp_out_degc_e1: input
                .temp_out
                .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
            hum_out_e3: input
                .hum_out
                .map(|x| (x.get::<percent>() * 1e1).round() as i16),
            rain_day_um: input.rain_day.get::<micrometer>().round() as i32,
            rain_act_um: input.rain_act.get::<micrometer>().round() as i32,
            wind_act_mms: input.wind_act.get::<millimeter_per_second>().round()
                as i32,
            wind_gust_mms: input
                .wind_gust
                .get::<millimeter_per_second>()
                .round() as i32,
            wind_dir_deg_e1: (input.wind_dir.get::<degree>() * 1e1).round()
                as i16,
            baro_sea_pa: input.baro_sea.get::<pascal>().round() as i32,
            baro_abs_pa: input.baro_abs.get::<pascal>().round() as i32,
            uv_index_e1: (input.uv_index.get::<ratio>() * 1e1).round() as i16,
            dew_point_degc_e1: input
                .dew_point
                .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
            temp_x1_degc_e1: input
                .temp_x1
                .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
            hum_x1_e3: input
                .hum_x1
                .map(|x| (x.get::<percent>() * 1e1).round() as i16),
            temp_x2_degc_e1: input
                .temp_x2
                .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
            hum_x2_e3: input
                .hum_x2
                .map(|x| (x.get::<percent>() * 1e1).round() as i16),
            temp_x3_degc_e1: input
                .temp_x3
                .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
            hum_x3_e3: input
                .hum_x3
                .map(|x| (x.get::<percent>() * 1e1).round() as i16),
        })
    }
}
