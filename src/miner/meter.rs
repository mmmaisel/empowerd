use super::{Miner, MinerResult, MinerState};
use crate::models::{InfluxObject, InfluxResult, Meter};
use chrono::{DateTime, Utc};
use slog::{error, trace, warn, Logger};
use sml_client::SmlClient;
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::watch;

pub struct MeterMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    interval: Duration,
    logger: Logger,
    sml_client: SmlClient,
    meter_device: String,
}

impl MeterMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        interval: Duration,
        meter_device: String,
        meter_baud: u32,
        logger: Logger,
    ) -> Result<MeterMiner, String> {
        return Ok(MeterMiner {
            canceled: canceled,
            influx: influx,
            interval: interval,
            logger: logger.clone(),
            sml_client: SmlClient::new(
                meter_device.clone(),
                meter_baud,
                Some(logger),
            ),
            meter_device: meter_device,
        });
    }

    // TODO: dedup
    pub async fn mine(&mut self) -> MinerResult {
        let now = match Miner::sleep_aligned(
            self.interval,
            &mut self.canceled,
            &self.logger,
            "Meter",
        )
        .await
        {
            Err(e) => {
                return MinerResult::Err(format!(
                    "sleep_aligned failed in MeterMiner: {}",
                    e
                ));
            }
            Ok(state) => match state {
                MinerState::Canceled => return MinerResult::Canceled,
                MinerState::Running(x) => x,
            },
        };

        let mut meter_data = self.sml_client.get_consumed_produced().await;
        for i in 1..3u8 {
            if let Err(e) = meter_data {
                if i == 2 {
                    match usb_reset::reset_device(&self.meter_device) {
                        Ok(()) => {
                            warn!(
                                self.logger,
                                "Reset device {}", &self.meter_device
                            );
                        }
                        Err(e) => {
                            error!(
                                self.logger,
                                "Reset device {} failed: {}",
                                &self.meter_device,
                                e
                            );
                        }
                    }
                }
                error!(
                    self.logger,
                    "Get electric meter data failed, {}, retrying...", e
                );
                meter_data = self.sml_client.get_consumed_produced().await;
            } else {
                break;
            }
        }
        let (consumed, produced) = match meter_data {
            Ok((x, y)) => (x, y),
            Err(e) => {
                error!(
                    self.logger,
                    "Get electric meter data failed, {}, giving up!", e
                );
                return MinerResult::Running;
            }
        };

        let power = match Meter::into_single(
            self.influx.json_query(Meter::query_last()).await,
        ) {
            InfluxResult::Some(last_record) => {
                trace!(self.logger, "Read {:?} from database", last_record);
                3600.0
                    * (consumed
                        - last_record.energy_consumed
                        - (produced - last_record.energy_produced))
                    / ((now - last_record.time.timestamp() as u64) as f64)
            }
            InfluxResult::None => 0.0,
            InfluxResult::Err(e) => {
                error!(self.logger, "Query dachs database failed: {}", e);
                return MinerResult::Running;
            }
        };

        let meter = Meter::new(
            DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(now)),
            consumed,
            produced,
            power,
        );
        trace!(self.logger, "Writing {:?} to database", &meter);
        if let Err(e) = self.influx.query(&meter.save_query()).await {
            error!(self.logger, "Save MeterData failed, {}", e);
        }
        return MinerResult::Running;
    }
}
