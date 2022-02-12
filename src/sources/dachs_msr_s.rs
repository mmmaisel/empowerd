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
use super::{PollResult, PollState, Sources};
use crate::interval_sleep;
use crate::models::{Generator, InfluxObject, InfluxResult};
use chrono::{DateTime, Utc};
use dachs_client::DachsClient;
use slog::{error, trace, Logger};
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::watch;

pub struct DachsMsrSSource {
    canceled: watch::Receiver<PollState>,
    influx: influxdb::Client,
    name: String,
    interval: Duration,
    logger: Logger,
    dachs_client: DachsClient,
}

impl DachsMsrSSource {
    pub fn new(
        canceled: watch::Receiver<PollState>,
        influx: influxdb::Client,
        name: String,
        interval: Duration,
        dachs_addr: String,
        dachs_pw: String,
        logger: Logger,
    ) -> Result<Self, String> {
        return Ok(Self {
            canceled: canceled,
            influx: influx,
            name: name,
            interval: interval,
            logger: logger.clone(),
            dachs_client: DachsClient::new(dachs_addr, dachs_pw, Some(logger)),
        });
    }

    pub async fn poll(&mut self) -> PollResult {
        let now = interval_sleep!(self);

        let (dachs_runtime, dachs_energy) = match tokio::time::timeout(
            std::time::Duration::from_secs(15),
            async {
                let runtime = match self.dachs_client.get_runtime().await {
                    Ok(runtime) => {
                        trace!(self.logger, "Runtime: {} s", runtime);
                        runtime
                    }
                    Err(err) => {
                        error!(self.logger, "{}", err);
                        return Err(());
                    }
                };
                let energy = match self.dachs_client.get_total_energy().await {
                    Ok(energy) => {
                        trace!(self.logger, "Energy: {} kWh", energy);
                        energy
                    }
                    Err(err) => {
                        error!(self.logger, "{}", err);
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
                Err(_) => return PollResult::Running,
            },
            Err(err) => {
                error!(self.logger, "Query Dachs data timed out: {}", err);
                return PollResult::Running;
            }
        };

        let power = match Generator::into_single(
            self.influx
                .json_query(Generator::query_last(&self.name))
                .await,
        ) {
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
                    self.logger,
                    "Query {} database failed: {}", &self.name, e
                );
                return PollResult::Running;
            }
        };

        let record = Generator::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(now)),
            dachs_energy.into(),
            power,
            dachs_runtime.into(),
        );

        trace!(self.logger, "Writing {:?} to database", &record);
        if let Err(e) = self.influx.query(&record.save_query(&self.name)).await
        {
            error!(self.logger, "Save generator data failed, {}", e);
        }
        return PollResult::Running;
    }
}