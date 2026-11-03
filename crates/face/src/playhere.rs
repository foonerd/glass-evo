//! Play here from the More sheet: the player's sound on the device that
//! draws the face, with the device's own volume on a page over the theme,
//! laid out as a car dash wants it: the slider thick across the left two
//! thirds with a large knob and the number, and one large card at the
//! right, Stop while the sound plays and Play while it stands, as the
//! Power page's cards are. A tap on the sheet's tile starts the sound and
//! opens the page; the page leaves by itself eight seconds after the last
//! touch, the sound playing on, and the tile on the sheet wears it. The
//! face draws the page and says what the finger asked; the host plays (a
//! page in a browser today) and tells the face back what it does. The
//! page's asks never go to the player: they are `play-here` commands the
//! host keeps.

use crate::browse::{rounded, Glyphs};
use crate::lists::{Measure, Panel};
use overlay::face::{ui, Fonts, Frame, PointerKind, TextStyle};

/// Milliseconds the page stays after the last touch; then it leaves by
/// itself, the sound untouched.
pub const LINGER_MS: u64 = 8000;
/// The warm colour of the Stop card and its word, the Power page's own.
const WARM: [u8; 3] = [236, 100, 100];

/// What a touch on the page asks of the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The page took the touch and nothing more.
    Taken,
    /// The page is to shut: a tap beside it, or on its way out.
    Close,
    /// Play the sound here, at this gain (0 to 100).
    Start(u8),
    /// The gain moved, while playing or not.
    Gain(u8),
    /// Stop playing here.
    Stop,
}

/// What a finger went down on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Press {
    Leave,
    Slider,
    Card,
}

#[derive(Default)]
pub struct PlayHereView {
    pub open: bool,
    /// Whether the host plays the sound now, as it last said.
    pub on: bool,
    /// The device's volume, 0 to 100: the host's last word, or the finger's
    /// while it drags.
    pub gain: u8,
    dragging: bool,
    pressed: Option<Press>,
    closing: bool,
    touched_at: u64,
    glyphs: Glyphs,
}

impl PlayHereView {
    pub fn open(&mut self, now: u64) {
        self.open = true;
        self.pressed = None;
        self.dragging = false;
        self.closing = false;
        self.touched_at = now;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.pressed = None;
        self.dragging = false;
        self.closing = false;
    }

    /// The host's word on what it does; a finger on the slider keeps its own.
    pub fn set(&mut self, on: bool, gain: u8) {
        self.on = on;
        if !self.dragging {
            self.gain = gain.min(100);
        }
    }

    /// Every frame: the page leaves by itself once its linger has passed
    /// without a touch, never under a finger. True when it went.
    pub fn tick(&mut self, now: u64) -> bool {
        if !self.open || self.dragging || self.pressed.is_some() {
            return false;
        }
        if now.saturating_sub(self.touched_at) >= LINGER_MS {
            self.close();
            return true;
        }
        false
    }

    /// The body: what the header leaves of the panel.
    fn body(panel: &Panel, m: &Measure) -> (i32, u32) {
        (
            panel.y + m.header as i32,
            panel.h.saturating_sub(m.header).max(1),
        )
    }

    /// A panel wide enough for the slider and the card side by side; a
    /// narrower one (a drawer) stacks the card under the slider.
    fn wide(panel: &Panel, m: &Measure) -> bool {
        panel.w >= m.cell * 5
    }

    /// The card: on a wide panel at the right, the body's height less a
    /// margin above and below, a quarter of the panel wide; on a narrow
    /// one across the bottom half. Stop while the sound plays, Play while
    /// it stands.
    pub(crate) fn card(&self, panel: &Panel, m: &Measure) -> (i32, i32, u32, u32) {
        let (by, bh) = Self::body(panel, m);
        if Self::wide(panel, m) {
            let w = (panel.w / 4)
                .max(m.cell * 3 / 2)
                .min(panel.w.saturating_sub(m.pad * 2));
            let top = m.pad * 5 / 3;
            let bottom = m.pad * 2;
            let h = bh.saturating_sub(top + bottom).max(m.row * 2).min(bh);
            let x = panel.x + panel.w as i32 - m.pad as i32 - w as i32;
            let y = by + (bh as i32 - h as i32) / 2;
            (x, y, w, h)
        } else {
            let w = panel.w.saturating_sub(m.pad * 2).max(1);
            let h = (bh / 2)
                .saturating_sub(m.pad * 2)
                .max(m.row * 2)
                .min(bh / 2);
            let x = panel.x + m.pad as i32;
            let y = by + bh as i32 - m.pad as i32 - h as i32;
            (x, y, w, h)
        }
    }

