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
use crate::task_group::TaskResult;
use crate::GpioSwitch;
use slog::{debug, error};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

pub struct PoweroffTimerProcessor {
    base: ProcessorBase,
    gpio_input: watch::Receiver<bool>,
    gpio_output: Arc<GpioSwitch>,
    gpio_id: usize,
    on_time: Duration,
    sleep_time: Duration,
}

impl PoweroffTimerProcessor {
    pub fn new(
        base: ProcessorBase,
        gpio_input: watch::Receiver<bool>,
        gpio_output: Arc<GpioSwitch>,
        gpio_id: usize,
        on_time: Duration,
    ) -> Self {
        Self {
            base,
            gpio_input,
            gpio_output,
            gpio_id,
            on_time,
            sleep_time: on_time,
        }
    }

    pub async fn run(&mut self) -> TaskResult {
        tokio::select! {
            _ = self.base.canceled.changed() => {
                return TaskResult::Canceled(self.base.name.clone());
            }
            _ = tokio::time::sleep(self.sleep_time) => {
                if let Err(e) = self.update_output(false) {
                    return e;
                }
                debug!(self.base.logger, "Poweroff GPIO '{}'", self.gpio_id);
            }
            x = self.gpio_input.changed() => {
                if let Err(e) = x {
                    return TaskResult::Err(format!(
                        "Reading gpio input failed: {}",
                        e
                    ));
                }

                let value = self.gpio_input.borrow().to_owned();
                if let Err(e) = self.update_output(value) {
                    return e;
                }
                debug!(self.base.logger, "Set GPIO '{}' to '{}' for '{}' seconds", self.gpio_id, value, self.sleep_time.as_secs());
            }
        };

        TaskResult::Running
    }

    fn update_output(&mut self, value: bool) -> Result<(), TaskResult> {
        let channel = match self.gpio_output.get_channel(self.gpio_id) {
            Ok(x) => x,
            Err(e) => {
                error!(self.base.logger, "{}", e);
                return Err(TaskResult::Running);
            }
        };
        if let Err(e) = self.gpio_output.set_open_raw(channel, value) {
            error!(self.base.logger, "{}", e);
            return Err(TaskResult::Running);
        }
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
}
