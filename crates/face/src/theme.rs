//! A face theme: the tokens every piece of the face is drawn from. A theme
//! is a text its author writes, sections of `key = value`, in a folder of
//! its own; what it leaves out keeps its default, and what the user sets in
//! the Manager goes over it, key for key. Nothing here draws: the face
//! reads a `Theme` and the look the artwork gives it.

use crate::clock::{ClockKind, DialStyle, Paints};
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

/// Where the date stands: at the top of the screen on a glass of its own,
/// or with the clock, above it or below it, on the clock's glass.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DatePlace {
    Top,
    Above,
    Below,
}

/// A block of the idle screen's grid, three rows by three columns in equal
/// thirds above the bar: the rows and the columns an element occupies,
/// each from one to another of the three (0, 1, 2), both ends included.
/// One cell, a row, four cells, all nine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cells {
    pub rows: (u8, u8),
    pub columns: (u8, u8),
}

/// Where an element stands across what it occupies.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Across {
    Left,
    #[default]
    Centre,
    Right,
}

/// Where an element stands down what it occupies.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Down {
    Top,
    #[default]
    Middle,
    Bottom,
}

/// What the forecast shows: today as a line, the next 24 hours every so
/// many as columns, or the week as seven columns.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Span {
    #[default]
    Today,
    Hours(u8),
    Week,
}

/// A span from its word: `today`, `hours2`, `hours3`, `hours4`, `hours6`, `week`.
pub fn span(text: &str) -> Option<Span> {
    match text.trim().to_ascii_lowercase().as_str() {
        "today" => Some(Span::Today),
        "hours2" => Some(Span::Hours(2)),
        "hours3" => Some(Span::Hours(3)),
        "hours4" => Some(Span::Hours(4)),
        "hours6" => Some(Span::Hours(6)),
        "week" => Some(Span::Week),
        _ => None,
    }
}

/// Where an element stands inside the cells it occupies: in their middle
/// unless said.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Align {
    pub across: Across,
    pub down: Down,
}

/// The cells a place names: `<rows> <columns>`, each one of the grid's
/// names (`top`, `middle`, `bottom`; `left`, `centre`, `right`) or a range
/// of them (`middle left-centre`, `top-bottom right`). Nothing for anything
/// else, an empty value among it.
pub fn cells(text: &str) -> Option<Cells> {
    let mut words = text.split_whitespace();
    let (rows, columns) = (words.next()?, words.next()?);
    if words.next().is_some() {
        return None;
    }
    let range = |word: &str, names: [&str; 3]| -> Option<(u8, u8)> {
        let index = |name: &str| {
            let name = name.to_ascii_lowercase();
            let name = if name == "center" {
                "centre"
            } else {
                name.as_str()
            };
            names.iter().position(|n| *n == name).map(|i| i as u8)
        };
        match word.split_once('-') {
            Some((from, to)) => {
                let (from, to) = (index(from)?, index(to)?);
                Some((from.min(to), from.max(to)))
            }
            None => index(word).map(|i| (i, i)),
        }
    };
    Some(Cells {
        rows: range(rows, ["top", "middle", "bottom"])?,
        columns: range(columns, ["left", "centre", "right"])?,
    })
}

