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
    units::{second, watt, watt_hour, Energy, Power, Time},
    Battery, InfluxResult,
};
use crate::task_group::TaskResult;
use slog::error;
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

    pub async fn run(&mut self) -> TaskResult {
        let (timing, _oversample) = match self.base.sleep_aligned().await {
            Ok(x) => (Time::new::<second>(x.now as f64), x.oversample),
            Err(e) => return e,
        };

        let (wh_in, wh_out, charge) =
            match self.battery_client.get_in_out_charge().await {
                Ok((x, y, z)) => (
                    Energy::new::<watt_hour>(x as f64),
                    Energy::new::<watt_hour>(y as f64),
                    Energy::new::<watt_hour>(z),
                ),
                Err(e) => {
                    error!(self.base.logger, "Get battery data failed: {}", e);
                    return TaskResult::Running;
                }
            };

        let power = match self.base.query_last::<Battery>().await {
            InfluxResult::Some(last_record) => {
                (wh_in
                    - last_record.energy_in
                    - (wh_out - last_record.energy_out))
                    / (timing - last_record.time)
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

        let record = Battery::new(timing, charge, wh_in, wh_out, power);
        self.base.notify_processors(&record);
        let _: Result<(), ()> = self.base.save_record(record).await;
        TaskResult::Running
    }
}

#[test]
fn test_power_calculation() {
    let old = Battery::new(
        Time::new::<second>(1680966000.0),
        Energy::new::<watt_hour>(5000.0),
        Energy::new::<watt_hour>(430.0),
        Energy::new::<watt_hour>(497.0),
        Power::new::<watt>(0.0),
    );
    let new = Battery::new(
        Time::new::<second>(1680966300.0),
        Energy::new::<watt_hour>(5000.0),
        Energy::new::<watt_hour>(476.0),
        Energy::new::<watt_hour>(497.0),
        Power::new::<watt>(0.0),
    );

    let power =
        (new.energy_in - old.energy_in - (new.energy_out - old.energy_out))
            / (new.time - old.time);
    assert_eq!(552.0, power.get::<watt>());
}
