//! Power from the More sheet: off, reboot, and standby where the board
//! has it, through Volumio's own actions, the plugins' hooks first, as
//! the player's own menu does. Nothing happens on one touch: the look
//! says how the page asks, on a second screen with a large Power off and
//! a Cancel that cancels by itself, or by a tile held for two seconds
//! while a ring fills around its glyph. The face sends `power` once and
//! shuts the page; the player does the rest.

use crate::browse::{rounded, Glyphs};
use crate::lists::{Measure, Panel};
use crate::theme::PowerAsk;
use overlay::face::{ui, Fonts, Frame, PointerKind, TextStyle};

/// How long the second screen's question stands before it cancels itself.
pub const ASK_MS: u64 = 8000;
/// How long a tile is held before the action is taken.
pub const HOLD_MS: u64 = 2000;
/// The red of the action that cannot be undone.
const RED: [u8; 3] = [236, 100, 100];

/// What the page can do to the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Off,
    Reboot,
    Standby,
}

impl Kind {
    /// The word the `power` command carries.
    pub fn word(self) -> &'static str {
        match self {
            Kind::Off => "off",
            Kind::Reboot => "reboot",
            Kind::Standby => "standby",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Kind::Off => "Power off",
            Kind::Reboot => "Reboot",
            Kind::Standby => "Standby",
        }
    }

    fn line(self) -> &'static str {
        match self {
            Kind::Off => "Volumio shuts down, plugins first",
            Kind::Reboot => "the player starts again",
            Kind::Standby => "the board's own standby",
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Kind::Off => "power",
            Kind::Reboot => "rotate-ccw",
            Kind::Standby => "moon",
        }
    }

    fn question(self) -> &'static str {
        match self {
            Kind::Off => "Power off the player?",
            Kind::Reboot => "Reboot the player?",
            Kind::Standby => "Put the player in standby?",
        }
    }
}

/// What a touch on the page became.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The page took the touch and nothing more.
    Taken,
    /// The page is to shut: a tap beside it, or on its way out.
    Close,
    /// The action to send the player, once.
    Do(Kind),
}

/// What a finger went down on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Press {
    Tile(Kind),
    Confirm,
    Cancel,
    Leave,
}

#[derive(Default)]
pub struct PowerView {
    pub open: bool,
    hold: bool,
    standby: bool,
    pressed: Option<Press>,
    /// The second screen: what it asks about, and since when.
    asking: Option<(Kind, u64)>,
    /// A tile held: which, and since when.
    holding: Option<(Kind, u64)>,
    closing: bool,
    glyphs: Glyphs,
}

impl PowerView {
    pub fn open(&mut self) {
        self.open = true;
        self.pressed = None;
        self.asking = None;
        self.holding = None;
        self.closing = false;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.pressed = None;
        self.asking = None;
        self.holding = None;
        self.closing = false;
    }

    /// What the page works with, each frame: how the look asks, and
    /// whether the player has standby to offer.
    pub fn set(&mut self, ask: PowerAsk, standby: bool) {
        self.hold = ask == PowerAsk::Hold;
        self.standby = standby;
        if !standby && matches!(self.asking, Some((Kind::Standby, _))) {
            self.asking = None;
        }
    }

    /// Whether the second screen is up.
    pub fn asking(&self) -> Option<Kind> {
        self.asking.map(|(kind, _)| kind)
    }

    /// The actions the page offers: standby only where the board has it.
    fn kinds(&self) -> Vec<Kind> {
        let mut kinds = vec![Kind::Off, Kind::Reboot];
        if self.standby {
            kinds.push(Kind::Standby);
        }
        kinds
    }

