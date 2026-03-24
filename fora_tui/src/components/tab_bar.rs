use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use ratatui::Frame;

use crate::theme::Theme;

/// Renders the tab bar at the top of the screen.
pub fn render_tab_bar(frame: &mut Frame, area: Rect, titles: &[&str], active: usize) {
    let tab_titles: Vec<Line> = titles
        .iter()
        .enumerate()
        .map(|(i, title)| {
            if i == active {
                Line::from(Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .fg(Theme::TAB_ACTIVE_FG)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {} ", title),
                    Style::default().fg(Theme::TAB_INACTIVE_FG),
                ))
            }
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .select(active)
        .highlight_style(
            Style::default()
                .fg(Theme::TAB_ACTIVE_FG)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled("│", Style::default().fg(Theme::BORDER)));

    frame.render_widget(tabs, area);
}
