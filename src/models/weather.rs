extern crate influx_db_client;
extern crate serde_json;

use influx_db_client::Points;

use super::*;
use influx_derive::InfluxData;

use bresser6in1_usb::{Data as BresserData};

#[derive(Debug, InfluxData)]
#[influx(measurement_name = "weather")]
pub struct WeatherData
{
    pub timestamp: i64,
    pub temperature_in: f64,
    pub humidity_in: f64,
    pub temperature_out: f64,
    pub humidity_out: f64,
    pub rain_day: f64,
    pub rain_actual: f64,
    pub wind_actual: f64,
    pub wind_gust: f64,
    pub wind_dir: f64,
    pub baro_sea: f64,
    pub baro_absolute: f64,
    pub uv_index: f64,
    pub dew_point: f64,
    // TODO: add support for Option<> in influx-derive
    pub temperature_x1: f64,
    pub humidity_x1: f64,
    pub temperature_x2: f64,
    pub humidity_x2: f64,
    pub temperature_x3: f64,
    pub humidity_x3: f64,
}

impl WeatherData
{
    pub fn new(timestamp: i64, data: BresserData) -> Result<WeatherData, String>
    {
        let temperature_x1 = match data.temperature_x1 {
            Some(x) => x,
            None => return Err(format!(
                "temperature_x1 is None in given {:?}", data)),
        };
        let humidity_x1 = match data.humidity_x1 {
            Some(x) => x,
            None => return Err(format!(
                "humidity_x1 is None in given {:?}", data)),
        };
        let temperature_x2 = match data.temperature_x2 {
            Some(x) => x,
            None => return Err(format!(
                "temperature_x2 is None in given {:?}", data)),
        };
        let humidity_x2 = match data.humidity_x2 {
            Some(x) => x,
            None => return Err(format!(
                "humidity_x2 is None in given {:?}", data)),
        };
        let temperature_x3 = match data.temperature_x3 {
            Some(x) => x,
            None => return Err(format!(
                "temperature_x3 is None in given {:?}", data)),
        };
        let humidity_x3 = match data.humidity_x3 {
            Some(x) => x,
            None => return Err(format!(
                "humidity_x3 is None in given {:?}", data)),
        };

        return Ok(WeatherData
        {
            timestamp: timestamp,
            temperature_in: data.temperature_in as f64,
            humidity_in: data.humidity_in as f64,
            temperature_out: data.temperature_out as f64,
            humidity_out: data.humidity_out as f64,
            rain_day: data.rain_day as f64,
            rain_actual: data.rain_actual as f64,
            wind_actual: data.wind_actual as f64,
            wind_gust: data.wind_gust as f64,
            wind_dir: data.wind_dir as f64,
            baro_sea: data.baro_sea as f64,
            baro_absolute: data.baro_absolute as f64,
            uv_index: data.uv_index as f64,
            dew_point: data.dew_point as f64,
            temperature_x1: temperature_x1 as f64,
            humidity_x1: humidity_x1 as f64,
            temperature_x2: temperature_x2 as f64,
            humidity_x2: humidity_x2 as f64,
            temperature_x3: temperature_x3 as f64,
            humidity_x3: humidity_x3 as f64,
        });
    }
}