    /// The tiles on the page: each action with its box.
    pub(crate) fn tiles(&self, panel: &Panel, m: &Measure) -> Vec<(Kind, (i32, i32, u32, u32))> {
        let kinds = self.kinds();
        let n = kinds.len() as u32;
        let pad = m.pad;
        let top = panel.y + m.header as i32 + pad as i32;
        let body_h = panel.h.saturating_sub(m.header + 2 * pad + m.small * 2);
        let w = ((panel.w.saturating_sub(pad * (n + 1))) / n).min(m.cell * 12 / 5);
        let h = (w * 5 / 6).min(body_h.max(m.row * 2));
        let total = n * w + (n - 1) * pad;
        let x0 = panel.x + (panel.w as i32 - total as i32) / 2;
        kinds
            .into_iter()
            .enumerate()
            .map(|(i, kind)| (kind, (x0 + i as i32 * (w + pad) as i32, top, w, h)))
            .collect()
    }

    /// The second screen's two tiles: the action, red, and Cancel.
    pub(crate) fn asks(&self, panel: &Panel, m: &Measure) -> [(i32, i32, u32, u32); 2] {
        let pad = m.pad;
        let top = panel.y
            + m.header as i32
            + pad as i32
            + (m.title * 13 / 10) as i32
            + m.small as i32
            + pad as i32;
        let w = ((panel.w.saturating_sub(pad * 3)) / 2).min(m.cell * 3);
        let h = (w * 2 / 5).max(m.row * 2);
        let total = 2 * w + pad;
        let x0 = panel.x + (panel.w as i32 - total as i32) / 2;
        [(x0, top, w, h), (x0 + (w + pad) as i32, top, w, h)]
    }

    fn within((x0, y0, w, h): (i32, i32, u32, u32), x: i32, y: i32) -> bool {
        x >= x0 && y >= y0 && x < x0 + w as i32 && y < y0 + h as i32
    }

    fn at(&self, x: i32, y: i32, panel: &Panel, m: &Measure) -> Option<Press> {
        if Self::within(panel.leave_rect(m), x, y) {
            return Some(Press::Leave);
        }
        if self.asking.is_some() {
            let [confirm, cancel] = self.asks(panel, m);
            if Self::within(confirm, x, y) {
                return Some(Press::Confirm);
            }
            if Self::within(cancel, x, y) {
                return Some(Press::Cancel);
            }
            return None;
        }
        self.tiles(panel, m)
            .into_iter()
            .find(|(_, rect)| Self::within(*rect, x, y))
            .map(|(kind, _)| Press::Tile(kind))
    }

    /// A touch on the page, at the display's clock.
    pub fn pointer(
        &mut self,
        kind: PointerKind,
        x: i32,
        y: i32,
        panel: &Panel,
        m: &Measure,
        now: u64,
    ) -> Action {
        if !panel.contains(x, y) {
            // A tap beside the page shuts it, as beside the queue.
            match kind {
                PointerKind::Down => {
                    self.closing = true;
                    self.holding = None;
                }
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
                if self.hold {
                    if let Some(Press::Tile(kind)) = on {
                        self.holding = Some((kind, now));
                    }
                }
                Action::Taken
            }
            PointerKind::Move => {
                // A finger that left the tile lets the hold go.
                if let Some((kind, _)) = self.holding {
                    if on != Some(Press::Tile(kind)) {
                        self.holding = None;
                    }
                }
                Action::Taken
            }
            PointerKind::Up => {
                let was = self.pressed.take();
                self.holding = None;
                if was.is_none() || was != on {
                    return Action::Taken;
                }
                match was {
                    Some(Press::Leave) => Action::Close,
                    Some(Press::Confirm) => {
                        let (kind, _) = self.asking.take().unwrap_or((Kind::Off, now));
                        Action::Do(kind)
                    }
                    Some(Press::Cancel) => {
                        self.asking = None;
                        Action::Taken
                    }
                    Some(Press::Tile(kind)) => {
                        if !self.hold {
                            self.asking = Some((kind, now));
                        }
                        Action::Taken
                    }
                    None => Action::Taken,
                }
            }
        }
    }

