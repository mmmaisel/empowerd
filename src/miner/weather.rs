use super::{Miner, MinerResult, MinerState};
use crate::models::{InfluxObject, Weather};
use bresser6in1_usb::{Client as BresserClient, PID, VID};
use slog::{error, trace, warn, Logger};
use std::time::Duration;
use tokio::sync::watch;

pub struct WeatherMiner {
    canceled: watch::Receiver<MinerState>,
    influx: influxdb::Client,
    interval: Duration,
    logger: Logger,
    //bresser_client: BresserClient,
}

impl WeatherMiner {
    pub fn new(
        canceled: watch::Receiver<MinerState>,
        influx: influxdb::Client,
        interval: Duration,
        logger: Logger,
    ) -> Result<WeatherMiner, String> {
        return Ok(WeatherMiner {
            canceled: canceled,
            influx: influx,
            interval: interval,
            logger: logger.clone(),
        });
    }

    // TODO: dedup
    pub async fn mine(&mut self) -> MinerResult {
        let now = match Miner::sleep_aligned(
            self.interval,
            &mut self.canceled,
            &self.logger,
            "Weather",
        )
        .await
        {
            Err(e) => {
                return MinerResult::Err(format!(
                    "sleep_aligned failed in WeatherMiner: {}",
                    e
                ));
            }
            Ok(state) => match state {
                MinerState::Canceled => return MinerResult::Canceled,
                MinerState::Running(x) => x,
            },
        };

        let logger2 = self.logger.clone();
        let weather_data = match tokio::task::spawn_blocking(move || {
            // TODO: move bresser client (allocated buffers, ...) back to Miner struct
            let mut bresser_client = BresserClient::new(Some(logger2.clone()));
            let mut weather_data = bresser_client.read_data();
            for i in 1..3u8 {
                if let Err(e) = weather_data {
                    if i == 2 {
                        match usb_reset::reset_vid_pid(VID, PID) {
                            Ok(()) => {
                                warn!(logger2, "Reset device {}:{}", VID, PID);
                            }
                            Err(e) => {
                                error!(
                                    logger2,
                                    "Reset device {}:{} failed: {}",
                                    VID,
                                    PID,
                                    e
                                );
                            }
                        }
                    }
                    error!(
                        logger2,
                        "Get weather data failed, {}, retrying...", e
                    );
                    weather_data = bresser_client.read_data();
                } else {
                    break;
                }
            }
            return weather_data;
        })
        .await
        {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.logger,
                    "Joining blocking Bresser USB task failed: {}", e
                );
                return MinerResult::Running;
            }
        };
        let mut weather_data = match weather_data {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.logger,
                    "Get weather data failed, {}, giving up!", e
                );
                return MinerResult::Running;
            }
        };
        weather_data.timestamp = now as u32;

        let weather = Weather::new(weather_data);
        trace!(self.logger, "Writing {:?} to database", &weather);
        if let Err(e) = self.influx.query(&weather.save_query()).await {
            error!(self.logger, "Save WeatherData failed, {}", e);
        }
        return MinerResult::Running;
    }
}
