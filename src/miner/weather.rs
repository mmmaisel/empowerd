use super::{Miner, MinerResult, MinerState};
use crate::models::{InfluxObject, Weather};
use bresser6in1_usb::{Client as BresserClient, Data as BresserData};
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use slog::{debug, error, info, trace, Logger};
use std::time::{Duration, UNIX_EPOCH};
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
        match Miner::sleep_aligned(
            self.interval,
            &mut self.canceled,
            &self.logger,
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
                MinerState::Running(_) => (),
            },
        };

        let logger2 = self.logger.clone();
        let weather_data = match tokio::task::spawn_blocking(move || {
            // TODO: move bresser client (allocated buffers, ...) back to Miner struct
            let mut bresser_client = BresserClient::new(Some(logger2.clone()));
            let mut weather_data = bresser_client.read_data();
            for _ in 1..3 {
                if let Err(e) = weather_data {
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
        let weather_data = match weather_data {
            Ok(x) => x,
            Err(e) => {
                error!(
                    self.logger,
                    "Get weather data failed, {}, giving up!", e
                );
                return MinerResult::Running;
            }
        };

        let weather = Weather::new(weather_data);
        trace!(self.logger, "Writing {:?} to database", &weather);
        // XXX: extract save method
        if let Err(e) = self.influx.query(&weather.into_query("weather")).await
        {
            error!(self.logger, "Save WeatherData failed, {}", e);
        }
        return MinerResult::Running;
    }
}
