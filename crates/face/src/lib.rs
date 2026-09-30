//! The face: what glass-evo draws over Glass's display and does with the
//! touches the theme's controls do not take. This is v0: a bar of controls
//! at the foot of the picture (previous, play or pause, next, volume down
//! and up) and a clock when the player stands still on a screen that is
//! the display's own. Everything else the display does, the theme, the
//! meters, the artwork, the never-empty screen, it does as before.

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
    pub fn for_picture(width: u32, height: u32) -> Self {
        let h = (height / 10).max(40).min(height);
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

/// The face's state between frames: the commands not yet taken, and the
/// button a finger is down on.
#[derive(Default)]
pub struct Face {
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
}

const INK: [u8; 4] = [235, 235, 240, 255];
const INK_DIM: [u8; 4] = [235, 235, 240, 110];

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
fn glyph(frame: &mut Frame, button: Button, playing: bool, rect: (i32, i32, u32, u32)) {
    let (x, y, w, h) = rect;
    let s = (h * 2 / 5).max(8); // the glyph's size
    let cx = x + w as i32 / 2;
    let cy = y + h as i32 / 2;
    let bar = (s / 5).max(2);
    match button {
        Button::Previous => {
            ui::fill(frame, cx - s as i32 / 2, cy - s as i32 / 2, bar, s, INK);
            triangle(
                frame,
                cx - s as i32 / 2 + bar as i32 + 1,
                cy - s as i32 / 2,
                s - bar - 1,
                s,
                false,
                INK,
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
                    INK,
                );
                ui::fill(frame, cx + (gap / 2) as i32, cy - s as i32 / 2, bar, s, INK);
            } else {
                triangle(
                    frame,
                    cx - s as i32 / 2 + 2,
                    cy - s as i32 / 2,
                    s,
                    s,
                    true,
                    INK,
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
                INK,
            );
            ui::fill(
                frame,
                cx + s as i32 / 2 - bar as i32,
                cy - s as i32 / 2,
                bar,
                s,
                INK,
            );
        }
        Button::VolumeDown => {
            ui::fill(frame, cx - s as i32 / 2, cy - bar as i32 / 2, s, bar, INK);
        }
        Button::VolumeUp => {
            ui::fill(frame, cx - s as i32 / 2, cy - bar as i32 / 2, s, bar, INK);
            ui::fill(frame, cx - bar as i32 / 2, cy - s as i32 / 2, bar, s, INK);
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
        let bar = Bar::for_picture(view.width, view.height);
        let status = view.input.metadata.status.as_str();
        let playing = status == "play";
        // The bar: frosted dark over the picture, a hairline above it.
        ui::fill(frame, bar.x, bar.y, bar.w, bar.h, [10, 10, 12, 175]);
        ui::fill(frame, bar.x, bar.y, bar.w, 1, [255, 255, 255, 36]);
        for (i, button) in BUTTONS.iter().enumerate() {
            let rect = bar.button_rect(i);
            if self.pressed == Some(*button) {
                ui::fill(frame, rect.0, rect.1, rect.2, rect.3, [255, 255, 255, 40]);
            }
            glyph(frame, *button, playing, rect);
        }
        // The clock: when the player stands still on the display's own screen.
        if view.ours && !playing {
            let size = (view.height / 5).max(24);
            if let Some(line) = ui::line(
                view.fonts,
                TextStyle::Bold,
                size,
                [INK[0], INK[1], INK[2]],
                &clock_text(),
            ) {
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
                ui::blit(frame, &line, x, y, INK_DIM[3].max(200));
            }
        }
        true
    }

    fn pointer(&mut self, kind: PointerKind, x: i32, y: i32, view: &View) -> bool {
        let bar = Bar::for_picture(view.width, view.height);
        let hit = bar.button_at(x, y);
        match kind {
            PointerKind::Down => {
                self.pressed = hit;
                hit.is_some()
            }
            PointerKind::Move => hit.is_some() || self.pressed.is_some(),
            PointerKind::Up => {
                let was = self.pressed.take();
                if let (Some(pressed), Some(under)) = (was, hit) {
                    if pressed == under {
                        self.act(pressed, view.input.metadata.volume);
                    }
                }
                was.is_some() || hit.is_some()
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
        let bar = Bar::for_picture(1280, 720);
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
            Bar::for_picture(320, 240).h,
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
    fn the_glyphs_and_the_clock_draw_inside_the_frame() {
        let mut frame = Frame {
            blend: Default::default(),
            width: 320,
            height: 240,
            rgba: vec![0; 320 * 240 * 4],
        };
        let bar = Bar::for_picture(320, 240);
        for (i, b) in BUTTONS.iter().enumerate() {
            glyph(&mut frame, *b, i % 2 == 0, bar.button_rect(i));
        }
        assert!(frame.rgba.iter().any(|&v| v != 0), "something was drawn");
        let text = clock_text();
        assert!(
            text.is_empty() || text.len() == 5,
            "HH:MM or nothing: {text}"
        );
    }
}
