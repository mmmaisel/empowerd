extern crate influx_db_client;
extern crate serde_json;

//use futures::future::Future;
use influx_db_client::{Client, Point, Points, Value, Precision};
use serde_json::Value::Number;

#[derive(Debug)]
pub struct SolarData
{
    timestamp: i64,
    power: f64,
    total_energy: f64,
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
            total_energy: energy
        };
    }

    pub fn first(conn: &Client) -> Result<SolarData, String>
    {
        let mut queried = SolarData::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY ASC LIMIT 1",
            SolarData::SERIES_NAME))?;
        // TODO: validate only 1 received
        return Ok(queried.pop().unwrap());
    }

    pub fn last(conn: &Client) -> Result<SolarData, String>
    {
        let mut queried = SolarData::load(conn, format!(
            "SELECT * FROM \"{}\" GROUP BY * ORDER BY DESC LIMIT 1",
            SolarData::SERIES_NAME))?;
        // TODO: validate only 1 received
        return Ok(queried.pop().unwrap());
    }

    // TODO: deduplicate all below
    fn to_point(&self) -> Point
    {
        let mut point = Point::new(SolarData::SERIES_NAME);
        point.add_timestamp(self.timestamp);
        point.add_field("total", Value::Float(self.total_energy));
        point.add_field("power", Value::Float(self.power));
        return point;
    }

    pub fn save(&self, conn: &Client) -> Result<(), String>
    {
        // TODO: correct error handling
        conn.write_point(self.to_point(), Some(Precision::Seconds), None).
            expect("💩️ influx");
        println!("wrote {:?} to influx", self);

        return Ok(());
    }

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

    fn load(conn: &Client, query: String)
        -> Result<Vec<SolarData>, String>
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
                "power" => mapping.1 = i,
                "total" => mapping.2 = i,
                _ => panic!("error")
            };
        }
        // TODO: validate that there are no -1 in tuple

        println!("mapping: {:?}", mapping);
        println!("values: {:?}", series.values);

        let data: Vec<SolarData> = series.values.into_iter().map(|val|
        {
            let timestamp: i64 = match &val[mapping.0]
            {
                Number(x) => x.as_i64().unwrap(),
                _ => return panic!("serde")
            };
            let power: f64 = match &val[mapping.1]
            {
                Number(x) => x.as_f64().unwrap(),
                _ => return panic!("serde")
            };
            let total_energy: f64 = match &val[mapping.2]
            {
                Number(x) => x.as_f64().unwrap(),
                _ => return panic!("serde")
            };
            return SolarData
            {
                timestamp: timestamp,
                power: power,
                total_energy: total_energy
            };
        }).collect();

        println!("{:?}", data);
        return Ok(data);
    }
}
