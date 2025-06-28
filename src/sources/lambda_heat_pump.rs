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
    task_group::{TaskResult, TaskTiming},
    Error,
};
use diesel_async::{pooled_connection::deadpool::Object, AsyncPgConnection};
use lambda_client::{LambdaClient, LambdaContext, LambdaMode};
use slog::{debug, trace, Logger};
use std::{collections::VecDeque, time::Duration};
use tokio::time::{error::Elapsed, timeout};

pub struct LambdaHeatPumpSource {
    base: SourceBase,
    client: LambdaClient,
    count: u64,
    energy_interval: Energy,
    heat_interval: Energy,
    cold_interval: Energy,
    defrost_interval: Energy,
    cops: VecDeque<i16>,
    acc_support: Option<bool>,
    last_energy_acc: Energy,
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
            acc_support: None,
            last_energy_acc: Energy::new::<watt_hour>(0.0),
        })
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let conn = self.base.get_database().await?;

        let mut context =
            timeout(Duration::from_secs(1), self.connect_and_introspect())
                .await
                .map_err(Self::timeout_err)??;

        let mode = timeout(
            Duration::from_secs(1),
            Self::get_cop_and_mode(&mut context, &mut self.cops),
        )
        .await
        .map_err(Self::timeout_err)??;

        let power = timeout(Duration::from_secs(1), async {
            match self.acc_support {
                Some(true) => self.get_power_acc(&mut context).await,
                Some(false) => self.get_power_direct(&mut context).await,
                None => unreachable!(),
            }
        })
        .await
        .map_err(Self::timeout_err)??;

        self.update_energies(power, mode);

        if !timing.oversample {
            // Discard incomplete samples.
            if self.count != self.base.oversample_factor {
                debug!(
                    self.base.logger,
                    "Lambda: Discarding incomplete sample"
                );
                self.reset_sample();
                return Ok(());
            }
            self.fetch_boiler_and_save(context, timing, conn).await?;
            self.reset_sample();
        }

        Ok(())
    }

    fn timeout_err(_e: Elapsed) -> Error {
        Error::Temporary("Query Lambda Heat Pump data timed out".into())
    }

    fn query_err(e: String) -> Error {
        Error::Temporary(format!("Query Lambda Heat Pump data failed: {e}",))
    }

    async fn connect_and_introspect(&mut self) -> Result<LambdaContext, Error> {
        let mut context = self.client.open().await.map_err(|e| {
            Error::Temporary(format!(
                "Could not connect to Lambda Heat Pump: {e}"
            ))
        })?;

        if self.acc_support.is_none() {
            let acc_support = Self::check_acc_support(&mut context).await;
            debug!(self.base.logger, "Using energy accumulator: {acc_support}");
            self.acc_support = Some(acc_support);
        }

        Ok(context)
    }

    async fn check_acc_support(context: &mut LambdaContext) -> bool {
        context.get_total_energy().await.is_ok()
    }

    async fn get_cop_and_mode(
        context: &mut LambdaContext,
        cops: &mut VecDeque<i16>,
    ) -> Result<LambdaMode, Error> {
        let cop = context.get_cop().await.map_err(Self::query_err)?;
        cops.pop_front();
        cops.push_back(cop);

        let mode = context.get_op_mode().await.map_err(Self::query_err)?;

        Ok(mode)
    }

    async fn get_power_direct(
        &mut self,
        context: &mut LambdaContext,
    ) -> Result<Power, Error> {
        let power =
            context.get_current_power().await.map_err(Self::query_err)?;

        Ok(Power::new::<watt>(power as f64))
    }

    async fn get_power_acc(
        &mut self,
        context: &mut LambdaContext,
    ) -> Result<Power, Error> {
        let energy = Energy::new::<watt_hour>(
            context.get_total_energy().await.map_err(Self::query_err)? as f64,
        );

        let power = if self.last_energy_acc < Energy::new::<watt_hour>(1.0) {
            Power::new::<watt>(0.0)
        } else {
            let delta_t = Time::new::<second>(
                (self.base.interval.as_secs() / self.base.oversample_factor)
                    as f64,
            );

            (energy - self.last_energy_acc) / delta_t
        };

        self.last_energy_acc = energy;
        Ok(power)
    }

    fn update_energies(&mut self, power: Power, mode: LambdaMode) {
        // Calculate energy from oversampled power.
        // TODO: split off state struct for calcs only that can be tested
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
    }

    async fn get_boiler_temps(
        context: &mut LambdaContext,
    ) -> Result<(i16, i16, i16), Error> {
        context.get_boiler_temps().await.map_err(Self::query_err)
    }

    async fn fetch_boiler_and_save(
        &mut self,
        mut context: LambdaContext,
        timing: TaskTiming,
        mut conn: Object<AsyncPgConnection>,
    ) -> TaskResult {
        // Get accumulated energy from database.
        let last_record = match Heatpump::last(&mut conn, self.base.series_id)
            .await
        {
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
                    - Time::new::<second>(self.base.interval.as_secs() as f64),
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
        let cold_total = self.cold_interval + last_record.cold;
        let defrost_total = self.defrost_interval + last_record.defrost;

        let cop = Ratio::new::<percent>(
            self.cops.iter().map(|x| *x as i32).sum::<i32>() as f64
                / self.cops.len() as f64,
        );

        let boiler = timeout(
            Duration::from_secs(1),
            Self::get_boiler_temps(&mut context),
        )
        .await
        .map_err(Self::timeout_err)??;

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
