use std::collections::HashMap;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::theme::Theme;

/// Common job data for the detail pane, usable across tabs.
pub struct JobDetail<'a> {
    pub id: &'a str,
    pub display_name: &'a str,
    pub experiment_name: &'a str,
    pub job_type: &'a str,
    pub status: &'a str,
    pub compute_target: &'a str,
    pub created_at: Option<&'a str>,
    pub command: Option<&'a str>,
    pub environment_id: Option<&'a str>,
    pub description: Option<&'a str>,
    pub tags: Option<&'a HashMap<String, String>>,
}

/// Renders the detail pane for a selected job.
pub fn render_job_detail(frame: &mut Frame, area: Rect, job: &JobDetail) {
    let block = Block::default()
        .title(format!(" {} ", job.display_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER_ACTIVE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    add_section(&mut lines, "General");
    add_field(&mut lines, "Job ID", job.id);
    add_field(&mut lines, "Display Name", job.display_name);
    add_field(&mut lines, "Experiment", job.experiment_name);
    add_field(&mut lines, "Type", job.job_type);
    add_field(&mut lines, "Status", job.status);
    add_field(&mut lines, "Compute", job.compute_target);
    add_field(&mut lines, "Created", job.created_at.unwrap_or("—"));

    if let Some(cmd) = job.command {
        lines.push(Line::from(""));
        add_section(&mut lines, "Command");
        add_field(&mut lines, "Command", cmd);
    }

    if let Some(env) = job.environment_id {
        add_field(&mut lines, "Environment", env);
    }

    if let Some(desc) = job.description {
        if !desc.is_empty() {
            lines.push(Line::from(""));
            add_section(&mut lines, "Description");
            lines.push(Line::from(Span::styled(
                desc.to_string(),
                Style::default().fg(Theme::FG),
            )));
        }
    }

    if let Some(tags) = job.tags {
        if !tags.is_empty() {
            lines.push(Line::from(""));
            add_section(&mut lines, "Tags");
            for (k, v) in tags {
                add_field(&mut lines, k, v);
            }
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
        Span::styled(format!("  {}: ", label), Style::default().fg(Theme::DIM)),
        Span::styled(value.to_string(), Style::default().fg(Theme::FG)),
    ]));
}
