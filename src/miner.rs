use std::net;
use slog::Logger;
use influx_db_client::Client;

extern crate dachs_client;
extern crate sma_client;
extern crate sml_client;

use dachs_client::*;
use sma_client::*;
use sml_client::*;

use crate::models::*;
use crate::settings::*;

pub struct StromMiner
{
    influx_conn: Client,
    dachs_client: DachsClient,
    sma_client: SmaClient,
    sml_client: SmlClient,
    sma_pw: String,
    sma_addr: net::SocketAddr,
    logger: Logger
}

impl StromMiner
{
    pub fn new(s: Settings, logger: Logger) -> Result<StromMiner, String>
    {
        let influx_conn = Client::new(
            format!("http://{}", s.db_url), s.db_name).
            set_authentication(s.db_user, s.db_pw);
        let dachs_client = DachsClient::new(
            s.dachs_addr, s.dachs_pw, Some(logger.new(o!())));
        let sma_client = SmaClient::new(Some(logger.new(o!())))?;
        let sml_client = SmlClient::new(
            s.meter_device, s.meter_baud, Some(logger.new(o!())));
        let sma_addr = SmaClient::sma_sock_addr(s.sma_addr)?;

        return Ok(StromMiner
        {
            influx_conn: influx_conn,
            dachs_client: dachs_client,
            sma_client: sma_client,
            sml_client: sml_client,
            sma_pw: s.sma_pw,
            sma_addr: sma_addr,
            logger: logger
        });
    }

    pub fn mine_dachs_data(&mut self, now: i64)
    {
        let dachs_runtime = match self.dachs_client.get_runtime()
        {
            Ok(runtime) =>
            {
                trace!(self.logger, "Runtime: {} s", runtime);
                runtime
            }
            Err(err) =>
            {
                error!(self.logger, "{}", err);
                return;
            }
        };
        let dachs_energy = match self.dachs_client.get_total_energy()
        {
            Ok(energy) =>
            {
                trace!(self.logger, "Energy: {} kWh", energy);
                energy
            }
            Err(err) =>
            {
                error!(self.logger, "{}", err);
                return;
            }
        };
        self.save_dachs_data(now, dachs_runtime as f64,
            dachs_energy as f64);
    }

    // TODO: everywhere: only use f64 where necessary
    pub fn save_dachs_data(&self, timestamp: i64, dachs_runtime: f64,
        dachs_energy: f64)
    {
        let last_record = match DachsData::last(&self.influx_conn)
        {
            Ok(x) => x,
            Err(e) =>
            {
                if e.series_exists()
                {
                    error!(self.logger, "Query error {}", e);
                }
                else
                {
                    let model = DachsData::new(
                        timestamp, 0.0, dachs_runtime, dachs_energy);
                    if let Err(e) = model.save(&self.influx_conn)
                    {
                        error!(self.logger, "Save DachsData failed, {}", e);
                    }
                    else
                    {
                        trace!(self.logger, "Wrote {:?} to database", model);
                    }
                }
                return;
            }
        };
        trace!(self.logger, "Read {:?} from database", last_record);
        // TODO: derive nonlinear power from delta timestamp and delta runtime
        let power: f64 = if dachs_runtime != last_record.runtime
        {
            800.0
        }
        else
        {
            0.0
        };

        let model = DachsData::new(
            timestamp, power, dachs_runtime, dachs_energy);
        if let Err(e) = model.save(&self.influx_conn)
        {
            error!(self.logger, "Save DachsData failed, {}", e);
        }
        else
        {
            trace!(self.logger, "Wrote {:?} to database", model);
        }
    }

    pub fn get_last_solar_record(&self) -> Result<SolarData, ()>
    {
        let last_record = match SolarData::last(&self.influx_conn)
        {
            Ok(x) => x,
            Err(e) =>
            {
                if e.series_exists()
                {
                    error!(self.logger, "Query error {}", e);
                    return Err(());
                }
                else
                {
                    SolarData::new(0, 0.0, 0.0)
                }
            }
        };
        trace!(self.logger, "Read {:?} from database", last_record);
        return Ok(last_record);
    }

