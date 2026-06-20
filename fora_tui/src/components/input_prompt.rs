//! A centred single-line text-input modal.
//!
//! Used by the Files sub-tab to ask for a download destination. The
//! caller drives it with [`InputPrompt::open`] and feeds keys via
//! [`InputPrompt::handle_key`], acting on the returned [`InputOutcome`].

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::theme::Theme;

/// Outcome of feeding a key to an [`InputPrompt`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputOutcome {
    /// Still editing; the key was consumed.
    Pending,
    /// The user submitted the trimmed value (prompt now closed).
    Submitted(String),
    /// The user cancelled with `Esc` (prompt now closed).
    Cancelled,
}

/// A modal prompt with a title and one editable line of text.
#[derive(Debug, Default)]
pub struct InputPrompt {
    pub active: bool,
    title: String,
    value: String,
    /// Cursor position as a character (not byte) offset into `value`.
    cursor: usize,
}

impl InputPrompt {
    /// Open the prompt with a `title` and an `initial` value. The cursor
    /// is placed at the end of the initial value.
    pub fn open(&mut self, title: impl Into<String>, initial: impl Into<String>) {
        self.title = title.into();
        self.value = initial.into();
        self.cursor = self.char_len();
        self.active = true;
    }

    /// Close the prompt and clear its contents.
    pub fn close(&mut self) {
        self.active = false;
        self.value.clear();
        self.cursor = 0;
    }

