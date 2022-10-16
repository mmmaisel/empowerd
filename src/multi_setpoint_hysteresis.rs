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

#[derive(Debug, PartialEq)]
struct Setpoint {
    pub input_range: [f64; 2],
    pub output: f64,
}

pub struct MultiSetpointHysteresis {
    setpoints: Vec<Setpoint>,
    last_index: usize,
}

impl MultiSetpointHysteresis {
    pub fn new_linspace(
        in_start: f64,
        in_stop: f64,
        out_0: f64,
        out_start: f64,
        out_stop: f64,
        out_1: f64,
        num_points: i32,
        hysteresis: f64,
    ) -> Result<Self, String> {
        if num_points < 2 {
            return Err("'num_points' must be greater than 1!".into());
        } else if hysteresis < 0.0 {
            return Err("'hysteresis' must be positive!".into());
        } else if in_start >= in_stop {
            return Err("'in_start' must be smaller than 'in_stop'".into());
        }

        let spacing_in = (in_stop - in_start) / (num_points - 2) as f64;
        let spacing_out = (out_stop - out_start) / (num_points - 2) as f64;

        let setpoints: Vec<Setpoint> = (0..num_points)
            .map(|i| {
                let (input_range, output) = if i == 0 {
                    ([-f64::MAX, in_start + hysteresis], out_0)
                } else if i == num_points - 1 {
                    ([in_stop - hysteresis, f64::MAX], out_1)
                } else {
                    (
                        [
                            in_start + ((i - 1) as f64) * spacing_in
                                - hysteresis,
                            in_start + (i as f64) * spacing_in + hysteresis,
                        ],
                        out_start + (i as f64) * spacing_out,
                    )
                };

                Setpoint {
                    input_range,
                    output,
                }
            })
            .collect();

        Ok(Self {
            setpoints,
            last_index: 0,
        })
    }

    pub fn process(&mut self, value: f64) -> f64 {
        let last_point = &self.setpoints[self.last_index];
        if value > last_point.input_range[0]
            && value < last_point.input_range[1]
        {
            return last_point.output;
        }

        let new_idx = self
            .setpoints
            .iter()
            .enumerate()
            .find_map(|(i, point)| {
                if value > point.input_range[0] && value < point.input_range[1]
                {
                    Some(i)
                } else {
                    None
                }
            })
            .unwrap();

        self.last_index = new_idx;
        self.setpoints[self.last_index].output
    }
}

#[test]
fn test_parameter_validation() {
    if let Ok(_) = MultiSetpointHysteresis::new_linspace(
        20000.0, 10000.0, 0.0, 0.0, 0.0, 0.0, 5, 0.1,
    ) {
        panic!("Start greater than stop was not rejected!");
    }

    if let Ok(_) = MultiSetpointHysteresis::new_linspace(
        10000.0, 20000.0, 0.0, 0.0, 0.0, 0.0, 1, 0.1,
    ) {
        panic!("Single point range was not rejected!");
    }

    if let Ok(_) = MultiSetpointHysteresis::new_linspace(
        10000.0, 20000.0, 0.0, 0.0, 0.0, 0.0, 2, -123.0,
    ) {
        panic!("Invalid hysteresis was not rejected!");
    }
}

#[test]
fn test_linspace_init() {
    let ctrl = match MultiSetpointHysteresis::new_linspace(
        10000.0, 20000.0, 250.0, 250.0, 100.0, 0.0, 2, 1000.0,
    ) {
        Ok(x) => x,
        Err(e) => {
            panic!("Creating MultiSetpointHysteresis object failed: {}", e)
        }
    };

    let expected = vec![
        Setpoint {
            input_range: [-f64::MAX, 11000.0],
            output: 250.0,
        },
        Setpoint {
            input_range: [19000.0, f64::MAX],
            output: 0.0,
        },
    ];
    assert_eq!(expected, ctrl.setpoints);

    let ctrl = match MultiSetpointHysteresis::new_linspace(
        10000.0, 20000.0, 250.0, 250.0, 100.0, 0.0, 5, 1000.0,
    ) {
        Ok(x) => x,
        Err(e) => {
            panic!("Creating MultiSetpointHysteresis object failed: {}", e)
        }
    };

    let expected = vec![
        Setpoint {
            input_range: [-f64::MAX, 11000.0],
            output: 250.0,
        },
        Setpoint {
            input_range: [9000.0, 14333.333333333334],
            output: 200.0,
        },
        Setpoint {
            input_range: [12333.333333333334, 17666.666666666668],
            output: 150.0,
        },
        Setpoint {
            input_range: [15666.666666666668, 21000.0],
            output: 100.0,
        },
        Setpoint {
            input_range: [19000.0, f64::MAX],
            output: 0.0,
        },
    ];
    assert_eq!(expected, ctrl.setpoints);
}

#[test]
fn test_processing() {
    let mut ctrl = match MultiSetpointHysteresis::new_linspace(
        10000.0, 20000.0, 250.0, 250.0, 100.0, 0.0, 5, 1000.0,
    ) {
        Ok(x) => x,
        Err(e) => {
            panic!("Creating MultiSetpointHysteresis object failed: {}", e)
        }
    };

    assert_eq!(0.0, ctrl.process(40001.0));
    assert_eq!(0.0, ctrl.process(20001.0));
    assert_eq!(0.0, ctrl.process(20000.0));
    assert_eq!(100.0, ctrl.process(18500.0));
    assert_eq!(100.0, ctrl.process(20000.0));
    assert_eq!(100.0, ctrl.process(17500.0));
    assert_eq!(150.0, ctrl.process(15000.0));
    assert_eq!(250.0, ctrl.process(5000.0));
    assert_eq!(250.0, ctrl.process(-15000.0));
    assert_eq!(250.0, ctrl.process(10000.0));
    assert_eq!(200.0, ctrl.process(14000.0));
    assert_eq!(200.0, ctrl.process(12500.0));
    assert_eq!(150.0, ctrl.process(15000.0));
    assert_eq!(0.0, ctrl.process(25000.0));
}
