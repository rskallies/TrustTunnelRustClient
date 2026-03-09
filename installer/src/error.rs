use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Windows service error: {0}")]
    Service(#[from] windows_service::Error),

    #[error("Windows API error: {0}")]
    Windows(#[from] windows::core::Error),

    #[error("{0}")]
    Other(&'static str),
}
