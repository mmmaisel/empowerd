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
use crate::models::{
    units::{second, watt, watt_hour, Abbreviation, Energy, Power, Time},
    InfluxResult, SimpleMeter,
};
use crate::task_group::TaskResult;
use slog::{debug, error, trace};
use sma_client::{SmaClient, TimestampedInt};
use std::net::SocketAddr;

pub struct SunnyBoySpeedwireSource {
    base: SourceBase,
    sma_client: SmaClient,
    sma_pw: String,
    sma_addr: SocketAddr,
}

impl SunnyBoySpeedwireSource {
    pub fn new(
        base: SourceBase,
        sma_pw: String,
        sma_addr: String,
    ) -> Result<Self, String> {
        let sma_socket_addr: SocketAddr =
            match SmaClient::sma_sock_addr(sma_addr) {
                Ok(x) => x,
                Err(e) => {
                    return Err(format!("Could not parse sma_addr: {}", e))
                }
            };

        let logger = base.logger.clone();
        Ok(Self {
            base,
            sma_client: SmaClient::new(Some(logger)),
            sma_pw: sma_pw,
            sma_addr: sma_socket_addr,
        })
    }

    // XXX: this function is much too long
    pub async fn run(&mut self) -> TaskResult {
        let (now, _oversample) = match self.base.sleep_aligned().await {
            Ok(x) => (Time::new::<second>(x.now as f64), x.oversample),
            Err(e) => return e,
        };

        let last_record = match self.base.query_last::<SimpleMeter>().await {
            InfluxResult::Some(x) => x,
            InfluxResult::None => SimpleMeter::new(
                Time::new::<second>(0.0),
                Energy::new::<watt_hour>(0.0),
                Power::new::<watt>(0.0),
            ),
            InfluxResult::Err(e) => {
                error!(
                    self.base.logger,
                    "Query {} database failed: {}", &self.base.name, e
                );
                return TaskResult::Running;
            }
        };
        let session = match self.sma_client.open(self.sma_addr, None) {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.base.logger,
                    "Could not open SMA Client session: {}", e
                );
                return TaskResult::Running;
            }
        };
        let identity = match self.sma_client.identify(&session).await {
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

        self.sma_client.set_dst(identity.susy_id, identity.serial);

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
            last_record.time.into_format_args(second, Abbreviation),
            now.into_format_args(second, Abbreviation),
        );

        let mut last_time = last_record.time;
        let mut last_energy = last_record.energy;

        // TODO: this command is not accepted by SMA, needs -86400 ?
        //   this data is delayed by about one hour?
        let points: Vec<TimestampedInt> = match self
            .sma_client
            .get_day_data(
                &session,
                last_time.get::<second>() as u32,
                now.get::<second>() as u32,
            )
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
                        let energy =
                            Energy::new::<watt_hour>(point.value as f64);
                        if point.timestamp as u64
                            == last_time.get::<second>() as u64
                        {
                            return false;
                        } else if energy < last_energy {
                            // Sometimes, the last value from speedwire is just garbage.
                            debug!(
                                self.base.logger,
                                "Energy meter run backwards. Ignoring point."
                            );
                            return false;
                        } else if point.value as u32 == 0xFFFFFFFF {
                            debug!(self.base.logger, "Skipping NaN SMA record");
                            return false;
                        } else {
                            last_energy = energy;
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
        last_energy = last_record.energy;

        if let Err(e) = self.sma_client.logout(&session).await {
            error!(self.base.logger, "Logout failed: {}", e);
        }

        let num_points = points.len();
        for point in points {
            let time = Time::new::<second>(point.timestamp as f64);
            let energy = Energy::new::<watt_hour>(point.value as f64);
            let power = if last_time.get::<second>() as i64 == 0 {
                Power::new::<watt>(0.0)
            } else {
                (energy - last_energy) / (time - last_time)
            };

            let record = SimpleMeter::new(time, energy, power);
            last_energy = energy;
            last_time = time;

            self.base.notify_processors(&record);
            if self.base.save_record(record).await.is_err() {
                return TaskResult::Running;
            }
        }

        trace!(
            self.base.logger,
            "Wrote {} simple meter records to database",
            num_points
        );
        TaskResult::Running
    }
}
