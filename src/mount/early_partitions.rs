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
    ffi::{CStr, CString},
    io::Error,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use crate::{fstab::*, mount::verity::Dm};
use sabaton_hal::bootloader::BootControl;
use crate::uevent::{*};

pub const VBMETA_PARTITION_NAME_WITHOUT_SUFFIX : &str = "/dev/block/by-name/vbmeta"; 

macro_rules! c_str {
    ($s:expr) => {{
        concat!($s, "\0").as_ptr() as *const std::os::raw::c_char
    }};
}

/// The location of the fstab
pub const FSTAB_LOCATION: &str = "/etc/fstab";

fn should_prepare_verity(fstab_entries : &[FsEntry]) -> bool {
    for entry in fstab_entries {
        if entry.is_verity_protected() && entry.is_first_stage_mount() {
            return true
        }
    }
    false
}

/// Mount all the partitions that are marked for early mount
pub fn mount_early_partitions(boot_hal: &mut dyn BootControl) -> Result<(), std::io::Error> {
    let fstab_contents = std::fs::read_to_string(FSTAB_LOCATION)?;
    let root_temp_mount = CString::new("/new_root").unwrap();

    let suffix = boot_hal.partition_suffix(boot_hal.current_slot()?)?;
    let mut fstab_entries = FsEntry::parse_entries(&fstab_contents, suffix)?;

    let mut socket = create_and_bind_netlink_socket().unwrap();
    let mut next_dm_index = 0;

    
    let (mut dm, verity_partition_name) = if should_prepare_verity(&fstab_entries) {
        crate::mount::verity::load_dm().unwrap();
        // verity partition is called vbmeta_<suffix>
        let verity_partition_name = format!("{}_{}",VBMETA_PARTITION_NAME_WITHOUT_SUFFIX,suffix);
        let c_verity_partition_name = CString::new(verity_partition_name.as_str())?;
        ensure_mount_device_is_created(&c_verity_partition_name, &mut socket)
            .map_err(|e|{
                log::error!("Cannot create device for {}",verity_partition_name);
                e
            })?;
        let dm = Dm::new(Path::new(&verity_partition_name))
            .map_err(|e| {
                log::error!("DM setup error");
                std::io::Error::from(std::io::ErrorKind::Other)
            })?;
        
            log::info!("DM Open Success");
            (Some(dm), Some(PathBuf::from(verity_partition_name)))
    } else {
        (None, None)
    };
    

    log::debug!("Fstab entries:{:?}", fstab_entries);
    let root_cmp = CString::new("/").unwrap();

    if let Some(root) = fstab_entries.iter_mut().find(|e| e.mountpoint == root_cmp) {
        if !root.is_first_stage_mount() {
            log::error!("/ is not marked for first stage mount");
        } else {
            let mut count = 5;
            while count > 0 {
                if ensure_mount_device_is_created(root.fs_spec.as_c_str(), &mut socket).is_ok() { 
                    log::info!("early mount devices created");
                    break;
                } else {
                    log::info!("early mount devices not ready. will retry {} more time(s)", count);
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    count -= 1;
                }
            }
            //ensure_mount_device_is_created(root.fs_spec.as_c_str(), &mut socket)?;
            log::debug!("/dev paths created!");
            // mount the root partition, but into /mnt/system for now. We will make this the new
            // root later
            root.mountpoint = root_temp_mount.clone();
            if root.is_verity_protected() {
                let dm_device = format!("dm-{}", next_dm_index);
                next_dm_index += 1;
                create_dm_device(root, dm.as_mut().unwrap(), verity_partition_name.as_ref().unwrap(),&dm_device)?;
                let device = create_dm_device_entry(&dm_device,&mut socket)?;
                let mut e = root.clone();        
                e.fs_spec = CString::new(device.to_str().unwrap()).unwrap();
                mount_partition(&e)?;
            } else {
                mount_partition(root)?;
            }
            // switch it back so we won't attempt to mount it again
            root.mountpoint = root_cmp.clone();
            
        }
    } else {
        log::error!("Could not find '/' directory in fstab. fatal");
    }

    // Before mounting the root, we need to switch to the new root
    log::info!("Switching to new root:{:?}", &root_temp_mount);
    switch_to_new_root(&root_temp_mount)?;

    // now mount the other partitions
    for e in fstab_entries {
        // we have already mounted the root above, skip it
        if e.mountpoint == root_cmp {
            continue;
        }
        ensure_mount_device_is_created(e.fs_spec.as_c_str(), &mut socket)?;
        
        if e.is_verity_protected() {
            let dm_device = format!("dm-{}", next_dm_index);
            next_dm_index += 1;
            create_dm_device(&e, dm.as_mut().unwrap(), verity_partition_name.as_ref().unwrap(),&dm_device)?;
            let device = create_dm_device_entry(&dm_device,&mut socket)?;
            let mut e = e.clone();        
            e.fs_spec = CString::new(device.to_str().unwrap()).unwrap();
            mount_partition(&e)?;
        } else {
            mount_partition(&e)?;
        }
    }
    Ok(())
}


