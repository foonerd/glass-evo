//! The face: what glass-evo draws over Glass's display and does with the
//! touches the theme's controls do not take. Playing shows the theme and
//! nothing else; a touch brings the bar of controls (previous, play or
//! pause, next, volume down and up, more), which leaves by itself; stopped
//! or paused shows the clock with the bar. More opens a sheet above the
//! bar with repeat, random and mute; a long press on volume down mutes.
//! Everything else the display does, the theme, the meters, the artwork,
//! the never-empty screen, it does as before.

use glass::face::{ui, Command, Frame, Metadata, PointerKind, TextStyle};
use glass::{Overlay, View};

/// The workspace version, as Cargo knows it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The line the binary prints to say what it is.
pub fn banner() -> String {
    format!("glass-evo {VERSION}")
}

/// The bar's buttons, left to right.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    Previous,
    Toggle,
    Next,
    VolumeDown,
    VolumeUp,
    More,
}

const BUTTONS: [Button; 6] = [
    Button::Previous,
    Button::Toggle,
    Button::Next,
    Button::VolumeDown,
    Button::VolumeUp,
    Button::More,
];

/// The sheet's tiles, left to right.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Repeat,
    Random,
    Mute,
}

const TILES: [Tile; 3] = [Tile::Repeat, Tile::Random, Tile::Mute];

/// How far a volume button moves the volume, in points of a hundred.
pub const VOLUME_STEP: u32 = 5;

/// How long the bar takes to fade in or out.
pub const FADE_MS: u64 = 200;
/// How long the bar stays after the last touch while playing: long enough
/// for a hand reaching out, in a car too.
pub const LINGER_MS: u64 = 6000;
/// How long after playback begins the bar leaves.
pub const AFTER_PLAY_MS: u64 = 2000;
/// How long a finger rests on volume down before it mutes.
pub const HOLD_MS: u64 = 600;

/// The bar's place on a picture: the foot, a tenth of the height and at
/// least forty pixels, six buttons of equal width across it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bar {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Bar {
    /// `scale` is the face size the display hands over: 1 as designed,
    /// more for a hand at arm's length; the bar never takes more than
    /// a third of the picture.
    pub fn for_picture(width: u32, height: u32, scale: f32) -> Self {
        let h = ((height as f32 / 10.0 * scale.max(0.5)).round() as u32)
            .max(40)
            .min(height / 3);
        Self {
            x: 0,
            y: height.saturating_sub(h) as i32,
            w: width,
            h,
        }
    }

    fn slot_width(&self) -> u32 {
        self.w / BUTTONS.len() as u32
    }

    /// The button under a point, if the point is on the bar.
    pub fn button_at(&self, x: i32, y: i32) -> Option<Button> {
        if self.w == 0
            || y < self.y
            || y >= self.y + self.h as i32
            || x < self.x
            || x >= self.x + self.w as i32
        {
            return None;
        }
        let slot = ((x - self.x) as u32 * BUTTONS.len() as u32 / self.w) as usize;
        Some(BUTTONS[slot.min(BUTTONS.len() - 1)])
    }

    /// A button's rectangle: x, y, width, height.
    pub fn button_rect(&self, index: usize) -> (i32, i32, u32, u32) {
        let bw = self.slot_width();
        (self.x + (index as u32 * bw) as i32, self.y, bw, self.h)
    }

    /// The sheet More opens: three tiles the width of a button each,
    /// standing on the bar at its right end.
    pub fn sheet(&self) -> Sheet {
        let tile = self.slot_width();
        let w = tile * TILES.len() as u32;
        Sheet {
            x: self.x + self.w as i32 - w as i32,
            y: self.y - self.h as i32,
            w,
            h: self.h,
        }
    }
}

/// The sheet's place: above the bar's right end.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sheet {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Sheet {
    /// The tile under a point, if the point is on the sheet.
    pub fn tile_at(&self, x: i32, y: i32) -> Option<Tile> {
        if self.w == 0
            || y < self.y
            || y >= self.y + self.h as i32
            || x < self.x
            || x >= self.x + self.w as i32
        {
            return None;
        }
        let slot = ((x - self.x) as u32 * TILES.len() as u32 / self.w) as usize;
        Some(TILES[slot.min(TILES.len() - 1)])
    }

    /// A tile's rectangle: x, y, width, height.
    pub fn tile_rect(&self, index: usize) -> (i32, i32, u32, u32) {
        let tw = self.w / TILES.len() as u32;
        (self.x + (index as u32 * tw) as i32, self.y, tw, self.h)
    }
}

