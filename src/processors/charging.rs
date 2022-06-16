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
use crate::sinks::ke_contact::KeContactSink;
use crate::task_group::TaskResult;
use chrono::Utc;
use slog::{debug, error, warn};
use std::sync::Arc;
use tokio::sync::watch;

pub struct ChargingProcessor {
    base: ProcessorBase,
    power_input: watch::Receiver<Model>,
    wallbox_input: watch::Receiver<Model>,
    power_output: watch::Sender<Model>,
    wallbox_output: Arc<KeContactSink>,
    skipped_events: u8,
    filter: PT1,
}

impl ChargingProcessor {
    pub fn new(
        base: ProcessorBase,
        power_input: watch::Receiver<Model>,
        wallbox_input: watch::Receiver<Model>,
        power_output: watch::Sender<Model>,
        wallbox_output: Arc<KeContactSink>,
        tau: f64,
    ) -> Self {
        Self {
            base,
            power_input,
            wallbox_output,
            power_output,
            wallbox_input,
            skipped_events: 0,
            filter: PT1::new(tau, 0.0, 0.0, 16000.0, Utc::now()),
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
                        format!("Reading power input failed: {}", e)
                    );
                }
            }
            x = self.wallbox_input.changed() => {
                if let Err(e) = x {
                    return TaskResult::Err(
                        format!("Reading wallbox input failed: {}", e)
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

        let wallbox = match *self.wallbox_input.borrow() {
            Model::SimpleMeter(ref x) => x.clone(),
            Model::None => return TaskResult::Running,
            _ => {
                error!(
                    self.base.logger,
                    "Received invalid model from wallbox input: {:?}",
                    *self.wallbox_input.borrow()
                );
                return TaskResult::Running;
            }
        };

        if (wallbox.time - available_power.time).num_seconds().abs() > 15 {
            self.skipped_events += 1;
            if self.skipped_events >= 2 {
                warn!(
                    self.base.logger,
                    "Skipping charging processor due to missing events"
                );
            }
            return TaskResult::Running;
        }
        self.skipped_events = 0;

        let charging_power = self
            .filter
            .process(available_power.power + wallbox.power, wallbox.time);

        if charging_power < 6.0 * 230.0 && wallbox.power < 10.0
            || charging_power < 7.0 * 230.0 && wallbox.power >= 10.0
        {
            debug!(self.base.logger, "Disable charging");
            if let Err(e) = self.wallbox_output.set_enable(false).await {
                error!(self.base.logger, "{}", e);
                return TaskResult::Running;
            }
        } else {
            let charging_current = (charging_power / 230.0 * 1000.0) as u16;
            debug!(self.base.logger, "Set current to {} mA", charging_current);
            if let Err(e) =
                self.wallbox_output.set_max_current(charging_current).await
            {
                error!(self.base.logger, "{}", e);
                return TaskResult::Running;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if let Err(e) = self.wallbox_output.set_enable(true).await {
                error!(self.base.logger, "{}", e);
                return TaskResult::Running;
            }
        }

        available_power.power -= wallbox.power;
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