/// An alignment as written: `left`, `centre` or `right` and `top`,
/// `middle` or `bottom`, in either order; a side not named is the middle.
/// Nothing where a word is none of these.
pub fn align(text: &str) -> Option<Align> {
    let mut align = Align::default();
    for word in text.split_whitespace() {
        match word.to_ascii_lowercase().as_str() {
            "left" => align.across = Across::Left,
            "centre" | "center" => align.across = Across::Centre,
            "right" => align.across = Across::Right,
            "top" => align.down = Down::Top,
            "middle" => align.down = Down::Middle,
            "bottom" => align.down = Down::Bottom,
            _ => return None,
        }
    }
    Some(align)
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
    /// `clock.show`: whether there is a clock when nothing plays.
    pub clock_show: bool,
    /// `clock.place`: the cells of the grid the clock occupies. None is
    /// the arrangement before the grid: the middle of what the date and
    /// the bar leave.
    pub clock_cells: Option<Cells>,
    /// `clock.align`: where the clock stands inside the cells it occupies.
    pub clock_align: Align,
    /// `clock.margin`: units of a 720th of the picture's height between the
    /// clock's glass and the sides of its cells it is aligned to; 0 puts it
    /// in the corner.
    pub clock_margin: f32,
    /// `clock.format`: the time as a pattern, `%H:%M` for 13:05,
    /// `%-I:%M %p` for 1:05 PM, `%H:%M:%S` with the seconds.
    pub clock_format: String,
    /// `clock.ink`, `clock.opacity`: the clock's own.
    pub clock_ink: Option<[u8; 3]>,
    pub clock_opacity: f32,
    /// `clock.glass`: how much of the glass behind the clock and the date
    /// shows, 0 for no glass at all.
    pub clock_glass: f32,
    /// `clock.tint`: the colour of the clock's glass when it has its own;
    /// `tint` for the theme's.
    pub clock_tint: Option<[u8; 3]>,
    /// `clock.face`: the clock set in `type`, or drawn: `seven` or
    /// `sixteen` segments, a `flip` clock, or a `dial` with hands.
    pub clock_face: ClockKind,
    /// `clock.dial`: a dial's style: `station`, `numbers`, `roman`, `plain`.
    pub clock_dial: DialStyle,
    /// `clock.unlit`: how much of an unlit segment shows, 0 to 0.5.
    pub clock_unlit: f32,
    /// `clock.hands`, `clock.marks`: a dial's hands, and its marks and
    /// numerals; `ink` for the clock's ink, but that a station dial has
    /// dark ones of its own.
    pub clock_hands: Option<[u8; 3]>,
    pub clock_marks: Option<[u8; 3]>,
    /// `clock.second`: the second hand: `accent` for the look's (a station
    /// dial has a red one of its own), or a colour.
    pub clock_second: Option<[u8; 3]>,
    /// `clock.disc`: the disc behind a dial's hands: `style` for the
    /// style's own (a station dial has a light one, the others none),
    /// `none`, or a colour.
    pub clock_disc: Disc,
    /// `clock.card`: a flip clock's cards.
    pub clock_card: [u8; 3],
    /// `date.show`: whether the date stands with the clock.
    pub date_show: bool,
    /// `weather.show`: whether the forecast shows, where the player holds
    /// one for a place the user chose.
    pub weather_show: bool,
    /// `weather.place`: the cells of the grid the forecast occupies; the
    /// forecast stands on the grid and only, so none hides it.
    pub weather_cells: Option<Cells>,
    /// `weather.align`: where the forecast stands inside its cells.
    pub weather_align: Align,
    /// `weather.margin`: as `clock.margin`, for the forecast.
    pub weather_margin: f32,
    /// `weather.ink`, `weather.opacity`: the forecast's own, as the date's
    /// are the date's; the ink the theme's unless said.
    pub weather_ink: Option<[u8; 3]>,
    pub weather_opacity: f32,
    /// `weather.glass`, `weather.tint`: the glass behind the forecast and
    /// its colour, the theme's tint unless said.
    pub weather_glass: f32,
    pub weather_tint: Option<[u8; 3]>,
    /// `weather.span`: what the forecast shows, today unless said.
    pub weather_span: Span,
    /// `weather.heat`: the temperatures in the colour of their degree, from
    /// `weather.cold` at -10 °C through the forecast's ink at 12 to
    /// `weather.warm` at 30; off unless said. `weather.heat.days` colours
    /// the week's lows and highs too; `weather.heat.date` sets the date in
    /// the colour of the temperature now: the one link between pieces, by
    /// the user's switch.
    pub weather_heat: bool,
    pub weather_cold: [u8; 3],
    pub weather_warm: [u8; 3],
    pub weather_heat_days: bool,
    pub weather_heat_date: bool,
    /// `weather.motion`: the skies move, each its own small motion at its
    /// own pace; on unless said. Off, every sky stands still and nothing
    /// about the screen's refresh changes. `weather.thunder`: thunder's
    /// flashes, the one motion that can startle; off unless said.
    pub weather_motion: bool,
    pub weather_thunder: bool,
    /// `weather.colour`: the skies in colour, a yellow sun, a dark storm
    /// with a yellow bolt, blues for rain, greys for cloud and fog; on
    /// unless said. Off, every sky is in the forecast's ink.
    pub weather_colour: bool,
    /// `date.format`: the date as a pattern, `%A %-d %B` for Thursday 1
    /// October, `%d/%m/%Y` for 01/10/2026.
    pub date_format: String,
    /// `date.place`: `top` of the screen, or `above` or `below` the clock.
    pub date_place: DatePlace,
    /// `date.place` as cells of the grid: where the date occupies cells
    /// instead of standing by its three words. None is the arrangement
    /// before the grid.
    pub date_cells: Option<Cells>,
    /// `date.align`: where the date stands inside the cells it occupies.
    pub date_align: Align,
    /// `date.margin`: as `clock.margin`, for the date.
    pub date_margin: f32,
    /// `date.ink`, `date.opacity`: the date's own.
    pub date_ink: Option<[u8; 3]>,
    pub date_opacity: f32,
    /// `date.glass`: how much of the glass behind a date at the top of the
    /// screen shows, 0 for none; with the clock, the date is on the clock's.
    pub date_glass: f32,
    /// `date.tint`: the colour of that glass when it has its own.
    pub date_tint: Option<[u8; 3]>,
    /// `measure.bar`, `measure.clock`: heights in units of a 720th of the
    /// picture's height.
    pub measure_bar: f32,
    pub measure_clock: f32,
    /// `measure.date`: the date's height, in the same units.
    pub measure_date: f32,
    /// `measure.weather`: the forecast's height, in the same units.
    pub measure_weather: f32,
    /// `idle.picture`: a picture of the user's own shown behind the clock
    /// and the date when nothing plays, in the theme's place, by its file's
    /// name in the folder such pictures are kept in; empty for none.
    pub idle_picture: String,
    /// `idle.dim`: how far that picture is darkened, 0 to 0.9.
    pub idle_dim: f32,
    /// `idle.off`: minutes with nothing playing and no touch after which
    /// the screen goes black; 0 for never.
    pub idle_off_min: u32,
    /// `idle.fade`: milliseconds the screen takes to go black and to come
    /// back, up to two minutes; 0 for at once.
    pub idle_fade_ms: u32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: String::new(),
            tint: Paint::Artwork,
            accent: Paint::Artwork,
            ink: [242, 242, 245],
            bar: 0.75,
            sheet: 0.78,
            hairline: 0.12,
            frost: Frost::Auto,
            buttons_ink: None,
            buttons_opacity: 1.0,
            clock_show: true,
            clock_cells: None,
            clock_align: Align::default(),
            clock_margin: 20.0,
            clock_format: "%H:%M".to_string(),
            clock_ink: None,
            clock_opacity: 0.86,
            clock_glass: 0.55,
            clock_tint: None,
            clock_face: ClockKind::Type,
            clock_dial: DialStyle::Station,
            clock_unlit: 0.08,
            clock_hands: None,
            clock_marks: None,
            clock_second: None,
            clock_disc: Disc::Style,
            clock_card: [23, 23, 26],
            date_show: false,
            weather_show: true,
            weather_cells: cells("bottom left-right"),
            weather_align: Align::default(),
            weather_margin: 20.0,
            weather_ink: None,
            weather_opacity: 0.86,
            weather_glass: 0.55,
            weather_tint: None,
            weather_span: Span::Today,
            weather_heat: false,
            weather_cold: [59, 139, 255],
            weather_warm: [255, 75, 43],
            weather_heat_days: false,
            weather_heat_date: false,
            weather_motion: true,
            weather_thunder: false,
            weather_colour: true,
            date_format: "%A %-d %B".to_string(),
            date_place: DatePlace::Top,
            date_cells: None,
            date_align: Align::default(),
            date_margin: 20.0,
            date_ink: None,
            date_opacity: 0.86,
            date_glass: 0.55,
            date_tint: None,
            measure_bar: 72.0,
            measure_clock: 144.0,
            measure_date: 40.0,
            measure_weather: 40.0,
            idle_picture: String::new(),
            idle_dim: 0.25,
            idle_off_min: 0,
            idle_fade_ms: 500,
        }
    }
}

