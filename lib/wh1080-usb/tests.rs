extern crate hidapi;

use hidapi::HidApi;

//#[test]
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

use std::{slice, time::Duration};

use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Result, TransferType, UsbContext,
};

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

#[test]
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
        Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Interrupt),
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
            let buf =
                unsafe { slice::from_raw_parts_mut((&mut vec[..]).as_mut_ptr(), vec.capacity()) };

            let timeout = Duration::from_secs(5);

            match transfer_type {
                TransferType::Interrupt => {
                    match handle.read_interrupt(endpoint.address, buf, timeout) {
                        Ok(len) => {
                            unsafe { vec.set_len(len) };
                            println!(" - read: {:?}", vec);
                        }
                        Err(err) => println!("could not read from endpoint: {}", err),
                    }
                }
                TransferType::Bulk => match handle.read_bulk(endpoint.address, buf, timeout) {
                    Ok(len) => {
                        unsafe { vec.set_len(len) };
                        println!(" - read: {:?}", vec);
                    }
                    Err(err) => println!("could not read from endpoint: {}", err),
                },
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
