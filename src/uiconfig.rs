/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2025 Max Maisel

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
use serde::Serialize;

use super::settings::{Settings, SinkType, SourceType, WeatherLabels};

#[derive(Clone, Debug, Default, Serialize)]
pub struct Ranges {
    production: [Option<f64>; 2],
    consumption: Option<f64>,
    battery: [Option<f64>; 2],
    boiler: [Option<f64>; 2],
    heating: Option<f64>,
    cop: Option<f64>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UiConfig {
    batteries: Vec<i32>,
    controls: bool,
    generators: Vec<i32>,
    heatpumps: Vec<i32>,
    meters: Vec<i32>,
    solars: Vec<i32>,
    wallboxes: Vec<i32>,
    weathers: Vec<i32>,
    labels: WeatherLabels,
    ranges: Ranges,
}

fn add_assign_opt(lhs: &mut Option<f64>, rhs: f64) {
    *lhs = match lhs {
        None => Some(rhs),
        Some(x) => Some(*x + rhs),
    }
}

impl UiConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        let mut config = Self::default();

        for source in &settings.sources {
            match &source.variant {
                SourceType::Debug(_) => {}
                SourceType::SunnyIsland(setting) => {
                    config.batteries.push(source.series_id);
                    if let Some(model) = &setting.model {
                        add_assign_opt(
                            &mut config.ranges.battery[0],
                            model.threshold,
                        );
                        add_assign_opt(
                            &mut config.ranges.battery[1],
                            model.capacity,
                        );
                    }
                }
                SourceType::SunnyBoyStorage(setting) => {
                    config.batteries.push(source.series_id);
                    if let Some(model) = &setting.model {
                        add_assign_opt(
                            &mut config.ranges.battery[0],
                            model.threshold,
                        );
                        add_assign_opt(
                            &mut config.ranges.battery[1],
                            model.capacity,
                        );
                    }
                }
                SourceType::SunspecSolar(setting) => {
                    config.solars.push(source.series_id);
                    if let Some(model) = &setting.model {
                        add_assign_opt(
                            &mut config.ranges.production[1],
                            model.peak_power,
                        );
                    }
                }
                SourceType::DachsMsrS(_setting) => {
                    config.generators.push(source.series_id);
                }
                SourceType::KeContact(_setting) => {
                    config.wallboxes.push(source.series_id);
                }
                SourceType::LambdaHeatPump(setting) => {
                    config.heatpumps.push(source.series_id);
                    if let Some(model) = &setting.model {
                        config.ranges.cop = Some(
                            config.ranges.cop.map_or(model.peak_cop, |x| {
                                x.max(model.peak_cop)
                            }),
                        );
                        add_assign_opt(
                            &mut config.ranges.heating,
                            model.peak_heat,
                        );
                    }
                }
                SourceType::SmaMeter(setting) => {
                    config.meters.push(source.series_id);
                    if let Some(model) = &setting.model {
                        config.ranges.consumption = Some(model.peak_power);
                    }
                }
                SourceType::SmlMeter(setting) => {
                    config.meters.push(source.series_id);
                    if let Some(model) = &setting.model {
                        config.ranges.consumption = Some(model.peak_power);
                    }
                }
                SourceType::SunnyBoySpeedwire(setting) => {
                    config.solars.push(source.series_id);
                    if let Some(model) = &setting.model {
                        add_assign_opt(
                            &mut config.ranges.production[1],
                            model.peak_power,
                        );
                    }
                }
                SourceType::Bresser6in1(setting) => {
                    config.weathers.push(source.series_id);
                    config.labels = setting.labels.clone();
                }
            }
        }

        config.ranges.production[0] = config.ranges.consumption.map(|x| -x);

        for sink in &settings.sinks {
            match sink.variant {
                SinkType::Debug => (),
                SinkType::Gpio(_) => config.controls = true,
                SinkType::ModbusCoil(_) => config.controls = true,
                SinkType::KeContact(_) => config.controls = true,
                SinkType::LambdaHeatPump(_) => config.controls = true,
            }
        }

        config
    }
}