fn plain(name: &str) -> Command {
    Command {
        name: name.to_string(),
        value: None,
    }
}

/// The command a button sends, given the volume as the player has it;
/// none for More, which only opens the sheet.
pub fn command_for(button: Button, volume: u32) -> Option<Command> {
    match button {
        Button::Previous => Some(plain("previous")),
        Button::Toggle => Some(plain("toggle")),
        Button::Next => Some(plain("next")),
        Button::VolumeDown => Some(Command::with(
            "volume",
            serde_json::json!(volume.saturating_sub(VOLUME_STEP)),
        )),
        Button::VolumeUp => Some(Command::with(
            "volume",
            serde_json::json!((volume + VOLUME_STEP).min(100)),
        )),
        Button::More => None,
    }
}

/// Mute or unmute, as the player stands.
pub fn mute_command(muted: bool) -> Command {
    Command::with(
        "volume",
        serde_json::json!(if muted { "unmute" } else { "mute" }),
    )
}

/// The command a tile sends, given the player's state: repeat walks off,
/// all, single and off again; random and mute turn over.
pub fn tile_command(tile: Tile, meta: &Metadata) -> Command {
    match tile {
        Tile::Repeat => {
            let next = match (meta.repeat, meta.repeat_single) {
                (false, _) => "all",
                (true, false) => "single",
                (true, true) => "off",
            };
            Command::with("repeat", serde_json::json!(next))
        }
        Tile::Random => Command::with("random", serde_json::json!(!meta.random)),
        Tile::Mute => mute_command(meta.mute),
    }
}

/// Whether a tile is lit: its mode is on in the player.
pub fn tile_lit(tile: Tile, meta: &Metadata) -> bool {
    match tile {
        Tile::Repeat => meta.repeat,
        Tile::Random => meta.random,
        Tile::Mute => meta.mute,
    }
}

/// Whether the bar is on the glass, and how much of it: playing, it is
/// hidden until a touch and leaves by itself; not playing, it stays.
/// Times are the display's clock in milliseconds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Presence {
    visible: bool,
    /// When the last fade began; none when the bar is settled.
    changed_at: Option<u64>,
    leave_at: Option<u64>,
    playing: bool,
}

impl Presence {
    /// As the face starts: the bar there, settled, until playback says
    /// otherwise.
    pub fn new() -> Self {
        Self {
            visible: true,
            changed_at: None,
            leave_at: None,
            playing: false,
        }
    }

    fn set(&mut self, visible: bool, now: u64) {
        if self.visible != visible {
            // A fade in the other direction starts where this one stands.
            let done = match self.changed_at {
                Some(at) => now.saturating_sub(at).min(FADE_MS),
                None => FADE_MS,
            };
            self.changed_at = Some(now.saturating_sub(FADE_MS - done));
            self.visible = visible;
        }
    }

    /// Every frame: playback beginning sends the bar away after a moment;
    /// playback ending brings it back; a linger that ran out sends it away.
    pub fn tick(&mut self, now: u64, playing: bool) {
        if playing != self.playing {
            self.playing = playing;
            if playing {
                self.leave_at = Some(now + AFTER_PLAY_MS);
            } else {
                self.leave_at = None;
                self.set(true, now);
            }
        }
        if self.playing {
            if let Some(at) = self.leave_at {
                if now >= at {
                    self.leave_at = None;
                    self.set(false, now);
                }
            }
        }
    }

    /// A touch on the bar keeps it a while longer.
    pub fn kept(&mut self, now: u64) {
        if self.playing {
            self.leave_at = Some(now + LINGER_MS);
        }
    }

    /// A touch on the picture away from the bar: brings the bar when it is
    /// away, sends it away when it is there and the player plays.
    pub fn touched(&mut self, now: u64) {
        if !self.playing {
            return;
        }
        if self.visible {
            self.leave_at = None;
            self.set(false, now);
        } else {
            self.set(true, now);
            self.leave_at = Some(now + LINGER_MS);
        }
    }

    /// Whether the bar takes touches now.
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// How much of the bar shows, 0 to 255, through its fade.
    pub fn alpha(&self, now: u64) -> u8 {
        let since = match self.changed_at {
            Some(at) => now.saturating_sub(at).min(FADE_MS),
            None => FADE_MS,
        };
        let up = (since * 255 / FADE_MS) as u8;
        if self.visible {
            up
        } else {
            255 - up
        }
    }
}

impl Default for Presence {
    fn default() -> Self {
        Self::new()
    }
}

