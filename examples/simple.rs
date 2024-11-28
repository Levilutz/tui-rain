use ratatui::Frame;
use tokio::time;
use tui_rain::{CharacterSet, Rain};

#[tokio::main]
async fn main() {
    let mut terminal = ratatui::init();
    terminal.clear().unwrap();

    let mut tick_interval = time::interval(time::Duration::from_secs_f64(1.0 / 60.0));

    let start_time = time::Instant::now();

    for _ in 0..600 {
        tick_interval.tick().await;
        terminal
            .draw(|frame| render(frame, start_time.elapsed()))
            .unwrap();
    }

    ratatui::restore();
}

fn render(frame: &mut Frame, elapsed: time::Duration) {
    let rain = Rain::new_matrix(elapsed);
    frame.render_widget(rain, frame.area());
}
