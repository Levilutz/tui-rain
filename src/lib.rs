use std::{cmp::Ordering, time::Duration, u64};

use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg64Mcg;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
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

    /// A torrential rain. Equivalent to `Relative { sparseness: 20 }`.
    Torrential,

    /// A showering rain. Equivalent to `Relative { sparseness: 50 }`.
    Showering,

    /// A sprinkling rain. Equivalent to `Relative { sparseness: 100 }`.
    Sprinkling,
}

impl RainDensity {
    /// Get the absolute number of drops given an area.
    fn num_drops(&self, area: Rect) -> usize {
        match self {
            RainDensity::Absolute { num_drops } => *num_drops,
            RainDensity::Relative { sparseness } if *sparseness == 0 => 0,
            RainDensity::Relative { sparseness } => {
                (area.width * area.height) as usize / *sparseness
            }
            RainDensity::Torrential => RainDensity::Relative { sparseness: 20 }.num_drops(area),
            RainDensity::Showering => RainDensity::Relative { sparseness: 50 }.num_drops(area),
            RainDensity::Sprinkling => RainDensity::Relative { sparseness: 100 }.num_drops(area),
        }
    }
}

/// The speed of the rain.
pub enum RainSpeed {
    /// An absolute target speed in pixels / second.
    Absolute { speed: f64 },

    /// A beating rain. Equivalent to `Absolute { speed: 20.0 }`.
    Beating,

    /// A pouring rain. Equivalent to `Absolute { speed: 10.0 }`.
    Pouring,

    /// A trickling rain. Equivalent to `Absolute { speed: 5.0 }`.
    Trickling,
}

impl RainSpeed {
    /// Get the absolute speed.
    fn speed(&self) -> f64 {
        match self {
            RainSpeed::Absolute { speed } => *speed,
            RainSpeed::Beating => 20.0,
            RainSpeed::Pouring => 10.0,
            RainSpeed::Trickling => 5.0,
        }
    }
}

/// A character set for the rain.
pub enum CharacterSet {
    /// An explicit enumeration of character options. This is the least performant.
    Explicit { options: Vec<char> },

    /// A range of unicode values.
    UnicodeRange { start: u32, len: u32 },

    /// Half-width Japanese Kana characters. This is the closest to the original.
    ///
    /// Equivalent to `CharacterSet::UnicodeRange { start: 0xFF66, len: 56 }`.
    HalfKana,

    /// The lowercase English alphabet.
    ///
    /// Equivalent to `CharacterSet::UnicodeRange { start: 0x61, len: 26 }`.
    Lowercase,
}

impl CharacterSet {
    fn get(&self, seed: u32) -> char {
        match self {
            CharacterSet::Explicit { options } => options[seed as usize % options.len()],
            CharacterSet::UnicodeRange { start, len } => {
                char::from_u32((seed % len) + start).unwrap()
            }
            CharacterSet::HalfKana => CharacterSet::UnicodeRange {
                start: 0xFF66,
                len: 56,
            }
            .get(seed),
            CharacterSet::Lowercase => CharacterSet::UnicodeRange {
                start: 0x61,
                len: 26,
            }
            .get(seed),
        }
    }

    fn size(&self) -> usize {
        match self {
            CharacterSet::Explicit { options } => options.len(),
            CharacterSet::UnicodeRange { start: _, len } => *len as usize,
            CharacterSet::HalfKana => 56,
            CharacterSet::Lowercase => 26,
        }
    }
}

pub struct Rain {
    elapsed: Duration,
    seed: u64,
    rain_density: RainDensity,
    rain_speed: RainSpeed,
    rain_speed_variance: f64,
    tail_lifespan: Duration,
    color: Color,
    noise_interval: Duration,
    character_set: CharacterSet,
}

