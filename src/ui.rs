use std::time::SystemTime;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, ConfirmAction};

pub fn draw(frame: &mut Frame, app: &App, list_state: &mut ListState) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .split(area);

    // Title bar
    let path_display = app.current_node().path.to_string_lossy();
    let title = Paragraph::new(format!(" DiskView  {}", path_display))
        .style(Style::default().fg(Color::Cyan).bold());
    frame.render_widget(title, chunks[0]);

    // Content list
    let children = app.current_children();
    let max_size = children.first().map(|c| c.size).unwrap_or(1).max(1);

    // Reserve space: indicator(3) + name(30) + size(10) + spaces(4) + pct(4) = 51 minimum
    let bar_width = (area.width as usize).saturating_sub(55).max(8);

    let items: Vec<ListItem> = if children.is_empty() {
        vec![ListItem::new("  (no subdirectories)").style(Style::default().fg(Color::DarkGray))]
    } else {
        children
            .iter()
            .map(|child| {
                let size_str = format_size(child.size);
                let pct = (child.size as f64 / max_size as f64 * 100.0) as usize;
                let filled = (child.size as f64 / max_size as f64 * bar_width as f64) as usize;
                let empty = bar_width - filled;

                let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

                let raw_name = if !child.children.is_empty() {
                    format!("{}/", child.name)
                } else {
                    child.name.clone()
                };
                let name = truncate(&raw_name, 30);

                let text = format!(
                    "  {:<30} {:>9}  {}  {:>3}%",
                    name, size_str, bar, pct
                );
                ListItem::new(text)
            })
            .collect()
    };

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .bold(),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, chunks[1], list_state);

    // Status bar
    let age = format_age(app.scanned_at);
    let total = format_size(app.root.size);
    let nav_hint = if app.nav_stack.is_empty() {
        ""
    } else {
        "  [Bksp] Up"
    };
    let has_children = !children.is_empty();
    let action_hints = if has_children {
        "  [d] Delete  [c] Clean"
    } else {
        ""
    };
    let status = format!(
        " Total: {}  Cache: {}{}   [↑↓] Move  [Enter] Open  [r] Rescan{}  [q] Quit",
        total, age, nav_hint, action_hints
    );
    frame.render_widget(
        Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
        chunks[2],
    );

    if let Some((action, path)) = &app.confirm {
        draw_confirm_overlay(frame, action, path, app.selected_size());
    }
}

fn draw_confirm_overlay(
    frame: &mut Frame,
    action: &ConfirmAction,
    path: &std::path::Path,
    size: u64,
) {
    let area = frame.area();
    let popup_width = (area.width).min(70).max(40);
    let popup_height = 7u16;
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let (title, verb, color) = match action {
        ConfirmAction::Delete => ("Confirm Delete", "delete", Color::Red),
        ConfirmAction::Clean => ("Confirm Clean", "remove contents of", Color::Yellow),
    };

    let path_str = path.to_string_lossy();
    let size_str = format_size(size);
    let body = format!(
        "\n  This will {} folder:\n  {}\n  Size: {}\n\n  [y / Enter] Confirm   [n / Esc] Cancel",
        verb, path_str, size_str
    );

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title_style(Style::default().fg(color).bold());

    let paragraph = Paragraph::new(body)
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, popup_area);
}

pub fn draw_scanning(frame: &mut Frame, count: u64, label: &str) {
    let area = frame.area();
    let text = format!(" {}... {} directories processed", label, count);
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::Yellow)),
        area,
    );
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_age(scanned_at: SystemTime) -> String {
    let secs = SystemTime::now()
        .duration_since(scanned_at)
        .unwrap_or_default()
        .as_secs();

    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars - 1).collect();
        format!("{}…", truncated)
    }
}
