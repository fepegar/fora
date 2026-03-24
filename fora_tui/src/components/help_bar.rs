use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::theme::Theme;

/// Renders the help bar at the bottom of the screen with context-sensitive key hints.
pub fn render_help_bar(frame: &mut Frame, area: Rect, hints: &[(&str, &str)]) {
    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut result = vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default().fg(Theme::HELP_KEY_FG),
                ),
                Span::styled(
                    format!("{} ", desc),
                    Style::default().fg(Theme::HELP_DESC_FG),
                ),
            ];
            if i < hints.len() - 1 {
                result.push(Span::styled(" ", Style::default().fg(Theme::BORDER)));
            }
            result
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
