//! Themes from the More sheet: the installed themes as a gallery of the
//! Manager's own previews, three a row, the one on show marked; a theme
//! opened shows its variants the same way. Apply theme sets the theme as
//! the Manager has it; a variant tapped sets the theme to that one alone.
//! Previous and Next on the sheet step the themes without the page.
//! Nothing else is offered: no delete, no cut, no edit. The page asks the
//! plugin for what it shows, as the browser asks for a location, and
//! takes the answer with its serial.

use crate::browse::{rounded, Covers, Glyphs};
use crate::lists::{blit_clipped, fill_clipped, fitted};
use crate::lists::{Measure, Panel};
use overlay::face::{ui, Command, Fonts, Frame, Input, PointerKind, TextStyle, Themes};
use std::hash::{Hash, Hasher};

/// What a touch on the page became.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Taken,
    Close,
    Send(Command),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Press {
    Tile(usize),
    Leave,
    Back,
    Apply,
}

/// A page of the gallery: the themes (`folder` empty) or one theme's
/// variants, with where it is scrolled to.
struct Page {
    folder: String,
    title: String,
    scroll: f32,
}

/// A finger down: where, the scroll then, and whether it dragged.
struct Drag {
    y0: i32,
    scroll0: f32,
    moved: bool,
}

#[derive(Default)]
pub struct ThemesView {
    pub open: bool,
    pages: Vec<Page>,
    serial: u32,
    pressed: Option<Press>,
    drag: Option<Drag>,
    closing: bool,
    covers: Covers,
    glyphs: Glyphs,
}

const COLS: usize = 3;
const DRAG_PX: i32 = 8;

impl ThemesView {
    /// Open on the gallery; the ask for it is the command returned.
    pub fn open(&mut self) -> Command {
        self.open = true;
        self.pages = vec![Page {
            folder: String::new(),
            title: "Themes".to_string(),
            scroll: 0.0,
        }];
        self.pressed = None;
        self.drag = None;
        self.closing = false;
        self.ask(String::new())
    }

    pub fn close(&mut self) {
        self.open = false;
        self.pages.clear();
        self.pressed = None;
        self.drag = None;
        self.closing = false;
    }

    /// The ask for a page, under a new serial: the themes, or a theme's variants.
    fn ask(&mut self, folder: String) -> Command {
        self.serial = self.serial.wrapping_add(1);
        Command::with(
            "themes",
            serde_json::json!({ "folder": folder, "serial": self.serial }),
        )
    }

    fn page(&self) -> Option<&Page> {
        self.pages.last()
    }

    /// Whether a theme's variants are on show, not the gallery.
    fn on_theme(&self) -> bool {
        self.page().is_some_and(|p| !p.folder.is_empty())
    }

