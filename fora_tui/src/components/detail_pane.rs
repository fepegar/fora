use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Block, Borders, Chart, Dataset, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Tabs, Wrap,
};
use ratatui::Frame;

use crate::app::Action;
use crate::client::AzureClient;
use crate::components::job_detail::{self, JobDetail};
use crate::tabs::ActionSender;
use crate::theme::Theme;

// ── Types ──────────────────────────────────────────────────────────────

/// Sub-tabs within the detail pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Info,
    Metrics,
}

impl DetailTab {
    const ALL: &[DetailTab] = &[DetailTab::Info, DetailTab::Metrics];

    fn index(self) -> usize {
        match self {
            DetailTab::Info => 0,
            DetailTab::Metrics => 1,
        }
    }

    fn label(self) -> &'static str {
        match self {
            DetailTab::Info => "Info",
            DetailTab::Metrics => "Metrics",
        }
    }
}

/// Result of key handling in the detail pane.
pub enum DetailKeyResult {
    /// The key was consumed by the detail pane.
    Consumed,
    /// The user pressed Esc — the parent should close the pane.
    Close,
    /// The user switched to the Metrics tab and metrics need fetching.
    FetchMetrics {
        run_id: String,
        metric_keys: Vec<String>,
    },
    /// The key was not handled.
    Ignored,
}

/// State of metrics for a given run.
#[derive(Debug, Clone)]
pub enum MetricsState {
    NotFetched,
    Loading,
    LoadingPartial(Vec<MetricSeries>),
    Loaded(Vec<MetricSeries>),
    Error(String),
}

/// A single metric's history, pre-processed for charting.
#[derive(Debug, Clone)]
pub struct MetricSeries {
    pub key: String,
    pub points: Vec<(f64, f64)>,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

// ── Colors for metric charts ───────────────────────────────────────────

const CHART_COLORS: &[Color] = &[
    Color::Cyan,
    Color::Yellow,
    Color::Green,
    Color::Magenta,
    Color::Red,
    Color::Blue,
    Color::LightCyan,
    Color::LightYellow,
];

/// Height of each individual metric chart.
const CHART_HEIGHT: u16 = 12;

// ── DetailPane ─────────────────────────────────────────────────────────

/// Stateful detail pane with sub-tabs (Info, Metrics).
pub struct DetailPane {
    pub active_tab: DetailTab,
    // Info tab state
    info_scroll: u16,
    info_total_lines: u16,
    info_visible_height: u16,
    // Metrics tab state
    metrics_scroll: u16,
    metrics_total_height: u16,
    metrics_visible_height: u16,
    /// Cached metrics per run_id.
    metrics_cache: HashMap<String, MetricsState>,
    /// The run_id currently displayed.
    current_run_id: Option<String>,
}

impl Default for DetailPane {
    fn default() -> Self {
        Self {
            active_tab: DetailTab::Info,
            info_scroll: 0,
            info_total_lines: 0,
            info_visible_height: 0,
            metrics_scroll: 0,
            metrics_total_height: 0,
            metrics_visible_height: 0,
            metrics_cache: HashMap::new(),
            current_run_id: None,
        }
    }
}

impl DetailPane {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset scroll when a new job is selected or the pane is opened.
    pub fn reset(&mut self) {
        self.active_tab = DetailTab::Info;
        self.info_scroll = 0;
        self.metrics_scroll = 0;
        self.current_run_id = None;
    }

