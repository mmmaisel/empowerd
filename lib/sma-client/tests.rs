use std::time::*;
use crate::SmaClient;

#[test]
fn read_solar_data() {
    let sma_addr = match SmaClient::sma_sock_addr("192.168.5.75".to_string()) {
        Ok(x) => x,
        Err(e) => panic!(e)
    };

    let mut sma_client = match SmaClient::new(None) {
        Ok(x) => x,
        Err(e) => panic!(e)
    };

    let result = sma_client.identify(sma_addr);
    let identity = match result {
        Err(e) => panic!("Could not identify SMA device, {}", e),
        Ok(x) => x
    };

    eprintln!("{} is {:X}, {:X}", sma_addr, identity.susy_id,
        identity.serial);

    sma_client.set_dst(sma_addr, identity.susy_id, identity.serial);

    if let Err(e) = sma_client.logout() {
        panic!("Logout failed: {}", e);
    }
    if let Err(e) = sma_client.login(&"0000".to_string()) {
        panic!("Login failed: {}", e);
    }

    let to = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(x) => x.as_secs() as u32,
        Err(e) => panic!(e)
    };
    let from = to - 3600;

    eprintln!("GetDayData from {} to {}", from, to);

    let data = match sma_client.get_day_data(from, to) {
        Err(e) => panic!("Get Day Data failed: {}", e),
        Ok(x) => x
    };

    eprintln!("Get Day data returned {:?}", data);

    if let Err(e) = sma_client.logout() {
        panic!("Logout failed: {}", e);
    }
}
