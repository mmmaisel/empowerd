/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
extern crate chrono;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, Offset};

#[derive(Debug)]
pub struct Data {
    pub timestamp: u32,
    pub temperature_in: f32,
    pub humidity_in: u8,
    pub temperature_out: Option<f32>,
    pub humidity_out: Option<u8>,
    pub rain_day: f32,
    pub rain_actual: f32,
    pub wind_actual: f32,
    pub wind_gust: f32,
    pub wind_dir: u16,
    pub baro_sea: u16,
    pub baro_absolute: u16,
    pub uv_index: f32,
    pub dew_point: Option<f32>,
    pub temperature_x1: Option<f32>,
    pub humidity_x1: Option<u8>,
    pub temperature_x2: Option<f32>,
    pub humidity_x2: Option<u8>,
    pub temperature_x3: Option<f32>,
    pub humidity_x3: Option<u8>,
}

impl Data {
    pub fn from_string(message: &str) -> Result<Data, String> {
        let mut tokens = message.split_whitespace();
        let _ = tokens.next();

        let date = match tokens.next() {
            Some(x) => match NaiveDate::parse_from_str(x, "%Y-%m-%d") {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'date' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let time = match tokens.next() {
            Some(x) => match NaiveTime::parse_from_str(x, "%H:%M") {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'time' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let local_utc_offset =
            Local::now().date().offset().fix().local_minus_utc() as i64;
        let timestamp =
            NaiveDateTime::new(date, time).timestamp() - local_utc_offset;

        let temperature_in = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'temperature_in' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let humidity_in = match tokens.next() {
            Some(x) => match x.parse::<u8>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'humidity_in' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let temperature_out = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => Some(y),
                Err(_e) => None,
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let humidity_out = match tokens.next() {
            Some(x) => match x.parse::<u8>() {
                Ok(y) => Some(y),
                Err(_e) => None,
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let rain_day = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'rain_day' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let rain_actual = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'rain_actual' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let wind_actual = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'wind_actual' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let wind_gust = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'wind_gust' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let wind_dir = match tokens.next() {
            Some(x) => match x.parse::<u16>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'wind_dir' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        if let None = tokens.next() {
            return Err("Unexpected end of data found.".to_string());
        }

        let baro_sea = match tokens.next() {
            Some(x) => match x.parse::<u16>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'baro_sea' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let baro_absolute = match tokens.next() {
            Some(x) => match x.parse::<u16>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'baro_absolute' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let uv_index = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => y,
                Err(e) => {
                    return Err(format!(
                        "Could not parse 'uv_index' from '{}': {}, {}",
                        message,
                        e.to_string(),
                        x
                    ))
                }
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let dew_point = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => Some(y),
                Err(_e) => None,
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let _unknown = tokens.next();

        let temperature_x1 = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => Some(y),
                Err(_) => None,
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let humidity_x1 = match tokens.next() {
            Some(x) => match x.parse::<u8>() {
                Ok(y) => Some(y),
                Err(_) => None,
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let temperature_x2 = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => Some(y),
                Err(_) => None,
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let humidity_x2 = match tokens.next() {
            Some(x) => match x.parse::<u8>() {
                Ok(y) => Some(y),
                Err(_) => None,
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let temperature_x3 = match tokens.next() {
            Some(x) => match x.parse::<f32>() {
                Ok(y) => Some(y),
                Err(_) => None,
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        let humidity_x3 = match tokens.next() {
            Some(x) => match x.parse::<u8>() {
                Ok(y) => Some(y),
                Err(_) => None,
            },
            None => return Err("Unexpected end of data found.".to_string()),
        };

        for _ in 0..8 {
            if let None = tokens.next() {
                return Err("Unexpected end of data found.".to_string());
            }
        }

        if let Some(_end) = tokens.next() {
            return Err("garbage at end of string".to_string());
        }

        return Ok(Data {
            timestamp: timestamp as u32,
            temperature_in: temperature_in,
            humidity_in: humidity_in,
            temperature_out: temperature_out,
            humidity_out: humidity_out,
            rain_day: rain_day,
            rain_actual: rain_actual,
            wind_actual: wind_actual,
            wind_gust: wind_gust,
            wind_dir: wind_dir,
            baro_sea: baro_sea,
            baro_absolute: baro_absolute,
            uv_index: uv_index,
            dew_point: dew_point,
            temperature_x1: temperature_x1,
            humidity_x1: humidity_x1,
            temperature_x2: temperature_x2,
            humidity_x2: humidity_x2,
            temperature_x3: temperature_x3,
            humidity_x3: humidity_x3,
        });
    }
}
