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
            celsius, joule, percent, ratio, second, watt, watt_hour, Energy,
            Power, Ratio, Temperature, Time,
        },
        Heatpump,
    },
    task_group::TaskResult,
    Error,
};
use lambda_client::{LambdaClient, LambdaMode};
use slog::{trace, Logger};
use std::collections::VecDeque;

pub struct LambdaHeatPumpSource {
    base: SourceBase,
    client: LambdaClient,
    count: u64,
    energy_interval: Energy,
    heat_interval: Energy,
    cold_interval: Energy,
    defrost_interval: Energy,
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
            heat_interval: Energy::new::<joule>(0.0),
            cold_interval: Energy::new::<joule>(0.0),
            defrost_interval: Energy::new::<joule>(0.0),
            cops,
        })
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let mut conn = self.base.get_database().await?;

        let (mode, power) = match tokio::time::timeout(
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

                let mode = context.get_op_mode().await.map_err(|e| {
                    Error::Temporary(format!(
                        "Query Lambda Heat Pump data failed: {e}",
                    ))
                })?;
                let power = context.get_current_power().await.map_err(|e| {
                    Error::Temporary(format!(
                        "Query Lambda Heat Pump data failed: {e}",
                    ))
                })?;

                Ok((mode, Power::new::<watt>(power as f64)))
            },
        )
        .await
        {
            Ok(Ok(x)) => x,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                return Err(Error::Temporary(format!(
                    "Query Lambda Heat Pump data timed out: {e}",
                )))
            }
        };

        // Calculate energy from oversampled power.
        let delta_t = Time::new::<second>(
            (self.base.interval.as_secs() / self.base.oversample_factor) as f64,
        );
        self.energy_interval += power * delta_t;
        self.count += 1;

        let cop_interval =
            Ratio::new::<percent>(*self.cops.back().unwrap_or(&0) as f64);
        match mode {
            LambdaMode::Heating => {
                self.heat_interval += power * delta_t * cop_interval;
            }
            LambdaMode::Cooling => {
                self.cold_interval += power * delta_t * cop_interval;
            }
            LambdaMode::Defrosting => {
                self.defrost_interval += power * delta_t * cop_interval;
            }
        }

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
                        heat: Energy::new::<watt_hour>(0.0),
                        cold: Energy::new::<watt_hour>(0.0),
                        defrost: Energy::new::<watt_hour>(0.0),
                        cop: Ratio::new::<ratio>(0.0),
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
            let heat_total = self.heat_interval + last_record.heat;
            let cold_total = self.cold_interval + last_record.heat;
            let defrost_total = self.defrost_interval + last_record.heat;

            let cop = Ratio::new::<percent>(
                self.cops.iter().map(|x| *x as i32).sum::<i32>() as f64
                    / self.cops.len() as f64,
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
                heat: heat_total,
                cold: cold_total,
                defrost: defrost_total,
                cop,
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
            record.insert(&mut conn, self.base.series_id).await?;

            self.reset_sample();
        }

        Ok(())
    }

    fn reset_sample(&mut self) {
        self.energy_interval = Energy::new::<joule>(0.0);
        self.heat_interval = Energy::new::<joule>(0.0);
        self.cold_interval = Energy::new::<joule>(0.0);
        self.defrost_interval = Energy::new::<joule>(0.0);
        self.count = 0;
    }
}
