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
use super::SourceBase;
use crate::task_group::{TaskResult, TaskState};
use slog::Logger;
use std::time::Duration;
use tokio::sync::watch;

pub struct DummySource {
    base: SourceBase,
}

impl DummySource {
    pub fn new(
        canceled: watch::Receiver<TaskState>,
        influx: influxdb::Client,
        name: String,
        interval: Duration,
        logger: Logger,
    ) -> Self {
        Self {
            base: SourceBase::new(canceled, influx, name, interval, logger),
        }
    }

    pub async fn run(&mut self) -> TaskResult {
        match self.base.sleep_aligned().await {
            Ok(_) => (),
            Err(e) => return e,
        };
        TaskResult::Running
    }
}
