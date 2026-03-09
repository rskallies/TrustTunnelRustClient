use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Protocol error: {0}")]
    Protocol(&'static str),
}

// Required so GuiError can be returned from Tauri commands.
impl serde::Serialize for GuiError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
