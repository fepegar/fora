use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::theme::Theme;

use super::state::ComputeRow;

/// Renders the detail pane for a selected compute resource.
pub fn render_compute_detail(frame: &mut Frame, area: Rect, compute: &ComputeRow) {
    let block = Block::default()
        .title(format!(" {} ", compute.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER_ACTIVE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    add_section(&mut lines, "General");
    add_field(&mut lines, "Name", &compute.name);
    add_field(&mut lines, "Type", &compute.compute_type);
    add_field(
        &mut lines,
        "VM Size",
        compute.vm_size.as_deref().unwrap_or("—"),
    );
    add_field(
        &mut lines,
        "Location",
        compute.location.as_deref().unwrap_or("—"),
    );
    add_field(
        &mut lines,
        "Priority",
        compute.vm_priority.as_deref().unwrap_or("—"),
    );
    add_field(
        &mut lines,
        "Provisioning",
        &compute
            .provisioning_state
            .as_ref()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| "—".to_string()),
    );
    add_field(
        &mut lines,
        "Allocation",
        &compute
            .allocation_state
            .as_ref()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|| "—".to_string()),
    );

    lines.push(Line::from(""));
    add_section(&mut lines, "Nodes");
    add_field(&mut lines, "Current", &opt_i32(compute.current_node_count));
    add_field(&mut lines, "Min", &opt_i32(compute.min_nodes));
    add_field(&mut lines, "Max", &opt_i32(compute.max_nodes));
    add_field(&mut lines, "Idle", &opt_i32(compute.idle_nodes));
    add_field(&mut lines, "Running", &opt_i32(compute.running_nodes));
    add_field(&mut lines, "Preparing", &opt_i32(compute.preparing_nodes));
    add_field(&mut lines, "Unusable", &opt_i32(compute.unusable_nodes));

    if let Some(ref desc) = compute.description {
        if !desc.is_empty() {
            lines.push(Line::from(""));
            add_section(&mut lines, "Description");
            lines.push(Line::from(Span::styled(
                desc.as_str(),
                Style::default().fg(Theme::FG),
            )));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn opt_i32(v: Option<i32>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "—".to_string())
}

fn add_section(lines: &mut Vec<Line<'_>>, title: &'static str) {
    lines.push(Line::from(Span::styled(
        title,
        Style::default()
            .fg(Theme::ACCENT)
            .add_modifier(Modifier::BOLD),
    )));
}

fn add_field(lines: &mut Vec<Line<'_>>, label: &str, value: &str) {
    lines.push(Line::from(vec![
        Span::styled(format!("  {}: ", label), Style::default().fg(Theme::DIM)),
        Span::styled(value.to_string(), Style::default().fg(Theme::FG)),
    ]));
}
