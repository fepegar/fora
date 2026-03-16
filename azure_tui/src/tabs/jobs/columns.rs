
use crate::theme;
use crate::widgets::table::ColumnDef;

use super::state::JobRow;

pub fn default_columns() -> Vec<ColumnDef<JobRow>> {
    vec![
        ColumnDef::new("display_name", "Display Name", (|r: &JobRow| {
            r.display_name.clone()
        }) as fn(&JobRow) -> String, 30)
        .with_min_width(10),

        ColumnDef::new("status", "Status", (|r: &JobRow| {
            let sym = theme::job_status_symbol(&r.status);
            let label = format!("{:?}", r.status);
            format!("{} {}", sym, label)
        }) as fn(&JobRow) -> String, 16)
        .with_style(|r: &JobRow| theme::job_status_style(&r.status))
        .with_min_width(6),

        ColumnDef::new("experiment", "Experiment", (|r: &JobRow| {
            r.experiment_name.clone()
        }) as fn(&JobRow) -> String, 20)
        .with_min_width(8),

        ColumnDef::new("type", "Type", (|r: &JobRow| {
            r.job_type.clone()
        }) as fn(&JobRow) -> String, 10)
        .with_min_width(6),

        ColumnDef::new("compute", "Compute", (|r: &JobRow| {
            r.compute_target.clone()
        }) as fn(&JobRow) -> String, 16)
        .with_min_width(6),

        ColumnDef::new("created", "Created", (|r: &JobRow| {
            r.created_at
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "—".to_string())
        }) as fn(&JobRow) -> String, 18)
        .with_min_width(10),

        ColumnDef::new("id", "Job ID", (|r: &JobRow| {
            r.id.clone()
        }) as fn(&JobRow) -> String, 20)
        .hidden()
        .with_min_width(8),
    ]
}
