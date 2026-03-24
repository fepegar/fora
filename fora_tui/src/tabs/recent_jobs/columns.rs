use crate::format::format_runtime;
use crate::theme;
use crate::widgets::table::ColumnDef;

use super::state::RecentJobRow;

pub fn default_columns() -> Vec<ColumnDef<RecentJobRow>> {
    vec![
        ColumnDef::new(
            "display_name",
            "Display Name",
            (|r: &RecentJobRow| r.display_name.clone()) as fn(&RecentJobRow) -> String,
            30,
        )
        .with_min_width(10),
        ColumnDef::new(
            "status",
            "Status",
            (|r: &RecentJobRow| {
                let sym = mlflow_status_symbol(&r.status);
                format!("{} {}", sym, r.status)
            }) as fn(&RecentJobRow) -> String,
            16,
        )
        .with_style(|r: &RecentJobRow| mlflow_status_style(&r.status))
        .with_min_width(6),
        ColumnDef::new(
            "experiment",
            "Experiment",
            (|r: &RecentJobRow| r.experiment_name.clone()) as fn(&RecentJobRow) -> String,
            20,
        )
        .with_min_width(8),
        ColumnDef::new(
            "runtime",
            "Runtime",
            (|r: &RecentJobRow| format_runtime(r.start_time, r.end_time))
                as fn(&RecentJobRow) -> String,
            12,
        )
        .with_min_width(6),
        ColumnDef::new(
            "started",
            "Started",
            (|r: &RecentJobRow| {
                r.start_time
                    .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "—".to_string())
            }) as fn(&RecentJobRow) -> String,
            18,
        )
        .with_min_width(10),
        ColumnDef::new(
            "compute",
            "Compute",
            (|r: &RecentJobRow| r.compute_target.as_deref().unwrap_or("—").to_string())
                as fn(&RecentJobRow) -> String,
            16,
        )
        .with_min_width(6),
        ColumnDef::new(
            "type",
            "Type",
            (|r: &RecentJobRow| r.job_type.as_deref().unwrap_or("—").to_string())
                as fn(&RecentJobRow) -> String,
            10,
        )
        .hidden()
        .with_min_width(6),
        ColumnDef::new(
            "id",
            "Job ID",
            (|r: &RecentJobRow| r.id.clone()) as fn(&RecentJobRow) -> String,
            20,
        )
        .hidden()
        .with_min_width(8),
    ]
}

fn mlflow_status_symbol(status: &str) -> &'static str {
    match status {
        "FINISHED" => "✓",
        "FAILED" => "✗",
        "RUNNING" => "●",
        "KILLED" => "✕",
        "SCHEDULED" | "STARTING" => "◯",
        _ => "?",
    }
}

fn mlflow_status_style(status: &str) -> ratatui::style::Style {
    use ratatui::style::Style;
    match status {
        "FINISHED" => Style::default().fg(theme::Theme::SUCCESS),
        "FAILED" => Style::default().fg(theme::Theme::ERROR),
        "RUNNING" => Style::default().fg(theme::Theme::RUNNING),
        "KILLED" => Style::default().fg(theme::Theme::DIM),
        "SCHEDULED" | "STARTING" => Style::default().fg(theme::Theme::WARNING),
        _ => Style::default().fg(theme::Theme::DIM),
    }
}