    /// The slider's track: thick, with the number's room at its right; on
    /// a wide panel across the room left of the card in the middle of the
    /// body's height, on a narrow one across the panel above the card.
    pub(crate) fn track(&self, panel: &Panel, m: &Measure) -> (i32, i32, u32, u32) {
        let (by, bh) = Self::body(panel, m);
        let (cx, cy, _, _) = self.card(panel, m);
        let number = m.title * 3;
        let x = panel.x + m.pad as i32 * 2;
        let (right, below) = if Self::wide(panel, m) {
            (cx - m.pad as i32, by + bh as i32)
        } else {
            (panel.x + panel.w as i32 - m.pad as i32, cy)
        };
        let w = ((right - x).max(0) as u32)
            .saturating_sub(m.pad + number)
            .max(1);
        let room = (below - by).max(1) as u32;
        let h = (m.row * 3 / 2).min(room);
        let y = by + (room as i32 - h as i32) / 2;
        (x, y, w, h)
    }

    fn within((x0, y0, w, h): (i32, i32, u32, u32), x: i32, y: i32) -> bool {
        x >= x0 && y >= y0 && x < x0 + w as i32 && y < y0 + h as i32
    }

    fn at(&self, x: i32, y: i32, panel: &Panel, m: &Measure) -> Option<Press> {
        if Self::within(panel.leave_rect(m), x, y) {
            return Some(Press::Leave);
        }
        if Self::within(self.card(panel, m), x, y) {
            return Some(Press::Card);
        }
        // The slider takes a finger anywhere under the header that is not
        // the card's: left of it on a wide panel, above it on a narrow one.
        let (cx, cy, _, _) = self.card(panel, m);
        let slider_room = if Self::wide(panel, m) {
            x < cx - m.pad as i32 / 2
        } else {
            y < cy - m.pad as i32 / 2
        };
        if y >= panel.y + m.header as i32 && slider_room && panel.contains(x, y) {
            return Some(Press::Slider);
        }
        None
    }

