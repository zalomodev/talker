use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Audio capture error: {0}")]
    Audio(String),

    #[error("VAD error: {0}")]
    Vad(String),

    #[error("Transcription error: {0}")]
    Transcription(String),

    #[error("Translation error: {0}")]
    Translation(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Credential error: {0}")]
    Credential(String),

    #[error("Settings error: {0}")]
    Settings(String),

    #[error("Window tracking error: {0}")]
    Window(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Unsupported platform or operation: {0}")]
    Unsupported(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
