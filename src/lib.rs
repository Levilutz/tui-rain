#![doc = include_str!("../README.md")]

use std::{cmp::Ordering, time::Duration, u64};

use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg64Mcg;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::Widget,
};

/// A configuration for the density of the rain effect.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum RainDensity {
    /// An absolute target number of drops to have in the frame.
    Absolute { num_drops: usize },

    /// Compute the number of drops based on the frame size. Lower value is denser.
    ///
    /// Is converted to an absolute value, with 1 drop per `sparseness` pixels.
    Relative { sparseness: usize },

    /// A dense rain. Equivalent to `Relative { sparseness: 20 }`.
    Dense,

    /// A normal rain. Equivalent to `Relative { sparseness: 50 }`.
    Normal,

    /// A sparse rain. Equivalent to `Relative { sparseness: 100 }`.
    Sparse,
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
            RainDensity::Dense => RainDensity::Relative { sparseness: 20 }.num_drops(area),
            RainDensity::Normal => RainDensity::Relative { sparseness: 50 }.num_drops(area),
            RainDensity::Sparse => RainDensity::Relative { sparseness: 100 }.num_drops(area),
        }
    }
}

/// The speed of the rain.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum RainSpeed {
    /// An absolute target speed in pixels / second.
    Absolute { speed: f64 },

    /// A fast rain. Equivalent to `Absolute { speed: 20.0 }`.
    Fast,

    /// A normal rain. Equivalent to `Absolute { speed: 10.0 }`.
    Normal,

    /// A slow rain. Equivalent to `Absolute { speed: 5.0 }`.
    Slow,
}

impl RainSpeed {
    /// Get the absolute speed.
    fn speed(&self) -> f64 {
        match self {
            RainSpeed::Absolute { speed } => *speed,
            RainSpeed::Fast => 20.0,
            RainSpeed::Normal => 10.0,
            RainSpeed::Slow => 5.0,
        }
    }
}

/// A character set for the rain.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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

#[derive(Clone, PartialEq, Debug)]
pub struct Rain {
    elapsed: Duration,
    seed: u64,
    rain_density: RainDensity,
    rain_speed: RainSpeed,
    rain_speed_variance: f64,
    tail_lifespan: Duration,
    color: Color,
    head_color: Color,
    bold_dim_effect: bool,
    noise_interval: Duration,
    character_set: CharacterSet,
}

