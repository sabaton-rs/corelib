use thiserror::Error;

#[derive(Error, Debug)]
pub enum BootloaderMessageError {
    #[error("Priority out of range")]
    PriorityOutOfRange,
    #[error("CRC Error")]
    CrcFailure,
    #[error("Insufficient bytes")]
    InsufficientBytes,
    #[error("Data too long")]
    DataTooLong,
}
