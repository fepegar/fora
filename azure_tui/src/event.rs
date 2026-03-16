use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use tokio::sync::mpsc;

/// Terminal events (crossterm events + internal tick).
#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

/// Spawns a background task that reads terminal events and sends tick events.
pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        tokio::spawn(async move {
            loop {
                let has_event =
                    tokio::task::spawn_blocking(move || event::poll(tick_rate).unwrap_or(false))
                        .await
                        .unwrap_or(false);

                if has_event {
                    if let Ok(ev) = event::read() {
                        let event = match ev {
                            CrosstermEvent::Key(key) => Some(Event::Key(key)),
                            CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
                            _ => None,
                        };
                        if let Some(e) = event {
                            if event_tx.send(e).is_err() {
                                return;
                            }
                        }
                    }
                } else {
                    // Tick
                    if event_tx.send(Event::Tick).is_err() {
                        return;
                    }
                }
            }
        });

        Self { rx, _tx: tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
