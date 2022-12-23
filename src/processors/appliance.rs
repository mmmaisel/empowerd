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
use super::ProcessorBase;
use crate::models::Model;
use crate::pt1::PT1;
use crate::sinks::ArcSink;
use crate::task_group::TaskResult;
use chrono::Utc;
use slog::{debug, error, warn};
use tokio::sync::watch;

pub struct ApplianceProcessor {
    base: ProcessorBase,
    power_input: watch::Receiver<Model>,
    appliance_input: watch::Receiver<Model>,
    power_output: watch::Sender<Model>,
    appliance_output: ArcSink,
    skipped_events: u8,
    filter: PT1,
}

impl ApplianceProcessor {
    pub fn new(
        base: ProcessorBase,
        power_input: watch::Receiver<Model>,
        appliance_input: watch::Receiver<Model>,
        power_output: watch::Sender<Model>,
        appliance_output: ArcSink,
        tau: f64,
    ) -> Self {
        Self {
            base,
            power_input,
            appliance_output,
            power_output,
            appliance_input,
            skipped_events: 0,
            filter: PT1::new(tau, 0.0, 0.0, 16000.0, Utc::now()),
        }
    }

    pub fn validate_appliance(appliance: &ArcSink) -> bool {
        match appliance {
            ArcSink::KeContact(_) => true,
            _ => false,
        }
    }

    pub async fn run(&mut self) -> TaskResult {
        tokio::select! {
            _ = self.base.canceled.changed() => {
                return TaskResult::Canceled(self.base.name.clone());
            }
            x = self.power_input.changed() => {
                if let Err(e) = x {
                    return TaskResult::Err(
                        format!("Reading available power failed: {}", e)
                    );
                }
            }
            x = self.appliance_input.changed() => {
                if let Err(e) = x {
                    return TaskResult::Err(
                        format!("Reading current appliance power failed: {}", e)
                    );
                }
            }
        };

        let mut available_power = match *self.power_input.borrow() {
            Model::AvailablePower(ref x) => x.clone(),
            Model::None => return TaskResult::Running,
            _ => {
                error!(
                    self.base.logger,
                    "Received invalid model from power input: {:?}",
                    *self.power_input.borrow()
                );
                return TaskResult::Running;
            }
        };

        let appliance = match *self.appliance_input.borrow() {
            Model::SimpleMeter(ref x) => x.clone(),
            Model::None => return TaskResult::Running,
            _ => {
                error!(
                    self.base.logger,
                    "Received invalid model from appliance input: {:?}",
                    *self.appliance_input.borrow()
                );
                return TaskResult::Running;
            }
        };

        if (appliance.time - available_power.time).num_seconds().abs() > 15 {
            self.skipped_events += 1;
            if self.skipped_events >= 2 {
                warn!(
                    self.base.logger,
                    "Skipping appliance processor due to missing events"
                );
            }
            return TaskResult::Running;
        }
        self.skipped_events = 0;

        let appliance_power = self
            .filter
            .process(available_power.power + appliance.power, appliance.time);

        let result = match &self.appliance_output {
            ArcSink::KeContact(wallbox) => {
                wallbox
                    .set_available_power(appliance_power, appliance.power)
                    .await
            }
            _ => Err("Unsupported appliance type".into()),
        };
        if let Err(e) = result {
            error!(self.base.logger, "{}", e);
            return TaskResult::Running;
        }

        available_power.power -= appliance.power;
        debug!(
            self.base.logger,
            "Available power after {}: {}",
            self.base.name,
            available_power.power
        );
        self.power_output.send_replace(available_power.into());

        TaskResult::Running
    }
}
