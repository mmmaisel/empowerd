/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2022 Max Maisel

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
use bresser6in1_usb::{Client as BresserClient, PID, VID};
use slog::{debug, error, warn, Logger};
use std::time::Duration;

pub struct Bresser6in1Source {
    base: SourceBase,
    bresser_client: BresserClient,
}

impl Bresser6in1Source {
    pub fn new(base: SourceBase) -> Self {
        let logger = base.logger.clone();
        Self {
            base,
            bresser_client: BresserClient::new(Some(logger)),
        }
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let mut conn = self.base.database.get().await.map_err(|e| {
            Error::Temporary(format!(
                "Getting database connection from pool failed: {e}",
            ))
        })?;

        let mut weather_data = tokio::task::block_in_place(|| {
            let mut weather_data = self.bresser_client.read_data();
            for i in 1..4u8 {
                if let Err(e) = weather_data {
                    if i == 2 {
                        match usb_reset::reset_vid_pid(VID, PID) {
                            Ok(()) => {
                                warn!(
                                    self.base.logger,
                                    "Reset device {VID:X}:{PID:X}",
                                );
                                std::thread::sleep(Duration::from_secs(5));
                            }
                            Err(e) => {
                                error!(
                                    self.base.logger,
                                    "Reset device {VID:X}:{PID:X} failed: {e}",
                                );
                            }
                        }
                    }
                    debug!(
                        self.base.logger,
                        "Get weather data failed, {e}, retrying...",
                    );
                    weather_data = self.bresser_client.read_data();
                } else {
                    break;
                }
            }
            return weather_data;
        })
        .map_err(|e| {
            Error::Temporary(format!(
                "Get weather data failed, {e}, giving up!",
            ))
        })?;
        weather_data.timestamp = timing.now as u32;

        let record = Weather::new(weather_data);
        self.base.notify_processors(&record);
        record
            .insert(&mut conn, self.base.series_id)
            .await
            .map_err(|e| {
                Error::Temporary(format!(
                    "Inserting {} record into database failed: {}",
                    &self.base.name, e,
                ))
            })?;

        Ok(())
    }
}