    /// Update metrics state from an action.
    pub fn handle_action(&mut self, action: &Action) {
        match action {
            Action::MetricBatchLoaded { run_id, metric } => {
                let (key, points) = metric;
                let new_series = make_metric_series(key.clone(), points.clone());

                let state = self
                    .metrics_cache
                    .remove(run_id)
                    .unwrap_or(MetricsState::Loading);

                let mut series = match state {
                    MetricsState::LoadingPartial(existing) => existing,
                    MetricsState::Loading => Vec::new(),
                    other => {
                        // Unexpected state — keep new data, don't lose it
                        self.metrics_cache.insert(run_id.clone(), other);
                        return;
                    }
                };

                // Insert in sorted order by key
                let pos = series
                    .binary_search_by(|s| s.key.cmp(&new_series.key))
                    .unwrap_or_else(|p| p);
                series.insert(pos, new_series);

                self.metrics_cache
                    .insert(run_id.clone(), MetricsState::LoadingPartial(series));
            }
            Action::MetricsFetchComplete { run_id } => {
                if let Some(MetricsState::LoadingPartial(series)) =
                    self.metrics_cache.remove(run_id)
                {
                    self.metrics_cache
                        .insert(run_id.clone(), MetricsState::Loaded(series));
                }
                // If still Loading (0 metrics returned), mark as Loaded with empty vec
                if let Some(MetricsState::Loading) = self.metrics_cache.get(run_id) {
                    self.metrics_cache
                        .insert(run_id.clone(), MetricsState::Loaded(Vec::new()));
                }
            }
            Action::MetricsFetchFailed { run_id, error } => {
                self.metrics_cache
                    .insert(run_id.clone(), MetricsState::Error(error.clone()));
            }
            _ => {}
        }
    }

    /// Handle a key event. Returns a result indicating what the parent should do.
    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        run_id: &str,
        metric_keys: &[String],
    ) -> DetailKeyResult {
        match key.code {
            KeyCode::Left => {
                if self.active_tab != DetailTab::Info {
                    self.active_tab = DetailTab::Info;
                }
                DetailKeyResult::Consumed
            }
            KeyCode::Right => {
                if self.active_tab != DetailTab::Metrics {
                    self.active_tab = DetailTab::Metrics;
                    return self.maybe_fetch_metrics(run_id, metric_keys);
                }
                DetailKeyResult::Consumed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_up();
                DetailKeyResult::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_down();
                DetailKeyResult::Consumed
            }
            KeyCode::Home => {
                self.scroll_to_top();
                DetailKeyResult::Consumed
            }
            KeyCode::End => {
                self.scroll_to_end();
                DetailKeyResult::Consumed
            }
            KeyCode::Esc => DetailKeyResult::Close,
            _ => DetailKeyResult::Ignored,
        }
    }

    fn scroll_up(&mut self) {
        match self.active_tab {
            DetailTab::Info => {
                self.info_scroll = self.info_scroll.saturating_sub(1);
            }
            DetailTab::Metrics => {
                self.metrics_scroll = self.metrics_scroll.saturating_sub(1);
            }
        }
    }

    fn scroll_down(&mut self) {
        match self.active_tab {
            DetailTab::Info => {
                self.info_scroll = self.info_scroll.saturating_add(1);
            }
            DetailTab::Metrics => {
                self.metrics_scroll = self.metrics_scroll.saturating_add(1);
            }
        }
    }

    fn scroll_to_top(&mut self) {
        match self.active_tab {
            DetailTab::Info => self.info_scroll = 0,
            DetailTab::Metrics => self.metrics_scroll = 0,
        }
    }

    fn scroll_to_end(&mut self) {
        match self.active_tab {
            DetailTab::Info => {
                self.info_scroll = self
                    .info_total_lines
                    .saturating_sub(self.info_visible_height);
            }
            DetailTab::Metrics => {
                self.metrics_scroll = self
                    .metrics_total_height
                    .saturating_sub(self.metrics_visible_height);
            }
        }
    }

    /// Check if metrics need fetching for the current run.
    fn maybe_fetch_metrics(&mut self, run_id: &str, metric_keys: &[String]) -> DetailKeyResult {
        if metric_keys.is_empty() {
            return DetailKeyResult::Consumed;
        }

        match self.metrics_cache.get(run_id) {
            Some(MetricsState::Loaded(_))
            | Some(MetricsState::Loading)
            | Some(MetricsState::LoadingPartial(_)) => DetailKeyResult::Consumed,
            _ => {
                self.metrics_cache
                    .insert(run_id.to_string(), MetricsState::Loading);
                DetailKeyResult::FetchMetrics {
                    run_id: run_id.to_string(),
                    metric_keys: metric_keys.to_vec(),
                }
            }
        }
    }

