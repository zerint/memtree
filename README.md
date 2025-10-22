# MemTree

A fast, zero-dependency Linux memory visualization tool that displays process memory usage in a tree-like structure. Now rewritten in Rust for improved performance and lower memory footprint!

## Features

- Hierarchical process tree visualization
- Shows individual and total memory usage (including all child processes)
- Sorts processes by total memory consumption
- Zero external dependencies (reads directly from `/proc`)
- Fast and memory-efficient

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

```bash
memtree | less -S
```

## Example
```
15584.82 MB - systemd (PID: 1, Memory: 15.17 MB, CMD: /sbin/init splash)
  5277.44 MB - bash (PID: 138371, Memory: 3.50 MB, CMD: /bin/bash)
    5273.94 MB - firefox (PID: 138374, Memory: 666.84 MB, CMD: /snap/firefox/5239/usr/lib/
      615.24 MB - Isolated Web Co (PID: 139593, Memory: 615.24 MB, CMD: /snap/firefox/5239>
      559.3 MB - Isolated Web Co (PID: 139155, Memory: 559.30 MB, CMD: /snap/firefox/5239/>
      359.79 MB - WebExtensions (PID: 138649, Memory: 359.79 MB, CMD: /snap/firefox/5239/u>
      283.6 MB - Isolated Web Co (PID: 825400, Memory: 283.60 MB, CMD: /snap/firefox/5239/>
      ...
```
