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
use crate::models::Model;
use crate::sinks::ArcSink;
use crate::task_group::TaskResult;
use crate::tri_state::TriState;
use slog::{debug, error, warn};
use tokio::sync::{mpsc, oneshot, watch};

#[derive(Debug)]
pub enum Command {
    SetForceOnOff {
        force_on_off: TriState,
        resp: oneshot::Sender<()>,
    },
    GetForceOnOff {
        resp: oneshot::Sender<TriState>,
    },
}

pub struct ApplianceProcessor {
    base: ProcessorBase,
    command_input: mpsc::Receiver<Command>,
    power_input: watch::Receiver<Model>,
    appliance_input: watch::Receiver<Model>,
    power_output: watch::Sender<Model>,
    appliance_output: ArcSink,
    skipped_events: u8,
    was_enabled: bool,
    force_on_off: TriState,
}

impl ApplianceProcessor {
    pub fn new(
        base: ProcessorBase,
        command_input: mpsc::Receiver<Command>,
        power_input: watch::Receiver<Model>,
        appliance_input: watch::Receiver<Model>,
        power_output: watch::Sender<Model>,
        appliance_output: ArcSink,
    ) -> Self {
        Self {
            base,
            command_input,
            power_input,
            appliance_output,
            power_output,
            appliance_input,
            skipped_events: 0,
            was_enabled: false,
            force_on_off: TriState::Auto,
        }
    }

    pub fn validate_appliance(appliance: &ArcSink) -> bool {
        match appliance {
            ArcSink::KeContact(_) => true,
            ArcSink::LambdaHeatPump(_) => true,
            _ => false,
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

        let target_power = match self.force_on_off {
            TriState::Auto => available_power.power + appliance.power,
            TriState::Off => 0.0,
            TriState::On => 230.0 * 16.0, // Maximum power of one phase
        };

        let result = match &self.appliance_output {
            ArcSink::KeContact(wallbox) => {
                wallbox
                    .set_available_power(target_power, appliance.power)
                    .await
            }
            ArcSink::LambdaHeatPump(lambda) => {
                lambda.set_available_power(target_power).await
            }
            _ => Err("Unsupported appliance type".into()),
        };
        let enabled = match result {
            Ok(x) => x,
            Err(e) => {
                error!(self.base.logger, "{}", e);
                return TaskResult::Running;
            }
        };

        if enabled && !self.was_enabled {
            available_power.power =
                (available_power.power - target_power).max(0.0);
        } else {
            available_power.power =
                (available_power.power - appliance.power).max(0.0);
        }

        debug!(
            self.base.logger,
            "Available power after {}: {}",
            self.base.name,
            available_power.power
        );
        self.power_output.send_replace(available_power.into());

        TaskResult::Running
    }

    fn handle_command(&mut self, command: Command) -> Result<(), String> {
        match command {
            Command::SetForceOnOff { force_on_off, resp } => {
                self.force_on_off = force_on_off;
                if let Err(_) = resp.send(()) {
                    return Err("Sending SetForceOnOff response failed!".into());
                }
            }
            Command::GetForceOnOff { resp } => {
                if let Err(_) = resp.send(self.force_on_off) {
                    return Err("Sending GetForceOnOff response failed!".into());
                }
            }
        }

        Ok(())
    }
}
