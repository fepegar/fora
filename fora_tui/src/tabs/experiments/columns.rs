use crate::format::format_runtime;
use crate::tabs::recent_jobs::state::RecentJobRow;
use crate::theme;
use crate::widgets::table::ColumnDef;

/// Column definitions for job rows within an expanded experiment.
pub fn job_columns(tz: chrono_tz::Tz) -> Vec<ColumnDef<RecentJobRow>> {
    vec![
        ColumnDef::new(
            "display_name",
            "Display Name",
            |r: &RecentJobRow| format!("  {}", r.display_name),
            28,
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
            "runtime",
            "Runtime",
            |r: &RecentJobRow| format_runtime(r.start_time, r.end_time),
            12,
        )
        .with_min_width(6),
        ColumnDef::new(
            "user",
            "User",
            |r: &RecentJobRow| r.user.as_deref().unwrap_or("—").to_string(),
            20,
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
        .hidden()
        .with_min_width(10),
        ColumnDef::new("id", "Job ID", |r: &RecentJobRow| r.id.clone(), 20)
            .hidden()
            .with_min_width(8),
    ]
}
