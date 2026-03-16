use ratatui::style::Style;

use crate::theme::Theme;
use crate::widgets::table::ColumnDef;

use super::state::ComputeRow;

pub fn default_columns() -> Vec<ColumnDef<ComputeRow>> {
    vec![
        ColumnDef::new("name", "Name", (|r: &ComputeRow| r.name.clone()) as fn(&ComputeRow) -> String, 20).with_min_width(8),
        ColumnDef::new("type", "Type", (|r: &ComputeRow| r.compute_type.clone()) as fn(&ComputeRow) -> String, 16).with_min_width(6),
        ColumnDef::new("vm_size", "VM Size", (|r: &ComputeRow| {
            r.vm_size.clone().unwrap_or_else(|| "—".to_string())
        }) as fn(&ComputeRow) -> String, 20)
        .with_min_width(8),
        ColumnDef::new("state", "State", (|r: &ComputeRow| {
            r.provisioning_state
                .as_ref()
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "—".to_string())
        }) as fn(&ComputeRow) -> String, 12)
        .with_style(|r: &ComputeRow| {
            let color = match r.provisioning_state {
                Some(azure_ml::models::ProvisioningState::Succeeded) => Theme::SUCCESS,
                Some(azure_ml::models::ProvisioningState::Failed) => Theme::ERROR,
                Some(azure_ml::models::ProvisioningState::Creating)
                | Some(azure_ml::models::ProvisioningState::Updating) => Theme::WARNING,
                _ => Theme::DIM,
            };
            Style::default().fg(color)
        })
        .with_min_width(6),
        ColumnDef::new("running", "Running", (|r: &ComputeRow| {
            r.running_nodes
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".to_string())
        }) as fn(&ComputeRow) -> String, 8)
        .with_min_width(4),
        ColumnDef::new("idle", "Idle", (|r: &ComputeRow| {
            r.idle_nodes
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".to_string())
        }) as fn(&ComputeRow) -> String, 6)
        .with_min_width(4),
        ColumnDef::new("nodes", "Nodes", (|r: &ComputeRow| {
            r.current_node_count
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".to_string())
        }) as fn(&ComputeRow) -> String, 6)
        .with_min_width(4),
        ColumnDef::new("max", "Max", (|r: &ComputeRow| {
            r.max_nodes
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".to_string())
        }) as fn(&ComputeRow) -> String, 5)
        .with_min_width(4),
        ColumnDef::new("priority", "Priority", (|r: &ComputeRow| {
            r.vm_priority.clone().unwrap_or_else(|| "—".to_string())
        }) as fn(&ComputeRow) -> String, 12)
        .hidden()
        .with_min_width(6),
    ]
}
