use ratatui::style::{Color, Modifier, Style};

use azure_ml::models::JobStatus;

/// Application colour palette.
pub struct Theme;

impl Theme {
    // Base colours
    pub const BG: Color = Color::Reset;
    pub const FG: Color = Color::White;
    pub const DIM: Color = Color::DarkGray;
    pub const ACCENT: Color = Color::Cyan;
    pub const ACCENT_DIM: Color = Color::Blue;
    pub const BORDER: Color = Color::DarkGray;
    pub const BORDER_ACTIVE: Color = Color::Cyan;

    // Status colours
    pub const SUCCESS: Color = Color::Green;
    pub const ERROR: Color = Color::Red;
    pub const WARNING: Color = Color::Yellow;
    pub const INFO: Color = Color::Blue;
    pub const RUNNING: Color = Color::Cyan;
    pub const QUEUED: Color = Color::DarkGray;

    // Table
    pub const TABLE_HEADER_FG: Color = Color::Cyan;
    pub const TABLE_SELECTED_BG: Color = Color::DarkGray;
    pub const TABLE_STRIPE_BG: Color = Color::Rgb(30, 30, 40);

    // Tab bar
    pub const TAB_ACTIVE_FG: Color = Color::Cyan;
    pub const TAB_INACTIVE_FG: Color = Color::DarkGray;

    // Help bar
    pub const HELP_KEY_FG: Color = Color::Cyan;
    pub const HELP_DESC_FG: Color = Color::DarkGray;

    // Modal
    pub const MODAL_BG: Color = Color::Rgb(30, 30, 46);
    pub const MODAL_BORDER: Color = Color::Cyan;
    pub const MODAL_SELECTED_BG: Color = Color::Rgb(60, 60, 80);
}

// --- Style helpers ---

pub fn header_style() -> Style {
    Style::default()
        .fg(Theme::TABLE_HEADER_FG)
        .add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().bg(Theme::TABLE_SELECTED_BG)
}

pub fn stripe_style(index: usize) -> Style {
    if index % 2 == 1 {
        Style::default().bg(Theme::TABLE_STRIPE_BG)
    } else {
        Style::default()
    }
}

pub fn border_style(active: bool) -> Style {
    Style::default().fg(if active {
        Theme::BORDER_ACTIVE
    } else {
        Theme::BORDER
    })
}

// --- Status symbols and colours ---

pub fn job_status_symbol(status: &JobStatus) -> &'static str {
    match status {
        JobStatus::Completed => "✓",
        JobStatus::Failed => "✗",
        JobStatus::Running => "●",
        JobStatus::Queued => "◯",
        JobStatus::Preparing | JobStatus::Provisioning => "⏳",
        JobStatus::CancelRequested | JobStatus::Canceled => "✕",
        JobStatus::Starting | JobStatus::Finalizing => "▸",
        JobStatus::NotStarted => "○",
        JobStatus::Paused => "⏸",
        JobStatus::NotResponding => "⚠",
        _ => "?",
    }
}

pub fn job_status_color(status: &JobStatus) -> Color {
    match status {
        JobStatus::Completed => Theme::SUCCESS,
        JobStatus::Failed => Theme::ERROR,
        JobStatus::Running | JobStatus::Finalizing => Theme::RUNNING,
        JobStatus::Queued | JobStatus::NotStarted => Theme::QUEUED,
        JobStatus::Preparing | JobStatus::Provisioning | JobStatus::Starting => Theme::WARNING,
        JobStatus::CancelRequested | JobStatus::Canceled => Theme::DIM,
        JobStatus::NotResponding => Theme::ERROR,
        JobStatus::Paused => Theme::WARNING,
        _ => Theme::DIM,
    }
}

pub fn job_status_style(status: &JobStatus) -> Style {
    Style::default().fg(job_status_color(status))
}
