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
use crate::models::units::{ratio, Time};

pub struct PT1<T> {
    tau: Time,
    value: T,
    min: T,
    max: T,
    last_time: Time,
}

impl<T> PT1<T>
where
    T: Copy
        + std::cmp::PartialOrd
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<f64, Output = T>,
{
    pub fn new(tau: Time, value: T, min: T, max: T, last_time: Time) -> Self {
        Self {
            tau,
            value,
            min,
            max,
            last_time,
        }
    }

    pub fn process(&mut self, value: T, time: Time) -> T {
        let delta_t = time - self.last_time;
        self.last_time = time;
        self.value = self.value
            + (value - self.value) * (delta_t / self.tau).get::<ratio>();

        if self.value > self.max {
            self.value = self.max;
        } else if self.value < self.min {
            self.value = self.min;
        }

        self.value
    }
}

#[cfg(test)]
use crate::models::units::second;

#[test]
fn test_pt1() {
    let mut filter = PT1::new(
        Time::new::<second>(2.0),
        0.0,
        0.0,
        10000.0,
        Time::new::<second>(0.0),
    );

    let input = vec![
        0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ];
    let expected_output = vec![
        0.0, 0.0, 0.0, 0.5, 0.75, 0.875, 0.9375, 0.9688, 0.9844, 0.9922,
        0.9961, 0.9980, 0.9990, 0.4995, 0.2498, 0.1249, 0.0624, 0.0312, 0.0156,
        0.0078, 0.0039, 0.0020, 0.0010,
    ];

    for i in 0..input.len() {
        let output = filter.process(input[i], Time::new::<second>(i as f64));
        assert!(
            (expected_output[i] - output).abs() < 0.0001,
            "expected {} to be {}",
            output,
            expected_output[i]
        );
    }
}
