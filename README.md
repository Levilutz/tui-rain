# tui-rain

tui-rain is a simple widget to generate various rain effects for ratatui.

Features:

- Highly configurable
- Practically stateless
- Backend-agnostic

## Examples

```sh
cargo run --example matrix
```

```sh
cargo run --example rain
```

```sh
cargo run --example snow
```

## Usage

The `Rain` struct is a simple stateless ratatui widget. It can be initially constructed from a few helper functions for defaults, and further configured from there.

It requires an `elapsed` duration to determine what frame to render. This can be provided by just tracking the time the animation was started, and computing `start_time.elapsed()` at render-time. See [simple.rs](examples/simple.rs) for an absolutely minimal example.

Construction functions:

- `new_matrix` builds a classic matrix rain of green half-width kana characters
- `new_rain` builds drops of fast blue `|` characters
- `new_snow` builds slow-falling white `*` characters
- `new_emoji_soup` builds a chaotic flood of emoji characters, which may not work well on all terminals

## Configuration

There are a variety of configuration options available. They can be chained to sequentially apply the configuration:

```rust
Rain::new_matrix(elapsed)
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
Rain::new_matrix(elapsed)
    .with_character_set(CharacterSet::Explicit {
        options: vec!['a', 'b', 'c'],
    });
```

More performant is to provide a unicode range:

```rust
Rain::new_matrix(elapsed)
    .with_character_set(CharacterSet::UnicodeRange {
        start: 0x61,
        len: 26,
    });
```

Preset unicode ranges include:

- `CharacterSet::HalfKana`
- `CharacterSet::Lowercase`

### Density

Density can be configured either as an absolute number of target drops to have on the screen, or a relative number to scale with screen size. The actual number of drops on the srceen at any time is randomly distributed `~ bin(2n, 0.5)`.

You can provide an absolute number of drops:

```rust
Rain::new_matrix(elapsed)
    .with_rain_density(RainDensity::Absolute {
        num_drops: 100,
    });
```

Or a ratio of screen pixels to drops (lower is more dense):

```rust
Rain::new_matrix(elapsed)
    .with_rain_density(RainDensity::Relative {
        sparseness: 50,
    });
```

Preset relative options include:

- `RainDensity::Sprinkling`
- `RainDensity::Showering`
- `RainDensity::Torrential`

### Speed

Speed can be configured as an absolute value of pixels per second, or as a preset.

For an absolute speed in pixels per second:

```rust
Rain::new_matrix(elapsed)
    .with_rain_speed(RainSpeed::Absolute {
        speed: 10.0,
    });
```

Preset options include:

- `RainSpeed::Trickling`
- `RainSpeed::Pouring`
- `RainSpeed::Beating`

### Speed Variance

To avoid perfectly consistent patterns, you can configure some variance in the speed of each drop. This can also give an impression of parallax (depth).

For example, a value of `0.1` will cause each drop's speed to be uniformly distrbuted within ±10% of the target speed:

```rust
Rain::new_matrix(elapsed)
    .with_rain_speed_variance(0.1);
```

### Tail lifespan

You can make the rain drop tails appear shorter / longer by configuring how long the tail effect lasts:

```rust
Rain::new_matrix(elapsed)
    .with_tail_lifespan(Duration::from_secs(5));
```

### Color

You can change the base color for each drop:

```rust
Rain::new_matrix(elapsed)
    .with_color(ratatui::style::Color::LightGreen);
```

The head color will always be white and cannot currently be configured. The bold / dim effects that automatically get applied over a drop's length may tweak the color inadvertently, depending on your terminal.

### Noise Interval

A more subtle effect is that glyphs already rendered in a drop occasionally switch characters before dissapearing. The time interval between each character switch is per-glyph, and can be adjusted:

```rust
Rain::new_matrix(elapsed)
    .with_noise_interval(Duration::from_secs(10));
```

### Random seed

The random seed can be configured. Given a constant screen size, results should be reproducible across executions, operating systems, and architectures.

```rust
Rain::new_matrix(elapsed)
    .with_seed(1234);
```
