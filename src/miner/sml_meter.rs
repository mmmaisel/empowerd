use super::{Miner, MinerResult, MinerState};
use crate::miner_sleep;
use crate::models::{InfluxObject, InfluxResult, Meter};
use chrono::{DateTime, Utc};
use slog::{debug, error, trace, warn, Logger};
use sml_client::SmlClient;
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::watch;

pub struct SmlMeterMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    name: String,
    interval: Duration,
    logger: Logger,
    sml_client: SmlClient,
    meter_device: String,
}

impl SmlMeterMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        name: String,
        interval: Duration,
        meter_device: String,
        meter_baud: u32,
        logger: Logger,
    ) -> Result<SmlMeterMiner, String> {
        if interval < Duration::from_secs(5) {
            return Err("SmlMeterMiner:poll_interval must be >= 5".into());
        }
        return Ok(SmlMeterMiner {
            canceled: canceled,
            influx: influx,
            name: name,
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

    pub async fn mine(&mut self) -> MinerResult {
        let now = miner_sleep!(self);

        let mut meter_data = self.sml_client.get_consumed_produced().await;
        for i in 1..4u8 {
            if let Err(e) = meter_data {
                if i == 2 {
                    match usb_reset::reset_path(&self.meter_device) {
                        Ok(()) => {
                            warn!(
                                self.logger,
                                "Reset device {}", &self.meter_device
                            );
                            tokio::time::sleep(Duration::from_secs(5)).await;
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
                debug!(
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
            self.influx.json_query(Meter::query_last(&self.name)).await,
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
        if let Err(e) = self.influx.query(&meter.save_query(&self.name)).await {
            error!(self.logger, "Save MeterData failed, {}", e);
        }
        return MinerResult::Running;
    }
}
