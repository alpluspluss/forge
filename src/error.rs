use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid target triple: {0}")]
    InvalidTarget(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Build error: {0}")]
    Build(String),

    #[error("Compiler error: {0}")]
    Compiler(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

pub type ForgeResult<T> = Result<T, ForgeError>;