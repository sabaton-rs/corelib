use std::{path::Path, io::Read};

use devicemapper::{DM, DevId, DmName, DmOptions};
use sabaton_hal::verity::VerityPartitionHeader;

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