/// The disc behind a dial's hands: the style's own, none, or a colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Disc {
    Style,
    None,
    Fixed([u8; 3]),
}

/// A pattern for a time or a date, as `strftime` reads it: short, and of
/// printing characters only.
pub fn pattern(text: &str) -> Option<String> {
    let text = text.trim();
    let fits = !text.is_empty() && text.len() <= 48 && text.chars().all(|c| !c.is_control());
    fits.then(|| text.to_string())
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

/// `on` or `off`, said any of the usual ways.
pub fn switch(text: &str) -> Option<bool> {
    match text.trim().to_ascii_lowercase().as_str() {
        "on" | "true" | "yes" | "1" => Some(true),
        "off" | "false" | "no" | "0" => Some(false),
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
                "idle.picture" => self.idle_picture = v.trim().to_string(),
                "idle.dim" => self.idle_dim = units(v, 0.0, 0.9).unwrap_or(self.idle_dim),
                "idle.fade" => {
                    self.idle_fade_ms = v
                        .trim()
                        .parse::<u32>()
                        .map(|ms| ms.min(120_000))
                        .unwrap_or(self.idle_fade_ms)
                }
                "idle.off" => {
                    self.idle_off_min = v
                        .trim()
                        .parse::<u32>()
                        .map(|m| m.min(1440))
                        .unwrap_or(self.idle_off_min)
                }
                "colours.tint" | "colors.tint" => self.tint = paint(v).unwrap_or(self.tint),
                "colours.accent" | "colors.accent" => self.accent = paint(v).unwrap_or(self.accent),
                "colours.ink" | "colors.ink" => self.ink = colour(v).unwrap_or(self.ink),
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
                "clock.glass" | "clock.plate" => {
                    self.clock_glass = v
                        .parse::<f32>()
                        .ok()
                        .filter(|n| n.is_finite())
                        .map(|n| n.clamp(0.0, 1.0))
                        .unwrap_or(self.clock_glass)
                }
                "clock.tint" => {
                    self.clock_tint = if v.eq_ignore_ascii_case("tint") {
                        None
                    } else {
                        colour(v).or(self.clock_tint)
                    }
                }
                "date.tint" => {
                    self.date_tint = if v.eq_ignore_ascii_case("tint") {
                        None
                    } else {
                        colour(v).or(self.date_tint)
                    }
                }
                "clock.face" => self.clock_face = ClockKind::parse(v).unwrap_or(self.clock_face),
                "clock.dial" => self.clock_dial = DialStyle::parse(v).unwrap_or(self.clock_dial),
                "clock.unlit" => {
                    self.clock_unlit = v
                        .parse::<f32>()
                        .ok()
                        .filter(|n| n.is_finite())
                        .map(|n| n.clamp(0.0, 0.5))
                        .unwrap_or(self.clock_unlit)
                }
                "clock.hands" => {
                    self.clock_hands = if v.eq_ignore_ascii_case("ink") {
                        None
                    } else {
                        colour(v).or(self.clock_hands)
                    }
                }
                "clock.marks" => {
                    self.clock_marks = if v.eq_ignore_ascii_case("ink") {
                        None
                    } else {
                        colour(v).or(self.clock_marks)
                    }
                }
                "clock.second" => {
                    self.clock_second = if v.eq_ignore_ascii_case("accent") {
                        None
                    } else {
                        colour(v).or(self.clock_second)
                    }
                }
                "clock.disc" => {
                    self.clock_disc = match v.to_ascii_lowercase().as_str() {
                        "style" => Disc::Style,
                        "none" | "off" => Disc::None,
                        _ => colour(v).map(Disc::Fixed).unwrap_or(self.clock_disc),
                    }
                }
                "clock.card" => self.clock_card = colour(v).unwrap_or(self.clock_card),
                "clock.show" => self.clock_show = switch(v).unwrap_or(self.clock_show),
                // Empty, or anything that names no cells: as before the grid.
                "clock.place" => self.clock_cells = cells(v),
                "clock.align" => self.clock_align = align(v).unwrap_or(self.clock_align),
                "clock.margin" => {
                    self.clock_margin = units(v, 0.0, 200.0).unwrap_or(self.clock_margin)
                }
                "clock.format" => {
                    if let Some(format) = pattern(v) {
                        self.clock_format = format;
                    }
                }
                "date.show" => self.date_show = switch(v).unwrap_or(self.date_show),
                "weather.show" => self.weather_show = switch(v).unwrap_or(self.weather_show),
                "weather.span" => self.weather_span = span(v).unwrap_or(self.weather_span),
                "weather.heat" => self.weather_heat = switch(v).unwrap_or(self.weather_heat),
                "weather.cold" => self.weather_cold = colour(v).unwrap_or(self.weather_cold),
                "weather.warm" => self.weather_warm = colour(v).unwrap_or(self.weather_warm),
                "weather.heat.days" => {
                    self.weather_heat_days = switch(v).unwrap_or(self.weather_heat_days)
                }
                "weather.heat.date" => {
                    self.weather_heat_date = switch(v).unwrap_or(self.weather_heat_date)
                }
                "weather.motion" => self.weather_motion = switch(v).unwrap_or(self.weather_motion),
                "weather.thunder" => {
                    self.weather_thunder = switch(v).unwrap_or(self.weather_thunder)
                }
                "weather.colour" | "weather.color" => {
                    self.weather_colour = switch(v).unwrap_or(self.weather_colour)
                }
                // Cells of the grid, or nothing: the forecast has no place off it.
                "weather.place" => {
                    self.weather_cells = if v.trim().is_empty() {
                        None
                    } else {
                        cells(v).or(self.weather_cells)
                    }
                }
                "weather.align" => self.weather_align = align(v).unwrap_or(self.weather_align),
                "weather.margin" => {
                    self.weather_margin = units(v, 0.0, 200.0).unwrap_or(self.weather_margin)
                }
                "weather.ink" => {
                    self.weather_ink = if v.eq_ignore_ascii_case("ink") {
                        None
                    } else {
                        colour(v).or(self.weather_ink)
                    }
                }
                "weather.opacity" => {
                    self.weather_opacity = share(v).unwrap_or(self.weather_opacity)
                }
                "weather.glass" => {
                    self.weather_glass = v
                        .parse::<f32>()
                        .ok()
                        .filter(|n| n.is_finite())
                        .map(|n| n.clamp(0.0, 1.0))
                        .unwrap_or(self.weather_glass)
                }
                "weather.tint" => {
                    self.weather_tint = if v.eq_ignore_ascii_case("tint") {
                        None
                    } else {
                        colour(v).or(self.weather_tint)
                    }
                }
                "measure.weather" => {
                    self.measure_weather = units(v, 16.0, 360.0).unwrap_or(self.measure_weather)
                }
                "date.format" => {
                    if let Some(format) = pattern(v) {
                        self.date_format = format;
                    }
                }
                // One of the three words from before the grid, or cells of the grid.
                "date.place" => match v.to_ascii_lowercase().as_str() {
                    "top" => (self.date_place, self.date_cells) = (DatePlace::Top, None),
                    "above" => (self.date_place, self.date_cells) = (DatePlace::Above, None),
                    "below" => (self.date_place, self.date_cells) = (DatePlace::Below, None),
                    _ => {
                        if let Some(on) = cells(v) {
                            self.date_cells = Some(on);
                        }
                    }
                },
                "date.align" => self.date_align = align(v).unwrap_or(self.date_align),
                "date.margin" => {
                    self.date_margin = units(v, 0.0, 200.0).unwrap_or(self.date_margin)
                }
                "date.glass" => {
                    self.date_glass = v
                        .parse::<f32>()
                        .ok()
                        .filter(|n| n.is_finite())
                        .map(|n| n.clamp(0.0, 1.0))
                        .unwrap_or(self.date_glass)
                }
                "date.ink" => {
                    self.date_ink = if v.eq_ignore_ascii_case("ink") {
                        None
                    } else {
                        colour(v).or(self.date_ink)
                    }
                }
                "date.opacity" => self.date_opacity = share(v).unwrap_or(self.date_opacity),
                "measure.date" => {
                    self.measure_date = units(v, 16.0, 360.0).unwrap_or(self.measure_date)
                }
                "measure.bar" => {
                    self.measure_bar = units(v, 48.0, 240.0).unwrap_or(self.measure_bar)
                }
                "measure.clock" => {
                    self.measure_clock = units(v, 48.0, 720.0).unwrap_or(self.measure_clock)
                }
                _ => {}
            }
        }
    }

    /// The tokens in use: the defaults, the theme's text over them, the
    /// user's settings over that.
    pub fn resolve(theme_text: Option<&str>, settings: &BTreeMap<String, String>) -> Self {
        Self::layered(&[theme_text], settings)
    }

    /// The tokens in use where more than one text has a say: the defaults,
    /// each text over what came before it, the user's settings over all.
    pub fn layered(texts: &[Option<&str>], settings: &BTreeMap<String, String>) -> Self {
        let mut theme = Self::default();
        for text in texts.iter().flatten() {
            theme.apply(&keys(text));
        }
        theme.apply(settings);
        theme
    }

    /// The colours a drawn clock is drawn in: each the user's or the
    /// theme's where one is said, else the face's own. A station dial comes
    /// light, with dark hands and a red second hand; the other dials and
    /// the segments come in the clock's ink, the second hand in `accent`,
    /// the colour of what is lit in the look.
    pub fn paints(&self, accent: [u8; 3]) -> Paints {
        let ink = self.clock_ink.unwrap_or(self.ink);
        let station = self.clock_face == ClockKind::Dial && self.clock_dial == DialStyle::Station;
        let dark = [21, 21, 21];
        Paints {
            ink,
            unlit: self.clock_unlit,
            hands: self.clock_hands.unwrap_or(if station { dark } else { ink }),
            marks: self.clock_marks.unwrap_or(if station { dark } else { ink }),
            second: self
                .clock_second
                .unwrap_or(if station { [214, 42, 30] } else { accent }),
            disc: match self.clock_disc {
                Disc::Style => station.then_some([245, 245, 242]),
                Disc::None => None,
                Disc::Fixed(colour) => Some(colour),
            },
            card: self.clock_card,
        }
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
    fn a_place_names_cells_of_the_grid_and_an_alignment_inside_them() {
        let one = |rows, columns| Cells { rows, columns };
        assert_eq!(cells("middle right"), Some(one((1, 1), (2, 2))));
        assert_eq!(
            cells("middle left-right"),
            Some(one((1, 1), (0, 2))),
            "a row"
        );
        assert_eq!(
            cells("Middle-Bottom center-right"),
            Some(one((1, 2), (1, 2))),
            "four cells, in any case and either spelling"
        );
        assert_eq!(
            cells("bottom-top right-left"),
            Some(one((0, 2), (0, 2))),
            "all nine, either way round"
        );
        for none in ["", "top", "middle left right", "upper left", "top-low left"] {
            assert_eq!(cells(none), None, "{none:?} names no cells");
        }
        assert_eq!(align(""), Some(Align::default()));
        assert_eq!(
            align("bottom right"),
            Some(Align {
                across: Across::Right,
                down: Down::Bottom
            }),
            "either order"
        );
        assert_eq!(
            align("left"),
            Some(Align {
                across: Across::Left,
                down: Down::Middle
            })
        );
        assert_eq!(align("leftish"), None);
        let mut theme = Theme::default();
        assert_eq!(theme.clock_cells, None, "off the grid until placed on it");
        theme.apply(&settings(&[
            ("clock.place", "middle-bottom centre-right"),
            ("clock.align", "right bottom"),
        ]));
        assert_eq!(theme.clock_cells, cells("middle-bottom centre-right"));
        assert_eq!(
            theme.clock_align,
            Align {
                across: Across::Right,
                down: Down::Bottom
            }
        );
        // The date: its three old words take it off the grid, cells put it on.
        theme.apply(&settings(&[
            ("date.place", "top right"),
            ("date.align", "right top"),
            ("date.margin", "0"),
        ]));
        assert_eq!(theme.date_cells, cells("top right"));
        assert_eq!(
            theme.date_align,
            Align {
                across: Across::Right,
                down: Down::Top
            }
        );
        assert_eq!(theme.date_margin, 0.0);
        theme.apply(&settings(&[("date.place", "below")]));
        assert_eq!(
            (theme.date_place, theme.date_cells),
            (DatePlace::Below, None)
        );
        theme.apply(&settings(&[("clock.place", ""), ("clock.margin", "0")]));
        assert_eq!(theme.clock_margin, 0.0, "no margin: the corner");
        theme.apply(&settings(&[("clock.margin", "wide")]));
        assert_eq!(theme.clock_margin, 0.0, "a word is no margin");
        assert_eq!(
            theme.clock_cells, None,
            "an empty place takes it off the grid"
        );
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
            "theme.description",
            "colours.tint",
            "colours.accent",
            "colours.ink",
            "glass.bar",
            "glass.sheet",
            "glass.hairline",
            "glass.frost",
            "buttons.ink",
            "buttons.opacity",
            "clock.show",
            "clock.format",
            "clock.ink",
            "clock.opacity",
            "clock.glass",
            "clock.tint",
            "date.show",
            "weather.show",
            "weather.span",
            "weather.heat",
            "weather.cold",
            "weather.warm",
            "weather.heat.days",
            "weather.heat.date",
            "weather.motion",
            "weather.thunder",
            "weather.colour",
            "date.format",
            "date.place",
            "date.align",
            "date.margin",
            "clock.place",
            "clock.align",
            "clock.margin",
            "date.ink",
            "date.opacity",
            "date.glass",
            "date.tint",
            "measure.date",
            "measure.bar",
            "measure.clock",
            "clock.face",
            "clock.dial",
            "clock.unlit",
            "clock.hands",
            "clock.marks",
            "clock.second",
            "clock.disc",
            "clock.card",
        ] {
            assert!(written.contains_key(key), "the example documents {key}");
        }
        assert_eq!(
            written.len(),
            62,
            "and nothing but what the face reads and the line for a list"
        );
    }

    #[test]
    fn the_clock_and_the_date_are_shown_or_not_each_in_its_own_pattern() {
        let mut theme = Theme::default();
        assert!(
            theme.clock_show && !theme.date_show,
            "as designed: a clock, no date"
        );
        assert_eq!(
            (theme.clock_format.as_str(), theme.date_format.as_str()),
            ("%H:%M", "%A %-d %B")
        );
        assert_eq!(
            theme.date_place,
            DatePlace::Top,
            "a date, when wanted, stands at the top of the screen"
        );
        theme.apply(&settings(&[
            ("clock.format", "%-I:%M %p"),
            ("date.show", "on"),
            ("date.format", " %d/%m/%Y "),
            ("date.place", "Above"),
            ("date.ink", "#88aaff"),
            ("date.opacity", "0.5"),
            ("measure.date", "60"),
            ("clock.glass", "0"),
        ]));
        assert_eq!(theme.clock_format, "%-I:%M %p");
        assert!(theme.date_show);
        assert_eq!(theme.date_place, DatePlace::Above);
        assert_eq!(theme.date_format, "%d/%m/%Y");
        assert_eq!(
            (theme.date_ink, theme.date_opacity, theme.measure_date),
            (Some([136, 170, 255]), 0.5, 60.0)
        );
        assert_eq!(
            theme.clock_glass, 0.0,
            "no glass behind the clock is a choice"
        );
        theme.apply(&settings(&[
            ("clock.show", "perhaps"),
            ("clock.format", ""),
            ("date.place", "sideways"),
            ("clock.plate", "0.4"),
        ]));
        assert!(theme.clock_show, "what cannot be read changes nothing");
        assert_eq!(theme.date_place, DatePlace::Above);
        assert_eq!(theme.clock_format, "%-I:%M %p");
        assert_eq!(
            theme.clock_glass, 0.4,
            "the key's earlier name is still read"
        );
        theme.apply(&settings(&[("clock.show", "Off"), ("date.show", "0")]));
        assert!(!theme.clock_show && !theme.date_show);
        // The clock's glass and the date's each take a colour of their own, or the theme's tint.
        assert_eq!((theme.clock_tint, theme.date_tint), (None, None));
        theme.apply(&settings(&[
            ("clock.tint", "#102030"),
            ("date.tint", "#302010"),
        ]));
        assert_eq!(
            (theme.clock_tint, theme.date_tint),
            (Some([16, 32, 48]), Some([48, 32, 16]))
        );
        theme.apply(&settings(&[
            ("clock.tint", "nonsense"),
            ("date.tint", "Tint"),
        ]));
        assert_eq!(
            (theme.clock_tint, theme.date_tint),
            (Some([16, 32, 48]), None),
            "what cannot be read changes nothing; tint is the theme's again"
        );
        assert_eq!(pattern(&"%H".repeat(30)), None, "a pattern is short");
        assert_eq!(pattern("%H\n%M"), None, "and of printing characters");
        assert_eq!(switch(" TRUE "), Some(true));
        assert_eq!(switch(""), None);
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

    #[test]
    fn a_drawn_clock_takes_its_colours_from_the_theme_and_its_face() {
        let settings = |pairs: &[(&str, &str)]| -> BTreeMap<String, String> {
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        };
        let accent = [10, 200, 30];
        // As it comes: the clock in type, a station dial's colours its own.
        let plain = Theme::default();
        assert_eq!(
            (plain.clock_face, plain.clock_dial),
            (ClockKind::Type, DialStyle::Station)
        );
        let station = Theme::resolve(None, &settings(&[("clock.face", "dial")]));
        let paints = station.paints(accent);
        assert_eq!(paints.disc, Some([245, 245, 242]));
        assert_eq!(
            (paints.hands, paints.marks, paints.second),
            ([21, 21, 21], [21, 21, 21], [214, 42, 30])
        );
        // Another dial: the clock's ink, the look's accent, no disc.
        let roman = Theme::resolve(
            None,
            &settings(&[
                ("clock.face", "dial"),
                ("clock.dial", "roman"),
                ("clock.ink", "#ffcc00"),
            ]),
        );
        let paints = roman.paints(accent);
        assert_eq!(
            (paints.hands, paints.marks, paints.second, paints.disc),
            ([255, 204, 0], [255, 204, 0], accent, None)
        );
        // The user's own word over all of it, and back to the face's own.
        let own = Theme::resolve(
            None,
            &settings(&[
                ("clock.face", "dial"),
                ("clock.hands", "#112233"),
                ("clock.marks", "#445566"),
                ("clock.second", "#778899"),
                ("clock.disc", "#000000"),
                ("clock.unlit", "0.9"),
                ("clock.card", "#202020"),
            ]),
        );
        let paints = own.paints(accent);
        assert_eq!(
            (paints.hands, paints.marks, paints.second),
            ([0x11, 0x22, 0x33], [0x44, 0x55, 0x66], [0x77, 0x88, 0x99])
        );
        assert_eq!(
            (paints.disc, paints.unlit, paints.card),
            (Some([0, 0, 0]), 0.5, [0x20, 0x20, 0x20])
        );
        let mut back = own.clone();
        back.apply(&settings(&[
            ("clock.hands", "ink"),
            ("clock.second", "accent"),
            ("clock.disc", "none"),
        ]));
        let paints = back.paints(accent);
        assert_eq!(
            (paints.hands, paints.second, paints.disc),
            ([21, 21, 21], [214, 42, 30], None)
        );
        // A word that is none changes nothing.
        let mut odd = station.clone();
        odd.apply(&settings(&[
            ("clock.face", "sundial"),
            ("clock.dial", "cuckoo"),
            ("clock.disc", "sky"),
        ]));
        assert_eq!(
            (odd.clock_face, odd.clock_dial, odd.clock_disc),
            (ClockKind::Dial, DialStyle::Station, Disc::Style)
        );
    }
}
