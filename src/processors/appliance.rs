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
use crate::models::{
    units::{second, watt, Power},
    Model,
};
use crate::sinks::ArcSink;
use crate::task_group::TaskResult;
use crate::tri_state::TriState;
use slog::{debug, error, warn};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State {
    Off,
    On,
}

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
    retransmit_interval: Duration,
    skipped_events: u8,
    last_target_power: f64,
    last_appliance_power: f64,
    state: State,
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
        retransmit_interval: Duration,
    ) -> Self {
        Self {
            base,
            command_input,
            power_input,
            appliance_output,
            power_output,
            appliance_input,
            retransmit_interval,
            skipped_events: 0,
            last_target_power: 0.0,
            last_appliance_power: 0.0,
            state: State::Off,
            force_on_off: TriState::Auto,
        }
    }

    pub fn validate_appliance(appliance: &ArcSink) -> bool {
        matches!(
            appliance,
            ArcSink::KeContact(_) | ArcSink::LambdaHeatPump(_)
        )
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
            _ = time::sleep(self.retransmit_interval) => {
                if let Err(e) = Self::set_output(
                    &self.appliance_output,
                    self.last_target_power,
                    self.last_appliance_power
                ).await {
                    error!(self.base.logger, "{}", e);
                };
                return TaskResult::Running;
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
            Model::Heatpump(ref x) => x.into(),
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

        if (appliance.time.timestamp()
            - available_power.time.get::<second>() as i64)
            .abs()
            > 15
        {
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

        let (new_state, output_power, target_power) = Self::calc_power(
            self.force_on_off,
            self.state,
            available_power.power.get::<watt>(),
            appliance.power,
        );

        if let Err(e) = Self::set_output(
            &self.appliance_output,
            target_power,
            appliance.power,
        )
        .await
        {
            error!(self.base.logger, "{}", e);
            return TaskResult::Running;
        };

        debug!(
            self.base.logger,
            "Appliance '{}' is {:?}", self.base.name, new_state
        );
        debug!(
            self.base.logger,
            "Appliance '{}' power: {}", self.base.name, target_power
        );
        debug!(
            self.base.logger,
            "Available power after {}: {}", self.base.name, output_power,
        );

        self.last_appliance_power = available_power.power.get::<watt>();
        self.last_target_power = target_power;
        self.state = new_state;

        available_power.power = Power::new::<watt>(output_power);
        self.power_output.send_replace(available_power.into());

        TaskResult::Running
    }

    fn calc_power(
        force_on_off: TriState,
        state: State,
        input_power: f64,
        appliance_power: f64,
    ) -> (State, f64, f64) {
        match force_on_off {
            TriState::Auto => match state {
                State::Off => {
                    if input_power > 0.0 {
                        // Switch on the appliance. Divert all input power to
                        // newly enabled appliance. We dont know the actual
                        // power consumption of the appliance yet so set the
                        // excess power to zero.
                        (State::On, 0.0, input_power)
                    } else {
                        // Not enough power available to enable appliance.
                        (State::Off, input_power, 0.0)
                    }
                }
                State::On => {
                    if -input_power > appliance_power {
                        // Available power is smaller than current appliance
                        // power consumption so disable the appliance.
                        (State::Off, input_power, 0.0)
                    } else {
                        // Adjust appliance power to match available input
                        // power. Add measured appliance power to output
                        // because it is implicitely subtracted from the
                        // measured sum power.
                        // Set excess power to input since we do not
                        // know the new actual power consumption of the appliance.
                        (State::On, input_power, input_power + appliance_power)
                    }
                }
            },
            // Appliance is forced off so all input power is excess power.
            TriState::Off => (State::Off, input_power, 0.0),
            TriState::On => {
                // Maximum power of one phase
                let max_power = 230.0 * 16.0;
                // Appliance is forced on.
                // All input power is excess power. Actual appliance power
                // consumption is implicitely subtractur from measured input
                // in next iteration.
                (State::On, input_power, max_power)
            }
        }
    }

    async fn set_output(
        output: &ArcSink,
        target_power: f64,
        current_power: f64,
    ) -> Result<bool, String> {
        match output {
            ArcSink::KeContact(wallbox) => {
                wallbox
                    .set_available_power(target_power, current_power)
                    .await
            }
            ArcSink::LambdaHeatPump(lambda) => {
                lambda.set_available_power(target_power).await
            }
            _ => Err("Unsupported appliance type".into()),
        }
    }

    fn handle_command(&mut self, command: Command) -> Result<(), String> {
        match command {
            Command::SetForceOnOff { force_on_off, resp } => {
                self.force_on_off = force_on_off;
                if resp.send(()).is_err() {
                    return Err("Sending SetForceOnOff response failed!".into());
                }
            }
            Command::GetForceOnOff { resp } => {
                if resp.send(self.force_on_off).is_err() {
                    return Err("Sending GetForceOnOff response failed!".into());
                }
            }
        }

        Ok(())
    }
}

#[test]
fn test_state_transitions() {
    assert_eq!(
        ApplianceProcessor::calc_power(TriState::Auto, State::Off, 100.0, 0.0),
        (State::On, 0.0, 100.0),
        "Appliance did not switch on",
    );
    assert_eq!(
        ApplianceProcessor::calc_power(TriState::Auto, State::Off, 0.0, 100.0),
        (State::Off, 0.0, 0.0),
        "Appliance switched on when it should not",
    );
    assert_eq!(
        ApplianceProcessor::calc_power(TriState::Auto, State::Off, -100.0, 0.0),
        (State::Off, -100.0, 0.0),
        "Negative excess power is not passed on",
    );
    assert_eq!(
        ApplianceProcessor::calc_power(
            TriState::Auto,
            State::On,
            -101.0,
            100.0
        ),
        (State::Off, -101.0, 0.0),
        "Appliance did not switch off",
    );
    assert_eq!(
        ApplianceProcessor::calc_power(TriState::Auto, State::On, 100.0, 100.0),
        (State::On, 100.0, 200.0),
        "Appliance power did not increase",
    );
    assert_eq!(
        ApplianceProcessor::calc_power(
            TriState::Auto,
            State::On,
            -100.0,
            200.0
        ),
        (State::On, -100.0, 100.0),
        "Appliance power did not decrease",
    );
    assert_eq!(
        ApplianceProcessor::calc_power(TriState::Auto, State::On, 0.0, 200.0),
        (State::On, 0.0, 200.0),
        "Appliance power was not kept",
    );
    assert_eq!(
        ApplianceProcessor::calc_power(TriState::Auto, State::On, 100.0, 200.0),
        (State::On, 100.0, 300.0),
        "Excess power is incorrect",
    );
}

#[test]
fn test_force_on() {
    assert_eq!(
        ApplianceProcessor::calc_power(TriState::On, State::Off, 100.0, 200.0),
        (State::On, 100.0, 3680.0),
        "Excess power is incorrect if there is still power available",
    );
    assert_eq!(
        ApplianceProcessor::calc_power(TriState::On, State::Off, -100.0, 200.0),
        (State::On, -100.0, 3680.0),
        "Excess power is incorrect if there is no power available",
    );
}

#[test]
fn test_force_off() {
    assert_eq!(
        ApplianceProcessor::calc_power(TriState::Off, State::Off, 100.0, 200.0),
        (State::Off, 100.0, 0.0),
        "Excess power is incorrect if there is still power available",
    );
    assert_eq!(
        ApplianceProcessor::calc_power(
            TriState::Off,
            State::Off,
            -100.0,
            200.0
        ),
        (State::Off, -100.0, 0.0),
        "Excess power is incorrect if there is no power available",
    );
}
