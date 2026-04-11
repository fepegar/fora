use crate::format::format_runtime;
use crate::theme;
use crate::widgets::table::ColumnDef;

use super::state::RecentJobRow;

pub fn default_columns(tz: chrono_tz::Tz) -> Vec<ColumnDef<RecentJobRow>> {
    vec![
        ColumnDef::new(
            "display_name",
            "Display Name",
            |r: &RecentJobRow| r.display_name.clone(),
            30,
        )
        .with_min_width(10),
        ColumnDef::new(
            "status",
            "Status",
            |r: &RecentJobRow| {
                let sym = theme::job_status_symbol(&r.status);
                let name = theme::job_status_display_name(&r.status);
                format!("{} {}", sym, name)
            },
            16,
        )
        .with_style(|r: &RecentJobRow| theme::job_status_style(&r.status))
        .with_min_width(6),
        ColumnDef::new(
            "experiment",
            "Experiment",
            |r: &RecentJobRow| r.experiment_name.clone(),
            20,
        )
        .with_min_width(8),
        ColumnDef::new(
            "runtime",
            "Runtime",
            |r: &RecentJobRow| format_runtime(r.start_time, r.end_time),
            12,
        )
        .with_min_width(6),
        ColumnDef::new(
            "started",
            "Started",
            move |r: &RecentJobRow| {
                r.start_time
                    .map(|t| t.with_timezone(&tz).format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "—".to_string())
            },
            18,
        )
        .with_min_width(10),
        ColumnDef::new(
            "compute",
            "Compute",
            |r: &RecentJobRow| r.compute_target.as_deref().unwrap_or("—").to_string(),
            16,
        )
        .with_min_width(6),
        ColumnDef::new(
            "type",
            "Type",
            |r: &RecentJobRow| r.job_type.as_deref().unwrap_or("—").to_string(),
            10,
        )
        .hidden()
        .with_min_width(6),
        ColumnDef::new("id", "Job ID", |r: &RecentJobRow| r.id.clone(), 20)
            .hidden()
            .with_min_width(8),
    ]
}
