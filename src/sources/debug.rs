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
use crate::models::{
    units::{joule, second, watt, Energy, Power, Time},
    SimpleMeter,
};
use crate::task_group::TaskResult;
use slog::debug;

pub struct DebugSource {
    base: SourceBase,
    energy: Energy,
    phase: f64,
}

impl DebugSource {
    pub fn new(base: SourceBase) -> Self {
        Self {
            base,
            energy: Energy::new::<joule>(0.0),
            phase: 0.0,
        }
    }

    pub async fn run(&mut self) -> TaskResult {
        let (now, _oversample) = match self.base.sleep_aligned().await {
            Ok(x) => (Time::new::<second>(x.now as f64), x.oversample),
            Err(e) => return e,
        };

        let power = Power::new::<watt>(self.phase.sin().abs());
        let energy_inc =
            power * Time::new::<second>(self.base.interval.as_secs() as f64);
        let record = SimpleMeter::new(now, self.energy + energy_inc, power);
        self.energy += energy_inc;
        self.phase += 0.1;

        debug!(self.base.logger, "Emitting {:?}", &record);
        self.base.notify_processors(&record);
        TaskResult::Running
    }
}
