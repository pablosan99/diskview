# diskview

An interactive terminal disk usage analyzer built with Rust and [Ratatui](https://github.com/ratatui-org/ratatui).

## Features

- Interactive TUI for browsing directory sizes
- Parallel directory scanning with live progress
- Result caching to speed up repeated runs (24-hour expiry by default)
- Delete files/directories or clean folder contents directly from the UI
- Vim-style keyboard navigation

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Scan current directory
diskview

# Scan a specific path
diskview /path/to/directory

# Force rescan, ignoring cache
diskview --refresh

# Set cache expiry in hours
diskview --expiry 48
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `l` / `→` / `Enter` | Enter directory |
| `h` / `←` / `u` / `Backspace` | Go up |
| `d` | Delete selected item |
| `c` | Clean selected directory (delete contents) |
| `r` | Rescan current directory |
| `q` / `Esc` | Quit |

Deletion and cleaning require confirmation (`y` to confirm, `n`/`Esc` to cancel).

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) — terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — cross-platform terminal control
- [rayon](https://github.com/rayon-rs/rayon) — parallel scanning
- [walkdir](https://github.com/BurntSushi/walkdir) — directory traversal
- [clap](https://github.com/clap-rs/clap) — CLI argument parsing
- [serde](https://serde.rs) / [serde_json](https://github.com/serde-rs/json) — cache serialization

## License

MIT
