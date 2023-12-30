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
    units::{second, watt, watt_hour, Abbreviation, Energy, Power, Time},
};
use crate::Error;
use chrono::NaiveDateTime;
use diesel::prelude::{
    AsChangeset, ExpressionMethods, Identifiable, Insertable, Queryable,
    Selectable,
};

#[derive(AsChangeset, Identifiable, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::simple_meters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(series_id, time))]
pub struct RawSimpleMeter {
    pub series_id: i32,
    pub time: NaiveDateTime,
    pub energy_wh: i64,
    pub power_w: i32,
}

#[derive(Clone, Debug)]
pub struct SimpleMeter {
    pub time: Time,
    pub energy: Energy,
    pub power: Power,
}

impl_timeseries!(RawSimpleMeter, SimpleMeter, simple_meters);

impl SimpleMeter {
    pub fn calc_power(&self, other: &Self) -> Power {
        if self.time == other.time {
            Power::new::<watt>(0.0)
        } else {
            (self.energy - other.energy) / (self.time - other.time).abs()
        }
    }
}

impl From<RawSimpleMeter> for SimpleMeter {
    fn from(input: RawSimpleMeter) -> Self {
        Self {
            time: Time::new::<second>(input.time.timestamp() as f64),
            energy: Energy::new::<watt_hour>(input.energy_wh as f64),
            power: Power::new::<watt>(input.power_w as f64),
        }
    }
}

impl TryFrom<&SimpleMeter> for RawSimpleMeter {
    type Error = Error;
    fn try_from(input: &SimpleMeter) -> Result<Self, Self::Error> {
        Ok(Self {
            series_id: 0,
            time: NaiveDateTime::from_timestamp_opt(
                input.time.get::<second>() as i64,
                0,
            )
            .ok_or_else(|| {
                Error::InvalidInput(format!(
                    "Invalid timestamp: {:?}",
                    input.time.into_format_args(second, Abbreviation),
                ))
            })?,
            energy_wh: input.energy.get::<watt_hour>().round() as i64,
            power_w: input.power.get::<watt>().round() as i32,
        })
    }
}
