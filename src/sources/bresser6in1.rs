/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2025 Max Maisel

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
use super::SourceBase;
use crate::{models::Weather, task_group::TaskResult, Error};
use slog::{error, warn, Logger};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, timeout};
use ws6in1_proto::{client::Ws6in1Client, parser::Ws6in1Data};

pub struct Bresser6in1Source {
    base: SourceBase,
    last_sync: u64,
}

impl Bresser6in1Source {
    pub fn new(base: SourceBase) -> Self {
        Self { base, last_sync: 0 }
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    async fn read_data(
        &mut self,
        client: &mut Ws6in1Client,
    ) -> Result<Ws6in1Data, String> {
        match timeout(Duration::from_secs(20), client.read_weather_data()).await
        {
            Ok(Ok(data)) => Ok(data),
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn read_data_with_retry(
        &mut self,
        client: &mut Ws6in1Client,
    ) -> Result<Ws6in1Data, Error> {
        let mut weather_data = self.read_data(client).await;

        for i in 1..4u8 {
            if let Ok(data) = weather_data {
                return Ok(data);
            }

            if i == 2 {
                match usb_reset::reset_vid_pid(
                    Ws6in1Client::VENDOR_ID,
                    Ws6in1Client::PRODUCT_ID,
                ) {
                    Ok(()) => {
                        warn!(
                            self.base.logger,
                            "Reset device {:X}:{:X}",
                            Ws6in1Client::VENDOR_ID,
                            Ws6in1Client::PRODUCT_ID
                        );
                        sleep(Duration::from_secs(5)).await;
                    }
                    Err(e) => {
                        error!(
                            self.base.logger,
                            "Reset device {:X}:{:X} failed: {e}",
                            Ws6in1Client::VENDOR_ID,
                            Ws6in1Client::PRODUCT_ID
                        );
                    }
                }
            }

            weather_data = self.read_data(client).await;
        }

        weather_data.map_err(|e| {
            Error::Temporary(format!(
                "Get weather data failed, {e}, giving up!",
            ))
        })
    }

    async fn sync_datetime(&mut self, client: &mut Ws6in1Client) -> TaskResult {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| {
                Error::Temporary(format!("Getting system time failed: {e}"))
            })?
            .as_secs();
        client.write_datetime(now as i64).await.map_err(|e| {
            Error::Temporary(format!("Synchronizing Ws6in1 time failed: {e}"))
        })?;

        self.last_sync = now;
        Ok(())
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let mut conn = self.base.get_database().await?;

        let mut client = Ws6in1Client::new().await.map_err(|e| {
            Error::Temporary(format!("Open Ws6in1 client failed: {e}"))
        })?;
        let mut weather_data = self.read_data_with_retry(&mut client).await?;
        weather_data.local_timestamp = timing.now as i64;

        let record = Weather::new(weather_data);
        self.base.notify_processors(&record);
        record.insert(&mut conn, self.base.series_id).await?;

        if timing.now > self.last_sync + 3600 {
            self.sync_datetime(&mut client).await?;
        }

        Ok(())
    }
}
