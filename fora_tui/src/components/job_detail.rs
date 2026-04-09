use std::collections::HashMap;

use azure_ml::models::JobStatus;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;

use crate::theme::{self, Theme};

/// Common job data for the detail pane, usable across tabs.
pub struct JobDetail<'a> {
    pub id: &'a str,
    pub display_name: &'a str,
    pub experiment_name: &'a str,
    pub job_type: &'a str,
    pub status: &'a JobStatus,
    pub compute_target: &'a str,
    pub created_at: Option<&'a str>,
    pub runtime: Option<&'a str>,
    pub command: Option<&'a str>,
    pub environment_id: Option<&'a str>,
    pub description: Option<&'a str>,
    pub tags: Option<&'a HashMap<String, String>>,
}

/// Renders the detail pane for a selected job.
/// Returns the total content line count (for scroll management).
pub fn render_job_detail(frame: &mut Frame, area: Rect, job: &JobDetail, scroll: u16) -> u16 {
    let block = Block::default()
        .title(format!(" {} ", job.display_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER_ACTIVE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 4 || inner.height < 2 {
        return 0;
    }

    let content_width = inner.width.saturating_sub(2); // 1 left pad + 1 right (scrollbar area)
    let mut lines: Vec<Line> = Vec::new();

    // ── General ──────────────────
    let created = job.created_at.unwrap_or("—");
    let runtime = job.runtime.unwrap_or("—");
    let general_fields: Vec<(&str, &str)> = vec![
        ("Job ID", job.id),
        ("Display Name", job.display_name),
        ("Experiment", job.experiment_name),
        ("Type", job.job_type),
        ("Compute", job.compute_target),
        ("Created", created),
        ("Runtime", runtime),
    ];
    let label_w = max_label_width(&general_fields).max("Status".len());

    add_section_header(&mut lines, "General", content_width);
    for (label, value) in &general_fields {
        add_aligned_field(&mut lines, label, value, label_w);
    }
    add_status_field(&mut lines, "Status", job.status, label_w);

    // ── Execution ──────────────────
    if job.command.is_some() || job.environment_id.is_some() {
        lines.push(Line::from(""));
        add_section_header(&mut lines, "Execution", content_width);

        let mut exec_labels: Vec<&str> = Vec::new();
        if job.command.is_some() {
            exec_labels.push("Command");
        }
        if job.environment_id.is_some() {
            exec_labels.push("Environment");
        }
        let exec_w = exec_labels.iter().map(|l| l.len()).max().unwrap_or(0);

        if let Some(cmd) = job.command {
            add_aligned_field(&mut lines, "Command", cmd, exec_w);
        }
        if let Some(env) = job.environment_id {
            let short_env = extract_environment_name(env);
            add_aligned_field(&mut lines, "Environment", &short_env, exec_w);
        }
    }

    // ── Description ──────────────────
    if let Some(desc) = job.description {
        if !desc.is_empty() {
            lines.push(Line::from(""));
            add_section_header(&mut lines, "Description", content_width);
            lines.push(Line::from(Span::styled(
                format!(" {}", desc),
                Style::default().fg(Theme::FG),
            )));
        }
    }

    // ── Tags ──────────────────
    if let Some(tags) = job.tags {
        if !tags.is_empty() {
            lines.push(Line::from(""));
            add_section_header(&mut lines, "Tags", content_width);
            let mut sorted_tags: Vec<(&String, &String)> = tags.iter().collect();
            sorted_tags.sort_by_key(|(k, _)| k.as_str());
            let tag_w = sorted_tags.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
            for (k, v) in &sorted_tags {
                add_aligned_field(&mut lines, k, v, tag_w);
            }
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

    // Scrollbar when content overflows
    if total_lines > inner.height {
        let max_scroll = total_lines.saturating_sub(inner.height) as usize;
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
    }

    total_lines
}

// ── Shared detail-pane helpers ──────────────────────────────────────────────

/// Section header: `── Title ────────────────`
pub fn add_section_header(lines: &mut Vec<Line<'_>>, title: &str, width: u16) {
    let prefix = "── ";
    let title_part = format!("{} ", title);
    let used = prefix.len() + title_part.len();
    let remaining = (width as usize).saturating_sub(used);
    let suffix: String = "─".repeat(remaining);

    lines.push(Line::from(vec![
        Span::styled(prefix, Style::default().fg(Theme::ACCENT)),
        Span::styled(
            title_part,
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(suffix, Style::default().fg(Theme::ACCENT)),
    ]));
}

/// Field with right-aligned label: `     Label  value`
pub fn add_aligned_field(
    lines: &mut Vec<Line<'_>>,
    label: &str,
    value: &str,
    max_label_width: usize,
) {
    let padded = format!("{:>width$}", label, width = max_label_width);
    lines.push(Line::from(vec![
        Span::styled(format!(" {} ", padded), Style::default().fg(Theme::DIM)),
        Span::styled(value.to_string(), Style::default().fg(Theme::FG)),
    ]));
}

/// Compute the max label width from a set of (label, value) pairs.
pub fn max_label_width(fields: &[(&str, &str)]) -> usize {
    fields.iter().map(|(l, _)| l.len()).max().unwrap_or(0)
}

/// Status field with colored symbol.
fn add_status_field(
    lines: &mut Vec<Line<'_>>,
    label: &str,
    status: &JobStatus,
    max_label_width: usize,
) {
    let padded = format!("{:>width$}", label, width = max_label_width);
    let symbol = theme::job_status_symbol(status);
    let color = theme::job_status_color(status);
    let name = theme::job_status_display_name(status);
    lines.push(Line::from(vec![
        Span::styled(format!(" {} ", padded), Style::default().fg(Theme::DIM)),
        Span::styled(format!("{} {}", symbol, name), Style::default().fg(color)),
    ]));
}

/// Extracts a short environment name from a full ARM resource ID.
/// Input:  `.../environments/MyEnv/versions/3`
/// Output: `MyEnv:3`
/// Falls back to the original string if the pattern doesn't match.
fn extract_environment_name(env_id: &str) -> String {
    // Look for ".../environments/<name>/versions/<ver>"
    let parts: Vec<&str> = env_id.split('/').collect();
    if let Some(env_idx) = parts.iter().position(|&p| p == "environments") {
        let name = parts.get(env_idx + 1).copied().unwrap_or(env_id);
        if let Some(ver_idx) = parts.iter().position(|&p| p == "versions") {
            let version = parts.get(ver_idx + 1).copied().unwrap_or("latest");
            format!("{}:{}", name, version)
        } else {
            name.to_string()
        }
    } else {
        env_id.to_string()
    }
}

/// Renders a scrollbar and returns the clamped scroll position.
/// Reusable across detail panes.
pub fn render_detail_scrollbar(frame: &mut Frame, inner: Rect, scroll: u16, total_lines: u16) {
    if total_lines > inner.height {
        let max_scroll = total_lines.saturating_sub(inner.height) as usize;
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
    }
}
