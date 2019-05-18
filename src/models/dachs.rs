extern crate influx_db_client;
extern crate serde_json;

//use futures::future::Future;
use influx_db_client::{Client, Point, Points, Value, Precision};
use serde_json::Value::Number;

#[derive(Debug)]
pub struct DachsData
{
    timestamp: i64,
    runtime: f64,
    total_energy: f64,
}

impl DachsData
{
    const SERIES_NAME: &'static str = "dachs";

    pub fn new(timestamp: i64, runtime: f64, energy: f64) -> DachsData
    {
        return DachsData
        {
            timestamp: timestamp,
            runtime: runtime,
            total_energy: energy
        };
    }

    pub fn first(conn: &Client) -> Result<DachsData, String>
    {
        let mut queried = DachsData::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY ASC LIMIT 1",
            DachsData::SERIES_NAME))?;
        // TODO: validate only 1 received
        return Ok(queried.pop().unwrap());
    }

    pub fn last(conn: &Client) -> Result<DachsData, String>
    {
        let mut queried = DachsData::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY DESC LIMIT 1",
            DachsData::SERIES_NAME))?;
        // TODO: validate only 1 received
        return Ok(queried.pop().unwrap());
    }

    pub fn save(&self, conn: &Client) -> Result<(), String>
    {
        // TODO: write multiple points in one call

        // TODO: correct error handling
        let mut measurement = Point::new(DachsData::SERIES_NAME);
        measurement.add_timestamp(self.timestamp);
        measurement.add_field("total", Value::Float(self.total_energy));
        measurement.add_field("runtime", Value::Float(self.runtime));
        conn.write_point(measurement, Some(Precision::Seconds), None).
            expect("💩️ influx");
        println!("wrote {:?} to influx", self);

        return Ok(());
    }

    fn load(conn: &Client, query: String)
        -> Result<Vec<DachsData>, String>
    {
        let mut queried = match conn.query(&query,
            Some(Precision::Seconds))
        {
            Ok(x) => match x
            {
                None => return Err("nothing received".to_string()),
                Some(x) => x
            },
            Err(e) => return Err(format!("query error {}", e))
        };

        // TODO: dont unwrap, expect or panic
        let mut series = match queried.pop().unwrap().series
        {
            None => panic!("no series"),
            Some(mut x) => x.pop().unwrap()
        };

        // TODO: use logger for this
        println!("Got series {}", series.name);
        // TODO: how to tag unused
        let mut mapping = (1, 1, 1);
        for (i, col_name) in series.columns.iter().enumerate()
        {
            println!("column {} is {}", i, col_name);
            match col_name.as_ref()
            {
                "time" => mapping.0 = i,
                "runtime" => mapping.1 = i,
                "total" => mapping.2 = i,
                _ => panic!("error")
            };
        }
        // TODO: validate that there are no -1 in tuple

        println!("mapping: {:?}", mapping);
        println!("values: {:?}", series.values);

        let data: Vec<DachsData> = series.values.into_iter().map(|val|
        {
            let timestamp: i64 = match &val[mapping.0]
            {
                Number(x) => x.as_i64().unwrap(),
                _ => panic!("serde")
            };
            let runtime: f64 = match &val[mapping.1]
            {
                Number(x) => x.as_f64().unwrap(),
                _ => panic!("serde")
            };
            let total_energy: f64 = match &val[mapping.2]
            {
                Number(x) => x.as_f64().unwrap(),
                _ => panic!("serde")
            };
            return DachsData
            {
                timestamp: timestamp,
                runtime: runtime,
                total_energy: total_energy
            };
        }).collect();

        println!("{:?}", data);
        return Ok(data);
    }
}
