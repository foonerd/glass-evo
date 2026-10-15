//! The queue as a list the face shows over the theme: in the shape the look
//! names, with the playing track marked, a tap playing from a row and a
//! drag scrolling. The list is one piece; only its frame differs by shape.

use crate::theme::ListShape;
use overlay::face::{ui, Fonts, Frame, Input, PointerKind, TextStyle};
use std::hash::{Hash, Hasher};

/// Where a list stands on the screen, in pixels, and the shape it took.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Panel {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub shape: ListShape,
}

impl Panel {
    /// The panel for a shape on a screen `w` by `h`, above a bar `bar_h`
    /// high at the bottom, at the face's `scale`: a sheet from a third of
    /// the way down, a drawer of two fifths at the right and never under
    /// 360 units wide, or the whole screen above the bar.
    pub fn place(shape: ListShape, w: u32, h: u32, bar_h: u32, scale: f32) -> Panel {
        let (w, h) = (w.max(1), h.max(1));
        let above = h.saturating_sub(bar_h).max(1);
        match shape.on(w, h) {
            ListShape::Sheet => {
                let top = (h / 3).min(above.saturating_sub(1));
                Panel {
                    x: 0,
                    y: top as i32,
                    w,
                    h: above - top,
                    shape: ListShape::Sheet,
                }
            }
            ListShape::Drawer => {
                let width = (w * 2 / 5).max((360.0 * scale) as u32).min(w);
                Panel {
                    x: (w - width) as i32,
                    y: 0,
                    w: width,
                    h: above,
                    shape: ListShape::Drawer,
                }
            }
            _ => Panel {
                x: 0,
                y: 0,
                w,
                h: above,
                shape: ListShape::Screen,
            },
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w as i32 && y < self.y + self.h as i32
    }
}

/// What a touch on the list came to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The list took the touch and nothing more.
    Taken,
    /// The list is to shut: a tap beside it, or on its way back.
    Close,
    /// Play from this place in the queue.
    Play(usize),
}

/// The lines set for one row at a size, and the row's number.
struct RowLines {
    number: Option<Frame>,
    title: Option<Frame>,
    accented: Option<Frame>,
    subtitle: Option<Frame>,
    duration: Option<Frame>,
}

/// The rows as set for a queue: by the queue's shared address, the size
/// set at, the width the lines fit, the accent and the ink they were set in.
struct RowCache {
    queue: usize,
    size: u32,
    width: u32,
    ink: [u8; 3],
    accent: [u8; 3],
    rows: Vec<RowLines>,
}

/// The measures of a list at the face's scale, in pixels.
#[derive(Clone, Copy, Debug)]
pub struct Measure {
    pub header: u32,
    pub row: u32,
    pub title: u32,
    pub small: u32,
    pub pad: u32,
}

impl Measure {
    pub fn at(scale: f32) -> Measure {
        let unit = |n: f32| ((n * scale).round() as u32).max(1);
        Measure {
            header: unit(60.0),
            row: unit(58.0),
            title: unit(22.0),
            small: unit(16.0),
            pad: unit(24.0),
        }
    }
}

/// The queue view's state between frames.
#[derive(Default)]
pub struct QueueView {
    pub open: bool,
    /// How far the rows are scrolled, in pixels.
    scroll: f32,
    /// A finger down: where it went down, the scroll then, and whether it
    /// has moved far enough to be a drag rather than a tap.
    drag: Option<(i32, f32, bool)>,
    /// The row a finger is down on, for its highlight.
    pressed: Option<usize>,
    /// The press beside the list, or on its way back, that shuts it.
    closing: bool,
    /// Whether the rows were brought to the playing track since the list
    /// opened, so it opens on what plays and is not pulled back after.
    brought: bool,
    cache: Option<RowCache>,
}

/// How far a finger moves before a press is a drag, not a tap.
const DRAG_PX: i32 = 8;

impl QueueView {
    /// Open the list on the playing track.
    pub fn open(&mut self) {
        self.open = true;
        self.brought = false;
        self.drag = None;
        self.pressed = None;
        self.closing = false;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.drag = None;
        self.pressed = None;
        self.closing = false;
    }

