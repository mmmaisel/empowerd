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
#[derive(juniper::GraphQLObject)]
/// Reads an available power controller.
pub struct AvailablePower {
    /// References the channel.
    pub id: i32,
    /// Current battery charge threshold for enable.
    pub threshold: f64,
    /// Currently available power.
    pub power: f64,
    /// Name of the channel.
    pub name: String,
}

impl AvailablePower {
    pub fn new(id: i32, name: String) -> Self {
        Self {
            id,
            threshold: 0.0,
            power: 0.0,
            name,
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
/// Controls an available power controller.
pub struct InputAvailablePower {
    /// References the channel.
    pub id: i32,
    /// Current battery charge threshold for enable.
    pub threshold: f64,
}
