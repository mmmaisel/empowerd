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

use crate::tri_state::TriState;

#[derive(juniper::GraphQLObject)]
/// Reads an appliance.
pub struct Appliance {
    /// References the appliance.
    pub id: i32,
    /// If the appliance is forced on/off or in automatic mode.
    pub force_on_off: TriState,
    /// Name of the appliance.
    pub name: String,
}

impl Appliance {
    pub fn new(id: i32, name: String) -> Self {
        Self {
            id,
            force_on_off: TriState::Auto,
            name,
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
/// Controls an appliance.
pub struct InputAppliance {
    /// References the appliance.
    pub id: i32,
    /// If the appliance is forced on/off or in automatic mode.
    pub force_on_off: TriState,
}
