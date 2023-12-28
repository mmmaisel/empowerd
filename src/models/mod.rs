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
pub mod available_power;
pub mod influx;
pub mod postgres;

pub use available_power::AvailablePower;
pub use influx::*;

#[derive(Clone, Debug)]
pub enum Model {
    None,
    AvailablePower(AvailablePower),
    Battery(Battery),
    BidirectionalMeter(BidirectionalMeter),
    Generator(Generator),
    Heatpump(Heatpump),
    SimpleMeter(SimpleMeter),
    Weather(Weather),
}

// Conversions to Model

impl From<AvailablePower> for Model {
    fn from(record: AvailablePower) -> Self {
        Model::AvailablePower(record)
    }
}

impl From<Battery> for Model {
    fn from(record: Battery) -> Self {
        Model::Battery(record)
    }
}

impl From<BidirectionalMeter> for Model {
    fn from(record: BidirectionalMeter) -> Self {
        Model::BidirectionalMeter(record)
    }
}

impl From<Generator> for Model {
    fn from(record: Generator) -> Self {
        Model::Generator(record)
    }
}

impl From<Heatpump> for Model {
    fn from(record: Heatpump) -> Self {
        Model::Heatpump(record)
    }
}

impl From<SimpleMeter> for Model {
    fn from(record: SimpleMeter) -> Self {
        Model::SimpleMeter(record)
    }
}
impl From<Weather> for Model {
    fn from(record: Weather) -> Self {
        Model::Weather(record)
    }
}

// "Upcasting" to SimpleMeter

impl From<&Heatpump> for SimpleMeter {
    fn from(record: &Heatpump) -> Self {
        SimpleMeter::new(record.time, record.energy, record.power)
    }
}