/// Create a device manager device entry
/// device_name indicates thethe dm device created
/// for example dm-0, dm-1, etc.
/// Returns the  path to the device that is created.
fn create_dm_device_entry(device_name: &str,mut nl_socket: &mut NLSocket) -> Result<PathBuf, std::io::Error> {

    let device = PathBuf::from(format!("/sys/block/{}",device_name));
    log::debug!("Create DM device for {}",device.display());

    
    let _action = regenerate_uevent_for_dir(&device, &mut nl_socket, &mut |e| {
        //log::debug!("Event {:?}", e);

        // look for partition name if device is searched by name
        let matched = if let Some(p_name) = e.get_devname() {
            p_name == device_name
        } else {
            false
        };

        if matched {
            handle_events::handle_uevent::<pal::permissions::DefaultImpl>(e).unwrap();
            UEventGenerateAction::Stop
        } else {
            UEventGenerateAction::Continue
        }
    });

    let device  = Path::new("/dev/block").join(device_name);

    if !device.exists() {
        log::error!("{} device entry not created", device_name);
        Err(Error::new(std::io::ErrorKind::NotFound, "path not found"))
    } else {
        Ok(device)
    }
}

/// Create the device entry for the the provided entry. The device entries can be
/// of the form  /dev/block/<name>  or /dev/block/by-name/<partition-name>
pub fn ensure_mount_device_is_created(
    fs_spec: &CStr,
    mut nl_socket: &mut NLSocket,
) -> Result<(), std::io::Error> {
    let path = Path::new(fs_spec.to_str().unwrap());
    log::debug!("ensure dev created for : {}", path.display());
    // we allow early mounting of tmpfs
    if path == Path::new("tmpfs") {
        return Ok(())
    }

    if !path.starts_with("/dev/block") {
        panic!("filesystem spec in fstab must start with /dev/block");
    }

    // return right away if the device already exists
    if path.exists() {
        return Ok(());
    }

    let mut path_components = path.components().skip(3).take(2);

    if let Some(third) = path_components.next() {
        let (device_is_by_name, device_name, search_path) = if third.as_os_str() == "by-name" {
            let device = path_components.next().expect("Expected device name");
            // if we have the device by name we need to search broader
            (true, device, PathBuf::new().join("/sys/class/block"))
        } else {
            // third is the device name
            // if we have the device name, narrow the search down to the provided device
            (false, third, Path::new("/sys/class/block").join(&third))
        };

        //println!("Going to regen for {}", &search_path.display());

        let _action = regenerate_uevent_for_dir(&search_path, &mut nl_socket, &mut |e| {
            log::debug!("Event {:?}", e);

            // look for partition name if device is searched by name
            let matched = if device_is_by_name {
                if let Some(p_name) = e.get_partition_name() {
                    p_name == device_name.as_os_str()
                } else {
                    false
                }
            } else if let Some(p_name) = e.get_devname() {
                p_name == device_name.as_os_str()
            } else {
                false
            };

            if matched {
                handle_events::handle_uevent::<pal::permissions::DefaultImpl>(e).unwrap();
                UEventGenerateAction::Stop
            } else {
                UEventGenerateAction::Continue
            }
        });

        if device_is_by_name {
            if !Path::new("/dev/block/by-name").join(&device_name).exists() { 
                return Err(Error::new(std::io::ErrorKind::NotFound, "path not found"));
            }
        } else if !Path::new("/dev/block").join(&device_name).exists() {
            return Err(Error::new(std::io::ErrorKind::NotFound, "path not found"));
        }
        Ok(())
    } else {
        Err(Error::new(std::io::ErrorKind::NotFound, "path not found"))
    }
}

// Mount a verity protected partition
fn create_dm_device(entry:&FsEntry, dm : &mut Dm, verity_partition: &Path, name: &str) -> Result<(), std::io::Error> {

    let protected_partition = Path::new(entry.fs_spec.to_str().unwrap());
    //let name = protected_partition.file_name().unwrap();
    //let name = format!("verified-{}",name.to_str().unwrap());
    //let name = format!("dm-{}","0");
    dm.create_dm_device(Path::new(&entry.fs_spec.to_str().unwrap()), verity_partition, name)
        .map_err(|e|{
            std::io::Error::from(std::io::ErrorKind::PermissionDenied)
        })?;

    Ok(())
    //let mut e = entry.clone();

    //let verified_device_path = format!("/dev/mapper/{}",&name);

    //e.fs_spec = CString::new(verified_device_path.as_str()).unwrap();

    //mount_partition(&e)
}

