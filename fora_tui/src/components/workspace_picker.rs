use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::config::WorkspaceConfig;
use crate::theme::Theme;

/// Modal popup for selecting a workspace.
#[derive(Debug)]
pub struct WorkspacePicker {
    pub active: bool,
    pub workspaces: Vec<WorkspaceConfig>,
    state: ListState,
}

impl WorkspacePicker {
    pub fn new(workspaces: Vec<WorkspaceConfig>) -> Self {
        let mut state = ListState::default();
        if !workspaces.is_empty() {
            state.select(Some(0));
        }
        Self {
            active: false,
            workspaces,
            state,
        }
    }

    pub fn open(&mut self) {
        self.active = true;
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    /// Handle a key event. Returns Some(index) if a workspace was selected.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<usize> {
        if !self.active {
            return None;
        }

        match key.code {
            KeyCode::Esc => {
                self.close();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self
                    .state
                    .selected()
                    .map(|i| i.saturating_sub(1))
                    .unwrap_or(0);
                self.state.select(Some(i));
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self
                    .state
                    .selected()
                    .map(|i| (i + 1).min(self.workspaces.len().saturating_sub(1)))
                    .unwrap_or(0);
                self.state.select(Some(i));
                None
            }
            KeyCode::Enter => {
                let selected = self.state.selected();
                self.close();
                selected
            }
            _ => None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.active {
            return;
        }

        let modal = centered_rect(50, 60, area);

        frame.render_widget(Clear, modal);

        let block = Block::default()
            .title(" Select Workspace ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::MODAL_BORDER))
            .style(Style::default().bg(Theme::MODAL_BG));

        let items: Vec<ListItem> = self
            .workspaces
            .iter()
            .map(|ws| {
                let lines = vec![
                    Line::from(Span::styled(
                        &ws.name,
                        Style::default()
                            .fg(Theme::ACCENT)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("  {} / {}", ws.resource_group, ws.workspace_name),
                        Style::default().fg(Theme::DIM),
                    )),
                ];
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Theme::MODAL_SELECTED_BG));

        frame.render_stateful_widget(list, modal, &mut self.state);
    }
}

/// Confirmation dialog for destructive actions.
#[derive(Debug, Default)]
pub struct ConfirmDialog {
    pub active: bool,
    pub message: String,
    pub selected_yes: bool,
}

impl ConfirmDialog {
    pub fn show(&mut self, message: String) {
        self.active = true;
        self.message = message;
        self.selected_yes = false;
    }

    /// Handle a key event. Returns Some(true) for confirm, Some(false) for cancel.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<bool> {
        if !self.active {
            return None;
        }

        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                self.selected_yes = !self.selected_yes;
                None
            }
            KeyCode::Enter => {
                let result = self.selected_yes;
                self.active = false;
                Some(result)
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                self.active = false;
                Some(false)
            }
            KeyCode::Char('y') => {
                self.active = false;
                Some(true)
            }
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.active {
            return;
        }

        let modal = centered_rect(40, 20, area);
        frame.render_widget(Clear, modal);

        let block = Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::WARNING))
            .style(Style::default().bg(Theme::MODAL_BG));

        let chunks =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(block.inner(modal));

        let message = Paragraph::new(self.message.as_str()).style(Style::default().fg(Theme::FG));

        let yes_style = if self.selected_yes {
            Style::default()
                .fg(Theme::SUCCESS)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::DIM)
        };
        let no_style = if !self.selected_yes {
            Style::default()
                .fg(Theme::ERROR)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::DIM)
        };

        let buttons = Line::from(vec![
            Span::styled(" [Y]es ", yes_style),
            Span::raw("  "),
            Span::styled(" [N]o ", no_style),
        ]);

        frame.render_widget(block, modal);
        frame.render_widget(message, chunks[0]);
        frame.render_widget(Paragraph::new(buttons), chunks[1]);
    }
}

/// Create a centered rectangle with the given percentage of parent.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
