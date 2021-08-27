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
#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

use lazy_static::lazy_static;
use regex::Regex;
use slog::{Logger, trace};

#[cfg(test)]
mod tests;

pub struct DachsClient {
    client: reqwest::Client,
    url: String,
    password: String,
    logger: Option<Logger>,
}

lazy_static! {
    static ref EXTR_TOTAL_ENERGY: regex::Regex =
        Regex::new(r"Hka_Bd\.ulArbeitElektr=([0-9]+\.[0-9]+)\s").unwrap();
    static ref EXTR_RUNTIME: regex::Regex =
        Regex::new(r"Hka_Bd\.ulBetriebssekunden=([0-9]+\.[0-9]+)\s").unwrap();
}

impl DachsClient {
    const USERNAME: &'static str = "glt";
    const KEY_TOTAL_ENERGY: &'static str = "Hka_Bd.ulArbeitElektr";
    const KEY_RUNTIME: &'static str = "Hka_Bd.ulBetriebssekunden";

    pub fn new(
        url: String,
        password: String,
        logger: Option<Logger>,
    ) -> DachsClient {
        let client = reqwest::Client::new();
        return DachsClient {
            client: client,
            url: format!("http://{}:8080", url),
            password: password,
            logger: logger,
        };
    }

    async fn query_glt(&self, key: &str) -> Result<String, String> {
        let result = self
            .client
            .get(&format!("{url}/getKey?k={key}", url = self.url, key = key))
            .basic_auth(DachsClient::USERNAME, Some(&self.password))
            .send()
            .await;

        let text = match result {
            Ok(result) => match result.status() {
                reqwest::StatusCode::OK => result.text().await,
                reqwest::StatusCode::UNAUTHORIZED => {
                    return Err("💩️ Unauthorized".to_string());
                }
                _ => {
                    return Err(
                        "💩️ Dachs GLT API returned an error".to_string()
                    );
                }
            },
            Err(_) => {
                return Err("💩️ Querying Dachs GLT API failed.".to_string())
            }
        };

        return match text {
            Ok(text) => {
                match &self.logger {
                    Some(x) => trace!(x, "query_glt {}: {}", key, &text),
                    None => (),
                }
                Ok(text)
            }
            Err(_) => {
                return Err(
                    "💩️ Decoding Dachs GLT API response failed.".to_string()
                )
            }
        };
    }

    pub async fn get_total_energy(&self) -> Result<i32, String> {
        let result = self.query_glt(DachsClient::KEY_TOTAL_ENERGY).await?;
        let extracted_val = match EXTR_TOTAL_ENERGY.captures(&result) {
            Some(x) => x,
            None => {
                return Err("💩️ Parsing Dachs total energy failed.".to_string())
            }
        };

        let energy: f64 =
            extracted_val.get(1).unwrap().as_str().parse().unwrap();

        match &self.logger {
            Some(x) => trace!(x, "Energy f64: {}", &energy),
            None => (),
        }
        return Ok(energy as i32);
    }

    pub async fn get_runtime(&self) -> Result<i32, String> {
        let result = self.query_glt(DachsClient::KEY_RUNTIME).await?;
        let extracted_val = match EXTR_RUNTIME.captures(&result) {
            Some(x) => x,
            None => return Err("💩️ Parsing Dachs runtime failed.".to_string()),
        };

        let runtime_h: f64 =
            extracted_val.get(1).unwrap().as_str().parse().unwrap();

        match &self.logger {
            Some(x) => trace!(x, "Runtime h: {}", &runtime_h),
            None => (),
        }
        return Ok((runtime_h * 3600.0) as i32);
    }
}
