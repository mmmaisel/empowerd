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
struct Setpoint<T, U> {
    pub input_range: [T; 2],
    pub output: U,
}

pub struct MultiSetpointHysteresis<T, U> {
    setpoints: Vec<Setpoint<T, U>>,
    last_index: usize,
}

pub struct LinspaceBuilder<T, U> {
    in_start: T,
    in_stop: T,
    out_low: U,
    out_start: U,
    out_stop: U,
    out_high: U,
    num_points: i32,
    hysteresis: T,

    min: T,
    max: T,
}

impl<T, U> LinspaceBuilder<T, U>
where
    T: Default
        + Copy
        + std::cmp::PartialOrd
        + std::ops::Add<T, Output = T>
        + std::ops::Sub<T, Output = T>
        + std::ops::Mul<f64, Output = T>
        + std::ops::Div<f64, Output = T>,
    U: Default
        + Copy
        + std::ops::Add<U, Output = U>
        + std::ops::Sub<U, Output = U>
        + std::ops::Mul<f64, Output = U>
        + std::ops::Div<f64, Output = U>,
{
    pub fn new(min: T, max: T) -> Self {
        Self {
            in_start: T::default(),
            in_stop: T::default(),
            out_low: U::default(),
            out_start: U::default(),
            out_stop: U::default(),
            out_high: U::default(),
            num_points: 0,
            hysteresis: T::default(),
            min,
            max,
        }
    }

    pub fn input_range(mut self, start: T, stop: T) -> Self {
        self.in_start = start;
        self.in_stop = stop;
        self
    }

    pub fn output_range(mut self, low: U, start: U, stop: U, high: U) -> Self {
        self.out_low = low;
        self.out_start = start;
        self.out_stop = stop;
        self.out_high = high;
        self
    }

    pub fn point_count(mut self, num: i32) -> Self {
        self.num_points = num;
        self
    }

    pub fn hysteresis(mut self, hysteresis: T) -> Self {
        self.hysteresis = hysteresis;
        self
    }

    pub fn build(self) -> Result<MultiSetpointHysteresis<T, U>, String> {
        if self.num_points < 2 {
            return Err("'num_points' must be greater than 1!".into());
        } else if self.hysteresis < T::default() {
            return Err("'hysteresis' must be positive!".into());
        } else if self.in_start >= self.in_stop {
            return Err("'in_start' must be smaller than 'in_stop'".into());
        }

        let spacing_in: T =
            (self.in_stop - self.in_start) / ((self.num_points - 2) as f64);
        let spacing_out: U =
            (self.out_stop - self.out_start) / ((self.num_points - 2) as f64);

        let setpoints: Vec<Setpoint<T, U>> = (0..self.num_points)
            .map(|i| {
                let (input_range, output) = if i == 0 {
                    ([self.min, self.in_start + self.hysteresis], self.out_low)
                } else if i == self.num_points - 1 {
                    ([self.in_stop - self.hysteresis, self.max], self.out_high)
                } else {
                    (
                        [
                            self.in_start + spacing_in * ((i - 1) as f64)
                                - self.hysteresis,
                            self.in_start
                                + spacing_in * (i as f64)
                                + self.hysteresis,
                        ],
                        self.out_start + spacing_out * (i as f64),
                    )
                };

                Setpoint {
                    input_range,
                    output,
                }
            })
            .collect();

        Ok(MultiSetpointHysteresis {
            setpoints,
            last_index: 0,
        })
    }
}

impl<T, U> MultiSetpointHysteresis<T, U>
where
    T: std::cmp::PartialOrd,
    U: Copy,
{
    pub fn process(&mut self, value: T) -> U {
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
    if let Ok(_) = LinspaceBuilder::new(-f64::MAX, f64::MAX)
        .input_range(20000.0, 10000.0)
        .output_range(0.0, 0.0, 0.0, 0.0)
        .point_count(5)
        .hysteresis(0.1)
        .build()
    {
        panic!("Start greater than stop was not rejected!");
    }

    if let Ok(_) = LinspaceBuilder::new(-f64::MAX, f64::MAX)
        .input_range(10000.0, 20000.0)
        .output_range(0.0, 0.0, 0.0, 0.0)
        .point_count(1)
        .hysteresis(0.1)
        .build()
    {
        panic!("Single point range was not rejected!");
    }

    if let Ok(_) = LinspaceBuilder::new(-f64::MAX, f64::MAX)
        .input_range(10000.0, 20000.0)
        .output_range(0.0, 0.0, 0.0, 0.0)
        .point_count(2)
        .hysteresis(-123.0)
        .build()
    {
        panic!("Invalid hysteresis was not rejected!");
    }
}

#[test]
fn test_linspace_init() {
    let ctrl = match LinspaceBuilder::new(-f64::MAX, f64::MAX)
        .input_range(10000.0, 20000.0)
        .output_range(250.0, 250.0, 100.0, 0.0)
        .point_count(2)
        .hysteresis(1000.0)
        .build()
    {
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

    let ctrl = match LinspaceBuilder::new(-f64::MAX, f64::MAX)
        .input_range(10000.0, 20000.0)
        .output_range(250.0, 250.0, 100.0, 0.0)
        .point_count(5)
        .hysteresis(1000.0)
        .build()
    {
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
    let mut ctrl = match LinspaceBuilder::new(-f64::MAX, f64::MAX)
        .input_range(10000.0, 20000.0)
        .output_range(250.0, 250.0, 100.0, 0.0)
        .point_count(5)
        .hysteresis(1000.0)
        .build()
    {
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
