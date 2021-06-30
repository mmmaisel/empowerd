use nix::ioctl_none;
use std::{fs::OpenOptions, os::unix::io::AsRawFd};
use udev::{Device, Enumerator};

// see /usr/include/linux/usbdevice_fs.h
const USBDEVFS_MAGIC: u8 = b'U';
const USBDEVFS_RESET: u8 = 20;

ioctl_none!(usbdevfs_reset, USBDEVFS_MAGIC, USBDEVFS_RESET);

pub fn reset_path(path: &str) -> Result<(), String> {
    let device = find_device(|device| {
        if let Some(devnode) = device.devnode() {
            if let Some(devpath) = devnode.to_str() {
                devpath == path
            } else {
                false
            }
        } else {
            false
        }
    })?;

    return reset_device(device);
}

pub fn reset_vid_pid(vid: u16, pid: u16) -> Result<(), String> {
    let vid = format!("{:04x}", vid);
    let pid = format!("{:04x}", pid);

    let device = find_device(|device| {
        if let Some(dvid) = device.property_value("ID_VENDOR_ID") {
            if let Some(dpid) = device.property_value("ID_MODEL_ID") {
                dvid.to_string_lossy() == vid && dpid.to_string_lossy() == pid
            } else {
                false
            }
        } else {
            false
        }
    })?;

    return reset_device(device);
}

fn find_device<F: Fn(&Device) -> bool>(
    cb: F,
) -> Result<Option<Device>, String> {
    let mut enumerator =
        Enumerator::new().map_err(|e| format!("Enumerate failed: {}", e))?;

    let device = enumerator
        .scan_devices()
        .map_err(|e| format!("Scan devices failed: {}", e))?
        .find(cb);
    return Ok(device);
}

fn reset_device(mut device: Option<Device>) -> Result<(), String> {
    if device.is_none() {
        return Err("No device found".into());
    }

    while let Some(dev) = device {
        if let Some(driver) = dev.driver() {
            if driver == "usb" {
                let devnode = match dev.devnode() {
                    Some(x) => x,
                    None => {
                        return Err(format!("USB parent device has no node"))
                    }
                };

                return match OpenOptions::new().write(true).open(&devnode) {
                    Ok(devfile) => unsafe {
                        if let Err(e) = usbdevfs_reset(devfile.as_raw_fd()) {
                            Err(format!("Reset USB device failed: {}", e))
                        } else {
                            Ok(())
                        }
                    },
                    Err(e) => Err(format!(
                        "Could not open USB device node \"{}\": {}",
                        &devnode.display(), e
                    )),
                };
            }
        }
        device = dev.parent();
    }

    return Err("No parent USB device found".into());
}