impl Rain {
    /// Construct a new rain widget with defaults for matrix rain.
    ///
    /// Defaults are:
    /// - seed: 1234
    /// - rain_density: Showering
    /// - rain_speed: Trickling
    /// - rain_speed_variance: 0.5,
    /// - tail_lifespan: 2s
    /// - color: LightGreen
    /// - noise_interval: 5s
    /// - character_set: HalfKana
    pub fn new_matrix(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
            rain_density: RainDensity::Showering,
            rain_speed: RainSpeed::Trickling,
            rain_speed_variance: 0.5,
            tail_lifespan: Duration::from_secs(2),
            color: Color::LightGreen,
            noise_interval: Duration::from_secs(5),
            character_set: CharacterSet::HalfKana,
        }
    }

    /// Construct a new rain widget with defaults for standard rain.
    ///
    /// Defaults are:
    /// - seed: 1234
    /// - rain_density: Torrential
    /// - rain_speed: Beating
    /// - rain_speed_variance: 0.5,
    /// - tail_lifespan: 250ms
    /// - color: LightBlue
    /// - noise_interval: 1s
    /// - character_set: UnicodeRange { start: 0x7c, len: 1 }
    pub fn new_rain(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
            rain_density: RainDensity::Torrential,
            rain_speed: RainSpeed::Beating,
            rain_speed_variance: 0.5,
            tail_lifespan: Duration::from_millis(250),
            color: Color::LightBlue,
            noise_interval: Duration::from_secs(1),
            character_set: CharacterSet::UnicodeRange {
                start: 0x7c,
                len: 1,
            },
        }
    }

    /// Construct a new rain widget with defaults for snow.
    ///
    /// Defaults are:
    /// - seed: 1234
    /// - rain_density: Torrential
    /// - rain_speed: Absolute { speed: 2.0 }
    /// - rain_speed_variance: 0.1,
    /// - tail_lifespan: 500ms
    /// - color: White
    /// - noise_interval: 1s
    /// - character_set: UnicodeRange { start: 0x2a, len: 1 }
    pub fn new_snow(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
            rain_density: RainDensity::Torrential,
            rain_speed: RainSpeed::Absolute { speed: 2.0 },
            rain_speed_variance: 0.1,
            tail_lifespan: Duration::from_millis(500),
            color: Color::White,
            noise_interval: Duration::from_secs(1),
            character_set: CharacterSet::UnicodeRange {
                start: 0x2a,
                len: 1,
            },
        }
    }

    /// Construct a new rain widget with defaults for emoji soup.
    ///
    /// Terminals that render emojis as two characters wide will not enjoy this.
    ///
    /// Defaults are:
    /// - seed: 1234
    /// - rain_density: Torrential
    /// - rain_speed: Pouring
    /// - rain_speed_variance: 0.1,
    /// - tail_lifespan: 500ms
    /// - color: White
    /// - noise_interval: 1s
    /// - character_set: UnicodeRange { start: 0x1f600, len: 80 }
    pub fn new_emoji_soup(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
            rain_density: RainDensity::Torrential,
            rain_speed: RainSpeed::Pouring,
            rain_speed_variance: 0.1,
            tail_lifespan: Duration::from_millis(500),
            color: Color::White,
            noise_interval: Duration::from_secs(1),
            character_set: CharacterSet::UnicodeRange {
                start: 0x1f600,
                len: 80,
            },
        }
    }

    /// Set the random seed for the generation. Default is 1234.
    pub fn with_seed(mut self, seed: u64) -> Rain {
        self.seed = seed;
        self
    }

    /// Set the target density for the rain. Default is Showering.
    pub fn with_rain_density(mut self, rain_density: RainDensity) -> Rain {
        self.rain_density = rain_density;
        self
    }

    /// Set the target speed for the rain. Default is Trickling.
    pub fn with_rain_speed(mut self, rain_speed: RainSpeed) -> Rain {
        self.rain_speed = rain_speed;
        self
    }

    /// Set the rain speed variance. Default is 0.5.
    ///
    /// Value of 0.1 means rain will be uniformly distributed ±10% of the target speed.
    pub fn with_rain_speed_variance(mut self, rain_speed_variance: f64) -> Rain {
        self.rain_speed_variance = rain_speed_variance;
        self
    }

    /// Set the tail lifespan for the rain. Default is 2 seconds.
    pub fn with_tail_lifespan(mut self, tail_lifespan: Duration) -> Rain {
        self.tail_lifespan = tail_lifespan;
        self
    }

    /// Set the color for the rain. Default is LightGreen.
    pub fn with_color(mut self, color: Color) -> Rain {
        self.color = color;
        self
    }

    /// Set the interval between random character changes. Default is 5 seconds.
    pub fn with_noise_interval(mut self, noise_interval: Duration) -> Rain {
        self.noise_interval = noise_interval;
        self
    }

    /// Set the character set for the drops. Defualt is HalfKana.
    pub fn with_character_set(mut self, character_set: CharacterSet) -> Rain {
        self.character_set = character_set;
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
        // We don't actually have n drops with tracks equal to the screen height.
        // We actually have 2n drops with tracks ranging from 1.5 to 2.5 the screen height.
        // This introduces more randomness to the apparent n and reduces cyclic appearance.
        let num_drops = self.rain_density.num_drops(area) * 2;
        let drop_track_lens: Vec<usize> = (0..num_drops)
            .map(|_| (area.height as u64 * 3 / 2 + rng.next_u64() % area.height as u64) as usize)
            .collect();
        let entropy: Vec<Vec<u64>> = drop_track_lens
            .iter()
            .map(|track_len| {
                (0..*track_len)
                    .into_iter()
                    .map(|_| rng.next_u64())
                    .collect()
            })
            .collect();

        let mut glyphs: Vec<Glyph> = entropy
            .into_iter()
            .map(|drop_entropy| {
                build_drop(
                    &self.character_set,
                    drop_entropy,
                    elapsed,
                    area.width,
                    area.height,
                    self.rain_speed.speed(),
                    self.rain_speed_variance,
                    self.tail_lifespan.as_secs_f64(),
                    self.noise_interval.as_secs_f64(),
                    self.color,
                )
            })
            .flatten()
            .collect();

        glyphs.sort_by(|a, b| a.age.partial_cmp(&b.age).unwrap_or(Ordering::Equal));

        for glyph in glyphs {
            buf[(glyph.x, glyph.y)].set_char(glyph.content);
            buf[(glyph.x, glyph.y)].set_style(glyph.style);
        }
    }
}

