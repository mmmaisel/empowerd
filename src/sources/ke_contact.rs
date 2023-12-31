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
    misc::parse_socketaddr_with_default,
    models::{
        units::{second, watt, watt_hour, Energy, Power, Time},
        SimpleMeter,
    },
    task_group::TaskResult,
    Error,
};
use kecontact_client::KeContactClient;
use slog::{error, trace};

pub struct KeContactSource {
    base: SourceBase,
    client: KeContactClient,
}

impl KeContactSource {
    pub fn new(base: SourceBase, address: String) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 7090)?;
        let client = KeContactClient::new(address, Some(base.logger.clone()));

        Ok(Self { base, client })
    }

    pub async fn run(&mut self) -> TaskResult {
        let (now, _oversample) = match self.base.sleep_aligned().await {
            Ok(x) => (Time::new::<second>(x.now as f64), x.oversample),
            Err(e) => return e,
        };

        let mut conn = match self.base.database.get().await {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.base.logger,
                    "Getting database connection from pool failed: {}", e
                );
                return TaskResult::Running;
            }
        };

        let report = match tokio::time::timeout(
            std::time::Duration::from_secs(15),
            async {
                match self.client.power_report().await {
                    Ok(x) => Ok(x),
                    Err(e) => {
                        error!(
                            self.base.logger,
                            "Query KeContact data failed: {}", e
                        );
                        Err(())
                    }
                }
            },
        )
        .await
        {
            Ok(result) => match result {
                Ok(x) => x,
                Err(_) => return TaskResult::Running,
            },
            Err(e) => {
                error!(
                    self.base.logger,
                    "Query KeContact data timed out: {}", e
                );
                return TaskResult::Running;
            }
        };

        let energy = Energy::new::<watt_hour>((report.e_total as f64) / 10.0);

        let mut record = SimpleMeter {
            time: now,
            energy,
            power: Power::new::<watt>(0.0),
        };
        record.power =
            match SimpleMeter::last(&mut conn, self.base.series_id).await {
                Ok(last_record) => {
                    trace!(
                        self.base.logger,
                        "Read {:?} from database",
                        last_record
                    );
                    record.calc_power(&last_record)
                }
                Err(Error::NotFound) => Power::new::<watt>(0.0),
                Err(e) => {
                    error!(
                        self.base.logger,
                        "Query {} database failed: {}", &self.base.name, e
                    );
                    return TaskResult::Running;
                }
            };

        self.base.notify_processors(&record);
        if let Err(e) = record.insert(&mut conn, self.base.series_id).await {
            error!(
                self.base.logger,
                "Inserting {} record into database failed: {}",
                &self.base.name,
                e
            );
        }

        TaskResult::Running
    }
}
