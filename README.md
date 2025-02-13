# Forge

A fast C/C++ build system with cross-compilation support, workspace management, and incremental builds. Forge 
supports parallel compilation and caching by default out of the box with workspace support for multi-project builds.

## Quick Start

```bash
# initialize a new project
forge init --name project

# build it, or:
forge build

# build with release optimization
forge build --release

# now run the project!
forge run

# clean build artifacts
forge clean
```

### Project Configuration

Simply create a `forge.toml` in your project root:

```toml
[build]
compiler = "g++"
target = "myapp"

[profiles.debug]
opt_level = "0"
debug_info = true

[profiles.release]
opt_level = "3"
lto = true

[compiler]
flags = ["Wall", "-std=c++20"]
libraries = ["fmt"]

[paths]
src = "src"
include = ["include"]
```

### Workspace Support

Create a workspace for multiple projects:

```toml
[workspace]
members = ["app", "lib"]

[build]
compiler = "g++"
target = "workspace"
```

### Cross Compilation

Configure cross-compilation targets:

```toml
[cross]
target = "aarch64-unknown-linux-gnu"
toolchain = "/opt/cross"
sysroot = "/opt/sysroot"
```

## Installation

```bash
git clone https://github.com/alpluspluss/forge.git
cd forge
cargo install --path .
```

## Requirements

- Rust 1.82 or higher
- C or C++ compiler (GCC, Clang, MSVC)

## License

MIT. See [LICENSE](LICENSE) for more details.

## Contributing

Pull requests are always welcome. For major changes, 
please open an issue first to discuss what you would like to change.