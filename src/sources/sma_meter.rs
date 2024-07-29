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
use sma_proto::{
    client::{SmaClient, SmaSession},
    SmaEndpoint,
};
use std::net::Ipv4Addr;
use tokio::time::{self, Duration};

pub struct SmaMeterSource {
    base: SourceBase,
    sma_client: SmaClient,
    meter_endpoint: SmaEndpoint,
    bind_addr: Ipv4Addr,
}

impl SmaMeterSource {
    pub fn new(
        base: SourceBase,
        bind_addr: Ipv4Addr,
        susy_id: u16,
        serial: u32,
    ) -> Result<Self, String> {
        Ok(Self {
            base,
            sma_client: SmaClient::new(SmaEndpoint::dummy()),
            meter_endpoint: SmaEndpoint { susy_id, serial },
            bind_addr,
        })
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let mut conn = self.base.get_database().await?;

        let session =
            SmaSession::open_multicast(self.bind_addr).map_err(|e| {
                Error::Temporary(format!(
                    "Could not open SMA Client multicast session: {e}",
                ))
            })?;

        let (_timestamp_ms, data) =
            time::timeout(Duration::from_millis(500), async {
                self.sma_client
                    .read_em_message(&session, &self.meter_endpoint)
                    .await
            })
            .await
            .map_err(|_e| {
                Error::Temporary("Fetching EM data timed out".into())
            })?
            .map_err(|e| {
                Error::Temporary(format!("Fetching EM data failed: {e}"))
            })?;

        let mut produced = Energy::new::<joule>(0f64);
        let mut consumed = Energy::new::<joule>(0f64);
        let mut found = 0u8;

        for obis in data {
            if obis.id == 0x00010800 {
                found |= 1 << 0;
                consumed = Energy::new::<joule>(obis.value as f64);
            } else if obis.id == 0x00020800 {
                found |= 1 << 1;
                produced = Energy::new::<joule>(obis.value as f64);
            }
        }

        if found != 0x03 {
            return Err(Error::Temporary(format!(
                "Received data did not include produced or \
                    consumed record: bitfield {found}"
            )));
        }

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
