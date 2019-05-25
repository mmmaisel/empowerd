extern crate influx_db_client;
extern crate serde_json;

//use futures::future::Future;
use influx_db_client::Points;

use super:: load_series;
use influx_derive::{InfluxLoad, InfluxPoint};

#[derive(Debug, InfluxLoad, InfluxPoint)]
pub struct DachsData
{
    // TODO: better, db consistent field names
    pub timestamp: i64,
    pub power: f64,
    pub runtime: f64,
    pub energy: f64
}

impl DachsData
{
    const SERIES_NAME: &'static str = "dachs";

    pub fn new(timestamp: i64, power: f64, runtime: f64, energy: f64)
        -> DachsData
    {
        return DachsData
        {
            timestamp: timestamp,
            power: power,
            runtime: runtime,
            energy: energy
        };
    }

    // TODO: generic this
    pub fn first(conn: &Client) -> Result<DachsData, String>
    {
        let mut queried = DachsData::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY \"time\" ASC LIMIT 1",
            DachsData::SERIES_NAME))?;
        // TODO: validate only 1 received
        return Ok(queried.pop().unwrap());
    }

    // TODO: generic this
    pub fn last(conn: &Client) -> Result<DachsData, String>
    {
        let mut queried = DachsData::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY \"time\" DESC LIMIT 1",
            DachsData::SERIES_NAME))?;
        // TODO: validate only 1 received
        return Ok(queried.pop().unwrap());
    }

    // TODO: generic this
    pub fn save(&self, conn: &Client) -> Result<(), String>
    {
        // TODO: correct error handling
        conn.write_point(self.to_point(), Some(Precision::Seconds), None).
            expect("💩️ influx");
        println!("wrote {:?} to influx", self);

        return Ok(());
    }

    // TODO: generic this
    pub fn save_all(conn: &Client, data: Vec<DachsData>)
        -> Result<(), String>
    {
        let points: Points = data.into_iter().map(|x|
        {
            return x.to_point();
        }).collect();

        // TODO: correct error handling
        conn.write_points(points, Some(Precision::Seconds), None).
            expect("💩️ influx");
        println!("wrote points to influx");

        return Ok(());
    }
}
