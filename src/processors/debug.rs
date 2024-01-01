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
use crate::{
    models::Model, sinks::debug::DebugSink, task_group::TaskResult, Error,
};
use slog::{debug, Logger};
use std::sync::Arc;
use tokio::sync::watch;

pub struct DebugProcessor {
    base: ProcessorBase,
    input: watch::Receiver<Model>,
    output: Arc<DebugSink>,
}

impl DebugProcessor {
    pub fn new(
        base: ProcessorBase,
        input: watch::Receiver<Model>,
        output: Arc<DebugSink>,
    ) -> Self {
        Self {
            base,
            input,
            output,
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
            x = self.input.changed() => {
                if let Err(e) = x {
                    return Err(Error::Bug(
                        format!("Reading input failed: {e}")
                    ));
                }
                let y = self.input.borrow().clone();
                debug!(
                    self.base.logger,
                    "Input of {} has changed: {:?}",
                    self.base.name,
                    y,
                );
                self.output.log_value(y).await;
            }
        };

        Ok(())
    }
}
