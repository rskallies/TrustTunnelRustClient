use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(&'static str),

    #[error("Invalid argument: {0}")]
    InvalidArgument(&'static str),

    #[error("Windows service error: {0}")]
    WindowsService(#[from] windows_service::Error),
}
