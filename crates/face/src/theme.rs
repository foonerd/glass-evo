//! A face theme: the tokens every piece of the face is drawn from. A theme
//! is a text its author writes, sections of `key = value`, in a folder of
//! its own; what it leaves out keeps its default, and what the user sets in
//! the Manager goes over it, key for key. Nothing here draws: the face
//! reads a `Theme` and the look the artwork gives it.

use std::collections::BTreeMap;

/// A colour a theme fixes, or leaves to the artwork.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Paint {
    Artwork,
    Fixed([u8; 3]),
}

/// Whether glass over a moving picture is frosted: as suits the board,
/// which the plugin says, or as the user or the theme has it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Frost {
    Auto,
    On,
    Off,
}

/// The tokens, with the defaults of the design language.
#[derive(Clone, PartialEq, Debug)]
pub struct Theme {
    /// `theme.name`, for a list; empty for the built-in look.
    pub name: String,
    /// `colours.tint`: the colour of glass; `artwork` or `#rrggbb`.
    pub tint: Paint,
    /// `colours.accent`: what is lit; `artwork` or `#rrggbb`.
    pub accent: Paint,
    /// `colours.ink`: text and icons.
    pub ink: [u8; 3],
    /// `colours.plate`: behind text that sits on a picture.
    pub plate: [u8; 3],
    /// `glass.bar`, `glass.sheet`: how much of the glass shows, 0.1 to 1.
    pub bar: f32,
    pub sheet: f32,
    /// `glass.hairline`: the line on a glass's inner edge.
    pub hairline: f32,
    /// `glass.frost`: `auto`, `on` or `off`.
    pub frost: Frost,
    /// `buttons.ink`, `buttons.opacity`: the icons, when they have their own.
    pub buttons_ink: Option<[u8; 3]>,
    pub buttons_opacity: f32,
    /// `clock.ink`, `clock.opacity`, `clock.plate`: the clock's own.
    pub clock_ink: Option<[u8; 3]>,
    pub clock_opacity: f32,
    pub clock_plate: f32,
    /// `measure.bar`, `measure.clock`: heights in units of a 720th of the
    /// picture's height.
    pub measure_bar: f32,
    pub measure_clock: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: String::new(),
            tint: Paint::Artwork,
            accent: Paint::Artwork,
            ink: [242, 242, 245],
            plate: [0, 0, 0],
            bar: 0.75,
            sheet: 0.78,
            hairline: 0.12,
            frost: Frost::Auto,
            buttons_ink: None,
            buttons_opacity: 1.0,
            clock_ink: None,
            clock_opacity: 0.86,
            clock_plate: 0.55,
            measure_bar: 72.0,
            measure_clock: 144.0,
        }
    }
}

/// `#rrggbb` or `#rgb` as a colour.
pub fn colour(text: &str) -> Option<[u8; 3]> {
    let hex = text.trim().strip_prefix('#')?;
    let digit = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let b = hex.as_bytes();
    match b.len() {
        6 => Some([
            digit(b[0])? * 16 + digit(b[1])?,
            digit(b[2])? * 16 + digit(b[3])?,
            digit(b[4])? * 16 + digit(b[5])?,
        ]),
        3 => Some([digit(b[0])? * 17, digit(b[1])? * 17, digit(b[2])? * 17]),
        _ => None,
    }
}

/// A theme's text as its keys, each under its section: `[glass]` and
/// `bar = 0.6` is `glass.bar`. A key written before any section, or
/// written whole (`glass.bar = 0.6`), stands as it is.
pub fn keys(text: &str) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name.trim().to_ascii_lowercase();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        if key.is_empty() {
            continue;
        }
        let name = if section.is_empty() {
            key
        } else {
            format!("{section}.{key}")
        };
        found.insert(name, value.trim().to_string());
    }
    found
}

