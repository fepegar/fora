use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::theme::Theme;

use super::state::JobRow;

/// Renders the detail pane for a selected job.
pub fn render_job_detail(frame: &mut Frame, area: Rect, job: &JobRow) {
    let block = Block::default()
        .title(format!(" {} ", job.display_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER_ACTIVE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    add_section(&mut lines, "General");
    add_field(&mut lines, "Job ID", &job.id);
    add_field(&mut lines, "Display Name", &job.display_name);
    add_field(&mut lines, "Experiment", &job.experiment_name);
    add_field(&mut lines, "Type", &job.job_type);
    add_field(
        &mut lines,
        "Status",
        &format!("{:?}", job.status),
    );
    add_field(&mut lines, "Compute", &job.compute_target);
    add_field(
        &mut lines,
        "Created",
        &job.created_at
            .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| "—".to_string()),
    );

    if let Some(ref cmd) = job.command {
        lines.push(Line::from(""));
        add_section(&mut lines, "Command");
        add_field(&mut lines, "Command", cmd);
    }

    if let Some(ref env) = job.environment_id {
        add_field(&mut lines, "Environment", env);
    }

    if let Some(ref desc) = job.description {
        if !desc.is_empty() {
            lines.push(Line::from(""));
            add_section(&mut lines, "Description");
            lines.push(Line::from(Span::styled(
                desc.as_str(),
                Style::default().fg(Theme::FG),
            )));
        }
    }

    if !job.tags.is_empty() {
        lines.push(Line::from(""));
        add_section(&mut lines, "Tags");
        for (k, v) in &job.tags {
            add_field(&mut lines, k, v);
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
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
        Span::styled(
            format!("  {}: ", label),
            Style::default().fg(Theme::DIM),
        ),
        Span::styled(value.to_string(), Style::default().fg(Theme::FG)),
    ]));
}
