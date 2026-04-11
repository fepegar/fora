use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::components::workspace_picker::centered_rect;
use crate::theme::Theme;

/// Column visibility/ordering picker modal.
#[derive(Debug, Default)]
pub struct ColumnPicker {
    pub active: bool,
    pub columns: Vec<ColumnEntry>,
    state: ListState,
    /// Index of the column currently being moved, if any.
    moving: Option<usize>,
    /// Whether any changes were made (ordering or visibility).
    pub changed: bool,
    /// Whether the user explicitly requested saving (via `s` key).
    pub save_requested: bool,
}

#[derive(Debug, Clone)]
pub struct ColumnEntry {
    pub id: &'static str,
    pub label: &'static str,
    pub visible: bool,
}

impl ColumnPicker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, columns: Vec<ColumnEntry>) {
        self.columns = columns;
        self.active = true;
        self.moving = None;
        self.changed = false;
        self.save_requested = false;
        if !self.columns.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn close(&mut self) {
        self.active = false;
        self.moving = None;
    }

    /// Handle a key event. Returns true if the picker consumed the event.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.active {
            return false;
        }

        if let Some(moving_idx) = self.moving {
            // Move mode: ↑↓ swap column position, Enter/Space/Esc to confirm
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if moving_idx > 0 {
                        self.columns.swap(moving_idx, moving_idx - 1);
                        self.moving = Some(moving_idx - 1);
                        self.state.select(Some(moving_idx - 1));
                        self.changed = true;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if moving_idx + 1 < self.columns.len() {
                        self.columns.swap(moving_idx, moving_idx + 1);
                        self.moving = Some(moving_idx + 1);
                        self.state.select(Some(moving_idx + 1));
                        self.changed = true;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc => {
                    self.moving = None;
                }
                _ => {}
            }
            return true;
        }

        // Browse mode
        match key.code {
            KeyCode::Esc | KeyCode::Char('c') => {
                self.close();
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self
                    .state
                    .selected()
                    .map(|i| i.saturating_sub(1))
                    .unwrap_or(0);
                self.state.select(Some(i));
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.columns.len();
                let i = self
                    .state
                    .selected()
                    .map(|i| (i + 1).min(len.saturating_sub(1)))
                    .unwrap_or(0);
                self.state.select(Some(i));
                true
            }
            KeyCode::Char('v') => {
                if let Some(i) = self.state.selected() {
                    if i < self.columns.len() {
                        self.columns[i].visible = !self.columns[i].visible;
                        self.changed = true;
                    }
                }
                true
            }
            KeyCode::Char('s') => {
                if self.changed {
                    self.save_requested = true;
                    self.close();
                }
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(i) = self.state.selected() {
                    if i < self.columns.len() {
                        self.moving = Some(i);
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Returns the current column settings (id, visible) in display order.
    pub fn get_columns(&self) -> Vec<(&'static str, bool)> {
        self.columns.iter().map(|c| (c.id, c.visible)).collect()
    }

    pub fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        if self.moving.is_some() {
            vec![("↑↓", "Move"), ("Enter", "Done"), ("Esc", "Done")]
        } else {
            let mut hints = vec![("↑↓", "Navigate"), ("Enter", "Reorder"), ("v", "Toggle")];
            if self.changed {
                hints.push(("s", "Save"));
            }
            hints.push(("Esc", "Close"));
            hints
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.active {
            return;
        }

        let modal = centered_rect(40, 60, area);
        frame.render_widget(Clear, modal);

        let title = if let Some(idx) = self.moving {
            let name = self.columns.get(idx).map(|c| c.label).unwrap_or("?");
            format!(" Moving: {} (↑↓) ", name)
        } else {
            " Columns (Enter: reorder, v: toggle) ".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MODAL_BORDER))
            .style(Style::default().bg(Theme::MODAL_BG));

        let items: Vec<ListItem> = self
            .columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                let is_moving = self.moving == Some(idx);

                let vis_icon = if col.visible { "●" } else { "○" };
                let vis_style = if col.visible {
                    Style::default().fg(Theme::SUCCESS)
                } else {
                    Style::default().fg(Theme::DIM)
                };

                let label_style = if is_moving {
                    Style::default()
                        .fg(Theme::ACCENT)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::FG)
                };

                let mut spans = Vec::new();

                if is_moving {
                    spans.push(Span::styled("≡ ", Style::default().fg(Theme::ACCENT)));
                } else {
                    spans.push(Span::raw("  "));
                }

                spans.push(Span::styled(format!("{} ", vis_icon), vis_style));
                spans.push(Span::styled(col.label, label_style));

                ListItem::new(Line::from(spans))
            })
            .collect();

        let highlight_style = if self.moving.is_some() {
            Style::default().bg(Theme::ACCENT_DIM)
        } else {
            Style::default().bg(Theme::MODAL_SELECTED_BG)
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style);

        frame.render_stateful_widget(list, modal, &mut self.state);

        // Render help hints at the bottom of the modal
        let hints = self.key_hints();
        let hint_spans: Vec<Span> = hints
            .iter()
            .flat_map(|(key, desc)| {
                vec![
                    Span::styled(
                        format!(" {} ", key),
                        Style::default().fg(Theme::HELP_KEY_FG),
                    ),
                    Span::styled(
                        format!("{} ", desc),
                        Style::default().fg(Theme::HELP_DESC_FG),
                    ),
                ]
            })
            .collect();

        let hint_area = ratatui::layout::Rect {
            x: modal.x + 1,
            y: modal.y + modal.height.saturating_sub(1),
            width: modal.width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Line::from(hint_spans)).style(Style::default().bg(Theme::MODAL_BG)),
            hint_area,
        );
    }
}
