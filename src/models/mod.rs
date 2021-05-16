use influxdb::{
    integrations::serde_integration::DatabaseQueryResult, Error, Query,
    ReadQuery,
};

pub mod battery;
pub mod dachs;
pub mod meter;
pub mod solar;
pub mod weather;

pub use battery::Battery;
pub use dachs::Dachs;
pub use meter::Meter;
pub use solar::Solar;
pub use weather::Weather;

pub enum InfluxResult<T> {
    Some(T),
    None,
    Err(String),
}

pub trait InfluxObject<T: 'static + Send + for<'de> serde::Deserialize<'de>> {
    const FIELDS: &'static str;
    const MEASUREMENT: &'static str;

    fn query_last() -> ReadQuery {
        return <dyn Query>::raw_read_query(format!(
            "SELECT time, {} FROM {} ORDER BY DESC LIMIT 1",
            Self::FIELDS,
            Self::MEASUREMENT
        ));
    }

    fn query_first() -> ReadQuery {
        return <dyn Query>::raw_read_query(format!(
            "SELECT time, {} FROM {} ORDER BY ASC LIMIT 1",
            Self::FIELDS,
            Self::MEASUREMENT
        ));
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
