
pub enum TimeBaseType {
    SynchronizedMaster,
    SynchronizedSlave,
    OffsetMaster,
    OffsetSlave,
    Local,
}

pub trait TimeBaseStatus {
    fn creation_time(&self) -> std::time::Instant;
    fn update_counter(&self) -> u8;
    fn time_leap(&self) -> std::time::Duration;
    fn time_zone(&self) -> String;
}

pub trait TimeBase {
    /// Get the type of this TimeBase.
    fn get_type(&self) -> TimeBaseType;
    fn get_rate_deviation(&self) -> std::time::Duration;
    fn now(&self) -> std::time::Instant;
    fn get_time_base_status(&self) -> dyn TimeBaseStatus;
}

pub trait SynchSlaveTimeBase : TimeBase {
    fn calculate_time_diff(&self, instant: std::time::Instant) -> std::time::Duration;
}

pub trait LocalTimeBase {
    fn set_time(time: std::time::Instant) -> Result<(),std::io::Error>;
    fn update_time(time: std::time::Instant) -> Result<(),std::io::Error>;
}

pub trait SynchronizedMasterTimeBase : TimeBase {
    fn set_time(&self,time: std::time::Instant) -> Result<(),std::io::Error>;
    fn update_time(&self,time: std::time::Instant) -> Result<(),std::io::Error>;
    fn set_rate_correction(&self,deviation: std::time::Duration);
}

pub trait OffsetMasterTimeBase : TimeBase {
    fn set_offset(&self,offset: std::time::Duration);
    fn offset(&self,) -> std::time::Duration;
    fn get_synchonized_master(&self) -> Option<&dyn SynchronizedMasterTimeBase>;
    fn set_time(&self,time: std::time::Instant) -> Result<(),std::io::Error>;
    fn update_time(&self,time: std::time::Instant) -> Result<(),std::io::Error>;
    fn set_rate_correction(&self,deviation: std::time::Duration);
}

pub trait OffsetSlaveTimeBase {

}