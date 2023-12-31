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

pub mod units {
    pub use uom::fmt::DisplayStyle::Abbreviation;
    pub use uom::si::{
        angle::degree,
        energy::{joule, kilowatt_hour, watt_hour},
        f64::{
            Angle, Energy, Length, Power, Pressure, Ratio,
            ThermodynamicTemperature as Temperature, Time, Velocity,
        },
        length::{micrometer, millimeter},
        power::watt,
        pressure::{hectopascal, pascal},
        ratio::{percent, ratio},
        thermodynamic_temperature::degree_celsius as celsius,
        time::{millisecond, second},
        velocity::{meter_per_second, millimeter_per_second},
    };
}

pub use available_power::AvailablePower;
pub use postgres::{
    run_migrations, Battery, BidirMeter, Generator, Heatpump, SimpleMeter,
    Weather,
};

#[derive(Clone, Debug)]
pub enum Model {
    None,
    AvailablePower(AvailablePower),
    Battery(Battery),
    BidirMeter(BidirMeter),
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

impl From<BidirMeter> for Model {
    fn from(record: BidirMeter) -> Self {
        Model::BidirMeter(record)
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
        SimpleMeter {
            time: record.time,
            energy: record.energy,
            power: record.power,
        }
    }
}
