//! The player's library, playlists and radios as the face shows them over
//! the theme: the sources as tiles, a location's items as rows with a
//! cover or an icon, a tap opening a folder or playing a track from its
//! list, a plus adding to the queue, the way back up the pages the finger
//! came down, and a search typed on a keyboard the face draws. The lists
//! come from the player through the engine's input; the asks go to the
//! player as commands.

use crate::icon;
use crate::lists::{blit_clipped, clock, dimmed, fill_clipped, fitted, Measure, Panel};
use crate::lucide;
use overlay::face::{
    read_art, ui, Browse, BrowseItem, Command, Fonts, Frame, Input, PointerKind, TextStyle,
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// What a touch on the browser came to.
#[derive(Debug, PartialEq)]
pub enum Action {
    /// The browser took the touch and nothing more.
    Taken,
    /// The browser is to shut: a tap beside it, or its way out.
    Close,
    /// A command for the player: an ask, a play, an add.
    Send(Command),
}

/// One page the finger came down to: the location, its word, where it
/// was scrolled when left, and whether it is the keyboard.
#[derive(Clone, Debug, PartialEq)]
struct Page {
    uri: String,
    title: String,
    scroll: f32,
    keys: bool,
}

/// What a finger is down on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Press {
    Row(usize),
    Add(usize),
    /// The add-and-play symbol before the plus.
    Play(usize),
    /// The clear-and-play symbol before that.
    Replace(usize),
    Back,
    Close,
    Key(usize),
}

/// The lines set for one row at a size.
struct RowLines {
    title: Option<Frame>,
    subtitle: Option<Frame>,
    duration: Option<Frame>,
}

/// The rows as set for one answer: by its serial, the size set at, the
/// width the lines fit and the ink; each row set when first drawn.
struct RowCache {
    serial: u32,
    size: u32,
    width: u32,
    ink: [u8; 3],
    rows: Vec<Option<RowLines>>,
}

/// A cover being read for a row, on a player by a thread.
#[cfg(not(target_arch = "wasm32"))]
struct CoverReading(std::sync::mpsc::Receiver<Option<Frame>>);
#[cfg(target_arch = "wasm32")]
struct CoverReading(Option<Frame>);

