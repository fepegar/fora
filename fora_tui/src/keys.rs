//! Key definitions and utilities for the TUI application.
//!
//! This module centralizes key bindings to make them easier to maintain
//! and modify across the entire application.

/// Constants for help text
pub mod help_text {
    pub const SELECT_KEYS: &str = "Enter/Space";
    pub const NAV_KEYS: &str = "↕";
    pub const SWITCH_PANEL_KEYS: &str = "↔";
}

/// Key combination patterns as match arms for use in match statements
#[macro_export]
macro_rules! select_keys {
    () => {
        KeyCode::Enter | KeyCode::Char(' ')
    };
}

#[macro_export]
macro_rules! nav_up_keys {
    () => {
        KeyCode::Up | KeyCode::Char('k')
    };
}

#[macro_export]
macro_rules! nav_down_keys {
    () => {
        KeyCode::Down | KeyCode::Char('j')
    };
}

#[macro_export]
macro_rules! nav_left_keys {
    () => {
        KeyCode::Left | KeyCode::Char('h')
    };
}

#[macro_export]
macro_rules! nav_right_keys {
    () => {
        KeyCode::Right | KeyCode::Char('l')
    };
}

#[macro_export]
macro_rules! escape_keys {
    () => {
        KeyCode::Esc | KeyCode::Char('q')
    };
}
