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
use crate::settings::{Settings, SinkType};
use gpio_switch::GpioCreateInfo;
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
    KeContact(Arc<KeContactSink>),
    GpioSwitch(Arc<GpioSwitch>),
}

pub fn make_sinks(
    logger: Logger,
    settings: &Settings,
) -> Result<BTreeMap<String, ArcSink>, String> {
    let mut sinks = BTreeMap::new();
    let mut gpios = Vec::new();

    for sink in &settings.sinks {
        match &sink.variant {
            SinkType::Debug => {
                let obj = DebugSink::new(sink.name.clone(), logger.clone());
                sinks.insert(sink.name.clone(), ArcSink::Debug(Arc::new(obj)));
            }
            SinkType::KeContact(setting) => {
                let obj = KeContactSink::new(
                    sink.name.clone(),
                    setting.address.clone(),
                    logger.clone(),
                )?;
                sinks.insert(
                    sink.name.clone(),
                    ArcSink::KeContact(Arc::new(obj)),
                );
            }
            SinkType::Gpio(gpio) => {
                gpios.push(GpioCreateInfo {
                    name: sink.name.clone(),
                    icon: gpio.icon.clone(),
                    dev: gpio.dev.clone(),
                    num: gpio.num,
                });
            }
        }
    }

    let gpio_switch = GpioSwitch::new(gpios)
        .map_err(|e| format!("Could not create GPIO switch: {}", e))?;
    sinks.insert(
        "_GpioSwitch".into(),
        ArcSink::GpioSwitch(Arc::new(gpio_switch)),
    );
    Ok(sinks)
}
