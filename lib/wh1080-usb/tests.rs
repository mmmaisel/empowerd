extern crate hidapi;

use hidapi::HidApi;

#[test]
fn with_hid() {
    let api = match HidApi::new() {
        Ok(x) => x,
        Err(e) => panic!("Error initialising hidapi: {}", e)
    };

    println!("Initialised API");

    let device = match api.open(0x1941, 0x8021) {
        Ok(x) => x,
        Err(e) => panic!("Error opening device: {}", e)
    };

    println!("Opened Device");

    let mut buffer = [0; 1024];
    for _ in 0..10 {
        match device.read(&mut buffer) {
            Ok(x) => println!("Received {} bytes: {:02X?}", x, &buffer[0..x]),
            Err(e) => panic!("Error reading device: {}", e)
        }
    }
}