    /// Every frame: a hold that ran out is the action, once; a question
    /// that stood too long cancels itself.
    pub fn tick(&mut self, now: u64) -> Option<Kind> {
        if let Some((kind, since)) = self.asking {
            if now.saturating_sub(since) >= ASK_MS {
                self.asking = None;
            }
            let _ = kind;
        }
        if let Some((kind, since)) = self.holding {
            if now.saturating_sub(since) >= HOLD_MS {
                self.holding = None;
                self.pressed = None;
                return Some(kind);
            }
        }
        None
    }

    /// Whether the page changes between frames on its own: a ring filling
    /// or a question counting down.
    pub fn moving(&self) -> bool {
        self.holding.is_some() || self.asking.is_some()
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
        now: u64,
    ) {
        let dim = [ink[0] / 2 + 48, ink[1] / 2 + 48, ink[2] / 2 + 48];
        let clip = (panel.x, panel.y, panel.w, panel.h);
        // The header, as the queue's.
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
        let title = match self.asking {
            Some((kind, _)) => kind.title(),
            None => "Power",
        };
        if let Some(line) = ui::line(fonts, TextStyle::Bold, m.title * 13 / 10, ink, title) {
            ui::blit(frame, &line, title_x, hy, alpha);
        }
        let centred = |frame: &mut Frame, text: &str, style, size, colour, cx: i32, y: i32| {
            if let Some(line) = ui::line(fonts, style, size, colour, text) {
                ui::blit(frame, &line, cx - line.width as i32 / 2, y, alpha);
            }
        };
        let foot_y = panel.y + panel.h as i32 - m.pad as i32 - m.small as i32;
        if let Some((kind, since)) = self.asking {
            // The question, its line, the two tiles, and the count down.
            let cx = panel.x + panel.w as i32 / 2;
            let qy = panel.y + m.header as i32 + m.pad as i32;
            centred(
                frame,
                kind.question(),
                TextStyle::Bold,
                m.title * 13 / 10,
                ink,
                cx,
                qy,
            );
            centred(
                frame,
                "Volumio does it the way its own menu does: plugins close first.",
                TextStyle::Regular,
                m.small,
                dim,
                cx,
                qy + (m.title * 13 / 10) as i32 + m.small as i32 / 3,
            );
            let [confirm, cancel] = self.asks(panel, m);
            let radius = m.pad * 2 / 3;
            let lit = |press| self.pressed == Some(press);
            let red_fill = if lit(Press::Confirm) {
                alpha / 2
            } else {
                alpha / 4
            };
            rounded(
                frame,
                confirm.0,
                confirm.1,
                confirm.2,
                confirm.3,
                radius,
                [RED[0], RED[1], RED[2], red_fill],
                clip,
            );
            let grey = if lit(Press::Cancel) {
                alpha / 5
            } else {
                alpha / 10
            };
            rounded(
                frame,
                cancel.0,
                cancel.1,
                cancel.2,
                cancel.3,
                radius,
                [255, 255, 255, grey],
                clip,
            );
            let glyph_size = m.title * 2;
            for (rect, glyph, word, colour) in [
                (confirm, kind.glyph(), kind.title(), RED),
                (cancel, "x", "Cancel", ink),
            ] {
                let cx = rect.0 + rect.2 as i32 / 2;
                let g = self.glyphs.get(glyph, glyph_size, colour).clone();
                ui::blit(
                    frame,
                    &g,
                    cx - g.width as i32 / 2,
                    rect.1 + (rect.3 as i32 * 2 / 5) - g.height as i32 / 2,
                    alpha,
                );
                centred(
                    frame,
                    word,
                    TextStyle::Bold,
                    m.title * 11 / 10,
                    colour,
                    cx,
                    rect.1 + rect.3 as i32 * 2 / 3,
                );
            }
            let left = ASK_MS
                .saturating_sub(now.saturating_sub(since))
                .div_ceil(1000);
            let count = format!("Cancels by itself in {left} s");
            centred(frame, &count, TextStyle::Regular, m.small, dim, cx, foot_y);
            return;
        }
        // The page: the actions as large tiles, a ring on the one held.
        let radius = m.pad * 2 / 3;
        let glyph_size = m.title * 5 / 2;
        for (kind, rect) in self.tiles(panel, m) {
            let lit = self.pressed == Some(Press::Tile(kind))
                || matches!(self.holding, Some((k, _)) if k == kind);
            let fill = if lit {
                [accent[0], accent[1], accent[2], alpha / 4]
            } else {
                [255, 255, 255, alpha / 10]
            };
            rounded(frame, rect.0, rect.1, rect.2, rect.3, radius, fill, clip);
            let cx = rect.0 + rect.2 as i32 / 2;
            let gy = rect.1 + rect.3 as i32 * 2 / 5;
            if let Some((k, since)) = self.holding {
                if k == kind {
                    let progress = (now.saturating_sub(since) as f32 / HOLD_MS as f32).min(1.0);
                    let r = glyph_size as f32 * 0.85;
                    ring(
                        frame,
                        cx as f32,
                        gy as f32,
                        r,
                        (m.small / 4).max(3) as f32,
                        progress,
                        RED,
                        dim,
                        alpha,
                        clip,
                    );
                }
            }
            let g = self.glyphs.get(kind.glyph(), glyph_size, ink).clone();
            ui::blit(
                frame,
                &g,
                cx - g.width as i32 / 2,
                gy - g.height as i32 / 2,
                alpha,
            );
            centred(
                frame,
                kind.title(),
                TextStyle::Bold,
                m.title * 11 / 10,
                ink,
                cx,
                rect.1 + rect.3 as i32 * 3 / 5,
            );
            let line = if self.hold {
                "hold for two seconds"
            } else {
                kind.line()
            };
            centred(
                frame,
                line,
                TextStyle::Regular,
                m.small,
                dim,
                cx,
                rect.1 + rect.3 as i32 * 3 / 5 + (m.title * 11 / 10) as i32 + m.small as i32 / 2,
            );
        }
        let foot = if self.hold {
            "A finger held on a tile for two seconds does it; a lift before that does nothing."
        } else {
            "A tap asks once more on the next screen; nothing happens on one touch."
        };
        centred(
            frame,
            foot,
            TextStyle::Regular,
            m.small,
            dim,
            panel.x + panel.w as i32 / 2,
            foot_y,
        );
    }
}

