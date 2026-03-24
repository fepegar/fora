use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::components::job_detail::{
    add_aligned_field, add_section_header, max_label_width, render_detail_scrollbar,
};
use crate::theme::Theme;

use super::state::ComputeRow;

/// Renders the detail pane for a selected compute resource.
/// Returns the total content line count (for scroll management).
pub fn render_compute_detail(
    frame: &mut Frame,
    area: Rect,
    compute: &ComputeRow,
    scroll: u16,
) -> u16 {
    let block = Block::default()
        .title(format!(" {} ", compute.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER_ACTIVE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 4 || inner.height < 2 {
        return 0;
    }

    let content_width = inner.width.saturating_sub(2);
    let mut lines: Vec<Line> = Vec::new();

    // ── General ──────────────────
    let vm_size = compute.vm_size.as_deref().unwrap_or("—");
    let location = compute.location.as_deref().unwrap_or("—");
    let priority = compute.vm_priority.as_deref().unwrap_or("—");
    let provisioning = compute
        .provisioning_state
        .as_ref()
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|| "—".to_string());
    let allocation = compute
        .allocation_state
        .as_ref()
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|| "—".to_string());

    let general_fields: Vec<(&str, &str)> = vec![
        ("Name", &compute.name),
        ("Type", &compute.compute_type),
        ("VM Size", vm_size),
        ("Location", location),
        ("Priority", priority),
        ("Provisioning", &provisioning),
        ("Allocation", &allocation),
    ];
    let label_w = max_label_width(&general_fields);

    add_section_header(&mut lines, "General", content_width);
    for (label, value) in &general_fields {
        add_aligned_field(&mut lines, label, value, label_w);
    }

    // ── Nodes ──────────────────
    lines.push(Line::from(""));
    let node_fields: Vec<(&str, String)> = vec![
        ("Current", opt_i32(compute.current_node_count)),
        ("Min", opt_i32(compute.min_nodes)),
        ("Max", opt_i32(compute.max_nodes)),
        ("Idle", opt_i32(compute.idle_nodes)),
        ("Running", opt_i32(compute.running_nodes)),
        ("Preparing", opt_i32(compute.preparing_nodes)),
        ("Unusable", opt_i32(compute.unusable_nodes)),
    ];
    let node_label_w = node_fields.iter().map(|(l, _)| l.len()).max().unwrap_or(0);

    add_section_header(&mut lines, "Nodes", content_width);
    for (label, value) in &node_fields {
        add_aligned_field(&mut lines, label, value, node_label_w);
    }

    // ── Description ──────────────────
    if let Some(ref desc) = compute.description {
        if !desc.is_empty() {
            lines.push(Line::from(""));
            add_section_header(&mut lines, "Description", content_width);
            lines.push(Line::from(Span::styled(
                format!(" {}", desc),
                Style::default().fg(Theme::FG),
            )));
        }
    }

    let total_lines = lines.len() as u16;

    // Content area with left padding
    let content_area = Rect {
        x: inner.x + 1,
        y: inner.y,
        width: content_width,
        height: inner.height,
    };

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(paragraph, content_area);

    render_detail_scrollbar(frame, inner, scroll, total_lines);

    total_lines
}

fn opt_i32(v: Option<i32>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "—".to_string())
}