    /// The plugin's answer to the page on show, where it has come.
    fn answer<'a>(&self, input: &'a Input) -> Option<&'a Themes> {
        input.themes.as_deref().filter(|t| t.serial == self.serial)
    }

    /// What the gallery draws from, for the face's stamp.
    pub fn stamp(&self, input: &Input, h: &mut impl Hasher) {
        self.open.hash(h);
        if !self.open {
            return;
        }
        self.serial.hash(h);
        self.pressed.hash(h);
        self.pages.len().hash(h);
        if let Some(p) = self.page() {
            (p.scroll as i32).hash(h);
        }
        input
            .themes
            .as_ref()
            .map(|t| (std::sync::Arc::as_ptr(t) as usize, t.serial))
            .hash(h);
        self.covers.ready().hash(h);
    }

    /// The tiles' geometry: three a row under the header, each a preview
    /// with its word and a line under it; the rows run on past the page
    /// and scroll.
    fn tile_box(panel: &Panel, m: &Measure) -> (i32, i32, u32, u32, u32) {
        let pad = m.pad;
        let w = (panel.w.saturating_sub(pad * (COLS as u32 + 1))) / COLS as u32;
        let thumb_h = (w.saturating_sub(pad)) * 9 / 16;
        let h = pad / 2 + thumb_h + m.small / 2 + m.title + m.small + pad / 2;
        let x0 = panel.x + pad as i32;
        let y0 = panel.y + m.header as i32 + pad as i32 / 2;
        (x0, y0, w.max(1), h.max(1), thumb_h.max(1))
    }

    /// Each tile's box on the page as scrolled, in the items' order.
    fn tiles(&self, panel: &Panel, m: &Measure, n: usize) -> Vec<(i32, i32, u32, u32)> {
        let (x0, y0, w, h, _) = Self::tile_box(panel, m);
        let scroll = self.page().map(|p| p.scroll).unwrap_or(0.0) as i32;
        let gap = m.pad;
        (0..n)
            .map(|i| {
                let (col, row) = ((i % COLS) as i32, (i / COLS) as i32);
                (
                    x0 + col * (w + gap) as i32,
                    y0 + row * (h + gap / 2) as i32 - scroll,
                    w,
                    h,
                )
            })
            .collect()
    }

    /// How far the page can scroll, for `n` tiles.
    fn scroll_most(panel: &Panel, m: &Measure, n: usize) -> f32 {
        let (_, y0, _, h, _) = Self::tile_box(panel, m);
        let rows = n.div_ceil(COLS) as i32;
        let all = rows * (h + m.pad / 2) as i32;
        let window = panel.y + panel.h as i32 - y0;
        (all - window).max(0) as f32
    }

    /// The Apply button's box at the header's right end, on a theme's page.
    fn apply_rect(panel: &Panel, m: &Measure) -> (i32, i32, u32, u32) {
        let w = m.cell;
        let h = m.header * 2 / 3;
        (
            panel.x + panel.w as i32 - w as i32 - m.pad as i32 / 2,
            panel.y + (m.header as i32 - h as i32) / 2,
            w,
            h,
        )
    }

    fn within((x0, y0, w, h): (i32, i32, u32, u32), x: i32, y: i32) -> bool {
        x >= x0 && y >= y0 && x < x0 + w as i32 && y < y0 + h as i32
    }

    fn at(&self, x: i32, y: i32, panel: &Panel, m: &Measure, input: &Input) -> Option<Press> {
        let (lx, ly, lw, lh) = panel.leave_rect(m);
        if Self::within((lx, ly, lw, lh), x, y) {
            return Some(Press::Leave);
        }
        if self.on_theme() {
            if Self::within((lx + lw as i32, ly, lw, lh), x, y) {
                return Some(Press::Back);
            }
            if Self::within(Self::apply_rect(panel, m), x, y) {
                return Some(Press::Apply);
            }
        }
        if y < panel.y + m.header as i32 {
            return None;
        }
        let n = self.answer(input).map(|t| t.items.len()).unwrap_or(0);
        self.tiles(panel, m, n)
            .into_iter()
            .position(|rect| Self::within(rect, x, y))
            .map(Press::Tile)
    }

    /// A touch on the page.
    pub fn pointer(
        &mut self,
        kind: PointerKind,
        x: i32,
        y: i32,
        panel: &Panel,
        m: &Measure,
        input: &Input,
    ) -> Action {
        if !panel.contains(x, y) {
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
        match kind {
            PointerKind::Down => {
                self.closing = false;
                self.pressed = self.at(x, y, panel, m, input);
                let scroll0 = self.page().map(|p| p.scroll).unwrap_or(0.0);
                self.drag = Some(Drag {
                    y0: y,
                    scroll0,
                    moved: false,
                });
                Action::Taken
            }
            PointerKind::Move => {
                let n = self.answer(input).map(|t| t.items.len()).unwrap_or(0);
                let most = Self::scroll_most(panel, m, n);
                if let Some(drag) = self.drag.as_mut() {
                    if !drag.moved && (y - drag.y0).abs() >= DRAG_PX {
                        drag.moved = true;
                        self.pressed = None;
                    }
                    if drag.moved {
                        let scroll = (drag.scroll0 - (y - drag.y0) as f32).clamp(0.0, most);
                        if let Some(page) = self.pages.last_mut() {
                            page.scroll = scroll;
                        }
                    }
                }
                Action::Taken
            }
            PointerKind::Up => {
                let was = self.pressed.take();
                let dragged = self.drag.take().is_some_and(|d| d.moved);
                if dragged || was.is_none() || was != self.at(x, y, panel, m, input) {
                    return Action::Taken;
                }
                match was {
                    Some(Press::Leave) => Action::Close,
                    Some(Press::Back) => {
                        self.pages.pop();
                        if self.pages.is_empty() {
                            return Action::Close;
                        }
                        Action::Send(self.ask(String::new()))
                    }
                    Some(Press::Apply) => {
                        let folder = self.page().map(|p| p.folder.clone()).unwrap_or_default();
                        Action::Send(Command::with(
                            "theme",
                            serde_json::json!({ "folder": folder }),
                        ))
                    }
                    Some(Press::Tile(i)) => {
                        let Some(item) = self.answer(input).and_then(|t| t.items.get(i)).cloned()
                        else {
                            return Action::Taken;
                        };
                        if item.meter.is_empty() {
                            // A theme: its variants, on a page of their own.
                            self.pages.push(Page {
                                folder: item.folder.clone(),
                                title: item.title.clone(),
                                scroll: 0.0,
                            });
                            Action::Send(self.ask(item.folder))
                        } else {
                            Action::Send(Command::with(
                                "theme",
                                serde_json::json!({ "folder": item.folder, "meter": item.meter }),
                            ))
                        }
                    }
                    None => Action::Taken,
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        frame: &mut Frame,
        fonts: &Fonts,
        input: &Input,
        panel: &Panel,
        m: &Measure,
        ink: [u8; 3],
        accent: [u8; 3],
        leave: Option<&Frame>,
        alpha: u8,
    ) {
        let dim = [ink[0] / 2 + 48, ink[1] / 2 + 48, ink[2] / 2 + 48];
        let clip = (panel.x, panel.y, panel.w, panel.h);
        // The header: the way out, the way back on a theme's page, the
        // title, and Apply theme at the right end on a theme's page.
        let (lx, ly, lw, lh) = panel.leave_rect(m);
        let mut title_x = panel.x + m.pad as i32;
        if let Some(leave) = leave {
            let x = lx + (lw as i32 - leave.width as i32) / 2;
            let y = ly + (lh as i32 - leave.height as i32) / 2;
            ui::blit(frame, leave, x, y, alpha);
            title_x = lx + lw as i32 + m.pad as i32 / 4;
        }
        if self.on_theme() {
            let back = self
                .glyphs
                .get("chevron-left", m.title + m.small / 2, ink)
                .clone();
            let x = lx + lw as i32 + (lw as i32 - back.width as i32) / 2;
            let y = ly + (lh as i32 - back.height as i32) / 2;
            ui::blit(frame, &back, x, y, alpha);
            title_x = lx + 2 * lw as i32 + m.pad as i32 / 4;
            let (ax, ay, aw, ah) = Self::apply_rect(panel, m);
            let wash = if self.pressed == Some(Press::Apply) {
                alpha / 4
            } else {
                alpha / 10
            };
            rounded(
                frame,
                ax,
                ay,
                aw,
                ah,
                m.pad / 3,
                [accent[0], accent[1], accent[2], wash],
                clip,
            );
            if let Some(line) = ui::line(fonts, TextStyle::Bold, m.title, ink, "Apply theme") {
                ui::blit(
                    frame,
                    &line,
                    ax + (aw as i32 - line.width as i32) / 2,
                    ay + (ah as i32 - line.height as i32) / 2,
                    alpha,
                );
            }
        }
        let title = self.page().map(|p| p.title.as_str()).unwrap_or("Themes");
        let hy = panel.y + (m.header as i32 - m.title as i32 * 13 / 10) / 2;
        if let Some(line) = ui::line(fonts, TextStyle::Bold, m.title * 13 / 10, ink, title) {
            ui::blit(frame, &line, title_x, hy, alpha);
        }
        // The body: the tiles, clipped under the header.
        let body = (
            panel.x,
            panel.y + m.header as i32,
            panel.w,
            panel.h.saturating_sub(m.header),
        );
        let Some(answer) = self.answer(input) else {
            if let Some(line) = ui::line(
                fonts,
                TextStyle::Regular,
                m.small,
                dim,
                "Asking the player…",
            ) {
                ui::blit(
                    frame,
                    &line,
                    panel.x + m.pad as i32,
                    body.1 + m.pad as i32,
                    alpha,
                );
            }
            return;
        };
        if answer.items.is_empty() {
            if let Some(line) = ui::line(
                fonts,
                TextStyle::Regular,
                m.small,
                dim,
                "Nothing installed to show.",
            ) {
                ui::blit(
                    frame,
                    &line,
                    panel.x + m.pad as i32,
                    body.1 + m.pad as i32,
                    alpha,
                );
            }
            return;
        }
        let (_, _, _, _, thumb_h) = Self::tile_box(panel, m);
        let rects = self.tiles(panel, m, answer.items.len());
        for (i, (item, rect)) in answer.items.iter().zip(rects).enumerate() {
            let (x, y, w, h) = rect;
            if y + h as i32 <= body.1 || y >= body.1 + body.3 as i32 {
                continue;
            }
            let lit = self.pressed == Some(Press::Tile(i));
            let wash = if lit { alpha / 5 } else { alpha / 12 };
            rounded(frame, x, y, w, h, m.pad / 2, [255, 255, 255, wash], body);
            let inner = (m.pad / 2) as i32;
            let (px, py) = (x + inner, y + inner);
            let side = w.saturating_sub(2 * inner as u32);
            match self.covers.get_box(&item.art_file, side, thumb_h) {
                Some(cover) => {
                    let cx = px + (side as i32 - cover.width as i32) / 2;
                    let cy = py + (thumb_h as i32 - cover.height as i32) / 2;
                    blit_clipped(frame, cover, cx, cy, alpha, body);
                }
                None => {
                    fill_clipped(
                        frame,
                        px,
                        py,
                        side,
                        thumb_h,
                        [255, 255, 255, alpha / 16],
                        body,
                    );
                }
            }
            if item.on_show {
                // The accent around the one on show.
                let t = (m.small / 5).max(2);
                let c = [accent[0], accent[1], accent[2], alpha];
                fill_clipped(
                    frame,
                    px - t as i32,
                    py - t as i32,
                    side + 2 * t,
                    t,
                    c,
                    body,
                );
                fill_clipped(
                    frame,
                    px - t as i32,
                    py + thumb_h as i32,
                    side + 2 * t,
                    t,
                    c,
                    body,
                );
                fill_clipped(frame, px - t as i32, py, t, thumb_h, c, body);
                fill_clipped(frame, px + side as i32, py, t, thumb_h, c, body);
            }
            let ty = py + thumb_h as i32 + (m.small / 2) as i32;
            if let Some(line) = fitted(fonts, TextStyle::Regular, m.title, ink, &item.title, side) {
                blit_clipped(frame, &line, px, ty, alpha, body);
            }
            let sub = if item.on_show && item.subtitle.is_empty() {
                "on show"
            } else {
                item.subtitle.as_str()
            };
            if !sub.is_empty() {
                if let Some(line) = fitted(fonts, TextStyle::Regular, m.small, dim, sub, side) {
                    blit_clipped(
                        frame,
                        &line,
                        px,
                        ty + m.title as i32 + (m.small / 4) as i32,
                        alpha,
                        body,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ListShape;
    use overlay::face::{ThemeItem, Themes};

    fn page() -> (Panel, Measure) {
        (
            Panel::place(ListShape::Screen, 1280, 720, 96, 1.0),
            Measure::at(1.0),
        )
    }

    fn answer(serial: u32, folder: &str, n: usize) -> Input {
        let items = (0..n)
            .map(|i| ThemeItem {
                title: format!("theme {i}"),
                subtitle: if folder.is_empty() {
                    "2 variants".into()
                } else {
                    String::new()
                },
                art: String::new(),
                art_file: String::new(),
                folder: if folder.is_empty() {
                    format!("f{i}")
                } else {
                    folder.to_string()
                },
                meter: if folder.is_empty() {
                    String::new()
                } else {
                    format!("m{i}")
                },
                on_show: i == 1,
            })
            .collect();
        Input {
            themes: Some(std::sync::Arc::new(Themes {
                title: if folder.is_empty() {
                    "Themes".into()
                } else {
                    folder.into()
                },
                folder: folder.to_string(),
                items,
                serial,
            })),
            ..Default::default()
        }
    }

    fn tap(
        view: &mut ThemesView,
        (x, y): (i32, i32),
        panel: &Panel,
        m: &Measure,
        input: &Input,
    ) -> Action {
        view.pointer(PointerKind::Down, x, y, panel, m, input);
        view.pointer(PointerKind::Up, x, y, panel, m, input)
    }

    fn val(c: &Command) -> &serde_json::Value {
        c.value.as_ref().expect("a command with a value")
    }

    fn centre((x, y, w, h): (i32, i32, u32, u32)) -> (i32, i32) {
        (x + w as i32 / 2, y + h as i32 / 2)
    }

    #[test]
    fn the_gallery_asks_opens_a_theme_applies_a_variant_or_the_theme_and_goes_back() {
        let (panel, m) = page();
        let mut view = ThemesView::default();
        let ask = view.open();
        assert_eq!(ask.name, "themes");
        assert_eq!(val(&ask)["folder"], "");
        let serial = val(&ask)["serial"].as_u64().unwrap() as u32;
        let none = Input::default();
        assert!(view.answer(&none).is_none(), "nothing yet");
        let stale = answer(serial.wrapping_sub(1), "", 3);
        assert!(
            view.answer(&stale).is_none(),
            "another serial's answer is not this ask's"
        );
        let gallery = answer(serial, "", 9);
        assert_eq!(view.answer(&gallery).map(|t| t.items.len()), Some(9));
        // A theme tapped: its variants asked under a new serial.
        let third = centre(view.tiles(&panel, &m, 9)[2]);
        let Action::Send(ask2) = tap(&mut view, third, &panel, &m, &gallery) else {
            panic!("a theme tapped asks for its variants");
        };
        assert_eq!(val(&ask2)["folder"], "f2");
        assert!(view.on_theme());
        let serial2 = val(&ask2)["serial"].as_u64().unwrap() as u32;
        let variants = answer(serial2, "f2", 4);
        // A variant tapped applies that one alone.
        let second = centre(view.tiles(&panel, &m, 4)[1]);
        let Action::Send(apply) = tap(&mut view, second, &panel, &m, &variants) else {
            panic!("a variant tapped applies it");
        };
        assert_eq!(
            (
                apply.name.as_str(),
                &val(&apply)["folder"],
                &val(&apply)["meter"]
            ),
            ("theme", &serde_json::json!("f2"), &serde_json::json!("m1"))
        );
        // Apply theme at the header's right end applies the theme as set.
        let Action::Send(whole) = tap(
            &mut view,
            centre(ThemesView::apply_rect(&panel, &m)),
            &panel,
            &m,
            &variants,
        ) else {
            panic!("apply theme");
        };
        assert_eq!(val(&whole), &serde_json::json!({ "folder": "f2" }));
        // Back asks the gallery again; the way out shuts.
        let (lx, ly, lw, lh) = panel.leave_rect(&m);
        let Action::Send(again) = tap(
            &mut view,
            (lx + lw as i32 + lw as i32 / 2, ly + lh as i32 / 2),
            &panel,
            &m,
            &variants,
        ) else {
            panic!("back asks the gallery");
        };
        assert_eq!(val(&again)["folder"], "");
        assert!(!view.on_theme());
        assert_eq!(
            tap(
                &mut view,
                (lx + lw as i32 / 2, ly + lh as i32 / 2),
                &panel,
                &m,
                &gallery
            ),
            Action::Close
        );
    }

    #[test]
    fn a_drag_scrolls_the_rows_and_is_no_tap_and_a_tap_beside_a_sheet_shuts() {
        let (panel, m) = page();
        let mut view = ThemesView::default();
        let ask = view.open();
        let serial = val(&ask)["serial"].as_u64().unwrap() as u32;
        let gallery = answer(serial, "", 24);
        let most = ThemesView::scroll_most(&panel, &m, 24);
        assert!(most > 0.0, "eight rows run past the page");
        let (x, y) = centre(view.tiles(&panel, &m, 24)[0]);
        view.pointer(PointerKind::Down, x, y, &panel, &m, &gallery);
        view.pointer(PointerKind::Move, x, y - 120, &panel, &m, &gallery);
        assert_eq!(
            view.pointer(PointerKind::Up, x, y - 120, &panel, &m, &gallery),
            Action::Taken,
            "a drag is no tap"
        );
        assert_eq!(view.page().unwrap().scroll, 120.0_f32.min(most));
        assert!(view.tiles(&panel, &m, 24)[0].1 < y, "the rows moved up");
        let sheet = Panel::place(ListShape::Sheet, 1280, 720, 96, 1.0);
        assert_eq!(
            tap(&mut view, (10, 10), &sheet, &m, &gallery),
            Action::Close
        );
    }

    #[test]
    fn the_gallery_draws_its_tiles_and_marks_the_one_on_show() {
        let (panel, m) = page();
        let mut view = ThemesView::default();
        let ask = view.open();
        let serial = val(&ask)["serial"].as_u64().unwrap() as u32;
        let gallery = answer(serial, "", 6);
        let fonts = Fonts::default();
        let mut frame = Frame {
            blend: Default::default(),
            width: 1280,
            height: 720,
            rgba: vec![0; 1280 * 720 * 4],
        };
        view.draw(
            &mut frame,
            &fonts,
            &gallery,
            &panel,
            &m,
            [235, 235, 240],
            [120, 170, 255],
            None,
            255,
        );
        let inked = frame.rgba.chunks(4).filter(|p| p[3] > 0).count();
        assert!(inked > 20_000, "six tiles are drawn: {inked}");
        let accent = frame
            .rgba
            .chunks(4)
            .filter(|p| p[3] > 0 && p[2] as u32 > p[0] as u32 + 60)
            .count();
        assert!(accent > 500, "the accent frames the one on show: {accent}");
        let mut asking = Frame {
            blend: Default::default(),
            width: 1280,
            height: 720,
            rgba: vec![0; 1280 * 720 * 4],
        };
        view.draw(
            &mut asking,
            &fonts,
            &Input::default(),
            &panel,
            &m,
            [235, 235, 240],
            [120, 170, 255],
            None,
            255,
        );
        let few = asking.rgba.chunks(4).filter(|p| p[3] > 0).count();
        assert!(
            few < inked,
            "without an answer only the header and a word: {few}"
        );
    }
}