    /// What the list draws from, for the face's stamp: open, where it is
    /// scrolled, what is pressed, which queue and which track plays.
    pub fn stamp(&self, input: &Input, h: &mut impl Hasher) {
        self.open.hash(h);
        if !self.open {
            return;
        }
        (self.scroll as i32).hash(h);
        self.pressed.hash(h);
        (std::sync::Arc::as_ptr(&input.queue) as usize).hash(h);
        input.queue_position.hash(h);
    }

    /// The rows' top within the panel, the rows above and the rows below
    /// the panel's window, and the furthest the rows may scroll.
    fn extent(&self, panel: &Panel, m: &Measure, rows: usize) -> (u32, f32) {
        let window = panel.h.saturating_sub(m.header);
        let all = rows as f32 * m.row as f32;
        (window, (all - window as f32).max(0.0))
    }

    /// The row under a point, where the rows are.
    fn row_at(&self, panel: &Panel, m: &Measure, rows: usize, y: i32) -> Option<usize> {
        let top = panel.y + m.header as i32;
        if y < top || y >= panel.y + panel.h as i32 {
            return None;
        }
        let at = ((y - top) as f32 + self.scroll) / m.row as f32;
        (at >= 0.0 && (at as usize) < rows).then_some(at as usize)
    }

    /// Bring the playing row into the window's middle, once per opening.
    fn bring(&mut self, panel: &Panel, m: &Measure, input: &Input) {
        if self.brought {
            return;
        }
        self.brought = true;
        let rows = input.queue.len();
        let (window, most) = self.extent(panel, m, rows);
        let Some(at) = input.queue_position else {
            self.scroll = 0.0;
            return;
        };
        let centre = at as f32 * m.row as f32 - (window as f32 - m.row as f32) / 2.0;
        self.scroll = centre.clamp(0.0, most);
    }

    /// A touch on the list, in picture pixels, with the panel it stands in.
    /// Every touch inside is the list's; a touch outside a sheet or a
    /// drawer shuts it, a touch on the way back at the top of the whole
    /// screen likewise.
    pub fn pointer(
        &mut self,
        kind: PointerKind,
        x: i32,
        y: i32,
        panel: &Panel,
        m: &Measure,
        input: &Input,
    ) -> Action {
        let rows = input.queue.len();
        let inside = panel.contains(x, y);
        let on_back = panel.shape == ListShape::Screen
            && inside
            && y < panel.y + m.header as i32
            && x < panel.x + (m.header * 2) as i32;
        match kind {
            PointerKind::Down => {
                self.closing = !inside || on_back;
                if self.closing {
                    return Action::Taken;
                }
                self.drag = Some((y, self.scroll, false));
                self.pressed = self.row_at(panel, m, rows, y);
                Action::Taken
            }
            PointerKind::Move => {
                if let Some((y0, scroll0, moved)) = self.drag {
                    let dy = y - y0;
                    let moved = moved || dy.abs() > DRAG_PX;
                    if moved {
                        let (_, most) = self.extent(panel, m, rows);
                        self.scroll = (scroll0 - dy as f32).clamp(0.0, most);
                        self.pressed = None;
                    }
                    self.drag = Some((y0, scroll0, moved));
                }
                Action::Taken
            }
            PointerKind::Up => {
                if std::mem::take(&mut self.closing) {
                    return Action::Close;
                }
                let drag = self.drag.take();
                let pressed = self.pressed.take();
                match (drag, pressed) {
                    (Some((_, _, false)), Some(row))
                        if self.row_at(panel, m, rows, y) == Some(row) =>
                    {
                        Action::Play(row)
                    }
                    _ => Action::Taken,
                }
            }
        }
    }

