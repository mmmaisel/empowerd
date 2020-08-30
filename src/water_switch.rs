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
            if let Err(e) = pin.set_direction(Direction::In) {
                panic!("Failed to uninitialize pin: {}", e)
            }
            if let Err(e) = pin.unexport() {
                panic!("Failed to uninitialize pin: {}", e)
            }
        }
    }
}

impl WaterSwitch {
    pub fn new(pin_nums: Vec<i64>) -> Result<WaterSwitch, String> {
        let pins = pin_nums
            .into_iter()
            .map(|pin_num| {
                let pin_num = TryInto::<u64>::try_into(pin_num)
                    .map_err(|e| e.to_string())?;
                let pin = Pin::new(pin_num);
                pin.export().map_err(|e| e.to_string())?;
                thread::sleep(time::Duration::from_millis(100));
                pin.set_direction(Direction::Out)
                    .map_err(|e| e.to_string())?;
                pin.set_value(1).map_err(|e| e.to_string())?;
                return Ok(pin);
            })
            .collect::<Result<Vec<Pin>, String>>()?;

        return Ok(WaterSwitch { pins: pins });
    }

    pub fn set_open(&self, channel: usize, open: bool) -> Result<(), String> {
        match self.pins.get(channel) {
            Some(pin) => match pin.set_value(if open { 0 } else { 1 }) {
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
                    Ok(x) => Ok(x == 0),
                    Err(e) => Err(e.to_string()),
                };
            })
            .collect();
    }
}
