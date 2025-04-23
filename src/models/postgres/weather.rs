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
    impl_timeseries, schema,
    units::{
        celsius, degree, hectopascal, meter_per_second, micrometer, millimeter,
        millimeter_per_second, pascal, percent, ratio, second, Abbreviation,
        Angle, Length, Pressure, Ratio, Temperature, Time, Velocity,
    },
};
use crate::Error;
use chrono::{DateTime, NaiveDateTime};
use diesel::prelude::{
    AsChangeset, ExpressionMethods, Identifiable, Insertable, Queryable,
    Selectable,
};
use ws6in1_proto::parser::Ws6in1Data;

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
    pub rain_day_um: Option<i32>,
    pub rain_act_um: Option<i32>,
    pub wind_act_mms: Option<i32>,
    pub wind_gust_mms: Option<i32>,
    pub wind_dir_deg_e1: Option<i16>,
    pub baro_sea_pa: i32,
    pub baro_abs_pa: i32,
    pub uv_index_e1: Option<i16>,
    pub dew_point_degc_e1: Option<i16>,
    pub temp_x1_degc_e1: Option<i16>,
    pub hum_x1_e3: Option<i16>,
    pub temp_x2_degc_e1: Option<i16>,
    pub hum_x2_e3: Option<i16>,
    pub temp_x3_degc_e1: Option<i16>,
    pub hum_x3_e3: Option<i16>,
    pub temp_x4_degc_e1: Option<i16>,
    pub hum_x4_e3: Option<i16>,
    pub temp_x5_degc_e1: Option<i16>,
    pub hum_x5_e3: Option<i16>,
    pub temp_x6_degc_e1: Option<i16>,
    pub hum_x6_e3: Option<i16>,
    pub temp_x7_degc_e1: Option<i16>,
    pub hum_x7_e3: Option<i16>,
}

#[derive(Clone, Debug)]
pub struct Weather {
    pub time: Time,
    pub temp_in: Temperature,
    pub hum_in: Ratio,
    pub temp_out: Option<Temperature>,
    pub hum_out: Option<Ratio>,
    pub rain_day: Option<Length>,
    pub rain_act: Option<Length>,
    pub wind_act: Option<Velocity>,
    pub wind_gust: Option<Velocity>,
    pub wind_dir: Option<Angle>,
    pub baro_sea: Pressure,
    pub baro_abs: Pressure,
    pub uv_index: Option<Ratio>,
    pub dew_point: Option<Temperature>,
    pub temp_x1: Option<Temperature>,
    pub hum_x1: Option<Ratio>,
    pub temp_x2: Option<Temperature>,
    pub hum_x2: Option<Ratio>,
    pub temp_x3: Option<Temperature>,
    pub hum_x3: Option<Ratio>,
    pub temp_x4: Option<Temperature>,
    pub hum_x4: Option<Ratio>,
    pub temp_x5: Option<Temperature>,
    pub hum_x5: Option<Ratio>,
    pub temp_x6: Option<Temperature>,
    pub hum_x6: Option<Ratio>,
    pub temp_x7: Option<Temperature>,
    pub hum_x7: Option<Ratio>,
}

impl_timeseries!(RawWeather, Weather, weathers);

