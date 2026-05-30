# Feature: File Count Column

## Goal

Show how many files live inside each directory alongside its size. This helps
identify directories that are large because of many small files vs. a few large ones.

## Display

Add a `Files` column between the name and the size bar:

```
  name                            files      size        bar              pct
  ▶ node_modules/               42 831    512.3 MB  ████████████░░░░  100%
    src/                           124      1.2 MB  ░░░░░░░░░░░░░░░░    0%
    dist/                           18     98.4 MB  ███░░░░░░░░░░░░░   19%
```

File count is right-aligned in a fixed 8-character column.

## Changes required

### `src/scanner.rs`
- Add `file_count: u64` to `DirNode`.
- In `scan_dir`: accumulate `direct_file_count` (non-dir entries), add children's
  `file_count` values, store total in the node.
- Update `Serialize`/`Deserialize` derives (automatic — no manual impl needed).

> **Note:** existing cache files will fail to deserialize (missing field). The cache
> load in `src/cache.rs` already silently returns `None` on any parse error, so old
> caches simply trigger a rescan — no migration needed.

### `src/ui.rs`
- Update the format string in the list item builder to include the `file_count`
  column: `{:<30} {:>8}  {:>9}  {}  {:>3}%` with `format_count(child.file_count)`.
- Add `format_count(n: u64) -> String` that abbreviates large numbers:
  `999` → `999`, `1 000` → `1.0 K`, `1 500 000` → `1.5 M`.
- Adjust `bar_width` calculation to account for the extra column width (~10 chars).

### Column width budget (100-column terminal)

| Field | Width |
|-------|-------|
| indicator | 3 |
| name | 30 |
| files | 9 |
| size | 10 |
| gaps | 4 |
| pct | 4 |
| bar | remainder (≥ 8) |

## Acceptance criteria

- `file_count` for a leaf directory equals the number of direct non-dir files.
- `file_count` for a parent equals the recursive total across all descendants.
- The count column does not overflow or truncate on typical terminal widths (≥ 80).
- Old cache entries trigger a silent rescan rather than a crash.
