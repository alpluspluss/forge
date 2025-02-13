mod config;
mod builder;
mod compiler;
mod workspace;
mod cache;
mod target;
mod toolchains;
mod error;

use std::{
    path::{Path, PathBuf},
    time::Instant,
};
use structopt::StructOpt;
use crate::{
    builder::Builder,
    workspace::Workspace,
    error::ForgeResult,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "forge", about = "A fast C/C++ build system with cross-compilation support")]
enum Forge {
    #[structopt(name = "build", about = "Build projects")]
    Build {
        #[structopt(long, parse(from_os_str), help = "Path to workspace or project")]
        path: Option<PathBuf>,

        #[structopt(long, help = "Specific workspace members to build")]
        members: Vec<String>,

        #[structopt(short = "j", long = "jobs", help = "Number of parallel jobs")]
        jobs: Option<usize>,

        #[structopt(long = "target", help = "Target triple for cross-compilation")]
        target: Option<String>,

        #[structopt(long = "toolchain", help = "Path to cross-compilation toolchain")]
        toolchain: Option<String>,

        #[structopt(long = "sysroot", parse(from_os_str), help = "Path to sysroot")]
        sysroot: Option<PathBuf>,

        #[structopt(long = "profile", help = "Build profile (debug/release)")]
        profile: Option<String>,

        #[structopt(long = "release", help = "Build with release profile")]
        release: bool,
    },

    #[structopt(name = "init", about = "Initialize a new project or workspace")]
    Init {
        #[structopt(parse(from_os_str), help = "Path to create project")]
        path: Option<PathBuf>,

        #[structopt(long, help = "Initialize as a workspace")]
        workspace: bool,

        #[structopt(long, help = "Project name")]
        name: Option<String>,

        #[structopt(long, help = "Target triple")]
        target: Option<String>,
    },

    #[structopt(name = "clean", about = "Clean build artifacts")]
    Clean {
        #[structopt(long, parse(from_os_str), help = "Path to workspace or project")]
        path: Option<PathBuf>,

        #[structopt(long, help = "Specific workspace members to clean")]
        members: Vec<String>,
    },
}

fn init_project(
    path: &Path,
    is_workspace: bool,
    name: Option<&str>,
    target: Option<&str>,
) -> ForgeResult<()> {
    let name = name.unwrap_or_else(|| {
        path.file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("project")
    });

    let native_target = format!("{}-unknown-{}",
                                std::env::consts::ARCH,
                                match std::env::consts::OS {
                                    "macos" => "darwin",
                                    os => os
                                }
    );

    let default_compiler = match std::env::consts::OS {
        "windows" => "cl.exe",
        _ => "g++",
    };

    std::fs::create_dir_all(path.join("src"))?;
    std::fs::create_dir_all(path.join("include"))?;

    let config = if is_workspace {
        format!(
            r#"[workspace]
members = []
exclude = []

[build]
compiler = "{compiler}"
target = "{name}"
jobs = 12

[profiles.debug]
opt_level = "0"
debug_info = true
lto = false
extra_flags = ["-g"]

[profiles.release]
opt_level = "3"
debug_info = false
lto = true
extra_flags = ["-march=native"]

[compiler]
flags = ["-Wall", "-std=c++17"]
warnings_as_errors = true
library_paths = []
libraries = []

[paths]
src = "src"
include = ["include"]
build = "build"
"#,
            compiler = default_compiler
        )
    } else {
        format!(
            r#"
[build]
compiler = "{compiler}"
target = "{name}"

[cross]
target = "{target}"
toolchain = ""
sysroot = ""
extra_flags = []

[profiles.debug]
opt_level = "0"
debug_info = true
lto = false

[profiles.release]
opt_level = "3"
debug_info = false
lto = true
extra_flags = ["-march=native"]

[paths]
src = "src"
include = ["include"]
build = "build"

[compiler]
flags = ["-Wall", "-std=c++20"]
definitions = {{ VERSION = "0.1.0" }}
warnings_as_errors = true
library_paths = []
libraries = []
"#,
            target = target.unwrap_or(&native_target),
            compiler = default_compiler
        )
    };

    std::fs::write(path.join("forge.toml"), config)?;

    let example_src = r#"#include <iostream>
#include "example.hpp"

int main()
{
    std::cout << "Hello from Forge!" << std::endl;
    return 0;
}
"#;
    std::fs::write(path.join("src").join("main.cpp"), example_src)?;

    let example_header = r#"#pragma once

class Example
{
public:
    Example() = default;
    ~Example() = default;
};
"#;
    std::fs::write(path.join("include").join("example.hpp"), example_header)?;

    println!(
        "Initialized new {} project: {}",
        if is_workspace { "workspace" } else { "forge" },
        path.display()
    );
    Ok(())
}

fn main() {
    env_logger::init();

    let opt = Forge::from_args();
    match opt {
        Forge::Build {
            path,
            members,
            jobs,
            target,
            toolchain,
            sysroot,
            profile,
            release,
        } => {
            let start = Instant::now();

            if let Some(n) = jobs {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(n)
                    .build_global()
                    .unwrap();
            }

            let path = path.unwrap_or_else(|| std::env::current_dir().unwrap());

            let profile = if release {
                Some("release".to_string())
            } else {
                profile
            };

            match Workspace::new(&path) {
                Ok(workspace) => {
                    let workspace_clone = workspace.clone();
                    let filtered_members = workspace_clone.filter_members(&members);
                    let builder = Builder::new(
                        workspace,
                        target.as_deref(),
                        toolchain.as_deref(),
                        sysroot.as_deref(),
                        profile.as_deref(),
                    );

                    if let Err(e) = builder.build(&filtered_members) {
                        eprintln!("Build failed: {}", e);
                        std::process::exit(1);
                    }
                    println!("Build completed in {:.2}s", start.elapsed().as_secs_f32());
                }
                Err(e) => {
                    eprintln!("Failed to load workspace: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Forge::Init { path, workspace, name, target } => {
            let path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            if let Err(e) = init_project(&path, workspace, name.as_deref(), target.as_deref()) {
                eprintln!("Failed to initialize project: {}", e);
                std::process::exit(1);
            }
        }

        Forge::Clean { path, members } => {
            let path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            match Workspace::new(&path) {
                Ok(workspace) => {
                    let workspace_clone = workspace.clone();
                    let filtered_members = workspace_clone.filter_members(&members);
                    let builder = Builder::new(
                        workspace,
                        None,
                        None,
                        None,
                        None,
                    );
                    if let Err(e) = builder.clean(&filtered_members) {
                        eprintln!("Clean failed: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(_e) => (),
            }
        }
    }
}