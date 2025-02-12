mod config;
mod builder;
mod compiler;
mod workspace;
mod cache;

use std::{
    path::{Path, PathBuf},
    time::Instant,
};
use structopt::StructOpt;
use crate::{
    builder::Builder,
    workspace::Workspace,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "forge", about = "A fast C/C++ build system")]
enum Forge {
    #[structopt(name = "build")]
    Build {
        #[structopt(long, parse(from_os_str))]
        path: Option<PathBuf>,

        #[structopt(long)]
        members: Vec<String>,

        #[structopt(short = "j", long = "jobs")]
        jobs: Option<usize>,
    },

    #[structopt(name = "init")]
    Init {
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,

        #[structopt(long)]
        workspace: bool,

        #[structopt(long)]
        name: Option<String>,
    },
}

fn init_project(path: &Path, is_workspace: bool, name: Option<&str>) -> Result<(), String> {
    let name = name.unwrap_or_else(|| {
        path.file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("project")
    });

    std::fs::create_dir_all(path.join("src"))
        .map_err(|e| format!("Failed to create src directory: {}", e))?;
    std::fs::create_dir_all(path.join("include"))
        .map_err(|e| format!("Failed to create include directory: {}", e))?;

    let config = if is_workspace {
        format!(
            r#"[workspace]
members = []
exclude = []

[build]
compiler = "clang++"
target = "{}"
jobs = 12

[compiler]
flags = ["-Wall", "-std=c++17"]
warnings_as_errors = true

[paths]
src = "src"
include = "include"
build = "build"
"#,
            name
        )
    } else {
        format!(
            r#"[build]
compiler = "g++"
target = "{}"

[paths]
src = "src"
include = "include"
build = "build"

[compiler]
flags = ["-Wall", "-std=c++20"]
definitions = {{ VERSION = "0.1.0" }}
warnings_as_errors = true
"#,
            name
        )
    };

    std::fs::write(path.join("forge.toml"), config)
        .map_err(|e| format!("Failed to write forge.toml: {}", e))?;

    let example_src = r#"#include <iostream>
#include "example.hpp"

int main() {
    std::cout << "Hello from Forge!" << std::endl;
    return 0;
}
"#;
    std::fs::write(path.join("src").join("main.cpp"), example_src)
        .map_err(|e| format!("Failed to write main.cpp: {}", e))?;

    let example_header = r#"#pragma once

"#;
    std::fs::write(path.join("include").join("example.hpp"), example_header)
        .map_err(|e| format!("Failed to write example.hpp: {}", e))?;

    println!(
        "Initialized new {} project: {}",
        if is_workspace { "workspace" } else { "forge" },
        path.display()
    );
    Ok(())
}

fn main() {
    let opt = Forge::from_args();
    match opt {
        Forge::Build { path, members, jobs } => {
            let start = Instant::now();

            if let Some(n) = jobs {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(n)
                    .build_global()
                    .unwrap();
            }

            let path = path.unwrap_or_else(|| std::env::current_dir().unwrap());

            match Workspace::new(&path) {
                Ok(workspace) => {
                    let filtered_members = workspace.filter_members(&members);
                    let builder = Builder::new(workspace.clone());

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
        Forge::Init { path, workspace, name } => {
            let path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            if let Err(e) = init_project(&path, workspace, name.as_deref()) {
                eprintln!("Failed to initialize project: {}", e);
                std::process::exit(1);
            }
        }
    }
}