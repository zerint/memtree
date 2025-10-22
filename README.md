# MemTree

A fast, zero-dependency Linux memory visualization tool that displays process memory usage in a tree-like structure. Now rewritten in Rust for improved performance and lower memory footprint!

## Features

- **Interactive TUI**: Navigate and explore processes with keyboard controls
- **Collapsible tree structure**: Expand/collapse process branches to focus on what matters
- **Hierarchical visualization**: See parent-child process relationships at a glance
- **Memory aggregation**: Shows individual and total memory usage (including all child processes)
- **Smart sorting**: Processes sorted by total memory consumption
- **Zero external dependencies**: Reads directly from `/proc` - no crates needed!
- **Fast and memory-efficient**: Compiled Rust binary with minimal overhead

## Installation

### Pre-built Binary
```bash
sudo curl -L https://github.com/zerint/memtree/releases/latest/download/memtree -o /usr/local/sbin/memtree
sudo chmod +x /usr/local/sbin/memtree
```

### Build from Source (Rust)

```bash
# Clone the repository
git clone https://github.com/zerint/memtree.git
cd memtree

# Build release binary
cargo build --release

# Install system-wide (optional)
sudo cp target/release/memtree /usr/local/bin/memtree
```

### Python Version (Legacy)

The original Python version is still available in `memtree.py`:

```bash
# Requires Python 3.12+ and psutil
poetry install
poetry run python memtree.py
```

## Usage

Simply run `memtree` to launch the interactive viewer:

```bash
memtree
```

### Interactive Controls

- **Arrow keys (↑/↓)**: Navigate up and down through processes
- **Enter or Space**: Expand/collapse the selected process to show/hide children
- **q**: Quit the application

### Visual Indicators

- `>` : Selected process (highlighted)
- `[-]` : Expanded node with children (press Enter to collapse)
- `[+]` : Collapsed node with children (press Enter to expand)

## Example

```
MemTree - Interactive Process Memory Viewer
Arrow keys: navigate | Enter/Space: expand/collapse | q: quit

> [-] 383.33 MB - process_api (PID: 1, Memory: 12.76 MB, CMD: /process_api --addr...)
    [-] 370.57 MB - sh (PID: 19, Memory: 3.72 MB, CMD: /bin/sh -c mkdir -p...)
      [+] 366.85 MB - environment-man (PID: 21, Memory: 53.15 MB, CMD: /usr/local/bin...)
        313.7 MB - claude (PID: 81, Memory: 296.79 MB, CMD: claude)
```

The interface dynamically updates as you expand and collapse nodes, allowing you to drill down into memory-hungry process trees while keeping the overall view clean and manageable.
