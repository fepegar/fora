use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use crate::app::Action;
use crate::cache::Cache;
use crate::client::AzureClient;
use crate::components::column_picker::{ColumnEntry, ColumnPicker};
use crate::tabs::{ActionSender, Tab};
use crate::theme;
use crate::widgets::table::{self, ListState};

use super::columns::default_columns;
use super::detail::render_compute_detail;
use super::fetch::fetch_compute;
use super::state::{ComputeRow, ComputeState};

pub struct ComputeTab {
    all_compute: Vec<ComputeRow>,
    list_state: ListState,
    columns: Vec<table::ColumnDef<ComputeRow>>,
    state: ComputeState,
    column_picker: ColumnPicker,
    cache: Cache<Vec<ComputeRow>>,
    client: Option<AzureClient>,
}

impl ComputeTab {
    pub fn new(
        client: Option<AzureClient>,
        refresh_interval: u64,
        column_config: Option<&[String]>,
    ) -> Self {
        let mut columns = default_columns();
        table::apply_column_config(&mut columns, column_config);

        Self {
            all_compute: Vec::new(),
            list_state: ListState::new(),
            columns,
            state: ComputeState::default(),
            column_picker: ColumnPicker::new(),
            cache: Cache::new(Duration::from_secs(refresh_interval)),
            client,
        }
    }

    pub fn set_client(&mut self, client: AzureClient) {
        self.client = Some(client);
        self.all_compute.clear();
        self.list_state.set_total(0);
    }

    fn selected_compute(&self) -> Option<&ComputeRow> {
        self.list_state
            .selected()
            .and_then(|i| self.all_compute.get(i))
    }
}

impl Tab for ComputeTab {
    fn title(&self) -> &str {
        "Compute"
    }

    fn handle_key(&mut self, key: KeyEvent, action_tx: &ActionSender) -> bool {
        if self.column_picker.active {
            let was_active = self.column_picker.active;
            self.column_picker.handle_key(key);

            // Apply changes live
            if self.column_picker.changed {
                let updated = self.column_picker.get_columns();
                for (i, (id, visible)) in updated.iter().enumerate() {
                    if let Some(col) = self.columns.iter_mut().find(|c| c.id == *id) {
                        col.visible = *visible;
                        col.order = i;
                    }
                }
                self.columns.sort_by_key(|c| c.order);
            }

            // Persist only when user explicitly saves
            if was_active && !self.column_picker.active && self.column_picker.save_requested {
                let visible_ids: Vec<String> = self
                    .columns
                    .iter()
                    .filter(|c| c.visible)
                    .map(|c| c.id.to_string())
                    .collect();
                let _ = action_tx.send(Action::SaveColumnConfig {
                    tab: "compute".to_string(),
                    columns: visible_ids,
                });
            }
            return true;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.select_prev();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.select_next();
                true
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.state.detail_open = !self.state.detail_open;
                true
            }
            KeyCode::Esc => {
                if self.state.detail_open {
                    self.state.detail_open = false;
                    true
                } else {
                    false
                }
            }
            KeyCode::Char('c') => {
                let entries: Vec<ColumnEntry> = self
                    .columns
                    .iter()
                    .map(|c| ColumnEntry {
                        id: c.id,
                        label: c.label,
                        visible: c.visible,
                    })
                    .collect();
                self.column_picker.open(entries);
                true
            }
            KeyCode::Char('r') => {
                let cache = self.cache.clone();
                let tx = action_tx.clone();
                tokio::spawn(async move {
                    cache.invalidate().await;
                    let _ = tx.send(Action::RefreshRequested);
                });
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, action: &Action) {
        if let Action::ComputeLoaded(compute) = action {
            self.all_compute = compute.clone();
            self.all_compute
                .sort_by(|a, b| b.running_nodes.unwrap_or(0).cmp(&a.running_nodes.unwrap_or(0)));
            self.list_state.set_total(self.all_compute.len());
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.state.detail_open {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

            self.render_list(frame, chunks[0]);

            if let Some(compute) = self.selected_compute() {
                render_compute_detail(frame, chunks[1], compute);
            }
        } else {
            self.render_list(frame, area);
        }

        // Overlays
        self.column_picker.render(frame, area);
    }

    fn tick(&mut self, action_tx: &ActionSender) {
        if self.client.is_some() {
            let client = self.client.clone();
            let cache = self.cache.clone();
            let tx = action_tx.clone();

            tokio::spawn(async move {
                if cache.is_stale().await && !cache.is_loading().await {
                    if let Some(client) = client {
                        if cache.start_loading().await {
                            match fetch_compute(&client).await {
                                Ok(compute) => {
                                    cache.set(compute.clone()).await;
                                    let _ = tx.send(Action::ComputeLoaded(compute));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::Error(format!(
                                        "Failed to fetch compute: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        if self.column_picker.active {
            return self.column_picker.key_hints();
        }

        let mut hints = vec![
            ("↑↓", "Navigate"),
            ("Enter", "Details"),
            ("c", "Columns"),
            ("r", "Refresh"),
        ];
        if self.state.detail_open {
            hints.push(("Esc", "Close Detail"));
        }
        hints
    }
}

impl ComputeTab {
    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let loading = if self.all_compute.is_empty() {
            " (loading…)"
        } else {
            ""
        };

        let block = Block::default()
            .title(format!(" Compute [{}]{} ", self.all_compute.len(), loading))
            .borders(Borders::ALL)
            .border_style(theme::border_style(true));

        let mut table_state = self.list_state.table_state.clone();
        table::render_table(
            frame,
            area,
            &self.columns,
            &self.all_compute,
            &mut table_state,
            block,
        );
    }
}
