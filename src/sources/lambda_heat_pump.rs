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
use crate::models::{Heatpump, InfluxResult};
use crate::task_group::TaskResult;
use chrono::{DateTime, Utc};
use lambda_client::LambdaClient;
use slog::{error, trace};
use std::collections::VecDeque;
use std::time::{Duration, UNIX_EPOCH};

pub struct LambdaHeatPumpSource {
    base: SourceBase,
    client: LambdaClient,
    count: u64,
    energy: f64,
    cops: VecDeque<i16>,
}

impl LambdaHeatPumpSource {
    pub fn new(base: SourceBase, address: String) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 502)?;
        let client = LambdaClient::new(address);
        let cops = vec![0; base.oversample_factor as usize].into();

        Ok(Self {
            base,
            client,
            count: 0,
            energy: 0.0,
            cops,
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

                match context.get_cop().await {
                    Ok(x) => {
                        self.cops.pop_front();
                        self.cops.push_back(x);
                    }
                    Err(e) => {
                        error!(
                            self.base.logger,
                            "Query Lambda Heat Pump data failed: {}", e
                        );
                        return Err(());
                    }
                }

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
            let last_record = match self.base.query_last::<Heatpump>().await {
                InfluxResult::Some(last_record) => {
                    trace!(
                        self.base.logger,
                        "Read {:?} from database",
                        last_record
                    );
                    last_record
                }
                InfluxResult::None => Heatpump::new(
                    DateTime::<Utc>::from(
                        UNIX_EPOCH
                            + Duration::from_secs(
                                timing.now - self.base.interval.as_secs(),
                            ),
                    ),
                    0.0,
                    0.0,
                    None,
                    None,
                    None,
                    None,
                    None,
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
            let energy_wh = self.energy / 3.6e3 + last_record.energy;
            let cop = self.cops.iter().map(|x| *x as i32).sum::<i32>() as f64
                / self.cops.len() as f64
                / 100.0;

            let mut context = match self.client.open().await {
                Ok(x) => x,
                Err(e) => {
                    error!(
                        self.base.logger,
                        "Could not connect to Lambda Heat Pump: {}", e
                    );
                    return TaskResult::Running;
                }
            };
            let boiler = match context.get_boiler_temps().await {
                Ok(x) => x,
                Err(e) => {
                    error!(
                        self.base.logger,
                        "Query Lambda Heat Pump data failed: {}", e
                    );
                    return TaskResult::Running;
                }
            };

            // Commit new sample to database.
            let record = Heatpump::new(
                DateTime::<Utc>::from(
                    UNIX_EPOCH + Duration::from_secs(timing.now),
                ),
                energy_wh,
                (energy_wh - last_record.energy) * 3.6e3
                    / ((timing.now - last_record.time.timestamp() as u64)
                        as f64),
                Some(
                    last_record.total_heat.unwrap_or(0.0)
                        + (energy_wh - last_record.energy) * cop,
                ),
                Some(cop),
                Some(boiler.0 as f64 / 10.0),
                Some(boiler.1 as f64 / 10.0),
                Some(boiler.2 as f64 / 10.0),
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
