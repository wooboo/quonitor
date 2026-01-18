use serde::{Serialize, Deserialize};

#[derive(Debug, thiserror::Error)]
pub enum QuonitorError {
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    Config(String),
}

impl Serialize for QuonitorError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, QuonitorError>;
