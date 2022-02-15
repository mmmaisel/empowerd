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
use crate::models::{InfluxObject, InfluxResult, SimpleMeter};
use crate::task_group::{TaskResult, TaskState};
use chrono::{DateTime, Utc};
use slog::{debug, error, trace, Logger};
use sma_client::{SmaClient, TimestampedInt};
use std::net::SocketAddr;
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::watch;

pub struct SunnyBoySpeedwireSource {
    base: SourceBase,
    sma_client: SmaClient,
    sma_pw: String,
    sma_addr: SocketAddr,
}

impl SunnyBoySpeedwireSource {
    pub fn new(
        canceled: watch::Receiver<TaskState>,
        influx: influxdb::Client,
        name: String,
        interval: Duration,
        sma_pw: String,
        sma_addr: String,
        logger: Logger,
    ) -> Result<Self, String> {
        let sma_socket_addr: SocketAddr =
            match SmaClient::sma_sock_addr(sma_addr) {
                Ok(x) => x,
                Err(e) => {
                    return Err(format!("Could not parse sma_addr: {}", e))
                }
            };

        Ok(Self {
            base: SourceBase::new(
                canceled,
                influx,
                name,
                interval,
                logger.clone(),
            ),
            sma_client: SmaClient::new(Some(logger)),
            sma_pw: sma_pw,
            sma_addr: sma_socket_addr,
        })
    }

    // XXX: this function is much too long
    pub async fn run(&mut self) -> TaskResult {
        let now = match self.base.sleep_aligned().await {
            Ok(x) => x,
            Err(e) => return e,
        };

        let last_record = match SimpleMeter::into_single(
            self.base
                .influx
                .json_query(SimpleMeter::query_last(&self.base.name))
                .await,
        ) {
            InfluxResult::Some(x) => x,
            InfluxResult::None => {
                SimpleMeter::new(DateTime::<Utc>::from(UNIX_EPOCH), 0.0, 0.0)
            }
            InfluxResult::Err(e) => {
                error!(
                    self.base.logger,
                    "Query {} database failed: {}", &self.base.name, e
                );
                return TaskResult::Running;
            }
        };
        let session = match self.sma_client.open().await {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.base.logger,
                    "Could not open SMA Client session: {}", e
                );
                return TaskResult::Running;
            }
        };
        let identity =
            match self.sma_client.identify(&session, self.sma_addr).await {
                Err(e) => {
                    error!(
                        self.base.logger,
                        "Could not identify SMA device, {}", e
                    );
                    return TaskResult::Running;
                }
                Ok(x) => x,
            };

        trace!(
            self.base.logger,
            "{} is {:X}, {:X}",
            self.sma_addr,
            identity.susy_id,
            identity.serial
        );

        self.sma_client.set_dst(
            self.sma_addr,
            identity.susy_id,
            identity.serial,
        );

        if let Err(e) = self.sma_client.logout(&session).await {
            error!(self.base.logger, "Logout failed: {}", e);
            return TaskResult::Running;
        }
        if let Err(e) = self.sma_client.login(&session, &self.sma_pw).await {
            error!(self.base.logger, "Login failed: {}", e);
            return TaskResult::Running;
        }

        trace!(
            self.base.logger,
            "GetDayData from {} to {}",
            last_record.time,
            now
        );

        // TODO: this command is not accepted by SMA, needs -86400 ?
        //   this data is delayed by about one hour?
        let last_timestamp = last_record.time.timestamp();
        let points: Vec<TimestampedInt> = match self
            .sma_client
            .get_day_data(&session, last_timestamp as u32, now as u32)
            .await
        {
            Err(e) => {
                error!(self.base.logger, "Get Day Data failed: {}", e);
                return TaskResult::Running;
            }
            Ok(points) => {
                trace!(self.base.logger, "Get Day data returned {:?}", points);
                points
                    .into_iter()
                    .filter(|point| {
                        if point.timestamp as i64 == last_timestamp {
                            return false;
                        } else if point.value as u32 == 0xFFFFFFFF {
                            debug!(self.base.logger, "Skipping NaN SMA record");
                            return false;
                        } else {
                            return true;
                        }
                    })
                    .collect()
            }
        };

        // TODO: handle double data (identical timestamps)
        //   (handled in database?) and missing ones (delta_t > 300)
        // TODO: handle NaN (0xFFFFFFFF, 0xFFFF) values(in SMA client validators)
        // TODO: always UTC, handle DST transition

        if let Err(e) = self.sma_client.logout(&session).await {
            error!(self.base.logger, "Logout failed: {}", e);
        }

        let mut last_energy = last_record.energy as f64;
        let mut last_timestamp = last_timestamp as i64;

        // TODO: move this calculation to model
        let num_points = points.len();
        for point in points {
            // TODO: this is an ugly mess
            let power = if last_timestamp == 0 {
                0.0
            } else {
                3600.0 * ((point.value as f64) - last_energy)
                    / (((point.timestamp as i64) - last_timestamp) as f64)
            };

            let record = SimpleMeter::new(
                DateTime::<Utc>::from(
                    UNIX_EPOCH + Duration::from_secs(point.timestamp as u64),
                ),
                point.value.into(),
                power,
            );
            last_energy = point.value as f64;
            last_timestamp = point.timestamp as i64;

            if let Err(e) = self
                .base
                .influx
                .query(&record.save_query(&self.base.name))
                .await
            {
                error!(
                    self.base.logger,
                    "Save simple meter data failed, {}", e
                );
                return TaskResult::Running;
            }
        }

        trace!(
            self.base.logger,
            "Wrote {} simple meter records to database",
            num_points
        );
        return TaskResult::Running;
    }
}
