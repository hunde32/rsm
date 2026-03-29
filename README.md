# rsm - Rusty Symlink Manager

[![Build Status](https://img.shields.io/github/actions/workflow/status/hunde32/rsm/ci.yml?branch=main)](https://github.com/hunde32/rsm/actions)
[![Crates.io](https://img.shields.io/crates/v/rsm-cli)](https://crates.io/crates/rsm-cli)
[![Docs.rs](https://docs.rs/rsm-cli/badge.svg)](https://docs.rs/rsm-cli/latest/rsm-cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-blue.svg)](https://www.rust-lang.org)

rsm is a high-performance command-line utility written in Rust. It is designed to manage, synchronize, and enforce complex [symbolic link](https://en.wikipedia.org/wiki/Symbolic_link) structures across your filesystem using [declarative](https://en.wikipedia.org/wiki/Declarative_programming) TOML configuration files.

## Overview

rsm provides a thread-safe, [idempotent](https://en.wikipedia.org/wiki/Idempotence) engine to ensure your filesystem state matches your configuration. Built with the Rust 2024 edition, it leverages multi-threaded execution to handle massive link maps with zero-cost abstractions and memory safety.

### Comparison

| Feature | GNU Stow | rsm |
| :--- | :---: | :---: |
| **Execution Strategy** | Sequential (Single-thread) | **Parallel (Rayon Multi-thread)** |
| **Configuration** | Folder-based / Symlink folding | **TOML-based / Absolute Paths** |
| **Recursive Sync** | Limited | **Native Tree Traversal** |
| **Environment Filters** | Manual Shell Logic | **Native OS & Host Awareness** |
| **Stale Link Cleanup** | Manual | **Native (`--prune`)** |

---

## Core Features

### Performance and Engine
* **Multi-threaded Sync:** Powered by the Rayon library, rsm uses a work-stealing scheduler to saturate available CPU cores, enabling the processing of high-volume link maps in seconds.
* **Path Canonicalization:** Automatically resolves relative paths to absolute ones to prevent broken links during directory migration.
* **Dry Run Capabilities:** Use the `--dry-run` flag to preview filesystem changes without committing them.

### Advanced Link Management
* **Recursive Mapping:** Mirror entire folder structures with a single entry by setting `recursive = true`.
* **Orphan Pruning:** The `--prune` flag identifies and deletes symlinks in destination folders that no longer have a corresponding source file.
* **Force Overwrite:** Use `--force` to allow rsm to remove existing files or directories to make room for new symlinks.

### Environment Awareness
* **OS and Host Logic:** Filter link entries based on the operating system (Linux, macOS, Windows) or specific hostnames, allowing for a single configuration file across multiple machines.
* **Tagging System:** Group symlinks using tags (e.g., "work", "ui", "shell") and synchronize only specific subsets using the `--tag` flag.
* **Glob Pattern Ignores:** Support for `.gitignore` style patterns (e.g., `*.bak`, `node_modules/`) to exclude specific files during recursive synchronization.

---

## Configuration

### Default Behavior
rsm searches for its configuration at `~/.config/rsm/rsm.toml`. Use the `--config` flag to specify a custom path.

### Initialization
Generate a documented template in your current directory:
```bash
rsm init
```

### Example Schema
```toml
# RSM Configuration
global_ignores = [".git", ".DS_Store", "node_modules"]

[[links]]
target = "~/.config/hypr/"
source = "~/dotfiles/hyprland/"
recursive = true
tags = ["wm", "ui"]
os = "linux"
ignore = ["*.bak", "secrets.conf"]

[[links]]
target = "~/.bashrc"
source = "~/dotfiles/bash/bashrc"
tags = ["shell"]
host = "my-work-laptop"
```

---

## Installation

### Arch Linux (AUR)
```bash
yay -S rsm
```

### Build from Source
```bash
git clone [https://github.com/yourusername/rsm.git](https://github.com/yourusername/rsm.git)
cd rsm
cargo build --release
```

---

## Usage

* **Synchronize State:** `rsm sync`
* **Filter by Tag:** `rsm sync --tag wm`
* **Prune Stale Links:** `rsm sync --prune`
* **System Info:** `rsm info` (View architecture and OS details)
* **Preview Changes:** `rsm sync --dry-run`

---

## Contributing

Contributions are welcome. To contribute to rsm:

1. **Fork** the repository.
2. **Create a feature branch** (`git checkout -b feature/name`).
3. **Commit** your changes following Rust best practices.
4. **Ensure tests pass** using `cargo test`.
5. **Open a Pull Request** with a detailed description of your changes.

---

## License

This project is licensed under the MIT License.
