#[derive(Debug)]
pub struct Data
{
    pub timestamp: u32,     // format??
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
    pub dew_point: f32
}

impl Data {
    pub fn from_string(message: String) -> Result<Data, String> {
        let mut tokens = message.split_whitespace();
        let _ = tokens.next();

        let date = match tokens.next() {
            Some(x) => 0, // TODO: convert it
            None => return Err("Unexpected end of data found.".to_string())
        };

        let time = match tokens.next() {
            Some(x) => 0, // TODO: convert it
            None => return Err("Unexpected end of data found.".to_string())
        };

        let temperature_in = match tokens.next() {
            Some(x) => {
                match x.parse::<f32>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let humidity_in = match tokens.next() {
            Some(x) => {
                match x.parse::<u8>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let temperature_out = match tokens.next() {
            Some(x) => {
                match x.parse::<f32>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let humidity_out = match tokens.next() {
            Some(x) => {
                match x.parse::<u8>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let rain_day = match tokens.next() {
            Some(x) => {
                match x.parse::<f32>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let rain_actual = match tokens.next() {
            Some(x) => {
                match x.parse::<f32>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let wind_actual = match tokens.next() {
            Some(x) => {
                match x.parse::<f32>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let wind_gust = match tokens.next() {
            Some(x) => {
                match x.parse::<f32>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let wind_dir = match tokens.next() {
            Some(x) => {
                match x.parse::<u16>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        if let None = tokens.next() {
            return Err("Unexpected end of data found.".to_string());
        }

        let baro_sea = match tokens.next() {
            Some(x) => {
                match x.parse::<u16>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let baro_absolute = match tokens.next() {
            Some(x) => {
                match x.parse::<u16>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let uv_index = match tokens.next() {
            Some(x) => {
                match x.parse::<f32>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        let dew_point = match tokens.next() {
            Some(x) => {
                match x.parse::<f32>() {
                    Ok(y) => y,
                    Err(e) => return Err(format!("{}, {}", e.to_string(), x))
                }
            }
            None => return Err("Unexpected end of data found.".to_string())
        };

        for _ in 0..15 {
            if let None = tokens.next() {
                return Err("Unexpected end of data found.".to_string())
            }
        }

        if let Some(_end) = tokens.next() {
            return Err("garbage at end of string".to_string());
        }

        return Ok(Data {
            timestamp: 0, // TODO
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
            dew_point: dew_point
        });
    }
}