    fn gain_at(&self, x: i32, panel: &Panel, m: &Measure) -> u8 {
        let (tx, _, tw, _) = self.track(panel, m);
        if tw == 0 {
            return self.gain;
        }
        (((x - tx) as f32 / tw as f32) * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8
    }

    pub fn pointer(
        &mut self,
        kind: PointerKind,
        x: i32,
        y: i32,
        panel: &Panel,
        m: &Measure,
        now: u64,
    ) -> Action {
        self.touched_at = now;
        if !panel.contains(x, y) && !self.dragging {
            // A tap beside the page shuts it, as beside the queue.
            match kind {
                PointerKind::Down => self.closing = true,
                PointerKind::Up => {
                    if std::mem::take(&mut self.closing) {
                        return Action::Close;
                    }
                }
                PointerKind::Move => {}
            }
            return Action::Taken;
        }
        let on = self.at(x, y, panel, m);
        match kind {
            PointerKind::Down => {
                self.closing = false;
                self.pressed = on;
                if on == Some(Press::Slider) {
                    self.dragging = true;
                    let gain = self.gain_at(x, panel, m);
                    self.gain = gain;
                    return Action::Gain(gain);
                }
                Action::Taken
            }
            PointerKind::Move => {
                if self.dragging {
                    let gain = self.gain_at(x, panel, m);
                    if gain != self.gain {
                        self.gain = gain;
                        return Action::Gain(gain);
                    }
                }
                Action::Taken
            }
            PointerKind::Up => {
                let was = self.pressed.take();
                if self.dragging {
                    self.dragging = false;
                    let gain = self.gain_at(x, panel, m);
                    self.gain = gain;
                    return Action::Gain(gain);
                }
                if was.is_none() || was != on {
                    return Action::Taken;
                }
                match was {
                    Some(Press::Leave) => Action::Close,
                    Some(Press::Card) if self.on => Action::Stop,
                    Some(Press::Card) => Action::Start(self.gain),
                    _ => Action::Taken,
                }
            }
        }
    }

    pub fn stamp(&self, h: &mut impl std::hash::Hasher) {
        use std::hash::Hash;
        (
            self.open,
            self.on,
            self.gain,
            self.dragging,
            self.pressed.is_some(),
        )
            .hash(h);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        frame: &mut Frame,
        fonts: &Fonts,
        panel: &Panel,
        m: &Measure,
        ink: [u8; 3],
        accent: [u8; 3],
        leave: Option<&Frame>,
        alpha: u8,
    ) {
        let dim = [ink[0] / 2 + 48, ink[1] / 2 + 48, ink[2] / 2 + 48];
        let clip = (panel.x, panel.y, panel.w, panel.h);
        // The header, as the queue's and the Power page's.
        let hx = panel.x + m.pad as i32;
        let hy = panel.y + (m.header as i32 - m.title as i32 * 13 / 10) / 2;
        let mut title_x = hx;
        if let Some(leave) = leave {
            let (lx, ly, lw, lh) = panel.leave_rect(m);
            let x = lx + (lw as i32 - leave.width as i32) / 2;
            let y = ly + (lh as i32 - leave.height as i32) / 2;
            ui::blit(frame, leave, x, y, alpha);
            title_x = lx + lw as i32 + m.pad as i32 / 4;
        }
        let title = if self.on { "Playing here" } else { "Play here" };
        if let Some(line) = ui::line(fonts, TextStyle::Bold, m.title * 13 / 10, ink, title) {
            ui::blit(frame, &line, title_x, hy, alpha);
        }
        // The slider: its caption above, a thick track, the part up to the
        // gain in the accent, a large knob, and the number at its right.
        let (tx, ty, tw, th) = self.track(panel, m);
        if let Some(line) = ui::line(
            fonts,
            TextStyle::Regular,
            m.small * 5 / 4,
            dim,
            "Volume on this device",
        ) {
            ui::blit(
                frame,
                &line,
                tx,
                ty - m.pad as i32 - line.height as i32,
                alpha,
            );
        }
        let bar_h = (th * 2 / 7).max(6);
        let bar_y = ty + (th as i32 - bar_h as i32) / 2;
        rounded(
            frame,
            tx,
            bar_y,
            tw,
            bar_h,
            bar_h / 2,
            [255, 255, 255, alpha / 8],
            clip,
        );
        let filled = (tw as f32 * self.gain as f32 / 100.0).round() as u32;
        if filled > 0 {
            rounded(
                frame,
                tx,
                bar_y,
                filled,
                bar_h,
                bar_h / 2,
                [accent[0], accent[1], accent[2], alpha],
                clip,
            );
        }
        let knob = th;
        let kx = tx + filled as i32 - knob as i32 / 2;
        let ky = ty;
        let knob_fill = if self.dragging {
            [accent[0], accent[1], accent[2], alpha]
        } else {
            [ink[0], ink[1], ink[2], alpha]
        };
        rounded(frame, kx, ky, knob, knob, knob / 2, knob_fill, clip);
        let number = format!("{}", self.gain);
        if let Some(line) = ui::line(fonts, TextStyle::Bold, m.title * 9 / 5, ink, &number) {
            ui::blit(
                frame,
                &line,
                tx + tw as i32 + m.pad as i32,
                ty + (th as i32 - line.height as i32) / 2,
                alpha,
            );
        }
        // The card: Stop in the warm colour while the sound plays, Play in
        // the ink while it stands; a ring of light while pressed.
        let (cx, cy, cw, ch) = self.card(panel, m);
        let lit = self.pressed == Some(Press::Card);
        let (fill, word_colour, word) = if self.on {
            (
                [
                    WARM[0],
                    WARM[1],
                    WARM[2],
                    if lit { alpha / 2 } else { alpha / 4 },
                ],
                WARM,
                "Stop",
            )
        } else {
            (
                [255, 255, 255, if lit { alpha / 5 } else { alpha / 10 }],
                ink,
                "Play",
            )
        };
        rounded(frame, cx, cy, cw, ch, m.pad * 2 / 3, fill, clip);
        let glyph_size = (m.title * 5).min(cw * 2 / 3);
        let g = self
            .glyphs
            .get("headphones", glyph_size, word_colour)
            .clone();
        let gx = cx + (cw as i32 - g.width as i32) / 2;
        let gy = cy + ch as i32 * 2 / 5 - g.height as i32 / 2;
        ui::blit(frame, &g, gx, gy, alpha);
        if let Some(line) = ui::line(fonts, TextStyle::Bold, m.title * 9 / 5, word_colour, word) {
            ui::blit(
                frame,
                &line,
                cx + (cw as i32 - line.width as i32) / 2,
                cy + ch as i32 * 2 / 3,
                alpha,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ListShape;

    fn panel(shape: ListShape) -> (Panel, Measure) {
        (Panel::place(shape, 1280, 720, 72, 1.0), Measure::at(1.0))
    }

    fn blank() -> Frame {
        Frame {
            width: 1280,
            height: 720,
            rgba: vec![0; 1280 * 720 * 4],
            blend: Default::default(),
        }
    }

    #[test]
    fn the_card_and_the_slider_sit_inside_the_panel_in_every_shape_the_card_at_the_right() {
        for shape in [ListShape::Sheet, ListShape::Drawer, ListShape::Screen] {
            let (panel, m) = panel(shape);
            let view = PlayHereView::default();
            let card = view.card(&panel, &m);
            let track = view.track(&panel, &m);
            for (name, (x, y, w, h)) in [("card", card), ("track", track)] {
                assert!(
                    x >= panel.x && y >= panel.y + m.header as i32,
                    "{shape:?} {name} starts under the header"
                );
                assert!(
                    x + w as i32 <= panel.x + panel.w as i32
                        && y + h as i32 <= panel.y + panel.h as i32,
                    "{shape:?} {name} ends inside: {:?} in {:?}",
                    (x, y, w, h),
                    (panel.x, panel.y, panel.w, panel.h)
                );
            }
            if PlayHereView::wide(&panel, &m) {
                assert!(
                    track.0 + track.2 as i32 + (m.title * 3) as i32 <= card.0,
                    "{shape:?}: the slider and its number stop before the card"
                );
            } else {
                assert!(
                    track.1 + track.3 as i32 <= card.1,
                    "{shape:?}: the slider stands above the card on a narrow panel"
                );
                assert!(
                    track.0 + track.2 as i32 + (m.title * 3) as i32 <= panel.x + panel.w as i32,
                    "{shape:?}: the slider and its number fit the width"
                );
            }
            assert!(card.3 >= m.row * 2, "{shape:?}: the card is large");
            assert!(track.3 >= m.row, "{shape:?}: the knob is large");
        }
    }

    #[test]
    fn a_tap_on_the_card_starts_at_the_gain_and_stops_while_playing() {
        let (panel, m) = panel(ListShape::Sheet);
        let mut view = PlayHereView::default();
        view.open(0);
        view.set(false, 80);
        let (x, y, w, h) = view.card(&panel, &m);
        let (cx, cy) = (x + w as i32 / 2, y + h as i32 / 2);
        assert_eq!(
            view.pointer(PointerKind::Down, cx, cy, &panel, &m, 100),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, cx, cy, &panel, &m, 150),
            Action::Start(80)
        );
        view.set(true, 80);
        assert_eq!(
            view.pointer(PointerKind::Down, cx, cy, &panel, &m, 200),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, cx, cy, &panel, &m, 250),
            Action::Stop
        );
        // A finger that leaves the card before lifting does nothing.
        assert_eq!(
            view.pointer(PointerKind::Down, cx, cy, &panel, &m, 300),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, 5, 5, &panel, &m, 350),
            Action::Taken
        );
    }

    #[test]
    fn a_finger_on_the_slider_sets_the_gain_as_it_moves_and_keeps_it_from_the_host_meanwhile() {
        let (panel, m) = panel(ListShape::Sheet);
        let mut view = PlayHereView::default();
        view.open(0);
        view.set(true, 50);
        let (tx, ty, tw, th) = view.track(&panel, &m);
        let cy = ty + th as i32 / 2;
        assert_eq!(
            view.pointer(PointerKind::Down, tx + tw as i32 / 4, cy, &panel, &m, 100),
            Action::Gain(25)
        );
        // The host's word does not move the knob under the finger.
        view.set(true, 90);
        assert_eq!(view.gain, 25);
        assert_eq!(
            view.pointer(PointerKind::Move, tx + tw as i32 / 2, cy, &panel, &m, 200),
            Action::Gain(50)
        );
        assert_eq!(
            view.pointer(PointerKind::Move, tx + tw as i32 / 2, cy, &panel, &m, 300),
            Action::Taken,
            "the same place again says nothing"
        );
        assert_eq!(
            view.pointer(PointerKind::Up, tx + tw as i32 + 40, cy, &panel, &m, 400),
            Action::Gain(100),
            "past the end is the end"
        );
        assert!(!view.dragging);
        view.set(true, 90);
        assert_eq!(view.gain, 90, "with the finger gone the host's word stands");
        // A finger well below the track, still left of the card, is the slider's too.
        let (_, _, _, ph) = (panel.x, panel.y, panel.w, panel.h);
        let low = panel.y + ph as i32 - 2;
        assert_eq!(
            view.pointer(PointerKind::Down, tx, low, &panel, &m, 500),
            Action::Gain(0)
        );
        view.pointer(PointerKind::Up, tx, low, &panel, &m, 550);
    }

    #[test]
    fn the_page_leaves_by_itself_after_the_linger_but_never_under_a_finger() {
        let (panel, m) = panel(ListShape::Sheet);
        let mut view = PlayHereView::default();
        view.open(1000);
        assert!(!view.tick(1000 + LINGER_MS - 1));
        assert!(view.open);
        let (tx, ty, tw, th) = view.track(&panel, &m);
        let (cx, cy) = (tx + tw as i32 / 2, ty + th as i32 / 2);
        view.pointer(PointerKind::Down, cx, cy, &panel, &m, 5000);
        assert!(
            !view.tick(5000 + LINGER_MS + 1),
            "a finger on the slider holds the page"
        );
        view.pointer(PointerKind::Up, cx, cy, &panel, &m, 6000);
        assert!(!view.tick(6000 + LINGER_MS - 1));
        assert!(view.tick(6000 + LINGER_MS));
        assert!(!view.open);
    }

    #[test]
    fn a_tap_beside_the_page_or_on_leave_shuts_it() {
        let (panel, m) = panel(ListShape::Sheet);
        let mut view = PlayHereView::default();
        view.open(0);
        assert_eq!(
            view.pointer(PointerKind::Down, 5, 5, &panel, &m, 10),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, 5, 5, &panel, &m, 20),
            Action::Close
        );
        let (lx, ly, lw, lh) = panel.leave_rect(&m);
        let (cx, cy) = (lx + lw as i32 / 2, ly + lh as i32 / 2);
        assert_eq!(
            view.pointer(PointerKind::Down, cx, cy, &panel, &m, 30),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, cx, cy, &panel, &m, 40),
            Action::Close
        );
    }

    #[test]
    fn the_page_draws_its_track_its_knob_and_its_card_inside_the_panel() {
        let (panel, m) = panel(ListShape::Sheet);
        let mut view = PlayHereView::default();
        view.open(0);
        view.set(true, 70);
        let fonts = Fonts::default();
        let mut frame = blank();
        view.draw(
            &mut frame,
            &fonts,
            &panel,
            &m,
            [235, 235, 240],
            [240, 90, 60],
            None,
            255,
        );
        let inked =
            |x: i32, y: i32| frame.rgba[((y as u32 * frame.width + x as u32) * 4 + 3) as usize] > 0;
        let (tx, ty, tw, th) = view.track(&panel, &m);
        assert!(
            inked(tx + tw as i32 / 4, ty + th as i32 / 2),
            "the filled part of the track"
        );
        let (x, y, w, h) = view.card(&panel, &m);
        assert!(inked(x + w as i32 / 2, y + h as i32 / 2), "the card");
        // Nothing drawn below the panel: the bar's room is the bar's.
        let below = panel.y + panel.h as i32;
        for x in (panel.x..panel.x + panel.w as i32).step_by(37) {
            assert!(
                !inked(x, below + 2),
                "nothing past the panel's bottom at x {x}"
            );
        }
        let mut h = std::collections::hash_map::DefaultHasher::new();
        view.stamp(&mut h);
    }
}
