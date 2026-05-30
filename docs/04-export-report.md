# Feature: Export Report

## Goal

Let the user dump the current scan tree to a plain-text file so they can share it,
diff it against a later scan, or pipe it into other tools.

## Key binding

`e` — export the full tree to a file and show a one-second confirmation toast.

## Output format

A UTF-8 text file with an indented tree, sorted in the same order as the current
sort mode:

```
DiskView report — /home/user/projects — 2026-05-30 14:22:11
Total: 4.2 GB

/home/user/projects
├── node_modules/        42 831 files    512.3 MB
│   ├── lodash/           1 024 files     18.1 MB
│   └── typescript/         800 files     42.0 MB
├── src/                    124 files      1.2 MB
└── dist/                    18 files     98.4 MB
```

## Output location

Default: `diskview-<sanitised-path>-<timestamp>.txt` in the current working directory.

Sanitised path: replace `/`, `\`, `:` with `-`, strip leading separators.
Example: `/home/user/projects` → `diskview-home-user-projects-20260530-142211.txt`.

## Changes required

### `src/export.rs` (new file)
- `pub fn export(root: &DirNode, sort_mode: SortMode, scan_path: &Path, scanned_at: SystemTime) -> Result<PathBuf>`
- Renders the tree recursively with `├──` / `└──` / `│` box-drawing characters.
- Returns the path of the written file.
- Uses only `std` (no extra deps needed).

### `src/app.rs`
- Add `export_toast: Option<String>` field — holds a short message to display after export.
- Add `set_toast(msg: String)` and `clear_toast()`.

### `src/main.rs`
- Add `mod export;`.
- Map `KeyCode::Char('e')` → call `export::export(...)`, on success call
  `app.set_toast(format!("Exported to {}", path.display()))`.
- On the next key event (or after 1 s via poll timeout), call `app.clear_toast()`.

### `src/ui.rs`
- If `app.export_toast` is `Some(msg)`, draw a small centered overlay with the
  message (similar to the confirm overlay but auto-dismissing, styled in green).

### `src/main.rs` mod list
```rust
mod export;
```

## Acceptance criteria

- `e` produces a file in the current working directory.
- The file tree matches what is visible in the TUI (same sort order).
- The toast disappears after the next keypress or 1-second poll cycle.
- Exporting a very large tree (10 000+ nodes) completes in under 1 second.
- If the write fails (e.g. read-only filesystem), the error is shown in the toast
  instead of crashing.
