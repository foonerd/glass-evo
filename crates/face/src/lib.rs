//! The face: what glass-evo draws over Glass's display and does with the
//! touches the theme's controls do not take. Playing shows the theme and
//! nothing else; a touch brings the bar of controls (previous, play or
//! pause, next, volume down and up), which leaves by itself; stopped or
//! paused shows the clock with the bar. Everything else the display does,
//! the theme, the meters, the artwork, the never-empty screen, it does as
//! before.

use glass::face::{ui, Command, Frame, PointerKind, TextStyle};
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
}

const BUTTONS: [Button; 5] = [
    Button::Previous,
    Button::Toggle,
    Button::Next,
    Button::VolumeDown,
    Button::VolumeUp,
];

/// How far a volume button moves the volume, in points of a hundred.
pub const VOLUME_STEP: u32 = 5;

/// How long the bar takes to fade in or out.
pub const FADE_MS: u64 = 200;
/// How long the bar stays after the last touch while playing: long enough
/// for a hand reaching out, in a car too.
pub const LINGER_MS: u64 = 6000;
/// How long after playback begins the bar leaves.
pub const AFTER_PLAY_MS: u64 = 2000;

/// The bar's place on a picture: the foot, a tenth of the height and at
/// least forty pixels, five buttons of equal width across it.
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
        let slot = ((x - self.x) as u32 * 5 / self.w) as usize;
        Some(BUTTONS[slot.min(4)])
    }

    /// A button's rectangle: x, y, width, height.
    pub fn button_rect(&self, index: usize) -> (i32, i32, u32, u32) {
        let bw = self.w / 5;
        (self.x + (index as u32 * bw) as i32, self.y, bw, self.h)
    }
}

