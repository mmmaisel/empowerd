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

    let mut buffer = [0; 32];
    for addr in 0..32 {
        let addr_h = ((addr as u16)*32/256) as u8;
        let addr_l = ((addr as u16)*32%256) as u8;
        let read_0_cmd = vec![0x00, 0xA1, addr_h, addr_l,
            0x20, 0xA1, addr_h, addr_l, 0x20];
        println!("Reading address {:X}", addr*32);
        match device.write(&read_0_cmd) {
            Ok(count) => {
                if count != read_0_cmd.len() {
                    panic!("Not all data was written");
                }
            }
            Err(e) => panic!("Writing data failed: {}", e)
        }

        match device.read(&mut buffer) {
            Ok(x) => println!("Received {} bytes: {:02X?}", x, &buffer[0..x]),
            Err(e) => panic!("Error reading device: {}", e)
        }
    }
}
