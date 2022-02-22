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
use crate::models::Model;
use slog::{debug, Logger};

pub struct DebugSink {
    name: String,
    logger: Logger,
}

impl DebugSink {
    pub fn new(name: String, logger: Logger) -> Self {
        Self { name, logger }
    }

    pub async fn log_value(&self, value: Model) {
        debug!(
            self.logger,
            "DebugSink {} received value: {:?}", self.name, value
        );
    }
}
