//extern crate futures;
//extern crate influx_db_client;

//use futures::future::Future;
//use influx_db_client::{Client, Point, Points, Value, Precision};
//use sma::SmaData;

pub mod dachs;
pub mod solar;

//pub use dachs as dachs_model;
//pub use solar as solar_model;

//pub struct InfluxConnector
//{
//    client: Client
//}
//
//impl InfluxConnector
//{
//    pub fn new(influx_addr: String, influx_name: String)
//        -> InfluxConnector
//    {
//        return InfluxConnector
//        {
//            client: Client::new(
//                format!("http://{}", influx_addr), influx_name)
//        };
//    }
//
//    // TODO: get rid of pre-parsed shit from influx client Series
//
//    // TODO: to get latest value:
//    // SELECT * FROM <SERIES> GROUP BY * ORDER BY ASC LIMIT 1
//
//    // TODO: calculate deltas, not here
//    pub fn write_int_series(name: &'static str, data: Vec<sma::TimestampedInt>)
//        -> Result<(), String>
//    {
//        // TODO: unify dachs and sma data, timespamps everywhere
//        // TODO: write multiple points in one call
//
// /*        let mut measurement = Point::new("solar");
//        measurement.add_timestamp(timestamp);
//        measurement.add_field("total", Value::Float(total));
//        measurement.add_field("power", Value::Float(power));
//        client.write_point(measurement, Some(Precision::Seconds), None).
//            expect("💩️ influx");*/
//        return Err("Not implemented".to_string());
//    }
//
//    pub fn write_int(name: &'static str, data: i32, timestamp: u64)
//        -> Result<(), String>
//    {
//        return Err("Not implemented".to_string());
//    }
//}