/// The face's state between frames: the bar's presence, whether the sheet
/// is open, the commands not yet taken, what a finger is down on and
/// since when, and whether that press already acted as a long one.
#[derive(Default)]
pub struct Face {
    presence: Presence,
    sheet_open: bool,
    pending: Vec<Command>,
    pressed: Option<Button>,
    pressed_tile: Option<Tile>,
    down_at: u64,
    held: bool,
}

impl Face {
    pub fn new() -> Self {
        Self::default()
    }

    /// The commands queued so far, for a test to look at.
    pub fn pending(&self) -> &[Command] {
        &self.pending
    }

    /// The bar's presence, for a test to look at.
    pub fn presence(&self) -> &Presence {
        &self.presence
    }

    /// Whether the sheet is open, for a test to look at.
    pub fn sheet_open(&self) -> bool {
        self.sheet_open
    }
}

const INK: [u8; 3] = [235, 235, 240];

/// Whether the face says what it sees: the display's own log level, as the
/// launcher hands it over, at its finest.
fn verbose() -> bool {
    std::env::var("GLASS_LOG")
        .map(|v| v.contains("verbose"))
        .unwrap_or(false)
}

fn scaled(a: u8, by: u8) -> u8 {
    (a as u32 * by as u32 / 255) as u8
}

/// A filled triangle pointing right (or left), drawn row by row.
fn triangle(frame: &mut Frame, x: i32, y: i32, w: u32, h: u32, right: bool, rgba: [u8; 4]) {
    if w == 0 || h == 0 {
        return;
    }
    let half = h as f32 / 2.0;
    for row in 0..h {
        let d = (row as f32 + 0.5 - half).abs() / half; // 0 at the middle, 1 at the ends
        let span = ((1.0 - d) * w as f32).round() as u32;
        if span == 0 {
            continue;
        }
        let sx = if right { x } else { x + (w - span) as i32 };
        ui::fill(frame, sx, y + row as i32, span, 1, rgba);
    }
}

/// A straight stroke between two points, drawn as squares along it.
fn stroke(frame: &mut Frame, from: (i32, i32), to: (i32, i32), thick: u32, rgba: [u8; 4]) {
    let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs()).max(1);
    let half = thick as i32 / 2;
    for i in 0..=steps {
        let x = from.0 + (to.0 - from.0) * i / steps;
        let y = from.1 + (to.1 - from.1) * i / steps;
        ui::fill(frame, x - half, y - half, thick, thick, rgba);
    }
}

/// The glyph's box inside a rectangle: its size, its centre, its stroke.
fn glyph_box(rect: (i32, i32, u32, u32)) -> (i32, i32, i32, u32) {
    let (x, y, w, h) = rect;
    let s = (h * 2 / 5).max(8);
    (x + w as i32 / 2, y + h as i32 / 2, s as i32, (s / 5).max(2))
}

/// A button's glyph, drawn with fills: shapes, not fonts, so any theme's
/// fonts do. Volume down wears a slash while the player is muted.
fn glyph(
    frame: &mut Frame,
    button: Button,
    playing: bool,
    muted: bool,
    rect: (i32, i32, u32, u32),
    alpha: u8,
) {
    let (cx, cy, s, bar) = glyph_box(rect);
    let ink = [INK[0], INK[1], INK[2], alpha];
    let (left, top) = (cx - s / 2, cy - s / 2);
    match button {
        Button::Previous => {
            ui::fill(frame, left, top, bar, s as u32, ink);
            triangle(
                frame,
                left + bar as i32 + 1,
                top,
                s as u32 - bar - 1,
                s as u32,
                false,
                ink,
            );
        }
        Button::Toggle => {
            if playing {
                let gap = bar as i32;
                ui::fill(frame, cx - bar as i32 - gap / 2, top, bar, s as u32, ink);
                ui::fill(frame, cx + gap / 2, top, bar, s as u32, ink);
            } else {
                triangle(frame, left + 2, top, s as u32, s as u32, true, ink);
            }
        }
        Button::Next => {
            triangle(frame, left, top, s as u32 - bar - 1, s as u32, true, ink);
            ui::fill(frame, cx + s / 2 - bar as i32, top, bar, s as u32, ink);
        }
        Button::VolumeDown => {
            ui::fill(frame, left, cy - bar as i32 / 2, s as u32, bar, ink);
            if muted {
                stroke(frame, (left, top + s), (left + s, top), bar, ink);
            }
        }
        Button::VolumeUp => {
            ui::fill(frame, left, cy - bar as i32 / 2, s as u32, bar, ink);
            ui::fill(frame, cx - bar as i32 / 2, top, bar, s as u32, ink);
        }
        Button::More => {
            let dot = (bar * 3 / 2).max(3);
            for i in -1..=1 {
                ui::fill(
                    frame,
                    cx + i * (s / 2 - dot as i32 / 2) - dot as i32 / 2,
                    cy - dot as i32 / 2,
                    dot,
                    dot,
                    ink,
                );
            }
        }
    }
}

