// Standard errors

use thiserror::Error;

#[derive(Debug)]
pub struct ECode(i32);

impl ECode {
    pub fn from_raw_error(error:i32) -> Self {
        ECode(error)
    }
}

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid Argument")]
    InvalidArgument,
    #[error("Input out of range")]
    InputOutOfRange,
    #[error("Not Implemented")]
    NotImplemented,
    #[error("Error code")]
    ErrorCode(ECode),
    #[error("unknown error")]
    Unknown,
}