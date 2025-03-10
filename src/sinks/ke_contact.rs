/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2025 Max Maisel

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
use crate::models::units::{watt, Power};
use kecontact_client::KeContactClient;
use slog::{debug, Logger};

pub struct KeContactSink {
    name: String,
    client: KeContactClient,
    phases: f64,
    logger: Logger,
}

impl KeContactSink {
    pub fn new(
        name: String,
        address: String,
        phases: f64,
        logger: Logger,
    ) -> Result<Self, String> {
        let address = parse_socketaddr_with_default(&address, 7090)?;
        let client = KeContactClient::new(address, Some(logger.clone()));

        Ok(Self {
            name,
            client,
            phases,
            logger,
        })
    }

    pub async fn set_available_power(
        &self,
        charging_power: Power,
        current_power: Power,
    ) -> Result<bool, String> {
        if charging_power < Power::new::<watt>(6.0 * 230.0 * self.phases)
            && current_power < Power::new::<watt>(10.0)
            || charging_power < Power::new::<watt>(7.0 * 230.0 * self.phases)
                && current_power >= Power::new::<watt>(10.0)
        {
            debug!(self.logger, "Disable charging");
            if let Err(e) = self.client.set_enable(false).await {
                return Err(format!("Disabling {} failed: {}", self.name, e));
            }
            return Ok(false);
        } else {
            let charging_current =
                (charging_power.get::<watt>() / 230.0 / self.phases * 1000.0)
                    as u16;
            debug!(self.logger, "Set current to {} mA", charging_current);
            if let Err(e) = self.client.set_max_current(charging_current).await
            {
                return Err(format!(
                    "Setting max current for {} failed: {}",
                    self.name, e
                ));
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if let Err(e) = self.client.set_enable(true).await {
                return Err(format!("Enabling {} failed: {}", self.name, e));
            }
            return Ok(true);
        }
    }
}
