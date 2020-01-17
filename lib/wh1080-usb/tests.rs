extern crate hidapi;

use hidapi::HidApi;

use crate::WH1080Client;

#[test]
fn read_data_from_usb() {
    let mut client = WH1080Client::new(None);

    match client.device_info() {
        Ok(x) => println!("{}", x),
        Err(e) => panic!(e),
    }

    match client.read_data() {
        Ok(_) => (),
        Err(e) => panic!(e),
    }
}

//#[test]
fn with_hid() {
    let api = match HidApi::new() {
        Ok(x) => x,
        Err(e) => panic!("Error initialising hidapi: {}", e),
    };

    println!("Initialised API");

    let device = match api.open(0x1941, 0x8021) {
        Ok(x) => x,
        Err(e) => panic!("Error opening device: {}", e),
    };

    println!("Opened Device");

    let mut buffer = [0; 256];
    /*let read_0_cmd =
        vec![0x00, 0xfc, 0x03, 0x00, 0x00, 0x00, 0x2f, 0xa1, 0xfd]; // ???
        //vec![0x00, 0xfc, 0x07, 0x00, 0x00, 0x00, 0xe5, 0x50, 0xfd]; // ???
        vec![0x00, 0xfc, 0x08, 0x14, 0x01, 0x0c, 0xa0, 0x5c, 0xfd]; // ???
        //vec![0x00, 0xfc, 0x09, 0x0d, 0x17, 0x24, 0x59, 0xfb, 0xfd]; // more output ?
        //vec![0x00, 0xfc, 0xd4, 0x01, 0x00, 0x00, 0xe1, 0xbf, 0xfd]; // more output ?
        //vec![0x00, 0xfc, 0xd5, 0x01, 0x00, 0x00, 0x97, 0x0b, 0xfd]; // more output ?
    match device.write(&read_0_cmd) {
        Ok(count) => {
            if count != read_0_cmd.len() {
                panic!("Not all data was written");
            }
        }
        Err(e) => panic!("Writing data failed: {}", e),
    }*/
    for addr in 0..16 {
        match device.read(&mut buffer) {
            Ok(x) => {
                println!("Received {} bytes: {}", x,
                    String::from_utf8_lossy(&buffer[0..x]));
                println!("{:2X?}", &buffer[0..x]);
            }
            Err(e) => panic!("Error reading device: {}", e),
        }
    }
}

use std::{slice, time::Duration};

use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Result,
    TransferType, UsbContext,
};

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

//#[test]
fn with_rusb() {
    let vid = 0x1941;
    let pid = 0x8021;

    match Context::new() {
        Ok(mut context) => match open_device(&mut context, vid, pid) {
            Some((mut device, device_desc, mut handle)) => {
                read_device(&mut device, &device_desc, &mut handle).unwrap()
            }
            None => println!("could not find device {:04x}:{:04x}", vid, pid),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

fn read_device<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    handle: &mut DeviceHandle<T>,
) -> Result<()> {
    handle.reset()?;

    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);
    println!("Languages: {:?}", languages);

    if languages.len() > 0 {
        let language = languages[0];

        println!(
            "Manufacturer: {:?}",
            handle
                .read_manufacturer_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Product: {:?}",
            handle
                .read_product_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Serial Number: {:?}",
            handle
                .read_serial_number_string(language, device_desc, timeout)
                .ok()
        );
    }

    match find_readable_endpoint(device, device_desc, TransferType::Interrupt) {
        Some(endpoint) => {
            read_endpoint(handle, endpoint, TransferType::Interrupt)
        }
        None => println!("No readable interrupt endpoint"),
    }

    match find_readable_endpoint(device, device_desc, TransferType::Bulk) {
        Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Bulk),
        None => println!("No readable bulk endpoint"),
    }

    Ok(())
}

fn find_readable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,
) -> Option<Endpoint> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    if endpoint_desc.direction() == Direction::In
                        && endpoint_desc.transfer_type() == transfer_type
                    {
                        return Some(Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        });
                    }
                }
            }
        }
    }

    None
}

