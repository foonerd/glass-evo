//! Play here from the More sheet: the player's sound on the device that
//! draws the face, with the device's own volume on a page over the theme.
//! The face draws the page and says what the finger asked; the host plays
//! the sound (a page in a browser today) and tells the face back what it
//! does, so the tile and the slider wear the truth. The page's asks never
//! go to the player: they are `play-here` commands the host keeps.

use crate::browse::{rounded, Glyphs};
use crate::lists::{Measure, Panel};
use overlay::face::{ui, Fonts, Frame, PointerKind, TextStyle};

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
    Tile,
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
    glyphs: Glyphs,
}

impl PlayHereView {
    pub fn open(&mut self) {
        self.open = true;
        self.pressed = None;
        self.dragging = false;
        self.closing = false;
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

    /// The slider's track: across the page under its header and line.
    pub(crate) fn track(&self, panel: &Panel, m: &Measure) -> (i32, i32, u32, u32) {
        let pad = m.pad;
        let top = panel.y + m.header as i32 + pad as i32 + m.small as i32 + pad as i32;
        let label = m.title * 2;
        let x = panel.x + pad as i32 * 2;
        let w = panel.w.saturating_sub(pad * 4 + label).max(m.cell);
        (x, top, w, m.row)
    }

    /// The one tile under the slider: Play here, or Stop while playing.
    pub(crate) fn tile(&self, panel: &Panel, m: &Measure) -> (i32, i32, u32, u32) {
        let (_, ty, _, th) = self.track(panel, m);
        let pad = m.pad;
        let w = (panel.w.saturating_sub(pad * 2)).min(m.cell * 3);
        let h = (w * 2 / 5).max(m.row * 2);
        let x = panel.x + (panel.w as i32 - w as i32) / 2;
        (x, ty + th as i32 + pad as i32 * 2, w, h)
    }

    fn within((x0, y0, w, h): (i32, i32, u32, u32), x: i32, y: i32) -> bool {
        x >= x0 && y >= y0 && x < x0 + w as i32 && y < y0 + h as i32
    }

    fn at(&self, x: i32, y: i32, panel: &Panel, m: &Measure) -> Option<Press> {
        if Self::within(panel.leave_rect(m), x, y) {
            return Some(Press::Leave);
        }
        // The slider takes a finger a row above or below its track too.
        let (tx, ty, tw, th) = self.track(panel, m);
        if Self::within((tx, ty - th as i32, tw, th * 3), x, y) {
            return Some(Press::Slider);
        }
        if Self::within(self.tile(panel, m), x, y) {
            return Some(Press::Tile);
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
    ) -> Action {
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
                    Some(Press::Tile) if self.on => Action::Stop,
                    Some(Press::Tile) => Action::Start(self.gain),
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
        if let Some(line) = ui::line(fonts, TextStyle::Bold, m.title * 13 / 10, ink, "Play here") {
            ui::blit(frame, &line, title_x, hy, alpha);
        }
        let centred = |frame: &mut Frame, text: &str, style, size, colour, cx: i32, y: i32| {
            if let Some(line) = ui::line(fonts, style, size, colour, text) {
                ui::blit(frame, &line, cx - line.width as i32 / 2, y, alpha);
            }
        };
        let cx = panel.x + panel.w as i32 / 2;
        // The line under the header: what this is.
        centred(
            frame,
            "The player's sound on this device, a few tenths of a second behind the room.",
            TextStyle::Regular,
            m.small,
            dim,
            cx,
            panel.y + m.header as i32 + m.pad as i32 / 2,
        );
        // The slider: a track, the part up to the gain in the accent, a knob,
        // and the number beside it.
        let (tx, ty, tw, th) = self.track(panel, m);
        let bar_h = (th / 4).max(4);
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
        let knob = th * 3 / 4;
        let kx = tx + filled as i32 - knob as i32 / 2;
        let ky = ty + (th as i32 - knob as i32) / 2;
        let knob_fill = if self.dragging {
            [accent[0], accent[1], accent[2], alpha]
        } else {
            [ink[0], ink[1], ink[2], alpha]
        };
        rounded(frame, kx, ky, knob, knob, knob / 2, knob_fill, clip);
        let number = format!("{}", self.gain);
        if let Some(line) = ui::line(fonts, TextStyle::Bold, m.title, ink, &number) {
            ui::blit(
                frame,
                &line,
                tx + tw as i32 + m.pad as i32,
                ty + (th as i32 - line.height as i32) / 2,
                alpha,
            );
        }
        // The tile: Play here, or Stop while the host plays.
        let rect = self.tile(panel, m);
        let lit = self.pressed == Some(Press::Tile);
        let fill = if self.on {
            [
                accent[0],
                accent[1],
                accent[2],
                if lit { alpha / 3 } else { alpha / 5 },
            ]
        } else {
            [255, 255, 255, if lit { alpha / 5 } else { alpha / 10 }]
        };
        rounded(
            frame,
            rect.0,
            rect.1,
            rect.2,
            rect.3,
            m.pad * 2 / 3,
            fill,
            clip,
        );
        let glyph_size = m.title * 2;
        let g = self.glyphs.get("headphones", glyph_size, ink).clone();
        let gx = rect.0 + rect.2 as i32 / 2 - g.width as i32 / 2;
        let gy = rect.1 + rect.3 as i32 * 2 / 5 - g.height as i32 / 2;
        ui::blit(frame, &g, gx, gy, alpha);
        centred(
            frame,
            if self.on { "Stop" } else { "Play here" },
            TextStyle::Bold,
            m.title * 11 / 10,
            ink,
            rect.0 + rect.2 as i32 / 2,
            rect.1 + rect.3 as i32 * 2 / 3,
        );
        // The foot: where the level comes from.
        let foot_y = panel.y + panel.h as i32 - m.pad as i32 - m.small as i32;
        centred(
            frame,
            "At the player's full level, before its volume; the slider is this device's own.",
            TextStyle::Regular,
            m.small,
            dim,
            cx,
            foot_y,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ListShape;

    fn panel() -> (Panel, Measure) {
        (
            Panel::place(ListShape::Sheet, 1280, 720, 72, 1.0),
            Measure::at(1.0),
        )
    }

    #[test]
    fn a_tap_on_the_tile_starts_at_the_gain_and_stops_while_playing() {
        let (panel, m) = panel();
        let mut view = PlayHereView::default();
        view.open();
        view.set(false, 80);
        let (x, y, w, h) = view.tile(&panel, &m);
        let (cx, cy) = (x + w as i32 / 2, y + h as i32 / 2);
        assert_eq!(
            view.pointer(PointerKind::Down, cx, cy, &panel, &m),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, cx, cy, &panel, &m),
            Action::Start(80)
        );
        view.set(true, 80);
        assert_eq!(
            view.pointer(PointerKind::Down, cx, cy, &panel, &m),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, cx, cy, &panel, &m),
            Action::Stop
        );
        // A finger that leaves the tile before lifting does nothing.
        assert_eq!(
            view.pointer(PointerKind::Down, cx, cy, &panel, &m),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, cx, cy + h as i32 * 2, &panel, &m),
            Action::Taken
        );
    }

    #[test]
    fn a_finger_on_the_slider_sets_the_gain_as_it_moves_and_keeps_it_from_the_host_meanwhile() {
        let (panel, m) = panel();
        let mut view = PlayHereView::default();
        view.open();
        view.set(true, 50);
        let (tx, ty, tw, th) = view.track(&panel, &m);
        let cy = ty + th as i32 / 2;
        assert_eq!(
            view.pointer(PointerKind::Down, tx + tw as i32 / 4, cy, &panel, &m),
            Action::Gain(25)
        );
        // The host's word does not move the knob under the finger.
        view.set(true, 90);
        assert_eq!(view.gain, 25);
        assert_eq!(
            view.pointer(PointerKind::Move, tx + tw as i32 / 2, cy, &panel, &m),
            Action::Gain(50)
        );
        assert_eq!(
            view.pointer(PointerKind::Move, tx + tw as i32 / 2, cy, &panel, &m),
            Action::Taken,
            "the same place again says nothing"
        );
        assert_eq!(
            view.pointer(PointerKind::Up, tx + tw as i32 + 500, cy, &panel, &m),
            Action::Gain(100),
            "past the end is the end"
        );
        assert!(!view.dragging);
        view.set(true, 90);
        assert_eq!(view.gain, 90, "with the finger gone the host's word stands");
    }

    #[test]
    fn a_tap_beside_the_page_or_on_leave_shuts_it() {
        let (panel, m) = panel();
        let mut view = PlayHereView::default();
        view.open();
        assert_eq!(
            view.pointer(PointerKind::Down, 5, 5, &panel, &m),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, 5, 5, &panel, &m),
            Action::Close
        );
        let (lx, ly, lw, lh) = panel.leave_rect(&m);
        let (cx, cy) = (lx + lw as i32 / 2, ly + lh as i32 / 2);
        assert_eq!(
            view.pointer(PointerKind::Down, cx, cy, &panel, &m),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, cx, cy, &panel, &m),
            Action::Close
        );
    }

    #[test]
    fn the_page_draws_its_track_its_knob_and_its_tile() {
        let (panel, m) = panel();
        let mut view = PlayHereView::default();
        view.open();
        view.set(true, 70);
        let fonts = Fonts::default();
        let mut frame = Frame {
            width: 1280,
            height: 720,
            rgba: vec![0; 1280 * 720 * 4],
            blend: Default::default(),
        };
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
        let (tx, ty, tw, th) = view.track(&panel, &m);
        let inked =
            |x: i32, y: i32| frame.rgba[((y as u32 * frame.width + x as u32) * 4 + 3) as usize] > 0;
        assert!(
            inked(tx + tw as i32 / 4, ty + th as i32 / 2),
            "the filled part of the track"
        );
        let (x, y, w, h) = view.tile(&panel, &m);
        assert!(inked(x + w as i32 / 2, y + h as i32 / 2), "the tile");
        let mut h = std::collections::hash_map::DefaultHasher::new();
        view.stamp(&mut h);
    }
}