/// A tile's glyph: a loop for repeat (with a stem inside for single), two
/// crossing strokes for random, a speaker for mute, struck through when
/// muted.
fn tile_glyph(
    frame: &mut Frame,
    tile: Tile,
    meta: &Metadata,
    rect: (i32, i32, u32, u32),
    alpha: u8,
) {
    let (cx, cy, s, bar) = glyph_box(rect);
    let ink = [INK[0], INK[1], INK[2], alpha];
    let (left, top) = (cx - s / 2, cy - s / 2);
    match tile {
        Tile::Repeat => {
            ui::fill(frame, left, top, s as u32, bar, ink);
            ui::fill(frame, left, top + s - bar as i32, s as u32, bar, ink);
            ui::fill(frame, left, top, bar, s as u32, ink);
            ui::fill(frame, left + s - bar as i32, top, bar, s as u32, ink);
            if meta.repeat && meta.repeat_single {
                ui::fill(
                    frame,
                    cx - bar as i32 / 2,
                    top + bar as i32 * 2,
                    bar,
                    (s - bar as i32 * 4).max(2) as u32,
                    ink,
                );
            }
        }
        Tile::Random => {
            stroke(frame, (left, top), (left + s, top + s), bar, ink);
            stroke(frame, (left, top + s), (left + s, top), bar, ink);
        }
        Tile::Mute => {
            let body = (s / 3).max(3);
            ui::fill(frame, left, cy - body / 2, body as u32, body as u32, ink);
            triangle(
                frame,
                left + body / 2,
                top,
                (s - body) as u32,
                s as u32,
                false,
                ink,
            );
            if meta.mute {
                stroke(frame, (left, top + s), (left + s, top), bar, ink);
            }
        }
    }
}

/// The wall clock, hours and minutes, in the player's own zone.
fn clock_text() -> String {
    // SAFETY: localtime_r writes only into the tm handed to it and reads
    // the clock; both live on this stack for the call.
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&now, &mut tm).is_null() {
            return String::new();
        }
        format!("{:02}:{:02}", tm.tm_hour, tm.tm_min)
    }
}

impl Overlay for Face {
    fn draw(&mut self, frame: &mut Frame, view: &View) -> bool {
        let meta = &view.input.metadata;
        let now = view.now_ms;
        let playing = meta.status == "play";
        self.presence.tick(now, playing);
        // A finger resting on volume down: mute, once, and no step on its lift.
        if self.pressed == Some(Button::VolumeDown)
            && !self.held
            && now.saturating_sub(self.down_at) >= HOLD_MS
        {
            self.held = true;
            self.pending.push(mute_command(meta.mute));
            self.presence.kept(now);
        }
        let alpha = self.presence.alpha(now);
        if alpha == 0 {
            // The bar is away, and the sheet with it.
            self.sheet_open = false;
        }
        let bar = Bar::for_picture(view.width, view.height, view.scale);
        let mut drawn = false;
        if alpha > 0 {
            // The bar: frosted dark over the picture, a hairline above it.
            ui::fill(
                frame,
                bar.x,
                bar.y,
                bar.w,
                bar.h,
                [10, 10, 12, scaled(175, alpha)],
            );
            ui::fill(
                frame,
                bar.x,
                bar.y,
                bar.w,
                1,
                [255, 255, 255, scaled(36, alpha)],
            );
            for (i, button) in BUTTONS.iter().enumerate() {
                let rect = bar.button_rect(i);
                let lit =
                    self.pressed == Some(*button) || (*button == Button::More && self.sheet_open);
                if lit {
                    ui::fill(
                        frame,
                        rect.0,
                        rect.1,
                        rect.2,
                        rect.3,
                        [255, 255, 255, scaled(40, alpha)],
                    );
                }
                glyph(frame, *button, playing, meta.mute, rect, alpha);
            }
            if self.sheet_open {
                let sheet = bar.sheet();
                ui::fill(
                    frame,
                    sheet.x,
                    sheet.y,
                    sheet.w,
                    sheet.h,
                    [10, 10, 12, scaled(200, alpha)],
                );
                ui::fill(
                    frame,
                    sheet.x,
                    sheet.y,
                    sheet.w,
                    1,
                    [255, 255, 255, scaled(36, alpha)],
                );
                for (i, tile) in TILES.iter().enumerate() {
                    let rect = sheet.tile_rect(i);
                    let lit = tile_lit(*tile, meta);
                    if lit || self.pressed_tile == Some(*tile) {
                        ui::fill(
                            frame,
                            rect.0,
                            rect.1,
                            rect.2,
                            rect.3,
                            [255, 255, 255, scaled(46, alpha)],
                        );
                    }
                    tile_glyph(
                        frame,
                        *tile,
                        meta,
                        rect,
                        if lit { alpha } else { scaled(150, alpha) },
                    );
                }
            }
            drawn = true;
        }
        // The clock: when the player stands still on the display's own screen.
        if view.ours && !playing {
            let size = ((view.height as f32 / 5.0 * view.scale.max(0.5)).round() as u32)
                .max(24)
                .min(view.height / 2);
            if let Some(line) = ui::line(view.fonts, TextStyle::Bold, size, INK, &clock_text()) {
                let x = (view.width.saturating_sub(line.width) / 2) as i32;
                let y = (view
                    .height
                    .saturating_sub(bar.h)
                    .saturating_sub(line.height)
                    / 2) as i32;
                ui::fill(
                    frame,
                    x - 12,
                    y - 6,
                    line.width + 24,
                    line.height + 12,
                    [10, 10, 12, 140],
                );
                ui::blit(frame, &line, x, y, 220);
                drawn = true;
            }
        }
        drawn
    }

