# RSM (Rusty Symlink Manager)

RSM is a high-performance, modular system utility written in Rust for managing symbolic links via a centralized configuration. It is designed for developers who need an environment-aware way to manage dotfiles or system configurations across multiple machines and operating systems.

## Features

- **XDG Compliance**: Automatically searches for configuration in `--config` paths, the current directory, or `~/.config/rsm/rsm.toml`.
- **Environment Awareness**: Filter symlinks based on the current Operating System.
- **Tagging System**: Organize links into groups (e.g., "work", "ui", "server") and apply them selectively.
- **Safety First**: Includes a dry-run mode to preview changes and a force flag to prevent accidental overwrites of existing files.
- **Atomic Operations**: Automatically creates missing parent directories and validates source paths before execution.

## Installation

### Prerequisites
- Rust and Cargo (Latest Stable)

### Building from Source
Clone the repository and install the binary using Cargo:

```bash
git clone https://github.com/hunde32/rsm.git
cd rsm
cargo install --path .
