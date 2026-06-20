//! Render the directory tree pane.
//!
//! The tree is rendered as a flat list of [`super::state::TreeRow`] with
//! depth-based indentation. The selected row gets a highlight; directory
//! rows are prefixed with a `▾`/`▸` glyph so collapse state is visible
//! at a glance.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::Frame;

use super::state::{FilesState, PaneFocus};
use crate::theme::Theme;

pub fn render_tree(frame: &mut Frame, area: Rect, state: &mut FilesState) {
    let focus_border = if matches!(state.focus, PaneFocus::Tree) {
        Theme::BORDER_ACTIVE
    } else {
        Theme::BORDER
    };
    let block = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(focus_border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 2 || inner.height < 1 {
        return;
    }

    if state.is_empty() {
        let placeholder = match state.dirs.get("") {
            Some(crate::components::files_view::state::DirState::Loading) | None => {
                "Loading artifacts…"
            }
            Some(crate::components::files_view::state::DirState::Error(e)) => e.as_str(),
            _ => "No artifacts",
        };
        let msg = ratatui::widgets::Paragraph::new(format!(" {} ", placeholder))
            .style(Style::default().fg(Theme::DIM));
        frame.render_widget(msg, inner);
        return;
    }

    let items: Vec<ListItem> = state
        .visible_rows
        .iter()
        .map(|row| {
            let indent: String = "  ".repeat(row.depth);
            let glyph = if row.is_dir {
                if state.is_dir_open(&row.path) {
                    "▾ "
                } else {
                    "▸ "
                }
            } else {
                "  "
            };
            let style = if row.is_dir {
                Style::default()
                    .fg(Theme::ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::FG)
            };
            let size_suffix = match row.size {
                Some(b) => format!(" ({})", human_size(b)),
                None => String::new(),
            };
            let label = format!("{}{}{}{}", indent, glyph, row.name, size_suffix);
            ListItem::new(Line::from(Span::styled(label, style)))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Theme::TABLE_SELECTED_BG)
                .fg(Theme::FG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");
    // Use the persistent list state so ratatui preserves the scroll offset
    // across frames and keeps the selection in view.
    state.tree_list_state.select(Some(state.selected_idx));
    frame.render_stateful_widget(list, inner, &mut state.tree_list_state);

    // Scrollbar
    if state.visible_rows.len() > inner.height as usize {
        let max = state
            .visible_rows
            .len()
            .saturating_sub(inner.height as usize);
        let mut sb_state = ScrollbarState::new(max).position(state.selected_idx.min(max));
        let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(sb, inner, &mut sb_state);
    }
}

fn human_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KiB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