    /// The current (untrimmed) value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Handle a key event. Returns [`InputOutcome::Pending`] while
    /// editing, or a terminal outcome (closing the prompt) on submit /
    /// cancel.
    pub fn handle_key(&mut self, key: KeyEvent) -> InputOutcome {
        if !self.active {
            return InputOutcome::Pending;
        }
        match key.code {
            // Suppress only pure Ctrl combos (e.g. Ctrl+C). Plain Alt and
            // AltGr (reported as Ctrl+Alt on some layouts, used to type
            // characters like `\`) must still insert their character.
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.insert_char(c);
                InputOutcome::Pending
            }
            KeyCode::Backspace => {
                self.backspace();
                InputOutcome::Pending
            }
            KeyCode::Delete => {
                self.delete();
                InputOutcome::Pending
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                InputOutcome::Pending
            }
            KeyCode::Right => {
                self.cursor = (self.cursor + 1).min(self.char_len());
                InputOutcome::Pending
            }
            KeyCode::Home => {
                self.cursor = 0;
                InputOutcome::Pending
            }
            KeyCode::End => {
                self.cursor = self.char_len();
                InputOutcome::Pending
            }
            KeyCode::Enter => {
                let value = self.value.trim().to_string();
                self.close();
                InputOutcome::Submitted(value)
            }
            KeyCode::Esc => {
                self.close();
                InputOutcome::Cancelled
            }
            _ => InputOutcome::Pending,
        }
    }

    fn char_len(&self) -> usize {
        self.value.chars().count()
    }

    /// Byte offset of the `char_idx`-th character (or the end of the
    /// string when `char_idx` is past the last character).
    fn byte_index(&self, char_idx: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.value.len())
    }

    fn insert_char(&mut self, c: char) {
        let bi = self.byte_index(self.cursor);
        self.value.insert(bi, c);
        self.cursor += 1;
    }

    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let bi = self.byte_index(self.cursor);
            self.value.remove(bi);
        }
    }

    fn delete(&mut self) {
        if self.cursor < self.char_len() {
            let bi = self.byte_index(self.cursor);
            self.value.remove(bi);
        }
    }

    /// Render the prompt centred within `area`.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.active || area.width == 0 || area.height == 0 {
            return;
        }

        // Never exceed the available area (and never let the 70% width fall
        // below a usable minimum without overflowing a narrow terminal).
        // Compute the 70% in u32 to avoid any u16 multiply overflow.
        let modal_w = ((area.width as u32 * 7 / 10) as u16)
            .max(24)
            .min(area.width)
            .max(1);
        let modal_h = area.height.clamp(1, 4);
        let modal = Rect {
            x: area.x + area.width.saturating_sub(modal_w) / 2,
            y: area.y + area.height.saturating_sub(modal_h) / 2,
            width: modal_w,
            height: modal_h,
        };

        frame.render_widget(Clear, modal);

        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MODAL_BORDER))
            .style(Style::default().bg(Theme::MODAL_BG));
        let inner = block.inner(modal);
        frame.render_widget(block, modal);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner);
        let input_area = rows[0];

        // Horizontal scroll so the cursor stays visible.
        let width = input_area.width as usize;
        let chars: Vec<char> = self.value.chars().collect();
        let cursor = self.cursor.min(chars.len());
        let start = if width > 0 && cursor >= width {
            // Scroll so the cursor stays in view. Using `cursor - width`
            // (not `+ 1`) avoids an empty visible slice when the area is a
            // single column and doesn't scroll a character earlier than
            // necessary when `cursor == width`.
            cursor - width
        } else {
            0
        };
        let visible: String = chars[start..].iter().take(width).collect();
        frame.render_widget(
            Paragraph::new(visible).style(Style::default().fg(Theme::FG)),
            input_area,
        );

        let cursor_x = input_area.x + (cursor - start) as u16;
        // Clamp to the last in-bounds column so the cursor never lands on
        // the modal border (one cell past the input area).
        let max_x = input_area.x + input_area.width.saturating_sub(1);
        frame.set_cursor_position((cursor_x.min(max_x), input_area.y));

        if rows.len() > 1 && rows[1].height > 0 {
            frame.render_widget(
                Paragraph::new("Enter: confirm    Esc: cancel")
                    .style(Style::default().fg(Theme::DIM)),
                rows[1],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn type_str(prompt: &mut InputPrompt, s: &str) {
        for c in s.chars() {
            prompt.handle_key(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn open_places_cursor_at_end_of_initial() {
        let mut p = InputPrompt::default();
        p.open("Save to", "./out.txt");
        assert!(p.active);
        assert_eq!(p.value(), "./out.txt");
        // Typing appends at the end.
        type_str(&mut p, "!");
        assert_eq!(p.value(), "./out.txt!");
    }

    #[test]
    fn typing_inserts_at_cursor() {
        let mut p = InputPrompt::default();
        p.open("t", "");
        type_str(&mut p, "abc");
        p.handle_key(key(KeyCode::Left));
        p.handle_key(key(KeyCode::Char('X')));
        assert_eq!(p.value(), "abXc");
    }

    #[test]
    fn backspace_and_delete() {
        let mut p = InputPrompt::default();
        p.open("t", "abc");
        p.handle_key(key(KeyCode::Backspace));
        assert_eq!(p.value(), "ab");
        p.handle_key(key(KeyCode::Home));
        p.handle_key(key(KeyCode::Delete));
        assert_eq!(p.value(), "b");
    }

    #[test]
    fn enter_submits_trimmed_and_closes() {
        let mut p = InputPrompt::default();
        p.open("t", "  spaced  ");
        let outcome = p.handle_key(key(KeyCode::Enter));
        assert_eq!(outcome, InputOutcome::Submitted("spaced".to_string()));
        assert!(!p.active);
    }

    #[test]
    fn esc_cancels_and_closes() {
        let mut p = InputPrompt::default();
        p.open("t", "abc");
        let outcome = p.handle_key(key(KeyCode::Esc));
        assert_eq!(outcome, InputOutcome::Cancelled);
        assert!(!p.active);
    }

    #[test]
    fn ctrl_char_is_not_inserted() {
        let mut p = InputPrompt::default();
        p.open("t", "");
        p.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert_eq!(p.value(), "");
    }

    #[test]
    fn altgr_char_is_inserted() {
        // AltGr is reported as Ctrl+Alt on some layouts and is used to type
        // characters like `\`; it must still insert.
        let mut p = InputPrompt::default();
        p.open("t", "");
        p.handle_key(KeyEvent::new(
            KeyCode::Char('\\'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        ));
        p.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT));
        assert_eq!(p.value(), "\\x");
    }

    #[test]
    fn inactive_prompt_ignores_keys() {
        let mut p = InputPrompt::default();
        assert_eq!(p.handle_key(key(KeyCode::Char('a'))), InputOutcome::Pending);
        assert_eq!(p.value(), "");
    }
}
