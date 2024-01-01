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
use crate::{
    models::{
        units::{kilowatt_hour, second, watt, Energy, Power, Time},
        Generator,
    },
    task_group::TaskResult,
    Error,
};
use dachs_client::DachsClient;
use slog::{trace, Logger};

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

        let (dachs_runtime, dachs_energy) = match tokio::time::timeout(
            std::time::Duration::from_secs(15),
            async {
                let runtime = match self.dachs_client.get_runtime().await {
                    Ok(runtime) => {
                        trace!(self.base.logger, "Runtime: {} s", runtime);
                        runtime
                    }
                    Err(e) => return Err(Error::Temporary(e)),
                };
                let energy = match self.dachs_client.get_total_energy().await {
                    Ok(energy) => {
                        trace!(self.base.logger, "Energy: {} kWh", energy);
                        energy
                    }
                    Err(e) => return Err(Error::Temporary(e)),
                };
                Ok((runtime, energy))
            },
        )
        .await
        {
            Ok(Ok((runtime, energy))) => (
                Time::new::<second>(runtime as f64),
                Energy::new::<kilowatt_hour>(energy as f64),
            ),
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                return Err(Error::Temporary(format!(
                    "Query Dachs data timed out: {e}",
                )))
            }
        };

        let power = match Generator::last(&mut conn, self.base.series_id).await
        {
            Ok(last_record) => {
                // TODO: derive nonlinear power from delta timestamp and delta runtime
                if (dachs_runtime.get::<second>() as i64)
                    != (last_record.runtime.get::<second>() as i64)
                {
                    Power::new::<watt>(800.0)
                } else {
                    Power::new::<watt>(0.0)
                }
            }
            Err(Error::NotFound) => Power::new::<watt>(0.0),
            Err(e) => {
                return Err(Error::Temporary(format!(
                    "Query {} database failed: {}",
                    &self.base.name, e,
                )))
            }
        };

        let record = Generator {
            time: Time::new::<second>(timing.now as f64),
            energy: dachs_energy,
            power,
            runtime: dachs_runtime,
        };

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
