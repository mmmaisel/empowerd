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
use crate::misc::parse_socketaddr_with_default;
use crate::models::{
    units::{second, watt, watt_hour, Abbreviation, Energy, Power, Time},
    InfluxResult, SimpleMeter,
};
use crate::task_group::TaskResult;
use slog::{error, trace};
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

    pub async fn run(&mut self) -> TaskResult {
        let (now, _oversample) = match self.base.sleep_aligned().await {
            Ok(x) => (Time::new::<second>(x.now as f64), x.oversample),
            Err(e) => return e,
        };

        let mut context = match self.client.open().await {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.base.logger,
                    "Could not open sunspec connection: {}", e
                );
                return TaskResult::Running;
            }
        };

        if self.client.models().is_empty() {
            if let Err(e) = self.client.introspect(&mut context).await {
                error!(
                    self.base.logger,
                    "Could not introspect sunspec device: {}", e
                );
                return TaskResult::Running;
            }
        }

        let energy = match self.client.get_total_yield(&mut context).await {
            Ok(x) => Energy::new::<watt_hour>(x),
            Err(e) => {
                error!(self.base.logger, "Could not read energy yield: {}", e);
                return TaskResult::Running;
            }
        };
        trace!(
            self.base.logger,
            "Total energy is {}",
            energy.into_format_args(watt_hour, Abbreviation),
        );

        let mut record = SimpleMeter::new(now, energy, Power::new::<watt>(0.0));
        record.power = match self.base.query_last::<SimpleMeter>().await {
            InfluxResult::Some(last_record) => {
                trace!(
                    self.base.logger,
                    "Read {:?} from database",
                    last_record
                );
                record.calc_power(&last_record)
            }
            InfluxResult::None => Power::new::<watt>(0.0),
            InfluxResult::Err(e) => {
                error!(
                    self.base.logger,
                    "Query {} database failed: {}", &self.base.name, e
                );
                return TaskResult::Running;
            }
        };

        self.base.notify_processors(&record);
        let _: Result<(), ()> = self.base.save_record(record).await;
        TaskResult::Running
    }
}
