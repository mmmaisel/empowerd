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
use chrono::{DateTime, Utc};

pub struct PT1 {
    tau: f64,
    value: f64,
    min: f64,
    max: f64,
    last_time: DateTime<Utc>,
}

impl PT1 {
    pub fn new(
        tau: f64,
        value: f64,
        min: f64,
        max: f64,
        last_time: DateTime<Utc>,
    ) -> Self {
        Self {
            tau,
            value,
            min,
            max,
            last_time,
        }
    }

    pub fn process(&mut self, value: f64, time: DateTime<Utc>) -> f64 {
        let delta_t =
            (time - self.last_time).num_milliseconds() as f64 / 1000.0;
        self.last_time = time;
        self.value = (self.value + delta_t / self.tau * (value - self.value))
            .clamp(self.min, self.max);
        self.value
    }
}

#[cfg(test)]
use chrono::{LocalResult, TimeZone};

#[test]
fn test_pt1() {
    let timestamp = match Utc.timestamp_opt(0, 0) {
        LocalResult::Single(x) => x,
        _ => panic!("Failed to construct timestamp!"),
    };
    let mut filter = PT1::new(2.0, 0.0, 0.0, 10000.0, timestamp);

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
        let timestamp = match Utc.timestamp_opt(i as i64, 0) {
            LocalResult::Single(x) => x,
            _ => panic!("Failed to construct timestamp!"),
        };
        let output = filter.process(input[i], timestamp);
        assert!(
            (expected_output[i] - output).abs() < 0.0001,
            "expected {} to be {}",
            output,
            expected_output[i]
        );
    }
}