fn mount_partition(entry: &FsEntry) -> Result<(), std::io::Error> {
    log::debug!(
        "Going to mount {:?} to {:?} type:{:?}",
        &entry.fs_spec, &entry.mountpoint, &entry.vfs_type
    );

    
    let ret = unsafe {
        libc::mount(
            entry.fs_spec.as_ptr(),
            entry.mountpoint.as_ptr(),
            entry.vfs_type.as_ptr(),
            entry.mount_options,
            std::ptr::null_mut(),
        )
    };

    if ret == 0 {
        log::debug!("Mount success:{}", ret);
        Ok(())
    } else {
        log::error!("Mount failed:{}", ret);
        Err(Error::from_raw_os_error(unsafe {
            *libc::__errno_location()
        }))
    }
}

/// Switch to the new root file-system. Move all existing mounts
/// into the new root
fn switch_to_new_root(new_root: &CStr) -> Result<(), std::io::Error> {
    let root_str = new_root.to_str().unwrap();
    // get existing mounts and move them
    for mount in get_all_mounts(new_root) {
        let new_mount_path = Path::new(root_str).join(mount.to_str().unwrap().trim_start_matches('/'));
        log::debug!("New move path:{}", &new_mount_path.display());
        let mut buf = Vec::new();
        buf.extend(new_mount_path.as_os_str().as_bytes());
        buf.push(0);
        //let res = unsafe { libc::mkdir(buf.as_ptr() as *const libc::c_char, 0755) };
        //if res != 0 {
        //    // ok to fail here if the directory already exists
        //    log::debug!("mkdir failed:{} error:{}",new_mount_path.display(), unsafe {*libc::__errno_location()});
        //} 
        

        let res = unsafe {
            libc::mount(
                mount.as_ptr(),
                buf.as_ptr() as *const libc::c_char,
                std::ptr::null(),
                libc::MS_MOVE,
                std::ptr::null(),
            )
        };

        if res != 0 {
            log::error!("Unable to move {:?} mount to {}:{}", mount, &new_mount_path.display(), unsafe{*libc::__errno_location()});
        } else {
            log::debug!("Moved {:?} to {:?}", mount, &new_mount_path);
        }
    }

    let res = unsafe { libc::chdir(new_root.as_ptr()) };
    if res != 0 {
        log::error!("Unable to chdir to new root {:?}", &new_root);
    } else {
        log::debug!("Chdir to new root {:?}", &new_root);
    }

    let res = unsafe {
        libc::mount(
            new_root.as_ptr(),
            c_str!("/"),
            std::ptr::null(),
            libc::MS_MOVE,
            std::ptr::null(),
        )
    };
    if res != 0 {
        log::error!("Unable to move {:?} mount to /", &new_root);
    }

    let res = unsafe { libc::chroot(c_str!(".")) };
    if res != 0 {
        log::error!("Unable to chroot");
    }

    Ok(())
}

/// Helper function for switching root. Get the the current mounts
/// that need to be moved to the new root
fn get_all_mounts(skip: &CStr) -> Vec<CString> {
    let file = unsafe { libc::setmntent(c_str!("/proc/mounts"), c_str!("re")) };
    if file.is_null() {
        panic!("Unable to open /proc/mounts");
    }

    let mut res: Vec<CString> = Vec::new();
    let root = unsafe { CStr::from_ptr(c_str!("/")) };

    let mut mentry;

    'outer: loop {
        mentry = unsafe { libc::getmntent(file) };
        if mentry.is_null() {
            break;
        } else {
            let mentry = unsafe { &*mentry };

            let mnt_dir = unsafe { CStr::from_ptr(mentry.mnt_dir) };

            // ignore the root and the one we have been asked to skip
            if mnt_dir == root || mnt_dir == skip {
                log::debug!("Skipping {:?}", mnt_dir);
                continue;
            }

            // also ignore if the new mount is within an existing mount.
            // this will get moved anyway.
            for path in res.iter() {
                if mnt_dir
                    .to_str()
                    .unwrap()
                    .starts_with(path.to_str().unwrap())
                {
                    log::debug!("Skipping sub-mount {:?}", mnt_dir);
                    continue 'outer;
                }
            }
            res.push(mnt_dir.to_owned());
        }
    }
    unsafe { libc::endmntent(file) };
    res
}
