/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2022 Max Maisel

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
use super::units;
use influxdb::{
    integrations::serde_integration::{DatabaseQueryResult, Series},
    Error, InfluxDbWriteable, ReadQuery, WriteQuery,
};

pub mod battery;
pub mod bidirectional_meter;
pub mod generator;
pub mod heatpump;
pub mod simple_meter;
pub mod weather;

pub use battery::Battery;
pub use bidirectional_meter::BidirectionalMeter;
pub use generator::Generator;
pub use heatpump::Heatpump;
pub use simple_meter::SimpleMeter;
pub use weather::Weather;

pub enum InfluxResult<T> {
    None,
    Some(T),
    Err(String),
}

pub enum InfluxSeriesResult<T> {
    None,
    Some(Series<T>),
    Err(String),
}

pub enum InfluxOrder {
    Asc,
    Desc,
}

impl std::fmt::Display for InfluxOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asc => write!(f, "ASC"),
            Self::Desc => write!(f, "DESC"),
        }
    }
}

pub trait InfluxObject<T: 'static + Send + for<'de> serde::Deserialize<'de>>:
    InfluxDbWriteable
{
    const FIELDS: &'static str;

    fn query_last(measurement: &str) -> ReadQuery {
        ReadQuery::new(format!(
            "SELECT time, {} FROM {} ORDER BY DESC LIMIT 1",
            Self::FIELDS,
            measurement,
        ))
    }

    fn query_first(measurement: &str) -> ReadQuery {
        ReadQuery::new(format!(
            "SELECT time, {} FROM {} ORDER BY ASC LIMIT 1",
            Self::FIELDS,
            measurement,
        ))
    }

    fn query_where(
        measurement: &str,
        query: &str,
        limit: Option<u32>,
        order: InfluxOrder,
    ) -> ReadQuery {
        let query = format!(
            "SELECT time, {} FROM {} WHERE {} ORDER BY {}",
            Self::FIELDS,
            measurement,
            query,
            order,
        );

        match limit {
            Some(lim) => ReadQuery::new(format!("{} LIMIT {}", query, lim)),
            None => ReadQuery::new(query),
        }
    }

    fn save_query(self, measurement: &str) -> WriteQuery
    where
        Self: Sized,
    {
        self.into_query(measurement)
    }

    fn into_series(
        response: Result<DatabaseQueryResult, Error>,
    ) -> InfluxSeriesResult<T> {
        let mut results = match response {
            Ok(x) => x,
            Err(e) => return InfluxSeriesResult::Err(e.to_string()),
        };
        let mut result = match results.deserialize_next::<T>() {
            Ok(x) => x,
            Err(e) => return InfluxSeriesResult::Err(e.to_string()),
        };

        if result.series.len() > 1 {
            return InfluxSeriesResult::Err(
                "Received more than one series".into(),
            );
        }
        return match result.series.pop() {
            None => InfluxSeriesResult::None,
            Some(x) => InfluxSeriesResult::Some(x),
        };
    }

    fn into_single(
        response: Result<DatabaseQueryResult, Error>,
    ) -> InfluxResult<T> {
        let mut series = match Self::into_series(response) {
            InfluxSeriesResult::None => return InfluxResult::None,
            InfluxSeriesResult::Some(x) => x,
            InfluxSeriesResult::Err(e) => return InfluxResult::Err(e),
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
