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
use crate::{sinks::GpioSwitch, task_group::TaskResult, Error};
use slog::{debug, Logger};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};

#[derive(Debug)]
pub enum Command {
    SetOnTime {
        on_time: Duration,
        resp: oneshot::Sender<()>,
    },
    GetOnTime {
        resp: oneshot::Sender<Duration>,
    },
}

pub struct PoweroffTimerProcessor {
    base: ProcessorBase,
    command_input: mpsc::Receiver<Command>,
    gpio_input: watch::Receiver<bool>,
    gpio_output: Arc<GpioSwitch>,
    gpio_id: usize,
    on_time: Duration,
    sleep_time: Duration,
}

impl PoweroffTimerProcessor {
    pub fn new(
        base: ProcessorBase,
        command_input: mpsc::Receiver<Command>,
        gpio_input: watch::Receiver<bool>,
        gpio_output: Arc<GpioSwitch>,
        gpio_id: usize,
        on_time: Duration,
    ) -> Self {
        Self {
            base,
            command_input,
            gpio_input,
            gpio_output,
            gpio_id,
            on_time,
            sleep_time: on_time,
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
                    self.handle_command(command)?;
                }
            }
            _ = tokio::time::sleep(self.sleep_time) => {
                self.update_output(false)?;
                debug!(self.base.logger, "Poweroff GPIO '{}'", self.gpio_id);
            }
            x = self.gpio_input.changed() => {
                if let Err(e) = x {
                    return Err(Error::Bug(format!(
                        "Reading gpio input failed: {e}",
                    )));
                }

                let value = self.gpio_input.borrow().to_owned();
                self.update_output(value)?;
                debug!(
                    self.base.logger,
                    "Set GPIO '{}' to '{}' for '{}' seconds",
                    self.gpio_id,
                    value,
                    self.sleep_time.as_secs(),
                );
            }
        };

        Ok(())
    }

    fn update_output(&mut self, value: bool) -> Result<(), Error> {
        let channel = self
            .gpio_output
            .get_channel(self.gpio_id)
            .map_err(Error::Temporary)?;
        self.gpio_output
            .set_open_raw(channel, value)
            .map_err(Error::Temporary)?;
        self.sleep_time = self.calc_sleep_time(value);

        Ok(())
    }

    fn calc_sleep_time(&self, value: bool) -> Duration {
        if value {
            self.on_time
        } else {
            Duration::from_secs(u64::MAX)
        }
    }

    fn handle_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::SetOnTime { on_time, resp } => {
                self.on_time = on_time;
                if resp.send(()).is_err() {
                    return Err(Error::Bug(
                        "Sending SetOnTime response failed!".into(),
                    ));
                }
            }
            Command::GetOnTime { resp } => {
                if resp.send(self.on_time).is_err() {
                    return Err(Error::Bug(
                        "Sending GetOnTime response failed!".into(),
                    ));
                }
            }
        }

        Ok(())
    }
}
