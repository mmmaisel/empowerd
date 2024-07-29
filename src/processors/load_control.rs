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
use super::ProcessorBase;
use crate::{
    models::{
        units::{watt, watt_hour, Abbreviation, Energy, Power},
        Model,
    },
    multi_setpoint_hysteresis::MultiSetpointHysteresis,
    seasonal::Seasonal,
    task_group::TaskResult,
    Error,
};
use slog::{debug, Logger};
use sma_proto::{
    client::{SmaClient, SmaSession},
    energymeter::ObisValue,
    SmaEndpoint,
};
use std::net::Ipv4Addr;
use tokio::{
    sync::{mpsc, oneshot, watch},
    time::{self, Duration},
};

#[cfg(debug_assertions)]
use {slog::trace, std::time::Instant};

const OBIS_SUM_BASE: u32 = 0x00010000;
const OBIS_L1_BASE: u32 = 0x00150000;
const OBIS_L2_BASE: u32 = 0x00290000;
const OBIS_L3_BASE: u32 = 0x003D0000;
const OBIS_ACTIVE_PWR_P: u32 = 0x00000400;
const OBIS_ACTIVE_PWR_N: u32 = 0x00010400;
const OBIS_REACTIVE_PWR_P: u32 = 0x00020400;
const OBIS_REACTIVE_PWR_N: u32 = 0x00030400;
const OBIS_APPARENT_PWR_P: u32 = 0x00080400;
const OBIS_APPARENT_PWR_N: u32 = 0x00090400;
const OBIS_CURRENT: u32 = 0x000a0400;
const OBIS_VOLTAGE: u32 = 0x000b0400;
const OBIS_POWER_FACTOR: u32 = 0x000c0400;
const OBIS_VERSION: u32 = 0x90000000;

#[derive(Debug)]
pub enum Command {
    SetChargeMode {
        enabled: bool,
        resp: oneshot::Sender<()>,
    },
    GetChargeMode {
        resp: oneshot::Sender<bool>,
    },
}

pub struct LoadControlProcessor {
    base: ProcessorBase,
    command_input: mpsc::Receiver<Command>,
    meter_endpoint: SmaEndpoint,
    battery_input: watch::Receiver<Model>,

    grid_power: Power,
    controller: MultiSetpointHysteresis<Energy, Power>,
    seasonal: Option<Seasonal>,
    charge_power: Power,
    charge_power_setpoint: Power,

    sma_client: SmaClient,
    session: SmaSession,
}

impl LoadControlProcessor {
    pub fn new(
        base: ProcessorBase,
        command_input: mpsc::Receiver<Command>,
        meter_susy_id: u16,
        meter_serial: u32,
        bind_addr: Ipv4Addr,
        ctrl_serial: u32,
        battery_input: watch::Receiver<Model>,
        controller: MultiSetpointHysteresis<Energy, Power>,
        seasonal: Option<Seasonal>,
        charge_power_setpoint: Power,
    ) -> Result<Self, String> {
        let ctrl_endpoint = SmaEndpoint {
            susy_id: meter_susy_id,
            serial: ctrl_serial,
        };
        let sma_client = SmaClient::new(ctrl_endpoint);
        let session = SmaSession::open_multicast(bind_addr)
            .map_err(|e| format!("Could not open SMA Client session: {e}"))?;

        Ok(Self {
            base,
            command_input,
            meter_endpoint: SmaEndpoint {
                susy_id: meter_susy_id,
                serial: meter_serial,
            },
            battery_input,
            grid_power: Power::new::<watt>(0.0),
            controller,
            seasonal,
            charge_power: Power::new::<watt>(0.0),
            charge_power_setpoint,
            sma_client,
            session,
        })
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        #[cfg(debug_assertions)]
        let now = Instant::now();

        match self.base.canceled.has_changed() {
            Ok(changed) => {
                if changed {
                    return Err(Error::Canceled(self.base.name.clone()));
                }
            }
            Err(e) => {
                return Err(Error::Bug(format!(
                    "Reading cancel event failed: {e}",
                )))
            }
        }

        let (timestamp_ms, em_data) =
            time::timeout(Duration::from_millis(500), async {
                self.sma_client
                    .read_em_message(&self.session, &self.meter_endpoint)
                    .await
            })
            .await
            .map_err(|_e| Error::Temporary("Reading EM data timed out".into()))?
            .map_err(|e| {
                Error::Temporary(format!("Reading EM data failed: {e}"))
            })?;

        let mut payload: EmIndependentPayload = em_data.try_into().map_err(|e|
            Error::Temporary(format!(
                "Extracting independent energymeter values from response failed: {e}",
            ))
        )?;

        let command_received = match self.command_input.try_recv() {
            Ok(command) => {
                self.handle_command(command)?;
                true
            }
            Err(mpsc::error::TryRecvError::Empty) => false,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                return Err(Error::Bug("Command input closed".to_string()))
            }
        };

