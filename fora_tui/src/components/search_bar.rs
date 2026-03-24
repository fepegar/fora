use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::theme::Theme;

/// An overlay search bar widget.
#[derive(Debug, Default)]
pub struct SearchBar {
    pub query: String,
    pub active: bool,
    cursor_pos: usize,
}

impl SearchBar {
    pub fn open(&mut self) {
        self.active = true;
        self.cursor_pos = self.query.len();
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
        self.active = false;
    }

    /// Handle a key event. Returns true if consumed.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.active {
            return false;
        }

        match key.code {
            KeyCode::Char(c) => {
                self.query.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.query.remove(self.cursor_pos);
                }
            }
            KeyCode::Left => {
                self.cursor_pos = self.cursor_pos.saturating_sub(1);
            }
            KeyCode::Right => {
                self.cursor_pos = (self.cursor_pos + 1).min(self.query.len());
            }
            KeyCode::Esc => {
                self.clear();
            }
            _ => {}
        }
        true
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.active {
            return;
        }

        let search_area = Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(3),
            width: area.width,
            height: 3.min(area.height),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::ACCENT))
            .title(" Search (Esc to close) ");

        let text = format!("/{}", &self.query);
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, search_area);
    }
}
