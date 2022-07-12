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

use libc::c_ulong;
use log::{debug, trace};
use std::{
    ffi::{CStr, CString},
    io::Error,
    path::PathBuf,
};

use std::str::FromStr;

/// FsManagerFlags
#[derive(Debug, Clone)]
pub enum FsManagerFlags {
    /// Mount this partition during early boot
    FirstStageMount,
    /// Use slot select mechanism to decide which partition to
    /// mount. The bootmanager HAL is used to get details about
    /// the slot.
    SlotSelect,
    /// This is a logical partition (using DM Mapper. Not supported yet)
    Logical,
    /// This fs is protected with metadata in the verity partition.
    Verity,
    /// Other flags
    Other(String),
}

impl FromStr for FsManagerFlags {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "slotselect" => Ok(FsManagerFlags::SlotSelect),
            "first_stage_mount" => Ok(FsManagerFlags::FirstStageMount),
            "verity" => Ok(FsManagerFlags::Verity),
            "logical" => Ok(FsManagerFlags::Logical),
            _ => Ok(FsManagerFlags::Other(String::from(s))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FsEntry {
    /// The device identifier
    pub fs_spec: CString,
    /// The mount point
    pub mountpoint: CString,
    /// Which filesystem type it is
    pub vfs_type: CString,
    /// Mount options to use. Directly store in the flags format
    pub mount_options: libc::c_ulong,
    /// Filessytem manager flags for special handling of each
    /// mount. For example, if a partition is affected
    /// by the dual partition scheme, then the slotselect flag must be set.
    pub fs_manager_flags: Vec<FsManagerFlags>,
}

impl FsEntry {
    pub fn parse_entries(contents: &str, slot_suffix: &str) -> Result<Vec<FsEntry>, Error> {
        let mut entries: Vec<FsEntry> = Vec::new();
        //let mut contents = String::new();
        //file.read_to_string(&mut contents)?;

        for line in contents.lines() {
            if line.starts_with("#") {
                trace!("Skipping commented line: {}", line);
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 5 {
                trace!("Unknown fstab entry: {}", line);
                continue;
            }

            let flags: Vec<FsManagerFlags> = parts[4]
                .split(",")
                .map(|s| FsManagerFlags::from_str(s).unwrap())
                .collect();

            let fs_spec = if flags
                .iter()
                .find(|f| matches!(f, FsManagerFlags::SlotSelect))
                .is_some()
            {
                let full_spec = format!("{}_{}", parts[0], slot_suffix);
                CString::new(full_spec).unwrap()
            } else {
                CString::new(parts[0]).unwrap()
            };

            let mut mount_options: libc::c_ulong = 0;
            for p in parts[3].split(",") {
                mount_options |= Self::get_mount_option(p);
            }

            let entry = FsEntry {
                fs_spec,
                mountpoint: CString::new(parts[1]).unwrap(),
                vfs_type: CString::new(parts[2]).unwrap(),
                mount_options,
                fs_manager_flags: flags,
            };
            entries.push(entry)
        }
        Ok(entries)
    }

    fn get_mount_option(option: &str) -> libc::c_ulong {
        match option {
            "ro" => libc::MS_RDONLY,
            "rw" => 0, // default is read/write so nothing to do here
            "dirsync" => libc::MS_DIRSYNC,
            "lazytime" => libc::MS_LAZYTIME,
            "mandlock" => libc::MS_MANDLOCK,
            "noatime" => libc::MS_NOATIME,
            "nodev" => libc::MS_NODEV,
            "nodiratime" => libc::MS_NODIRATIME,
            "noexec" => libc::MS_NOEXEC,
            "nosiud" => libc::MS_NOSUID,
            "silent" => libc::MS_SILENT,
            "strictatime" => libc::MS_STRICTATIME,
            "sync" => libc::MS_SYNC as c_ulong,
            _ => 0,
        }
    }

    pub fn is_first_stage_mount(&self) -> bool {
        for flag in self.fs_manager_flags.iter() {
            if let FsManagerFlags::FirstStageMount = flag {
                return true;
            }
        }
        false
    }

    pub fn is_slot_selected(&self) -> bool {
        for flag in self.fs_manager_flags.iter() {
            if let FsManagerFlags::SlotSelect = flag {
                return true;
            }
        }
        false
    }

    pub fn is_logical(&self) -> bool {
        for flag in self.fs_manager_flags.iter() {
            if let FsManagerFlags::Logical = flag {
                return true;
            }
        }
        false
    }

    pub fn is_verity_protected(&self) -> bool {
        for flag in self.fs_manager_flags.iter() {
            if let FsManagerFlags::Verity = flag {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_construct() {
        let fstab = r###"#fstab for initrd. 
#<dev>                          <mnt_point>     <type>  <mnt_flags options> <fs_mgr_flags>
# system partition must be mounted as root.
/dev/block/by-name/system           /           ext2    ro,noauto,nouser    slotselect,first_stage_mount,verity
/dev/block/by-name/vendor           /vendor     ext2    ro,noauto,nouser    slotselect,first_stage_mount
/dev/block/by-name/data             /data       ext2    rw,noauto,nouser    first_stage_mount
"###;

        let _entries = FsEntry::parse_entries(&fstab, "a");
    }
}
