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

trait InfluxObject<T: 'static + Send + for<'de> serde::Deserialize<'de>> {
    const FIELDS: &'static str;
    const MEASUREMENT: &'static str;

    fn query_last() -> ReadQuery {
        return Query::raw_read_query(format!(
            "SELECT time, {} FROM {} ORDER BY DESC LIMIT 1",
            Self::FIELDS,
            Self::MEASUREMENT
        ));
    }

    fn query_first() -> ReadQuery {
        return Query::raw_read_query(format!(
            "SELECT time, {} FROM {} ORDER BY ASC LIMIT 1",
            Self::FIELDS,
            Self::MEASUREMENT
        ));
    }

    fn into_single(result: Result<DatabaseQueryResult, Error>) -> T {
        let mut data = result
            .unwrap() // TODO: don't unwrap!!!
            .deserialize_next::<T>()
            .unwrap() // TODO: don't unwrap!!!
            .series
            .pop()
            .unwrap() // TODO: don't unwrap!!!
            .values;
        return data.pop().unwrap();
    }
}
