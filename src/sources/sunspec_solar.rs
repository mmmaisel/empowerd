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
        units::{second, watt, watt_hour, Abbreviation, Energy, Power, Time},
        SimpleMeter,
    },
    task_group::TaskResult,
    Error,
};
use slog::{trace, Logger};
use sunspec_client::SunspecClient;

pub struct SunspecSolarSource {
    base: SourceBase,
    client: SunspecClient,
}

impl SunspecSolarSource {
    pub fn new(
        base: SourceBase,
        address: String,
        id: Option<u8>,
    ) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 502)?;
        let client = SunspecClient::new(address, id, Some(base.logger.clone()));

        Ok(Self { base, client })
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let mut conn = self.base.get_database().await?;

        let mut context = self.client.open().await.map_err(|e| {
            Error::Temporary(format!("Could not open sunspec connection: {e}",))
        })?;

        if self.client.models().is_empty() {
            self.client.introspect(&mut context).await.map_err(|e| {
                Error::Temporary(format!(
                    "Could not introspect sunspec device: {e}",
                ))
            })?;
        }

        let energy = match self.client.get_total_yield(&mut context).await {
            Ok(x) => Energy::new::<watt_hour>(x),
            Err(e) => {
                return Err(Error::Temporary(format!(
                    "Could not read energy yield: {e}",
                )))
            }
        };
        trace!(
            self.base.logger,
            "Total energy is {}",
            energy.into_format_args(watt_hour, Abbreviation),
        );

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
