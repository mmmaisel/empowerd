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
/// Reads the grid mode.
pub struct LoadControl {
    /// If the grid charge mode is enabled.
    pub charge_mode: bool,
    /// Displayed name.
    pub name: String,
}

impl LoadControl {
    pub fn new(name: String) -> Self {
        Self {
            charge_mode: false,
            name,
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
/// Controls the grid mode.
pub struct InputLoadControl {
    /// If the grid charge mode is enabled.
    pub charge_mode: bool,
}
