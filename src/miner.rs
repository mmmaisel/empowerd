use std::net;
use std::time;
extern crate dachs;
extern crate sma;

use dachs::*;
use sma::*;

pub struct StromMiner
{
    dachs_client: DachsClient,
    sma_client: SmaClient,
    sma_pw: String,
    sma_addr: net::SocketAddr
}

impl StromMiner
{
    pub fn new(dachs_addr: String, dachs_pw: String, sma_addr: String,
        sma_pw: String) -> StromMiner
    {
        return StromMiner
        {
            dachs_client: DachsClient::new(dachs_addr, dachs_pw),
            sma_client: SmaClient::new(),
            sma_pw: sma_pw,
            sma_addr: SmaClient::sma_sock_addr(sma_addr).unwrap() // TODO: dont panic
        };
    }

    pub fn get_dachs_data(&mut self)
    {
        let dachs_runtime = self.dachs_client.get_runtime();
        let dachs_energy = self.dachs_client.get_total_energy();

        match dachs_runtime
        {
            Ok(runtime) => println!("Runtime: {} s", runtime),
            Err(err) => println!("{}", err)
        }

        match dachs_energy
        {
            Ok(energy) => println!("Energy: {} kWh", energy),
            Err(err) => println!("{}", err)
        }

//        let start_value = influx.get_lastest
        let last_value: u32 = 0;

        // TODO: convert runtime to current_power
        // TODO: write data into DB
    }

    pub fn get_sma_data(&mut self)
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

        println!("GetDayData from {} to {}", now-86400, now);

        match self.sma_client.get_day_data(now-86400, now)
        {
            Err(e) => println!("Get Day Data failed: {}", e),
            Ok(x) =>
            {
                println!("Get Day data returned {:?}", x);
            }
        }

        // TODO: handle double data (identical timestamps)
        //   (handled in database?) and missing ones (delta_t > 300)
        // TODO: handle NaN (0xFFFFFFFF, 0xFFFF) values
        // TODO: always UTC, handle DST transition

        match self.sma_client.logout()
        {
            Err(e) => println!("Logout failed: {}", e),
            Ok(_) => ()
        }

        // TODO: convert to current_power
        // TODO: write data into DB
    }
}
