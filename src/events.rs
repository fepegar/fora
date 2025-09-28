//! Event handling module for AZML TUI application

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedSender;

use crate::app::AppEvent;

pub struct EventHandler {
    sender: UnboundedSender<AppEvent>,
    last_tick: Instant,
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(sender: UnboundedSender<AppEvent>) -> Self {
        Self {
            sender,
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(50),
        }
    }

    pub async fn handle_events(&mut self) {
        let timeout = self
            .tick_rate
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                    self.handle_key_event(key);
                }
                Ok(Event::Mouse(_)) => {
                    // Handle mouse events if needed
                }
                Ok(Event::Resize(_, _)) => {
                    // Handle resize events if needed
                }
                _ => {}
            }
        }

        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
        }
    }

    fn handle_key_event(&self, key: KeyEvent) {
        // Send the raw KeyPress event to let the app handle tab shortcuts properly
        let _ = self.sender.send(AppEvent::KeyPress(key));
    }
}
