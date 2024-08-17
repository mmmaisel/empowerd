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
        celsius, percent, second, watt, watt_hour, Abbreviation, Energy, Power,
        Ratio, Temperature, Time,
    },
};
use crate::Error;
use chrono::{DateTime, NaiveDateTime};
use diesel::prelude::{
    AsChangeset, ExpressionMethods, Identifiable, Insertable, Queryable,
    Selectable,
};

#[derive(AsChangeset, Identifiable, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::heatpumps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(series_id, time))]
pub struct RawHeatpump {
    pub series_id: i32,
    pub time: NaiveDateTime,
    pub energy_wh: i64,
    pub power_w: i32,
    pub heat_wh: Option<i64>,
    pub cop_pct: Option<i16>,
    pub boiler_top_degc_e1: Option<i16>,
    pub boiler_mid_degc_e1: Option<i16>,
    pub boiler_bot_degc_e1: Option<i16>,
}

#[derive(Clone, Debug)]
pub struct Heatpump {
    pub time: Time,
    pub energy: Energy,
    pub power: Power,
    pub heat: Option<Energy>,
    pub cop: Option<Ratio>,
    pub boiler_top: Option<Temperature>,
    pub boiler_mid: Option<Temperature>,
    pub boiler_bot: Option<Temperature>,
}

impl_timeseries!(RawHeatpump, Heatpump, heatpumps);

impl Heatpump {
    // TODO: dedup
    pub fn calc_power(&self, other: &Self) -> Power {
        if self.time == other.time {
            Power::new::<watt>(0.0)
        } else {
            (self.energy - other.energy) / (self.time - other.time).abs()
        }
    }
}

impl From<RawHeatpump> for Heatpump {
    fn from(input: RawHeatpump) -> Self {
        Self {
            time: Time::new::<second>(input.time.and_utc().timestamp() as f64),
            energy: Energy::new::<watt_hour>(input.energy_wh as f64),
            power: Power::new::<watt>(input.power_w as f64),
            heat: input.heat_wh.map(|x| Energy::new::<watt_hour>(x as f64)),
            cop: input.cop_pct.map(|x| Ratio::new::<percent>(x as f64)),
            boiler_top: input
                .boiler_top_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            boiler_mid: input
                .boiler_mid_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
            boiler_bot: input
                .boiler_bot_degc_e1
                .map(|x| Temperature::new::<celsius>((x as f64) / 1e1)),
        }
    }
}

impl TryFrom<&Heatpump> for RawHeatpump {
    type Error = Error;
    fn try_from(input: &Heatpump) -> Result<Self, Self::Error> {
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
            energy_wh: input.energy.get::<watt_hour>().round() as i64,
            power_w: input.power.get::<watt>().round() as i32,
            heat_wh: input.heat.map(|x| x.get::<watt_hour>().round() as i64),
            cop_pct: input.cop.map(|x| x.get::<percent>().round() as i16),
            boiler_top_degc_e1: input
                .boiler_top
                .map(|x| (x.get::<celsius>() * 10.0).round() as i16),
            boiler_mid_degc_e1: input
                .boiler_mid
                .map(|x| (x.get::<celsius>() * 10.0).round() as i16),
            boiler_bot_degc_e1: input
                .boiler_bot
                .map(|x| (x.get::<celsius>() * 10.0).round() as i16),
        })
    }
}
