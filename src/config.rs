use crate::error::{ForgeError, ForgeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub build: BuildConfig,
    pub paths: PathConfig,
    pub compiler: CompilerConfig,
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    #[serde(default)]
    pub cross: Option<CrossConfig>,
    #[serde(default)]
    pub profiles: HashMap<String, BuildProfile>,
    #[serde(default)]
    pub testing: Option<TestConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BuildConfig {
    pub compiler: String,
    pub target: String,
    #[serde(default)]
    pub jobs: Option<usize>,
    #[serde(default = "default_profile")]
    pub default_profile: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathConfig {
    #[serde(default)]
    pub src: String,
    #[serde(default = "default_include_paths")]
    pub include: Vec<String>,
    #[serde(default = "default_build_path")]
    pub build: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompilerConfig {
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub definitions: HashMap<String, String>,
    #[serde(default)]
    pub warnings_as_errors: bool,
    #[serde(default)]
    pub library_paths: Vec<String>,
    #[serde(default)]
    pub libraries: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct WorkspaceConfig {
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CrossConfig {
    pub target: String,
    pub toolchain: Option<String>,
    pub sysroot: Option<PathBuf>,
    #[serde(default)]
    pub extra_flags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BuildProfile {
    pub opt_level: String,
    pub debug_info: bool,
    pub lto: bool,
    #[serde(default)]
    pub extra_flags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    #[serde(default = "default_test_patterns")]
    pub patterns: Vec<String>,
    pub test_dir: Option<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub libs: Vec<String>,
    pub main: Option<String>,
}

fn default_profile() -> String {
    "debug".to_string()
}

fn default_include_paths() -> Vec<String> {
    vec!["include".to_string()]
}

fn default_build_path() -> String {
    "build".to_string()
}

fn default_test_patterns() -> Vec<String> {
    vec!["*_test.cpp".to_string(), "test_*.cpp".to_string()]
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            src: String::new(),
            include: default_include_paths(),
            build: default_build_path(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> ForgeResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ForgeError::Config(format!("Failed to read config: {}", e)))?;

        let mut config: Config = toml::from_str(&content)
            .map_err(|e| ForgeError::Config(format!("Failed to parse config: {}", e)))?;

        if !config.profiles.contains_key(&config.build.default_profile) {
            config.profiles.insert(
                config.build.default_profile.clone(),
                BuildProfile {
                    opt_level: "0".to_string(),
                    debug_info: true,
                    lto: false,
                    extra_flags: vec![],
                },
            );
        }

        Ok(config)
    }

    pub fn default_for_member(name: &str) -> Self {
        let mut config = Config {
            build: BuildConfig {
                compiler: "g++".to_string(),
                target: name.to_string(),
                jobs: None,
                default_profile: "debug".to_string(),
            },
            paths: PathConfig::default(),
            compiler: CompilerConfig {
                flags: vec!["-Wall".to_string(), "-std=c++17".to_string()],
                definitions: HashMap::new(),
                warnings_as_errors: false,
                library_paths: vec![],
                libraries: vec![],
            },
            workspace: WorkspaceConfig::default(),
            cross: None,
            profiles: HashMap::new(),
            testing: Some(TestConfig {
                patterns: default_test_patterns(),
                test_dir: None,
                exclude: vec![],
                flags: vec![],
                libs: vec![],
                main: None,
            }),
        };

        config.profiles.insert("debug".to_string(), BuildProfile {
            opt_level: "0".to_string(),
            debug_info: true,
            lto: false,
            extra_flags: vec![],
        });
        config.profiles.insert("release".to_string(), BuildProfile {
            opt_level: "3".to_string(),
            debug_info: false,
            lto: true,
            extra_flags: vec!["-march=native".to_string()],
        });

        config
    }

    pub fn get_profile(&self, name: Option<&str>) -> Option<&BuildProfile> {
        name.map_or_else(
            || self.profiles.get(&self.build.default_profile),
            |n| self.profiles.get(n),
        )
    }
}