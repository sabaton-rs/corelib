use std::{path::Path, io::Read};

use devicemapper::{DM, DevId, DmName, DmOptions, DmFlags};
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

    pub fn create_dm_device(&self, protected_partition:&Path, verity_partition : &Path,name : &str) -> Result<(), CoreError> {

        let dm_name = DmName::new(name)
            .map_err(|e|{
                log::error!("Create DMName");
                CoreError::DMError
            })?;

        let protected_partition_name = protected_partition.file_name().unwrap();
        let name = Path::new(protected_partition_name);
        let table_entry = self.partition_header.get_entry(name).ok_or(CoreError::DMPartition)?;

        let num_blocks = protected_partition.metadata().unwrap().len() / table_entry.data_block_size as u64;


        let verity_table_string = format!("{} {} {} {} {} {} {} {} {} {}",
            1, // version 
            protected_partition.canonicalize().unwrap().display(),
            verity_partition.canonicalize().unwrap().display(),
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
            num_blocks,
            "verity".into(),
            verity_table_string,
        )];

        let id = DevId::Name(dm_name);
        let r = self.dm.table_load(&id, &table, DmOptions::default().set_flags(DmFlags::DM_READONLY))
            .map_err(|e|{
                log::error!("Error loading DM table : {}", e);
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

