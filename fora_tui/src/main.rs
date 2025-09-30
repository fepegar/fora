use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, Terminal};
use std::io;
use tokio::sync::mpsc;

mod app;
mod azure;
mod cache;
mod config;
mod events;
mod keys;
mod navigation;
mod tab_registry;
mod tabs;
mod ui;

use app::{App, AppEvent};
use events::EventHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and event channels
    let (tx, rx) = mpsc::unbounded_channel::<AppEvent>();
    let mut app = App::new(tx.clone())
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let mut event_handler = EventHandler::new(tx);

    // Run the app
    let result = run_app(&mut terminal, &mut app, &mut event_handler, rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(result?)
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: &mut EventHandler,
    mut rx: mpsc::UnboundedReceiver<AppEvent>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        tokio::select! {
            _ = event_handler.handle_events() => {},
            Some(event) = rx.recv() => {
                match event {
                    AppEvent::Quit => break,
                    _ => app.handle_event(event).await,
                }
            }
        }
    }
    Ok(())
}