    /// Render the detail pane into the given area.
    pub fn render(&mut self, frame: &mut Frame, area: Rect, job: &JobDetail, run_id: &str) {
        let block = Block::default()
            .title(format!(" {} ", job.display_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER_ACTIVE));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width < 4 || inner.height < 3 {
            return;
        }

        // Split: 1 row for sub-tab bar, rest for content
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner);

        self.render_tab_bar(frame, chunks[0]);

        self.current_run_id = Some(run_id.to_string());

        match self.active_tab {
            DetailTab::Info => {
                self.render_info_tab(frame, chunks[1], job);
            }
            DetailTab::Metrics => {
                self.render_metrics_tab(frame, chunks[1], run_id);
            }
        }
    }

    fn render_tab_bar(&self, frame: &mut Frame, area: Rect) {
        let titles: Vec<Line> = DetailTab::ALL
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let style = if i == self.active_tab.index() {
                    Style::default()
                        .fg(Theme::TAB_ACTIVE_FG)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::TAB_INACTIVE_FG)
                };
                Line::from(Span::styled(format!(" {} ", tab.label()), style))
            })
            .collect();

        let tabs = Tabs::new(titles)
            .select(self.active_tab.index())
            .highlight_style(
                Style::default()
                    .fg(Theme::TAB_ACTIVE_FG)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(Span::styled("│", Style::default().fg(Theme::BORDER)));

        frame.render_widget(tabs, area);
    }

    fn render_info_tab(&mut self, frame: &mut Frame, area: Rect, job: &JobDetail) {
        if area.width < 4 || area.height < 2 {
            return;
        }

        let content_width = area.width.saturating_sub(2);
        let mut lines: Vec<Line> = Vec::new();

        // Reuse the same field-building logic from job_detail
        let created = job.created_at.unwrap_or("—");
        let general_fields: Vec<(&str, &str)> = vec![
            ("Job ID", job.id),
            ("Display Name", job.display_name),
            ("Experiment", job.experiment_name),
            ("Type", job.job_type),
            ("Compute", job.compute_target),
            ("Created", created),
        ];
        let label_w = job_detail::max_label_width(&general_fields).max("Status".len());

        job_detail::add_section_header(&mut lines, "General", content_width);
        for (label, value) in &general_fields {
            job_detail::add_aligned_field(&mut lines, label, value, label_w);
        }
        add_status_field(&mut lines, "Status", job.status, label_w);

        // Execution
        if job.command.is_some() || job.environment_id.is_some() {
            lines.push(Line::from(""));
            job_detail::add_section_header(&mut lines, "Execution", content_width);

            let mut exec_labels: Vec<&str> = Vec::new();
            if job.command.is_some() {
                exec_labels.push("Command");
            }
            if job.environment_id.is_some() {
                exec_labels.push("Environment");
            }
            let exec_w = exec_labels.iter().map(|l| l.len()).max().unwrap_or(0);

            if let Some(cmd) = job.command {
                job_detail::add_aligned_field(&mut lines, "Command", cmd, exec_w);
            }
            if let Some(env) = job.environment_id {
                let short_env = extract_environment_name(env);
                job_detail::add_aligned_field(&mut lines, "Environment", &short_env, exec_w);
            }
        }

        // Description
        if let Some(desc) = job.description {
            if !desc.is_empty() {
                lines.push(Line::from(""));
                job_detail::add_section_header(&mut lines, "Description", content_width);
                lines.push(Line::from(Span::styled(
                    format!(" {}", desc),
                    Style::default().fg(Theme::FG),
                )));
            }
        }

        // Tags
        if let Some(tags) = job.tags {
            if !tags.is_empty() {
                lines.push(Line::from(""));
                job_detail::add_section_header(&mut lines, "Tags", content_width);
                let mut sorted_tags: Vec<(&String, &String)> = tags.iter().collect();
                sorted_tags.sort_by_key(|(k, _)| k.as_str());
                let tag_w = sorted_tags.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
                for (k, v) in &sorted_tags {
                    job_detail::add_aligned_field(&mut lines, k, v, tag_w);
                }
            }
        }

        let total_lines = lines.len() as u16;
        self.info_total_lines = total_lines;
        self.info_visible_height = area.height;

        // Clamp scroll
        let max_scroll = total_lines.saturating_sub(area.height);
        self.info_scroll = self.info_scroll.min(max_scroll);

        // Content area with left padding
        let content_area = Rect {
            x: area.x + 1,
            y: area.y,
            width: content_width,
            height: area.height,
        };

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.info_scroll, 0));
        frame.render_widget(paragraph, content_area);

        // Scrollbar
        if total_lines > area.height {
            let max_scroll = total_lines.saturating_sub(area.height) as usize;
            let mut scrollbar_state =
                ScrollbarState::new(max_scroll).position(self.info_scroll as usize);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    fn render_metrics_tab(&mut self, frame: &mut Frame, area: Rect, run_id: &str) {
        if area.width < 4 || area.height < 2 {
            return;
        }

        let state = self
            .metrics_cache
            .get(run_id)
            .cloned()
            .unwrap_or(MetricsState::NotFetched);

        match state {
            MetricsState::NotFetched => {
                let msg = Paragraph::new(" No metrics recorded for this run")
                    .style(Style::default().fg(Theme::DIM));
                frame.render_widget(msg, area);
                self.metrics_total_height = 0;
                self.metrics_visible_height = area.height;
            }
            MetricsState::Loading => {
                let msg =
                    Paragraph::new(" Loading metrics…").style(Style::default().fg(Theme::DIM));
                frame.render_widget(msg, area);
                self.metrics_total_height = 0;
                self.metrics_visible_height = area.height;
            }
            MetricsState::Error(ref err) => {
                let msg = Paragraph::new(format!(" Error: {}", err))
                    .style(Style::default().fg(Theme::ERROR))
                    .wrap(Wrap { trim: false });
                frame.render_widget(msg, area);
                self.metrics_total_height = 0;
                self.metrics_visible_height = area.height;
            }
            MetricsState::LoadingPartial(ref series) => {
                if series.is_empty() {
                    let msg =
                        Paragraph::new(" Loading metrics…").style(Style::default().fg(Theme::DIM));
                    frame.render_widget(msg, area);
                    self.metrics_total_height = 0;
                    self.metrics_visible_height = area.height;
                    return;
                }

                self.render_metric_charts(frame, area, series, true);
            }
            MetricsState::Loaded(ref series) => {
                if series.is_empty() {
                    let msg = Paragraph::new(" No metrics recorded for this run")
                        .style(Style::default().fg(Theme::DIM));
                    frame.render_widget(msg, area);
                    self.metrics_total_height = 0;
                    self.metrics_visible_height = area.height;
                    return;
                }

                self.render_metric_charts(frame, area, series, false);
            }
        }
    }

    fn render_metric_charts(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        series: &[MetricSeries],
        still_loading: bool,
    ) {
        // Each chart gets CHART_HEIGHT rows, with 1 row gap between them
        let loading_indicator_height: u16 = if still_loading { 2 } else { 0 };
        let total_height: u16 = (series.len() as u16) * CHART_HEIGHT
            + (series.len().saturating_sub(1) as u16) // gaps
            + loading_indicator_height;
        self.metrics_total_height = total_height;
        self.metrics_visible_height = area.height;

        // Clamp scroll
        let max_scroll = total_height.saturating_sub(area.height);
        self.metrics_scroll = self.metrics_scroll.min(max_scroll);

        let scroll = self.metrics_scroll;

        // Render each chart, accounting for scroll offset
        let mut y_offset: u16 = 0;
        for (i, metric) in series.iter().enumerate() {
            let chart_y_start = y_offset;
            let chart_y_end = y_offset + CHART_HEIGHT;

            // Check if this chart is visible after scrolling
            if chart_y_end > scroll && chart_y_start < scroll + area.height {
                let visible_start = chart_y_start.saturating_sub(scroll);
                let visible_y = area.y + visible_start;
                let available_height = (area.y + area.height).saturating_sub(visible_y);
                let chart_visible_height = CHART_HEIGHT
                    .min(available_height)
                    .min(chart_y_end.saturating_sub(scroll));

                if chart_visible_height >= 3 {
                    let chart_area = Rect {
                        x: area.x,
                        y: visible_y,
                        width: area.width,
                        height: chart_visible_height,
                    };

                    let color = CHART_COLORS[i % CHART_COLORS.len()];
                    self.render_single_chart(frame, chart_area, metric, color);
                }
            }

            y_offset = chart_y_end + 1; // +1 for gap
        }

        // Loading indicator for partial state
        if still_loading {
            let indicator_y_start = y_offset;
            let indicator_y_end = y_offset + loading_indicator_height;
            if indicator_y_end > scroll && indicator_y_start < scroll + area.height {
                let visible_start = indicator_y_start.saturating_sub(scroll);
                let visible_y = area.y + visible_start;
                let available_height = (area.y + area.height).saturating_sub(visible_y);
                if available_height > 0 {
                    let indicator_area = Rect {
                        x: area.x,
                        y: visible_y,
                        width: area.width,
                        height: loading_indicator_height.min(available_height),
                    };
                    let msg =
                        Paragraph::new(format!(" Loading more metrics… ({} loaded)", series.len()))
                            .style(Style::default().fg(Theme::DIM));
                    frame.render_widget(msg, indicator_area);
                }
            }
        }

        // Scrollbar
        if total_height > area.height {
            let max_scroll_usize = total_height.saturating_sub(area.height) as usize;
            let mut scrollbar_state =
                ScrollbarState::new(max_scroll_usize).position(self.metrics_scroll as usize);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    fn render_single_chart(
        &self,
        frame: &mut Frame,
        area: Rect,
        metric: &MetricSeries,
        color: Color,
    ) {
        if metric.points.is_empty() || area.height < 3 {
            return;
        }

        // Compute axis bounds with a small margin
        let x_min = metric.min_x;
        let x_max = if metric.max_x == metric.min_x {
            metric.min_x + 1.0
        } else {
            metric.max_x
        };

        let y_range = metric.max_y - metric.min_y;
        let y_margin = if y_range == 0.0 { 0.5 } else { y_range * 0.05 };
        let y_min = metric.min_y - y_margin;
        let y_max = metric.max_y + y_margin;

        let dataset = Dataset::default()
            .name(metric.key.as_str())
            .marker(Marker::Braille)
            .style(Style::default().fg(color))
            .data(&metric.points);

        let x_label_min = format_number(x_min);
        let x_label_max = format_number(x_max);
        let y_label_min = format_number(y_min);
        let y_label_max = format_number(y_max);

        let chart = Chart::new(vec![dataset])
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" {} ", metric.key),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Theme::BORDER)),
            )
            .x_axis(
                Axis::default()
                    .title("step")
                    .style(Style::default().fg(Theme::DIM))
                    .bounds([x_min, x_max])
                    .labels(vec![Span::raw(x_label_min), Span::raw(x_label_max)]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Theme::DIM))
                    .bounds([y_min, y_max])
                    .labels(vec![Span::raw(y_label_min), Span::raw(y_label_max)]),
            );

        frame.render_widget(chart, area);
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Build a `MetricSeries` from a key and points.
fn make_metric_series(key: String, points: Vec<(f64, f64)>) -> MetricSeries {
    let min_x = points.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = points.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
    let max_y = points
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max);
    MetricSeries {
        key,
        points,
        min_x,
        max_x,
        min_y,
        max_y,
    }
}

