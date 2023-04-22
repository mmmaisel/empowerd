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
use chrono::{DateTime, Duration, Local, Utc};
use spa::{calc_sunrise_and_set, SunriseAndSet};

pub struct SeasonalBuilder {
    latitude: f64,
    longitude: f64,
    offset: f64,
    gain: f64,
    phase: f64,
}

impl SeasonalBuilder {
    pub fn new() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            offset: 0.0,
            gain: 1.0,
            phase: 0.0,
        }
    }

    pub fn build(self) -> Result<Seasonal, String> {
        if self.latitude < -90.0 || self.latitude > 90.0 {
            return Err("Invalid latitude".into());
        }
        if self.longitude < -180.0 || self.longitude > 180.0 {
            return Err("Invalid longitude".into());
        }

        Ok(Seasonal {
            latitude: self.latitude,
            longitude: self.longitude,
            offset: self.offset,
            gain: self.gain,
            phase: self.phase,
        })
    }

    pub fn latitude(mut self, latitude: f64) -> Self {
        self.latitude = latitude;
        self
    }

    pub fn longitude(mut self, longitude: f64) -> Self {
        self.longitude = longitude;
        self
    }

    pub fn offset_hour(mut self, offset: f64) -> Self {
        self.offset = offset * 3600.0;
        self
    }

    pub fn gain_per_hour(mut self, gain: f64) -> Self {
        self.gain = gain / 3600.0;
        self
    }

    pub fn phase_days(mut self, phase: f64) -> Self {
        self.phase = phase * 24.0 * 3600.0;
        self
    }
}

pub struct Seasonal {
    latitude: f64,
    longitude: f64,
    offset: f64,
    gain: f64,
    phase: f64,
}

impl Seasonal {
    pub fn calc_correction(&self, time: DateTime<Utc>) -> f64 {
        let time = time + Duration::seconds(self.phase as i64);

        let daytime =
            match calc_sunrise_and_set(time, self.latitude, self.longitude)
                .unwrap()
            {
                SunriseAndSet::Daylight(sunrise, sunset) => {
                    (sunset - sunrise).num_seconds()
                }
                SunriseAndSet::PolarDay => 24 * 3600,
                SunriseAndSet::PolarNight => 0,
            };
        let deviation = daytime - 12 * 3600;
        println!("deviation sec: {}", deviation);
        (deviation as f64 + self.offset) * self.gain
    }
    pub fn current_correction(&self) -> f64 {
        self.calc_correction(DateTime::<Utc>::from(Local::now()))
    }
}

#[test]
fn test_seasonal_calculation() {
    let seasonal = SeasonalBuilder::new().gain_per_hour(100.0).build().unwrap();
    let now = DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z").unwrap();
    let seasonal_correction = seasonal.calc_correction(now.into());
    assert_eq!(
        12, seasonal_correction as i32,
        "Correction at equator is not approximately zero"
    );

    let seasonal = SeasonalBuilder::new()
        .latitude(50.0)
        .longitude(10.0)
        .gain_per_hour(100.0)
        .phase_days(-35.0)
        .build()
        .unwrap();
    let now = DateTime::parse_from_rfc3339("2023-04-22T12:00:00Z").unwrap();
    let seasonal_correction = seasonal.calc_correction(now.into());
    assert_eq!(
        2, seasonal_correction as i32,
        "Correction at equinox is not approximately zero"
    );

    let seasonal = SeasonalBuilder::new()
        .latitude(50.0)
        .longitude(10.0)
        .gain_per_hour(100.0)
        .build()
        .unwrap();
    let now = DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z").unwrap();
    let seasonal_correction = seasonal.calc_correction(now.into());
    assert_eq!(
        -383, seasonal_correction as i32,
        "Correction in winter is incorrect"
    );

    let seasonal = SeasonalBuilder::new()
        .latitude(50.0)
        .longitude(10.0)
        .gain_per_hour(50.0)
        .build()
        .unwrap();
    let now = DateTime::parse_from_rfc3339("2023-06-22T12:00:00Z").unwrap();
    let seasonal_correction = seasonal.calc_correction(now.into());
    assert_eq!(
        218, seasonal_correction as i32,
        "Correction in summer is incorrect"
    );

    let seasonal = SeasonalBuilder::new()
        .latitude(50.0)
        .longitude(10.0)
        .gain_per_hour(100.0)
        .offset_hour(1.5)
        .phase_days(-35.0)
        .build()
        .unwrap();
    let now = DateTime::parse_from_rfc3339("2023-04-22T12:00:00Z").unwrap();
    let seasonal_correction = seasonal.calc_correction(now.into());
    assert_eq!(
        152, seasonal_correction as i32,
        "Offset is not applied correctly"
    );
}