    fn pointer(&mut self, kind: PointerKind, x: i32, y: i32, view: &View) -> bool {
        let now = view.now_ms;
        let meta = &view.input.metadata;
        if verbose() {
            let kind_name = match kind {
                PointerKind::Down => "down",
                PointerKind::Move => "move",
                PointerKind::Up => "up",
            };
            println!(
                "glass: face: {kind_name} at {x},{y}: bar {} sheet {} alpha {} playing {} status {}",
                if self.presence.visible() { "there" } else { "away" },
                if self.sheet_open { "open" } else { "shut" },
                self.presence.alpha(now),
                self.presence.playing,
                meta.status
            );
        }
        let bar = Bar::for_picture(view.width, view.height, view.scale);
        let on_bar = if self.presence.visible() {
            bar.button_at(x, y)
        } else {
            None
        };
        let on_sheet = if self.presence.visible() && self.sheet_open {
            bar.sheet().tile_at(x, y)
        } else {
            None
        };
        match kind {
            PointerKind::Down => {
                self.pressed = on_bar;
                self.pressed_tile = on_sheet;
                self.down_at = now;
                self.held = false;
                if on_bar.is_some() || on_sheet.is_some() {
                    self.presence.kept(now);
                    return true;
                }
                false
            }
            PointerKind::Move => {
                on_bar.is_some()
                    || on_sheet.is_some()
                    || self.pressed.is_some()
                    || self.pressed_tile.is_some()
            }
            PointerKind::Up => {
                let was_tile = self.pressed_tile.take();
                let was = self.pressed.take();
                if let Some(tile) = was_tile {
                    if on_sheet == Some(tile) {
                        self.pending.push(tile_command(tile, meta));
                    }
                    self.presence.kept(now);
                    return true;
                }
                if let Some(button) = was {
                    if on_bar == Some(button) {
                        if button == Button::More {
                            self.sheet_open = !self.sheet_open;
                        } else if !(button == Button::VolumeDown && self.held) {
                            if let Some(command) = command_for(button, meta.volume) {
                                self.pending.push(command);
                            }
                        }
                    }
                    self.held = false;
                    self.presence.kept(now);
                    return true;
                }
                if on_bar.is_some() || on_sheet.is_some() {
                    return true;
                }
                // A tap on the picture: the sheet shuts if it is open, else
                // the bar comes or goes; the theme sees the tap too.
                if self.sheet_open {
                    self.sheet_open = false;
                    self.presence.kept(now);
                } else {
                    self.presence.touched(now);
                }
                false
            }
        }
    }

