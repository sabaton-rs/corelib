use std::{path::Path, io::Read};

use devicemapper::{DM, DevId, DmName, DmOptions, DmFlags};
use nix::ioctl_read;
use sabaton_hal::verity::VerityPartitionHeader;
use thiserror::private::PathAsDisplay;

use crate::error::CoreError;

pub struct Dm
{
    dm : DM,
    partition_header : VerityPartitionHeader,
}

impl Dm {
    pub fn new(verity_device_path:&Path) -> Result<Self, CoreError> {
        // try to open DM first
        let dm = DM::new()
            .map_err(|e|{
                log::error!("Error opening DM {}", e);
                CoreError::DMError
            })?;

        // attempt to open the verity device and read the header
        let mut file_handle = std::fs::OpenOptions::new().read(true).open(verity_device_path)
            .map_err(|e| {
                log::error!("Unable to open vbmeta partition : {} due to {}", verity_device_path.display(), e);
                CoreError::DMError
            })?;
        
        // read the public key
        let public_key = std::fs::read("/etc/veritykey.pub")
            .map_err(|e|{
                log::error!("Unable to open /etc/veritykey.pub due to {}",e);
                CoreError::DMError
            })?;

        // now read the verity header.  Read 1K length for now, 
        //TODO: better api to pass readable into verityheader

        let mut buffer = [0; 1024];
        file_handle.read(&mut buffer).map_err(|e|{
            log::error!("Unable to read from {} due to {}",verity_device_path.display(),e);
                CoreError::DMError
        })?;

        let partition_header = VerityPartitionHeader::create_from(&buffer, &public_key)
            .map_err(|e| {
                log::error!("Cannot create verity partition header: {}", e);
                CoreError::DMError
            })?;

        Ok(Self {
            dm,
            partition_header,
        })
        
    }

    pub fn create_dm_device(&self, protected_partition_from_fstab:&Path, verity_partition : &Path,name : &str) -> Result<(), CoreError> {

        let protected_partition = protected_partition_from_fstab.canonicalize()
            .map_err(|e| {
                log::error!("Canonicalize {}", protected_partition_from_fstab.display());
                CoreError::InvalidArgument
            })?;
        
        let verity_partition = verity_partition.canonicalize()
            .map_err(|e| {
                log::error!("Canonicalize {}", verity_partition.display());
                CoreError::InvalidArgument
            })?;

        let dm_name = DmName::new(name)
            .map_err(|e|{
                log::error!("Create DMName");
                CoreError::DMError
            })?;

        // create the device
        let _device = self.dm.device_create(dm_name, None, DmOptions::default())
            .map_err(|e|{
                log::error!("Cannot create device");
                CoreError::DMError
            })?;

        let protected_partition_name = protected_partition_from_fstab.file_name().unwrap();
        let name = Path::new(protected_partition_name);
        let table_entry = self.partition_header.get_entry(name).ok_or(CoreError::DMPartition)
            .map_err(|e| {
                log::error!("Cannot get entry for {}", name.display());
                CoreError::DMError
            })?;

        let partition_size_bytes = get_device_size(&protected_partition);
        let num_blocks = partition_size_bytes / table_entry.data_block_size as u64;
        log::info!("{} has {} blocks", protected_partition.display(), num_blocks);


        let verity_table_string = format!("{} {} {} {} {} {} {} {} {} {}",
            1, // version 
            protected_partition.display(),
            verity_partition.display(),
            table_entry.data_block_size,
            table_entry.hash_block_size,
            table_entry.num_blocks,
            table_entry.hash_start,
            table_entry.algorithm,
            hex::encode(table_entry.digest),
            hex::encode(table_entry.salt),
        );
        
        log::info!("dm :{}", &verity_table_string);

        let table = vec![(
            0u64,
            partition_size_bytes as u64,
            "verity".into(),
            verity_table_string,
        )];

        let id = DevId::Name(dm_name);
        let r = self.dm.table_load(&id, &table, DmOptions::default().set_flags(DmFlags::DM_READONLY))
            .map_err(|e|{
                log::error!("Error loading DM table : {}", e);
                CoreError::DMError
            });

        let r = self.dm.device_suspend(&id, DmOptions::default())
            .map_err(|e|{
                log::error!("Error resuming device");
                CoreError::DMError
            });

        Ok(())
    }
}

pub fn load_dm() -> Result<(), CoreError> {
    log::info!("load_dm");
    if let Ok(dm) = DM::new() {
    let (maj,min,patch) = dm.version()
        .map_err(|e|{
            log::error!("Unable to get DM version: {}", e);
            CoreError::DMError
        })?; 
        log::info!("DM version:{} {} {}", maj,min,patch);
    } else {
        log::error!("DM Init failed");
    }
    Ok(())
}


use std::os::unix::io::AsRawFd;
use std::fs::OpenOptions;

// Generate ioctl function
const BLKGETSIZE64_CODE: u8 = 0x12; // Defined in linux/fs.h
const BLKGETSIZE64_SEQ: u8 = 114;
ioctl_read!(ioctl_blkgetsize64, BLKGETSIZE64_CODE, BLKGETSIZE64_SEQ, u64);

/// Determine device size
fn get_device_size(path: &Path) -> u64 {
   let file = OpenOptions::new()
             .write(true)
             .open(path).unwrap();

   let fd = file.as_raw_fd();

   let mut cap = 0u64;
   let cap_ptr = &mut cap as *mut u64;

   unsafe {
      ioctl_blkgetsize64(fd, cap_ptr).unwrap();
   }
  
   cap
}
