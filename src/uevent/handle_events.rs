/*
   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

      http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use std::{
    fs::Permissions,
    io::Error,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::uevent::{Action, UEvent};
use nix::sys::stat::{mknod, mode_t, Mode, SFlag};
use nix::{
    sys::stat::makedev,
    unistd::{Gid, Uid},
};

// Functions to handle UEvent.

pub fn handle_uevent<P>(event: &UEvent) -> Result<(), std::io::Error>
where
    P: pal::permissions::DefaultAttributes,
{
    match event.action {
        Action::Unknown => panic!("Unknown action"),
        Action::Add => handle_add::<P>(event),
        Action::Change => todo!(),
        Action::Remove => todo!(),
    }
}

/// Add a device entry
/// Here is an example of a device entry for a block device.
/// /devices/platform/4010000000.pcie/pci0000:00/0000:00:02.0/virtio1/block/vda/vda6
pub fn handle_add<P>(event: &UEvent) -> Result<(), std::io::Error>
where
    P: pal::permissions::DefaultAttributes,
{
    assert_eq!(event.action, Action::Add);

    //Ignore if not a device entry
    if event.maybe_major.is_none() || event.maybe_minor.is_none() {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "Not a device"));
    }

    match event.maybe_subsystem.as_ref().unwrap().as_str() {
        "block" => {
            let mut device_path = PathBuf::new();
            device_path.push("/dev/block");
            let mut device_name = PathBuf::new();
            device_name.push(event.dev_path.clone());

            device_path.push(device_name.as_path().file_name().unwrap());

            let link_by_name = if let Some(name) = event.maybe_partitionname.as_ref() {
                let mut link_name = PathBuf::new();
                link_name.push("/dev/block/by-name");
                link_name.push(name);
                Some(link_name)
            } else {
                None
            };

            let attrs = P::get_file_attributes(&device_path);
            create_device(
                &device_path,
                attrs.mode,
                attrs.owner,
                attrs.group,
                event.get_dev_type(),
                event.maybe_major.unwrap(),
                event.maybe_minor.unwrap(),
            )?;
            if let Some(link) = link_by_name {
                create_links(
                    &device_path,
                    &vec![&link],
                    attrs.owner,
                    attrs.group,
                    attrs.mode,
                )?;
            }

            Ok(())
        }
        "usb" => {
            todo!()
        }
        "net" => {
            todo!()
        }
        any => {
            log::debug!("Ignoring unknown subsystem : {}", any);
            Ok(())
        }
    }
}

fn create_dir_if_needed(
    dev_path: &Path,
    uid: libc::uid_t,
    gid: libc::gid_t,
    mode: mode_t,
) -> Result<(), std::io::Error> {
    for p in dev_path.parent().unwrap().ancestors() {
        if !p.exists() {
            //create the directory
            std::fs::create_dir(p)?;
            std::fs::set_permissions(p, Permissions::from_mode(mode))?;
            nix::unistd::chown(p, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid))).map_err(
                |_e| {
                    std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "Unable to change permission",
                    )
                },
            )?;
        }
    }
    Ok(())
}

fn create_device(
    dev_path: &Path,
    mode: mode_t,
    uid: libc::uid_t,
    gid: libc::gid_t,
    kind: SFlag,
    major: u64,
    minor: u64,
) -> Result<(), std::io::Error> {
    create_dir_if_needed(dev_path, uid, gid, mode)?;
    if !dev_path.exists() {
        mknod(
            dev_path,
            kind,
            Mode::from_bits(mode).unwrap(),
            makedev(major, minor),
        )
        .map_err(|_e| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Unable to change permission",
            )
        })
    } else {
        Ok(())
        //Err(Error::new(std::io::ErrorKind::AlreadyExists, "Device already exists"))
    }
}

fn create_links(
    dev_path: &Path,
    links: &Vec<&Path>,
    uid: libc::uid_t,
    gid: libc::gid_t,
    mode: libc::mode_t,
) -> Result<(), std::io::Error> {
    for link in links {
        if !link.exists() {
            create_dir_if_needed(link, uid, gid, mode)?;
            std::os::unix::fs::symlink(dev_path, link)?;
        }
    }
    Ok(())
}
