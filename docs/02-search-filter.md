# Feature: Search / Filter

## Goal

Allow the user to narrow the visible list to entries whose name contains a typed
substring, without leaving the current directory level.

## Key bindings

| Key | Action |
|-----|--------|
| `/` | Enter filter mode — focus moves to an inline input box |
| `Esc` | Clear filter and exit filter mode |
| `Enter` | Confirm filter, return focus to the list |

While filter mode is active, printable characters append to the query; `Backspace`
removes the last character.

## UX behaviour

- The list updates on every keystroke (no need to press `Enter` to apply).
- Items that do not match are hidden; the selection resets to 0 on each change.
- Navigating into a directory (Enter/→) clears the filter automatically.
- The filter bar appears between the title bar and the list — one extra row shown
  only when filter mode is active.

## Changes required

### `src/app.rs`
- Add `filter: String` and `filter_active: bool` fields to `App`.
- Add `filtered_children(&self) -> Vec<&DirNode>` that returns only children whose
  `name` contains `self.filter` (case-insensitive).
- All navigation methods (`move_down`, `move_up`, `enter`, `start_confirm`) must
  operate on `filtered_children()` instead of `current_children()`.
- Add `push_filter_char(c: char)`, `pop_filter_char()`, `clear_filter()`,
  `set_filter_active(bool)` helpers.

### `src/ui.rs`
- When `app.filter_active`, insert a one-row filter bar between the title and the
  list: `  Filter: <query>█` with a blinking-cursor illusion (static `█` suffix).
- Update `Layout` constraints accordingly (`Length(1)` added only when active).
- Highlight matching substring in each list item name (optional, nice-to-have).

### `src/main.rs`
- Key routing: when `app.filter_active` is true, route printable chars and Backspace
  to filter methods before the normal movement keys.
- Map `KeyCode::Char('/')` → `app.set_filter_active(true)`.
- Map `Esc` in filter mode → `app.clear_filter(); app.set_filter_active(false)`.
- Map `Enter` in filter mode → `app.set_filter_active(false)` (keep filter applied).

## Acceptance criteria

- Typing `/log` shows only entries whose name contains "log" (case-insensitive).
- Pressing `Esc` restores the full list.
- The selection never goes out of bounds as items are filtered in/out.
- Navigating into a subdirectory clears the filter so the new level shows all entries.
