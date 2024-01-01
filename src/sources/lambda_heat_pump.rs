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
use crate::{
    misc::parse_socketaddr_with_default,
    models::{
        units::{
            celsius, joule, ratio, second, watt, watt_hour, Energy, Power,
            Ratio, Temperature, Time,
        },
        Heatpump,
    },
    task_group::TaskResult,
    Error,
};
use lambda_client::LambdaClient;
use slog::{trace, Logger};
use std::collections::VecDeque;

pub struct LambdaHeatPumpSource {
    base: SourceBase,
    client: LambdaClient,
    count: u64,
    energy_interval: Energy,
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
            energy_interval: Energy::new::<joule>(0.0),
            cops,
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

        let power = match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            async {
                let mut context = self.client.open().await.map_err(|e| {
                    Error::Temporary(format!(
                        "Could not connect to Lambda Heat Pump: {e}",
                    ))
                })?;

                match context.get_cop().await {
                    Ok(x) => {
                        self.cops.pop_front();
                        self.cops.push_back(x);
                    }
                    Err(e) => {
                        return Err(Error::Temporary(format!(
                            "Query Lambda Heat Pump data failed: {e}",
                        )))
                    }
                }

                context.get_current_power().await.map_err(|e| {
                    Error::Temporary(format!(
                        "Query Lambda Heat Pump data failed: {e}",
                    ))
                })
            },
        )
        .await
        {
            Ok(Ok(x)) => Power::new::<watt>(x as f64),
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                return Err(Error::Temporary(format!(
                    "Query Lambda Heat Pump data timed out: {e}",
                )))
            }
        };

        // Calculate energy from oversampled power.
        self.energy_interval += power
            * (Time::new::<second>(
                (self.base.interval.as_secs() / self.base.oversample_factor)
                    as f64,
            ));
        self.count += 1;

        if !timing.oversample {
            // Discard incomplete samples.
            if self.count != self.base.oversample_factor {
                self.reset_sample();
                return Ok(());
            }

            // Get accumulated energy from database.
            let last_record =
                match Heatpump::last(&mut conn, self.base.series_id).await {
                    Ok(last_record) => {
                        trace!(
                            self.base.logger,
                            "Read {:?} from database",
                            last_record
                        );
                        last_record
                    }
                    Err(Error::NotFound) => Heatpump {
                        time: Time::new::<second>(timing.now as f64)
                            - Time::new::<second>(
                                self.base.interval.as_secs() as f64
                            ),
                        energy: Energy::new::<watt_hour>(0.0),
                        power: Power::new::<watt>(0.0),
                        heat: None,
                        cop: None,
                        boiler_top: None,
                        boiler_mid: None,
                        boiler_bot: None,
                    },
                    Err(e) => {
                        return Err(Error::Temporary(format!(
                            "Query {} database failed: {}",
                            &self.base.name, e,
                        )))
                    }
                };

            // Update accumulated energy.
            let energy_total = self.energy_interval + last_record.energy;
            let cop = Ratio::new::<ratio>(
                self.cops.iter().map(|x| *x as i32).sum::<i32>() as f64
                    / self.cops.len() as f64
                    / 100.0,
            );

            let mut context = self.client.open().await.map_err(|e| {
                Error::Temporary(format!(
                    "Could not connect to Lambda Heat Pump: {e}",
                ))
            })?;
            let boiler = context.get_boiler_temps().await.map_err(|e| {
                Error::Temporary(format!(
                    "Query Lambda Heat Pump data failed: {e}",
                ))
            })?;

            // Commit new sample to database.
            let mut record = Heatpump {
                time: Time::new::<second>(timing.now as f64),
                energy: energy_total,
                power: Power::new::<watt>(0.0),
                heat: Some(
                    last_record.heat.unwrap_or(Energy::new::<joule>(0.0))
                        + (energy_total - last_record.energy) * cop,
                ),
                cop: Some(cop),
                boiler_top: Some(Temperature::new::<celsius>(
                    boiler.0 as f64 / 10.0,
                )),
                boiler_mid: Some(Temperature::new::<celsius>(
                    boiler.1 as f64 / 10.0,
                )),
                boiler_bot: Some(Temperature::new::<celsius>(
                    boiler.2 as f64 / 10.0,
                )),
            };
            record.power = record.calc_power(&last_record);

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

            self.reset_sample();
        }

        Ok(())
    }

    fn reset_sample(&mut self) {
        self.energy_interval = Energy::new::<joule>(0.0);
        self.count = 0;
    }
}
