use crate::{
    config::{BuildProfile, CompilerConfig},
    error::{ForgeError, ForgeResult},
    toolchains::Toolchain,
};
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub struct Compiler {
    include_regex: Regex,
    toolchain: Option<Toolchain>,
}

impl Compiler {
    pub fn new(toolchain: Option<Toolchain>) -> Self {
        Compiler {
            include_regex: Regex::new(r#"#include\s*[<"]([^>"]+)[>"]"#).unwrap(),
            toolchain,
        }
    }

    pub fn get_includes(&self, source_file: &Path, include_dirs: &[PathBuf]) -> Vec<PathBuf> {
        let content = match std::fs::read_to_string(source_file) {
            Ok(content) => content,
            Err(_) => return Vec::new(),
        };

        let mut includes = Vec::new();
        for cap in self.include_regex.captures_iter(&content) {
            let header = &cap[1];
            for dir in include_dirs {
                let path = dir.join(header);
                if path.exists() {
                    includes.push(path);
                    break;
                }
            }
        }

        includes
    }

    pub fn compile(
        &self,
        source: &Path,
        object: &Path,
        config: &CompilerConfig,
        profile: &BuildProfile,
        include_dirs: &[PathBuf],
        compiler: &str,
    ) -> ForgeResult<()> {
        println!("Compiling {}", source.display());

        // Create directories if they don't exist
        if let Some(parent) = object.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ForgeError::Compiler(format!("Failed to create directory: {}", e)))?;
        }

        let mut cmd = if let Some(toolchain) = &self.toolchain {
            toolchain.get_compiler_command(compiler)
        } else {
            Command::new(compiler)
        };

        cmd.arg("-c")
            .arg(source)
            .arg("-o")
            .arg(object);

        for dir in include_dirs {
            cmd.arg(format!("-I{}", dir.display()));
        }

        cmd.args(&config.flags);
        cmd.arg(format!("-O{}", profile.opt_level));
        if profile.debug_info {
            cmd.arg("-g");
        }

        if profile.lto {
            cmd.arg("-flto");
        }

        cmd.args(&profile.extra_flags);

        for (key, value) in &config.definitions {
            cmd.arg(format!("-D{}={}", key, value));
        }

        for path in &config.library_paths {
            cmd.arg(format!("-L{}", path));
        }

        if config.warnings_as_errors {
            cmd.arg("-Werror");
        }

        let output = cmd
            .output()
            .map_err(|e| ForgeError::Compiler(format!("Failed to execute compiler: {}", e)))?;

        if !output.status.success() {
            return Err(ForgeError::Compiler(
                String::from_utf8_lossy(&output.stderr).into_owned()
            ));
        }

        Ok(())
    }

    pub fn link(
        &self,
        objects: &[PathBuf],
        target: &Path,
        config: &CompilerConfig,
        profile: &BuildProfile,
        compiler: &str,
    ) -> ForgeResult<()> {
        println!("Linking {}", target.display());

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ForgeError::Compiler(format!("Failed to create directory: {}", e)))?;
        }

        let mut cmd = if let Some(toolchain) = &self.toolchain {
            toolchain.get_compiler_command(compiler)
        } else {
            Command::new(compiler)
        };

        cmd.args(objects)
            .arg("-o")
            .arg(target);

        for path in &config.library_paths {
            cmd.arg(format!("-L{}", path));
        }

        for lib in &config.libraries {
            cmd.arg(format!("-l{}", lib));
        }

        if profile.lto {
            cmd.arg("-flto");
        }

        cmd.args(&profile.extra_flags);
        let output = cmd
            .output()
            .map_err(|e| ForgeError::Compiler(format!("Failed to execute linker: {}", e)))?;

        if !output.status.success() {
            return Err(ForgeError::Compiler(
                String::from_utf8_lossy(&output.stderr).into_owned()
            ));
        }

        Ok(())
    }

    pub fn get_object_path(&self, source: &Path, build_dir: &Path) -> PathBuf {
        let stem = source.file_stem().unwrap().to_str().unwrap();
        build_dir.join(format!("{}.o", stem))
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new(None)
    }
}