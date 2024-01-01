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
use slog::{trace, Logger};

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

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let mut conn = self.base.get_database().await?;

        let report =
            tokio::time::timeout(std::time::Duration::from_secs(15), async {
                self.client.power_report().await.map_err(|e| {
                    Error::Temporary(format!(
                        "Query KeContact data failed: {e}",
                    ))
                })
            })
            .await
            .map_err(|e| {
                Error::Temporary(
                    format!("Query KeContact data timed out: {e}",),
                )
            })??;

        let energy = Energy::new::<watt_hour>((report.e_total as f64) / 10.0);

        let mut record = SimpleMeter {
            time: Time::new::<second>(timing.now as f64),
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
                    return Err(Error::Temporary(format!(
                        "Query {} database failed: {}",
                        &self.base.name, e,
                    )))
                }
            };

        self.base.notify_processors(&record);
        record.insert(&mut conn, self.base.series_id).await?;

        Ok(())
    }
}
