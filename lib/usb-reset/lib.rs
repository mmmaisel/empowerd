/******************************************************************************\
    empowerd - empowers the offline smart home
    Copyright (C) 2019 - 2021 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/
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
                        &devnode.display(),
                        e
                    )),
                };
            }
        }
        device = dev.parent();
    }

    return Err("No parent USB device found".into());
}
