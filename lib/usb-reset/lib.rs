use nix::ioctl_none;
use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    os::unix::io::AsRawFd,
    path::Path,
};
use udev::{Device, Enumerator};

// see /usr/include/linux/usbdevice_fs.h
const USBDEVFS_MAGIC: u8 = b'U';
const USBDEVFS_RESET: u8 = 20;

ioctl_none!(usbdevfs_reset, USBDEVFS_MAGIC, USBDEVFS_RESET);

pub fn reset_device(path: &str) -> Result<(), String> {
    let mut enumerator = Enumerator::new().map_err(|e| e.to_string())?;

    let mut device = enumerator
        .scan_devices()
        .map_err(|e| e.to_string())?
        .find(|device| {
            if let Some(devnode) = device.devnode() {
                if let Some(devpath) = devnode.to_str() {
                    devpath == path
                } else {
                    false
                }
            } else {
                false
            }
        });

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
                    Err(e) => {
                        Err(format!("Could not open USB device node: {}", e))
                    }
                };
            }
        }
        device = dev.parent();
    }

    return Err("No parent USB device found".into());
}
