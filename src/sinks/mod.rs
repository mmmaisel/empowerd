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
use crate::{
    settings::{Gpio, ModbusCoil, Settings, SinkType},
    switch_mux::{SwitchArgs, SwitchType},
    SwitchMux,
};
use slog::Logger;
use std::{collections::BTreeMap, fmt, net::SocketAddr, sync::Arc};
use tokio::sync::watch;

pub mod debug;
pub mod gpio_switch;
pub mod ke_contact;
pub mod lambda_heat_pump;
pub mod modbus_switch;

pub use debug::DebugSink;
pub use gpio_switch::GpioSwitch;
pub use ke_contact::KeContactSink;
pub use lambda_heat_pump::LambdaHeatPumpSink;
pub use modbus_switch::ModbusSwitch;

#[derive(Clone)]
pub enum ArcSink {
    Debug(Arc<DebugSink>),
    SwitchMux(Arc<SwitchMux>),
    LambdaHeatPump(Arc<LambdaHeatPumpSink>),
    KeContact(Arc<KeContactSink>),
}

impl fmt::Display for ArcSink {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match &self {
            ArcSink::Debug(_) => "Debug",
            ArcSink::SwitchMux(_) => "SwitchMux",
            ArcSink::LambdaHeatPump(_) => "LambdaHeatPump",
            ArcSink::KeContact(_) => "KeContact",
        };
        write!(f, "{}", name)
    }
}

pub struct SwitchProcCreateInfo {
    pub name: String,
    pub channel: watch::Receiver<bool>,
    pub on_time: u64,
}

pub fn make_sinks(
    logger: Logger,
    settings: &Settings,
) -> Result<(BTreeMap<String, ArcSink>, Vec<SwitchProcCreateInfo>), String> {
    let mut sinks = BTreeMap::new();
    let mut switches = BTreeMap::<SwitchType, Vec<SwitchArgs>>::new();
    let mut switch_proc_info = Vec::new();

    for sink in &settings.sinks {
        match &sink.variant {
            SinkType::Debug => {
                let obj = DebugSink::new(sink.name.clone(), logger.clone());
                sinks.insert(sink.name.clone(), ArcSink::Debug(Arc::new(obj)));
            }
            SinkType::Gpio(gpio) => {
                let proc = if gpio.on_time != Gpio::max_on_time() {
                    let (tx, rx) = watch::channel(false);
                    switch_proc_info.push(SwitchProcCreateInfo {
                        name: sink.name.clone(),
                        channel: rx,
                        on_time: gpio.on_time,
                    });
                    Some(tx)
                } else {
                    None
                };

                let typ = SwitchType::Gpio {
                    dev: gpio.dev.clone().into(),
                };
                let arg = SwitchArgs {
                    num: gpio.num as usize,
                    name: sink.name.clone(),
                    icon: gpio.icon.clone(),
                    proc,
                };

                if let Some(args) = switches.get_mut(&typ) {
                    args.push(arg);
                } else {
                    switches.insert(typ, vec![arg]);
                }
            }
            SinkType::ModbusCoil(coil) => {
                // TODO: dedup this with above
                let proc = if coil.on_time != ModbusCoil::max_on_time() {
                    let (tx, rx) = watch::channel(false);
                    switch_proc_info.push(SwitchProcCreateInfo {
                        name: sink.name.clone(),
                        channel: rx,
                        on_time: coil.on_time,
                    });
                    Some(tx)
                } else {
                    None
                };

                let typ = SwitchType::Modbus {
                    addr: SocketAddr::V4(coil.addr.clone()),
                    id: coil.id,
                };
                let arg = SwitchArgs {
                    num: coil.num as usize,
                    name: sink.name.clone(),
                    icon: coil.icon.clone(),
                    proc,
                };

                if let Some(args) = switches.get_mut(&typ) {
                    args.push(arg);
                } else {
                    switches.insert(typ, vec![arg]);
                }
            }
            SinkType::LambdaHeatPump(setting) => {
                let obj = LambdaHeatPumpSink::new(
                    sink.name.clone(),
                    setting.address.clone(),
                    logger.clone(),
                )?;
                sinks.insert(
                    sink.name.clone(),
                    ArcSink::LambdaHeatPump(Arc::new(obj)),
                );
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
        }
    }

    let switch_mux = SwitchMux::new(switches)
        .map_err(|e| format!("Could not create SwitchMux: {}", e))?;
    sinks.insert(
        "_SwitchMux".into(),
        ArcSink::SwitchMux(Arc::new(switch_mux)),
    );
    Ok((sinks, switch_proc_info))
}
