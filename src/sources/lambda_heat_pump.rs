/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2023 Max Maisel

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
use crate::misc::parse_socketaddr_with_default;
use crate::models::{InfluxResult, SimpleMeter};
use crate::task_group::TaskResult;
use chrono::{DateTime, Utc};
use lambda_client::LambdaClient;
use slog::{error, trace};
use std::time::{Duration, UNIX_EPOCH};

pub struct LambdaHeatPumpSource {
    base: SourceBase,
    client: LambdaClient,
    count: u64,
    energy: f64,
}

impl LambdaHeatPumpSource {
    pub fn new(base: SourceBase, address: String) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 502)?;
        let client = LambdaClient::new(address);

        Ok(Self {
            base,
            client,
            count: 0,
            energy: 0.0,
        })
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = match self.base.sleep_aligned().await {
            Ok(x) => x,
            Err(e) => return e,
        };

        let power = match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            async {
                let mut context = match self.client.open().await {
                    Ok(x) => x,
                    Err(e) => {
                        error!(
                            self.base.logger,
                            "Could not connect to Lambda Heat Pump: {}", e
                        );
                        return Err(());
                    }
                };

                match context.get_current_power().await {
                    Ok(x) => Ok(x),
                    Err(e) => {
                        error!(
                            self.base.logger,
                            "Query Lambda Heat Pump data failed: {}", e
                        );
                        Err(())
                    }
                }
            },
        )
        .await
        {
            Ok(result) => match result {
                Ok(x) => x as f64,
                Err(_) => return TaskResult::Running,
            },
            Err(e) => {
                error!(
                    self.base.logger,
                    "Query Lambda Heat Pump data timed out: {}", e
                );
                return TaskResult::Running;
            }
        };

        // Calculate energy from oversampled power.
        self.energy += power
            * (self.base.interval.as_secs() / self.base.oversample_factor)
                as f64;
        self.count += 1;

        if !timing.oversample {
            // Discard incomplete samples.
            if self.count != self.base.oversample_factor {
                self.reset_sample();
                return TaskResult::Running;
            }

            // Get accumulated energy from database.
            let last_record = match self.base.query_last::<SimpleMeter>().await
            {
                InfluxResult::Some(last_record) => {
                    trace!(
                        self.base.logger,
                        "Read {:?} from database",
                        last_record
                    );
                    last_record
                }
                InfluxResult::None => SimpleMeter::new(
                    DateTime::<Utc>::from(
                        UNIX_EPOCH
                            + Duration::from_secs(
                                timing.now - self.base.interval.as_secs(),
                            ),
                    ),
                    0.0,
                    0.0,
                ),
                InfluxResult::Err(e) => {
                    error!(
                        self.base.logger,
                        "Query {} database failed: {}", &self.base.name, e
                    );
                    return TaskResult::Running;
                }
            };

            // Update accumulated energy.
            self.energy += last_record.energy;

            // Commit new sample to database.
            let record = SimpleMeter::new(
                DateTime::<Utc>::from(
                    UNIX_EPOCH + Duration::from_secs(timing.now),
                ),
                self.energy,
                (self.energy - last_record.energy)
                    / ((timing.now - last_record.time.timestamp() as u64)
                        as f64),
            );
            self.base.notify_processors(&record);
            let _: Result<(), ()> = self.base.save_record(record).await;

            self.reset_sample();
        }
        TaskResult::Running
    }

    fn reset_sample(&mut self) {
        self.energy = 0.0;
        self.count = 0;
    }
}
