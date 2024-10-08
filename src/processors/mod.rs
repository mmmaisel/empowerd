/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2023 Max Maisel

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
    models::{
        units::{joule, watt, watt_hour, Energy, Power},
        Model,
    },
    multi_setpoint_hysteresis::LinspaceBuilder,
    seasonal::SeasonalBuilder,
    settings::{ProcessorType, Settings},
    sinks::{ArcSink, SwitchProcCreateInfo},
    task_group::{task_loop, TaskGroup, TaskGroupBuilder, TaskState},
};
use slog::{debug, error, Logger};
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};

mod appliance;
mod available_power;
mod debug;
mod dummy;
mod load_control;
mod poweroff_timer;

pub use appliance::{ApplianceProcessor, Command as ApplianceCmd};
pub use available_power::{
    AvailablePowerProcessor, Command as AvailablePowerCmd,
};
pub use debug::DebugProcessor;
pub use dummy::DummyProcessor;
pub use load_control::{Command as LoadControlCmd, LoadControlProcessor};
pub use poweroff_timer::{Command as PoweroffTimerCmd, PoweroffTimerProcessor};

pub const MAX_POWER_W: f64 = 12800.0;

#[derive(Debug)]
pub struct CommandSender<T> {
    pub name: String,
    pub switch_id: Option<usize>,
    pub tx: mpsc::Sender<T>,
}

