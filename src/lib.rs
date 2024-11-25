use std::{time::Duration, u64};

use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg64Mcg;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    widgets::Widget,
};

/// The density of the rain.
pub enum RainDensity {
    /// An absolute target number of drops to have in the frame.
    ///
    /// The number of drops at any point in time ends up being random ~ bin(2n, 0.5).
    Absolute { num_drops: usize },

    /// Compute the number of drops based on the frame size. Lower value is denser.
    ///
    /// Is converted to an absolute value, with 1 drop per `sparseness` pixels.
    Relative { sparseness: usize },

    /// A torrential rain. Equivalent to `Relative { sparseness: 100 }`.
    Torrential,

    /// A showering rain. Equivalent to `Relative { sparseness: 100 }`.
    Showering,

    /// A sprinkling rain. Equivalent to `Relative { sparseness: 100 }`.
    Sprinkling,
}

impl RainDensity {
    /// Get the absolute number of drops given a density.
    fn num_drops(&self, area: Rect) -> usize {
        match self {
            RainDensity::Absolute { num_drops } => *num_drops,
            RainDensity::Relative { sparseness } if *sparseness == 0 => 0,
            RainDensity::Relative { sparseness } => {
                (area.width * area.height) as usize / *sparseness
            }
            RainDensity::Torrential => RainDensity::Relative { sparseness: 100 }.num_drops(area),
            RainDensity::Showering => RainDensity::Relative { sparseness: 100 }.num_drops(area),
            RainDensity::Sprinkling => RainDensity::Relative { sparseness: 100 }.num_drops(area),
        }
    }
}

/// The speed of the rain.
pub enum RainSpeed {
    /// An absolute target speed in pixels / second.
    Absolute { speed: f64 },

    /// A beating rain. Equivalent to `Absolute { speed: 5 }`.
    Beating,

    /// A pouring rain. Equivalent to `Absolute { speed: 5 }`.
    Pouring,

    /// A trickling rain. Equivalent to `Absolute { speed: 5 }`.
    Trickling,
}

impl RainSpeed {
    /// Get the absolute speed.
    fn speed(&self) -> f64 {
        match self {
            RainSpeed::Absolute { speed } => *speed,
            RainSpeed::Beating => 5.0,
            RainSpeed::Pouring => 5.0,
            RainSpeed::Trickling => 5.0,
        }
    }
}

pub struct Rain {
    elapsed: Duration,
    seed: u64,
    rain_density: RainDensity,
    rain_speed: RainSpeed,
    tail_lifespan: Duration,
    color: Color,
    noise_rate: f64,
}

impl Rain {
    /// Construct a new rain widget. Requires only current elapsed duration.
    pub fn new(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
            rain_density: RainDensity::Showering,
            rain_speed: RainSpeed::Pouring,
            tail_lifespan: Duration::from_secs(1),
            color: Color::Green,
            noise_rate: 20.0,
        }
    }

    /// Set the random seed for the generation.
    pub fn with_seed(mut self, seed: u64) -> Rain {
        self.seed = seed;
        self
    }

    /// Set the target density for the rain.
    pub fn with_rain_density(mut self, rain_density: RainDensity) -> Rain {
        self.rain_density = rain_density;
        self
    }

    /// Set the target speed for the rain.
    pub fn with_rain_speed(mut self, rain_speed: RainSpeed) -> Rain {
        self.rain_speed = rain_speed;
        self
    }

    /// Set the tail lifespan for the rain.
    pub fn with_tail_lifespan(mut self, tail_lifespan: Duration) -> Rain {
        self.tail_lifespan = tail_lifespan;
        self
    }

    /// Set the color for the rain.
    pub fn with_color(mut self, color: Color) -> Rain {
        self.color = color;
        self
    }

    /// Set the number of seconds between random character changes. Lower is more frequent.
    pub fn with_noise_rate(mut self, noise_rate: f64) -> Rain {
        self.noise_rate = noise_rate;
        self
    }

    /// Build the rng. Uses a fast but portable and reproducible rng.
    fn build_rng(&self) -> impl RngCore {
        Pcg64Mcg::seed_from_u64(self.seed)
    }
}

impl Widget for Rain {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let elapsed = self.elapsed.as_secs_f64();
        let mut rng = self.build_rng();
        // let num_drops = self.rain_density.num_drops(area);
        // let drop_track_lens: Vec<usize> = (0..num_drops)
        //     .map(|_| (area.height as u64 + rng.next_u64() % (area.height as u64 * 2)) as usize)
        //     .collect();
        let track_pixel_seed: Vec<u64> = (0..area.height).map(|_| rng.next_u64()).collect();

        for y in 0..area.height {
            let time_offset = uniform(track_pixel_seed[y as usize], 0.0, self.noise_rate * 56.0);
            buf[(5, y)].set_symbol(
                &random_half_kana(((time_offset + elapsed) / self.noise_rate) as u64).to_string(),
            );
        }
    }
}

/// Pick a random half width kana. Seed is modulo'd to the correct range.
fn random_half_kana(seed: u64) -> char {
    char::from_u32((seed % 56) as u32 + 65382).unwrap()
}

/// Map a uniform random u64 to a random f64 in the range [lower, upper).
fn uniform(seed: u64, lower: f64, upper: f64) -> f64 {
    (seed as f64 / u64::MAX as f64) * (upper - lower) + lower
}
