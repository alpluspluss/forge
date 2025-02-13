use crate::{
    error::{ForgeError, ForgeResult},
    target::Target,
};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Toolchain {
    root: PathBuf,
    target: Target,
    sysroot: Option<PathBuf>,
    extra_flags: Vec<String>,
}

impl Toolchain {
    pub fn new(
        target: Target,
        toolchain_path: Option<&str>,
        sysroot: Option<&Path>,
        extra_flags: Vec<String>,
    ) -> ForgeResult<Self> {
        let root = if let Some(path) = toolchain_path {
            PathBuf::from(path)
        } else {
            PathBuf::from("/usr/local/bin")
        };

        Ok(Self {
            root,
            target,
            sysroot: sysroot.map(PathBuf::from),
            extra_flags,
        })
    }

    pub fn get_compiler_command(&self, compiler: &str) -> Command {
        let compiler_path = self.get_compiler_path(compiler);
        let mut cmd = Command::new(&compiler_path);

        // Add target specification
        cmd.arg(format!("--target={}", self.target.to_string()));

        // Add sysroot if specified
        if let Some(sysroot) = &self.sysroot {
            cmd.arg(format!("--sysroot={}", sysroot.display()));
        }

        // Add any extra flags
        cmd.args(&self.extra_flags);

        cmd
    }

    pub fn get_compiler_path(&self, compiler: &str) -> PathBuf {
        if self.target.is_windows() {
            self.root.join(format!("{}.exe", compiler))
        } else {
            let prefix = format!(
                "{}-{}-{}-",
                self.target.arch.to_string().to_lowercase(),
                self.target.vendor.to_string().to_lowercase(),
                self.target.os.to_string().to_lowercase()
            );
            self.root.join(format!("{}{}", prefix, compiler))
        }
    }

    pub fn get_sysroot(&self) -> Option<&Path> {
        self.sysroot.as_deref()
    }

    pub fn with_extra_flags(mut self, flags: Vec<String>) -> Self {
        self.extra_flags = flags;
        self
    }

    pub fn verify(&self) -> ForgeResult<()> {
        if !self.root.exists() {
            return Err(ForgeError::Config(format!(
                "Toolchain root directory does not exist: {}",
                self.root.display()
            )));
        }

        if let Some(sysroot) = &self.sysroot {
            if !sysroot.exists() {
                return Err(ForgeError::Config(format!(
                    "Sysroot directory does not exist: {}",
                    sysroot.display()
                )));
            }
        }

        Ok(())
    }
}