impl Theme {
    /// The keys laid over these tokens: a key the face does not know, or a
    /// value it cannot read, changes nothing, so a theme written for a
    /// later face still draws on an earlier one.
    pub fn apply(&mut self, keys: &BTreeMap<String, String>) {
        let paint = |v: &str| {
            if v.eq_ignore_ascii_case("artwork") {
                Some(Paint::Artwork)
            } else {
                colour(v).map(Paint::Fixed)
            }
        };
        // An opacity is never set so low that a control is there and cannot be seen.
        let share = |v: &str| {
            v.parse::<f32>()
                .ok()
                .filter(|n| n.is_finite())
                .map(|n| n.clamp(0.1, 1.0))
        };
        let units = |v: &str, least: f32, most: f32| {
            v.parse::<f32>()
                .ok()
                .filter(|n| n.is_finite())
                .map(|n| n.clamp(least, most))
        };
        for (key, value) in keys {
            let v = value.as_str();
            match key.as_str() {
                "theme.name" => self.name = v.to_string(),
                "colours.tint" | "colors.tint" => self.tint = paint(v).unwrap_or(self.tint),
                "colours.accent" | "colors.accent" => self.accent = paint(v).unwrap_or(self.accent),
                "colours.ink" | "colors.ink" => self.ink = colour(v).unwrap_or(self.ink),
                "colours.plate" | "colors.plate" => self.plate = colour(v).unwrap_or(self.plate),
                "glass.bar" => self.bar = share(v).unwrap_or(self.bar),
                "glass.sheet" => self.sheet = share(v).unwrap_or(self.sheet),
                "glass.hairline" => {
                    self.hairline = v
                        .parse::<f32>()
                        .ok()
                        .filter(|n| n.is_finite())
                        .map(|n| n.clamp(0.0, 1.0))
                        .unwrap_or(self.hairline)
                }
                "glass.frost" => {
                    self.frost = match v.to_ascii_lowercase().as_str() {
                        "on" | "true" | "yes" => Frost::On,
                        "off" | "false" | "no" => Frost::Off,
                        "auto" => Frost::Auto,
                        _ => self.frost,
                    }
                }
                "buttons.ink" => {
                    self.buttons_ink = if v.eq_ignore_ascii_case("ink") {
                        None
                    } else {
                        colour(v).or(self.buttons_ink)
                    }
                }
                "buttons.opacity" => {
                    self.buttons_opacity = share(v).unwrap_or(self.buttons_opacity)
                }
                "clock.ink" => {
                    self.clock_ink = if v.eq_ignore_ascii_case("ink") {
                        None
                    } else {
                        colour(v).or(self.clock_ink)
                    }
                }
                "clock.opacity" => self.clock_opacity = share(v).unwrap_or(self.clock_opacity),
                "clock.plate" => {
                    self.clock_plate = v
                        .parse::<f32>()
                        .ok()
                        .filter(|n| n.is_finite())
                        .map(|n| n.clamp(0.0, 1.0))
                        .unwrap_or(self.clock_plate)
                }
                "measure.bar" => {
                    self.measure_bar = units(v, 48.0, 240.0).unwrap_or(self.measure_bar)
                }
                "measure.clock" => {
                    self.measure_clock = units(v, 48.0, 360.0).unwrap_or(self.measure_clock)
                }
                _ => {}
            }
        }
    }

    /// The tokens in use: the defaults, the theme's text over them, the
    /// user's settings over that.
    pub fn resolve(theme_text: Option<&str>, settings: &BTreeMap<String, String>) -> Self {
        let mut theme = Self::default();
        if let Some(text) = theme_text {
            theme.apply(&keys(text));
        }
        theme.apply(settings);
        theme
    }

