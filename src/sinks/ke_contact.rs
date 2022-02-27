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
use crate::misc::parse_socketaddr_with_default;
use kecontact_client::KeContactClient;
use slog::Logger;

pub struct KeContactSink {
    name: String,
    client: KeContactClient,
}

impl KeContactSink {
    pub fn new(
        name: String,
        address: String,
        logger: Logger,
    ) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 7090)?;
        let client = KeContactClient::new(address, Some(logger.clone()));

        Ok(Self { name, client })
    }

    pub async fn set_max_current(&self, current: u16) -> Result<(), String> {
        self.client.set_max_current(current).await.map_err(|e| {
            format!("Setting max corrent for {} failed: {}", self.name, e)
        })
    }

    pub async fn set_enable(&self, enabled: bool) -> Result<(), String> {
        self.client.set_enable(enabled).await.map_err(|e| {
            format!("Setting max corrent for {} failed: {}", self.name, e)
        })
    }
}
