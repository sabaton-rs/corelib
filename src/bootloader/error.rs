use thiserror::Error;

#[derive(Error, Debug)]
pub enum BootloaderMessageError {
    #[error("Priority out of range")]
    PriorityOutOfRange,
    
}