/// A ring around a point: the part up to `progress` of the way round from
/// the top, clockwise, in one colour, the rest in another, feathered at
/// its edges, held to a clip box.
#[allow(clippy::too_many_arguments)]
fn ring(
    frame: &mut Frame,
    cx: f32,
    cy: f32,
    r: f32,
    width: f32,
    progress: f32,
    on: [u8; 3],
    off: [u8; 3],
    alpha: u8,
    clip: (i32, i32, u32, u32),
) {
    let (x0, y0) = ((cx - r - width) as i32, (cy - r - width) as i32);
    let (x1, y1) = ((cx + r + width) as i32 + 1, (cy + r + width) as i32 + 1);
    let (cx0, cy0) = (clip.0.max(0), clip.1.max(0));
    let cx1 = (clip.0 + clip.2 as i32).min(frame.width as i32);
    let cy1 = (clip.1 + clip.3 as i32).min(frame.height as i32);
    let turn = std::f32::consts::TAU;
    for y in y0.max(cy0)..y1.min(cy1) {
        for x in x0.max(cx0)..x1.min(cx1) {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let edge = width / 2.0 + 0.5 - (d - r).abs();
            if edge <= 0.0 {
                continue;
            }
            let cover = edge.min(1.0);
            let mut a = dx.atan2(-dy);
            if a < 0.0 {
                a += turn;
            }
            let colour = if a <= progress * turn { on } else { off };
            let a8 = (alpha as f32 * cover) as u32;
            let i = ((y as u32 * frame.width + x as u32) * 4) as usize;
            for c in 0..3 {
                let dst = frame.rgba[i + c] as u32;
                frame.rgba[i + c] = ((colour[c] as u32 * a8 + dst * (255 - a8)) / 255) as u8;
            }
            frame.rgba[i + 3] = frame.rgba[i + 3].max(a8 as u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ListShape;

    fn page() -> (Panel, Measure) {
        (
            Panel::place(ListShape::Screen, 1280, 720, 96, 1.0),
            Measure::at(1.0),
        )
    }

    fn blank() -> Frame {
        Frame {
            width: 1280,
            height: 720,
            rgba: vec![0; 1280 * 720 * 4],
            blend: Default::default(),
        }
    }

    fn centre((x, y, w, h): (i32, i32, u32, u32)) -> (i32, i32) {
        (x + w as i32 / 2, y + h as i32 / 2)
    }

    fn tap(
        view: &mut PowerView,
        (x, y): (i32, i32),
        panel: &Panel,
        m: &Measure,
        now: u64,
    ) -> Action {
        view.pointer(PointerKind::Down, x, y, panel, m, now);
        view.pointer(PointerKind::Up, x, y, panel, m, now)
    }

    #[test]
    fn the_second_screen_asks_once_cancels_by_itself_and_acts_on_the_second_tap() {
        let (panel, m) = page();
        let mut view = PowerView::default();
        view.open();
        view.set(PowerAsk::Screen, false);
        let off = centre(view.tiles(&panel, &m)[0].1);
        assert_eq!(
            tap(&mut view, off, &panel, &m, 1000),
            Action::Taken,
            "one touch does nothing"
        );
        assert_eq!(view.asking(), Some(Kind::Off), "it asks");
        let [confirm, cancel] = view.asks(&panel, &m);
        assert_eq!(
            tap(&mut view, centre(cancel), &panel, &m, 2000),
            Action::Taken
        );
        assert_eq!(view.asking(), None, "cancelled");
        tap(&mut view, off, &panel, &m, 3000);
        assert_eq!(view.tick(3000 + ASK_MS - 1), None);
        assert_eq!(view.asking(), Some(Kind::Off), "still asking");
        assert_eq!(view.tick(3000 + ASK_MS), None);
        assert_eq!(view.asking(), None, "cancelled by itself");
        tap(&mut view, off, &panel, &m, 20_000);
        assert_eq!(
            tap(&mut view, centre(confirm), &panel, &m, 21_000),
            Action::Do(Kind::Off)
        );
        assert_eq!(view.asking(), None);
        // A tap elsewhere on the page while asking is nothing: only the two answers count.
        let reboot = centre(view.tiles(&panel, &m)[1].1);
        tap(&mut view, reboot, &panel, &m, 30_000);
        assert_eq!(view.asking(), Some(Kind::Reboot));
        let [_, cancel] = view.asks(&panel, &m);
        let below = (
            cancel.0 + cancel.2 as i32 / 2,
            cancel.1 + cancel.3 as i32 + m.pad as i32 * 2,
        );
        assert_eq!(tap(&mut view, below, &panel, &m, 31_000), Action::Taken);
        assert_eq!(view.asking(), Some(Kind::Reboot), "the question stands");
    }

    #[test]
    fn a_tile_held_two_seconds_acts_and_a_lift_or_a_slide_before_does_nothing() {
        let (panel, m) = page();
        let mut view = PowerView::default();
        view.open();
        view.set(PowerAsk::Hold, false);
        let (x, y) = centre(view.tiles(&panel, &m)[0].1);
        view.pointer(PointerKind::Down, x, y, &panel, &m, 1000);
        assert!(view.moving());
        assert_eq!(view.tick(1000 + HOLD_MS - 1), None, "not yet");
        assert_eq!(
            view.pointer(PointerKind::Up, x, y, &panel, &m, 1000 + HOLD_MS - 1),
            Action::Taken,
            "a tap is nothing"
        );
        assert_eq!(view.tick(5000), None);
        assert_eq!(view.asking(), None, "and no second screen under hold");
        view.pointer(PointerKind::Down, x, y, &panel, &m, 10_000);
        view.pointer(PointerKind::Move, x + 600, y, &panel, &m, 10_500);
        assert_eq!(
            view.tick(10_000 + HOLD_MS),
            None,
            "a finger that slid off let the hold go"
        );
        view.pointer(PointerKind::Up, x + 600, y, &panel, &m, 13_000);
        view.pointer(PointerKind::Down, x, y, &panel, &m, 20_000);
        assert_eq!(
            view.tick(20_000 + HOLD_MS),
            Some(Kind::Off),
            "held: the action, once"
        );
        assert_eq!(view.tick(20_000 + HOLD_MS + 500), None, "and once only");
    }

    #[test]
    fn standby_is_offered_only_where_the_board_has_it_and_a_tap_beside_shuts() {
        let (panel, m) = page();
        let mut view = PowerView::default();
        view.open();
        view.set(PowerAsk::Screen, false);
        assert_eq!(view.tiles(&panel, &m).len(), 2);
        view.set(PowerAsk::Screen, true);
        assert_eq!(view.tiles(&panel, &m).len(), 3);
        assert_eq!(view.tiles(&panel, &m)[2].0, Kind::Standby);
        // The tiles sit side by side within the page, under the header.
        let tiles = view.tiles(&panel, &m);
        assert!(tiles.iter().all(|(_, (x, y, w, h))| *x >= panel.x
            && *y > panel.y + m.header as i32
            && x + *w as i32 <= panel.x + panel.w as i32
            && y + *h as i32 <= panel.y + panel.h as i32));
        assert!(tiles[0].1 .0 + tiles[0].1 .2 as i32 <= tiles[1].1 .0);
        // The way out and a tap beside a sheet shut the page.
        let (lx, ly, lw, lh) = panel.leave_rect(&m);
        assert_eq!(
            tap(
                &mut view,
                (lx + lw as i32 / 2, ly + lh as i32 / 2),
                &panel,
                &m,
                100
            ),
            Action::Close
        );
        let sheet = Panel::place(ListShape::Sheet, 1280, 720, 96, 1.0);
        view.open();
        assert_eq!(
            tap(&mut view, (10, 10), &sheet, &m, 200),
            Action::Close,
            "beside the sheet"
        );
    }

    #[test]
    fn the_page_draws_its_tiles_the_question_and_the_ring() {
        let (panel, m) = page();
        let mut view = PowerView::default();
        view.open();
        view.set(PowerAsk::Hold, true);
        let fonts = Fonts::default();
        let mut frame = blank();
        let (x, y) = centre(view.tiles(&panel, &m)[0].1);
        view.pointer(PointerKind::Down, x, y, &panel, &m, 1000);
        view.draw(
            &mut frame,
            &fonts,
            &panel,
            &m,
            [235, 235, 240],
            [120, 170, 255],
            None,
            255,
            1000 + HOLD_MS / 2,
        );
        let inked = frame.rgba.chunks(4).filter(|p| p[3] > 0).count();
        assert!(inked > 10_000, "the tiles and the ring are drawn: {inked}");
        view.set(PowerAsk::Screen, false);
        view.open();
        // Without Standby the tiles move: the Off tile's centre afresh.
        let (x, y) = centre(view.tiles(&panel, &m)[0].1);
        tap(&mut view, (x, y), &panel, &m, 5000);
        assert_eq!(view.asking(), Some(Kind::Off));
        let mut asked = blank();
        view.draw(
            &mut asked,
            &fonts,
            &panel,
            &m,
            [235, 235, 240],
            [120, 170, 255],
            None,
            255,
            5500,
        );
        let red = asked
            .rgba
            .chunks(4)
            .filter(|p| p[3] > 0 && p[0] as u32 > p[1] as u32 + 20)
            .count();
        assert!(red > 2000, "the question's red tile is drawn: {red}");
    }
}