    /// The lines for every row, set once per queue, size, width and inks.
    fn rows<'a>(
        &'a mut self,
        fonts: &Fonts,
        input: &Input,
        m: &Measure,
        width: u32,
        ink: [u8; 3],
        accent: [u8; 3],
    ) -> &'a [RowLines] {
        let key = std::sync::Arc::as_ptr(&input.queue) as usize;
        let stale = self.cache.as_ref().is_none_or(|c| {
            c.queue != key
                || c.size != m.row
                || c.width != width
                || c.ink != ink
                || c.accent != accent
        });
        if stale {
            let dim = dimmed(ink);
            let text_w = width.saturating_sub(m.pad * 2 + m.title * 3 + m.title * 3);
            let rows = input
                .queue
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let subtitle = match (entry.artist.is_empty(), entry.album.is_empty()) {
                        (false, false) => format!("{} · {}", entry.artist, entry.album),
                        (false, true) => entry.artist.clone(),
                        (true, false) => entry.album.clone(),
                        (true, true) => String::new(),
                    };
                    let playing = input.queue_position == Some(i as u32);
                    RowLines {
                        number: ui::line(
                            fonts,
                            TextStyle::Regular,
                            m.small,
                            dim,
                            &(i + 1).to_string(),
                        ),
                        title: fitted(fonts, TextStyle::Bold, m.title, ink, &entry.title, text_w),
                        accented: playing
                            .then(|| {
                                fitted(
                                    fonts,
                                    TextStyle::Bold,
                                    m.title,
                                    accent,
                                    &entry.title,
                                    text_w,
                                )
                            })
                            .flatten(),
                        subtitle: fitted(
                            fonts,
                            TextStyle::Regular,
                            m.small,
                            dim,
                            &subtitle,
                            text_w,
                        ),
                        duration: (entry.duration > 0.0)
                            .then(|| {
                                ui::line(
                                    fonts,
                                    TextStyle::Regular,
                                    m.small,
                                    dim,
                                    &clock(entry.duration),
                                )
                            })
                            .flatten(),
                    }
                })
                .collect();
            self.cache = Some(RowCache {
                queue: key,
                size: m.row,
                width,
                ink,
                accent,
                rows,
            });
        }
        &self.cache.as_ref().expect("set just now").rows
    }

    /// Draw the list in its panel: the header with the count and the
    /// playing place, the rows in the window, the playing row in the
    /// accent with a mark at its edge, the pressed row lit, a thin bar at
    /// the right for where the rows stand, and the way back on the whole
    /// screen. The panel's glass is the caller's, drawn before this.
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
        back: Option<&Frame>,
        alpha: u8,
    ) {
        self.bring(panel, m, input);
        let rows = input.queue.len();
        let dim = dimmed(ink);
        // The header.
        let hx = panel.x + m.pad as i32;
        let hy = panel.y + (m.header as i32 - m.title as i32 * 13 / 10) / 2;
        let mut title_x = hx;
        if let (ListShape::Screen, Some(back)) = (panel.shape, back) {
            let by = panel.y + (m.header as i32 - back.height as i32) / 2;
            ui::blit(frame, back, hx, by, alpha);
            title_x += back.width as i32 + m.pad as i32 / 2;
        }
        if let Some(line) = ui::line(fonts, TextStyle::Bold, m.title * 13 / 10, ink, "Queue") {
            ui::blit(frame, &line, title_x, hy, alpha);
        }
        let count = match input.queue_position {
            Some(at) if rows > 0 => format!("{rows} tracks · now {} of {rows}", at + 1),
            _ => format!("{rows} tracks"),
        };
        if let Some(line) = ui::line(fonts, TextStyle::Regular, m.small, dim, &count) {
            let x = panel.x + panel.w as i32 - m.pad as i32 - line.width as i32;
            let y = panel.y + (m.header as i32 - line.height as i32) / 2;
            ui::blit(frame, &line, x, y, alpha);
        }
        // The rows in the window.
        let clip = (
            panel.x,
            panel.y + m.header as i32,
            panel.w,
            panel.h.saturating_sub(m.header),
        );
        let (window, _) = self.extent(panel, m, rows);
        let scroll = self.scroll;
        let pressed = self.pressed;
        let position = input.queue_position;
        let first = (scroll / m.row as f32).floor().max(0.0) as usize;
        let last = ((scroll + window as f32) / m.row as f32).ceil() as usize;
        let lines = self.rows(fonts, input, m, panel.w, ink, accent);
        for (i, row) in lines.iter().enumerate().take(last.min(rows)).skip(first) {
            let y = clip.1 + (i as f32 * m.row as f32 - scroll).round() as i32;
            let playing = position == Some(i as u32);
            if pressed == Some(i) {
                fill_clipped(
                    frame,
                    panel.x,
                    y,
                    panel.w,
                    m.row,
                    [255, 255, 255, alpha / 10],
                    clip,
                );
            }
            if playing {
                let mark = (4.0 * m.row as f32 / 58.0).round().max(2.0) as u32;
                fill_clipped(
                    frame,
                    panel.x,
                    y + 8,
                    mark,
                    m.row - 16,
                    [accent[0], accent[1], accent[2], alpha],
                    clip,
                );
            }
            let x_text = panel.x + m.pad as i32 + m.title as i32 * 2;
            if let Some(number) = &row.number {
                let nx = panel.x + m.pad as i32 - 2;
                blit_clipped(
                    frame,
                    number,
                    nx,
                    y + (m.row as i32 - number.height as i32) / 2,
                    alpha,
                    clip,
                );
            }
            let title = if playing {
                row.accented.as_ref().or(row.title.as_ref())
            } else {
                row.title.as_ref()
            };
            if let Some(title) = title {
                blit_clipped(
                    frame,
                    title,
                    x_text,
                    y + m.row as i32 / 2 - title.height as i32 - 1,
                    alpha,
                    clip,
                );
            }
            if let Some(sub) = &row.subtitle {
                blit_clipped(frame, sub, x_text, y + m.row as i32 / 2 + 2, alpha, clip);
            }
            if let Some(duration) = &row.duration {
                let dx = panel.x + panel.w as i32
                    - m.pad as i32
                    - duration.width as i32
                    - m.title as i32 / 2;
                blit_clipped(
                    frame,
                    duration,
                    dx,
                    y + (m.row as i32 - duration.height as i32) / 2,
                    alpha,
                    clip,
                );
            }
            fill_clipped(
                frame,
                panel.x + m.pad as i32 / 2,
                y + m.row as i32 - 1,
                panel.w.saturating_sub(m.pad),
                1,
                [255, 255, 255, alpha / 10],
                clip,
            );
        }
        // Where the rows stand, as a thin bar at the right.
        let all = rows as f32 * m.row as f32;
        if all > window as f32 && window > 0 {
            let track = window.saturating_sub(m.pad);
            let len = ((window as f32 / all) * track as f32).max(m.pad as f32) as u32;
            let top = clip.1 + m.pad as i32 / 2 + ((scroll / all) * track as f32) as i32;
            let x = panel.x + panel.w as i32 - m.pad as i32 / 2 - 2;
            fill_clipped(
                frame,
                x,
                top,
                4,
                len.min(track),
                [255, 255, 255, alpha / 5],
                clip,
            );
        }
    }
}

