# Forge Build System Documentation

## Overview
Forge is a fast, parallel C/C++ build system written in Rust. It features workspace support, parallel compilation, and intelligent caching to minimize build times.

## Installation
```bash
# Clone and install
git clone https://github.com/your-username/forge
cd forge
cargo install --path .
```

## Basic Usage

### Creating a New Project
```bash
# initialize a basic project
forge init my-project

# initialize a workspace project
forge init my-workspace --workspace

# initialize with specific name
forge init --name my-app
```

### Building Projects
```bash
# build in current directory
forge build

# build with specific path
forge build --path /path/to/project

# build with 8 parallel jobs
forge build -j 8

# build specific workspace members
forge build --members lib1 lib2
```

## Project Structure
```
project/
├── forge.toml     # Build configuration
├── src/           # Source files
│   └── main.cpp
└── include/       # Header files
    └── example.hpp
```

## Configuration (forge.toml)

### Basic Project
```toml
[build]
compiler = "g++"
target = "my-app"

[paths]
src = "src"
include = "include"
build = "build"

[compiler]
flags = ["-Wall", "-std=c++17"]
definitions = { VERSION = "0.1.0" }
warnings_as_errors = true
```

### Workspace Project
```toml
[workspace]
members = ["lib1", "lib2"]
exclude = ["examples"]

[build]
compiler = "clang++"
target = "main"
jobs = 12

[compiler]
flags = ["-Wall", "-std=c++17"]
warnings_as_errors = true

[paths]
src = "src"
include = "include"
build = "build"
```

## Features
- Parallel compilation
- Dependency tracking
- Header file caching
- Workspace support
- Configurable compiler flags
- Build artifact caching

## FAQ

### Build Fails with "Compiler not found"
Ensure clang++ (or your chosen compiler) is installed and in your PATH.

### Workspace Member Not Building
Check that:
1. Member is listed in `workspace.members`
2. Member's `forge.toml` exists
3. Member isn't in `workspace.exclude`

# Forge Configuration Guide

## Configuration File (forge.toml)

### Root Level Sections
```toml
[build]      # build system configuration
[paths]      # dir path
[compiler]   # compiler settings
[workspace]  # workspace configuration (optional)
```

### Build Section
```toml
[build]
compiler = "g++"     
target = "app"       
jobs = 12            # number of parallel jobs (optional)
```

### Paths Section
```toml
[paths]
src = "src"          # src directory
include = "include"  # head directory
build = "build"      # build output directory
```

### Compiler Section
```toml
[compiler]
flags = [
    "-Wall",
    "-std=c++17",
    "-O2"
]

# macros
definitions = { VERSION = "1.0.0", DEBUG = "1" }
warnings_as_errors = true  # treat warnings as errors
```

### Workspace Section
```toml
[workspace]
members = [          # List of workspace members
    "lib1",
    "lib2",
    "app"
]
exclude = [          # Paths to exclude
    "examples",
    "tests"
]
```

## Hierarchical Configuration

Forge supports nested configuration files. Each workspace member can have its own `forge.toml`:

```
project/
├── forge.toml         # Workspace root config
├── lib1/
│   ├── forge.toml     # lib1 specific config
│   ├── src/
│   └── include/
└── lib2/
    ├── forge.toml     # lib2 specific config
    ├── src/
    └── include/
```

### Configuration Inheritance
- Member configs override workspace defaults
- Members can specify unique settings while inheriting others

### Example: Root forge.toml
```toml
[workspace]
members = ["lib1", "lib2"]

[build]
compiler = "clang++"

[compiler]
flags = ["-Wall", "-std=c++20"]
warnings_as_errors = true
```

### Example: Member forge.toml (lib1/forge.toml)
```toml
[build]
target = "lib1"        # override target name

[compiler]
flags = [              # override flags
    "-Wall",
    "-fPIC",           # library-specific flags
    "-shared"
]

# member-specific definitions
definitions = { LIB_VERSION = "1.0.0" }
```

## Configuration Resolution
1. Load workspace config
2. For each member:
    - Load member config
    - Override workspace defaults
    - Apply member-specific settings


## License
MIT License