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
        units::{second, watt, watt_hour, Abbreviation, Energy, Power, Time},
        SimpleMeter,
    },
    task_group::TaskResult,
    Error,
};
use slog::{debug, trace, Logger};
use sma_proto::{
    client::{SmaClient, SmaSession},
    inverter::SmaInvMeterValue,
    SmaEndpoint,
};
use std::net::Ipv4Addr;

pub struct SunnyBoySpeedwireSource {
    base: SourceBase,
    sma_client: SmaClient,
    sma_pw: String,
    sma_addr: Ipv4Addr,
}

impl SunnyBoySpeedwireSource {
    pub fn new(
        base: SourceBase,
        sma_pw: String,
        sma_addr: Ipv4Addr,
    ) -> Result<Self, String> {
        Ok(Self {
            base,
            sma_client: SmaClient::new(SmaEndpoint::dummy()),
            sma_pw,
            sma_addr,
        })
    }

    pub fn logger(&self) -> &Logger {
        &self.base.logger
    }

    // XXX: this function is much too long
    pub async fn run(&mut self) -> TaskResult {
        let timing = self.base.sleep_aligned().await?;
        let mut conn = self.base.get_database().await?;

        let mut last_record =
            match SimpleMeter::last(&mut conn, self.base.series_id).await {
                Ok(x) => x,
                Err(Error::NotFound) => SimpleMeter {
                    time: Time::new::<second>(0.0),
                    energy: Energy::new::<watt_hour>(0.0),
                    power: Power::new::<watt>(0.0),
                },
                Err(e) => {
                    return Err(Error::Temporary(format!(
                        "Query {} database failed: {}",
                        &self.base.name, e,
                    )))
                }
            };
        let session = SmaSession::open_unicast(self.sma_addr).map_err(|e| {
            Error::Temporary(format!("Could not open SMA Client session: {e}",))
        })?;
        let dst_id = self.sma_client.identify(&session).await.map_err(|e| {
            Error::Temporary(format!("Could not identify SMA device, {e}",))
        })?;

        trace!(self.base.logger, "Device is {dst_id:X?}");

        self.sma_client
            .logout(&session, &dst_id)
            .await
            .map_err(|e| Error::Temporary(format!("Logout failed: {e}")))?;
        self.sma_client
            .login(&session, &dst_id, &self.sma_pw)
            .await
            .map_err(|e| Error::Temporary(format!("Login failed: {e}")))?;

        trace!(
            self.base.logger,
            "GetDayData from {} to {}",
            last_record.time.into_format_args(second, Abbreviation),
            Time::new::<second>(timing.now as f64)
                .into_format_args(second, Abbreviation),
        );

        let mut last_energy = last_record.energy;

        // TODO: this command is not accepted by SMA, needs -86400 ?
        //   this data is delayed by about one hour?
        let points: Vec<SmaInvMeterValue> = match self
            .sma_client
            .get_day_data(
                &session,
                &dst_id,
                last_record.time.get::<second>() as u32,
                timing.now as u32,
            )
            .await
        {
            Err(e) => {
                return Err(Error::Temporary(format!(
                    "Get Day Data failed: {e}",
                )))
            }
            Ok(points) => {
                trace!(self.base.logger, "Get Day data returned {:?}", points);
                points
                    .into_iter()
                    .filter(|point| {
                        let energy =
                            Energy::new::<watt_hour>(point.energy_wh as f64);
                        if !point.is_valid() {
                            debug!(self.base.logger, "Skipping NaN SMA record");
                            return false;
                        } else if point.timestamp as u64
                            == last_record.time.get::<second>() as u64
                        {
                            return false;
                        } else if energy < last_energy {
                            // Sometimes, the last value from speedwire is just garbage.
                            debug!(
                                self.base.logger,
                                "Energy meter run backwards. Ignoring point {}.",
                                point.energy_wh
                            );
                            return false;
                        } else {
                            last_energy = energy;
                            return true;
                        }
                    })
                    .collect()
            }
        };

        // TODO: handle double data (identical timestamps)
        //   (handled in database?) and missing ones (delta_t > 300)
        // TODO: always UTC, handle DST transition

        self.sma_client
            .logout(&session, &dst_id)
            .await
            .map_err(|e| Error::Temporary(format!("Logout failed: {e}")))?;

        let num_points = points.len();
        for point in points {
            let time = Time::new::<second>(point.timestamp as f64);
            let energy = Energy::new::<watt_hour>(point.energy_wh as f64);
            let mut record = SimpleMeter {
                time,
                energy,
                power: Power::new::<watt>(0.0),
            };
            record.power = record.calc_power(&last_record);
            last_record = record.clone();

            self.base.notify_processors(&record);
            record.insert(&mut conn, self.base.series_id).await?;
        }

        trace!(
            self.base.logger,
            "Wrote {} simple meter records to database",
            num_points
        );
        Ok(())
    }
}
