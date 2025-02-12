use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub build: BuildConfig,
    pub paths: PathConfig,
    pub compiler: CompilerConfig,
    #[serde(default)]
    pub workspace: WorkspaceConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BuildConfig {
    pub compiler: String,
    pub target: String,
    pub jobs: Option<usize>,
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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct WorkspaceConfig {
    pub members: Vec<String>,
    pub exclude: Vec<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }

    pub fn default_for_member(name: &str) -> Self {
        Self {
            build: BuildConfig {
                compiler: "g++".to_string(),
                target: name.to_string(),
                jobs: None,
            },
            paths: PathConfig {
                src: "src".to_string(),
                include: "include".to_string(),
                build: "build".to_string(),
            },
            compiler: CompilerConfig {
                flags: vec!["-Wall".to_string(), "-std=c++17".to_string()],
                definitions: HashMap::new(),
                warnings_as_errors: false,
            },
            workspace: WorkspaceConfig::default(),
        }
    }
}