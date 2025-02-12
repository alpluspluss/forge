use std::{
    path::{Path, PathBuf},
    process::Command,
    fs,
};
use regex::Regex;
use crate::config::CompilerConfig;

pub struct Compiler {
    include_regex: Regex,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            include_regex: Regex::new(r#"#include\s*"([^"]+)""#).unwrap(),
        }
    }

    pub fn get_includes(&self, source_file: &Path, include_dir: &Path) -> Vec<PathBuf> {
        let content = match fs::read_to_string(source_file) {
            Ok(content) => content,
            Err(_) => return Vec::new(),
        };

        self.include_regex
            .captures_iter(&content)
            .filter_map(|cap| {
                let header = &cap[1];
                let path = include_dir.join(header);
                if path.exists() {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn compile(
        &self,
        source: &Path,
        object: &Path,
        config: &CompilerConfig,
        include_dir: &Path,
        compiler: &str,
    ) -> Result<(), String> {
        println!("Compiling {}", source.display());

        let mut cmd = Command::new(compiler);
        cmd.arg("-c")
            .arg(source)
            .arg("-o")
            .arg(object)
            .arg(format!("-I{}", include_dir.display()));

        /* cc flags & macros */
        cmd.args(&config.flags);
        for (key, value) in &config.definitions {
            cmd.arg(format!("-D{}={}", key, value));
        }

        if config.warnings_as_errors {
            cmd.arg("-Werror");
        }

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute compiler: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }

        Ok(())
    }

    pub fn link(
        &self,
        objects: &[PathBuf],
        target: &Path,
        compiler: &str,
    ) -> Result<(), String> {
        println!("Linking {}", target.display());

        // Create parent directories if they don't exist
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create target directory: {}", e))?;
        }

        let mut cmd = Command::new(compiler);
        cmd.args(objects)
            .arg("-o")
            .arg(target);

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute linker: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }

        Ok(())
    }

    pub fn get_object_path(&self, source: &Path, build_dir: &Path) -> PathBuf {
        let stem = source.file_stem().unwrap().to_str().unwrap();
        build_dir.join(format!("{}.o", stem))
    }
}