extern crate influx_db_client;
extern crate serde_json;

use std::fmt;

use influx_db_client::{Client, Precision, Series};
pub mod dachs;
pub mod solar;

pub use dachs::*;
pub use solar::*;

pub struct LoadError
{
    series_exists: bool,
    message: String
}

impl LoadError
{
    fn new(msg: String) -> LoadError
    {
        return LoadError
        {
            series_exists: true,
            message: msg
        };
    }

    fn no_series() -> LoadError
    {
        return LoadError
        {
            series_exists: false,
            message: String::new()
        };
    }

    pub fn series_exists(&self) -> bool
    {
        return self.series_exists;
    }
}

impl fmt::Display for LoadError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        return write!(f, "{}", self.message);
    }
}

fn load_series(conn: &Client, query: String) -> Result<Series, LoadError>
{
    let mut queried = match conn.query(&query,
        Some(Precision::Seconds))
    {
        Ok(x) => match x
        {
            None => return Err(LoadError::new("nothing received".to_string())),
            Some(x) => x
        },
        Err(e) => return Err(LoadError::new(format!("query error {}", e)))
    };

    // TODO: this is ugly, use and_then?
    let series = match queried.pop()
    {
        None => return Err(LoadError::new("no query results".to_string())),
        Some(x) => match x.series
        {
            None => return Err(LoadError::no_series()),
            Some(mut x) => match x.pop()
            {
                None => return Err(LoadError::no_series()),
                Some(x) => x
            }
        }
    };
    println!("{:?}", series);
    return Ok(series);
}
