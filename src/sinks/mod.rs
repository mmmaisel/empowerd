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
use crate::settings::{Settings, Sink};
use slog::Logger;
use std::collections::BTreeMap;
use std::sync::Arc;

pub mod debug;
pub mod gpio_switch;
pub mod ke_contact;

pub use debug::DebugSink;
pub use gpio_switch::GpioSwitch;
pub use ke_contact::KeContactSink;

#[derive(Clone)]
pub enum ArcSink {
    Debug(Arc<DebugSink>),
    //GpioSwitch(Arc<GpioSwitch>), should be GpioPin
}

pub fn make_sinks(
    logger: Logger,
    settings: &Settings,
) -> Result<BTreeMap<String, ArcSink>, String> {
    let mut sinks = BTreeMap::new();
    for sink in &settings.sinks {
        match sink {
            Sink::Debug(settings) => {
                let debug =
                    DebugSink::new(settings.name.clone(), logger.clone());
                sinks.insert(
                    settings.name.clone(),
                    ArcSink::Debug(Arc::new(debug)),
                );
            }
            _ => {
                return Err("Not implemented yet".into());
            }
        }
    }
    Ok(sinks)
}
