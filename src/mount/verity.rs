use devicemapper::{DM, DevId, DmName, DmOptions};

use crate::error::CoreError;

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