impl<T> CommandSender<T> {
    pub async fn issue_command<U>(
        &self,
        logger: &Logger,
        cmd: T,
        rx: oneshot::Receiver<U>,
    ) -> Result<U, String> {
        if let Err(e) = self.tx.send(cmd).await {
            error!(logger, "Sending command to '{}' failed: {}", self.name, e);
            return Err("Internal server error!".into());
        }

        match rx.await {
            Ok(x) => Ok(x),
            Err(e) => {
                error!(
                    logger,
                    "Receiving command from '{}' failed: {}", self.name, e
                );
                return Err("Internal server error!".into());
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ProcessorCommands {
    pub available_power: Vec<CommandSender<AvailablePowerCmd>>,
    pub appliance: Vec<CommandSender<ApplianceCmd>>,
    pub load_control: Option<CommandSender<LoadControlCmd>>,
    pub poweroff_timer: Vec<CommandSender<PoweroffTimerCmd>>,
}

pub struct ProcessorInfo {
    pub tasks: TaskGroup,
    pub commands: ProcessorCommands,
}

pub struct ProcessorBase {
    name: String,
    canceled: watch::Receiver<TaskState>,
    logger: Logger,
}

impl ProcessorBase {
    pub fn new(
        name: String,
        canceled: watch::Receiver<TaskState>,
        logger: Logger,
    ) -> Self {
        Self {
            name,
            canceled,
            logger,
        }
    }
}

pub fn processor_tasks(
    logger: Logger,
    settings: &Settings,
    mut inputs: BTreeMap<String, watch::Receiver<Model>>,
    sinks: BTreeMap<String, ArcSink>,
    switch_info: Vec<SwitchProcCreateInfo>,
) -> Result<ProcessorInfo, String> {
    let tasks = TaskGroupBuilder::new("processors".into(), logger.clone());
    let mut outputs = BTreeMap::<String, watch::Sender<Model>>::new();
    let mut commands = ProcessorCommands::default();

    for p in &settings.processors {
        let (tx, rx) = watch::channel(Model::None);
        inputs.insert(p.name.clone(), rx);
        outputs.insert(p.name.clone(), tx);
    }

    for p in &settings.processors {
        match &p.variant {
            ProcessorType::AvailablePower(setting) => {
                let battery_source = match inputs.get(&setting.battery_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing battery input for Processor {}",
                            &p.name
                        ))
                    }
                };
                let meter_source = match inputs.get(&setting.meter_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing meter input for Processor {}",
                            &p.name
                        ))
                    }
                };
                let power_output = match outputs.remove(&p.name) {
                    Some(x) => x,
                    None => {
                        return Err(format!(
                            "Missing power output for Processor {}",
                            &p.name
                        ))
                    }
                };
                let (command_tx, command_rx) = mpsc::channel(1);
                let mut processor = AvailablePowerProcessor::new(
                    ProcessorBase::new(
                        p.name.clone(),
                        tasks.cancel_rx(),
                        logger.clone(),
                    ),
                    command_rx,
                    battery_source,
                    meter_source,
                    power_output,
                    setting.battery_threshold,
                    setting.tau,
                );
                tasks.add_task(task_loop!(processor));
                commands.available_power.push(CommandSender {
                    name: p.name.clone(),
                    switch_id: None,
                    tx: command_tx,
                });
            }
            ProcessorType::Debug(setting) => {
                let sink = match sinks.get(&setting.output) {
                    Some(x) => x.to_owned(),
                    None => {
                        return Err(format!(
                            "Missing sink for Processor {}",
                            &p.name
                        ))
                    }
                };
                let source = match inputs.get(&setting.input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing source for Processor {}",
                            &p.name
                        ))
                    }
                };
                if let ArcSink::Debug(sink) = sink {
                    let mut processor = DebugProcessor::new(
                        ProcessorBase::new(
                            p.name.clone(),
                            tasks.cancel_rx(),
                            logger.clone(),
                        ),
                        source,
                        sink,
                    );
                    tasks.add_task(task_loop!(processor));
                } else {
                    return Err(
                        "Unsupported sink type for DebugProcessor".into()
                    );
                }
            }
            ProcessorType::Appliance(setting) => {
                let power_source = match inputs.get(&setting.power_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing power input for Processor {}",
                            &p.name
                        ))
                    }
                };
                let power_sink: watch::Sender<Model> =
                    match outputs.remove(&p.name) {
                        Some(x) => x,
                        None => {
                            return Err(format!(
                                "Missing power output for Processor {}",
                                &p.name
                            ))
                        }
                    };
                let appliance_source =
                    match inputs.get(&setting.appliance_input) {
                        Some(x) => x.clone(),
                        None => {
                            return Err(format!(
                                "Missing appliance source for Processor {}",
                                &p.name
                            ))
                        }
                    };
                let appliance_sink = match sinks.get(&setting.appliance_output)
                {
                    Some(x) => x.to_owned(),
                    None => {
                        return Err(format!(
                            "Missing sink 'appliance_output' for Processor {}",
                            &p.name
                        ))
                    }
                };

                if !ApplianceProcessor::validate_appliance(&appliance_sink) {
                    return Err(format!(
                        "Unsupported sink type '{}' for ApplianceProcessor",
                        &appliance_sink
                    ));
                }

                let seasonal = match setting.seasonal {
                    Some(ref x) => {
                        let builder = match settings.location {
                            Some(ref y) => SeasonalBuilder::new()
                                .latitude(y.latitude)
                                .longitude(y.longitude),
                            None => return Err(
                                "Location is required for seasonal correction"
                                    .into(),
                            ),
                        };
                        Some(
                            builder
                                .offset_hour(x.offset)
                                .gain_per_hour(x.gain)
                                .phase_days(x.phase as f64)
                                .build()?,
                        )
                    }
                    None => None,
                };

                let (command_tx, command_rx) = mpsc::channel(1);
                let mut processor = ApplianceProcessor::new(
                    ProcessorBase::new(
                        p.name.clone(),
                        tasks.cancel_rx(),
                        logger.clone(),
                    ),
                    command_rx,
                    power_source,
                    appliance_source,
                    power_sink,
                    appliance_sink,
                    Duration::from_secs(setting.retransmit_interval),
                    seasonal,
                );
                tasks.add_task(task_loop!(processor));
                commands.appliance.push(CommandSender {
                    name: p.name.clone(),
                    switch_id: None,
                    tx: command_tx,
                });
            }
            ProcessorType::LoadControl(setting) => {
                let battery_source = match inputs.get(&setting.battery_input) {
                    Some(x) => x.clone(),
                    None => {
                        return Err(format!(
                            "Missing battery input for Processor {}",
                            &p.name
                        ))
                    }
                };

                let controller = LinspaceBuilder::new(
                    Energy::new::<joule>(-f64::MAX),
                    Energy::new::<joule>(f64::MAX),
                )
                .input_range(
                    Energy::new::<watt_hour>(setting.battery_empty_cap),
                    Energy::new::<watt_hour>(setting.battery_threshold_cap),
                )
                .output_range(
                    Power::new::<watt>(-setting.basic_load),
                    Power::new::<watt>(-setting.basic_load),
                    Power::new::<watt>(-setting.min_grid_power),
                    Power::new::<watt>(0.0),
                )
                .point_count(setting.num_points)
                .hysteresis(Energy::new::<watt_hour>(setting.hysteresis_cap))
                .build()
                .map_err(|e| {
                    format!("Creating multi setpoint controller failed: {e}")
                })?;

                let seasonal = match setting.seasonal {
                    Some(ref x) => {
                        let builder = match settings.location {
                            Some(ref y) => SeasonalBuilder::new()
                                .latitude(y.latitude)
                                .longitude(y.longitude),
                            None => return Err(
                                "Location is required for seasonal correction"
                                    .into(),
                            ),
                        };
                        Some(
                            builder
                                .offset_hour(x.offset)
                                .gain_per_hour(x.gain)
                                .phase_days(x.phase as f64)
                                .build()?,
                        )
                    }
                    None => None,
                };

                let (command_tx, command_rx) = mpsc::channel(1);
                let mut processor = match LoadControlProcessor::new(
                    ProcessorBase::new(
                        p.name.clone(),
                        tasks.cancel_rx(),
                        logger.clone(),
                    ),
                    command_rx,
                    setting.meter_susy_id,
                    setting.meter_serial,
                    setting.bind_addr,
                    setting.ctrl_serial,
                    battery_source,
                    controller,
                    seasonal,
                    Power::new::<watt>(-setting.charge_power),
                ) {
                    Ok(x) => x,
                    Err(e) => {
                        return Err(format!(
                            "Could not create LoadControlProcessor: {}",
                            e
                        ))
                    }
                };
                tasks.add_task(task_loop!(processor));
                commands.load_control = Some(CommandSender {
                    name: p.name.clone(),
                    switch_id: None,
                    tx: command_tx,
                });
            }
        }
    }

    if let Some(x) = sinks.get("_SwitchMux") {
        match x {
            ArcSink::SwitchMux(switch_mux) => {
                for switch in switch_info.into_iter() {
                    let id = switch_mux.id_by_name(&switch.name)?;
                    let name =
                        format!("_PoweroffTimerProcessor_{}", switch.name);
                    let (command_tx, command_rx) = mpsc::channel(1);

                    let mut processor = PoweroffTimerProcessor::new(
                        ProcessorBase::new(
                            name.clone(),
                            tasks.cancel_rx(),
                            logger.clone(),
                        ),
                        command_rx,
                        switch.channel,
                        switch_mux.clone(),
                        id,
                        Duration::from_secs(switch.on_time),
                    );
                    tasks.add_task(task_loop!(processor));
                    commands.poweroff_timer.push(CommandSender {
                        name,
                        switch_id: Some(id),
                        tx: command_tx,
                    });
                }
            }
            _ => return Err("Unsupported sink type for PoweroffTimer".into()),
        }
    }

    if !tasks.has_tasks() {
        debug!(logger, "No processors enabled, using dummy");
        let mut dummy = DummyProcessor::new(ProcessorBase::new(
            "dummy".into(),
            tasks.cancel_rx(),
            logger,
        ));
        tasks.add_task(task_loop!(dummy));
    }

    Ok(ProcessorInfo {
        tasks: tasks.build(),
        commands,
    })
}