/// The command a button sends, given the volume as the player has it.
pub fn command_for(button: Button, volume: u32) -> Command {
    let plain = |name: &str| Command {
        name: name.to_string(),
        value: None,
    };
    match button {
        Button::Previous => plain("previous"),
        Button::Toggle => plain("toggle"),
        Button::Next => plain("next"),
        Button::VolumeDown => Command::with(
            "volume",
            serde_json::json!(volume.saturating_sub(VOLUME_STEP)),
        ),
        Button::VolumeUp => {
            Command::with("volume", serde_json::json!((volume + VOLUME_STEP).min(100)))
        }
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

/// The face's state between frames: the bar's presence, the commands not
/// yet taken, and the button a finger is down on.
#[derive(Default)]
pub struct Face {
    presence: Presence,
    pending: Vec<Command>,
    pressed: Option<Button>,
}

impl Face {
    pub fn new() -> Self {
        Self::default()
    }

    /// A button pressed and released in place: its command is queued.
    pub fn act(&mut self, button: Button, volume: u32) {
        self.pending.push(command_for(button, volume));
    }

    /// The commands queued so far, for a test to look at.
    pub fn pending(&self) -> &[Command] {
        &self.pending
    }

    /// The bar's presence, for a test to look at.
    pub fn presence(&self) -> &Presence {
        &self.presence
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

/// A button's glyph, drawn with fills: shapes, not fonts, so any theme's
/// fonts do.
fn glyph(frame: &mut Frame, button: Button, playing: bool, rect: (i32, i32, u32, u32), alpha: u8) {
    let (x, y, w, h) = rect;
    let s = (h * 2 / 5).max(8); // the glyph's size
    let cx = x + w as i32 / 2;
    let cy = y + h as i32 / 2;
    let bar = (s / 5).max(2);
    let ink = [INK[0], INK[1], INK[2], alpha];
    match button {
        Button::Previous => {
            ui::fill(frame, cx - s as i32 / 2, cy - s as i32 / 2, bar, s, ink);
            triangle(
                frame,
                cx - s as i32 / 2 + bar as i32 + 1,
                cy - s as i32 / 2,
                s - bar - 1,
                s,
                false,
                ink,
            );
        }
        Button::Toggle => {
            if playing {
                let gap = bar;
                ui::fill(
                    frame,
                    cx - (bar + gap / 2) as i32,
                    cy - s as i32 / 2,
                    bar,
                    s,
                    ink,
                );
                ui::fill(frame, cx + (gap / 2) as i32, cy - s as i32 / 2, bar, s, ink);
            } else {
                triangle(
                    frame,
                    cx - s as i32 / 2 + 2,
                    cy - s as i32 / 2,
                    s,
                    s,
                    true,
                    ink,
                );
            }
        }
        Button::Next => {
            triangle(
                frame,
                cx - s as i32 / 2,
                cy - s as i32 / 2,
                s - bar - 1,
                s,
                true,
                ink,
            );
            ui::fill(
                frame,
                cx + s as i32 / 2 - bar as i32,
                cy - s as i32 / 2,
                bar,
                s,
                ink,
            );
        }
        Button::VolumeDown => {
            ui::fill(frame, cx - s as i32 / 2, cy - bar as i32 / 2, s, bar, ink);
        }
        Button::VolumeUp => {
            ui::fill(frame, cx - s as i32 / 2, cy - bar as i32 / 2, s, bar, ink);
            ui::fill(frame, cx - bar as i32 / 2, cy - s as i32 / 2, bar, s, ink);
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
        let playing = view.input.metadata.status == "play";
        self.presence.tick(view.now_ms, playing);
        let alpha = self.presence.alpha(view.now_ms);
        let mut drawn = false;
        if alpha > 0 {
            let bar = Bar::for_picture(view.width, view.height, view.scale);
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
                if self.pressed == Some(*button) {
                    ui::fill(
                        frame,
                        rect.0,
                        rect.1,
                        rect.2,
                        rect.3,
                        [255, 255, 255, scaled(40, alpha)],
                    );
                }
                glyph(frame, *button, playing, rect, alpha);
            }
            drawn = true;
        }
        // The clock: when the player stands still on the display's own screen.
        if view.ours && !playing {
            let bar = Bar::for_picture(view.width, view.height, view.scale);
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
        if verbose() {
            let kind_name = match kind {
                PointerKind::Down => "down",
                PointerKind::Move => "move",
                PointerKind::Up => "up",
            };
            println!(
                "glass: face: {kind_name} at {x},{y}: bar {} alpha {} playing {} status {}",
                if self.presence.visible() {
                    "there"
                } else {
                    "away"
                },
                self.presence.alpha(now),
                self.presence.playing,
                view.input.metadata.status
            );
        }
        let bar = Bar::for_picture(view.width, view.height, view.scale);
        let hit = if self.presence.visible() {
            bar.button_at(x, y)
        } else {
            None
        };
        match kind {
            PointerKind::Down => {
                self.pressed = hit;
                if hit.is_some() {
                    self.presence.kept(now);
                }
                hit.is_some()
            }
            PointerKind::Move => hit.is_some() || self.pressed.is_some(),
            PointerKind::Up => {
                let was = self.pressed.take();
                if let (Some(pressed), Some(under)) = (was, hit) {
                    if pressed == under {
                        self.act(pressed, view.input.metadata.volume);
                        self.presence.kept(now);
                    }
                }
                if was.is_some() || hit.is_some() {
                    return true;
                }
                // A tap on the picture: the bar comes or goes; the theme sees the tap too.
                self.presence.touched(now);
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

    #[test]
    fn the_banner_names_the_face_and_its_version() {
        assert_eq!(banner(), format!("glass-evo {}", env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn the_bar_sits_at_the_foot_and_names_the_button_under_a_point() {
        let bar = Bar::for_picture(1280, 720, 1.0);
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
        assert_eq!(
            bar,
            Bar {
                x: 0,
                y: 648,
                w: 1280,
                h: 72
            }
        );
        assert_eq!(bar.button_at(10, 700), Some(Button::Previous));
        assert_eq!(bar.button_at(640, 700), Some(Button::Next));
        assert_eq!(bar.button_at(1279, 700), Some(Button::VolumeUp));
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
        assert_eq!(bar.button_rect(2), (512, 648, 256, 72));
    }

    #[test]
    fn a_button_pressed_and_released_in_place_queues_its_command_once() {
        let mut face = Face::new();
        face.act(Button::Toggle, 30);
        face.act(Button::VolumeDown, 3);
        face.act(Button::VolumeUp, 98);
        let names: Vec<(String, Option<serde_json::Value>)> = face
            .pending()
            .iter()
            .map(|c| (c.name.clone(), c.value.clone()))
            .collect();
        assert_eq!(names[0], ("toggle".to_string(), None));
        assert_eq!(
            names[1],
            ("volume".to_string(), Some(serde_json::json!(0))),
            "never below zero"
        );
        assert_eq!(
            names[2],
            ("volume".to_string(), Some(serde_json::json!(100))),
            "never above a hundred"
        );
        assert_eq!(command_for(Button::Previous, 50).name, "previous");
        assert_eq!(command_for(Button::Next, 50).name, "next");
        let taken = face.commands();
        assert_eq!(taken.len(), 3);
        assert!(face.pending().is_empty(), "taken once");
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

    /// A player's sequence, frame by frame: playing, the bar away after
    /// two seconds; a tap on the picture passes through and brings the
    /// bar; a tap on the bar within the linger acts.
    #[test]
    fn a_tap_brings_the_bar_and_the_next_tap_on_it_acts() {
        use glass::face::{Fonts, Input, Metadata};
        let input = Input {
            metadata: Metadata {
                status: "play".to_string(),
                volume: 50,
                ..Default::default()
            },
            ..Default::default()
        };
        let fonts = Fonts::default();
        let view = |now_ms: u64| View {
            input: &input,
            fonts: &fonts,
            width: 1280,
            height: 720,
            now_ms,
            ours: true,
            scale: 1.0,
        };
        let mut frame = Frame {
            blend: Default::default(),
            width: 1280,
            height: 720,
            rgba: vec![0; 1280 * 720 * 4],
        };
        let mut face = Face::new();
        assert!(
            face.draw(&mut frame, &view(0)),
            "the bar shows as the face starts"
        );
        assert!(
            face.draw(&mut frame, &view(2000)),
            "at two seconds the bar begins to leave"
        );
        assert!(
            face.draw(&mut frame, &view(2100)),
            "half way out it is still drawn"
        );
        assert!(
            !face.draw(&mut frame, &view(8000)),
            "eight seconds into playback nothing is drawn"
        );
        assert!(!face.presence().visible());
        assert!(
            !face.pointer(PointerKind::Down, 384, 684, &view(8000)),
            "a touch on the picture passes through"
        );
        assert!(!face.pointer(PointerKind::Up, 384, 684, &view(8050)));
        assert!(face.presence().visible(), "and brings the bar");
        assert!(face.commands().is_empty(), "nothing acted");
        assert!(face.draw(&mut frame, &view(9500)), "the bar is drawn");
        assert!(
            face.pointer(PointerKind::Down, 384, 684, &view(9500)),
            "a touch on the bar is the bar's"
        );
        assert!(face.pointer(PointerKind::Up, 384, 684, &view(9550)));
        let sent = face.commands();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].name, "toggle");
        face.draw(&mut frame, &view(9600));
        assert!(face.presence().visible(), "kept by the touch");
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

    #[test]
    fn the_glyphs_and_the_clock_draw_inside_the_frame() {
        let mut frame = Frame {
            blend: Default::default(),
            width: 320,
            height: 240,
            rgba: vec![0; 320 * 240 * 4],
        };
        let bar = Bar::for_picture(320, 240, 1.0);
        for (i, b) in BUTTONS.iter().enumerate() {
            glyph(&mut frame, *b, i % 2 == 0, bar.button_rect(i), 255);
        }
        assert!(frame.rgba.iter().any(|&v| v != 0), "something was drawn");
        let text = clock_text();
        assert!(
            text.is_empty() || text.len() == 5,
            "HH:MM or nothing: {text}"
        );
    }
}
