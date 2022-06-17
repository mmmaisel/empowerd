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
use crate::models::{AvailablePower, Model};
use crate::task_group::TaskResult;
use slog::{debug, error, warn};
use tokio::sync::watch;

pub struct AvailablePowerProcessor {
    base: ProcessorBase,
    battery_input: watch::Receiver<Model>,
    meter_input: watch::Receiver<Model>,
    power_output: watch::Sender<Model>,
    skipped_events: u8,
}

impl AvailablePowerProcessor {
    pub fn new(
        base: ProcessorBase,
        battery_input: watch::Receiver<Model>,
        meter_input: watch::Receiver<Model>,
        power_output: watch::Sender<Model>,
    ) -> Self {
        Self {
            base,
            battery_input,
            meter_input,
            power_output,
            skipped_events: 0,
        }
    }

    pub async fn run(&mut self) -> TaskResult {
        tokio::select! {
            _ = self.base.canceled.changed() => {
                return TaskResult::Canceled(self.base.name.clone());
            }
            x = self.meter_input.changed() => {
                if let Err(e) = x {
                    return TaskResult::Err(
                        format!("Reading meter input failed: {}", e)
                    );
                }
            }
            x = self.battery_input.changed() => {
                if let Err(e) = x {
                    return TaskResult::Err(
                        format!("Reading battery input failed: {}", e)
                    );
                }
            }
        };

        let (meter_time, meter_power) = match *self.meter_input.borrow() {
            Model::BidirectionalMeter(ref x) => (x.time, x.power),
            Model::SimpleMeter(ref x) => (x.time, x.power),
            Model::None => return TaskResult::Running,
            _ => {
                error!(
                    self.base.logger,
                    "Received invalid model from meter input: {:?}",
                    *self.meter_input.borrow()
                );
                return TaskResult::Running;
            }
        };

        let battery = match *self.battery_input.borrow() {
            Model::Battery(ref x) => x.clone(),
            Model::None => return TaskResult::Running,
            _ => {
                error!(
                    self.base.logger,
                    "Received invalid model from battery input: {:?}",
                    *self.battery_input.borrow()
                );
                return TaskResult::Running;
            }
        };

        if (meter_time - battery.time).num_seconds().abs() > 15 {
            self.skipped_events += 1;
            if self.skipped_events >= 2 {
                warn!(
                    self.base.logger,
                    "Skipping available power processor due to missing events"
                );
            }
            return TaskResult::Running;
        }
        self.skipped_events = 0;

        let available_power =
            AvailablePower::new(meter_time, battery.power - meter_power);
        debug!(
            self.base.logger,
            "Available power: {}", available_power.power
        );

        self.power_output.send_replace(available_power.into());

        TaskResult::Running
    }
}