fn read_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: Endpoint,
    transfer_type: TransferType,
) {
    println!("Reading from endpoint: {:?}", endpoint);

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface).ok();
            true
        }
        _ => false,
    };

    println!(" - kernel driver? {}", has_kernel_driver);

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let mut vec = Vec::<u8>::with_capacity(256);
            let buf = unsafe {
                slice::from_raw_parts_mut(
                    (&mut vec[..]).as_mut_ptr(),
                    vec.capacity(),
                )
            };

            let timeout = Duration::from_secs(10);

            match transfer_type {
                TransferType::Interrupt => {
                    match handle.read_interrupt(endpoint.address, buf, timeout)
                    {
                        Ok(len) => {
                            unsafe { vec.set_len(len) };
                            println!(" - read: {:X?}", vec);
                        }
                        Err(err) => {
                            println!("could not read from endpoint: {}", err)
                        }
                    }
                }
                TransferType::Bulk => {
                    match handle.read_bulk(endpoint.address, buf, timeout) {
                        Ok(len) => {
                            unsafe { vec.set_len(len) };
                            println!(" - read: {:X?}", vec);
                        }
                        Err(err) => {
                            println!("could not read from endpoint: {}", err)
                        }
                    }
                }
                _ => (),
            }
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface).ok();
    }
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}

