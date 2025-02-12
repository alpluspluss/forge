use std::env::consts::{OS, ARCH};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
}

#[derive(Debug, Clone)]
pub enum Architecture {
    X86_64,
    AArch64,
}

impl Platform {
    pub fn current() -> Self {
        match OS {
            "windows" => Platform::Windows,
            "linux" => Platform::Linux,
            "macos" => Platform::MacOS,
            _ => panic!("Unsupported platform: {}", OS)
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Platform::Windows => ".exe",
            _ => ""
        }
    }

    pub fn path_separator(&self) -> char {
        match self {
            Platform::Windows => '\\',
            _ => '/'
        }
    }

    pub fn default_compiler(&self) -> &str {
        match self {
            Platform::Windows => "cl.exe",
            _ => "g++"
        }
    }

    pub fn normalize_path(&self, path: &Path) -> PathBuf {
        let path_str = path.to_string_lossy();
        PathBuf::from(path_str.replace(['/', '\\'], &self.path_separator().to_string()))
    }
}

impl Architecture {
    pub fn current() -> Self {
        match ARCH {
            "x86_64" => Architecture::X86_64,
            "aarch64" => Architecture::AArch64,
            _ => panic!("Unsupported architecture: {}", ARCH)
        }
    }
}