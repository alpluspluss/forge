# Forge Build System Documentation

## Overview
Forge is a fast, parallel C/C++ build system written in Rust. It features workspace support, parallel compilation, incremental builds, cross-compilation support, and intelligent caching to minimize build times.

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
# Initialize a basic project
forge init my-project

# Initialize a workspace project
forge init my-workspace --workspace

# Initialize with specific name and target
forge init --name my-app --target aarch64-unknown-linux-gnu
```

### Building Projects
```bash
# Build in current directory
forge build

# Build with release profile
forge build --release

# Build with specific path
forge build --path /path/to/project

# Build with 8 parallel jobs
forge build -j 8

# Build specific workspace members
forge build --members lib1 lib2

# Cross-compile for ARM64
forge build --target aarch64-unknown-linux-gnu \
           --toolchain /opt/cross/bin/aarch64-linux-gnu- \
           --sysroot /opt/cross/sysroot

# Clean build artifacts
forge clean
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
default_profile = "debug"

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

[paths]
src = "src"
include = ["include"]
build = "build"

[compiler]
flags = ["-Wall", "-std=c++20"]
definitions = { VERSION = "0.1.0" }
warnings_as_errors = true
library_paths = []
libraries = []
```

### Cross-Compilation Project
```toml
[build]
compiler = "g++"
target = "my-app"

[cross]
target = "aarch64-unknown-linux-gnu"
toolchain = "/opt/cross/bin/aarch64-linux-gnu-"
sysroot = "/opt/cross/sysroot"
extra_flags = [
    "-march=armv8-a",
    "-mcpu=cortex-a72"
]

[profiles.debug]
opt_level = "0"
debug_info = true
lto = false

[profiles.release]
opt_level = "3"
debug_info = false
lto = true
extra_flags = ["-march=native"]

[compiler]
flags = ["-Wall", "-std=c++20"]
library_paths = ["/usr/aarch64-linux-gnu/lib"]
libraries = ["stdc++", "m"]
warnings_as_errors = true
```

### Workspace Project
```toml
[workspace]
members = ["lib1", "lib2"]
exclude = ["examples"]
dependencies = { lib2 = ["lib1"] }  # lib2 depends on lib1

[build]
compiler = "clang++"
target = "main"
jobs = 12

[compiler]
flags = ["-Wall", "-std=c++20"]
warnings_as_errors = true

[paths]
src = "src"
include = ["include"]
build = "build"
```

## Features
- Parallel compilation with progress tracking
- Smart incremental builds
- Cross-compilation support
- Build profiles (debug/release)
- Workspace dependency management
- Advanced build caching
- Platform-specific configurations
- Library linking support
- Multiple compiler support (GCC, Clang, MSVC)

## Advanced Features

### Build Profiles
Forge supports multiple build profiles with different optimization settings:
```toml
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
```

### Cross-Compilation
Configure cross-compilation settings:
```toml
[cross]
target = "aarch64-unknown-linux-gnu"
toolchain = "/opt/cross/bin/aarch64-linux-gnu-"
sysroot = "/opt/cross/sysroot"
extra_flags = []
```

### Dependency Management
Specify workspace member dependencies:
```toml
[workspace]
members = ["core", "gui"]
dependencies = { gui = ["core"] }  # gui depends on core
```

## FAQ

### Build Fails with "Compiler not found"
Ensure your compiler (g++, clang++, cl.exe) is installed and in your PATH.

### Cross-Compilation Issues
1. Verify toolchain installation
2. Check sysroot path
3. Ensure correct target triple
4. Verify library paths for the target platform

### Workspace Member Not Building
Check that:
1. Member is listed in `workspace.members`
2. Member's `forge.toml` exists
3. Member isn't in `workspace.exclude`
4. All dependencies are available

### Cache Issues
If incremental builds aren't working:
1. Try `forge clean`
2. Check file permissions
3. Verify compiler flags haven't changed
4. Check include paths

## License
MIT License