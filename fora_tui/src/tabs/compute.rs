use super::Tab;
use crate::app::AppEvent;
use crate::keys::help_text;
use crate::navigation::{NavigationContext, TabNavigator};
use crate::select_keys;
use crate::{azure::AzureClient, cache::CacheManager};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::any::Any;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum ComputeEvent {
    ComputeSelected(String),
    ComputeStarted(String),
    ComputeStopped(String),
    ComputeDeleted(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTarget {
    pub id: String,
    pub name: String,
    pub compute_type: ComputeType,
    pub status: ComputeStatus,
    pub node_count: Option<u32>,
    pub max_node_count: Option<u32>,
    pub min_node_count: Option<u32>,
    pub vm_size: Option<String>,
    pub created_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeType {
    AmlCompute,
    ComputeInstance,
    AksCompute,
    DataFactory,
    HDInsight,
}

impl std::fmt::Display for ComputeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputeType::AmlCompute => write!(f, "AML Compute"),
            ComputeType::ComputeInstance => write!(f, "Compute Instance"),
            ComputeType::AksCompute => write!(f, "AKS Compute"),
            ComputeType::DataFactory => write!(f, "Data Factory"),
            ComputeType::HDInsight => write!(f, "HDInsight"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeStatus {
    Creating,
    Running,
    Deleting,
    Deleted,
    Failed,
    Stopped,
}

impl std::fmt::Display for ComputeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputeStatus::Creating => write!(f, "Creating"),
            ComputeStatus::Running => write!(f, "Running"),
            ComputeStatus::Deleting => write!(f, "Deleting"),
            ComputeStatus::Deleted => write!(f, "Deleted"),
            ComputeStatus::Failed => write!(f, "Failed"),
            ComputeStatus::Stopped => write!(f, "Stopped"),
        }
    }
}

pub struct ComputeTab {
    // State
    compute_targets: Vec<ComputeTarget>,
    selected_compute: Option<String>,
    loading: bool,

    // UI state
    list_state: ListState,

    // Dependencies
    azure_client: AzureClient,
    cache: CacheManager,
    event_tx: UnboundedSender<AppEvent>,
    navigator: Option<TabNavigator>,
}

impl ComputeTab {
    pub fn new(
        azure_client: AzureClient,
        cache: CacheManager,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Self {
        Self {
            compute_targets: Vec::new(),
            selected_compute: None,
            loading: false,
            list_state: ListState::default(),
            azure_client,
            cache,
            event_tx,
            navigator: None,
        }
    }

    pub async fn handle_event(&mut self, event: ComputeEvent) {
        match event {
            ComputeEvent::ComputeSelected(compute_id) => {
                self.selected_compute = Some(compute_id);
            }
            ComputeEvent::ComputeStarted(compute_id) => {
                self.start_compute(compute_id).await;
            }
            ComputeEvent::ComputeStopped(compute_id) => {
                self.stop_compute(compute_id).await;
            }
            ComputeEvent::ComputeDeleted(compute_id) => {
                self.delete_compute(compute_id).await;
            }
        }
    }

    async fn load_compute_targets(&mut self) {
        self.loading = true;

        // Mock data for now - replace with actual Azure ML API calls
        self.compute_targets = vec![
            ComputeTarget {
                id: "compute-1".to_string(),
                name: "cpu-cluster".to_string(),
                compute_type: ComputeType::AmlCompute,
                status: ComputeStatus::Running,
                node_count: Some(2),
                max_node_count: Some(10),
                min_node_count: Some(0),
                vm_size: Some("Standard_DS3_v2".to_string()),
                created_time: Utc::now(),
            },
            ComputeTarget {
                id: "compute-2".to_string(),
                name: "gpu-instance".to_string(),
                compute_type: ComputeType::ComputeInstance,
                status: ComputeStatus::Stopped,
                node_count: Some(1),
                max_node_count: Some(1),
                min_node_count: Some(1),
                vm_size: Some("Standard_NC6".to_string()),
                created_time: Utc::now(),
            },
        ];

        self.loading = false;
    }

    async fn start_compute(&mut self, compute_id: String) {
        let _client = self.azure_client.clone();

        tokio::spawn(async move {
            // Pseudocode - implement actual Azure ML API calls
            // client.start_compute(&compute_id).await
            tracing::info!("Starting compute: {}", compute_id);
        });
    }

    async fn stop_compute(&mut self, compute_id: String) {
        let _client = self.azure_client.clone();

        tokio::spawn(async move {
            // Pseudocode - implement actual Azure ML API calls
            // client.stop_compute(&compute_id).await
            tracing::info!("Stopping compute: {}", compute_id);
        });
    }

    async fn delete_compute(&mut self, compute_id: String) {
        let _client = self.azure_client.clone();

        tokio::spawn(async move {
            // Pseudocode - implement actual Azure ML API calls
            // client.delete_compute(&compute_id).await
            tracing::info!("Deleting compute: {}", compute_id);
        });
    }
}

#[async_trait]
impl Tab for ComputeTab {
    async fn initialize(&mut self) {
        self.load_compute_targets().await;
    }

    async fn refresh(&mut self) {
        self.load_compute_targets().await;
    }

    async fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if !self.compute_targets.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.compute_targets.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Down => {
                if !self.compute_targets.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => (i + 1) % self.compute_targets.len(),
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                }
            }
            select_keys!() => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(compute) = self.compute_targets.get(selected) {
                        let _ = self.event_tx.send(AppEvent::ComputeEvent(
                            ComputeEvent::ComputeSelected(compute.id.clone()),
                        ));
                    }
                }
            }
            KeyCode::Char('s') => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(compute) = self.compute_targets.get(selected) {
                        match compute.status {
                            ComputeStatus::Stopped => {
                                let _ = self.event_tx.send(AppEvent::ComputeEvent(
                                    ComputeEvent::ComputeStarted(compute.id.clone()),
                                ));
                            }
                            ComputeStatus::Running => {
                                let _ = self.event_tx.send(AppEvent::ComputeEvent(
                                    ComputeEvent::ComputeStopped(compute.id.clone()),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
            KeyCode::Char('d')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(compute) = self.compute_targets.get(selected) {
                        let _ = self.event_tx.send(AppEvent::ComputeEvent(
                            ComputeEvent::ComputeDeleted(compute.id.clone()),
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .compute_targets
            .iter()
            .map(|compute| {
                let status_color = match compute.status {
                    ComputeStatus::Running => Color::Green,
                    ComputeStatus::Creating => Color::Yellow,
                    ComputeStatus::Stopped => Color::Gray,
                    ComputeStatus::Failed => Color::Red,
                    ComputeStatus::Deleting => Color::Magenta,
                    ComputeStatus::Deleted => Color::DarkGray,
                };

                let node_info = match (
                    compute.node_count,
                    compute.min_node_count,
                    compute.max_node_count,
                ) {
                    (Some(current), Some(min), Some(max)) => {
                        format!("Nodes: {}/{}-{}", current, min, max)
                    }
                    (Some(current), _, _) => format!("Nodes: {}", current),
                    _ => "Nodes: N/A".to_string(),
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            &compute.name,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            format!("[{}]", compute.status),
                            Style::default().fg(status_color),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            format!("{} | ", compute.compute_type),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(node_info, Style::default().fg(Color::Gray)),
                    ]),
                    Line::from(Span::styled(
                        compute.vm_size.as_deref().unwrap_or("No VM size"),
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    )),
                ])
            })
            .collect();

        // Use consistent border set with connecting corners
        let border_set = symbols::border::Set {
            top_left: symbols::line::VERTICAL_RIGHT,
            top_right: symbols::line::VERTICAL_LEFT,
            ..symbols::border::ROUNDED
        };

        let compute_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(border_set)
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(compute_list, area, &mut self.list_state.clone());

        // Render help text at the bottom
        let help_area = Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(3),
            width: area.width,
            height: 3,
        };

        let help_border_set = symbols::border::Set {
            top_left: symbols::line::VERTICAL_RIGHT,
            top_right: symbols::line::VERTICAL_LEFT,
            ..symbols::border::ROUNDED
        };

        let help_text = Paragraph::new(format!(
            "s: Start/Stop | Ctrl+d: Delete | {}: Select",
            help_text::SELECT_KEYS
        ))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_set(help_border_set)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

        f.render_widget(help_text, help_area);
    }

    fn title(&self) -> &str {
        "Compute"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    async fn on_navigation(&mut self, context: &NavigationContext) {
        // Handle navigation context if needed
        // For compute tab, we might want to refresh data when navigating to it
        if context.previous_tab.is_some() {
            self.load_compute_targets().await;
        }
    }

    fn set_navigator(&mut self, navigator: TabNavigator) {
        self.navigator = Some(navigator);
    }
}