impl Rain {
    /// Construct a new rain widget with defaults for matrix rain.
    pub fn new_matrix(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
            rain_density: RainDensity::Normal,
            rain_speed: RainSpeed::Slow,
            rain_speed_variance: 0.5,
            tail_lifespan: Duration::from_secs(2),
            color: Color::LightGreen,
            head_color: Color::White,
            bold_dim_effect: true,
            noise_interval: Duration::from_secs(5),
            character_set: CharacterSet::HalfKana,
        }
    }

    /// Construct a new rain widget with defaults for standard rain.
    pub fn new_rain(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
            rain_density: RainDensity::Dense,
            rain_speed: RainSpeed::Fast,
            rain_speed_variance: 0.5,
            tail_lifespan: Duration::from_millis(250),
            color: Color::LightBlue,
            head_color: Color::White,
            bold_dim_effect: true,
            noise_interval: Duration::from_secs(1),
            character_set: CharacterSet::UnicodeRange {
                start: 0x7c,
                len: 1,
            },
        }
    }

    /// Construct a new rain widget with defaults for snow.
    pub fn new_snow(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
            rain_density: RainDensity::Dense,
            rain_speed: RainSpeed::Absolute { speed: 2.0 },
            rain_speed_variance: 0.1,
            tail_lifespan: Duration::from_millis(500),
            color: Color::White,
            head_color: Color::White,
            bold_dim_effect: true,
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
    pub fn new_emoji_soup(elapsed: Duration) -> Rain {
        Rain {
            elapsed,
            seed: 1234,
            rain_density: RainDensity::Dense,
            rain_speed: RainSpeed::Normal,
            rain_speed_variance: 0.1,
            tail_lifespan: Duration::from_millis(500),
            color: Color::White,
            head_color: Color::White,
            bold_dim_effect: true,
            noise_interval: Duration::from_secs(1),
            character_set: CharacterSet::UnicodeRange {
                start: 0x1f600,
                len: 80,
            },
        }
    }

    /// Set the random seed for the generation.
    ///
    /// The random seed can be configured. Given a constant screen size, results should
    /// be reproducible across executions, operating systems, and architectures.
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::Rain;
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_seed(1234);
    /// ```
    pub fn with_seed(mut self, seed: u64) -> Rain {
        self.seed = seed;
        self
    }

    /// Set the target density for the rain.
    ///
    /// This can be configured as an absolute number of drops:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::{Rain, RainDensity};
    ///
    /// Rain::new_matrix(Duration::from_secs(0))
    ///     .with_rain_density(RainDensity::Absolute {
    ///         num_drops: 100,
    ///     });
    /// ```
    /// Or a ratio of screen pixels to drops (lower is more dense):
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::{Rain, RainDensity};
    ///
    /// Rain::new_matrix(Duration::from_secs(0))
    ///     .with_rain_density(RainDensity::Relative {
    ///         sparseness: 50,
    ///     });
    /// ```
    ///
    /// The actual number of drops on the screen at any time is randomly distributed
    /// between 0 and twice the target.
    ///
    /// Preset relative options include:
    ///
    /// - `RainDensity::Sparse`
    /// - `RainDensity::Normal`
    /// - `RainDensity::Dense`
    pub fn with_rain_density(mut self, rain_density: RainDensity) -> Rain {
        self.rain_density = rain_density;
        self
    }

    /// Set the target speed for the rain.
    ///
    /// Speed can be configured as an absolute value of pixels per second, or as a
    /// preset.
    ///
    /// For an absolute speed in pixels per second:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::{Rain, RainSpeed};
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_rain_speed(RainSpeed::Absolute {
    ///         speed: 10.0,
    ///     });
    /// ```
    ///
    /// Preset options include:
    ///
    /// - `RainSpeed::Slow`
    /// - `RainSpeed::Normal`
    /// - `RainSpeed::Fast`
    pub fn with_rain_speed(mut self, rain_speed: RainSpeed) -> Rain {
        self.rain_speed = rain_speed;
        self
    }

    /// Set the rain speed variance.
    ///
    /// To avoid perfectly consistent patterns, you can configure some variance in the
    /// speed of each drop. This can also give an impression of parallax (depth).
    ///
    /// For example, a value of `0.1` will cause each drop's speed to be uniformly
    /// distrbuted within ±10% of the target speed:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::Rain;
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_rain_speed_variance(0.1);
    /// ```
    ///
    /// The speed of an individual drop will never go below 0.001 pixels / second, but
    /// can vary arbitrarily high.
    pub fn with_rain_speed_variance(mut self, rain_speed_variance: f64) -> Rain {
        self.rain_speed_variance = rain_speed_variance;
        self
    }

    /// Set the tail lifespan for the rain.
    ///
    /// You can make the rain drop tails appear shorter / longer by configuring how long
    /// the tail effect lasts:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::Rain;
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_tail_lifespan(Duration::from_secs(5));
    /// ```
    ///
    /// The drop length is capped at the screen height to avoid strange wraparound
    /// effects.
    pub fn with_tail_lifespan(mut self, tail_lifespan: Duration) -> Rain {
        self.tail_lifespan = tail_lifespan;
        self
    }

    /// Set the color for the rain.
    ///
    /// You can change the tail color for each drop:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::Rain;
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_color(ratatui::style::Color::LightGreen);
    /// ```
    ///
    /// The color of the head is [independently configured](Rain::with_head_color). The
    /// bold / dim effects that automatically get applied over a drop's length may tweak
    /// the color inadvertently, but [this can be disabled](Rain::with_bold_dim_effect).
    pub fn with_color(mut self, color: Color) -> Rain {
        self.color = color;
        self
    }

    /// Set the head color for the rain.
    ///
    /// You can change the head color for each drop:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::Rain;
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_head_color(ratatui::style::Color::Green);
    /// ```
    ///
    /// The color of the tail is [independently configured](Rain::with_color). The
    /// bold / dim effects that automatically get applied over a drop's length may tweak
    /// the color inadvertently, but [this can be disabled](Rain::with_bold_dim_effect).
    pub fn with_head_color(mut self, head_color: Color) -> Rain {
        self.head_color = head_color;
        self
    }

    /// Set whether to apply the bold / dim effect.
    ///
    /// By default, the lower third of each drop has the bold effect applied, and the
    /// upper third has the dim effect applied. This produces an impression of the drop
    /// fading instead of abruptly ending.
    ///
    /// This may tweak the color of glyphs away from the base color on some terminals,
    /// so it can be disabled if desired:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::Rain;
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_bold_dim_effect(false);
    ///```
    pub fn with_bold_dim_effect(mut self, bold_dim_effect: bool) -> Rain {
        self.bold_dim_effect = bold_dim_effect;
        self
    }

    /// Set the interval between random character changes.
    ///
    /// A more subtle effect is that glyphs already rendered in a drop occasionally
    /// switch characters before dissapearing. The time interval between each character
    /// switch is per-glyph, and can be adjusted:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::Rain;
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_noise_interval(Duration::from_secs(10));
    /// ```
    pub fn with_noise_interval(mut self, noise_interval: Duration) -> Rain {
        self.noise_interval = noise_interval;
        self
    }

    /// Set the character set for the drops.
    ///
    /// The simplest option is to provide an explicit set of characters to choose from:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::{CharacterSet, Rain};
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_character_set(CharacterSet::Explicit {
    ///         options: vec!['a', 'b', 'c'],
    ///     });
    /// ```
    ///
    /// More performant is to provide a unicode range:
    ///
    /// ```
    /// use std::time::Duration;
    /// use tui_rain::{CharacterSet, Rain};
    ///
    /// let elapsed = Duration::from_secs(5);
    ///
    /// Rain::new_matrix(elapsed)
    ///     .with_character_set(CharacterSet::UnicodeRange {
    ///         start: 0x61,
    ///         len: 26,
    ///     });
    /// ```
    ///
    /// Preset unicode ranges include:
    ///
    /// - `CharacterSet::HalfKana` is the half-width Japanese kana character set (used
    /// in the classic matrix rain)
    /// - `CharacterSet::Lowercase` is the lowercase English character set
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

        // We construct entropy consistently every frame to mimic statefulness.
        // This is not a performance bottleneck, so caching wouldn't deliver much benefit.
        let entropy: Vec<Vec<u64>> = drop_track_lens
            .iter()
            .map(|track_len| {
                (0..*track_len)
                    .into_iter()
                    .map(|_| rng.next_u64())
                    .collect()
            })
            .collect();

        // For every entropy vec, construct a single drop (vertical line of glyphs).
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
                    self.head_color,
                    self.bold_dim_effect,
                )
            })
            .flatten()
            .collect();

        // Sort all the glyphs by age so drop heads always render on top.
        // This is a moderate bottleneck when the screen is large / there's a lot of glyphs.
        glyphs.sort_by(|a, b| a.age.partial_cmp(&b.age).unwrap_or(Ordering::Equal));

        // Actually render to the buffer.
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
    head_color: Color,
    bold_dim_effect: bool,
) -> Vec<Glyph> {
    // A single drop can expect to be called with the exact same entropy vec on each frame.
    // This means we can sample the entropy vec to reproducibly generate features every frame (e.g. speed).

    // Later code assumes at least 1 entry in the entropy vec, so break early if not.
    if entropy.len() == 0 {
        return vec![];
    }

    // The length of the entropy vec becomes the length of the drop's track.
    // This track is usually longer than the screen height by a random amount.
    let track_len = entropy.len() as u16;

    // Use some entropy to compute the drop's actual speed.
    // n.b. since the entropy vec is stable, the drop's speed will not vary over time.
    let rain_speed = uniform(
        entropy[0],
        rain_speed * (1.0 - rain_speed_variance),
        rain_speed * (1.0 + rain_speed_variance),
    )
    .max(1e-3); // Prevent speed from hitting 0 (if user specifies high variance)

    // Compute how long our drop will take to make 1 cycle given our track len and speed
    let cycle_time_secs = entropy.len() as f64 / rain_speed;

    // Use some entropy to compute a stable random time offset for this drop.
    // If this value were 0, every drop would start falling with an identical y value.
    let initial_cycle_offset_secs = uniform(entropy[0], 0.0, cycle_time_secs);

    // Compute how far we are into the current cycle and current drop head height.
    let current_cycle_offset_secs = (elapsed + initial_cycle_offset_secs) % cycle_time_secs;
    let head_y = (current_cycle_offset_secs * rain_speed) as u16;

    // Compute drop length given speed and tail lifespan.
    // Cap at screen height to avoid weird wraparound when tail length is long.
    let drop_len = ((rain_speed * tail_lifespan) as u16).min(height);

    // Render each glyph in the drop.
    (0..drop_len)
        .into_iter()
        .filter_map(|y_offset| {
            // Compute how long ago this glyph would have first appeared
            let age = y_offset as f64 / rain_speed;

            // If it would have first appeared before the rendering began, don't render.
            if age > elapsed {
                return None;
            }

            // Compute which cycle this particular glyph is a member of
            let cycle_num =
                ((elapsed + initial_cycle_offset_secs - age) / cycle_time_secs) as usize;

            // Don't render glyphs from cycle 0
            // (prevents drops from appearing to spawn in the middle of the screen)
            if cycle_num == 0 {
                return None;
            }

            // Get stable entropy to decide what column cycle X is rendered in.
            // This must be per-glyph to prevent drops from jumping side-to-side when they wrap around.
            let x_entropy = entropy[cycle_num % entropy.len()];
            let x = (x_entropy % width as u64) as u16;

            // Compute the y value for this glyph, and don't render if off the screen.
            let y = (head_y + track_len - y_offset) % track_len;
            if y >= height {
                return None;
            }

            // The 'noise' of glyphs randomly changing is actually modeled as every glyph in the track
            // just cycling through possible values veeeery slowly. We need a random offset for this
            // cycling so every glyph doesn't change at the same time.
            let time_offset = uniform(
                entropy[y as usize],
                0.0,
                noise_interval * character_set.size() as f64,
            );

            // Decide what character is rendered based on noise.
            let content = character_set.get(((time_offset + elapsed) / noise_interval) as u32);

            // Compute the styling for the glyph
            let mut style = Style::default();

            // Color appropriately depending on whether this glyph is the head.
            if age > 0.0 {
                style = style.fg(color)
            } else {
                style = style.fg(head_color)
            }

            // The lowest third of glyphs is bold, the highest third is dim
            if bold_dim_effect {
                if y_offset < drop_len / 3 {
                    style = style.bold().not_dim()
                } else if y_offset > drop_len * 2 / 3 {
                    style = style.dim().not_bold()
                } else {
                    style = style.not_bold().not_dim()
                }
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
