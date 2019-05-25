extern crate influx_db_client;
extern crate serde_json;

use influx_db_client::{Client, Point, Points, Precision, Series, Value};
pub mod dachs;
pub mod solar;

// TODO: this shall return series for derive
fn load_series(conn: &Client, query: String) -> Result<Series, String>
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

    // TODO: this is ugly, use and_then?
    let series = match queried.pop()
    {
        None => return Err("no query results".to_string()),
        Some(x) => match x.series
        {
            None => return Err("no series".to_string()),
            Some(mut x) => match x.pop()
            {
                None => return Err("no series".to_string()),
                Some(x) => x
            }
        }
    };
    println!("{:?}", series);
    return Ok(series);
}
