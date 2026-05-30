# Feature: Colorized Size Bars

## Goal

Color the progress bars and size labels to give an immediate visual signal about
relative disk usage, making hot-spots obvious at a glance without reading numbers.

## Color scheme

Thresholds are based on each item's share of the **largest sibling** (i.e. the same
`pct` value already computed in `ui.rs`):

| Share of largest | Bar color | Size label color |
|-----------------|-----------|-----------------|
| ≥ 75 % | Red | Red |
| 40 – 74 % | Yellow | Yellow |
| 10 – 39 % | Green | Green |
| < 10 % | Dark gray | Dark gray |

The **selected row** always uses the existing blue highlight background regardless of
bar color (Ratatui highlight_style overrides the item style).

## Changes required

### `src/ui.rs`

Replace the current plain-text bar construction with a `Line` of styled `Span`s so
that each segment can carry its own `Color`.

**Current approach** (produces a `String`):
```rust
let text = format!("  {:<30} {:>9}  {}  {:>3}%", name, size_str, bar, pct);
ListItem::new(text)
```

**New approach** (produces a `Line<'_>`):
```rust
fn size_color(pct: usize) -> Color {
    match pct {
        75..=100 => Color::Red,
        40..=74  => Color::Yellow,
        10..=39  => Color::Green,
        _        => Color::DarkGray,
    }
}

let color = size_color(pct);
let line = Line::from(vec![
    Span::raw(format!("  {:<30} ", name)),
    Span::styled(format!("{:>9}", size_str), Style::default().fg(color)),
    Span::raw("  "),
    Span::styled("█".repeat(filled), Style::default().fg(color)),
    Span::styled("░".repeat(empty),  Style::default().fg(Color::DarkGray)),
    Span::styled(format!("  {:>3}%", pct), Style::default().fg(color)),
]);
ListItem::new(line)
```

No changes to other modules — this is entirely in `ui.rs`.

## Acceptance criteria

- The bar color matches the size tier of each item independently.
- Selecting a row does not change bar colors (the blue bg is applied by Ratatui
  on top and remains readable).
- Items at 0 bytes (empty directories) use Dark gray.
- The feature works on terminals that report only 16 colors (Ratatui maps
  `Color::Red` / `Yellow` / `Green` to ANSI codes 1/3/2 automatically).
