use std::convert::TryInto;
use std::thread;
use std::time;
use sysfs_gpio::{Direction, Pin};

pub struct WaterSwitch {
    pins: Vec<Pin>,
}

impl Drop for WaterSwitch {
    fn drop(&mut self) {
        for pin in &self.pins {
            pin.set_direction(Direction::In).unwrap();
            pin.unexport().unwrap();
        }
    }
}

impl WaterSwitch {
    pub fn new(pin_nums: Vec<i64>) -> WaterSwitch {
        let pins = pin_nums
            .into_iter()
            .map(|pin_num| {
                // TODO: dont unwrap use nice errors
                let pin = Pin::new(pin_num.try_into().unwrap());
                pin.export().unwrap();
                thread::sleep(time::Duration::from_millis(100));
                pin.set_direction(Direction::Out).unwrap();
                return pin;
            })
            .collect();

        return WaterSwitch { pins: pins };
    }

    pub fn set_open(&self, channel: usize, open: bool) -> Result<(), String> {
        match self.pins.get(channel) {
            Some(pin) => match pin.set_value(if open { 1 } else { 0 }) {
                Ok(_) => return Ok(()),
                Err(e) => return Err(e.to_string()),
            },
            None => return Err("Valve index not found".into()),
        };
    }

    pub fn get_open(&self) -> Result<Vec<bool>, String> {
        return self
            .pins
            .iter()
            .map(|pin| {
                return match pin.get_value() {
                    Ok(x) => Ok(x != 0),
                    Err(e) => Err(e.to_string()),
                };
            })
            .collect();
    }
}