    pub fn mine_solar_data(&mut self, now: i64)
    {
        let last_record = match self.get_last_solar_record()
        {
            Ok(x) => x,
            Err(_) => return
        };
        let result = self.sma_client.identify(self.sma_addr);
        let identity = match result
        {
            Err(e) =>
            {
                error!(self.logger, "Could not identify SMA device, {}", e);
                return;
            }
            Ok(x) => x
        };

        trace!(self.logger, "{} is {:X}, {:X}", self.sma_addr,
            identity.susy_id, identity.serial);

        self.sma_client.set_dst(self.sma_addr, identity.susy_id,
            identity.serial);

        if let Err(e) = self.sma_client.logout()
        {
            error!(self.logger, "Logout failed: {}", e);
            return;
        }
        if let Err(e) = self.sma_client.login(&self.sma_pw)
        {
            error!(self.logger, "Login failed: {}", e);
            return;
        }

        trace!(self.logger, "GetDayData from {} to {}",
            last_record.timestamp, now);

        // TODO: this command is not accepted by SMA, needs -86400 ?
        //   this data is delayed by about one hour?
        let data = match self.sma_client.get_day_data(
            last_record.timestamp as u32, now as u32)
        {
            Err(e) =>
            {
                error!(self.logger, "Get Day Data failed: {}", e);
                return;
            }
            Ok(x) =>
            {
                trace!(self.logger, "Get Day data returned {:?}", x);
                x
            }
        };

        // TODO: handle double data (identical timestamps)
        //   (handled in database?) and missing ones (delta_t > 300)
        // TODO: handle NaN (0xFFFFFFFF, 0xFFFF) values(in SMA client validators)
        // TODO: always UTC, handle DST transition

        if let Err(e) = self.sma_client.logout()
        {
            error!(self.logger, "Logout failed: {}", e);
            return;
        }

        self.save_solar_data(data, Some(last_record));
    }

    pub fn save_solar_data(&self, data: Vec<TimestampedInt>,
        last_record: Option<SolarData>)
    {
        let last_record = match last_record
        {
            Some(x) => x,
            None =>
            {
                match self.get_last_solar_record()
                {
                    Ok(x) => x,
                    Err(_) => return
                }
            }
        };
        let mut last_energy = last_record.energy as f64;
        let mut last_timestamp = last_record.timestamp as i64;

        let records: Vec<SolarData> = data.into_iter().filter_map(|record|
        {
            if record.timestamp as i64 == last_timestamp
            {
                return Option::None;
            }

            if record.value as u32 == 0xFFFFFFFF
            {
                warn!(self.logger, "Skipping NaN SMA record");
                return Option::None;
            }

            // TODO: this is an ugly mess
            let power = if last_timestamp == 0
            {
                0.0
            }
            else
            {
                3600.0 * ((record.value as f64) - last_energy) /
                (((record.timestamp as i64) - last_timestamp) as f64)
            };

            last_energy = record.value as f64;
            last_timestamp = record.timestamp as i64;
            return Some(SolarData::new(
                record.timestamp as i64,
                power,
                record.value as f64));
        }).collect();

        let record_count = records.len();
        if let Err(e) = SolarData::save_all(&self.influx_conn, records)
        {
            error!(self.logger, "Save SolarData failed, {}", e);
        }
        else
        {
            trace!(self.logger, "Wrote {} solar records to database",
                record_count);
        }
    }

    pub fn mine_meter_data(&mut self, now: i64)
    {
        let mut meter_data = self.sml_client.get_consumed_produced();
        for _ in 1..3
        {
            if let Err(e) = meter_data
            {
                error!(self.logger,
                    "Get electric meter data failed, {}, retrying...", e);
                meter_data = self.sml_client.get_consumed_produced();
            }
            else
            {
                break;
            }
        }
        let (consumed, produced) = match meter_data
        {
            Ok((x, y)) => (x, y),
            Err(e) =>
            {
                error!(self.logger,
                    "Get electric meter data failed, {}, giving up!", e);
                return;
            }
        };

        self.save_meter_data(now, produced, consumed);
    }