impl Weather {
    pub fn new(data: Ws6in1Data) -> Self {
        Self {
            time: Time::new::<second>(data.local_timestamp as f64),
            temp_in: Temperature::new::<celsius>(
                data.indoor.temperature as f64,
            ),
            hum_in: Ratio::new::<percent>(data.indoor.humidity as f64),
            temp_out: data
                .outdoor
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            hum_out: data
                .outdoor
                .as_ref()
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            rain_day: data
                .outdoor
                .as_ref()
                .map(|x| Length::new::<millimeter>(x.rain_day as f64)),
            rain_act: data
                .outdoor
                .as_ref()
                .map(|x| Length::new::<millimeter>(x.rain_actual as f64)),
            wind_act: data.outdoor.as_ref().map(|x| {
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
            baro_abs: Pressure::new::<hectopascal>(
                data.indoor.baro_absolute as f64,
            ),
            uv_index: data
                .outdoor
                .as_ref()
                .map(|x| Ratio::new::<ratio>(x.uv_index as f64)),
            dew_point: data
                .outdoor
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.dew_point as f64)),
            temp_x1: data.ext[0]
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            hum_x1: data.ext[0]
                .as_ref()
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            temp_x2: data.ext[1]
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            hum_x2: data.ext[1]
                .as_ref()
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            temp_x3: data.ext[2]
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            hum_x3: data.ext[2]
                .as_ref()
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            temp_x4: data.ext[3]
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            hum_x4: data.ext[3]
                .as_ref()
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            temp_x5: data.ext[4]
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            hum_x5: data.ext[4]
                .as_ref()
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            temp_x6: data.ext[5]
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            hum_x6: data.ext[5]
                .as_ref()
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
            temp_x7: data.ext[6]
                .as_ref()
                .map(|x| Temperature::new::<celsius>(x.temperature as f64)),
            hum_x7: data.ext[6]
                .as_ref()
                .map(|x| Ratio::new::<percent>(x.humidity as f64)),
        }
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
            rain_day: input
                .rain_day_um
                .map(|x| Length::new::<micrometer>(x as f64)),
            rain_act: input
                .rain_act_um
                .map(|x| Length::new::<micrometer>(x as f64)),
            wind_act: input
                .wind_act_mms
                .map(|x| Velocity::new::<millimeter_per_second>(x as f64)),
            wind_gust: input
                .wind_gust_mms
                .map(|x| Velocity::new::<millimeter_per_second>(x as f64)),
            wind_dir: input
                .wind_dir_deg_e1
                .map(|x| Angle::new::<degree>((x as f64) / 1e1)),
            baro_sea: Pressure::new::<pascal>(input.baro_sea_pa as f64),
            baro_abs: Pressure::new::<pascal>(input.baro_abs_pa as f64),
            uv_index: input
                .uv_index_e1
                .map(|x| Ratio::new::<ratio>((x as f64) / 1e1)),
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
            temp_x4: input
                .temp_x4_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            hum_x4: input
                .hum_x4_e3
                .map(|x| Ratio::new::<percent>((x as f64) / 1e1)),
            temp_x5: input
                .temp_x5_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            hum_x5: input
                .hum_x5_e3
                .map(|x| Ratio::new::<percent>((x as f64) / 1e1)),
            temp_x6: input
                .temp_x6_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            hum_x6: input
                .hum_x6_e3
                .map(|x| Ratio::new::<percent>((x as f64) / 1e1)),
            temp_x7: input
                .temp_x7_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            hum_x7: input
                .hum_x7_e3
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
            rain_day_um: input
                .rain_day
                .map(|x| x.get::<micrometer>().round() as i32),
            rain_act_um: input
                .rain_act
                .map(|x| x.get::<micrometer>().round() as i32),
            wind_act_mms: input
                .wind_act
                .map(|x| x.get::<millimeter_per_second>().round() as i32),
            wind_gust_mms: input
                .wind_gust
                .map(|x| x.get::<millimeter_per_second>().round() as i32),
            wind_dir_deg_e1: input
                .wind_dir
                .map(|x| (x.get::<degree>() * 1e1).round() as i16),
            baro_sea_pa: input.baro_sea.get::<pascal>().round() as i32,
            baro_abs_pa: input.baro_abs.get::<pascal>().round() as i32,
            uv_index_e1: input
                .uv_index
                .map(|x| (x.get::<ratio>() * 1e1).round() as i16),
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
            temp_x4_degc_e1: input
                .temp_x4
                .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
            hum_x4_e3: input
                .hum_x4
                .map(|x| (x.get::<percent>() * 1e1).round() as i16),
            temp_x5_degc_e1: input
                .temp_x5
                .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
            hum_x5_e3: input
                .hum_x5
                .map(|x| (x.get::<percent>() * 1e1).round() as i16),
            temp_x6_degc_e1: input
                .temp_x6
                .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
            hum_x6_e3: input
                .hum_x6
                .map(|x| (x.get::<percent>() * 1e1).round() as i16),
            temp_x7_degc_e1: input
                .temp_x7
                .map(|x| (x.get::<celsius>() * 1e1).round() as i16),
            hum_x7_e3: input
                .hum_x7
                .map(|x| (x.get::<percent>() * 1e1).round() as i16),
        })
    }
}
