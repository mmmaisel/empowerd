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
use crate::models::{Generator, InfluxResult};
use crate::task_group::TaskResult;
use chrono::{DateTime, Utc};
use dachs_client::DachsClient;
use slog::{error, trace};
use std::time::{Duration, UNIX_EPOCH};

pub struct DachsMsrSSource {
    base: SourceBase,
    dachs_client: DachsClient,
}

impl DachsMsrSSource {
    pub fn new(base: SourceBase, dachs_addr: String, dachs_pw: String) -> Self {
        let logger = base.logger.clone();
        Self {
            base,
            dachs_client: DachsClient::new(dachs_addr, dachs_pw, Some(logger)),
        }
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = match self.base.sleep_aligned().await {
            Ok(x) => x,
            Err(e) => return e,
        };

        let (dachs_runtime, dachs_energy) = match tokio::time::timeout(
            std::time::Duration::from_secs(15),
            async {
                let runtime = match self.dachs_client.get_runtime().await {
                    Ok(runtime) => {
                        trace!(self.base.logger, "Runtime: {} s", runtime);
                        runtime
                    }
                    Err(err) => {
                        error!(self.base.logger, "{}", err);
                        return Err(());
                    }
                };
                let energy = match self.dachs_client.get_total_energy().await {
                    Ok(energy) => {
                        trace!(self.base.logger, "Energy: {} kWh", energy);
                        energy
                    }
                    Err(err) => {
                        error!(self.base.logger, "{}", err);
                        return Err(());
                    }
                };
                Ok((runtime, energy))
            },
        )
        .await
        {
            Ok(result) => match result {
                Ok((runtime, energy)) => (runtime, energy),
                Err(_) => return TaskResult::Running,
            },
            Err(err) => {
                error!(self.base.logger, "Query Dachs data timed out: {}", err);
                return TaskResult::Running;
            }
        };

        let power = match self.base.query_last::<Generator>().await {
            InfluxResult::Some(last_record) => {
                // TODO: derive nonlinear power from delta timestamp and delta runtime
                if dachs_runtime != last_record.runtime as i32 {
                    800.0
                } else {
                    0.0
                }
            }
            InfluxResult::None => 0.0,
            InfluxResult::Err(e) => {
                error!(
                    self.base.logger,
                    "Query {} database failed: {}", &self.base.name, e
                );
                return TaskResult::Running;
            }
        };

        let record = Generator::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(timing.now)),
            dachs_energy.into(),
            power,
            dachs_runtime.into(),
        );

        self.base.notify_processors(&record);
        let _: Result<(), ()> = self.base.save_record(record).await;
        TaskResult::Running
    }
}
