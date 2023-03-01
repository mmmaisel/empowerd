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
use super::ProcessorBase;
use crate::models::{AvailablePower, Model};
use crate::pt1::PT1;
use crate::task_group::TaskResult;
use chrono::Utc;
use slog::{debug, error, warn};
use tokio::sync::{mpsc, oneshot, watch};

#[derive(Debug)]
pub enum Command {
    SetThreshold {
        threshold: f64,
        resp: oneshot::Sender<()>,
    },
    GetThreshold {
        resp: oneshot::Sender<f64>,
    },
    GetPower {
        resp: oneshot::Sender<f64>,
    },
}

pub struct AvailablePowerProcessor {
    base: ProcessorBase,
    command_input: mpsc::Receiver<Command>,
    battery_input: watch::Receiver<Model>,
    meter_input: watch::Receiver<Model>,
    power_output: watch::Sender<Model>,
    battery_threshold: f64,
    skipped_events: u8,
    filter: PT1,
}

impl AvailablePowerProcessor {
    pub fn new(
        base: ProcessorBase,
        command_input: mpsc::Receiver<Command>,
        battery_input: watch::Receiver<Model>,
        meter_input: watch::Receiver<Model>,
        power_output: watch::Sender<Model>,
        battery_threshold: f64,
        tau: f64,
    ) -> Self {
        Self {
            base,
            command_input,
            battery_input,
            meter_input,
            power_output,
            battery_threshold,
            skipped_events: 0,
            filter: PT1::new(tau, 0.0, 0.0, 16000.0, Utc::now()),
        }
    }

    pub async fn run(&mut self) -> TaskResult {
        tokio::select! {
            _ = self.base.canceled.changed() => {
                return TaskResult::Canceled(self.base.name.clone());
            }
            x = self.command_input.recv() => {
                if let Some(command) = x {
                    if let Err(e) = self.handle_command(command) {
                        return TaskResult::Err(e);
                    }
                }
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

        let available_power = if battery.charge < self.battery_threshold {
            AvailablePower::new(meter_time, 0.0)
        } else {
            AvailablePower::new(
                meter_time,
                self.filter.process(battery.power - meter_power, meter_time),
            )
        };
        debug!(
            self.base.logger,
            "Available power: {}", available_power.power
        );

        self.power_output.send_replace(available_power.into());

        TaskResult::Running
    }

    fn handle_command(&mut self, command: Command) -> Result<(), String> {
        match command {
            Command::SetThreshold { threshold, resp } => {
                self.battery_threshold = threshold.abs();
                if let Err(_) = resp.send(()) {
                    return Err("Sending SetThreshold response failed!".into());
                }
            }
            Command::GetThreshold { resp } => {
                if let Err(_) = resp.send(self.battery_threshold) {
                    return Err("Sending GetThreshold response failed!".into());
                }
            }
            Command::GetPower { resp } => {
                let output = &*self.power_output.borrow();
                let power = match output {
                    Model::AvailablePower(x) => x.power,
                    Model::None => 0.0,
                    _ => {
                        return Err(format!(
                            "power_output has invalid type: {:?}",
                            output
                        ))
                    }
                };
                if let Err(_) = resp.send(power) {
                    return Err("Sending GetPower response failed!".into());
                }
            }
        }

        Ok(())
    }
}