    fn commands(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glass::face::{Fonts, Input};

    fn input(status: &str) -> Input {
        Input {
            metadata: Metadata {
                status: status.to_string(),
                volume: 50,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn view<'a>(input: &'a Input, fonts: &'a Fonts, now_ms: u64) -> View<'a> {
        View {
            input,
            fonts,
            width: 1280,
            height: 720,
            now_ms,
            ours: true,
            scale: 1.0,
        }
    }

    fn blank() -> Frame {
        Frame {
            blend: Default::default(),
            width: 1280,
            height: 720,
            rgba: vec![0; 1280 * 720 * 4],
        }
    }

    #[test]
    fn the_banner_names_the_face_and_its_version() {
        assert_eq!(banner(), format!("glass-evo {}", env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn the_bar_sits_at_the_foot_and_names_the_button_under_a_point() {
        let bar = Bar::for_picture(1280, 720, 1.0);
        assert_eq!(
            bar,
            Bar {
                x: 0,
                y: 648,
                w: 1280,
                h: 72
            }
        );
        assert_eq!(
            Bar::for_picture(1280, 720, 2.0).h,
            144,
            "car: twice the bar"
        );
        assert_eq!(
            Bar::for_picture(1280, 720, 9.0).h,
            240,
            "never more than a third"
        );
        assert_eq!(bar.button_at(10, 700), Some(Button::Previous));
        assert_eq!(bar.button_at(384, 684), Some(Button::Toggle));
        assert_eq!(bar.button_at(500, 700), Some(Button::Next));
        assert_eq!(bar.button_at(700, 700), Some(Button::VolumeDown));
        assert_eq!(bar.button_at(900, 700), Some(Button::VolumeUp));
        assert_eq!(bar.button_at(1279, 700), Some(Button::More));
        assert_eq!(
            bar.button_at(640, 600),
            None,
            "above the bar is the theme's"
        );
        assert_eq!(bar.button_at(-1, 700), None);
        assert_eq!(
            Bar::for_picture(320, 240, 1.0).h,
            40,
            "never thinner than forty pixels"
        );
        assert_eq!(bar.button_rect(2), (426, 648, 213, 72));
    }

    #[test]
    fn the_sheet_stands_on_the_bar_at_its_right_end() {
        let bar = Bar::for_picture(1280, 720, 1.0);
        let sheet = bar.sheet();
        assert_eq!(
            sheet,
            Sheet {
                x: 641,
                y: 576,
                w: 639,
                h: 72
            }
        );
        assert_eq!(sheet.tile_at(650, 600), Some(Tile::Repeat));
        assert_eq!(sheet.tile_at(960, 600), Some(Tile::Random));
        assert_eq!(sheet.tile_at(1270, 600), Some(Tile::Mute));
        assert_eq!(
            sheet.tile_at(600, 600),
            None,
            "left of the sheet is the theme's"
        );
        assert_eq!(sheet.tile_at(960, 660), None, "below it is the bar");
        assert_eq!(sheet.tile_rect(1), (854, 576, 213, 72));
    }

    #[test]
    fn the_buttons_and_tiles_send_the_players_own_commands() {
        let pair = |c: Command| (c.name, c.value);
        assert_eq!(
            pair(command_for(Button::Toggle, 30).unwrap()),
            ("toggle".to_string(), None)
        );
        assert_eq!(
            pair(command_for(Button::VolumeDown, 3).unwrap()),
            ("volume".to_string(), Some(serde_json::json!(0))),
            "never below zero"
        );
        assert_eq!(
            pair(command_for(Button::VolumeUp, 98).unwrap()),
            ("volume".to_string(), Some(serde_json::json!(100))),
            "never above a hundred"
        );
        assert_eq!(command_for(Button::Previous, 50).unwrap().name, "previous");
        assert_eq!(command_for(Button::Next, 50).unwrap().name, "next");
        assert!(
            command_for(Button::More, 50).is_none(),
            "More only opens the sheet"
        );
        let mut meta = Metadata::default();
        assert_eq!(
            pair(tile_command(Tile::Repeat, &meta)),
            ("repeat".to_string(), Some(serde_json::json!("all")))
        );
        meta.repeat = true;
        assert_eq!(
            pair(tile_command(Tile::Repeat, &meta)),
            ("repeat".to_string(), Some(serde_json::json!("single")))
        );
        meta.repeat_single = true;
        assert_eq!(
            pair(tile_command(Tile::Repeat, &meta)),
            ("repeat".to_string(), Some(serde_json::json!("off")))
        );
        assert_eq!(
            pair(tile_command(Tile::Random, &meta)),
            ("random".to_string(), Some(serde_json::json!(true)))
        );
        assert_eq!(
            pair(tile_command(Tile::Mute, &meta)),
            ("volume".to_string(), Some(serde_json::json!("mute")))
        );
        meta.mute = true;
        assert_eq!(
            pair(tile_command(Tile::Mute, &meta)),
            ("volume".to_string(), Some(serde_json::json!("unmute")))
        );
        assert!(
            tile_lit(Tile::Repeat, &meta)
                && tile_lit(Tile::Mute, &meta)
                && !tile_lit(Tile::Random, &meta)
        );
    }

    #[test]
    fn the_bar_leaves_when_playback_begins_and_comes_back_for_a_touch_while_playing() {
        let mut p = Presence::new();
        p.tick(1000, false);
        assert!(p.visible(), "stopped: the bar stays");
        assert_eq!(p.alpha(1000), 255);
        p.tick(2000, true);
        assert!(p.visible(), "playback began: the bar has a moment yet");
        p.tick(2000 + AFTER_PLAY_MS, true);
        assert!(!p.visible(), "then it leaves");
        assert_eq!(p.alpha(2000 + AFTER_PLAY_MS + FADE_MS), 0, "faded out");
        p.touched(10_000);
        assert!(p.visible(), "a touch on the picture brings it");
        assert_eq!(p.alpha(10_000 + FADE_MS / 2), 127, "half way in");
        p.tick(10_000 + LINGER_MS - 1, true);
        assert!(p.visible(), "it lingers");
        p.kept(10_000 + LINGER_MS - 1);
        p.tick(10_000 + LINGER_MS + 1, true);
        assert!(p.visible(), "a touch on the bar keeps it longer");
        p.tick(10_000 + 2 * LINGER_MS + 1, true);
        assert!(!p.visible(), "and it leaves again");
        p.touched(30_000);
        p.touched(30_500);
        assert!(!p.visible(), "a second tap on the picture sends it away");
        p.tick(40_000, false);
        assert!(p.visible(), "playback ended: the bar is back and stays");
        p.touched(41_000);
        assert!(p.visible(), "a tap while stopped changes nothing");
    }

    #[test]
    fn a_fade_reversed_half_way_starts_where_it_stands() {
        let mut p = Presence::new();
        p.tick(0, true);
        p.tick(AFTER_PLAY_MS, true); // leaves at AFTER_PLAY_MS
        let mid = AFTER_PLAY_MS + FADE_MS / 2;
        assert_eq!(p.alpha(mid), 128, "half way out");
        p.touched(mid);
        assert_eq!(p.alpha(mid), 127, "back in, from where it stood");
        assert_eq!(p.alpha(mid + FADE_MS / 2), 255);
    }

    /// A player's sequence, frame by frame: playing, the bar away after
    /// two seconds; a tap on the picture passes through and brings the
    /// bar; a tap on the bar within the linger acts.
    #[test]
    fn a_tap_brings_the_bar_and_the_next_tap_on_it_acts() {
        let input = input("play");
        let fonts = Fonts::default();
        let mut frame = blank();
        let mut face = Face::new();
        assert!(
            face.draw(&mut frame, &view(&input, &fonts, 0)),
            "the bar shows as the face starts"
        );
        assert!(
            face.draw(&mut frame, &view(&input, &fonts, 2000)),
            "at two seconds the bar begins to leave"
        );
        assert!(
            !face.draw(&mut frame, &view(&input, &fonts, 8000)),
            "eight seconds into playback nothing is drawn"
        );
        assert!(
            !face.pointer(PointerKind::Down, 384, 684, &view(&input, &fonts, 8000)),
            "a touch on the picture passes through"
        );
        assert!(!face.pointer(PointerKind::Up, 384, 684, &view(&input, &fonts, 8050)));
        assert!(face.presence().visible(), "and brings the bar");
        assert!(face.commands().is_empty(), "nothing acted");
        assert!(
            face.draw(&mut frame, &view(&input, &fonts, 9500)),
            "the bar is drawn"
        );
        assert!(
            face.pointer(PointerKind::Down, 384, 684, &view(&input, &fonts, 9500)),
            "a touch on the bar is the bar's"
        );
        assert!(face.pointer(PointerKind::Up, 384, 684, &view(&input, &fonts, 9550)));
        let sent = face.commands();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].name, "toggle");
        face.draw(&mut frame, &view(&input, &fonts, 9600));
        assert!(face.presence().visible(), "kept by the touch");
    }

    #[test]
    fn more_opens_the_sheet_a_tile_acts_and_a_tap_outside_shuts_it() {
        let input = input("pause");
        let fonts = Fonts::default();
        let mut frame = blank();
        let mut face = Face::new();
        face.draw(&mut frame, &view(&input, &fonts, 0));
        // A tile's place does nothing while the sheet is shut: it is the theme's.
        assert!(!face.pointer(PointerKind::Down, 960, 600, &view(&input, &fonts, 100)));
        assert!(!face.pointer(PointerKind::Up, 960, 600, &view(&input, &fonts, 150)));
        assert!(face.commands().is_empty());
        // More opens it.
        assert!(face.pointer(PointerKind::Down, 1200, 700, &view(&input, &fonts, 1000)));
        assert!(face.pointer(PointerKind::Up, 1200, 700, &view(&input, &fonts, 1050)));
        assert!(face.sheet_open());
        assert!(face.commands().is_empty(), "opening sends nothing");
        // Random, pressed and released in place, turns over.
        assert!(face.pointer(PointerKind::Down, 960, 600, &view(&input, &fonts, 2000)));
        assert!(face.pointer(PointerKind::Up, 960, 600, &view(&input, &fonts, 2050)));
        let sent = face.commands();
        assert_eq!(sent.len(), 1);
        assert_eq!(
            (sent[0].name.as_str(), sent[0].value.clone()),
            ("random", Some(serde_json::json!(true)))
        );
        assert!(face.sheet_open(), "the sheet stays for the next tile");
        // A press that slides off its tile sends nothing.
        assert!(face.pointer(PointerKind::Down, 1270, 600, &view(&input, &fonts, 3000)));
        assert!(face.pointer(PointerKind::Up, 700, 600, &view(&input, &fonts, 3050)));
        assert!(face.commands().is_empty());
        // A tap on the picture shuts the sheet and passes through.
        assert!(!face.pointer(PointerKind::Down, 300, 300, &view(&input, &fonts, 4000)));
        assert!(!face.pointer(PointerKind::Up, 300, 300, &view(&input, &fonts, 4050)));
        assert!(!face.sheet_open());
        assert!(face.presence().visible(), "the bar stays");
    }

    #[test]
    fn a_finger_resting_on_volume_down_mutes_once_and_steps_nothing() {
        let input = input("pause");
        let fonts = Fonts::default();
        let mut frame = blank();
        let mut face = Face::new();
        face.draw(&mut frame, &view(&input, &fonts, 0));
        assert!(face.pointer(PointerKind::Down, 700, 700, &view(&input, &fonts, 1000)));
        face.draw(&mut frame, &view(&input, &fonts, 1000 + HOLD_MS - 1));
        assert!(face.pending().is_empty(), "not yet");
        face.draw(&mut frame, &view(&input, &fonts, 1000 + HOLD_MS));
        face.draw(&mut frame, &view(&input, &fonts, 1000 + HOLD_MS + 400));
        assert!(face.pointer(PointerKind::Up, 700, 700, &view(&input, &fonts, 2100)));
        let sent = face.commands();
        assert_eq!(sent.len(), 1, "muted once, and no step on the lift");
        assert_eq!(
            (sent[0].name.as_str(), sent[0].value.clone()),
            ("volume", Some(serde_json::json!("mute")))
        );
        // A short press steps as before.
        assert!(face.pointer(PointerKind::Down, 700, 700, &view(&input, &fonts, 3000)));
        assert!(face.pointer(PointerKind::Up, 700, 700, &view(&input, &fonts, 3100)));
        let sent = face.commands();
        assert_eq!(
            (sent[0].name.as_str(), sent[0].value.clone()),
            ("volume", Some(serde_json::json!(45)))
        );
    }

    #[test]
    fn the_glyphs_and_the_clock_draw_inside_the_frame() {
        let mut frame = Frame {
            blend: Default::default(),
            width: 320,
            height: 240,
            rgba: vec![0; 320 * 240 * 4],
        };
        let bar = Bar::for_picture(320, 240, 1.0);
        let meta = Metadata {
            repeat: true,
            repeat_single: true,
            mute: true,
            ..Default::default()
        };
        for (i, b) in BUTTONS.iter().enumerate() {
            glyph(&mut frame, *b, i % 2 == 0, true, bar.button_rect(i), 255);
        }
        let sheet = bar.sheet();
        for (i, t) in TILES.iter().enumerate() {
            tile_glyph(&mut frame, *t, &meta, sheet.tile_rect(i), 255);
        }
        assert!(frame.rgba.iter().any(|&v| v != 0), "something was drawn");
        let text = clock_text();
        assert!(
            text.is_empty() || text.len() == 5,
            "HH:MM or nothing: {text}"
        );
    }
}
