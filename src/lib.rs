use std::time::Duration;

use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg64Mcg;
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, widgets::Widget};

pub struct Rain {
    elapsed: Duration,
    seed: u64,
}

impl Rain {
    /// Construct a new rain widget. Requires only current elapsed duration.
    pub fn new(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
        }
    }

    /// Set the random seed for the generation.
    pub fn with_seed(mut self, seed: u64) -> Rain {
        self.seed = seed;
        self
    }

    /// Build the rng. Uses a fast but portable and reproducible rng.
    fn build_rng(&self) -> impl RngCore {
        Pcg64Mcg::seed_from_u64(self.seed)
    }
}

impl Widget for Rain {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut rng = self.build_rng();
        format!(
            "stable rand: {} after {}s",
            rng.next_u64(),
            self.elapsed.as_secs()
        )
        .light_green()
        .render(area, buf);
    }
}