//#[test]
fn decode_data() {
    let data = vec![
        0xFA, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xEC, 0x85, 0xFD, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x31, 0x36, 0x31,
        0x20, 0x32, 0x30, 0x32, 0x30, 0x2D, 0x30, 0x31, 0x2D, 0x31, 0x30, 0x20,
        0x31, 0x34, 0x3A, 0x34, 0x32, 0x20, 0x32, 0x30, 0x2E, 0x39, 0x20, 0x35,
        0x32, 0x20, 0x31, 0x30, 0x2E, 0x36, 0x20, 0x35, 0x33, 0x20, 0x31, 0x2E,
        0x37, 0x37, 0x38, 0x20, 0x30, 0x2E, 0x30, 0x20, 0x35, 0x2E, 0x34, 0x20,
        0x37, 0x2E, 0x39, 0x20, 0x32, 0x49, 0x0B, 0xFD, 0xFE, 0x00, 0x00, 0x00,
        0x00, 0x32, 0x36, 0x38, 0x34, 0x20, 0x57, 0x4E, 0x57, 0x20, 0x31, 0x30,
        0x31, 0x39, 0x20, 0x39, 0x35, 0x36, 0x20, 0x30, 0x20, 0x31, 0x2E, 0x33,
        0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D,
        0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D,
        0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0xE0, 0x1C, 0xFD,
        0xFE, 0x00, 0x00, 0x00, 0x00, 0x33, 0x1C, 0x2D, 0x20, 0x2D, 0x2D, 0x20,
        0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D,
        0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x61, 0xE8, 0xFD,
    ];

    // XXX: this are multiple HID blocks in one buffer: 0xFX, 0x00-3, ..., 0xFD
    let data2 = vec![
        0xFE, 0x00, 0x00, 0x00, 0x00, 0x33, 0x1D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D,
        0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E,
        0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x43, 0x2F, 0xFD, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x31, 0x36, 0x31,
        0x20, 0x32, 0x30, 0x32, 0x30, 0x2D, 0x30, 0x31, 0x2D, 0x31, 0x30, 0x20,
        0x31, 0x34, 0x3A, 0x35, 0x37, 0x20, 0x32, 0x31, 0x2E, 0x31, 0x20, 0x35,
        0x32, 0x20, 0x31, 0x30, 0x2E, 0x35, 0x20, 0x35, 0x33, 0x20, 0x31, 0x2E,
        0x37, 0x37, 0x38, 0x20, 0x30, 0x2E, 0x30, 0x20, 0x31, 0x36, 0x2E, 0x35,
        0x20, 0x31, 0x36, 0x2E, 0x35, 0x66, 0xE9, 0xFD, 0xFE, 0x00, 0x00, 0x00,
        0x00, 0x32, 0x36, 0x20, 0x32, 0x32, 0x37, 0x20, 0x53, 0x57, 0x20, 0x31,
        0x30, 0x31, 0x39, 0x20, 0x39, 0x35, 0x36, 0x20, 0x30, 0x20, 0x31, 0x2E,
        0x32, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20,
        0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D,
        0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x06, 0x1D, 0xFD,
        0xFE, 0x00, 0x00, 0x00, 0x00, 0x33, 0x1D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D,
        0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E,
        0x2D, 0x20, 0x2D, 0x2D, 0x20, 0x2D, 0x2D, 0x2E, 0x2D, 0x20, 0x2D, 0x2D,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x43, 0x2F, 0xFD,
    ];

    //@@þ1 6 1 2020-01-12 17:03 21.7 55 2.9 76 0.254 0.0 0.0 0.0 13 fý
    //@@þ2 6 5 SE 1023 960 0 -0.9 --.- --.- -- --.- -- --.- -- --.- ¯ÿý
    //@@þ3 . -- --.- -- --.- -- --.- -- wÛý

    //          date      time  t_in hum_in t_out hum_out  rain_d  rain_a  wind_a wind_g wind_d..
    //@@þ1 6 1 2020-01-12 17:03 21.7 55      2.9    76      0.254    0.0    0.0    0.0   13       fý
    //       ..dir wind baro_sea baro_abs  uv?  dew
    //@@þ2 6   5    SE     1023     960      0   -0.9 --.- --.- -- --.- -- --.- -- --.- ¯ÿý
    //@@þ3 . -- --.- -- --.- -- --.- -- wÛý

    //@@ûA 6 &dateutc=now&baromin=30.21&tempf=37.2&humidity=76&wind ".ý
    //@@ûB 6 speedmph=0&windgustmph=0&winddir=135&dewptf=30.3&raini ¶Æý
    //@@ûC 6 n=0&dailyrainin=0.01&UV=0&indoortempf=71.0&indoorhumid _0ý
    //@@ûD   ity=55ý

    //2020-01-12 13:24 21.1 52 5.6 65 0.0.0 2.5 2.5 16 S 1024 961 0 -0.5
    //--.- --.- -- --.- -- --.- -- --.- 3-- --.- -- --.- -- --.- --
    //&dateutc=now&baromin=30.24&tempf=42.0&humidity=65&windspeedmph=1.5
    //&windgustmph=1.5&winddir=169&dewptf=31.1&rainin=0&dailyrainin=0.01
    //&UV=0&indoortempf=69.9&indoorhumidity=52

    // dateutc=now        [actual]
    // &baromin=30.24     [inHG]    ==  1008 hPa    [hPa]  =  [inHg] /0.030
    // &temf=42.0         [F]       ==  5,5  °C     [C]    =  5/9 *([F] - 32)
    // &humidity=65       [%]
    // &windspeedmph=1.5  [mph]     ==  2.41 km/h   [km/h] =  [mph] * 0.44704
    // &windgustmph=1.5   [mph]     ==  2.41 km/h
    // &winddir=169       [deg]
    // &dewptf= 31.1      [F]       ==  -0,5 °C     [C]    =  5/9  * ([F] - 32)
    // &rainin=0.01       [Inch]    ==  0.25 mm     [mm]   =  [in] *  25.40
    // &UV= 0             [Index]
    // &indoortempf=69.9  [F]       ==  21,05°C
    // &indoorhumidity=52 [%]


    println!("data1: {}", String::from_utf8_lossy(&data));
    println!("data2: {}", String::from_utf8_lossy(&data2));

    println!("data len: {}", data.len());
    println!("data2 len: {}", data2.len());
}