/// A Glyph to be rendered on the screen.
struct Glyph {
    x: u16,
    y: u16,
    age: f64,
    content: char,
    style: Style,
}

/// Build a drop from the given consistent initial entropy state.
///
/// The entropy vector's length becomes the drop's track length, so ensure it's at least
/// the window height.
fn build_drop(
    character_set: &CharacterSet,
    entropy: Vec<u64>,
    elapsed: f64,
    width: u16,
    height: u16,
    rain_speed: f64,
    rain_speed_variance: f64,
    tail_lifespan: f64,
    noise_interval: f64,
    color: Color,
) -> Vec<Glyph> {
    if entropy.len() == 0 {
        return vec![];
    }
    let track_len = entropy.len() as u16;
    let rain_speed = uniform(
        entropy[0],
        rain_speed * (1.0 - rain_speed_variance),
        rain_speed * (1.0 + rain_speed_variance),
    )
    .max(1e-3);
    let cycle_time_secs = entropy.len() as f64 / rain_speed;
    let initial_cycle_offset_secs = uniform(entropy[0], 0.0, cycle_time_secs);
    let current_cycle_offset_secs = (elapsed + initial_cycle_offset_secs) % cycle_time_secs;
    let head_y = (current_cycle_offset_secs * rain_speed) as u16;
    let drop_len = ((rain_speed * tail_lifespan) as u16).min(height);
    (0..drop_len)
        .into_iter()
        .filter_map(|y_offset| {
            let age = y_offset as f64 / rain_speed;
            if age > elapsed {
                return None;
            }
            let cycle_num =
                ((elapsed + initial_cycle_offset_secs - age) / cycle_time_secs) as usize;
            if cycle_num == 0 {
                return None;
            }
            let x_entropy = entropy[cycle_num % entropy.len()];
            let x = (x_entropy % width as u64) as u16;
            let y = (head_y + track_len - y_offset) % track_len;
            if y >= height {
                return None;
            }
            let time_offset = uniform(
                entropy[y as usize],
                0.0,
                noise_interval * character_set.size() as f64,
            );
            let content = character_set.get(((time_offset + elapsed) / noise_interval) as u32);
            let mut style = Style::default();
            if age > 0.0 {
                style = style.fg(color)
            }
            if y_offset < drop_len / 3 {
                style = style.bold()
            } else if y_offset > drop_len * 2 / 3 {
                style = style.dim()
            }
            Some(Glyph {
                x,
                y,
                age,
                content,
                style,
            })
        })
        .collect()
}

/// Map a uniform random u64 to a uniform random f64 in the range [lower, upper).
fn uniform(seed: u64, lower: f64, upper: f64) -> f64 {
    (seed as f64 / u64::MAX as f64) * (upper - lower) + lower
}
