use std::collections::HashMap;

use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, Row, Table, TableState};
use ratatui::Frame;

use crate::theme::{self};

/// A column definition for the dynamic table widget.
pub struct ColumnDef<T> {
    pub id: &'static str,
    pub label: &'static str,
    pub extract: Box<dyn Fn(&T) -> String>,
    pub style: Option<fn(&T) -> Style>,
    pub min_width: u16,
    pub default_width: u16,
    pub visible: bool,
    pub order: usize,
}

impl<T> ColumnDef<T> {
    pub fn new(
        id: &'static str,
        label: &'static str,
        extract: impl Fn(&T) -> String + 'static,
        default_width: u16,
    ) -> Self {
        Self {
            id,
            label,
            extract: Box::new(extract),
            style: None,
            min_width: 4,
            default_width,
            visible: true,
            order: 0,
        }
    }

    pub fn with_style(mut self, style_fn: fn(&T) -> Style) -> Self {
        self.style = Some(style_fn);
        self
    }

    pub fn with_min_width(mut self, min: u16) -> Self {
        self.min_width = min;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }
}

/// Renders a table with dynamic columns, handling column cropping when space is limited.
pub fn render_table<T>(
    frame: &mut Frame,
    area: Rect,
    columns: &[ColumnDef<T>],
    items: &[T],
    table_state: &mut TableState,
    block: Block<'_>,
) {
    let visible_columns = get_visible_columns(columns, area.width.saturating_sub(2));

    if visible_columns.is_empty() {
        frame.render_widget(block, area);
        return;
    }

    let widths: Vec<Constraint> = visible_columns
        .iter()
        .map(|col| Constraint::Min(col.min_width.max(col.default_width)))
        .collect();

    let header_cells: Vec<Span> = visible_columns
        .iter()
        .map(|col| Span::styled(col.label, theme::header_style()))
        .collect();
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let cells: Vec<Span> = visible_columns
                .iter()
                .map(|col| {
                    let text = (col.extract)(item);
                    let style = col
                        .style
                        .map(|f| f(item))
                        .unwrap_or_else(|| theme::stripe_style(idx));
                    Span::styled(text, style)
                })
                .collect();
            Row::new(cells).style(theme::stripe_style(idx))
        })
        .collect();

    let table = Table::new(rows, &widths)
        .header(header)
        .block(block)
        .highlight_style(theme::selected_style())
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(table, area, table_state);
}

/// Determine which columns fit in the available width, cropping from the right.
fn get_visible_columns<'a, T>(
    columns: &'a [ColumnDef<T>],
    available_width: u16,
) -> Vec<&'a ColumnDef<T>> {
    let mut sorted: Vec<&ColumnDef<T>> = columns.iter().filter(|c| c.visible).collect();
    sorted.sort_by_key(|c| c.order);

    let mut result = Vec::new();
    let mut used_width: u16 = 0;
    // Account for highlight symbol width and spacing between columns
    let highlight_width: u16 = 2;
    let remaining = available_width.saturating_sub(highlight_width);

    for col in sorted {
        let col_width = col.default_width + 1; // +1 for spacing
        if used_width + col_width <= remaining {
            result.push(col);
            used_width += col_width;
        } else {
            break;
        }
    }

    result
}

/// State for the table list (selection, scroll offset).
#[derive(Debug, Default)]
pub struct ListState {
    pub table_state: TableState,
    pub total_items: usize,
}

impl ListState {
    pub fn new() -> Self {
        let mut ts = TableState::default();
        ts.select(Some(0));
        Self {
            table_state: ts,
            total_items: 0,
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.table_state.selected()
    }

    pub fn select_next(&mut self) {
        if self.total_items == 0 {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map(|i| (i + 1).min(self.total_items - 1))
            .unwrap_or(0);
        self.table_state.select(Some(i));
    }

    pub fn select_prev(&mut self) {
        if self.total_items == 0 {
            return;
        }
        let i = self
            .table_state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.table_state.select(Some(i));
    }

    pub fn set_total(&mut self, total: usize) {
        self.total_items = total;
        if let Some(sel) = self.table_state.selected() {
            if sel >= total && total > 0 {
                self.table_state.select(Some(total - 1));
            } else if total == 0 {
                self.table_state.select(None);
            }
        } else if total > 0 {
            self.table_state.select(Some(0));
        }
    }
}

/// Apply saved column config (ordering + visibility) to column definitions.
/// Listed column IDs are visible in that order; unlisted columns are hidden and appended at the end.
pub fn apply_column_config<T>(columns: &mut Vec<ColumnDef<T>>, config: Option<&[String]>) {
    let Some(visible_ids) = config else {
        for (i, col) in columns.iter_mut().enumerate() {
            col.order = i;
        }
        return;
    };

    let order_map: HashMap<&str, usize> = visible_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    let visible_count = visible_ids.len();

    for col in columns.iter_mut() {
        if let Some(&order) = order_map.get(col.id) {
            col.order = order;
            col.visible = true;
        } else {
            // Not in config list — hidden, placed after visible columns
            col.order = visible_count + col.order;
            col.visible = false;
        }
    }

    columns.sort_by_key(|c| c.order);
}
