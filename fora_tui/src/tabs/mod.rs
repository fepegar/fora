use crate::navigation::{NavigationContext, TabNavigator};
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use std::any::Any;

pub mod compute;
pub mod experiments;
pub mod home;
pub mod jobs;

#[async_trait]
pub trait Tab: Send + Sync {
    async fn initialize(&mut self);
    async fn refresh(&mut self);
    async fn handle_key(&mut self, key: KeyEvent);
    fn render(&mut self, f: &mut Frame, area: Rect);
    fn title(&self) -> &str;

    // Navigation support
    async fn on_navigation(&mut self, context: &NavigationContext);
    fn set_navigator(&mut self, navigator: TabNavigator);

    // For downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
