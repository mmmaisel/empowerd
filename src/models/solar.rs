extern crate influx_db_client;
extern crate serde_json;

// TODO: write tests

//use futures::future::Future;
use influx_db_client::Points;

use super::load_series;
use influx_derive::{InfluxLoad, InfluxPoint};

#[derive(Debug, InfluxLoad, InfluxPoint)]
pub struct SolarData
{
    pub timestamp: i64,
    pub power: f64,
    pub energy: f64
}

impl SolarData
{
    const SERIES_NAME: &'static str = "solar";

    pub fn new(timestamp: i64, power: f64, energy: f64) -> SolarData
    {
        return SolarData
        {
            timestamp: timestamp,
            power: power,
            energy: energy
        };
    }

    // TODO: generic this
    pub fn first(conn: &Client) -> Result<SolarData, String>
    {
        let mut queried = SolarData::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY \"time\" ASC LIMIT 1",
            SolarData::SERIES_NAME))?;
        // TODO: validate only 1 received
        return Ok(queried.pop().unwrap());
    }

    // TODO: generic this
    pub fn last(conn: &Client) -> Result<SolarData, String>
    {
        let mut queried = SolarData::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY \"time\" DESC LIMIT 1",
            SolarData::SERIES_NAME))?;
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
    pub fn save_all(conn: &Client, data: Vec<SolarData>)
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
