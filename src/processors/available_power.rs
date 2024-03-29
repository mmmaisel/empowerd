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
use crate::{
    models::{
        units::{
            millisecond, second, watt, watt_hour, Abbreviation, Energy, Power,
            Time,
        },
        AvailablePower, Model,
    },
    pt1::PT1,
    task_group::TaskResult,
    Error,
};
use chrono::Utc;
use slog::{debug, warn, Logger};
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
    battery_threshold: Energy,
    skipped_events: u8,
    filter: PT1<Power>,
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
            battery_threshold: Energy::new::<watt_hour>(battery_threshold),
            skipped_events: 0,
            filter: PT1::new(
                Time::new::<second>(tau),
                Power::new::<watt>(0.0),
                Power::new::<watt>(-super::MAX_POWER_W),
                Power::new::<watt>(super::MAX_POWER_W),
                Time::new::<millisecond>(Utc::now().timestamp_millis() as f64),
            ),
        }
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        tokio::select! {
            _ = self.base.canceled.changed() => {
                return Err(Error::Canceled(self.base.name.clone()));
            }
            x = self.command_input.recv() => {
                if let Some(command) = x {
                    if let Err(e) = self.handle_command(command) {
                        return Err(Error::Bug(e));
                    }
                }
            }
            x = self.meter_input.changed() => {
                if let Err(e) = x {
                    return Err(Error::Bug(
                        format!("Reading meter input failed: {e}")
                    ));
                }
            }
            x = self.battery_input.changed() => {
                if let Err(e) = x {
                    return Err(Error::Bug(
                        format!("Reading battery input failed: {e}")
                    ));
                }
            }
        };

        let (meter_time, meter_power) = match *self.meter_input.borrow() {
            Model::BidirMeter(ref x) => (x.time, x.power),
            Model::SimpleMeter(ref x) => (x.time, x.power),
            Model::None => return Ok(()),
            _ => {
                return Err(Error::Bug(format!(
                    "Received invalid model from meter input: {:?}",
                    *self.meter_input.borrow()
                )))
            }
        };

        let battery = match *self.battery_input.borrow() {
            Model::Battery(ref x) => x.clone(),
            Model::None => return Ok(()),
            _ => {
                return Err(Error::Bug(format!(
                    "Received invalid model from battery input: {:?}",
                    *self.battery_input.borrow()
                )))
            }
        };

        if (meter_time - battery.time).abs() > Time::new::<second>(15.0) {
            self.skipped_events += 1;
            if self.skipped_events >= 2 {
                warn!(
                    self.base.logger,
                    "Skipping available power processor due to missing events"
                );
            }
            return Ok(());
        }
        self.skipped_events = 0;

        let filtered_power =
            self.filter.process(battery.power - meter_power, meter_time);
        debug!(
            self.base.logger,
            "Available power: {}",
            filtered_power.into_format_args(watt, Abbreviation)
        );
        let available_power = if battery.charge < self.battery_threshold {
            debug!(self.base.logger, "Battery is below threshold!");
            AvailablePower::new(
                meter_time,
                Power::new::<watt>(-super::MAX_POWER_W),
            )
        } else {
            AvailablePower::new(meter_time, filtered_power)
        };
        self.power_output.send_replace(available_power.into());

        Ok(())
    }

    fn handle_command(&mut self, command: Command) -> Result<(), String> {
        match command {
            Command::SetThreshold { threshold, resp } => {
                self.battery_threshold =
                    Energy::new::<watt_hour>(threshold.abs());
                if resp.send(()).is_err() {
                    return Err("Sending SetThreshold response failed!".into());
                }
            }
            Command::GetThreshold { resp } => {
                if resp
                    .send(self.battery_threshold.get::<watt_hour>())
                    .is_err()
                {
                    return Err("Sending GetThreshold response failed!".into());
                }
            }
            Command::GetPower { resp } => {
                let output = &*self.power_output.borrow();
                let power = match output {
                    Model::AvailablePower(x) => x.power,
                    Model::None => Power::new::<watt>(0.0),
                    _ => {
                        return Err(format!(
                            "power_output has invalid type: {output:?}",
                        ))
                    }
                };
                if resp.send(power.get::<watt>()).is_err() {
                    return Err("Sending GetPower response failed!".into());
                }
            }
        }

        Ok(())
    }
}
