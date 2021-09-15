/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

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
use super::{Miner, MinerResult, MinerState};
use crate::miner_sleep;
use slog::Logger;
use std::time::Duration;
use tokio::sync::watch;

pub struct DummyMiner {
    canceled: watch::Receiver<MinerState>,
    name: String,
    interval: Duration,
    logger: Logger,
}

impl DummyMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        name: String,
        interval: Duration,
        logger: Logger,
    ) -> Result<Self, String> {
        return Ok(Self {
            canceled: canceled,
            name: name,
            interval: interval,
            logger: logger.clone(),
        });
    }

    pub async fn mine(&mut self) -> MinerResult {
        miner_sleep!(self);
        return MinerResult::Running;
    }
}
