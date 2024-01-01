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
use super::SourceBase;
use crate::{
    models::{
        units::{joule, second, watt, Energy, Power, Time},
        BidirMeter,
    },
    task_group::TaskResult,
    Error,
};
use slog::{trace, Logger};
use sma_client::SmaClient;
use std::net::{Ipv4Addr, SocketAddr};

pub struct SmaMeterSource {
    base: SourceBase,
    sma_client: SmaClient,
    sma_addr: SocketAddr,
    bind_addr: Ipv4Addr,
}

impl SmaMeterSource {
    pub fn new(
        base: SourceBase,
        sma_addr: String,
        bind_addr: String,
    ) -> Result<Self, String> {
        let sma_socket_addr: SocketAddr = SmaClient::sma_sock_addr(sma_addr)
            .map_err(|e| format!("Could not parse sma_addr: {e}"))?;

        let bind_addr = bind_addr
            .parse::<Ipv4Addr>()
            .map_err(|e| format!("Could not parse bind_addr: {e}"))?;

        let logger = base.logger.clone();
        Ok(Self {
            base,
            sma_client: SmaClient::new(Some(logger)),
            sma_addr: sma_socket_addr,
            bind_addr,
        })
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let mut conn = self.base.get_database().await?;

        let session = self
            .sma_client
            .open(self.sma_addr, Some(self.bind_addr))
            .map_err(|e| {
                Error::Temporary(format!(
                    "Could not open SMA Client multicast session: {e}",
                ))
            })?;

        let (_header, data) =
            self.sma_client.fetch_em_data(&session).await.map_err(|e| {
                Error::Temporary(format!("Fetching EM data failed: {e}"))
            })?;

        let consumed = match data.get(&0x00010800) {
            Some(x) => Energy::new::<joule>(*x as f64),
            None => {
                return Err(Error::Temporary(
                    "Received data did not include consumed energy record."
                        .into(),
                ))
            }
        };

        let produced = match data.get(&0x00020800) {
            Some(x) => Energy::new::<joule>(*x as f64),
            None => {
                return Err(Error::Temporary(
                    "Received data did not include produced energy record."
                        .into(),
                ))
            }
        };

        let mut record = BidirMeter {
            time: Time::new::<second>(timing.now as f64),
            energy_in: consumed,
            energy_out: produced,
            power: Power::new::<watt>(0.0),
        };
        record.power =
            match BidirMeter::last(&mut conn, self.base.series_id).await {
                Ok(last_record) => {
                    trace!(
                        self.base.logger,
                        "Read {:?} from database",
                        last_record
                    );
                    record.calc_power(&last_record)
                }
                Err(Error::NotFound) => Power::new::<watt>(0.0),
                Err(e) => {
                    return Err(Error::Temporary(format!(
                        "Query {} database failed: {}",
                        &self.base.name, e,
                    )))
                }
            };

        self.base.notify_processors(&record);
        record.insert(&mut conn, self.base.series_id).await?;

        Ok(())
    }
}
