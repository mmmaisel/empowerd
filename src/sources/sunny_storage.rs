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
        Battery,
    },
    task_group::TaskResult,
    Error,
};
use slog::Logger;
use sunny_storage_client::{
    SunnyBoyStorageClient, SunnyIslandClient, SunnyStorageClient,
};

pub struct SunnyStorageSource {
    base: SourceBase,
    battery_client: Box<dyn SunnyStorageClient + Send + Sync>,
}

impl SunnyStorageSource {
    pub fn new(
        base: SourceBase,
        r#type: &'static str,
        address: String,
    ) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 502)?;
        let battery_client: Box<dyn SunnyStorageClient + Send + Sync> =
            match r#type {
                "sunny_island" => Box::new(SunnyIslandClient::new(
                    address,
                    Some(base.logger.clone()),
                )?),
                "sunny_boy_storage" => Box::new(SunnyBoyStorageClient::new(
                    address,
                    Some(base.logger.clone()),
                )?),
                _ => {
                    return Err(format!(
                        "ArgumentError: SunnyStorageClient type {} is invalid.",
                        r#type
                    ))
                }
            };

        Ok(Self {
            base,
            battery_client: battery_client,
        })
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

        let (energy_in, energy_out, charge) =
            match self.battery_client.get_in_out_charge().await {
                Ok((x, y, z)) => (
                    Energy::new::<watt_hour>(x as f64),
                    Energy::new::<watt_hour>(y as f64),
                    Energy::new::<watt_hour>(z),
                ),
                Err(e) => {
                    return Err(Error::Temporary(format!(
                        "Get battery data failed: {e}",
                    )))
                }
            };

        let mut record = Battery {
            time: Time::new::<second>(timing.now as f64),
            charge,
            energy_in,
            energy_out,
            power: Power::new::<watt>(0.0),
        };
        record.power = match Battery::last(&mut conn, self.base.series_id).await
        {
            Ok(last_record) => record.calc_power(&last_record),
            Err(Error::NotFound) => Power::new::<watt>(0.0),
            Err(e) => {
                return Err(Error::Temporary(format!(
                    "Query {} database failed: {}",
                    &self.base.name, e,
                )))
            }
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