    pub fn save_meter_data(&self, timestamp: i64, produced: f64, consumed: f64)
    {
        // TODO: deduplicate this
        let last_record = match MeterData::last(&self.influx_conn)
        {
            Ok(x) => x,
            Err(e) =>
            {
                if e.series_exists()
                {
                    error!(self.logger, "Query error {}", e);
                }
                else
                {
                    let model = MeterData::new(timestamp, 0.0, produced,
                        consumed);
                    if let Err(e) = model.save(&self.influx_conn)
                    {
                        error!(self.logger, "Save MeterData failed, {}", e);
                    }
                    else
                    {
                        trace!(self.logger, "Wrote {:?} to database", model);
                    }
                }
                return;
            }
        };
        trace!(self.logger, "Read {:?} from database", last_record);

        let power = 3600.0 * (
            consumed - last_record.energy_consumed -
            (produced - last_record.energy_produced) ) /
            ((timestamp - last_record.timestamp) as f64);

        // TODO: everywhere: only use f64 where necessary
        let model = MeterData::new(timestamp, power, produced, consumed);
        if let Err(e) = model.save(&self.influx_conn)
        {
            error!(self.logger, "Save MeterData failed, {}", e);
        }
        else
        {
            trace!(self.logger, "Wrote {:?} to database", model);
        }
    }

    pub fn save_gas_data(&self, timestamp: i64, current: f64, reset: bool)
    {
        // TODO: deduplicate this
        let last_record = match GasData::last(&self.influx_conn)
        {
            Ok(x) => x,
            Err(e) =>
            {
                if e.series_exists()
                {
                    error!(self.logger, "Query error {}", e);
                }
                else
                {
                    let model = GasData::new(timestamp, 0.0, current, 0.0);
                    if let Err(e) = model.save(&self.influx_conn)
                    {
                        error!(self.logger, "Save GasData failed, {}", e);
                    }
                    else
                    {
                        trace!(self.logger, "Wrote {:?} to database", model);
                    }
                }
                return;
            }
        };
        trace!(self.logger, "Read {:?} from database", last_record);

        let rate = if reset
        {
            0.0
        }
        else
        {
            (current - last_record.current) /
            ((timestamp - last_record.timestamp) as f64)
        };
        let total = if reset
        {
            0.0
        }
        else
        {
            current - last_record.current + last_record.total
        };

        // TODO: everywhere: only use f64 where necessary
        let model = GasData::new(timestamp, rate, total, current);
        if let Err(e) = model.save(&self.influx_conn)
        {
            error!(self.logger, "Save GasData failed, {}", e);
        }
        else
        {
            trace!(self.logger, "Wrote {:?} to database", model);
        }
    }

    pub fn save_water_data(&self, timestamp: i64, current: f64, reset: bool)
    {
        // TODO: deduplicate this
        let last_record = match WaterData::last(&self.influx_conn)
        {
            Ok(x) => x,
            Err(e) =>
            {
                if e.series_exists()
                {
                    error!(self.logger, "Query error {}", e);
                }
                else
                {
                    let model = WaterData::new(timestamp, 0.0, current, 0.0);
                    if let Err(e) = model.save(&self.influx_conn)
                    {
                        error!(self.logger, "Save WaterData failed, {}", e);
                    }
                    else
                    {
                        trace!(self.logger, "Wrote {:?} to database", model);
                    }
                }
                return;
            }
        };
        trace!(self.logger, "Read {:?} from database", last_record);

        let rate = if reset
        {
            0.0
        }
        else
        {
            (current - last_record.current) /
            ((timestamp - last_record.timestamp) as f64)
        };
        let total = if reset
        {
            0.0
        }
        else
        {
            current - last_record.current + last_record.total
        };

        // TODO: everywhere: only use f64 where necessary
        let model = WaterData::new(timestamp, rate, total, current);
        if let Err(e) = model.save(&self.influx_conn)
        {
            error!(self.logger, "Save WaterData failed, {}", e);
        }
        else
        {
            trace!(self.logger, "Wrote {:?} to database", model);
        }
    }
}
