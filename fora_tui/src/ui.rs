use crate::app::App;
use ratatui::{prelude::*, widgets::*};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(f.size());

    // Top bar with tabs (no border)
    draw_tabs(f, chunks[0], app);

    // Main content area - delegate to current tab
    if let Some(tab) = app.get_current_tab() {
        tab.render(f, chunks[1]);
    }
}

fn draw_tabs(f: &mut Frame, area: Rect, app: &mut App) {
    let tab_names: Vec<String> = app.tabs.iter().map(|tab| tab.to_string()).collect();
    let selected_index = app
        .tabs
        .iter()
        .position(|t| *t == app.current_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(tab_names)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                .border_style(Style::default().fg(Color::Gray))
                .border_type(BorderType::Rounded),
        )
        .select(selected_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("·");
    // .divider(" | ");

    f.render_widget(tabs, area);
}