impl CoverReading {
    /// A cover read into a box of `w` by `h`: a square for a row's cover,
    /// a theme's own shape for a preview.
    #[cfg(not(target_arch = "wasm32"))]
    fn of(path: String, w: u32, h: u32) -> Self {
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = send.send(read_art(Path::new(&path), w, h, None));
        });
        CoverReading(receive)
    }

    #[cfg(target_arch = "wasm32")]
    fn of(path: String, w: u32, h: u32) -> Self {
        CoverReading(read_art(Path::new(&path), w, h, None))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn done(&mut self) -> Option<Option<Frame>> {
        match self.0.try_recv() {
            Ok(found) => Some(found),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(None),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn done(&mut self) -> Option<Option<Frame>> {
        Some(self.0.take())
    }
}

enum Cover {
    Reading(CoverReading),
    Done(Option<Frame>),
}

/// The covers read for the rows, by file and box; let go all at once
/// when there are too many. The queue's rows and the themes' gallery share
/// the kind.
#[derive(Default)]
pub(crate) struct Covers {
    have: HashMap<(String, u32, u32), Cover>,
}

/// Lucide's glyphs rastered by name, side and ink, once each; an unknown
/// name is a folder.
#[derive(Default)]
pub(crate) struct Glyphs {
    map: HashMap<(String, u32, [u8; 3]), Frame>,
}

impl Glyphs {
    pub(crate) fn get(&mut self, name: &str, size: u32, ink: [u8; 3]) -> &Frame {
        let name = if lucide::ALL.iter().any(|(n, _)| *n == name) {
            name
        } else {
            "folder"
        };
        self.map
            .entry((name.to_string(), size, ink))
            .or_insert_with(|| {
                let shapes = lucide::ALL
                    .iter()
                    .find(|(n, _)| *n == name)
                    .map_or(&[][..], |(_, s)| s);
                icon::lucide(shapes, size, ink)
            })
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.map.len()
    }
}

const COVERS_MOST: usize = 96;

impl Covers {
    /// The cover of a file at a side, once read; reading starts at the
    /// first ask.
    /// A cover in a square of `size`, as a row wears it.
    pub(crate) fn get(&mut self, file: &str, size: u32) -> Option<&Frame> {
        self.get_box(file, size, size)
    }

    /// A cover in a box of `w` by `h`, as a theme's preview keeps its shape.
    pub(crate) fn get_box(&mut self, file: &str, w: u32, h: u32) -> Option<&Frame> {
        if file.is_empty() {
            return None;
        }
        if self.have.len() > COVERS_MOST {
            self.have.clear();
        }
        let key = (file.to_string(), w, h);
        let slot = self
            .have
            .entry(key)
            .or_insert_with(|| Cover::Reading(CoverReading::of(file.to_string(), w, h)));
        if let Cover::Reading(reading) = slot {
            if let Some(found) = reading.done() {
                *slot = Cover::Done(found);
            }
        }
        match slot {
            Cover::Done(found) => found.as_ref(),
            Cover::Reading(_) => None,
        }
    }

    /// How many covers are in hand, for the stamp.
    pub(crate) fn ready(&self) -> usize {
        self.have
            .values()
            .filter(|c| matches!(c, Cover::Done(Some(_))))
            .count()
    }
}

/// The keyboard's letters, four rows; the fifth row is backspace, space
/// and go.
const KEY_ROWS: [&str; 4] = ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"];
const KEY_BACK: usize = 36;
const KEY_SPACE: usize = 37;
const KEY_GO: usize = 38;
const KEYS: usize = 39;
/// The most letters a search takes.
const WORDS_MOST: usize = 60;

/// A key's place on the keyboard: x and width in tenths of the panel's
/// width, and its row.
fn key_place(i: usize) -> (f32, f32, usize) {
    let mut n = 0;
    for (row, letters) in KEY_ROWS.iter().enumerate() {
        let count = letters.len();
        if i < n + count {
            let offset = (10 - count) as f32 / 2.0;
            return (offset + (i - n) as f32, 1.0, row);
        }
        n += count;
    }
    match i {
        KEY_BACK => (0.0, 2.0, 4),
        KEY_SPACE => (2.0, 5.0, 4),
        _ => (7.0, 3.0, 4),
    }
}

fn key_letter(i: usize) -> Option<char> {
    KEY_ROWS.iter().flat_map(|row| row.chars()).nth(i)
}

/// The browser's state between frames.
#[derive(Default)]
pub struct BrowseView {
    pub open: bool,
    pages: Vec<Page>,
    /// The serial of the last ask; an answer with another serial is not
    /// this page's.
    serial: u32,
    /// Whether the last ask's answer is still to come.
    waiting: bool,
    /// How far the rows are scrolled, in pixels.
    scroll: f32,
    drag: Option<(i32, f32, bool)>,
    pressed: Option<Press>,
    closing: bool,
    /// The words typed for a search.
    words: String,
    cache: Option<RowCache>,
    covers: Covers,
    /// Lucide's glyphs rastered by name, side and ink.
    glyphs: Glyphs,
}

/// How far a finger moves before a press is a drag, not a tap.
const DRAG_PX: i32 = 8;

impl BrowseView {
    /// Open on the sources: the ask for them goes to the player.
    pub fn open_browse(&mut self) -> Command {
        self.open = true;
        self.pages = vec![Page {
            uri: String::new(),
            title: "Browse".to_string(),
            scroll: 0.0,
            keys: false,
        }];
        self.reset_touch();
        self.ask(String::new(), "Browse".to_string())
    }

    /// Open on the keyboard, the words kept from the last search.
    pub fn open_search(&mut self) {
        self.open = true;
        self.pages = vec![Page {
            uri: "search".to_string(),
            title: "Search".to_string(),
            scroll: 0.0,
            keys: true,
        }];
        self.waiting = false;
        self.reset_touch();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.reset_touch();
    }

    fn reset_touch(&mut self) {
        self.scroll = 0.0;
        self.drag = None;
        self.pressed = None;
        self.closing = false;
    }

    /// The ask for a location, under a new serial.
    fn ask(&mut self, uri: String, title: String) -> Command {
        self.serial = self.serial.wrapping_add(1);
        self.waiting = true;
        Command::with(
            "browse",
            serde_json::json!({ "uri": uri, "serial": self.serial, "title": title }),
        )
    }

    fn page(&self) -> Option<&Page> {
        self.pages.last()
    }

    /// Whether the page on show is the keyboard.
    fn on_keys(&self) -> bool {
        self.page().is_some_and(|p| p.keys)
    }

    /// Whether the page on show is the sources, drawn as tiles.
    fn on_sources(&self) -> bool {
        self.page().is_some_and(|p| p.uri.is_empty())
    }

    /// The player's answer to the page on show, where it has come.
    fn answer<'a>(&self, input: &'a Input) -> Option<&'a Browse> {
        input
            .browse
            .as_deref()
            .filter(|b| b.serial == self.serial && !self.on_keys())
    }

    /// What the browser draws from, for the face's stamp.
    pub fn stamp(&self, input: &Input, h: &mut impl Hasher) {
        self.open.hash(h);
        if !self.open {
            return;
        }
        self.serial.hash(h);
        self.waiting.hash(h);
        (self.scroll as i32).hash(h);
        self.pressed.hash(h);
        self.pages.len().hash(h);
        self.words.hash(h);
        input
            .browse
            .as_ref()
            .map(|b| (std::sync::Arc::as_ptr(b) as usize, b.serial))
            .hash(h);
        self.covers.ready().hash(h);
    }

    /// The columns and the cell height of the page's grid: one column of
    /// rows for a list, tiles across for the sources.
    fn grid(&self, panel: &Panel, m: &Measure) -> (usize, u32) {
        if self.on_sources() {
            let cols = (panel.w / (m.cell * 2)).clamp(2, 4) as usize;
            (cols, m.cell)
        } else {
            (1, m.row)
        }
    }

    /// The rows' window below the header, and the furthest they scroll.
    fn extent(
        &self,
        panel: &Panel,
        m: &Measure,
        count: usize,
        cols: usize,
        cell: u32,
    ) -> (u32, f32) {
        let window = panel.h.saturating_sub(m.header);
        let rows = count.div_ceil(cols.max(1));
        let all = rows as f32 * cell as f32;
        (window, (all - window as f32).max(0.0))
    }

    /// The item under a point, where the items are.
    fn item_at(
        &self,
        panel: &Panel,
        m: &Measure,
        count: usize,
        cols: usize,
        cell: u32,
        x: i32,
        y: i32,
    ) -> Option<usize> {
        let top = panel.y + m.header as i32;
        if y < top || y >= panel.y + panel.h as i32 || x < panel.x {
            return None;
        }
        let row = ((y - top) as f32 + self.scroll) / cell as f32;
        if row < 0.0 {
            return None;
        }
        let col = ((x - panel.x) as u32 * cols as u32 / panel.w.max(1)) as usize;
        let i = row as usize * cols + col.min(cols - 1);
        (i < count).then_some(i)
    }

    /// Whether a point is on the way out at the header's right end, which
    /// is there on every page below the first.
    fn on_close(&self, panel: &Panel, m: &Measure, x: i32, y: i32) -> bool {
        self.pages.len() > 1
            && y >= panel.y
            && y < panel.y + m.header as i32
            && x >= panel.x + panel.w as i32 - m.header as i32
            && x < panel.x + panel.w as i32
    }

    /// Whether a point is on a row's plus, at the row's right end.
    fn on_add(&self, panel: &Panel, m: &Measure, x: i32) -> bool {
        !self.on_sources() && x >= panel.x + panel.w as i32 - m.row as i32 - m.pad as i32 / 2
    }

    /// Whether a point is on a row's add-and-play symbol, the square
    /// before the plus.
    fn on_play(&self, panel: &Panel, m: &Measure, x: i32) -> bool {
        let plus = panel.x + panel.w as i32 - m.row as i32 - m.pad as i32 / 2;
        !self.on_sources() && x >= plus - m.row as i32 && x < plus
    }

    /// Whether a point is on a row's clear-and-play symbol, the square
    /// before the add-and-play one.
    fn on_replace(&self, panel: &Panel, m: &Measure, x: i32) -> bool {
        let play = panel.x + panel.w as i32 - 2 * m.row as i32 - m.pad as i32 / 2;
        !self.on_sources() && x >= play - m.row as i32 && x < play
    }

    /// The key under a point on the keyboard.
    fn key_at(&self, panel: &Panel, m: &Measure, x: i32, y: i32) -> Option<usize> {
        let (top, key_h) = self.keys_top(panel, m);
        if y < top || x < panel.x {
            return None;
        }
        let row = ((y - top) / key_h as i32) as usize;
        if row > 4 {
            return None;
        }
        let tenth = (x - panel.x) as f32 * 10.0 / panel.w.max(1) as f32;
        (0..KEYS).find(|&i| {
            let (kx, kw, kr) = key_place(i);
            kr == row && tenth >= kx && tenth < kx + kw
        })
    }

    /// Where the keys begin below the header and the words, and a key's height.
    fn keys_top(&self, panel: &Panel, m: &Measure) -> (i32, u32) {
        let top = panel.y + (m.header + m.row) as i32;
        let room = panel.h.saturating_sub(m.header + m.row);
        (top, (room / 5).clamp(m.small, m.row + m.small))
    }

    /// A touch on the browser, in picture pixels, with the panel it stands
    /// in. Every touch inside is the browser's; a touch outside a sheet or
    /// a drawer shuts it.
    pub fn pointer(
        &mut self,
        kind: PointerKind,
        x: i32,
        y: i32,
        panel: &Panel,
        m: &Measure,
        input: &Input,
    ) -> Action {
        let inside = panel.contains(x, y);
        let on_back =
            inside && y < panel.y + m.header as i32 && x < panel.x + (m.header * 2) as i32;
        let count = self.answer(input).map_or(0, |b| b.items.len());
        let (cols, cell) = self.grid(panel, m);
        match kind {
            PointerKind::Down => {
                if !inside {
                    self.closing = true;
                    return Action::Taken;
                }
                if on_back {
                    self.pressed = Some(Press::Back);
                    return Action::Taken;
                }
                if self.on_close(panel, m, x, y) {
                    self.pressed = Some(Press::Close);
                    return Action::Taken;
                }
                if self.on_keys() {
                    self.pressed = self.key_at(panel, m, x, y).map(Press::Key);
                    return Action::Taken;
                }
                self.drag = Some((y, self.scroll, false));
                self.pressed = self.item_at(panel, m, count, cols, cell, x, y).map(|i| {
                    if self.on_add(panel, m, x) {
                        Press::Add(i)
                    } else if self.on_play(panel, m, x) {
                        Press::Play(i)
                    } else if self.on_replace(panel, m, x) {
                        Press::Replace(i)
                    } else {
                        Press::Row(i)
                    }
                });
                Action::Taken
            }
            PointerKind::Move => {
                if let Some((y0, scroll0, moved)) = self.drag {
                    let dy = y - y0;
                    let moved = moved || dy.abs() > DRAG_PX;
                    if moved {
                        let (_, most) = self.extent(panel, m, count, cols, cell);
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
                let still = drag.is_none_or(|(_, _, moved)| !moved);
                match pressed {
                    Some(Press::Back) if on_back => self.back(),
                    Some(Press::Close) if self.on_close(panel, m, x, y) => {
                        self.close();
                        Action::Close
                    }
                    Some(Press::Key(k)) if self.key_at(panel, m, x, y) == Some(k) => self.key(k),
                    Some(Press::Row(i))
                        if still && self.item_at(panel, m, count, cols, cell, x, y) == Some(i) =>
                    {
                        self.tap(i, input)
                    }
                    Some(Press::Add(i))
                        if still && self.item_at(panel, m, count, cols, cell, x, y) == Some(i) =>
                    {
                        self.add(i, input)
                    }
                    Some(Press::Play(i))
                        if still && self.item_at(panel, m, count, cols, cell, x, y) == Some(i) =>
                    {
                        self.play_added(i, input)
                    }
                    Some(Press::Replace(i))
                        if still && self.item_at(panel, m, count, cols, cell, x, y) == Some(i) =>
                    {
                        self.play_replacing(i, input)
                    }
                    _ => Action::Taken,
                }
            }
        }
    }

    /// The way back: up a page, asking for it again, or out at the first.
    fn back(&mut self) -> Action {
        if self.pages.len() <= 1 {
            self.close();
            return Action::Close;
        }
        self.pages.pop();
        self.reset_touch();
        let page = self.pages.last().cloned().expect("a page left");
        if page.keys {
            self.waiting = false;
            return Action::Taken;
        }
        let command = self.ask(page.uri.clone(), page.title.clone());
        Action::Send(command)
    }

    /// A tap on an item: a folder or a source opens, a track plays its
    /// list from there, a heading is nothing.
    fn tap(&mut self, i: usize, input: &Input) -> Action {
        let Some(answer) = self.answer(input) else {
            return Action::Taken;
        };
        let Some(item) = answer.items.get(i) else {
            return Action::Taken;
        };
        if item.heading {
            return Action::Taken;
        }
        if item.leaf {
            let uri = answer.uri.clone();
            return Action::Send(Command::with(
                "play",
                serde_json::json!({ "uri": uri, "index": i }),
            ));
        }
        let (uri, title) = (item.uri.clone(), item.title.clone());
        if let Some(page) = self.pages.last_mut() {
            page.scroll = self.scroll;
        }
        self.pages.push(Page {
            uri: uri.clone(),
            title: title.clone(),
            scroll: 0.0,
            keys: false,
        });
        self.reset_touch();
        let command = self.ask(uri, title);
        Action::Send(command)
    }

    /// The clear-and-play symbol on an item: the queue emptied, then a
    /// track's list played from the track, or a collection played whole,
    /// as the player's own interface does it.
    fn play_replacing(&mut self, i: usize, input: &Input) -> Action {
        let Some(answer) = self.answer(input) else {
            return Action::Taken;
        };
        let Some(item) = answer.items.get(i) else {
            return Action::Taken;
        };
        if item.heading || item.uri.is_empty() {
            return Action::Taken;
        }
        if item.leaf {
            return Action::Send(Command::with(
                "play",
                serde_json::json!({ "uri": answer.uri, "index": i }),
            ));
        }
        Action::Send(Command::with(
            "play",
            serde_json::json!({ "item": {
                "uri": item.uri, "service": item.service, "title": item.title,
                "type": item.kind, "albumart": item.art,
            } }),
        ))
    }

    /// The add-and-play symbol on an item: the queue kept, a track's list
    /// goes in after it and plays from the track; a folder goes in whole
    /// and plays from its first.
    fn play_added(&mut self, i: usize, input: &Input) -> Action {
        let Some(answer) = self.answer(input) else {
            return Action::Taken;
        };
        let Some(item) = answer.items.get(i) else {
            return Action::Taken;
        };
        if item.heading || item.uri.is_empty() {
            return Action::Taken;
        }
        if item.leaf {
            return Action::Send(Command::with(
                "play",
                serde_json::json!({ "uri": answer.uri, "index": i, "add": true }),
            ));
        }
        Action::Send(Command::with(
            "play",
            serde_json::json!({ "item": {
                "uri": item.uri, "service": item.service, "title": item.title,
                "type": item.kind, "albumart": item.art,
            }, "add": true }),
        ))
    }

    /// The plus on an item: added to the queue, a folder whole.
    fn add(&mut self, i: usize, input: &Input) -> Action {
        let Some(item) = self.answer(input).and_then(|b| b.items.get(i)) else {
            return Action::Taken;
        };
        if item.heading || item.uri.is_empty() {
            return Action::Taken;
        }
        Action::Send(Command::with(
            "queue",
            serde_json::json!({ "item": {
                "uri": item.uri, "service": item.service, "title": item.title,
                "type": item.kind, "albumart": item.art,
            } }),
        ))
    }

    /// A key: a letter typed, one taken back, a space, or go.
    fn key(&mut self, k: usize) -> Action {
        match k {
            KEY_BACK => {
                self.words.pop();
            }
            KEY_SPACE => {
                if !self.words.is_empty()
                    && !self.words.ends_with(' ')
                    && self.words.len() < WORDS_MOST
                {
                    self.words.push(' ');
                }
            }
            KEY_GO => {
                let words = self.words.trim().to_string();
                if words.is_empty() {
                    return Action::Taken;
                }
                if let Some(page) = self.pages.last_mut() {
                    page.scroll = 0.0;
                }
                self.pages.push(Page {
                    uri: format!("search/{words}"),
                    title: "Search".to_string(),
                    scroll: 0.0,
                    keys: false,
                });
                self.reset_touch();
                self.serial = self.serial.wrapping_add(1);
                self.waiting = true;
                return Action::Send(Command::with(
                    "search",
                    serde_json::json!({ "value": words, "serial": self.serial }),
                ));
            }
            _ => {
                if let Some(letter) = key_letter(k) {
                    if self.words.len() < WORDS_MOST {
                        self.words.push(letter);
                    }
                }
            }
        }
        Action::Taken
    }

    /// Lucide's glyph by name at a side in an ink, rastered once; an
    /// unknown name is a folder.
    fn glyph(&mut self, name: &str, size: u32, ink: [u8; 3]) -> &Frame {
        self.glyphs.get(name, size, ink)
    }

    /// The lines of a row, set when first drawn.
    fn row_lines(
        &mut self,
        fonts: &Fonts,
        answer: &Browse,
        i: usize,
        m: &Measure,
        width: u32,
        ink: [u8; 3],
    ) {
        let stale = self.cache.as_ref().is_none_or(|c| {
            c.serial != answer.serial || c.size != m.row || c.width != width || c.ink != ink
        });
        if stale {
            self.cache = Some(RowCache {
                serial: answer.serial,
                size: m.row,
                width,
                ink,
                rows: (0..answer.items.len()).map(|_| None).collect(),
            });
        }
        let cache = self.cache.as_mut().expect("set just now");
        let Some(slot) = cache.rows.get_mut(i) else {
            return;
        };
        if slot.is_some() {
            return;
        }
        let item = &answer.items[i];
        let dim = dimmed(ink);
        // The cover or glyph at the left, the plus and a length at the right.
        let text_w =
            width.saturating_sub(m.pad * 2 + m.cover + m.pad / 2 + m.row * 3 + m.title * 3);
        *slot = Some(if item.heading {
            RowLines {
                title: fitted(
                    fonts,
                    TextStyle::Bold,
                    m.small,
                    dim,
                    &item.title.to_uppercase(),
                    text_w,
                ),
                subtitle: None,
                duration: None,
            }
        } else {
            RowLines {
                title: fitted(fonts, TextStyle::Bold, m.title, ink, &item.title, text_w),
                subtitle: fitted(
                    fonts,
                    TextStyle::Regular,
                    m.small,
                    dim,
                    &item.subtitle,
                    text_w,
                ),
                duration: (item.duration > 0.0)
                    .then(|| {
                        ui::line(
                            fonts,
                            TextStyle::Regular,
                            m.small,
                            dim,
                            &clock(item.duration),
                        )
                    })
                    .flatten(),
            }
        });
    }

    /// Draw the browser in its panel: the header with the way back or
    /// out, the page's word and the count; the sources as tiles, a list
    /// as rows with a cover or a glyph, a plus at the right and a chevron
    /// for what opens; the keyboard for a search. The panel's glass is
    /// the caller's, drawn before this.
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
        back: Option<&Frame>,
        alpha: u8,
    ) {
        let dim = dimmed(ink);
        // An answer that came for this page: the scroll it was left at.
        if self.waiting && self.answer(input).is_some() {
            self.waiting = false;
            self.scroll = self.page().map_or(0.0, |p| p.scroll);
        }
        // The header.
        let hy = panel.y + (m.header as i32 - m.title as i32 * 13 / 10) / 2;
        let mut title_x = panel.x + m.pad as i32;
        let glyph = if self.pages.len() > 1 { back } else { leave };
        if let Some(glyph) = glyph {
            let (lx, ly, lw, lh) = panel.leave_rect(m);
            if self.pressed == Some(Press::Back) {
                ui::fill(frame, lx, ly, lw, lh, [255, 255, 255, alpha / 10]);
            }
            ui::blit(
                frame,
                glyph,
                lx + (lw as i32 - glyph.width as i32) / 2,
                ly + (lh as i32 - glyph.height as i32) / 2,
                alpha,
            );
            title_x = lx + lw as i32 + m.pad as i32 / 4;
        }
        let title = self.page().map_or("", |p| p.title.as_str()).to_string();
        let title_w = panel
            .w
            .saturating_sub((title_x - panel.x) as u32 + m.pad * 6);
        if let Some(line) = fitted(
            fonts,
            TextStyle::Bold,
            m.title * 13 / 10,
            ink,
            &title,
            title_w,
        ) {
            ui::blit(frame, &line, title_x, hy, alpha);
        }
        let count = if self.on_keys() {
            String::new()
        } else if self.waiting {
            "…".to_string()
        } else {
            match self.answer(input) {
                Some(b) if b.items.is_empty() => "nothing here".to_string(),
                Some(b) => format!("{} items", b.items.iter().filter(|i| !i.heading).count()),
                None => String::new(),
            }
        };
        // The way out at the right end on every page below the first: one
        // tap leaves from any depth, where back goes up one page.
        let mut right = panel.x + panel.w as i32 - m.pad as i32;
        if self.pages.len() > 1 {
            let (cx, cy, side) = (
                panel.x + panel.w as i32 - m.header as i32,
                panel.y,
                m.header,
            );
            if self.pressed == Some(Press::Close) {
                ui::fill(frame, cx, cy, side, side, [255, 255, 255, alpha / 10]);
            }
            let glyph = self
                .glyph("chevron-down", m.title + m.small / 2, ink)
                .clone();
            ui::blit(
                frame,
                &glyph,
                cx + (side as i32 - glyph.width as i32) / 2,
                cy + (side as i32 - glyph.height as i32) / 2,
                alpha,
            );
            right = cx - m.pad as i32 / 4;
        }
        if let Some(line) = ui::line(fonts, TextStyle::Regular, m.small, dim, &count) {
            let x = right - line.width as i32;
            let y = panel.y + (m.header as i32 - line.height as i32) / 2;
            ui::blit(frame, &line, x, y, alpha);
        }
        if self.on_keys() {
            self.draw_keys(frame, fonts, panel, m, ink, alpha);
            return;
        }
        let Some(answer) = input
            .browse
            .as_ref()
            .filter(|b| b.serial == self.serial)
            .cloned()
        else {
            return;
        };
        let clip = (
            panel.x,
            panel.y + m.header as i32,
            panel.w,
            panel.h.saturating_sub(m.header),
        );
        let (cols, cell) = self.grid(panel, m);
        let (window, _) = self.extent(panel, m, answer.items.len(), cols, cell);
        let scroll = self.scroll;
        let first_row = (scroll / cell as f32).floor().max(0.0) as usize;
        let last_row = ((scroll + window as f32) / cell as f32).ceil() as usize;
        let col_w = panel.w / cols as u32;
        let pressed = self.pressed;
        for i in (first_row * cols)..(last_row * cols).min(answer.items.len()) {
            let item = &answer.items[i];
            let (row, col) = (i / cols, i % cols);
            let x = panel.x + (col as u32 * col_w) as i32;
            let y = clip.1 + (row as f32 * cell as f32 - scroll).round() as i32;
            let w = if col + 1 == cols {
                panel.w - col as u32 * col_w
            } else {
                col_w
            };
            if cols > 1 {
                let lit = pressed == Some(Press::Row(i));
                self.draw_tile(
                    frame,
                    fonts,
                    item,
                    (x, y, w, cell),
                    m,
                    ink,
                    alpha,
                    lit,
                    clip,
                );
            } else {
                if pressed == Some(Press::Row(i)) {
                    fill_clipped(frame, x, y, w, cell, [255, 255, 255, alpha / 10], clip);
                }
                self.row_lines(fonts, &answer, i, m, panel.w, ink);
                self.draw_row(frame, item, i, (x, y, w, cell), m, ink, accent, alpha, clip);
            }
        }
        // Where the rows stand, as a thin bar at the right.
        let rows = answer.items.len().div_ceil(cols);
        let all = rows as f32 * cell as f32;
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

    /// A source as a tile of lighter glass with rounded corners, its own
    /// picture or its glyph at the top left and its name at the bottom
    /// left, as the accepted mockup has it; lit while pressed.
    #[allow(clippy::too_many_arguments)]
    fn draw_tile(
        &mut self,
        frame: &mut Frame,
        fonts: &Fonts,
        item: &BrowseItem,
        rect: (i32, i32, u32, u32),
        m: &Measure,
        ink: [u8; 3],
        alpha: u8,
        lit: bool,
        clip: (i32, i32, u32, u32),
    ) {
        let (x, y, w, h) = rect;
        let gap = (m.pad / 2) as i32;
        let (tx, ty) = (x + gap, y + gap);
        let (tw, th) = (
            w.saturating_sub(2 * gap as u32).max(1),
            h.saturating_sub(2 * gap as u32).max(1),
        );
        let wash = if lit { alpha / 5 } else { alpha / 12 };
        rounded(
            frame,
            tx,
            ty,
            tw,
            th,
            m.pad / 2,
            [255, 255, 255, wash],
            clip,
        );
        let inner = m.pad as i32;
        let side = m.cover;
        let (px, py) = (tx + inner, ty + inner);
        let drawn = match self.covers.get(&item.art_file, side) {
            Some(cover) => {
                blit_clipped(frame, cover, px, py, alpha, clip);
                true
            }
            None => false,
        };
        if !drawn {
            let glyph = self.glyph(&item.icon, side, ink);
            blit_clipped(
                frame,
                glyph,
                px + (side as i32 - glyph.width as i32) / 2,
                py + (side as i32 - glyph.height as i32) / 2,
                alpha,
                clip,
            );
        }
        if let Some(line) = fitted(
            fonts,
            TextStyle::Regular,
            m.title,
            ink,
            &item.title,
            tw.saturating_sub(2 * inner as u32),
        ) {
            blit_clipped(
                frame,
                &line,
                tx + inner,
                ty + th as i32 - inner - line.height as i32,
                alpha,
                clip,
            );
        }
    }

    /// One row: the cover or glyph, the title and the line under it, a
    /// length for a track, a chevron for what opens, the plus at the end;
    /// a heading in small capitals with nothing else.
    #[allow(clippy::too_many_arguments)]
    fn draw_row(
        &mut self,
        frame: &mut Frame,
        item: &BrowseItem,
        i: usize,
        rect: (i32, i32, u32, u32),
        m: &Measure,
        ink: [u8; 3],
        accent: [u8; 3],
        alpha: u8,
        clip: (i32, i32, u32, u32),
    ) {
        let (x, y, w, h) = rect;
        let dim = dimmed(ink);
        let pad = m.pad as i32;
        if item.heading {
            if let Some(line) = self
                .cache
                .as_ref()
                .and_then(|c| c.rows[i].as_ref())
                .and_then(|r| r.title.as_ref())
            {
                blit_clipped(
                    frame,
                    line,
                    x + pad,
                    y + h as i32 - line.height as i32 - 4,
                    alpha,
                    clip,
                );
            }
            return;
        }
        // The cover, or the glyph in its place.
        let side = m.cover;
        let cy = y + (h as i32 - side as i32) / 2;
        let drawn = match self.covers.get(&item.art_file, side) {
            Some(cover) => {
                blit_clipped(frame, cover, x + pad, cy, alpha, clip);
                true
            }
            None => false,
        };
        if !drawn {
            let glyph = self.glyph(&item.icon, m.title + m.small / 2, dim);
            let gx = x + pad + (side as i32 - glyph.width as i32) / 2;
            let gy = cy + (side as i32 - glyph.height as i32) / 2;
            blit_clipped(frame, glyph, gx, gy, alpha, clip);
        }
        let x_text = x + pad + side as i32 + pad / 2;
        if let Some(lines) = self.cache.as_ref().and_then(|c| c.rows[i].as_ref()) {
            match (&lines.title, &lines.subtitle) {
                (Some(title), Some(sub)) => {
                    blit_clipped(
                        frame,
                        title,
                        x_text,
                        y + h as i32 / 2 - title.height as i32 - 1,
                        alpha,
                        clip,
                    );
                    blit_clipped(frame, sub, x_text, y + h as i32 / 2 + 2, alpha, clip);
                }
                (Some(title), None) => {
                    blit_clipped(
                        frame,
                        title,
                        x_text,
                        y + (h as i32 - title.height as i32) / 2,
                        alpha,
                        clip,
                    );
                }
                // No title: the line under it stands in its place.
                (None, Some(sub)) => {
                    blit_clipped(
                        frame,
                        sub,
                        x_text,
                        y + (h as i32 - sub.height as i32) / 2,
                        alpha,
                        clip,
                    );
                }
                (None, None) => {}
            }
        }
        // The plus at the right end, lit while pressed.
        let plus_x = x + w as i32 - pad / 2 - m.row as i32;
        if self.pressed == Some(Press::Add(i)) {
            fill_clipped(
                frame,
                plus_x,
                y,
                m.row,
                h,
                [accent[0], accent[1], accent[2], alpha / 4],
                clip,
            );
        }
        let plus = self.glyph("plus", m.title, dim);
        let (pw, ph) = (plus.width as i32, plus.height as i32);
        blit_clipped(
            frame,
            plus,
            plus_x + (m.row as i32 - pw) / 2,
            y + (h as i32 - ph) / 2,
            alpha,
            clip,
        );
        // Add and play before the plus: a play in a circle, lit while pressed.
        let play_x = plus_x - m.row as i32;
        if self.pressed == Some(Press::Play(i)) {
            fill_clipped(
                frame,
                play_x,
                y,
                m.row,
                h,
                [accent[0], accent[1], accent[2], alpha / 4],
                clip,
            );
        }
        let play = self.glyph("circle-play", m.title, dim);
        let (gw, gh) = (play.width as i32, play.height as i32);
        blit_clipped(
            frame,
            play,
            play_x + (m.row as i32 - gw) / 2,
            y + (h as i32 - gh) / 2,
            alpha,
            clip,
        );
        // Clear and play before add and play: a plain play, lit while pressed.
        let replace_x = play_x - m.row as i32;
        if self.pressed == Some(Press::Replace(i)) {
            fill_clipped(
                frame,
                replace_x,
                y,
                m.row,
                h,
                [accent[0], accent[1], accent[2], alpha / 4],
                clip,
            );
        }
        let replace = self.glyph("play", m.title, dim);
        let (rw, rh) = (replace.width as i32, replace.height as i32);
        blit_clipped(
            frame,
            replace,
            replace_x + (m.row as i32 - rw) / 2,
            y + (h as i32 - rh) / 2,
            alpha,
            clip,
        );
        // A length, or a chevron for what opens, before the three.
        let right = replace_x - pad / 2;
        if let Some(duration) = self
            .cache
            .as_ref()
            .and_then(|c| c.rows[i].as_ref())
            .and_then(|r| r.duration.as_ref())
        {
            blit_clipped(
                frame,
                duration,
                right - duration.width as i32,
                y + (h as i32 - duration.height as i32) / 2,
                alpha,
                clip,
            );
        } else if !item.leaf {
            let chevron = self.glyph("chevron-right", m.title, dim);
            blit_clipped(
                frame,
                chevron,
                right - chevron.width as i32,
                y + (h as i32 - chevron.height as i32) / 2,
                alpha,
                clip,
            );
        }
        fill_clipped(
            frame,
            x + pad / 2,
            y + h as i32 - 1,
            w.saturating_sub(m.pad),
            1,
            [255, 255, 255, alpha / 10],
            clip,
        );
    }

    /// The keyboard: the words typed with a caret, four rows of keys, and
    /// backspace, space and go below.
    fn draw_keys(
        &mut self,
        frame: &mut Frame,
        fonts: &Fonts,
        panel: &Panel,
        m: &Measure,
        ink: [u8; 3],
        alpha: u8,
    ) {
        let dim = dimmed(ink);
        let pad = m.pad as i32;
        // The words, or what to do.
        let field_y = panel.y + m.header as i32;
        let (text, colour) = if self.words.is_empty() {
            ("type, then go".to_string(), dim)
        } else {
            (format!("{}|", self.words), ink)
        };
        if let Some(line) = fitted(
            fonts,
            TextStyle::Regular,
            m.title,
            colour,
            &text,
            panel.w.saturating_sub(m.pad * 2),
        ) {
            ui::blit(
                frame,
                &line,
                panel.x + pad,
                field_y + (m.row as i32 - line.height as i32) / 2,
                alpha,
            );
        }
        ui::fill(
            frame,
            panel.x + pad / 2,
            field_y + m.row as i32 - 1,
            panel.w.saturating_sub(m.pad),
            1,
            [255, 255, 255, alpha / 10],
        );
        let (top, key_h) = self.keys_top(panel, m);
        let tenth = panel.w as f32 / 10.0;
        let gap = (2.0 * key_h as f32 / 48.0).round().max(1.0) as i32;
        for k in 0..KEYS {
            let (kx, kw, kr) = key_place(k);
            let x = panel.x + (kx * tenth).round() as i32 + gap;
            let y = top + (kr as u32 * key_h) as i32 + gap;
            let w = ((kw * tenth).round() as i32 - gap * 2).max(1) as u32;
            let h = (key_h as i32 - gap * 2).max(1) as u32;
            let lit = self.pressed == Some(Press::Key(k));
            let wash = if lit { alpha / 4 } else { alpha / 14 };
            ui::fill(frame, x, y, w, h, [255, 255, 255, wash]);
            let cap: Option<Frame> = match k {
                KEY_BACK => Some(self.glyph("chevron-left", m.title, ink).clone()),
                KEY_SPACE => None,
                KEY_GO => Some(self.glyph("search", m.title, ink).clone()),
                _ => key_letter(k).and_then(|c| {
                    ui::line(fonts, TextStyle::Regular, m.title, ink, &c.to_string())
                }),
            };
            if let Some(cap) = cap {
                ui::blit(
                    frame,
                    &cap,
                    x + (w as i32 - cap.width as i32) / 2,
                    y + (h as i32 - cap.height as i32) / 2,
                    alpha,
                );
            }
        }
    }
}

/// A fill with rounded corners of radius `r`, held to a clip box: the
/// rows through the corners are shorter by the circle's inset.
pub(crate) fn rounded(
    frame: &mut Frame,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    r: u32,
    rgba: [u8; 4],
    clip: (i32, i32, u32, u32),
) {
    let r = r.min(w / 2).min(h / 2);
    if r == 0 {
        fill_clipped(frame, x, y, w, h, rgba, clip);
        return;
    }
    let rf = r as f32;
    for row in 0..r {
        let d = rf - row as f32 - 0.5;
        let inset = (rf - (rf * rf - d * d).max(0.0).sqrt()).round() as i32;
        let short = w.saturating_sub(2 * inset as u32);
        fill_clipped(frame, x + inset, y + row as i32, short, 1, rgba, clip);
        fill_clipped(
            frame,
            x + inset,
            y + h as i32 - 1 - row as i32,
            short,
            1,
            rgba,
            clip,
        );
    }
    fill_clipped(
        frame,
        x,
        y + r as i32,
        w,
        h.saturating_sub(2 * r),
        rgba,
        clip,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ListShape;
    use std::sync::Arc;

    fn answer(serial: u32, uri: &str, items: Vec<BrowseItem>) -> Input {
        Input {
            browse: Some(Arc::new(Browse {
                uri: uri.to_string(),
                title: "Page".to_string(),
                serial,
                items,
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    fn item(title: &str, uri: &str, leaf: bool) -> BrowseItem {
        BrowseItem {
            title: title.to_string(),
            uri: uri.to_string(),
            service: "mpd".to_string(),
            kind: if leaf { "song" } else { "folder" }.to_string(),
            leaf,
            ..Default::default()
        }
    }

    fn pair(c: &Command) -> (String, serde_json::Value) {
        (c.name.clone(), c.value.clone().unwrap_or_default())
    }

    /// Opening asks for the sources under a serial; a tap on a source
    /// opens it under the next serial, with its word as the page's; a tap
    /// on a track plays the page's list from its place; the plus adds the
    /// item; the way back asks for the page above again, and out at the
    /// first; an answer under another serial is not the page's.
    #[test]
    fn taps_ask_open_play_add_and_go_back() {
        let mut view = BrowseView::default();
        let ask = view.open_browse();
        assert!(view.open);
        assert_eq!(
            pair(&ask),
            (
                "browse".to_string(),
                serde_json::json!({ "uri": "", "serial": 1, "title": "Browse" })
            )
        );
        let panel = Panel::place(ListShape::Sheet, 1280, 720, 72, 1.0);
        let m = Measure::at(1.0);
        let stale = answer(9, "", vec![item("Old", "old", false)]);
        assert!(
            view.answer(&stale).is_none(),
            "another serial is not this page's"
        );
        let sources = answer(
            1,
            "",
            vec![
                item("Music Library", "music-library", false),
                item("Web Radio", "radio", false),
            ],
        );
        assert_eq!(view.answer(&sources).map(|b| b.items.len()), Some(2));
        // The sources are tiles: the second in the second column.
        let (cols, cell) = view.grid(&panel, &m);
        assert!(cols >= 2);
        let (tx, ty) = (
            panel.x + (panel.w / cols as u32) as i32 * 3 / 2,
            panel.y + m.header as i32 + cell as i32 / 2,
        );
        assert_eq!(
            view.pointer(PointerKind::Down, tx, ty, &panel, &m, &sources),
            Action::Taken
        );
        let Action::Send(opened) = view.pointer(PointerKind::Up, tx, ty, &panel, &m, &sources)
        else {
            panic!("a tap on a source opens it");
        };
        assert_eq!(
            pair(&opened),
            (
                "browse".to_string(),
                serde_json::json!({ "uri": "radio", "serial": 2, "title": "Web Radio" })
            )
        );
        assert_eq!(view.pages.len(), 2);
        assert!(view.waiting);
        // Its answer: a folder and two tracks, as rows.
        let radio = answer(
            2,
            "radio",
            vec![
                item("Top 500", "radio/top500", false),
                item("One", "http://one", true),
                item("Two", "http://two", true),
            ],
        );
        let row_y = |i: i32| panel.y + m.header as i32 + m.row as i32 * i + m.row as i32 / 2;
        assert_eq!(
            view.pointer(PointerKind::Down, 300, row_y(2), &panel, &m, &radio),
            Action::Taken
        );
        let Action::Send(play) = view.pointer(PointerKind::Up, 300, row_y(2), &panel, &m, &radio)
        else {
            panic!("a tap on a track plays");
        };
        assert_eq!(
            pair(&play),
            (
                "play".to_string(),
                serde_json::json!({ "uri": "radio", "index": 2 })
            )
        );
        let plus_x = panel.x + panel.w as i32 - m.pad as i32 / 2 - m.row as i32 / 2;
        assert_eq!(
            view.pointer(PointerKind::Down, plus_x, row_y(0), &panel, &m, &radio),
            Action::Taken
        );
        let Action::Send(add) = view.pointer(PointerKind::Up, plus_x, row_y(0), &panel, &m, &radio)
        else {
            panic!("the plus adds");
        };
        assert_eq!(add.name, "queue");
        assert_eq!(add.value.as_ref().unwrap()["item"]["uri"], "radio/top500");
        // The symbol before the plus adds and plays: a track its list from
        // there, the queue kept; a folder whole.
        let play_x = plus_x - m.row as i32;
        assert_eq!(
            view.pointer(PointerKind::Down, play_x, row_y(2), &panel, &m, &radio),
            Action::Taken
        );
        let Action::Send(added) =
            view.pointer(PointerKind::Up, play_x, row_y(2), &panel, &m, &radio)
        else {
            panic!("the symbol adds and plays");
        };
        assert_eq!(
            pair(&added),
            (
                "play".to_string(),
                serde_json::json!({ "uri": "radio", "index": 2, "add": true })
            )
        );
        assert_eq!(
            view.pointer(PointerKind::Down, play_x, row_y(0), &panel, &m, &radio),
            Action::Taken
        );
        let Action::Send(whole) =
            view.pointer(PointerKind::Up, play_x, row_y(0), &panel, &m, &radio)
        else {
            panic!("a folder whole");
        };
        assert_eq!(whole.name, "play");
        assert_eq!(whole.value.as_ref().unwrap()["add"], true);
        assert_eq!(whole.value.as_ref().unwrap()["item"]["uri"], "radio/top500");
        // The symbol before that clears and plays: a track its list from
        // there, a collection whole, the queue emptied by the player.
        let replace_x = play_x - m.row as i32;
        assert_eq!(
            view.pointer(PointerKind::Down, replace_x, row_y(1), &panel, &m, &radio),
            Action::Taken
        );
        let Action::Send(cleared) =
            view.pointer(PointerKind::Up, replace_x, row_y(1), &panel, &m, &radio)
        else {
            panic!("the symbol clears and plays");
        };
        assert_eq!(
            pair(&cleared),
            (
                "play".to_string(),
                serde_json::json!({ "uri": "radio", "index": 1 })
            )
        );
        assert_eq!(
            view.pointer(PointerKind::Down, replace_x, row_y(0), &panel, &m, &radio),
            Action::Taken
        );
        let Action::Send(collection) =
            view.pointer(PointerKind::Up, replace_x, row_y(0), &panel, &m, &radio)
        else {
            panic!("a collection whole");
        };
        assert_eq!(
            collection.value.as_ref().unwrap()["item"]["uri"],
            "radio/top500"
        );
        assert!(
            collection.value.as_ref().unwrap().get("add").is_none(),
            "no add: the queue is emptied"
        );
        // A drag scrolls and plays nothing.
        assert_eq!(
            view.pointer(PointerKind::Down, 300, row_y(1), &panel, &m, &radio),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Move, 300, row_y(1) - 40, &panel, &m, &radio),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, 300, row_y(1) - 40, &panel, &m, &radio),
            Action::Taken
        );
        // Back asks for the sources again under a new serial.
        let (bx, by) = (panel.x + m.header as i32 / 2, panel.y + m.header as i32 / 2);
        assert_eq!(
            view.pointer(PointerKind::Down, bx, by, &panel, &m, &radio),
            Action::Taken
        );
        let Action::Send(again) = view.pointer(PointerKind::Up, bx, by, &panel, &m, &radio) else {
            panic!("back asks again");
        };
        assert_eq!(pair(&again).1["uri"], "");
        assert_eq!(pair(&again).1["serial"], 3);
        assert_eq!(view.pages.len(), 1);
        // Down a page again, the way out at the right end shuts from there;
        // on the first page there is none, the left glyph being the way out.
        assert!(!view.on_close(&panel, &m, panel.x + panel.w as i32 - 10, panel.y + 10));
        let sources = answer(
            3,
            "",
            vec![
                item("Music Library", "music-library", false),
                item("Web Radio", "radio", false),
            ],
        );
        assert_eq!(
            view.pointer(PointerKind::Down, tx, ty, &panel, &m, &sources),
            Action::Taken
        );
        assert!(matches!(
            view.pointer(PointerKind::Up, tx, ty, &panel, &m, &sources),
            Action::Send(_)
        ));
        assert_eq!(view.pages.len(), 2);
        let (cx, cy) = (panel.x + panel.w as i32 - 10, panel.y + 10);
        assert!(view.on_close(&panel, &m, cx, cy));
        assert_eq!(
            view.pointer(PointerKind::Down, cx, cy, &panel, &m, &sources),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, cx, cy, &panel, &m, &sources),
            Action::Close
        );
        assert!(!view.open, "one tap leaves from any depth");
        view.open_browse();
        // At the first page the way out shuts; so does a tap beside the sheet.
        assert_eq!(
            view.pointer(PointerKind::Down, bx, by, &panel, &m, &radio),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, bx, by, &panel, &m, &radio),
            Action::Close
        );
        assert!(!view.open);
        view.open_browse();
        assert_eq!(
            view.pointer(PointerKind::Down, 100, 100, &panel, &m, &radio),
            Action::Taken
        );
        assert_eq!(
            view.pointer(PointerKind::Up, 100, 100, &panel, &m, &radio),
            Action::Close
        );
    }

    /// The keyboard: letters typed, a space between words, one taken
    /// back, and go asking the player with the words under a new serial;
    /// go with nothing typed does nothing; back from the answer returns to
    /// the keys with the words kept.
    #[test]
    fn the_keyboard_types_and_go_searches() {
        let mut view = BrowseView::default();
        view.open_search();
        assert!(view.open && view.on_keys());
        let panel = Panel::place(ListShape::Screen, 1280, 720, 72, 1.0);
        let m = Measure::at(1.0);
        let input = Input::default();
        let press = |view: &mut BrowseView, k: usize| {
            let (kx, kw, kr) = key_place(k);
            let (top, key_h) = view.keys_top(&panel, &m);
            let x = panel.x + ((kx + kw / 2.0) * panel.w as f32 / 10.0) as i32;
            let y = top + (kr as u32 * key_h) as i32 + key_h as i32 / 2;
            assert_eq!(view.key_at(&panel, &m, x, y), Some(k), "key {k} at {x},{y}");
            view.pointer(PointerKind::Down, x, y, &panel, &m, &input);
            view.pointer(PointerKind::Up, x, y, &panel, &m, &input)
        };
        assert_eq!(
            press(&mut view, KEY_GO),
            Action::Taken,
            "nothing typed, nothing asked"
        );
        let d = KEY_ROWS[0].len() + KEY_ROWS[1].len() + 2; // 'd'
        let i = KEY_ROWS[0].len() + 7; // 'i'
        let r = KEY_ROWS[0].len() + 3; // 'r'
        let e = KEY_ROWS[0].len() + 2; // 'e'
        for k in [d, i, r, e, e] {
            press(&mut view, k);
        }
        press(&mut view, KEY_BACK);
        press(&mut view, KEY_SPACE);
        press(&mut view, KEY_SPACE);
        assert_eq!(view.words, "dire ");
        let Action::Send(search) = press(&mut view, KEY_GO) else {
            panic!("go searches");
        };
        assert_eq!(
            pair(&search),
            (
                "search".to_string(),
                serde_json::json!({ "value": "dire", "serial": 1 })
            )
        );
        assert!(!view.on_keys() && view.waiting);
        let found = answer(
            1,
            "search/dire",
            vec![item("Dire Straits", "artists://Dire", false)],
        );
        assert_eq!(view.answer(&found).map(|b| b.items.len()), Some(1));
        let (bx, by) = (panel.x + m.header as i32 / 2, panel.y + m.header as i32 / 2);
        view.pointer(PointerKind::Down, bx, by, &panel, &m, &found);
        assert_eq!(
            view.pointer(PointerKind::Up, bx, by, &panel, &m, &found),
            Action::Taken
        );
        assert!(view.on_keys(), "back from the answer is the keys again");
        assert_eq!(view.words, "dire ");
    }

    /// A glyph by name, an unknown name the folder's; the keys' places
    /// cover the ten tenths of a row.
    #[test]
    fn glyphs_and_keys_have_their_places() {
        let mut view = BrowseView::default();
        let heart = view.glyph("heart", 24, [255, 255, 255]).clone();
        let folder = view.glyph("no-such-icon", 24, [255, 255, 255]).clone();
        let folder_too = view.glyph("folder", 24, [255, 255, 255]).clone();
        assert_eq!(folder.rgba, folder_too.rgba);
        assert_ne!(heart.rgba, folder.rgba);
        assert_eq!(view.glyphs.len(), 2, "rastered once a name");
        for row in 0..5 {
            let covered: f32 = (0..KEYS)
                .map(key_place)
                .filter(|p| p.2 == row)
                .map(|p| p.1)
                .sum();
            assert!(
                (covered
                    - if row == 2 {
                        9.0
                    } else if row == 3 {
                        7.0
                    } else {
                        10.0
                    })
                .abs()
                    < 0.01,
                "row {row}"
            );
        }
        assert_eq!(key_letter(0), Some('1'));
        assert_eq!(key_letter(35), Some('m'));
        assert_eq!(key_letter(36), None);
    }
}
