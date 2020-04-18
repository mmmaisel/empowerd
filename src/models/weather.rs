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
}

impl WeatherData
{
    pub fn new(timestamp: i64, data: BresserData) -> WeatherData
    {
        return WeatherData
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
        };
    }
}
