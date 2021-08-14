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
use influxdb::{
    integrations::serde_integration::DatabaseQueryResult, Error,
    InfluxDbWriteable, Query, ReadQuery, WriteQuery,
};

pub mod battery;
pub mod generator;
pub mod meter;
pub mod solar;
pub mod weather;

pub use battery::Battery;
pub use generator::Generator;
pub use meter::Meter;
pub use solar::Solar;
pub use weather::Weather;

pub enum InfluxResult<T> {
    Some(T),
    None,
    Err(String),
}

pub trait InfluxObject<T: 'static + Send + for<'de> serde::Deserialize<'de>>:
    InfluxDbWriteable
{
    const FIELDS: &'static str;

    fn query_last(measurement: &str) -> ReadQuery {
        return <dyn Query>::raw_read_query(format!(
            "SELECT time, {} FROM {} ORDER BY DESC LIMIT 1",
            Self::FIELDS,
            measurement,
        ));
    }

    fn query_first(measurement: &str) -> ReadQuery {
        return <dyn Query>::raw_read_query(format!(
            "SELECT time, {} FROM {} ORDER BY ASC LIMIT 1",
            Self::FIELDS,
            measurement,
        ));
    }

    fn save_query(self, measurement: &str) -> WriteQuery
    where
        Self: Sized,
    {
        return self.into_query(measurement);
    }

    fn into_single(
        response: Result<DatabaseQueryResult, Error>,
    ) -> InfluxResult<T> {
        let mut results = match response {
            Ok(x) => x,
            Err(e) => return InfluxResult::Err(e.to_string()),
        };
        let mut result = match results.deserialize_next::<T>() {
            Ok(x) => x,
            Err(e) => return InfluxResult::Err(e.to_string()),
        };

        if result.series.len() > 1 {
            return InfluxResult::Err("Received more than one series".into());
        }
        let mut series = match result.series.pop() {
            None => return InfluxResult::None,
            Some(x) => x,
        };

        if series.values.len() > 1 {
            return InfluxResult::Err("Received more than one value".into());
        }
        return match series.values.pop() {
            None => InfluxResult::None,
            Some(x) => InfluxResult::Some(x),
        };
    }
}
