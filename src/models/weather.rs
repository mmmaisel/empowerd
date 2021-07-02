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
    pub humidity_in: f32,
    pub temperature_out: Option<f32>,
    pub humidity_out: Option<f32>,
    pub rain_day: f32,
    pub rain_actual: f32,
    pub wind_actual: f32,
    pub wind_gust: f32,
    pub wind_dir: f32,
    pub baro_sea: f32,
    pub baro_absolute: f32,
    pub uv_index: f32,
    pub dew_point: Option<f32>,
    pub temperature_x1: Option<f32>,
    pub humidity_x1: Option<f32>,
    pub temperature_x2: Option<f32>,
    pub humidity_x2: Option<f32>,
    pub temperature_x3: Option<f32>,
    pub humidity_x3: Option<f32>,
}

impl Weather {
    pub fn new(data: BresserData) -> Weather {
        return Weather {
            time: DateTime::<Utc>::from(
                UNIX_EPOCH + Duration::from_secs(data.timestamp as u64),
            ),
            temperature_in: data.temperature_in,
            humidity_in: data.humidity_in as f32,
            temperature_out: data.temperature_out.map(|x| x as f32),
            humidity_out: data.humidity_out.map(|x| x as f32),
            rain_day: data.rain_day,
            rain_actual: data.rain_actual,
            wind_actual: data.wind_actual,
            wind_gust: data.wind_gust,
            wind_dir: data.wind_dir as f32,
            baro_sea: data.baro_sea as f32,
            baro_absolute: data.baro_absolute as f32,
            uv_index: data.uv_index,
            dew_point: data.dew_point,
            temperature_x1: data.temperature_x1,
            humidity_x1: data.humidity_x1.map(|x| x as f32),
            temperature_x2: data.temperature_x2,
            humidity_x2: data.humidity_x2.map(|x| x as f32),
            temperature_x3: data.temperature_x3,
            humidity_x3: data.humidity_x3.map(|x| x as f32),
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
