use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub build: BuildConfig,
    pub paths: PathConfig,
    pub compiler: CompilerConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BuildConfig {
    pub compiler: String,
    pub target: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathConfig {
    pub src: String,
    pub include: String,
    pub build: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompilerConfig {
    pub flags: Vec<String>,
    pub definitions: HashMap<String, String>,
    pub warnings_as_errors: bool,
}

impl Config {
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }
}