// ── Metrics fetcher ────────────────────────────────────────────────────

/// Spawn a background task to fetch metric histories for a run.
///
/// Metrics are sent to the UI incrementally — each metric key dispatches
/// a `MetricBatchLoaded` action as soon as its history is ready, so
/// charts appear progressively rather than waiting for all metrics.
pub fn spawn_metrics_fetcher(
    client: AzureClient,
    run_id: String,
    metric_keys: Vec<String>,
    action_tx: ActionSender,
) {
    tokio::spawn(async move {
        let mlflow = client.mlflow();

        for key in &metric_keys {
            match mlflow.get_all_metric_history(&run_id, key).await {
                Ok(history) => {
                    // Skip data points with no value, NaN, or infinite values
                    let valid: Vec<_> = history
                        .iter()
                        .filter(|m| matches!(m.value, Some(v) if v.is_finite()))
                        .collect();
                    // Use step as x-axis; if all steps are 0 (absent), fall back to index
                    let all_zero_step = valid.iter().all(|m| m.step == 0);
                    let mut points: Vec<(f64, f64)> = if all_zero_step {
                        valid
                            .iter()
                            .enumerate()
                            .map(|(i, m)| (i as f64, m.value.unwrap()))
                            .collect()
                    } else {
                        valid
                            .iter()
                            .map(|m| (m.step as f64, m.value.unwrap()))
                            .collect()
                    };
                    points
                        .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                    if !points.is_empty() {
                        let _ = action_tx.send(Action::MetricBatchLoaded {
                            run_id: run_id.clone(),
                            metric: (key.clone(), points),
                        });
                    }
                }
                Err(e) => {
                    let _ = action_tx.send(Action::MetricsFetchFailed {
                        run_id: run_id.clone(),
                        error: format!("Failed to fetch metric '{}': {}", key, e),
                    });
                    return;
                }
            }
        }

        let _ = action_tx.send(Action::MetricsFetchComplete { run_id });
    });
}

