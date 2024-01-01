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
use crate::{task_group::TaskResult, Error};
use slog::Logger;
use std::time::Duration;

pub struct DummyProcessor {
    base: ProcessorBase,
}

impl DummyProcessor {
    pub fn new(base: ProcessorBase) -> Self {
        Self { base }
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        tokio::select! {
            _ = self.base.canceled.changed() =>
                return Err(Error::Canceled(self.base.name.clone())),
            _ = tokio::time::sleep(Duration::from_secs(86400)) => (),
        }

        Ok(())
    }
}