        let battery_changed =
            self.battery_input.has_changed().map_err(|e| {
                Error::Bug(format!("Reading battery input failed: {e}"))
            })?;

        if command_received || battery_changed {
            match *self.battery_input.borrow() {
                Model::None => (),
                Model::Battery(ref x) => {
                    let (new_grid_power, seasonal_correction) =
                        Self::calc_grid_power(
                            &mut self.controller,
                            &self.seasonal,
                            self.charge_power,
                            x.charge.to_owned(),
                        );
                    // Print a debug message when grid power has changed.
                    if (self.grid_power - new_grid_power).abs()
                        > Power::new::<watt>(0.1)
                    {
                        debug!(
                            self.base.logger,
                            "Importing {} from grid with seasonal correction {}",
                            new_grid_power
                                .into_format_args(watt, Abbreviation),
                            seasonal_correction.into_format_args(watt_hour, Abbreviation),
                        );
                    }
                    self.grid_power = new_grid_power;
                }
                _ => {
                    return Err(Error::Temporary(format!(
                        "Received invalid model from battery input: {:?}",
                        *self.battery_input.borrow()
                    )))
                }
            }
        }

        payload.apply_power_offset(self.grid_power.get::<watt>());
        if let Err(e) = self
            .sma_client
            .write_em_message(
                &self.session,
                timestamp_ms,
                payload.into_dependent().into(),
            )
            .await
        {
            return Err(Error::Temporary(format!(
                "Broadcasting EM message failed: {e}",
            )));
        }

        #[cfg(debug_assertions)]
        trace!(
            self.base.logger,
            "LoadCtrl loop took {} us",
            now.elapsed().as_micros()
        );

