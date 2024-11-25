use std::error::Error;

use crossterm::event::{Event, EventStream, KeyCode};
use futures::{FutureExt, StreamExt};
use ratatui::{style::Stylize, DefaultTerminal, Frame};
use tokio::time;
use tui_rain::Rain;

/// How much to smooth the FPS tracking.
///
/// Values closer to 1 are smoother, values closer to 0 are more responsive.
const FPS_SMOOTHING: f64 = 0.95;

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
    let tick_duration = time::Duration::from_secs_f64(1.0 / framerate);
    let mut tick_interval = time::interval(tick_duration);

    // Initialize start time to pass down to Rain widget.
    let start_time = time::Instant::now();

    // Initialize stuff to track smoothed FPS.
    let mut show_fps = true;
    let mut last_tick = time::Instant::now().checked_sub(tick_duration).unwrap();
    let mut fps: f64 = framerate;

    loop {
        // Wait for next tick or term signal
        tokio::select! {
            _ = tick_interval.tick() => {
                // Update FPS tracking
                let elapsed = last_tick.elapsed();
                last_tick = time::Instant::now();
                fps = fps.min(1e4) * FPS_SMOOTHING + (1.0 - FPS_SMOOTHING) / elapsed.as_secs_f64();

                // Render
                terminal.draw(|frame| render(frame, start_time.elapsed(), fps, show_fps))?;
            },

            event = reader.next().fuse() => match event {
                Some(Ok(Event::Key(key_event))) if key_event.code == KeyCode::Char('q') => {
                    return Ok(())
                },
                Some(Ok(Event::Key(key_event))) if key_event.code == KeyCode::Char('f') => {
                    show_fps = !show_fps
                },
                _ => {},
            },
        }
    }
}

fn render(frame: &mut Frame, elapsed: time::Duration, fps: f64, show_fps: bool) {
    frame.render_widget(Rain::new(elapsed), frame.area());
    if show_fps {
        frame.render_widget(
            format!("(f) FPS: {}", fps.round()).white().on_blue(),
            frame.area(),
        );
    }
}
