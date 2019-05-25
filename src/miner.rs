use std::net;
use std::time;
use influx_db_client::Client;

extern crate dachs;
extern crate sma;

use dachs::*;
use sma::*;

// TODO: fix those names
use super::models::dachs as dachs_model;
use super::models::solar as solar_model;

pub struct StromMiner
{
    influx_conn: Client,
    dachs_client: DachsClient,
    sma_client: SmaClient,
    sma_pw: String,
    sma_addr: net::SocketAddr
}

impl StromMiner
{
    pub fn new(db_url: String, db_name: String,
        dachs_addr: String, dachs_pw: String,
        sma_addr: String, sma_pw: String) -> StromMiner
    {
        return StromMiner
        {
            // TODO: add DB password
            influx_conn: Client::new(format!("http://{}", db_url), db_name),
            dachs_client: DachsClient::new(dachs_addr, dachs_pw),
            sma_client: SmaClient::new(),
            sma_pw: sma_pw,
            sma_addr: SmaClient::sma_sock_addr(sma_addr).unwrap() // TODO: dont panic
        };
    }

    // TODO: add INFO logs
    pub fn mine_dachs_data(&mut self, now: i64)
    {
        let dachs_runtime = match self.dachs_client.get_runtime()
        {
            // TODO: use slog everywhere
            Ok(runtime) =>
            {
                println!("Runtime: {} s", runtime);
                runtime
            }
            Err(err) =>
            {
                println!("{}", err);
                return;
            }
        };
        let dachs_energy = match self.dachs_client.get_total_energy()
        {
            Ok(energy) =>
            {
                println!("Energy: {} kWh", energy);
                energy
            }
            Err(err) =>
            {
                println!("{}", err);
                return;
            }
        };

        let last_record = match dachs_model::DachsData::last(&self.influx_conn)
        {
            Ok(x) => x,
            Err(e) =>
            {
                if e.series_exists()
                {
                    println!("Query error {}", e);
                }
                else
                {
                    let model = dachs_model::DachsData::new(
                        now, 0.0, dachs_runtime as f64, dachs_energy as f64);
                    model.save(&self.influx_conn);
                }
                return;
            }
        };
        // TODO: derive nonlinear power from delta timestamp and delta runtime
        let power: f64 = if last_record.runtime != dachs_runtime as f64
        {
            800.0
        }
        else
        {
            0.0
        };

        // TODO: everywhere: only use f64 where necessary
        let model = dachs_model::DachsData::new(
            now, power, dachs_runtime as f64, dachs_energy as f64);
        model.save(&self.influx_conn);
    }

    // TODO: add INFO logs
    pub fn mine_solar_data(&mut self, interval: i64)
    {
        let result = self.sma_client.identify(self.sma_addr);
        let identity = match result
        {
            Err(e) => panic!(e),
            Ok(x) => x
        };

        println!("{} is {:X}, {:X}", self.sma_addr, identity.susy_id,
            identity.serial);

        self.sma_client.set_dst(self.sma_addr, identity.susy_id,
            identity.serial);

        match self.sma_client.logout()
        {
            Err(e) => println!("Logout failed: {}", e),
            Ok(_) => ()
        }
        match self.sma_client.login(&self.sma_pw)
        {
            Err(e) => println!("Login failed: {}", e),
            Ok(_) => ()
        }

        let now = time::SystemTime::now().
            duration_since(time::UNIX_EPOCH).
            unwrap().as_secs() as u32;

        // TODO: derive interval from last influx timestamp
        println!("GetDayData from {} to {}", now-86400, now);

        // TODO: this command is not accepted by SMA, needs -86400 ?
        //   this data is delayed by about one hour?
        let data = match self.sma_client.get_day_data(now-86400, now)
        {
            Err(e) =>
            {
                println!("Get Day Data failed: {}", e);
                return;
            }
            Ok(x) =>
            {
                println!("Get Day data returned {:?}", x);
                x
            }
        };

        // TODO: handle double data (identical timestamps)
        //   (handled in database?) and missing ones (delta_t > 300)
        // TODO: handle NaN (0xFFFFFFFF, 0xFFFF) values
        // TODO: always UTC, handle DST transition

        match self.sma_client.logout()
        {
            Err(e) => println!("Logout failed: {}", e),
            Ok(_) => ()
        }

        let last_record = match solar_model::SolarData::last(&self.influx_conn)
        {
            Ok(x) => x,
            Err(e) =>
            {
                if e.series_exists()
                {
                    println!("Query error {}", e);
                }
                else
                {
                    // TODO: handle first record correctly
                    solar_model::SolarData::new(0, 0.0, 0.0);
                }
                return;
            }
        };
        let mut last_energy = last_record.energy as f64;
        let mut last_timestamp = last_record.timestamp as i64;

        let records: Vec<solar_model::SolarData> = data.into_iter().map(|record|
        {
            // TODO: this is an ugly mess
            let power = 3600.0 * ((record.value as f64) - last_energy) /
                (((record.timestamp as i64) - last_timestamp) as f64);
            last_energy = record.value as f64;
            last_timestamp = record.timestamp as i64;
            return solar_model::SolarData::new(
                record.timestamp as i64,
                power,
                record.value as f64);
        }).collect();
        solar_model::SolarData::save_all(&self.influx_conn, records);
    }
}