// ── Helpers ────────────────────────────────────────────────────────────

fn add_status_field(lines: &mut Vec<Line<'_>>, label: &str, status: &str, max_label_width: usize) {
    let padded = format!("{:>width$}", label, width = max_label_width);
    let symbol = status_symbol(status);
    let color = status_color(status);
    lines.push(Line::from(vec![
        Span::styled(format!(" {} ", padded), Style::default().fg(Theme::DIM)),
        Span::styled(format!("{} {}", symbol, status), Style::default().fg(color)),
    ]));
}

fn status_symbol(status: &str) -> &'static str {
    match status {
        "FINISHED" => "✓",
        "FAILED" => "✗",
        "RUNNING" => "●",
        "KILLED" => "✕",
        "SCHEDULED" | "STARTING" => "◯",
        _ => "?",
    }
}

fn status_color(status: &str) -> Color {
    match status {
        "FINISHED" => Theme::SUCCESS,
        "FAILED" => Theme::ERROR,
        "RUNNING" => Theme::RUNNING,
        "KILLED" => Theme::DIM,
        "SCHEDULED" | "STARTING" => Theme::WARNING,
        _ => Theme::DIM,
    }
}

fn extract_environment_name(env_id: &str) -> String {
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

/// Format a number for axis labels — compact representation.
fn format_number(n: f64) -> String {
    if n.abs() >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n.abs() >= 1_000.0 {
        format!("{:.1}k", n / 1_000.0)
    } else if n == n.floor() && n.abs() < 100_000.0 {
        format!("{}", n as i64)
    } else {
        format!("{:.4}", n)
    }
}
