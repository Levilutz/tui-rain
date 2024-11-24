use std::error::Error;

use crossterm::event::{Event, EventStream, KeyCode};
use futures::{FutureExt, StreamExt};
use ratatui::{DefaultTerminal, Frame};
use tokio::time;
use tui_rain::Rain;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    terminal.clear().unwrap();
    let result = main_loop(terminal, 60.0).await;
    ratatui::restore();
    result
}

async fn main_loop(mut terminal: DefaultTerminal, framerate: f64) -> Result<(), Box<dyn Error>> {
    // Read terminal events
    let mut reader = EventStream::new();

    // Set up interval for the target framerate
    let mut tick_interval = time::interval(time::Duration::from_secs_f64(1.0 / framerate));

    // Initialize start time to pass down to Rain widget.
    let start_time = time::Instant::now();

    loop {
        // Render
        terminal.draw(|frame| render(frame, start_time.elapsed()))?;

        // Wait for quit signal or next tick
        tokio::select! {
            _ = tick_interval.tick() => {},
            event = reader.next().fuse() => match event {
                Some(Ok(Event::Key(key_event))) if key_event.code == KeyCode::Char('q') => {
                    return Ok(())
                },
                _ => {},
            },
        }
    }
}

fn render(frame: &mut Frame, elapsed: time::Duration) {
    frame.render_widget(Rain::new(elapsed), frame.area());
}
