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
use crate::misc::parse_socketaddr_with_default;
use lambda_client::LambdaClient;

pub struct LambdaHeatPumpSink {
    name: String,
    client: LambdaClient,
}

impl LambdaHeatPumpSink {
    pub fn new(name: String, address: String) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 502)?;
        let client = LambdaClient::new(address);

        Ok(Self { name, client })
    }

    pub async fn set_available_power(&self, power: u16) -> Result<(), String> {
        let power = power
            .try_into()
            .map_err(|e| format!("Could not convert power to u16: {}", e))?;
        let mut context = self.client.open().await?;
        context.set_available_power(power).await.map_err(|e| {
            format!("Setting available power for {} failed: {}", self.name, e)
        })
    }
}