        Ok(())
    }

    fn calc_grid_power(
        controller: &mut MultiSetpointHysteresis<Energy, Power>,
        seasonal: &Option<Seasonal>,
        charge_power: Power,
        charge: Energy,
    ) -> (Power, Energy) {
        let seasonal_correction = match seasonal {
            Some(ref x) => Energy::new::<watt_hour>(x.current_correction()),
            None => Energy::new::<watt_hour>(0.0),
        };

        let new_grid_power = controller.process(charge + seasonal_correction);
        (new_grid_power + charge_power, seasonal_correction)
    }

    fn handle_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::SetChargeMode { enabled, resp } => {
                if enabled {
                    self.charge_power = self.charge_power_setpoint;
                } else {
                    self.charge_power = Power::new::<watt>(0.0);
                }

                if resp.send(()).is_err() {
                    return Err(Error::Bug(
                        "Sending SetChargeMode response failed!".into(),
                    ));
                }
            }
            Command::GetChargeMode { resp } => {
                let enabled = self.charge_power > Power::new::<watt>(0.1);
                if resp.send(enabled).is_err() {
                    return Err(Error::Bug(
                        "Sending GetChargeMode response failed!".into(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
struct EmIndependentPayload {
    active_pwr: [f64; 3],
    reactive_pwr: [f64; 3],
    voltage: [f64; 3],
    version: u32,
}

impl EmIndependentPayload {
    pub fn apply_power_offset(&mut self, offset_w: f64) {
        let phase_offset_w = offset_w / 3.0 * 10.0;
        self.active_pwr[0] += phase_offset_w;
        self.active_pwr[1] += phase_offset_w;
        self.active_pwr[2] += phase_offset_w;
    }

    pub fn into_dependent(self) -> EmDependentPayload {
        EmDependentPayload::from_independent(self)
    }
}

impl TryFrom<Vec<ObisValue>> for EmIndependentPayload {
    type Error = String;
    fn try_from(data: Vec<ObisValue>) -> Result<Self, Self::Error> {
        let mut found = 0u16;

        let mut active_l1_p = 0f64;
        let mut active_l1_n = 0f64;
        let mut reactive_l1_p = 0f64;
        let mut reactive_l1_n = 0f64;
        let mut voltage_l1 = 0f64;

        let mut active_l2_p = 0f64;
        let mut active_l2_n = 0f64;
        let mut reactive_l2_p = 0f64;
        let mut reactive_l2_n = 0f64;
        let mut voltage_l2 = 0f64;

        let mut active_l3_p = 0f64;
        let mut active_l3_n = 0f64;
        let mut reactive_l3_p = 0f64;
        let mut reactive_l3_n = 0f64;
        let mut voltage_l3 = 0f64;

        let mut version = 0u32;

        for obis in data {
            if obis.id == OBIS_L1_BASE + OBIS_ACTIVE_PWR_P {
                found |= 1 << 0;
                active_l1_p = obis.value as f64;
            } else if obis.id == OBIS_L1_BASE + OBIS_ACTIVE_PWR_N {
                found |= 1 << 1;
                active_l1_n = obis.value as f64;
            } else if obis.id == OBIS_L1_BASE + OBIS_REACTIVE_PWR_P {
                found |= 1 << 2;
                reactive_l1_p = obis.value as f64;
            } else if obis.id == OBIS_L1_BASE + OBIS_REACTIVE_PWR_N {
                found |= 1 << 3;
                reactive_l1_n = obis.value as f64;
            } else if obis.id == OBIS_L1_BASE + OBIS_VOLTAGE {
                found |= 1 << 4;
                voltage_l1 = obis.value as f64;
            } else if obis.id == OBIS_L2_BASE + OBIS_ACTIVE_PWR_P {
                found |= 1 << 5;
                active_l2_p = obis.value as f64;
            } else if obis.id == OBIS_L2_BASE + OBIS_ACTIVE_PWR_N {
                found |= 1 << 6;
                active_l2_n = obis.value as f64;
            } else if obis.id == OBIS_L2_BASE + OBIS_REACTIVE_PWR_P {
                found |= 1 << 7;
                reactive_l2_p = obis.value as f64;
            } else if obis.id == OBIS_L2_BASE + OBIS_REACTIVE_PWR_N {
                found |= 1 << 8;
                reactive_l2_n = obis.value as f64;
            } else if obis.id == OBIS_L2_BASE + OBIS_VOLTAGE {
                found |= 1 << 9;
                voltage_l2 = obis.value as f64;
            } else if obis.id == OBIS_L3_BASE + OBIS_ACTIVE_PWR_P {
                found |= 1 << 10;
                active_l3_p = obis.value as f64;
            } else if obis.id == OBIS_L3_BASE + OBIS_ACTIVE_PWR_N {
                found |= 1 << 11;
                active_l3_n = obis.value as f64;
            } else if obis.id == OBIS_L3_BASE + OBIS_REACTIVE_PWR_P {
                found |= 1 << 12;
                reactive_l3_p = obis.value as f64;
            } else if obis.id == OBIS_L3_BASE + OBIS_REACTIVE_PWR_N {
                found |= 1 << 13;
                reactive_l3_n = obis.value as f64;
            } else if obis.id == OBIS_L3_BASE + OBIS_VOLTAGE {
                found |= 1 << 14;
                voltage_l3 = obis.value as f64;
            } else if obis.id == OBIS_VERSION {
                found |= 1 << 15;
                version = obis.value as u32;
            }
        }

        if found != 0xFFFF {
            return Err(format!("Missing OBIS records: bitfield {found}"));
        }

        Ok(Self {
            active_pwr: [
                active_l1_p - active_l1_n,
                active_l2_p - active_l2_n,
                active_l3_p - active_l3_n,
            ],
            reactive_pwr: [
                reactive_l1_p - reactive_l1_n,
                reactive_l2_p - reactive_l2_n,
                reactive_l3_p - reactive_l3_n,
            ],
            voltage: [voltage_l1, voltage_l2, voltage_l3],
            version,
        })
    }
}

#[derive(Debug, PartialEq)]
struct EmDependentPayload {
    active_sum: f64,
    reactive_sum: f64,
    apparent_sum: f64,
    power_factor_sum: f64,

    active_pwr: [f64; 3],
    reactive_pwr: [f64; 3],
    apparent_pwr: [f64; 3],
    voltage: [f64; 3],
    current: [f64; 3],
    power_factor: [f64; 3],
    version: u32,
}

impl EmDependentPayload {
    fn from_independent(data: EmIndependentPayload) -> Self {
        let active_sum = data.active_pwr.iter().sum();
        let reactive_sum = data.reactive_pwr.iter().sum();
        let apparent_sum = Self::calc_apparent(active_sum, reactive_sum);

        let apparent_pwr = [
            Self::calc_apparent(data.active_pwr[0], data.reactive_pwr[0]),
            Self::calc_apparent(data.active_pwr[1], data.reactive_pwr[1]),
            Self::calc_apparent(data.active_pwr[2], data.reactive_pwr[2]),
        ];

        let current = [
            Self::calc_abs_ratio(apparent_pwr[0], data.voltage[0]) * 100000.0,
            Self::calc_abs_ratio(apparent_pwr[1], data.voltage[1]) * 100000.0,
            Self::calc_abs_ratio(apparent_pwr[2], data.voltage[2]) * 100000.0,
        ];

        let power_factor = [
            Self::calc_abs_ratio(data.active_pwr[0], apparent_pwr[0]) * 1000.0,
            Self::calc_abs_ratio(data.active_pwr[1], apparent_pwr[1]) * 1000.0,
            Self::calc_abs_ratio(data.active_pwr[2], apparent_pwr[2]) * 1000.0,
        ];

        Self {
            active_sum,
            reactive_sum,
            apparent_sum,
            power_factor_sum: (active_sum / apparent_sum).abs() * 1000.0,
            active_pwr: data.active_pwr,
            reactive_pwr: data.reactive_pwr,
            apparent_pwr,
            voltage: data.voltage,
            current,
            power_factor,
            version: data.version,
        }
    }

    fn calc_apparent(active: f64, reactive: f64) -> f64 {
        (active * active + reactive * reactive).sqrt() * active.signum()
    }

    fn calc_abs_ratio(x: f64, y: f64) -> f64 {
        (x / y).abs()
    }

    fn signed_to_obis(
        obis_p: u32,
        obis_n: u32,
        val: f64,
        data: &mut Vec<ObisValue>,
    ) {
        if val > 0.0 {
            data.push(ObisValue {
                id: obis_p,
                value: val as u64,
            });
            data.push(ObisValue {
                id: obis_n,
                value: 0,
            });
        } else {
            data.push(ObisValue {
                id: obis_p,
                value: 0,
            });
            data.push(ObisValue {
                id: obis_n,
                value: (-val) as u64,
            });
        }
    }
}

impl From<EmDependentPayload> for Vec<ObisValue> {
    fn from(data: EmDependentPayload) -> Self {
        let mut result = Vec::with_capacity(36);

        EmDependentPayload::signed_to_obis(
            OBIS_SUM_BASE + OBIS_ACTIVE_PWR_P,
            OBIS_SUM_BASE + OBIS_ACTIVE_PWR_N,
            data.active_sum,
            &mut result,
        );
        EmDependentPayload::signed_to_obis(
            OBIS_SUM_BASE + OBIS_REACTIVE_PWR_P,
            OBIS_SUM_BASE + OBIS_REACTIVE_PWR_N,
            data.reactive_sum,
            &mut result,
        );
        EmDependentPayload::signed_to_obis(
            OBIS_SUM_BASE + OBIS_APPARENT_PWR_P,
            OBIS_SUM_BASE + OBIS_APPARENT_PWR_N,
            data.apparent_sum,
            &mut result,
        );
        result.push(ObisValue {
            id: OBIS_SUM_BASE + OBIS_POWER_FACTOR,
            value: data.power_factor_sum as u64,
        });

        for (obis_base, i) in
            [(OBIS_L1_BASE, 0), (OBIS_L2_BASE, 1), (OBIS_L3_BASE, 2)]
        {
            EmDependentPayload::signed_to_obis(
                obis_base + OBIS_ACTIVE_PWR_P,
                obis_base + OBIS_ACTIVE_PWR_N,
                data.active_pwr[i],
                &mut result,
            );
            EmDependentPayload::signed_to_obis(
                obis_base + OBIS_REACTIVE_PWR_P,
                obis_base + OBIS_REACTIVE_PWR_N,
                data.reactive_pwr[i],
                &mut result,
            );
            EmDependentPayload::signed_to_obis(
                obis_base + OBIS_APPARENT_PWR_P,
                obis_base + OBIS_APPARENT_PWR_N,
                data.apparent_pwr[i],
                &mut result,
            );
            result.push(ObisValue {
                id: obis_base + OBIS_CURRENT,
                value: data.current[i] as u64,
            });
            result.push(ObisValue {
                id: obis_base + OBIS_VOLTAGE,
                value: data.voltage[i] as u64,
            });
            result.push(ObisValue {
                id: obis_base + OBIS_POWER_FACTOR,
                value: data.power_factor[i] as u64,
            });
        }

        result.push(ObisValue {
            id: OBIS_VERSION,
            value: data.version as u64,
        });

        result
    }
}

#[cfg(test)]
fn tuple_slice_to_obis_vec(slice: &[(u32, u64)]) -> Vec<ObisValue> {
    let mut result = Vec::with_capacity(slice.len());
    for (id, value) in slice {
        result.push(ObisValue {
            id: *id,
            value: *value,
        });
    }

    result
}

#[test]
fn test_obis_serialize() {
    let data = EmDependentPayload {
        active_sum: 127.0,
        reactive_sum: -1541.0,
        apparent_sum: 1547.0,
        power_factor_sum: 82.0,

        active_pwr: [28918.0, -9156.0, -19635.0],
        reactive_pwr: [-891.0, -457.0, -193.0],
        apparent_pwr: [28932.0, -9168.0, -19636.0],
        voltage: [233681.0, 235194.0, 237537.0],
        current: [12393.0, 3952.0, 8281.0],
        power_factor: [1000.0, 999.0, 1000.0],
        version: 0x02001252,
    };

    let expected = tuple_slice_to_obis_vec(&[
        (0x00010400, 127),
        (0x00020400, 0),
        (0x00030400, 0),
        (0x00040400, 1541),
        (0x00090400, 1547),
        (0x000a0400, 0),
        (0x000d0400, 82),
        (0x00150400, 28918),
        (0x00160400, 0),
        (0x00170400, 0),
        (0x00180400, 891),
        (0x001d0400, 28932),
        (0x001e0400, 0),
        (0x001f0400, 12393),
        (0x00200400, 233681),
        (0x00210400, 1000),
        (0x00290400, 0),
        (0x002a0400, 9156),
        (0x002b0400, 0),
        (0x002c0400, 457),
        (0x00310400, 0),
        (0x00320400, 9168),
        (0x00330400, 3952),
        (0x00340400, 235194),
        (0x00350400, 999),
        (0x003d0400, 0),
        (0x003e0400, 19635),
        (0x003f0400, 0),
        (0x00400400, 193),
        (0x00450400, 0),
        (0x00460400, 19636),
        (0x00470400, 8281),
        (0x00480400, 237537),
        (0x00490400, 1000),
        (0x90000000, 0x02001252),
    ]);

    assert_eq!(
        expected,
        <EmDependentPayload as Into<Vec<ObisValue>>>::into(data)
    );
}

#[test]
fn test_obis_deserialize() {
    let data = tuple_slice_to_obis_vec(&[
        (OBIS_SUM_BASE + OBIS_ACTIVE_PWR_P, 127),
        (OBIS_SUM_BASE + OBIS_ACTIVE_PWR_N, 0),
        (OBIS_SUM_BASE + OBIS_REACTIVE_PWR_P, 0),
        (OBIS_SUM_BASE + OBIS_REACTIVE_PWR_N, 1541),
        (OBIS_SUM_BASE + OBIS_APPARENT_PWR_P, 1547),
        (OBIS_SUM_BASE + OBIS_APPARENT_PWR_N, 0),
        (OBIS_SUM_BASE + OBIS_POWER_FACTOR, 82),
        (OBIS_L1_BASE + OBIS_ACTIVE_PWR_P, 28918),
        (OBIS_L1_BASE + OBIS_ACTIVE_PWR_N, 0),
        (OBIS_L1_BASE + OBIS_REACTIVE_PWR_P, 0),
        (OBIS_L1_BASE + OBIS_REACTIVE_PWR_N, 891),
        (OBIS_L1_BASE + OBIS_APPARENT_PWR_P, 28932),
        (OBIS_L1_BASE + OBIS_APPARENT_PWR_N, 0),
        (OBIS_L1_BASE + OBIS_CURRENT, 12393),
        (OBIS_L1_BASE + OBIS_VOLTAGE, 233681),
        (OBIS_L1_BASE + OBIS_POWER_FACTOR, 1000),
        (OBIS_L2_BASE + OBIS_ACTIVE_PWR_P, 0),
        (OBIS_L2_BASE + OBIS_ACTIVE_PWR_N, 9156),
        (OBIS_L2_BASE + OBIS_REACTIVE_PWR_P, 0),
        (OBIS_L2_BASE + OBIS_REACTIVE_PWR_N, 457),
        (OBIS_L2_BASE + OBIS_APPARENT_PWR_P, 0),
        (OBIS_L2_BASE + OBIS_APPARENT_PWR_N, 9168),
        (OBIS_L2_BASE + OBIS_CURRENT, 3952),
        (OBIS_L2_BASE + OBIS_VOLTAGE, 235194),
        (OBIS_L2_BASE + OBIS_POWER_FACTOR, 999),
        (OBIS_L3_BASE + OBIS_ACTIVE_PWR_P, 0),
        (OBIS_L3_BASE + OBIS_ACTIVE_PWR_N, 19635),
        (OBIS_L3_BASE + OBIS_REACTIVE_PWR_P, 0),
        (OBIS_L3_BASE + OBIS_REACTIVE_PWR_N, 193),
        (OBIS_L3_BASE + OBIS_APPARENT_PWR_P, 0),
        (OBIS_L3_BASE + OBIS_APPARENT_PWR_N, 19636),
        (OBIS_L3_BASE + OBIS_CURRENT, 8281),
        (OBIS_L3_BASE + OBIS_VOLTAGE, 237537),
        (OBIS_L3_BASE + OBIS_POWER_FACTOR, 1000),
        (OBIS_VERSION, 0x02001252),
    ]);

    let expected = EmIndependentPayload {
        active_pwr: [28918.0, -9156.0, -19635.0],
        reactive_pwr: [-891.0, -457.0, -193.0],
        voltage: [233681.0, 235194.0, 237537.0],
        version: 0x02001252,
    };

    assert_eq!(Ok(expected), data.try_into());
}

#[test]
fn test_apply_offset() {
    let mut data = EmIndependentPayload {
        active_pwr: [28918.0, -9156.0, -19635.0],
        reactive_pwr: [-891.0, -457.0, -193.0],
        voltage: [233681.0, 235194.0, 237537.0],
        version: 0x02001252,
    };
    data.apply_power_offset(-1000.0);

    let expected = EmIndependentPayload {
        active_pwr: [
            25584.666666666668,
            -12489.333333333332,
            -22968.333333333332,
        ],
        reactive_pwr: [-891.0, -457.0, -193.0],
        voltage: [233681.0, 235194.0, 237537.0],
        version: 0x02001252,
    };

    assert_eq!(expected, data);
}

#[test]
fn test_calc_dependent() {
    let independent = EmIndependentPayload {
        active_pwr: [28918.0, -9156.0, -19635.0],
        reactive_pwr: [-891.0, -457.0, -193.0],
        voltage: [233681.0, 235194.0, 237537.0],
        version: 0x02001252,
    };

    let dependent = independent.into_dependent();

    let expected = EmDependentPayload {
        active_sum: 127.0,
        reactive_sum: -1541.0,
        apparent_sum: 1546.2244339034357,
        power_factor_sum: 82.13555368504244,

        active_pwr: [28918.0, -9156.0, -19635.0],
        reactive_pwr: [-891.0, -457.0, -193.0],
        apparent_pwr: [
            28931.723159881094,
            -9167.397940528163,
            -19635.948512867923,
        ],
        voltage: [233681.0, 235194.0, 237537.0],
        current: [12380.862440626792, 3897.8026397476815, 8266.479964328893],
        power_factor: [999.5256708421667, 998.7566874916847, 999.9516950827559],
        version: 0x02001252,
    };

    assert_eq!(expected, dependent);
}
