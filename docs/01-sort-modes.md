# Feature: Sort Modes

## Goal

Let the user cycle through different sort orders so they can find large files by size,
locate items alphabetically, or see which directories contain the most files.

## Key binding

`s` — cycle through sort modes in the current view (does not trigger a rescan).

## Sort modes (in cycle order)

| Mode | Order | Description |
|------|-------|-------------|
| `size` (default) | descending | Largest items first — current behaviour |
| `name` | ascending | Alphabetical A → Z |
| `count` | descending | Most files first |

The active mode is shown in the status bar: `[s] Sort: size`.

## Changes required

### `src/scanner.rs`
- Add `file_count: u64` field to `DirNode` (sum of all non-directory entries in the subtree).
- Populate it during `scan_dir`.

### `src/app.rs`
- Add `SortMode` enum: `Size | Name | Count`.
- Add `sort_mode: SortMode` field to `App`.
- Add `cycle_sort(&mut self)` method that advances the mode and re-sorts `current_children` in place.
- Sorting is applied lazily on navigation (enter/go_up) and on mode change — avoids re-cloning the whole tree.

### `src/main.rs`
- Map `KeyCode::Char('s')` → `app.cycle_sort()`.

### `src/ui.rs`
- Show current sort mode in the status bar next to `[q] Quit`.

## Acceptance criteria

- Pressing `s` three times returns to the original order.
- Sort persists as the user navigates into subdirectories.
- The column header or status label reflects the active mode.
- `file_count` is correct: a directory with 2 sub-dirs each containing 3 files shows 6.
