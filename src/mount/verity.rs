use devicemapper::{DM, DevId, DmName, DmOptions};

use crate::error::CoreError;

pub fn load_dm() -> Result<(), CoreError> {
    log::info!("load_dm");
    let dm = DM::new().unwrap();
    let (maj,min,patch) = dm.version()
        .map_err(|e|{
            log::error!("Unable to get DM version: {}", e);
            CoreError::DMError
        })?; 
        log::info!("DM version:{} {} {}", maj,min,patch);
    Ok(())
}