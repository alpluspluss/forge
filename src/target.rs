use std::str::FromStr;
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::error::{ForgeError, ForgeResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Target {
    pub arch: Architecture,
    pub vendor: Vendor,
    pub os: OS,
    pub env: Environment,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    X86,
    X86_64,
    ARM,
    AArch64,
    RISCV64,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::X86 => write!(f, "i686"),
            Architecture::ARM => write!(f, "arm"),
            Architecture::AArch64 => write!(f, "aarch64"),
            Architecture::RISCV64 => write!(f, "riscv64"),
            Architecture::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Vendor {
    Unknown,
    PC,
    Apple,
    #[serde(other)]
    Other,
}

impl fmt::Display for Vendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Vendor::Unknown => write!(f, "unknown"),
            Vendor::PC => write!(f, "pc"),
            Vendor::Apple => write!(f, "apple"),
            Vendor::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OS {
    Linux,
    Windows,
    Darwin,
    None,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for OS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OS::Linux => write!(f, "linux"),
            OS::Windows => write!(f, "windows"),
            OS::Darwin => write!(f, "darwin"),
            OS::None => write!(f, "none"),
            OS::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment {
    GNU,
    MSVC,
    Musl,
    None,
    #[serde(other)]
    Unknown,
}

impl FromStr for Target {
    type Err = ForgeError;

    fn from_str(s: &str) -> ForgeResult<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() < 3 {
            return Err(ForgeError::InvalidTarget("Invalid target triple".to_string()));
        }

        let arch = match parts[0] {
            "x86_64" => Architecture::X86_64,
            "i686" => Architecture::X86,
            "aarch64" => Architecture::AArch64,
            "arm" => Architecture::ARM,
            "riscv64" => Architecture::RISCV64,
            _ => return Err(ForgeError::InvalidTarget(format!("Unknown architecture: {}", parts[0]))),
        };

        let vendor = match parts[1] {
            "pc" => Vendor::PC,
            "unknown" => Vendor::Unknown,
            "apple" => Vendor::Apple,
            _ => Vendor::Other,
        };

        let os = match parts[2] {
            "linux" => OS::Linux,
            "windows" => OS::Windows,
            "darwin" => OS::Darwin,
            "none" => OS::None,
            _ => OS::Unknown,
        };

        let env = if parts.len() > 3 {
            match parts[3] {
                "gnu" => Environment::GNU,
                "msvc" => Environment::MSVC,
                "musl" => Environment::Musl,
                _ => Environment::Unknown,
            }
        } else {
            Environment::None
        };

        Ok(Target {
            arch,
            vendor,
            os,
            env,
        })
    }
}

impl ToString for Target {
    fn to_string(&self) -> String {
        let arch = match self.arch {
            Architecture::X86_64 => "x86_64",
            Architecture::X86 => "i686",
            Architecture::AArch64 => "aarch64",
            Architecture::ARM => "arm",
            Architecture::RISCV64 => "riscv64",
            Architecture::Unknown => "unknown",
        };

        let vendor = match self.vendor {
            Vendor::PC => "pc",
            Vendor::Unknown => "unknown",
            Vendor::Apple => "apple",
            Vendor::Other => "other",
        };

        let os = match self.os {
            OS::Linux => "linux",
            OS::Windows => "windows",
            OS::Darwin => "darwin",
            OS::None => "none",
            OS::Unknown => "unknown",
        };

        let env = match self.env {
            Environment::GNU => "-gnu",
            Environment::MSVC => "-msvc",
            Environment::Musl => "-musl",
            Environment::None => "",
            Environment::Unknown => "-unknown",
        };

        format!("{}-{}-{}{}", arch, vendor, os, env)
    }
}

impl Target {
    pub fn host() -> ForgeResult<Self> {
        let triple = format!("{}-unknown-{}",
                             std::env::consts::ARCH,
                             std::env::consts::OS
        );
        Self::from_str(&triple)
    }

    pub fn is_windows(&self) -> bool {
        matches!(self.os, OS::Windows)
    }

    pub fn is_unix(&self) -> bool {
        matches!(self.os, OS::Linux | OS::Darwin)
    }

    pub fn executable_extension(&self) -> &'static str {
        if self.is_windows() { ".exe" } else { "" }
    }
}