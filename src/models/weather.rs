use super::InfluxObject;
use bresser6in1_usb::Data as BresserData;
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use serde::Deserialize;
use std::time::{Duration, UNIX_EPOCH};

#[derive(Deserialize, Debug, InfluxDbWriteable)]
pub struct Weather {
    pub time: DateTime<Utc>,
    pub temperature_in: f32,
    pub humidity_in: u8,
    pub temperature_out: f32,
    pub humidity_out: u8,
    pub rain_day: f32,
    pub rain_actual: f32,
    pub wind_actual: f32,
    pub wind_gust: f32,
    pub wind_dir: u16,
    pub baro_sea: u16,
    pub baro_absolute: u16,
    pub uv_index: f32,
    pub dew_point: f32,
    pub temperature_x1: Option<f32>,
    pub humidity_x1: Option<u8>,
    pub temperature_x2: Option<f32>,
    pub humidity_x2: Option<u8>,
    pub temperature_x3: Option<f32>,
    pub humidity_x3: Option<u8>,
}

impl Weather {
    pub fn new(data: BresserData) -> Weather {
        return Weather {
            time: DateTime::<Utc>::from(
                UNIX_EPOCH + Duration::from_secs(data.timestamp as u64),
            ),
            temperature_in: data.temperature_in,
            humidity_in: data.humidity_in,
            temperature_out: data.temperature_out,
            humidity_out: data.humidity_out,
            rain_day: data.rain_day,
            rain_actual: data.rain_actual,
            wind_actual: data.wind_actual,
            wind_gust: data.wind_gust,
            wind_dir: data.wind_dir,
            baro_sea: data.baro_sea,
            baro_absolute: data.baro_absolute,
            uv_index: data.uv_index,
            dew_point: data.dew_point,
            temperature_x1: data.temperature_x1,
            humidity_x1: data.humidity_x1,
            temperature_x2: data.temperature_x2,
            humidity_x2: data.humidity_x2,
            temperature_x3: data.temperature_x3,
            humidity_x3: data.humidity_x3,
        };
    }
}

impl InfluxObject<Weather> for Weather {
    const FIELDS: &'static str = "temperature_in, humidity_in, \
        temperature_out, humidity_out, rain_day, rain_actual, \
        wind_actual, wind_gust, wind_dir, baro_sea, baro_absolute, \
        uv_index, dew_point, temperature_x1, humidity_x1, \
        temperature_x2, humidity_x2, temperature_x3, humidity_x3";
    const MEASUREMENT: &'static str = "weather";
}