/// The ink at about half strength, for what is said in a lower voice.
fn dimmed(ink: [u8; 3]) -> [u8; 3] {
    [
        ink[0] / 2 + ink[0] / 8,
        ink[1] / 2 + ink[1] / 8,
        ink[2] / 2 + ink[2] / 8,
    ]
}

/// Seconds as a clock reads them.
fn clock(seconds: f32) -> String {
    let s = seconds.max(0.0).round() as u64;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// A line of text set to fit a width: whole where it does, else cut with
/// an ellipsis at the widest prefix that does.
fn fitted(
    fonts: &Fonts,
    style: TextStyle,
    size: u32,
    ink: [u8; 3],
    text: &str,
    width: u32,
) -> Option<Frame> {
    if text.is_empty() {
        return None;
    }
    let whole = ui::line(fonts, style, size, ink, text)?;
    if whole.width <= width {
        return Some(whole);
    }
    let chars: Vec<char> = text.chars().collect();
    let (mut low, mut high) = (0usize, chars.len());
    let mut best = None;
    while low < high {
        let mid = (low + high).div_ceil(2);
        let cut: String = chars[..mid]
            .iter()
            .collect::<String>()
            .trim_end()
            .to_string()
            + "…";
        match ui::line(fonts, style, size, ink, &cut) {
            Some(line) if line.width <= width => {
                best = Some(line);
                low = mid;
            }
            _ => high = mid - 1,
        }
    }
    best.or_else(|| ui::line(fonts, style, size, ink, "…"))
}

/// A fill held to a clip box.
fn fill_clipped(
    frame: &mut Frame,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    rgba: [u8; 4],
    clip: (i32, i32, u32, u32),
) {
    let x0 = x.max(clip.0);
    let y0 = y.max(clip.1);
    let x1 = (x + w as i32).min(clip.0 + clip.2 as i32);
    let y1 = (y + h as i32).min(clip.1 + clip.3 as i32);
    if x1 > x0 && y1 > y0 {
        ui::fill(frame, x0, y0, (x1 - x0) as u32, (y1 - y0) as u32, rgba);
    }
}

/// A picture laid at a place, held to a clip box.
fn blit_clipped(
    frame: &mut Frame,
    src: &Frame,
    x: i32,
    y: i32,
    alpha: u8,
    clip: (i32, i32, u32, u32),
) {
    let x0 = x.max(clip.0);
    let y0 = y.max(clip.1);
    let x1 = (x + src.width as i32).min(clip.0 + clip.2 as i32);
    let y1 = (y + src.height as i32).min(clip.1 + clip.3 as i32);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    if x0 == x && y0 == y && x1 == x + src.width as i32 && y1 == y + src.height as i32 {
        ui::blit(frame, src, x, y, alpha);
        return;
    }
    // The part within the box, cut out and laid.
    let (cw, ch) = ((x1 - x0) as u32, (y1 - y0) as u32);
    let mut part = Frame {
        blend: src.blend,
        width: cw,
        height: ch,
        rgba: vec![0; (cw * ch * 4) as usize],
    };
    for row in 0..ch {
        let sy = (y0 - y) as u32 + row;
        let from = ((sy * src.width + (x0 - x) as u32) * 4) as usize;
        let to = ((row * cw) * 4) as usize;
        part.rgba[to..to + (cw * 4) as usize]
            .copy_from_slice(&src.rgba[from..from + (cw * 4) as usize]);
    }
    ui::blit(frame, &part, x0, y0, alpha);
}

#[cfg(test)]
mod tests {
    use super::*;
    use overlay::face::QueueEntry;
    use std::sync::Arc;

    fn queue(n: usize, playing: Option<u32>) -> Input {
        Input {
            queue: Arc::new(
                (0..n)
                    .map(|i| QueueEntry {
                        title: format!("Track {}", i + 1),
                        artist: "Band".into(),
                        album: "Album".into(),
                        duration: 200.0,
                        ..Default::default()
                    })
                    .collect(),
            ),
            queue_position: playing,
            ..Default::default()
        }
    }

    /// The shapes by the screen: a sheet on a wide screen, the whole
    /// screen on a portrait one; each where it should be, above the bar.
    #[test]
    fn the_panel_takes_its_shape_from_the_look_and_the_screen() {
        let sheet = Panel::place(ListShape::Auto, 1280, 720, 64, 1.0);
        assert_eq!(sheet.shape, ListShape::Sheet);
        assert_eq!((sheet.x, sheet.y, sheet.w, sheet.h), (0, 240, 1280, 416));
        let tall = Panel::place(ListShape::Auto, 720, 1280, 64, 1.0);
        assert_eq!(tall.shape, ListShape::Screen);
        assert_eq!((tall.x, tall.y, tall.w, tall.h), (0, 0, 720, 1216));
        let drawer = Panel::place(ListShape::Drawer, 1280, 720, 64, 1.0);
        assert_eq!(drawer.shape, ListShape::Drawer);
        assert_eq!((drawer.x, drawer.y, drawer.w, drawer.h), (768, 0, 512, 656));
        assert!(drawer.contains(800, 10) && !drawer.contains(700, 10));
        let narrow = Panel::place(ListShape::Drawer, 480, 320, 48, 1.0);
        assert_eq!(narrow.w, 360, "never under 360 units wide");
        let screen = Panel::place(ListShape::Screen, 1280, 720, 64, 1.0);
        assert_eq!((screen.y, screen.h), (0, 656));
    }

    /// A tap on a row plays from it; a drag scrolls and plays nothing; a
    /// tap beside a sheet shuts it; the list opens on the playing track.
    #[test]
    fn a_tap_plays_a_drag_scrolls_and_a_tap_beside_shuts() {
        let input = queue(40, Some(20));
        let panel = Panel::place(ListShape::Sheet, 1280, 720, 64, 1.0);
        let m = Measure::at(1.0);
        let mut list = QueueView::default();
        list.open();
        let fonts = Fonts::default();
        let mut frame = Frame {
            blend: Default::default(),
            width: 1280,
            height: 720,
            rgba: vec![0; 1280 * 720 * 4],
        };
        list.draw(
            &mut frame,
            &fonts,
            &input,
            &panel,
            &m,
            [240, 240, 240],
            [80, 140, 240],
            None,
            255,
        );
        let (window, most) = list.extent(&panel, &m, 40);
        assert!(most > 0.0 && window > 0);
        let row_20_centre = 20.0 * m.row as f32 - (window as f32 - m.row as f32) / 2.0;
        assert!(
            (list.scroll - row_20_centre.clamp(0.0, most)).abs() < 0.5,
            "opened on the playing track"
        );
        // A tap on the first row in the window plays that row.
        let y = panel.y + m.header as i32 + 5;
        let row = list.row_at(&panel, &m, 40, y).unwrap();
        assert_eq!(
            list.pointer(PointerKind::Down, 300, y, &panel, &m, &input),
            Action::Taken
        );
        assert_eq!(
            list.pointer(PointerKind::Up, 300, y, &panel, &m, &input),
            Action::Play(row)
        );
        // A drag down by a hundred pixels scrolls up and plays nothing.
        let before = list.scroll;
        assert_eq!(
            list.pointer(PointerKind::Down, 300, y + 100, &panel, &m, &input),
            Action::Taken
        );
        assert_eq!(
            list.pointer(PointerKind::Move, 300, y + 200, &panel, &m, &input),
            Action::Taken
        );
        assert_eq!(
            list.pointer(PointerKind::Up, 300, y + 200, &panel, &m, &input),
            Action::Taken
        );
        assert!(
            (before - list.scroll - 100.0).abs() < 0.5,
            "scrolled by the drag: {} then {}",
            before,
            list.scroll
        );
        // A tap beside the sheet shuts it.
        assert_eq!(
            list.pointer(PointerKind::Down, 300, 10, &panel, &m, &input),
            Action::Taken
        );
        assert_eq!(
            list.pointer(PointerKind::Up, 300, 10, &panel, &m, &input),
            Action::Close
        );
        // The whole screen: the way back at the top left shuts it.
        let whole = Panel::place(ListShape::Screen, 1280, 720, 64, 1.0);
        let mut list = QueueView::default();
        list.open();
        assert_eq!(
            list.pointer(PointerKind::Down, 30, 20, &whole, &m, &input),
            Action::Taken
        );
        assert_eq!(
            list.pointer(PointerKind::Up, 30, 20, &whole, &m, &input),
            Action::Close
        );
    }

    /// Text is cut with an ellipsis where it does not fit, and a clock
    /// reads seconds as minutes and seconds, hours above an hour.
    #[test]
    fn long_text_is_cut_and_seconds_read_as_a_clock() {
        assert_eq!(clock(65.0), "1:05");
        assert_eq!(clock(3725.0), "1:02:05");
        let fonts = Fonts::default();
        let whole = fitted(
            &fonts,
            TextStyle::Bold,
            20,
            [255, 255, 255],
            "A long title that cannot fit here",
            80,
        );
        if let Some(whole) = whole {
            assert!(whole.width <= 80, "held to the width: {}", whole.width);
        }
        assert!(fitted(&fonts, TextStyle::Bold, 20, [255; 3], "", 80).is_none());
    }
}