    /// Whether glass over a moving picture is frosted, given the plugin's
    /// word on the board (`frost.suits` among the settings).
    pub fn frosted(&self, settings: &BTreeMap<String, String>) -> bool {
        match self.frost {
            Frost::On => true,
            Frost::Off => false,
            Frost::Auto => settings
                .get("frost.suits")
                .is_some_and(|v| v.eq_ignore_ascii_case("true")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn a_theme_text_is_its_keys_under_their_sections() {
        let found = keys("# a theme\n[Theme]\nname = Midnight\n\n[colours]\naccent = #FF8800\n ; note\ntint=artwork\n[glass]\nbar = 0.6\nnot a pair\n = lost\n");
        let pairs: Vec<(&str, &str)> = found
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            pairs,
            [
                ("colours.accent", "#FF8800"),
                ("colours.tint", "artwork"),
                ("glass.bar", "0.6"),
                ("theme.name", "Midnight")
            ]
        );
        assert_eq!(
            keys("glass.bar = 0.5\n")
                .get("glass.bar")
                .map(String::as_str),
            Some("0.5"),
            "a key written whole stands"
        );
    }

    #[test]
    fn colours_are_read_long_and_short_and_nothing_else() {
        assert_eq!(colour("#ff8800"), Some([255, 136, 0]));
        assert_eq!(colour(" #F80 "), Some([255, 136, 0]));
        assert_eq!(colour("ff8800"), None);
        assert_eq!(colour("#ff88"), None);
        assert_eq!(colour("#gg8800"), None);
    }

    #[test]
    fn the_defaults_the_theme_and_the_users_settings_lie_one_over_the_other() {
        let text = "[theme]\nname = Midnight\n[colours]\ntint = #101820\naccent = artwork\n[glass]\nbar = 0.6\nfrost = off\n[clock]\nink = #ffcc00\n";
        let theme = Theme::resolve(
            Some(text),
            &settings(&[
                ("glass.bar", "0.9"),
                ("colours.accent", "#00ff00"),
                ("size", "car"),
                ("theme", "Midnight"),
            ]),
        );
        assert_eq!(theme.name, "Midnight");
        assert_eq!(theme.tint, Paint::Fixed([16, 24, 32]), "the theme's");
        assert_eq!(
            theme.accent,
            Paint::Fixed([0, 255, 0]),
            "the user's over the theme's"
        );
        assert_eq!(theme.bar, 0.9, "the user's over the theme's");
        assert_eq!(theme.frost, Frost::Off);
        assert_eq!(theme.clock_ink, Some([255, 204, 0]));
        assert_eq!(
            theme.sheet,
            Theme::default().sheet,
            "what nobody set keeps its default"
        );
        assert_eq!(Theme::resolve(None, &settings(&[])), Theme::default());
    }

    #[test]
    fn what_cannot_be_read_changes_nothing_and_an_opacity_has_a_floor() {
        let mut theme = Theme::default();
        theme.apply(&settings(&[
            ("glass.bar", "0"),
            ("glass.sheet", "nonsense"),
            ("colours.ink", "red"),
            ("buttons.opacity", "7"),
            ("measure.bar", "9000"),
            ("a.key.of.a.later.face", "1"),
            ("glass.frost", "maybe"),
        ]));
        assert_eq!(theme.bar, 0.1, "never so low that a control cannot be seen");
        assert_eq!(theme.sheet, Theme::default().sheet);
        assert_eq!(theme.ink, Theme::default().ink);
        assert_eq!(theme.buttons_opacity, 1.0);
        assert_eq!(theme.measure_bar, 240.0);
        assert_eq!(theme.frost, Frost::Auto);
        theme.apply(&settings(&[("buttons.ink", "#112233")]));
        assert_eq!(theme.buttons_ink, Some([17, 34, 51]));
        theme.apply(&settings(&[("buttons.ink", "ink")]));
        assert_eq!(theme.buttons_ink, None, "back to the theme's ink");
    }

    #[test]
    fn the_example_theme_is_the_built_in_look_written_out() {
        let text = include_str!("../../../themes/Example/face.txt");
        let written = keys(text);
        let mut theme = Theme::default();
        theme.apply(&written);
        let built_in = Theme {
            name: "Example".to_string(),
            ..Theme::default()
        };
        assert_eq!(
            theme, built_in,
            "every value in the example is the default it documents"
        );
        for key in [
            "theme.name",
            "colours.tint",
            "colours.accent",
            "colours.ink",
            "colours.plate",
            "glass.bar",
            "glass.sheet",
            "glass.hairline",
            "glass.frost",
            "buttons.ink",
            "buttons.opacity",
            "clock.ink",
            "clock.opacity",
            "clock.plate",
            "measure.bar",
            "measure.clock",
        ] {
            assert!(written.contains_key(key), "the example documents {key}");
        }
        assert_eq!(written.len(), 16, "and nothing the face does not read");
    }

    #[test]
    fn frost_is_the_users_word_or_the_boards() {
        let mut theme = Theme::default();
        assert!(
            !theme.frosted(&settings(&[])),
            "auto with no word from the plugin is off"
        );
        assert!(theme.frosted(&settings(&[("frost.suits", "true")])));
        assert!(!theme.frosted(&settings(&[("frost.suits", "false")])));
        theme.frost = Frost::On;
        assert!(
            theme.frosted(&settings(&[("frost.suits", "false")])),
            "the user decides either way"
        );
        theme.frost = Frost::Off;
        assert!(!theme.frosted(&settings(&[("frost.suits", "true")])));
    }
}
