# tui-rain

tui-rain is a simple widget to generate various rain effects for ratatui.

<img src="https://github.com/user-attachments/assets/3fe50f0f-6c01-4864-96ff-58510fd7feb5" width=475 height=270 alt="matrix rain effect">

Features:

- Highly [configurable](#configuration)
- Practically stateless
- Backend-agnostic
- Transparent background

## Examples

### Matrix rain

A classic matrix rain of green half-width kana characters. Press `q` to quit, and `f` to show/hide the FPS tracker.

```sh
cargo run --example matrix
```

<img src="https://github.com/user-attachments/assets/3fe50f0f-6c01-4864-96ff-58510fd7feb5" width=475 height=270 alt="matrix rain effect">

### Normal rain

Drops of fast blue `|` characters. Press `q` to quit, and `f` to show/hide the FPS tracker.

```sh
cargo run --example rain
```

<img src="https://github.com/user-attachments/assets/7e983fa7-9258-4c7b-8b32-4dec56bca67f" width=475 height=270 alt="normal rain effect">

### Snow

Slow-falling white `*` characters. Press `q` to quit, and `f` to show/hide the FPS tracker.

```sh
cargo run --example snow
```

<img src="https://github.com/user-attachments/assets/1e989eba-c45b-4d23-b9d0-be2b06cfc83e" width=475 height=270 alt="snow effect">

### Emoji soup

A chaotic flood of emojis. Terminals that use Unicode version 9+ widths may experience jitter. Press `q` to quit, and `f` to show/hide the FPS tracker.

```sh
cargo run --example emoji
```

<img src="https://github.com/user-attachments/assets/f19895c2-0b47-49a3-8d84-d909206b5b4a" width=475 height=270 alt="emoji rain effect">

### Simple

A demonstration of fairly minimal code to render this widget. Does not listen for key events, and will automatically exit after ~10 seconds.

## Usage

The `Rain` struct is a simple stateless ratatui widget. It can be initially constructed from a few helper functions with defaults, and further configured from there.

Construction requires only an `elapsed` duration to determine what frame to render. This can be provided by just tracking the time the animation was started, and computing `start_time.elapsed()` at render-time. See [simple.rs](examples/simple.rs) for a minimal example.

Construction functions:

- `new_matrix` builds a classic matrix rain of green half-width kana characters
- `new_rain` builds drops of fast blue `|` characters
- `new_snow` builds slow-falling white `*` characters
- `new_emoji_soup` builds a chaotic flood of emojis (may jitter on some terminals)

## Configuration

There are a variety of configuration options available, and they can be sequentially chained:

```rust
use std::time::Duration;
use tui_rain::{CharacterSet, Rain, RainDensity, RainSpeed};

let elapsed = Duration::from_secs(5);

let rain = Rain::new_rain(elapsed)
    .with_character_set(CharacterSet::UnicodeRange {
        start: 0x61,
        len: 26,
    })
    .with_rain_density(RainDensity::Relative {
        sparseness: 50,
    })
    .with_rain_speed(RainSpeed::Absolute {
        speed: 10.0,
    })
    .with_rain_speed_variance(0.1)
    .with_tail_lifespan(Duration::from_secs(5))
    .with_color(ratatui::style::Color::LightGreen)
    .with_noise_interval(Duration::from_secs(10))
    .with_seed(1234);
```

### Character set

The simplest option is to provide an explicit set of characters to choose from:

```rust
use std::time::Duration;
use tui_rain::{CharacterSet, Rain};

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_character_set(CharacterSet::Explicit {
        options: vec!['a', 'b', 'c'],
    });
```

More performant is to provide a unicode range:

```rust
use std::time::Duration;
use tui_rain::{CharacterSet, Rain};

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_character_set(CharacterSet::UnicodeRange {
        start: 0x61,
        len: 26,
    });
```

Preset unicode ranges include:

- `CharacterSet::HalfKana` is the half-width Japanese kana character set (used in the classic matrix rain)
- `CharacterSet::Lowercase` is the lowercase English character set

### Density

This can be configured as an absolute number of drops:

```rust
use std::time::Duration;
use tui_rain::{Rain, RainDensity};

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_rain_density(RainDensity::Absolute {
        num_drops: 100,
    });
```

Or a ratio of screen pixels to drops (lower is more dense):

```rust
use std::time::Duration;
use tui_rain::{Rain, RainDensity};

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_rain_density(RainDensity::Relative {
        sparseness: 50,
    });
```

The actual number of drops on the screen at any time is randomly distributed between 0 and twice the target.

Preset relative options include:

- `RainDensity::Sparse`
- `RainDensity::Normal`
- `RainDensity::Dense`

### Speed

Speed can be configured as an absolute value of pixels per second, or as a preset.

For an absolute speed in pixels per second:

```rust
use std::time::Duration;
use tui_rain::{Rain, RainSpeed};

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_rain_speed(RainSpeed::Absolute {
        speed: 10.0,
    });
```

Preset options include:

- `RainSpeed::Slow`
- `RainSpeed::Normal`
- `RainSpeed::Fast`

### Speed Variance

To avoid perfectly consistent patterns, you can configure some variance in the speed of each drop. This can also give an impression of parallax (depth).

For example, a value of `0.1` will cause each drop's speed to be uniformly distrbuted within ±10% of the target speed:

```rust
use std::time::Duration;
use tui_rain::Rain;

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_rain_speed_variance(0.1);
```

The speed of an individual drop will never go below 0.001 pixels / second, but can vary arbitrarily high.

### Tail lifespan

You can make the rain drop tails appear shorter / longer by configuring how long the tail effect lasts:

```rust
use std::time::Duration;
use tui_rain::Rain;

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_tail_lifespan(Duration::from_secs(5));
```

The drop length is capped at the screen height to avoid strange wraparound effects.

### Color

You can change the tail color for each drop:

```rust
use std::time::Duration;
use tui_rain::Rain;

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_color(ratatui::style::Color::LightGreen);
```

The color of the head is [independently configured](#head-color). The bold / dim effects that automatically get applied over a drop's length may tweak the color inadvertently, but [this can be disabled](#bolddim-effect).

### Head Color

You can change the head color for each drop:

```rust
use std::time::Duration;
use tui_rain::Rain;

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_head_color(ratatui::style::Color::Green);
```

The color of the tail is [independently configured](#color). The bold / dim effects that automatically get applied over a drop's length may tweak the color inadvertently, but [this can be disabled](#bolddim-effect).

### Bold/Dim Effect

By default, the lower third of each drop has the bold effect applied, and the upper third has the dim effect applied. This produces an impression of the drop fading instead of abruptly ending.

This may tweak the color of glyphs away from the base color on some terminals, so it can be disabled if desired:

```rust
use std::time::Duration;
use tui_rain::Rain;

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_bold_dim_effect(false);
```

### Noise Interval

A more subtle effect is that glyphs already rendered in a drop occasionally switch characters before dissapearing. The time interval between each character switch is per-glyph, and can be adjusted:

```rust
use std::time::Duration;
use tui_rain::Rain;

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_noise_interval(Duration::from_secs(10));
```

### Random seed

The random seed can be configured. Given a constant screen size, results should be reproducible across executions, operating systems, and architectures.

```rust
use std::time::Duration;
use tui_rain::Rain;

let elapsed = Duration::from_secs(5);

Rain::new_matrix(elapsed)
    .with_seed(1234);
```
