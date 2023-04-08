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
use crate::models::{
    units::{joule, second, watt, Energy, Power, Time},
    BidirectionalMeter, InfluxResult,
};
use crate::task_group::TaskResult;
use slog::{error, trace};
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
            .map_err(|e| format!("Could not parse sma_addr: {}", e))?;

        let bind_addr = bind_addr
            .parse::<Ipv4Addr>()
            .map_err(|e| format!("Could not parse bind_addr: {}", e))?;

        let logger = base.logger.clone();
        Ok(Self {
            base,
            sma_client: SmaClient::new(Some(logger)),
            sma_addr: sma_socket_addr,
            bind_addr,
        })
    }

    pub async fn run(&mut self) -> TaskResult {
        let (now, _oversample) = match self.base.sleep_aligned().await {
            Ok(x) => (Time::new::<second>(x.now as f64), x.oversample),
            Err(e) => return e,
        };

        let session =
            match self.sma_client.open(self.sma_addr, Some(self.bind_addr)) {
                Ok(x) => x,
                Err(e) => {
                    error!(
                        self.base.logger,
                        "Could not open SMA Client multicast session: {}", e
                    );
                    return TaskResult::Running;
                }
            };

        let (_header, data) =
            match self.sma_client.fetch_em_data(&session).await {
                Ok((x, y)) => (x, y),
                Err(e) => {
                    error!(self.base.logger, "Fetching EM data failed: {}", e,);
                    return TaskResult::Running;
                }
            };

        let consumed = match data.get(&0x00010800) {
            Some(x) => Energy::new::<joule>(*x as f64),
            None => {
                error!(
                    self.base.logger,
                    "Received data did not include consumed energy record.",
                );
                return TaskResult::Running;
            }
        };

        let produced = match data.get(&0x00020800) {
            Some(x) => Energy::new::<joule>(*x as f64),
            None => {
                error!(
                    self.base.logger,
                    "Received data did not include produced energy record.",
                );
                return TaskResult::Running;
            }
        };

        let power = match self.base.query_last::<BidirectionalMeter>().await {
            InfluxResult::Some(last_record) => {
                trace!(
                    self.base.logger,
                    "Read {:?} from database",
                    last_record
                );
                (consumed
                    - last_record.energy_consumed
                    - (produced - last_record.energy_produced))
                    / (now - last_record.time)
            }
            InfluxResult::None => Power::new::<watt>(0.0),
            InfluxResult::Err(e) => {
                error!(
                    self.base.logger,
                    "Query {} database failed: {}", &self.base.name, e
                );
                return TaskResult::Running;
            }
        };

        let record = BidirectionalMeter::new(now, consumed, produced, power);
        self.base.notify_processors(&record);
        let _: Result<(), ()> = self.base.save_record(record).await;
        TaskResult::Running
    }
}
