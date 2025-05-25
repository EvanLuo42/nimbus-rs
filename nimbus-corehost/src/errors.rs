use std::ffi;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreHostError {
    #[error("Buffer too small: ")]
    BufferTooSmall,
    #[error("Invalid UTF-8 or missing null terminator")]
    FromBytesWithNulError(#[from] ffi::FromBytesWithNulError),
    #[error("Unknown error")]
    UnknownError
}

impl From<u32> for CoreHostError {
    fn from(value: u32) -> Self {
        match value {
            0x80008098 => CoreHostError::BufferTooSmall,
            _ => CoreHostError::UnknownError
        }
    }
}
