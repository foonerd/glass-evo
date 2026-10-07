//! The face: what glass-evo draws over Glass's display and does with the
//! touches the theme's controls do not take. Playing shows the theme and
//! nothing else; a touch brings the bar of controls (previous, play or
//! pause, next, volume down and up, more), which leaves by itself; stopped
//! or paused shows the clock and the date, where the theme and the user
//! want them and in the patterns they choose, with the bar over them. More opens a sheet above the
//! bar with repeat, random and mute; a long press on volume down mutes,
//! and while the player is muted that button shows it and unmutes.
//! Everything else the display does, the theme, the meters, the artwork,
//! the never-empty screen, it does as before.
//!
//! What the face draws with comes from a face theme, tokens in a text a
//! theme's author writes, with the user's settings over it; the colours a
//! theme leaves to the artwork are read from the cover, once per track.

use overlay::face::{
    blur, fit_art, host_picture, read_art, read_covering, read_to_string, sky_of, ui, Command,
    Fonts, Frame, Metadata, PointerKind, Sky, TextStyle, Weather,
};
use overlay::{Cover, Overlay, View};

/// What a page's module needs of the face's types to load fonts and hand
/// pictures back: the files a set of fonts comes from, the fonts, a picture.
pub use overlay::face::{
    FontFiles, Fonts as FaceFonts, Frame as FaceFrame, Weather as FaceWeather,
};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

pub mod clock;
pub mod icon;
pub mod look;
pub mod theme;
pub mod when;
use icon::Icon;
use look::Look;
use theme::{Across, Align, Cells, DatePlace, Down, Span, Theme};

pub use overlay::Wall;

/// The clock alone, as a set of the face's keys draws it `size` high at a
/// time of day: for a page that shows a look before it is saved, drawn by
/// what draws it on the screen. The picture of a drawn face; nothing for
/// a clock set in type, which a page sets in its own, or not shown.
pub fn clock_preview<'a>(
    drawn: &'a mut clock::Drawn,
    keys: &BTreeMap<String, String>,
    size: u32,
    wall: &Wall,
    now_ms: u64,
) -> Option<&'a Frame> {
    let theme = Theme::resolve(None, keys);
    if !theme.clock_show || theme.clock_face == clock::ClockKind::Type {
        return None;
    }
    // The second hand of a look that takes its accent from the cover is
    // shown in the accent of no cover.
    let accent = match theme.accent {
        theme::Paint::Fixed(colour) => colour,
        theme::Paint::Artwork => look::NEUTRAL.accent,
    };
    let text = format_time(&theme.clock_format, wall);
    drawn.set(
        &clock::Asked {
            kind: theme.clock_face,
            dial: theme.clock_dial,
            paints: theme.paints(accent),
            text: &text,
            wall,
            seconds: when::shows_seconds(&theme.clock_format),
            now_ms,
        },
        size,
    )?;
    drawn.frame()
}

/// The workspace version, as Cargo knows it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The line the binary prints to say what it is.
/// A line of the idle screen set in type, for a page: the date or the
/// clock in type as a look's keys describe them, `size` pixels high at a
/// moment, in the room its widest shape takes (every digit an 8) with the
/// words in the middle of it, as the face places a line. The fonts are the
/// page's, brought from the player; nothing where they have no bold face,
/// where the piece is not shown, or where the clock is drawn and not set.
pub fn line_preview(
    fonts: &Fonts,
    keys: &BTreeMap<String, String>,
    which: &str,
    size: u32,
    wall: &Wall,
    weather: Option<&Weather>,
) -> Option<Frame> {
    let theme = Theme::resolve(None, keys);
    let (shown, text, ink) = match which {
        "date" => (
            theme.date_show,
            format_time(&theme.date_format, wall),
            date_ink(&theme, weather),
        ),
        "clock" => (
            theme.clock_show && theme.clock_face == clock::ClockKind::Type,
            format_time(&theme.clock_format, wall),
            theme.clock_ink.unwrap_or(theme.ink),
        ),
        _ => return None,
    };
    if !shown || text.is_empty() {
        return None;
    }
    let widest: String = text
        .chars()
        .map(|c| if c.is_ascii_digit() { '8' } else { c })
        .collect();
    let room = ui::line(fonts, TextStyle::Bold, size, ink, &widest)?;
    let words = ui::line(fonts, TextStyle::Bold, size, ink, &text)?;
    let mut frame = Frame {
        blend: Default::default(),
        width: room.width,
        height: room.height,
        rgba: vec![0; (room.width * room.height * 4) as usize],
    };
    ui::blit(
        &mut frame,
        &words,
        (room.width as i32 - words.width as i32) / 2,
        0,
        255,
    );
    Some(frame)
}

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
/// How long a stop that still names a track is taken for the gap between
/// two tracks: the player says "stop" there for a moment, and the plugin
/// waits as long before it believes a stop.
pub const TRACK_CHANGE_MS: u64 = 5000;

/// Whether the player plays, the gap between two tracks bridged. A pause is
/// the listener's own and counts at once, as does a stop that names no
/// track, the queue's end; a stop that still names a track, after playing,
/// counts only once it has lasted.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Playing {
    was: bool,
    stopped_at: Option<u64>,
}

impl Playing {
    /// Every frame: the player's word, and whether the face takes it as
    /// playing.
    pub fn settle(&mut self, now: u64, meta: &Metadata) -> bool {
        if meta.status == "play" {
            self.was = true;
            self.stopped_at = None;
            return true;
        }
        let between = self.was && meta.status == "stop" && !meta.title.is_empty();
        if between {
            let at = *self.stopped_at.get_or_insert(now);
            if now.saturating_sub(at) < TRACK_CHANGE_MS {
                return true;
            }
        }
        self.was = false;
        self.stopped_at = None;
        false
    }
}

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
        Self::measured(width, height, scale, 72.0)
    }

    /// The bar at a theme's measure, in units of a 720th of the picture's
    /// height: 72 as designed.
    pub fn measured(width: u32, height: u32, scale: f32, units: f32) -> Self {
        let h = ((height as f32 / 720.0 * units * scale.max(0.5)).round() as u32)
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

    /// A button's rectangle: x, y, width, height. The last one runs to
    /// the bar's end.
    pub fn button_rect(&self, index: usize) -> (i32, i32, u32, u32) {
        let bw = self.slot_width();
        let x = index as u32 * bw;
        let w = if index + 1 == BUTTONS.len() {
            self.w - x
        } else {
            bw
        };
        (self.x + x as i32, self.y, w, self.h)
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

/// The command a button sends, given the player's state: the volume
/// buttons step from the volume as the player has it, and volume down on
/// a muted player unmutes instead; none for More, which only opens the
/// sheet.
pub fn command_for(button: Button, meta: &Metadata) -> Option<Command> {
    match button {
        Button::Previous => Some(plain("previous")),
        Button::Toggle => Some(plain("toggle")),
        Button::Next => Some(plain("next")),
        Button::VolumeDown if meta.mute => Some(mute_command(true)),
        Button::VolumeDown => Some(Command::with(
            "volume",
            serde_json::json!(meta.volume.saturating_sub(VOLUME_STEP)),
        )),
        Button::VolumeUp => Some(Command::with(
            "volume",
            serde_json::json!((meta.volume + VOLUME_STEP).min(100)),
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

/// The icon a button wears: play or pause as the player stands, and the
/// muted speaker on volume down while the player is muted.
pub fn button_icon(button: Button, meta: &Metadata) -> Icon {
    match button {
        Button::Previous => Icon::Previous,
        Button::Toggle if meta.status == "play" => Icon::Pause,
        Button::Toggle => Icon::Play,
        Button::Next => Icon::Next,
        Button::VolumeDown if meta.mute => Icon::Muted,
        Button::VolumeDown => Icon::Minus,
        Button::VolumeUp => Icon::Plus,
        Button::More => Icon::More,
    }
}

/// The icon a tile wears, saying how its mode stands in the player.
pub fn tile_icon(tile: Tile, meta: &Metadata) -> Icon {
    match tile {
        Tile::Repeat if meta.repeat && meta.repeat_single => Icon::RepeatOne,
        Tile::Repeat => Icon::Repeat,
        Tile::Random => Icon::Shuffle,
        Tile::Mute if meta.mute => Icon::Muted,
        Tile::Mute => Icon::Speaker,
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
/// since when, whether that press already acted as a long one, whether it
/// is the press that shuts the sheet, the icons at the size in use; and
/// what it draws with: the tokens, the look of the track, the icons in the
/// accent for what is lit, and the frost as last made.
#[derive(Default)]
pub struct Face {
    presence: Presence,
    playing: Playing,
    sheet_open: bool,
    pending: Vec<Command>,
    pressed: Option<Button>,
    pressed_tile: Option<Tile>,
    down_at: u64,
    held: bool,
    shutting: bool,
    icons: Option<icon::Set>,
    accents: Option<icon::Set>,
    tokens: Option<Tokens>,
    track: TrackLook,
    frost: Frost,
    clock: ClockFace,
    /// What was last drawn, as `stamp` puts it; none when nothing was.
    drawn: Option<u64>,
    /// The folders face themes are kept in, where the face was told them;
    /// else the launcher's word, `GLASS_FACES`.
    faces: Option<String>,
    /// The picture for when nothing plays, and the folder such pictures
    /// are kept in, where the face was told it; else the launcher's word,
    /// `GLASS_BACKGROUNDS`.
    idle: IdlePicture,
    backgrounds: Option<String>,
    /// When the player last came to stand still and when the screen was
    /// last touched, for the screen's going black; whether it is black
    /// now, and whether the press that wakes it is still down.
    idle_since: Option<u64>,
    touched_at: Option<u64>,
    dark: bool,
    waking: bool,
    /// When the screen began to go black, and when it began to come back,
    /// for the fade between.
    dark_since: Option<u64>,
    wake_since: Option<u64>,
    fade_ms: u32,
}

/// The tokens in use, read on the first frame, with what they were read
/// for: the theme on show and the face's settings. They stand until a
/// view names another theme or other settings.
struct Tokens {
    theme: Theme,
    frosted: bool,
    theme_dir: String,
    settings: BTreeMap<String, String>,
}

/// A theme's text by its folder's name in one of the folders face themes
/// are kept in; a name that is not one folder's is no theme.
fn theme_file(base: &Path, name: &str) -> Option<PathBuf> {
    let name = name.trim();
    if name.is_empty() || name.starts_with('.') || name.contains(['/', '\\']) {
        return None;
    }
    Some(base.join(name).join("face.txt"))
}

/// The folders face themes are kept in, the user's before the ones
/// glass-evo ships, parted by a colon. A theme is the first of its name
/// found, read where the display reads its own files: the file system on a
/// player, the page's file table in a browser.
fn theme_text(folders: &str, name: &str) -> Option<String> {
    folders
        .split(':')
        .filter(|folder| !folder.is_empty())
        .filter_map(|folder| theme_file(Path::new(folder), name))
        .find_map(|file| read_to_string(&file))
}

/// What the theme on show brings for the face: the `face.txt` beside its
/// `meters.txt`, read where the display reads the theme's own files. None
/// where no theme is on show or the theme brings none.
fn brought_text(theme_dir: &str) -> Option<String> {
    if theme_dir.is_empty() {
        return None;
    }
    read_to_string(&Path::new(theme_dir).join("face.txt"))
}

/// The tokens for a view, three texts one over the other: the look its
/// settings name, from the folders the face was told or, on a player, the
/// ones the launcher names in `GLASS_FACES`; over it what the theme on
/// show brings; over both the user's own settings.
fn tokens_for(view: &View, faces: Option<&str>) -> Tokens {
    let chosen = view.settings.get("theme").and_then(|name| {
        let folders = match faces {
            Some(folders) => folders.to_string(),
            None => std::env::var("GLASS_FACES").ok()?,
        };
        theme_text(&folders, name)
    });
    let brought = brought_text(view.theme_dir);
    let theme = Theme::layered(&[chosen.as_deref(), brought.as_deref()], view.settings);
    let frosted = theme.frosted(view.settings);
    Tokens {
        theme,
        frosted,
        theme_dir: view.theme_dir.to_string(),
        settings: view.settings.clone(),
    }
}

impl Tokens {
    /// Whether these are the tokens for a view: read for the theme it has
    /// on show and from the settings it has.
    fn stand_for(&self, view: &View) -> bool {
        self.theme_dir == view.theme_dir && &self.settings == view.settings
    }
}

/// The picture of the user's own for when nothing plays, read for the
/// screen's size and darkened once: on a player a thread reads it and the
/// frames go on with the theme until it is in; in a browser it is read at
/// the frame that asks.
#[derive(Default)]
struct IdlePicture {
    /// The file, the size and the darkening the picture in hand was read for.
    key: Option<(String, u32, u32, u32)>,
    frame: Option<Frame>,
    pending: Option<PictureReading>,
}

#[cfg(not(target_arch = "wasm32"))]
struct PictureReading(std::sync::mpsc::Receiver<Option<Frame>>);
#[cfg(target_arch = "wasm32")]
struct PictureReading(Option<Frame>);

/// A picture file made to cover `w` by `h` and darkened by `dim`.
fn idle_frame(path: &Path, w: u32, h: u32, dim: f32) -> Option<Frame> {
    let mut frame = read_covering(path, w, h)?;
    let keep = ((1.0 - dim.clamp(0.0, 0.9)) * 256.0) as u32;
    for pixel in frame.rgba.as_chunks_mut::<4>().0 {
        for channel in &mut pixel[..3] {
            *channel = ((u32::from(*channel) * keep) >> 8) as u8;
        }
        pixel[3] = 255;
    }
    Some(frame)
}

impl PictureReading {
    #[cfg(not(target_arch = "wasm32"))]
    fn of(path: PathBuf, w: u32, h: u32, dim: f32) -> Self {
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = send.send(idle_frame(&path, w, h, dim));
        });
        PictureReading(receive)
    }

    #[cfg(target_arch = "wasm32")]
    fn of(path: PathBuf, w: u32, h: u32, dim: f32) -> Self {
        PictureReading(idle_frame(&path, w, h, dim))
    }

    /// What was read, once it has been; none where the file is no picture.
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

/// The file of a picture by its name in the folder such pictures are kept
/// in; a name that is not one file's name there is no picture.
fn idle_file(folder: &str, name: &str) -> Option<PathBuf> {
    let name = name.trim();
    if folder.is_empty() || name.is_empty() || name.starts_with('.') || name.contains(['/', '\\']) {
        return None;
    }
    Some(Path::new(folder).join(name))
}

impl IdlePicture {
    /// Follow what is asked: the picture `file` for a screen of `w` by `h`,
    /// darkened by `dim`, or none. Says whether a picture is in hand.
    fn follow(&mut self, file: Option<PathBuf>, w: u32, h: u32, dim: f32) -> bool {
        let Some(file) = file else {
            *self = Self::default();
            return false;
        };
        let key = (
            file.to_string_lossy().into_owned(),
            w,
            h,
            (dim * 1000.0) as u32,
        );
        if self.key.as_ref() != Some(&key) {
            self.key = Some(key);
            self.frame = None;
            self.pending = Some(PictureReading::of(file, w, h, dim));
        }
        if let Some(found) = self.pending.as_mut().and_then(PictureReading::done) {
            self.pending = None;
            self.frame = found;
        }
        self.frame.is_some()
    }
}

/// The look of the track on show: read from the cover off the frame's
/// path, the look before it standing until the new one is in.
struct TrackLook {
    /// The cover's file the look in use was read from, or is being read
    /// from; none before the first frame, so the first always resolves.
    cover: Option<String>,
    look: Look,
    pending: Option<Reading>,
}

/// A cover being read for its colours. On a player a thread reads it and
/// the frames go on; in a browser, where there is no thread to hand it to,
/// it is read at the frame that asks, a sixty-fourth of the cover at most.
#[cfg(not(target_arch = "wasm32"))]
struct Reading(std::sync::mpsc::Receiver<Option<Look>>);
#[cfg(target_arch = "wasm32")]
struct Reading(Option<Look>);

impl Reading {
    #[cfg(not(target_arch = "wasm32"))]
    fn of(path: PathBuf) -> Self {
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = send.send(read_art(&path, 64, 64, None).map(|cover| look::of_cover(&cover)));
        });
        Reading(receive)
    }

    #[cfg(target_arch = "wasm32")]
    fn of(path: PathBuf) -> Self {
        Reading(read_art(&path, 64, 64, None).map(|cover| look::of_cover(&cover)))
    }

    /// What was read, once it has been: the cover's look, or none where
    /// the cover could not be read. Nothing while the reading goes on.
    #[cfg(not(target_arch = "wasm32"))]
    fn done(&mut self) -> Option<Option<Look>> {
        match self.0.try_recv() {
            Ok(found) => Some(found),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(None),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn done(&mut self) -> Option<Option<Look>> {
        Some(self.0.take())
    }
}

impl Default for TrackLook {
    fn default() -> Self {
        Self {
            cover: None,
            look: look::NEUTRAL,
            pending: None,
        }
    }
}

impl TrackLook {
    /// The look for the cover's file as the player has it now. Returns
    /// whether the look changed with this call.
    fn follow(&mut self, theme: &Theme, cover: &str) -> bool {
        let fixed = matches!(
            (theme.tint, theme.accent),
            (theme::Paint::Fixed(_), theme::Paint::Fixed(_))
        );
        if self.cover.as_deref() != Some(cover) {
            self.cover = Some(cover.to_string());
            self.pending = None;
            if cover.is_empty() || fixed {
                let before = self.look;
                self.look = look::resolve(theme, None);
                return self.look != before;
            }
            self.pending = Some(Reading::of(PathBuf::from(cover)));
        }
        if let Some(found) = self.pending.as_mut().and_then(Reading::done) {
            self.pending = None;
            let before = self.look;
            let from_cover = found.unwrap_or(look::NEUTRAL);
            self.look = Look {
                tint: match theme.tint {
                    theme::Paint::Fixed(c) => c,
                    theme::Paint::Artwork => from_cover.tint,
                },
                accent: match theme.accent {
                    theme::Paint::Fixed(c) => c,
                    theme::Paint::Artwork => from_cover.accent,
                },
            };
            return self.look != before;
        }
        false
    }
}

/// Frosted glass over a moving picture: a band of the frame taken at an
/// eighth, blurred there and stretched back. The band as last frosted is
/// kept with a sum of what lay under it, so a picture that stands still
/// under the glass is frosted once.
#[derive(Default)]
struct Frost {
    held: Vec<(Rect, u64, Vec<u8>)>,
}

type Rect = (u32, u32, u32, u32);

fn band_sum(frame: &Frame, (x, y, w, h): Rect) -> u64 {
    let mut sum = 0xcbf29ce484222325u64;
    for row in y..y + h {
        let start = ((row * frame.width + x) * 4) as usize;
        for chunk in frame.rgba[start..start + (w * 4) as usize]
            .as_chunks::<8>()
            .0
        {
            let word = u64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ]);
            sum = (sum ^ word).wrapping_mul(0x100000001b3);
        }
    }
    sum
}

impl Frost {
    /// Frost a rectangle of the frame in place, `alpha` of the way.
    fn apply(&mut self, frame: &mut Frame, x: i32, y: i32, w: u32, h: u32, alpha: u8) {
        // The part of the band that lies on the picture.
        let x1 = (x.saturating_add(w as i32)).clamp(0, frame.width as i32) as u32;
        let y1 = (y.saturating_add(h as i32)).clamp(0, frame.height as i32) as u32;
        let x = x.clamp(0, frame.width as i32) as u32;
        let y = y.clamp(0, frame.height as i32) as u32;
        let (w, h) = (x1.saturating_sub(x), y1.saturating_sub(y));
        if w < 8 || h < 4 || alpha == 0 {
            return;
        }
        let rect = (x, y, w, h);
        let sum = band_sum(frame, rect);
        let at = match self.held.iter().position(|(r, _, _)| *r == rect) {
            Some(at) => at,
            None => {
                // The bar's band, the sheet's, the clock's and the date's: four at most are kept.
                if self.held.len() >= 4 {
                    self.held.remove(0);
                }
                self.held.push((rect, !sum, Vec::new()));
                self.held.len() - 1
            }
        };
        if self.held[at].1 != sum {
            let mut band = Vec::with_capacity((w * h * 4) as usize);
            for row in y..y + h {
                let start = ((row * frame.width + x) * 4) as usize;
                band.extend_from_slice(&frame.rgba[start..start + (w * 4) as usize]);
            }
            let band = Frame {
                blend: Default::default(),
                width: w,
                height: h,
                rgba: band,
            };
            let small = blur(&fit_art(&band, (w / 8).max(4), (h / 8).max(2)), 2);
            self.held[at] = (rect, sum, fit_art(&small, w, h).rgba);
        }
        let frosted = &self.held[at].2;
        let a = alpha as u32;
        for row in 0..h {
            let to = (((y + row) * frame.width + x) * 4) as usize;
            let from = (row * w * 4) as usize;
            let (dst, src) = (
                &mut frame.rgba[to..to + (w * 4) as usize],
                &frosted[from..from + (w * 4) as usize],
            );
            if alpha == 255 {
                dst.copy_from_slice(src);
            } else {
                for (d, s) in dst.iter_mut().zip(src) {
                    *d = ((*s as u32 * a + *d as u32 * (255 - a)) / 255) as u8;
                }
            }
        }
    }
}

impl Face {
    pub fn new() -> Self {
        Self::default()
    }

    /// The face told where its themes are kept: folders parted by a colon,
    /// the user's before the ones that ship. In a browser, where no
    /// launcher names them, the page's file table holds them under these.
    pub fn with_faces(folders: &str) -> Self {
        Self {
            faces: Some(folders.to_string()),
            ..Self::default()
        }
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

fn share(of: f32, alpha: u8) -> u8 {
    (of.clamp(0.0, 1.0) * alpha as f32).round() as u8
}

/// Whether the face says what it sees: the display's own log level, as the
/// launcher hands it over, at its finest.
fn verbose() -> bool {
    std::env::var("GLASS_LOG")
        .map(|v| v.contains("verbose"))
        .unwrap_or(false)
}

/// An icon in the middle of a rectangle.
fn place(frame: &mut Frame, icon: &Frame, rect: (i32, i32, u32, u32), alpha: u8) {
    let (x, y, w, h) = rect;
    ui::blit(
        frame,
        icon,
        x + (w as i32 - icon.width as i32) / 2,
        y + (h as i32 - icon.height as i32) / 2,
        alpha,
    );
}

/// A time set in a pattern, as `strftime` reads it: `%H:%M`, `%-I:%M %p`,
/// `%A %-d %B`. Nothing when the pattern cannot be read or gives nothing.
pub fn format_time(pattern: &str, wall: &Wall) -> String {
    when::format(pattern, wall)
}

/// The size a line is set at so that it fits: the size wanted, or less by
/// as much as its room at that size exceeds the room there is; never
/// under twelve pixels.
fn fitted_size(wanted: u32, room: (u32, u32), most: (u32, u32)) -> u32 {
    if room.0 <= most.0 && room.1 <= most.1 {
        return wanted;
    }
    let by = (most.0 as f32 / room.0.max(1) as f32).min(most.1 as f32 / room.1.max(1) as f32);
    ((wanted as f32 * by).floor() as u32).max(12)
}

/// The least room a glass keeps about its words, in units of a 720th of
/// the picture's height: sideways, and above and below. Words larger than
/// fit beside the margins and the room designed take both, down to this.
const LEAST_PAD: (f32, f32) = (6.0, 4.0);

/// The room a glass keeps about words `words` long in a space `space`
/// long: the room designed while that much is left on either side, and
/// less as the words take more of the space, never under the least.
fn pad_about(space: u32, words: u32, designed: u32, least: u32) -> u32 {
    (space.saturating_sub(words) / 2).clamp(least.min(designed), designed)
}

/// A line set in type: rastered when its words, its size or its ink
/// change, not every frame; with the room the widest words of its shape
/// take, every digit an 8, so the glass behind it stands still while the
/// digits change; and set smaller than wanted where it would not fit.
#[derive(Default)]
struct Line {
    set_for: Option<(String, u32, [u8; 3])>,
    frame: Option<Frame>,
    /// The shape, the size wanted and the room there was, the fit was made for.
    fit_for: Option<(String, u32, (u32, u32))>,
    size: u32,
    room: (u32, u32),
}

impl Line {
    /// Set the words at the size wanted, or the largest that fits in
    /// `most`; the room they take, or nothing when there is nothing to
    /// set or no face to set it in.
    fn set(
        &mut self,
        fonts: &Fonts,
        text: String,
        wanted: u32,
        ink: [u8; 3],
        most: (u32, u32),
    ) -> Option<(u32, u32)> {
        if text.is_empty() {
            return None;
        }
        let widest: String = text
            .chars()
            .map(|c| if c.is_ascii_digit() { '8' } else { c })
            .collect();
        if self.fit_for.as_ref().map(|(w, s, m)| (w.as_str(), *s, *m))
            != Some((widest.as_str(), wanted, most))
        {
            let line = ui::line(fonts, TextStyle::Bold, wanted, ink, &widest)?;
            self.size = fitted_size(wanted, (line.width, line.height), most);
            self.room = (line.width, line.height);
            if self.size != wanted {
                let line = ui::line(fonts, TextStyle::Bold, self.size, ink, &widest)?;
                self.room = (line.width, line.height);
            }
            self.fit_for = Some((widest, wanted, most));
        }
        if self.set_for.as_ref().map(|(t, s, i)| (t.as_str(), *s, *i))
            != Some((text.as_str(), self.size, ink))
        {
            self.frame = ui::line(fonts, TextStyle::Bold, self.size, ink, &text);
            self.set_for = Some((text, self.size, ink));
        }
        self.frame.as_ref().map(|_| self.room)
    }

    /// The line in the middle of a width, at an opacity.
    fn place(&self, frame: &mut Frame, x: i32, y: i32, width: u32, opacity: f32) {
        if let Some(line) = self.frame.as_ref() {
            ui::blit(
                frame,
                line,
                x + (width as i32 - line.width as i32) / 2,
                y,
                share(opacity, 255),
            );
        }
    }
}

/// A temperature as a forecast says it: whole degrees.
fn degrees(value: f32) -> String {
    // Minus nought is nought.
    format!("{}\u{b0}", value.round() as i32)
}

/// Today's forecast in a line: how it is now where the player was told
/// (the sky and the temperature), then the day (its sky, its lowest and
/// its highest). The skies are drawn, the figures set in type; all of it
/// rastered when what it says, its size or its ink change, not every frame.
#[derive(Default)]
struct ForecastLine {
    /// Today's figures: the temperature now, the low, the slash, the high.
    now: Line,
    lo: Line,
    slash: Line,
    hi: Line,
    /// The skies' cycles, and which sky stands for now and for the day.
    skies: Skies,
    now_key: Option<SkyKey>,
    day_key: Option<SkyKey>,
    /// Whether the skies move, whether thunder flashes, whether they are
    /// in colour.
    motion: bool,
    flashes: bool,
    colour: bool,
    /// NOW, MIN and MAX, set letter by letter with room between, for a
    /// size and an ink; and how far up into the numbers' descent they sit.
    captions: Option<(u32, [u8; 3], [Frame; 3])>,
    caption_up: u32,
    /// The words, the size wanted and the room there was, the fit was made
    /// for, and the size it came to.
    fit_for: Option<((String, String), u32, (u32, u32), u32)>,
    has_now: bool,
    /// A span's columns, each a label over a sky over a figure; what they
    /// were fitted for (the span, their words, the size wanted, the room)
    /// and the size they came to; and their shape: the width of a column
    /// and the heights of the label and the figure rows.
    columns: Vec<Column>,
    columns_for: Option<(Span, Vec<Entry>, u32, (u32, u32), u32)>,
    column_width: u32,
    label_height: u32,
    figure_height: u32,
    /// Whether columns stand, or the line.
    spanned: bool,
}

/// One column of a span: its label (the hour, or the weekday), its sky,
/// and its figure (the temperature, or the low and the high).
#[derive(Default)]
struct Column {
    label: Line,
    /// The figure in parts: one for an hour's temperature; the low, the
    /// slash and the high for a day, so each temperature can take its own
    /// colour.
    parts: Vec<Line>,
    sky: Option<SkyKey>,
}

/// A sky as the cache knows it: its kind (as a number, the kind having no
/// hash of its own), day or night, hot or not, heavy or not, its side, in
/// colour or not, and its ink.
type SkyKey = (u8, bool, bool, bool, u32, bool, [u8; 3]);

/// The skies' frames: each cycle rastered once per key and kept while the
/// sky is on show; thunder's two frames, dark and lit.
#[derive(Default)]
struct Skies {
    frames: HashMap<SkyKey, (Sky, Vec<Frame>)>,
}

impl Skies {
    /// The key of a sky at a temperature (hot from 30 °C) and a code
    /// (heavy by it), a side, in colour or not, in an ink.
    fn key(
        sky: Sky,
        day: bool,
        celsius: f32,
        code: u8,
        side: u32,
        colour: bool,
        ink: [u8; 3],
    ) -> SkyKey {
        (
            sky as u8,
            day,
            celsius >= 30.0,
            icon::heavy(code),
            side.max(1),
            colour,
            ink,
        )
    }

    /// Have the cycle for a key rastered.
    fn keep(&mut self, key: SkyKey, sky: Sky) {
        self.frames.entry(key).or_insert_with(|| {
            let (_, day, hot, heavy, side, colour, ink) = key;
            let palette = if colour {
                icon::Palette::colour(ink)
            } else {
                icon::Palette::ink(ink)
            };
            let frames = if sky == Sky::Thunder {
                vec![
                    icon::sky_frame(sky, day, hot, heavy, side, &palette, 0, false),
                    icon::sky_frame(sky, day, hot, heavy, side, &palette, 0, true),
                ]
            } else {
                let count = icon::cycle(sky, day, hot).map_or(1, |(frames, _)| frames);
                (0..count)
                    .map(|frame| {
                        icon::sky_frame(sky, day, hot, heavy, side, &palette, frame, false)
                    })
                    .collect()
            };
            (sky, frames)
        });
    }

    /// Which frame of a key stands at a moment: 0 while the skies do not
    /// move, thunder's lit frame in a flash, else the cycle's.
    fn index(&self, key: SkyKey, t_ms: u64, motion: bool, flashes: bool) -> usize {
        let Some((sky, frames)) = self.frames.get(&key) else {
            return 0;
        };
        if !motion {
            0
        } else if *sky == Sky::Thunder {
            usize::from(flashes && icon::flash_at(t_ms))
        } else {
            (icon::frame_at(*sky, key.1, key.2, t_ms) as usize).min(frames.len().saturating_sub(1))
        }
    }

    /// The frame of a key at a moment.
    fn at(&self, key: SkyKey, t_ms: u64, motion: bool, flashes: bool) -> Option<&Frame> {
        let (_, frames) = self.frames.get(&key)?;
        frames.get(self.index(key, t_ms, motion, flashes))
    }

    /// Let go of every cycle but those in use.
    fn keep_only(&mut self, used: &[SkyKey]) {
        self.frames.retain(|key, _| used.contains(key));
    }
}

/// A temperature in Celsius, from the reading's unit.
fn celsius(temp: f32, unit: &str) -> f32 {
    if unit.eq_ignore_ascii_case("F") {
        (temp - 32.0) * 5.0 / 9.0
    } else {
        temp
    }
}

/// What a column says: its label, its figure in parts (each its words and
/// the temperature it is, none for the slash), the weather's code and
/// whether it is day.
type Entry = (String, Vec<(String, Option<f32>)>, u8, bool);

/// The forecast's proportions, Andrew's choices of 2026-10-07 (T21 and C4).
/// Today's size is its numbers' size, as it always was, so the size's
/// slider reaches as far as every other piece's: the skies, the captions
/// under the numbers and how far the captions sit up into the numbers'
/// descent are shares of that. A span's size is its skies'; its figures
/// and labels are shares of that.
const TODAY_SKY: f32 = 80.0 / 48.0;
const TODAY_CAPTION: f32 = 18.0 / 48.0;
const TODAY_CAPTION_UP: f32 = 6.0 / 48.0;
const FIGURE: f32 = 0.54;
const LABEL: f32 = 0.4;

/// The heatmap: the colour of a temperature, from the cold colour at
/// -10 °C through an ink at 12 to the warm colour at 30, straight between
/// the stops and held beyond them; read in Celsius.
fn heat_colour(celsius: f32, cold: [u8; 3], ink: [u8; 3], warm: [u8; 3]) -> [u8; 3] {
    let mix = |a: [u8; 3], b: [u8; 3], t: f32| -> [u8; 3] {
        let t = t.clamp(0.0, 1.0);
        [0, 1, 2].map(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8)
    };
    if celsius <= 12.0 {
        mix(cold, ink, (celsius + 10.0) / 22.0)
    } else {
        mix(ink, warm, (celsius - 12.0) / 18.0)
    }
}

/// The heatmap as the look sets it for a reading: its ends, the ink in the
/// middle, whether the week's days take it, and the reading's unit.
#[derive(Clone, Copy)]
struct Heat {
    cold: [u8; 3],
    ink: [u8; 3],
    warm: [u8; 3],
    days: bool,
    fahrenheit: bool,
}

impl Heat {
    fn from(theme: &Theme, weather: &Weather) -> Option<Heat> {
        theme.weather_heat.then_some(Heat {
            cold: theme.weather_cold,
            ink: theme.weather_ink.unwrap_or(theme.ink),
            warm: theme.weather_warm,
            days: theme.weather_heat_days,
            fahrenheit: weather.unit.eq_ignore_ascii_case("F"),
        })
    }

    /// The colour of a temperature in the reading's unit.
    fn of(&self, temp: f32) -> [u8; 3] {
        let celsius = if self.fahrenheit {
            (temp - 32.0) * 5.0 / 9.0
        } else {
            temp
        };
        heat_colour(celsius, self.cold, self.ink, self.warm)
    }
}

/// The date's ink: its own, or, where the look's `weather.heat.date` is on
/// and the player holds a reading, the heat colour of the temperature now
/// with the date's own ink as the middle of the scale.
fn date_ink(theme: &Theme, weather: Option<&Weather>) -> [u8; 3] {
    let own = theme.date_ink.unwrap_or(theme.ink);
    match weather.and_then(|w| w.now.map(|now| (now, w))) {
        Some((now, w)) if theme.weather_heat_date => {
            Heat::from(theme, w).map_or(own, |heat| Heat { ink: own, ..heat }.of(now))
        }
        _ => own,
    }
}

/// A share of a size, in whole pixels and never under eight: a size of
/// type or of a sky.
fn part(size: u32, share: f32) -> u32 {
    share_of(size, share).max(8)
}

/// A share of a size, in whole pixels: a distance.
fn share_of(size: u32, share: f32) -> u32 {
    (size as f32 * share).round() as u32
}

/// Whether a clock pattern shows twelve hours: it names the hour of
/// twelve or the half of the day.
fn twelve_hour(pattern: &str) -> bool {
    let plain = pattern
        .replace("%-", "%")
        .replace("%_", "%")
        .replace("%0", "%");
    ["%I", "%l", "%p", "%P"].iter().any(|c| plain.contains(c))
}

/// Words set letter by letter with a third of the size between, as a
/// caption is; nothing with no face to set them in.
fn caption(fonts: &Fonts, size: u32, ink: [u8; 3], words: &str) -> Option<Frame> {
    let letters = words
        .chars()
        .map(|c| ui::line(fonts, TextStyle::Bold, size, ink, &c.to_string()))
        .collect::<Option<Vec<Frame>>>()?;
    let track = (size / 3).max(3);
    let width = letters.iter().map(|l| l.width).sum::<u32>()
        + track * letters.len().saturating_sub(1) as u32;
    let height = letters.iter().map(|l| l.height).max()?;
    let mut frame = Frame {
        blend: Default::default(),
        width: width.max(1),
        height,
        rgba: vec![0; (width.max(1) * height * 4) as usize],
    };
    let mut x = 0;
    for letter in &letters {
        ui::blit(&mut frame, letter, x, 0, 255);
        x += (letter.width + track) as i32;
    }
    Some(frame)
}

impl ForecastLine {
    /// The gaps of a line whose skies are `side` high: between a sky and
    /// its figures, and between now and the day.
    fn gaps(side: u32) -> (u32, u32) {
        ((side / 4).max(2), (side * 3 / 4).max(4))
    }

    /// The room today's line takes with skies `side` high, the figure now
    /// `now` wide (or none), the day's figures `day` wide together, and
    /// the numbers and captions `stack` high.
    fn room(side: u32, now: Option<u32>, day: u32, stack: u32) -> (u32, u32) {
        let (near, apart) = Self::gaps(side);
        (
            now.map_or(0, |n| side + near + n + apart) + side + near + day,
            side.max(stack),
        )
    }

    /// The room `count` columns take, each `width` wide, with skies
    /// `side` high and the label and figure rows `label` and `figure`
    /// high: a gap of half a sky between columns, an eighth of a sky
    /// between the rows.
    fn columns_room(count: u32, side: u32, label: u32, figure: u32, width: u32) -> (u32, u32) {
        let gap = (side / 2).max(2);
        let between = (side / 8).max(2);
        (
            count * width + count.saturating_sub(1) * gap,
            label + between + side + between + figure,
        )
    }

    /// What a span's columns say, from the reading: the next hours every
    /// `step` of them, or the week's days.
    fn entries(weather: &Weather, span: Span, twelve: bool) -> Vec<Entry> {
        match span {
            Span::Today => Vec::new(),
            Span::Hours(step) => weather
                .hours
                .iter()
                .step_by(step.max(1) as usize)
                .take(24 / step.max(1) as usize)
                .map(|h| {
                    let label = if twelve {
                        let hour = h.hour % 12;
                        format!(
                            "{}{}",
                            if hour == 0 { 12 } else { hour },
                            if h.hour < 12 { "am" } else { "pm" }
                        )
                    } else {
                        format!("{:02}", h.hour)
                    };
                    (label, vec![(degrees(h.temp), Some(h.temp))], h.code, h.day)
                })
                .collect(),
            Span::Week => weather
                .days
                .iter()
                .take(7)
                .map(|d| {
                    (
                        when::day_name(d.weekday, true).to_string(),
                        vec![
                            (degrees(d.low), Some(d.low)),
                            (" / ".to_string(), None),
                            (degrees(d.high), Some(d.high)),
                        ],
                        d.code,
                        true,
                    )
                })
                .collect(),
        }
    }

    /// Set the forecast at the size wanted (the skies' height), or the
    /// largest at which it fits in `most`: today's line, or a span's
    /// columns; the room it takes, or nothing with no face to set it in
    /// or nothing to say.
    #[allow(clippy::too_many_arguments)]
    fn set(
        &mut self,
        fonts: &Fonts,
        weather: &Weather,
        span: Span,
        twelve: bool,
        heat: Option<Heat>,
        wanted: u32,
        ink: [u8; 3],
        most: (u32, u32),
    ) -> Option<(u32, u32)> {
        self.spanned = span != Span::Today;
        if self.spanned {
            self.set_columns(fonts, weather, span, twelve, heat, wanted, ink, most)
        } else {
            self.set_line(fonts, weather, heat, wanted, ink, most)
        }
    }

    /// Today as a line: a sky and the temperature now with NOW under it,
    /// a sky and the day's low and high with MIN and MAX under them. The
    /// numbers are set at the forecast's size, as the line always was, so
    /// the size reaches as far as it did; the skies and the captions are
    /// their shares of it.
    fn set_line(
        &mut self,
        fonts: &Fonts,
        weather: &Weather,
        heat: Option<Heat>,
        wanted: u32,
        ink: [u8; 3],
        most: (u32, u32),
    ) -> Option<(u32, u32)> {
        const ANY: (u32, u32) = (u32::MAX, u32::MAX);
        // Each number in the colour of its degree where the heatmap is on.
        let ink_of = |temp: f32| heat.map_or(ink, |heat| heat.of(temp));
        let (lo_words, hi_words) = (degrees(weather.low), degrees(weather.high));
        let now_words = weather.now.map(degrees);
        let words = (
            now_words.clone().unwrap_or_default(),
            format!("{lo_words} / {hi_words}"),
        );
        // The numbers and their captions, stacked, at a size.
        let stack_of = |size: u32, number_h: u32| -> Option<u32> {
            let cap = ui::line(fonts, TextStyle::Bold, part(size, TODAY_CAPTION), ink, "N")?.height;
            Some((number_h + cap).saturating_sub(share_of(size, TODAY_CAPTION_UP)))
        };
        let size = match &self.fit_for {
            Some((w, s, m, size)) if (w, *s, *m) == (&words, wanted, most) => *size,
            _ => {
                // Measured at the size wanted, every digit an 8 as a line is.
                let side = part(wanted, TODAY_SKY);
                let number = wanted;
                let lo = ui::line(fonts, TextStyle::Bold, number, ink, &widest(&lo_words))?;
                let slash = ui::line(fonts, TextStyle::Bold, number, ink, " / ")?;
                let hi = ui::line(fonts, TextStyle::Bold, number, ink, &widest(&hi_words))?;
                let now = match &now_words {
                    Some(text) => {
                        Some(ui::line(fonts, TextStyle::Bold, number, ink, &widest(text))?.width)
                    }
                    None => None,
                };
                let day = lo.width + slash.width + hi.width;
                let room = Self::room(side, now, day, stack_of(wanted, lo.height)?);
                let size = fitted_size(wanted, room, most);
                self.fit_for = Some((words, wanted, most, size));
                size
            }
        };
        let side = part(size, TODAY_SKY);
        let number = size;
        let lo = self
            .lo
            .set(fonts, lo_words, number, ink_of(weather.low), ANY)?;
        let slash = self.slash.set(fonts, " / ".to_string(), number, ink, ANY)?;
        let hi = self
            .hi
            .set(fonts, hi_words, number, ink_of(weather.high), ANY)?;
        let now = match (now_words, weather.now) {
            (Some(text), Some(temp)) => self.now.set(fonts, text, number, ink_of(temp), ANY),
            _ => None,
        };
        self.has_now = now.is_some();
        let cap = part(size, TODAY_CAPTION);
        self.caption_up = share_of(size, TODAY_CAPTION_UP);
        if self.captions.as_ref().map(|(s, i, _)| (*s, *i)) != Some((cap, ink)) {
            self.captions = Some((
                cap,
                ink,
                [
                    caption(fonts, cap, ink, "NOW")?,
                    caption(fonts, cap, ink, "MIN")?,
                    caption(fonts, cap, ink, "MAX")?,
                ],
            ));
        }
        // The skies for now and for the day, hot and heavy by their own reading.
        let now_key = Skies::key(
            sky_of(weather.code),
            weather.day,
            celsius(weather.now.unwrap_or(weather.high), &weather.unit),
            weather.code,
            side,
            self.colour,
            ink,
        );
        let day_key = Skies::key(
            sky_of(weather.today),
            true,
            celsius(weather.high, &weather.unit),
            weather.today,
            side,
            self.colour,
            ink,
        );
        self.skies.keep(now_key, sky_of(weather.code));
        self.skies.keep(day_key, sky_of(weather.today));
        self.skies.keep_only(&[now_key, day_key]);
        self.now_key = Some(now_key);
        self.day_key = Some(day_key);
        let stack = stack_of(size, lo.1)?;
        Some(Self::room(
            side,
            now.map(|n| n.0),
            lo.0 + slash.0 + hi.0,
            stack,
        ))
    }

    /// A span as columns, as wide as the widest of them, at the size
    /// wanted or the largest at which they all fit in `most`: the skies
    /// at the size, the figures and the labels their shares of it; each
    /// temperature in the colour of its degree where the heatmap is on,
    /// the week's where it is on for the days too.
    #[allow(clippy::too_many_arguments)]
    fn set_columns(
        &mut self,
        fonts: &Fonts,
        weather: &Weather,
        span: Span,
        twelve: bool,
        heat: Option<Heat>,
        wanted: u32,
        ink: [u8; 3],
        most: (u32, u32),
    ) -> Option<(u32, u32)> {
        const ANY: (u32, u32) = (u32::MAX, u32::MAX);
        let entries = Self::entries(weather, span, twelve);
        if entries.is_empty() {
            return None;
        }
        let count = entries.len() as u32;
        let heat = heat.filter(|heat| span != Span::Week || heat.days);
        let ink_of = |temp: Option<f32>| match (heat, temp) {
            (Some(heat), Some(temp)) => heat.of(temp),
            _ => ink,
        };
        let size = match &self.columns_for {
            Some((s, e, w, m, size)) if (*s, e, *w, *m) == (span, &entries, wanted, most) => *size,
            _ => {
                // Measured at the size wanted, every digit an 8, the widest
                // label or figure making every column's width.
                let (mut width, mut label_h, mut figure_h) = (0, 0, 0);
                for (label, parts, _, _) in &entries {
                    let l = ui::line(
                        fonts,
                        TextStyle::Bold,
                        part(wanted, LABEL),
                        ink,
                        &widest(label),
                    )?;
                    let (mut fw, mut fh) = (0, 0);
                    for (words, _) in parts {
                        let f = ui::line(
                            fonts,
                            TextStyle::Bold,
                            part(wanted, FIGURE),
                            ink,
                            &widest(words),
                        )?;
                        fw += f.width;
                        fh = fh.max(f.height);
                    }
                    width = width.max(l.width).max(fw);
                    label_h = label_h.max(l.height);
                    figure_h = figure_h.max(fh);
                }
                let room = Self::columns_room(count, wanted, label_h, figure_h, width.max(wanted));
                let size = fitted_size(wanted, room, most);
                self.columns_for = Some((span, entries.clone(), wanted, most, size));
                size
            }
        };
        self.columns.resize_with(entries.len(), Column::default);
        let (mut width, mut label_h, mut figure_h) = (0, 0, 0);
        for (column, (label, parts, _, _)) in self.columns.iter_mut().zip(&entries) {
            let l = column
                .label
                .set(fonts, label.clone(), part(size, LABEL), ink, ANY)?;
            column.parts.resize_with(parts.len(), Line::default);
            let (mut fw, mut fh) = (0, 0);
            for (line, (words, temp)) in column.parts.iter_mut().zip(parts) {
                let f = line.set(fonts, words.clone(), part(size, FIGURE), ink_of(*temp), ANY)?;
                fw += f.0;
                fh = fh.max(f.1);
            }
            width = width.max(l.0).max(fw);
            label_h = label_h.max(l.1);
            figure_h = figure_h.max(fh);
        }
        let width = width.max(size);
        // Each column's sky, hot by its own temperature (an hour's, a day's
        // high) and heavy by its code.
        let mut used = Vec::with_capacity(entries.len());
        for (column, (_, parts, code, day)) in self.columns.iter_mut().zip(&entries) {
            let temp = parts.iter().filter_map(|p| p.1).fold(f32::MIN, f32::max);
            let key = Skies::key(
                sky_of(*code),
                *day,
                celsius(temp, &weather.unit),
                *code,
                size,
                self.colour,
                ink,
            );
            self.skies.keep(key, sky_of(*code));
            column.sky = Some(key);
            used.push(key);
        }
        self.skies.keep_only(&used);
        self.column_width = width;
        self.label_height = label_h;
        self.figure_height = figure_h;
        Some(Self::columns_room(count, size, label_h, figure_h, width))
    }

    /// How the skies are drawn from here on: moving or still, thunder
    /// flashing or not, in colour or in the ink.
    fn drama(&mut self, motion: bool, flashes: bool, colour: bool) {
        self.motion = motion;
        self.flashes = flashes;
        self.colour = colour;
    }

    /// What the moving skies on show stand at, into a stamp: each sky's
    /// frame at the moment, so a display redraws when a frame changes and
    /// not before; nothing while the skies stand still.
    fn stamp(&self, t_ms: u64, h: &mut impl Hasher) {
        if !self.motion {
            return;
        }
        let keys = if self.spanned {
            self.columns
                .iter()
                .filter_map(|c| c.sky)
                .collect::<Vec<_>>()
        } else {
            [self.now_key, self.day_key].into_iter().flatten().collect()
        };
        for key in keys {
            self.skies.index(key, t_ms, true, self.flashes).hash(h);
        }
    }

    /// The sky a key stands at, at a moment.
    fn sky_at(&self, key: Option<SkyKey>, t_ms: u64) -> Option<&Frame> {
        key.and_then(|k| self.skies.at(k, t_ms, self.motion, self.flashes))
    }

    /// The forecast in the middle of a width, at an opacity, its skies at
    /// the frame a moment falls on.
    fn place(&self, frame: &mut Frame, x: i32, y: i32, width: u32, opacity: f32, t_ms: u64) {
        if self.spanned {
            self.place_columns(frame, x, y, width, opacity, t_ms);
        } else {
            self.place_line(frame, x, y, width, opacity, t_ms);
        }
    }

    fn place_line(&self, frame: &mut Frame, x: i32, y: i32, width: u32, opacity: f32, t_ms: u64) {
        let (Some(day_sky), Some((_, _, captions))) =
            (self.sky_at(self.day_key, t_ms), self.captions.as_ref())
        else {
            return;
        };
        let side = day_sky.width;
        let (near, apart) = Self::gaps(side);
        let now = self.has_now.then_some(self.now.room.0);
        let day = self.lo.room.0 + self.slash.room.0 + self.hi.room.0;
        let number_h = self.lo.room.1;
        let stack = (number_h + captions[0].height).saturating_sub(self.caption_up);
        let (total, row_h) = Self::room(side, now, day, stack);
        let mut at = x + (width as i32 - total as i32) / 2;
        let sky_y = y + (row_h as i32 - side as i32) / 2;
        let num_y = y + (row_h as i32 - stack as i32) / 2;
        let cap_y = num_y + number_h as i32 - self.caption_up as i32;
        let alpha = share(opacity, 255);
        let under = |frame: &mut Frame, caption: &Frame, left: i32, over: u32| {
            ui::blit(
                frame,
                caption,
                left + (over as i32 - caption.width as i32) / 2,
                cap_y,
                alpha,
            );
        };
        if let (Some(room), Some(sky)) = (now, self.sky_at(self.now_key, t_ms)) {
            ui::blit(frame, sky, at, sky_y, alpha);
            at += (side + near) as i32;
            self.now.place(frame, at, num_y, room, opacity);
            under(frame, &captions[0], at, room);
            at += (room + apart) as i32;
        }
        ui::blit(frame, day_sky, at, sky_y, alpha);
        at += (side + near) as i32;
        self.lo.place(frame, at, num_y, self.lo.room.0, opacity);
        under(frame, &captions[1], at, self.lo.room.0);
        at += self.lo.room.0 as i32;
        self.slash
            .place(frame, at, num_y, self.slash.room.0, opacity);
        at += self.slash.room.0 as i32;
        self.hi.place(frame, at, num_y, self.hi.room.0, opacity);
        under(frame, &captions[2], at, self.hi.room.0);
    }

    fn place_columns(
        &self,
        frame: &mut Frame,
        x: i32,
        y: i32,
        width: u32,
        opacity: f32,
        t_ms: u64,
    ) {
        let count = self.columns.len() as u32;
        let Some(side) = self
            .columns
            .first()
            .and_then(|c| self.sky_at(c.sky, t_ms))
            .map(|s| s.width)
        else {
            return;
        };
        let (label_h, figure_h, column_width) =
            (self.label_height, self.figure_height, self.column_width);
        let gap = (side / 2).max(2);
        let between = (side / 8).max(2);
        let total = Self::columns_room(count, side, label_h, figure_h, column_width).0;
        let mut at = x + (width as i32 - total as i32) / 2;
        let alpha = share(opacity, 255);
        for column in &self.columns {
            column.label.place(frame, at, y, column_width, opacity);
            if let Some(sky) = self.sky_at(column.sky, t_ms) {
                ui::blit(
                    frame,
                    sky,
                    at + (column_width as i32 - sky.width as i32) / 2,
                    y + (label_h + between) as i32,
                    alpha,
                );
            }
            // The figure's parts side by side, the whole in the middle of the column.
            let figure_w: u32 = column.parts.iter().map(|p| p.room.0).sum();
            let mut px = at + (column_width as i32 - figure_w as i32) / 2;
            let fy = y + (label_h + between + side + between) as i32;
            for line in &column.parts {
                line.place(frame, px, fy, line.room.0, opacity);
                px += line.room.0 as i32;
            }
            at += (column_width + gap) as i32;
        }
    }
}

/// A text with every digit an 8, the widest shape words of its kind take.
fn widest(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_ascii_digit() { '8' } else { c })
        .collect()
}

/// The clock and the date of the idle screen, and the forecast with them.
#[derive(Default)]
struct ClockFace {
    clock: Line,
    date: Line,
    forecast: ForecastLine,
    /// The clock where it is drawn and not set in type.
    drawn: clock::Drawn,
}

/// What a glass is drawn with: the look, the hairline, and the frost.
struct Glass<'a> {
    look: Look,
    hairline: f32,
    frost: Option<&'a mut Frost>,
}

impl Glass<'_> {
    /// A glass behind words, with room about them, in the look's tint or a
    /// colour of its own; none at no opacity.
    fn behind(
        &mut self,
        frame: &mut Frame,
        rect: (i32, i32, u32, u32),
        pad: (u32, u32),
        opacity: f32,
        own: Option<[u8; 3]>,
    ) {
        if opacity <= 0.0 {
            return;
        }
        let (x, y, w, h) = (
            rect.0 - pad.0 as i32,
            rect.1 - pad.1 as i32,
            rect.2 + 2 * pad.0,
            rect.3 + 2 * pad.1,
        );
        if let Some(frost) = self.frost.as_deref_mut() {
            frost.apply(frame, x, y, w, h, 255);
        }
        let t = own.unwrap_or(self.look.tint);
        ui::fill(frame, x, y, w, h, [t[0], t[1], t[2], share(opacity, 255)]);
        ui::fill(
            frame,
            x,
            y,
            w,
            1,
            [255, 255, 255, share(self.hairline, 255)],
        );
    }
}

/// Where the idle screen's words go: a date at the top of the screen in
/// a rectangle of its own; and in the middle of what the top date and the
/// bar leave, one rectangle for the clock and a date above or below it,
/// with the height each of the two starts at. A date placed with a clock
/// that is not shown takes the middle alone.
#[derive(Debug, PartialEq, Default)]
struct IdleLayout {
    top: Option<(i32, i32, u32, u32)>,
    middle: Option<(i32, i32, u32, u32)>,
    clock_y: Option<i32>,
    date_y: Option<i32>,
}

/// The measures the idle screen is laid out with, in pixels: the screen's
/// margin, the gap between a clock and a date, and the room a glass keeps
/// about its words.
#[derive(Clone, Copy)]
struct IdleMeasure {
    margin: u32,
    gap: u32,
    pad: (u32, u32),
}

impl IdleMeasure {
    /// What a date at the top takes of the picture's height, its glass
    /// and the margin under it included.
    fn top_takes(&self, date_height: u32) -> u32 {
        self.margin + date_height + 2 * self.pad.1 + self.margin
    }
}

fn idle_layout(
    picture: (u32, u32),
    below: u32,
    m: IdleMeasure,
    clock: Option<(u32, u32)>,
    date: Option<(u32, u32)>,
    place: DatePlace,
) -> IdleLayout {
    let mut layout = IdleLayout::default();
    let mut with_clock = date;
    let mut from = 0;
    if let (Some((w, h)), DatePlace::Top) = (date, place) {
        layout.top = Some((
            (picture.0 as i32 - w as i32) / 2,
            (m.margin + m.pad.1) as i32,
            w,
            h,
        ));
        with_clock = None;
        from = m.top_takes(h);
    }
    let gap = if clock.is_some() && with_clock.is_some() {
        m.gap
    } else {
        0
    };
    let width = clock.map_or(0, |c| c.0).max(with_clock.map_or(0, |d| d.0));
    let height = clock.map_or(0, |c| c.1) + gap + with_clock.map_or(0, |d| d.1);
    if width == 0 {
        return layout;
    }
    // In the middle of what is left, also where it is more than fits: words
    // larger than the picture run over its edges by as much on either side.
    let x = (picture.0 as i32 - width as i32) / 2;
    let space = picture.1 as i32 - below as i32 - from as i32;
    let y = from as i32 + (space - height as i32) / 2;
    layout.middle = Some((x, y, width, height));
    let date_first = place == DatePlace::Above;
    let mut at = y;
    if let (Some((_, h)), true) = (with_clock, date_first) {
        layout.date_y = Some(at);
        at += (h + gap) as i32;
    }
    if let Some((_, h)) = clock {
        layout.clock_y = Some(at);
        at += (h + gap) as i32;
    }
    if let (Some(_), false) = (with_clock, date_first) {
        layout.date_y = Some(at);
    }
    layout
}

/// Today's forecast set for a page, as the face sets it on the screen: the
/// look's keys for the size and the ink, the player's reading, `size`
/// pixels high, at a moment; nothing where the look hides it or the fonts
/// have no bold face.
pub fn forecast_preview(
    fonts: &Fonts,
    keys: &BTreeMap<String, String>,
    weather: &Weather,
    size: u32,
    _wall: &Wall,
    t_ms: u64,
) -> Option<Frame> {
    const ANY: (u32, u32) = (u32::MAX, u32::MAX);
    let theme = Theme::resolve(None, keys);
    if !theme.weather_show {
        return None;
    }
    let ink = theme.weather_ink.unwrap_or(theme.ink);
    let mut line = ForecastLine::default();
    line.drama(
        theme.weather_motion,
        theme.weather_thunder,
        theme.weather_colour,
    );
    let twelve = twelve_hour(&theme.clock_format);
    let heat = Heat::from(&theme, weather);
    let (width, height) = line.set(
        fonts,
        weather,
        theme.weather_span,
        twelve,
        heat,
        size,
        ink,
        ANY,
    )?;
    let mut frame = Frame {
        blend: Default::default(),
        width,
        height,
        rgba: vec![0; (width * height * 4) as usize],
    };
    line.place(&mut frame, 0, 0, width, 1.0, t_ms);
    Some(frame)
}

/// One of the idle screen's elements, in the order they stand in when
/// they share cells.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Piece {
    Clock,
    Date,
    Forecast,
}

/// The idle screen's grid: three rows by three columns in equal thirds
/// over the picture above the bar. The lines between the cells and the
/// edges, in pixels.
#[derive(Debug, PartialEq)]
struct Grid {
    xs: [i32; 4],
    ys: [i32; 4],
}

impl Grid {
    fn new(picture: (u32, u32), below: u32) -> Self {
        let lines = |whole: u32| {
            let at = |share: f32| (whole as f32 * share).round() as i32;
            [0, at(1.0 / 3.0), at(2.0 / 3.0), whole as i32]
        };
        Self {
            xs: lines(picture.0),
            ys: lines(picture.1.saturating_sub(below)),
        }
    }

    /// The rectangle a block of cells covers.
    fn area(&self, on: Cells) -> (i32, i32, u32, u32) {
        let (x, y) = (self.xs[on.columns.0 as usize], self.ys[on.rows.0 as usize]);
        let (right, foot) = (
            self.xs[on.columns.1 as usize + 1],
            self.ys[on.rows.1 as usize + 1],
        );
        (x, y, (right - x).max(0) as u32, (foot - y).max(0) as u32)
    }
}

/// Where words `size` large stand in an area by their alignment: `keep`
/// from the side they are aligned to, or in the middle. Words larger than
/// the area run over it, by as much on either side when in the middle.
fn aligned(
    area: (i32, i32, u32, u32),
    size: (u32, u32),
    align: Align,
    keep: (u32, u32),
) -> (i32, i32) {
    let (x, y, w, h) = (area.0, area.1, area.2 as i32, area.3 as i32);
    let (sw, sh, kx, ky) = (size.0 as i32, size.1 as i32, keep.0 as i32, keep.1 as i32);
    // Words that fit stand `keep` from the side they are aligned to. Words
    // larger than the area would then hang off the far side, and "top"
    // would move them down: so they stand `keep` from the far side instead
    // and run over the side they are aligned to, "top" always up and
    // "left" always left.
    let across = match align.across {
        Across::Left => (x + kx).min(x + w - kx - sw),
        Across::Centre => x + (w - sw) / 2,
        Across::Right => (x + w - kx - sw).max(x + kx),
    };
    let down = match align.down {
        Down::Top => (y + ky).min(y + h - ky - sh),
        Down::Middle => y + (h - sh) / 2,
        Down::Bottom => (y + h - ky - sh).max(y + ky),
    };
    (across, down)
}

impl ClockFace {
    /// Draw what the theme shows of the clock and the date, above a bar
    /// `below` high; whether anything was drawn. A clock the look places on
    /// the grid stands in its cells; the date stands as before, and where
    /// it was placed with the clock it has the middle to itself.
    fn draw(
        &mut self,
        frame: &mut Frame,
        view: &View,
        theme: &Theme,
        glass: &mut Glass,
        below: u32,
        wall: &Wall,
    ) -> bool {
        let unit = view.height as f32 / 720.0 * view.scale.max(0.5);
        let px = |units: f32| (unit * units).round() as u32;
        let m = IdleMeasure {
            margin: px(20.0),
            gap: px(8.0),
            pad: (px(28.0), px(12.0)),
        };
        let pad = m.pad;
        let least = (px(LEAST_PAD.0), px(LEAST_PAD.1));
        // A size the user set is the user's: the line is set at it, and what
        // does not fit on the picture runs over its edges. A size the look
        // comes with is fitted: the date no wider than the picture leaves
        // beside its glass and the margins, the clock in the whole width
        // and in what the date and the bar leave of the height, the margin
        // and the glass's room giving way to it down to the least a glass
        // keeps. The date is set first.
        const ANY: (u32, u32) = (u32::MAX, u32::MAX);
        let own = |key: &str| view.settings.contains_key(key);
        let widest = view.width.saturating_sub(2 * (m.margin + pad.0));
        let date = if theme.date_show && theme.date_cells.is_none() {
            let size = px(theme.measure_date).max(13);
            self.date.set(
                view.fonts,
                format_time(&theme.date_format, wall),
                size,
                date_ink(theme, view.input.weather.as_ref()),
                if own("measure.date") {
                    ANY
                } else {
                    (widest, view.height / 4)
                },
            )
        } else {
            None
        };
        let clock = if theme.clock_show && theme.clock_cells.is_none() {
            let taken = match (date, theme.date_place) {
                (Some((_, h)), DatePlace::Top) => m.top_takes(h),
                (Some((_, h)), _) => h + m.gap,
                (None, _) => 0,
            };
            let tallest = view
                .height
                .saturating_sub(below)
                .saturating_sub(taken)
                .saturating_sub(2 * least.1);
            let size = px(theme.measure_clock).max(24);
            let most = if own("measure.clock") {
                ANY
            } else {
                (view.width.saturating_sub(2 * least.0), tallest)
            };
            self.set_clock(view, theme, glass.look.accent, wall, size, most)
        } else {
            None
        };
        let layout = idle_layout(
            (view.width, view.height),
            below,
            m,
            clock,
            date,
            theme.date_place,
        );
        if let Some((x, y, w, h)) = layout.top {
            let about = (pad_about(view.width, w, pad.0, least.0), pad.1);
            glass.behind(
                frame,
                (x, y, w, h),
                about,
                theme.date_glass,
                theme.date_tint,
            );
            self.date.place(frame, x, y, w, theme.date_opacity);
        }
        if let Some((x, y, w, h)) = layout.middle {
            // The glass keeps the room designed where there is that much
            // beside the words, and what there is where they take more.
            let from = match (date, theme.date_place) {
                (Some((_, dh)), DatePlace::Top) => m.top_takes(dh),
                _ => 0,
            };
            let space = view.height.saturating_sub(below).saturating_sub(from);
            let about = (
                pad_about(view.width, w, pad.0, least.0),
                pad_about(space, h, pad.1, least.1),
            );
            glass.behind(
                frame,
                (x, y, w, h),
                about,
                theme.clock_glass,
                theme.clock_tint,
            );
            if let Some(at) = layout.clock_y {
                self.put_clock(frame, x, at, w, theme);
            }
            if let Some(at) = layout.date_y {
                self.date.place(frame, x, at, w, theme.date_opacity);
            }
        }
        let mut drawn = clock.is_some() || date.is_some();
        // On the grid: the clock and the date in the cells each occupies,
        // where each is aligned inside them; the two on the same cells with
        // the same alignment stand one under the other on one glass, the
        // clock first. A size that came with the look is fitted to the
        // cells less the margin and the least a glass keeps; a size the
        // user set is the user's, and runs over the cells and the screen's
        // edges where it is larger, as it does off the grid. The date is
        // set first, the clock in what it leaves of the cells' height.
        let grid = Grid::new((view.width, view.height), below);
        let mut blocks: Vec<(Cells, Align, f32, Vec<Piece>)> = Vec::new();
        let mut on_grid = |piece: Piece, on: Cells, align: Align, margin: f32| match blocks
            .iter_mut()
            .find(|b| b.0 == on && b.1 == align)
        {
            Some(block) => block.3.push(piece),
            None => blocks.push((on, align, margin, vec![piece])),
        };
        if let (true, Some(on)) = (theme.clock_show, theme.clock_cells) {
            on_grid(Piece::Clock, on, theme.clock_align, theme.clock_margin);
        }
        if let (true, Some(on)) = (theme.date_show, theme.date_cells) {
            on_grid(Piece::Date, on, theme.date_align, theme.date_margin);
        }
        // The forecast: on the grid and only, where the player holds one.
        let weather = view.input.weather.as_ref();
        if let (true, Some(on), Some(_)) = (theme.weather_show, theme.weather_cells, weather) {
            on_grid(
                Piece::Forecast,
                on,
                theme.weather_align,
                theme.weather_margin,
            );
        }
        for (on, align, margin, pieces) in blocks {
            let area = grid.area(on);
            let margin = px(margin);
            let room = (
                area.2.saturating_sub(2 * (margin + least.0)),
                area.3.saturating_sub(2 * (margin + least.1)),
            );
            let mut set: Vec<(Piece, (u32, u32))> = Vec::new();
            if pieces.contains(&Piece::Date) {
                let most = if own("measure.date") { ANY } else { room };
                let size = px(theme.measure_date).max(13);
                if let Some(sized) = self.set_date(view, theme, wall, size, most) {
                    set.push((Piece::Date, sized));
                }
            }
            // The forecast after the date, in what it leaves, in its own
            // size, ink and opacity.
            if let (true, Some(weather)) = (pieces.contains(&Piece::Forecast), weather) {
                let taken = set.iter().map(|(_, sized)| sized.1 + m.gap).sum::<u32>();
                let most = if own("measure.weather") {
                    ANY
                } else {
                    (room.0, room.1.saturating_sub(taken))
                };
                let size = px(theme.measure_weather).max(13);
                let ink = theme.weather_ink.unwrap_or(theme.ink);
                let twelve = twelve_hour(&theme.clock_format);
                let heat = Heat::from(theme, weather);
                self.forecast.drama(
                    theme.weather_motion,
                    theme.weather_thunder,
                    theme.weather_colour,
                );
                if let Some(sized) = self.forecast.set(
                    view.fonts,
                    weather,
                    theme.weather_span,
                    twelve,
                    heat,
                    size,
                    ink,
                    most,
                ) {
                    set.push((Piece::Forecast, sized));
                }
            }
            if pieces.contains(&Piece::Clock) {
                let taken = set.iter().map(|(_, sized)| sized.1 + m.gap).sum::<u32>();
                let most = if own("measure.clock") {
                    ANY
                } else {
                    (room.0, room.1.saturating_sub(taken))
                };
                let size = px(theme.measure_clock).max(24);
                if let Some(sized) =
                    self.set_clock(view, theme, glass.look.accent, wall, size, most)
                {
                    set.insert(0, (Piece::Clock, sized));
                }
            }
            let Some(first) = set.first().map(|(piece, _)| *piece) else {
                continue;
            };
            let width = set.iter().map(|(_, sized)| sized.0).max().unwrap_or(0);
            let height =
                set.iter().map(|(_, sized)| sized.1).sum::<u32>() + m.gap * (set.len() as u32 - 1);
            let about = (
                pad_about(area.2.saturating_sub(2 * margin), width, pad.0, least.0),
                pad_about(area.3.saturating_sub(2 * margin), height, pad.1, least.1),
            );
            let (x, y) = aligned(
                area,
                (width, height),
                align,
                (margin + about.0, margin + about.1),
            );
            let (strength, tint) = match first {
                Piece::Clock => (theme.clock_glass, theme.clock_tint),
                Piece::Date => (theme.date_glass, theme.date_tint),
                Piece::Forecast => (theme.weather_glass, theme.weather_tint),
            };
            glass.behind(frame, (x, y, width, height), about, strength, tint);
            let mut at = y;
            for (piece, sized) in &set {
                // Each to the side the block is aligned to.
                let across = match align.across {
                    Across::Left => x,
                    Across::Centre => x + (width as i32 - sized.0 as i32) / 2,
                    Across::Right => x + width as i32 - sized.0 as i32,
                };
                match piece {
                    Piece::Clock => self.put_clock(frame, across, at, sized.0, theme),
                    Piece::Date => self
                        .date
                        .place(frame, across, at, sized.0, theme.date_opacity),
                    Piece::Forecast => self.forecast.place(
                        frame,
                        across,
                        at,
                        sized.0,
                        theme.weather_opacity,
                        view.now_ms,
                    ),
                }
                at += (sized.1 + m.gap) as i32;
            }
            drawn = true;
        }
        drawn
    }

    /// Set the clock, in type or drawn, at a size or the largest that fits
    /// `most`; the room it takes.
    fn set_clock(
        &mut self,
        view: &View,
        theme: &Theme,
        accent: [u8; 3],
        wall: &Wall,
        size: u32,
        most: (u32, u32),
    ) -> Option<(u32, u32)> {
        let text = format_time(&theme.clock_format, wall);
        if theme.clock_face == clock::ClockKind::Type {
            return self.clock.set(
                view.fonts,
                text,
                size,
                theme.clock_ink.unwrap_or(theme.ink),
                most,
            );
        }
        // A drawn face: fitted by the room it takes at the size wanted,
        // every digit an 8 so the room stands still.
        let widest: String = text
            .chars()
            .map(|c| if c.is_ascii_digit() { '8' } else { c })
            .collect();
        let size = fitted_size(size, clock::room(theme.clock_face, &widest, size), most);
        self.drawn.set(
            &clock::Asked {
                kind: theme.clock_face,
                dial: theme.clock_dial,
                paints: theme.paints(accent),
                text: &text,
                wall,
                seconds: when::shows_seconds(&theme.clock_format),
                now_ms: view.now_ms,
            },
            size,
        )
    }

    /// Set the date at a size, or the largest that fits `most`; the room it takes.
    fn set_date(
        &mut self,
        view: &View,
        theme: &Theme,
        wall: &Wall,
        size: u32,
        most: (u32, u32),
    ) -> Option<(u32, u32)> {
        self.date.set(
            view.fonts,
            format_time(&theme.date_format, wall),
            size,
            date_ink(theme, view.input.weather.as_ref()),
            most,
        )
    }

    /// The clock as it was set, in the middle of a width.
    fn put_clock(&self, frame: &mut Frame, x: i32, y: i32, width: u32, theme: &Theme) {
        if theme.clock_face == clock::ClockKind::Type {
            self.clock.place(frame, x, y, width, theme.clock_opacity);
        } else if let Some(drawn) = self.drawn.frame() {
            ui::blit(
                frame,
                drawn,
                x + (width as i32 - drawn.width as i32) / 2,
                y,
                share(theme.clock_opacity, 255),
            );
        }
    }
}

impl Face {
    /// What a frame begins with: the player's word settled, the bar's
    /// presence moved on, a resting finger turned into a mute. Answers with
    /// what there is to draw: how much of the bar shows, and whether the
    /// clock stands. Asked twice for one frame it changes nothing the
    /// second time.
    fn advance(&mut self, view: &View) -> (u8, bool, bool, bool) {
        let meta = &view.input.metadata;
        let now = view.now_ms;
        // A change of track is not the player standing still: the bar does
        // not come up for it, and the clock does not show.
        let playing = self.playing.settle(now, meta);
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
        // What the look says of the idle screen, taken before anything else
        // of the face is touched.
        let (clock_show, date_show, off_min, fade_ms, name, dim) = {
            let theme = &self.tokens(view).theme;
            (
                theme.clock_show,
                // A forecast in hand stands where a date does, with one or alone.
                theme.date_show || (theme.weather_show && view.input.weather.is_some()),
                theme.idle_off_min,
                theme.idle_fade_ms,
                theme.idle_picture.clone(),
                theme.idle_dim,
            )
        };
        self.fade_ms = fade_ms;
        let idle = view.ours && !playing;
        let off_ms = u64::from(off_min) * 60_000;
        // The screen goes black after the minutes the look names with
        // nothing playing and no touch; music coming wakes it.
        let was_dark = self.dark;
        if idle {
            let since = *self.idle_since.get_or_insert(now);
            let last = since.max(self.touched_at.unwrap_or(0));
            self.dark = off_ms > 0 && now.saturating_sub(last) >= off_ms;
        } else {
            self.idle_since = None;
            self.dark = false;
        }
        // Going black begins now; coming back begins the moment it stops.
        if self.dark && !was_dark {
            self.dark_since = Some(now);
            self.wake_since = None;
        } else if !self.dark && was_dark {
            self.wake_since = Some(now);
            self.dark_since = None;
        }
        // Black that has come whole: nothing under it is drawn. While it
        // fades, in or out, what is under it is.
        let dark = self.black(now) == 255;
        let clock = idle && !dark && (clock_show || date_show);
        // The picture for when nothing plays, where one is chosen and read:
        // in a browser the page brings it (none until it is in), on a
        // machine it is read from the folder the launcher names.
        let file = if idle && !dark && !name.is_empty() {
            host_picture(&name).map(PathBuf::from).or_else(|| {
                let folder = match self.backgrounds.as_deref() {
                    Some(folder) => Some(folder.to_string()),
                    None => std::env::var("GLASS_BACKGROUNDS").ok(),
                };
                folder.and_then(|folder| idle_file(&folder, &name))
            })
        } else {
            None
        };
        let picture = self.idle.follow(file, view.width, view.height, dim);
        (alpha, clock, picture, dark)
    }

    /// How black the screen is at `now`, 0 to 255: going black over the
    /// fade from the moment it began, back over the fade from the moment it
    /// stopped, and 0 or 255 outside those.
    fn black(&self, now: u64) -> u8 {
        let fade = u64::from(self.fade_ms);
        let share = |since: u64| {
            (now.saturating_sub(since) * 255)
                .checked_div(fade)
                .map_or(255, |s| s.min(255) as u8)
        };
        if self.dark {
            self.dark_since.map_or(255, share)
        } else {
            self.wake_since.map_or(0, |since| 255 - share(since))
        }
    }

    /// The tokens for a view: read on the first frame, and again when the
    /// view names another theme or other settings, as happens under a face
    /// that goes on while a page or a remote takes the player's new
    /// configuration. What was made with the tokens before is then made
    /// again: the track's look is read anew, the look in use standing until
    /// it is in, and nothing counts as drawn.
    fn tokens(&mut self, view: &View) -> &Tokens {
        if !self.tokens.as_ref().is_some_and(|t| t.stand_for(view)) {
            self.tokens = Some(tokens_for(view, self.faces.as_deref()));
            self.track.cover = None;
            self.track.pending = None;
            self.accents = None;
            self.clock = ClockFace::default();
            self.drawn = None;
        }
        self.tokens.as_ref().expect("the tokens were read above")
    }
}

impl Face {
    /// Everything a drawing depends on, as one number: two frames with the
    /// same stamp are drawn alike on the same picture. How much of the bar
    /// shows, what is pressed, the sheet, what each button and tile wears,
    /// the look of the track, the picture's size, and what the clock and
    /// the date say.
    fn stamp(&self, view: &View, alpha: u8, time: Option<&Wall>) -> u64 {
        let meta = &view.input.metadata;
        let mut h = DefaultHasher::new();
        (alpha, view.width, view.height, view.scale.to_bits()).hash(&mut h);
        (self.track.look.tint, self.track.look.accent).hash(&mut h);
        self.dark.hash(&mut h);
        // The picture for when nothing plays, where it is in hand.
        self.idle
            .frame
            .as_ref()
            .and(self.idle.key.as_ref())
            .hash(&mut h);
        if alpha > 0 {
            self.pressed.map(|b| b as u8).hash(&mut h);
            self.pressed_tile.map(|t| t as u8).hash(&mut h);
            self.sheet_open.hash(&mut h);
            for button in BUTTONS {
                (button_icon(button, meta) as u8).hash(&mut h);
            }
            for tile in TILES {
                (tile_icon(tile, meta) as u8, tile_lit(tile, meta)).hash(&mut h);
            }
        }
        if let (Some(tm), Some(tokens)) = (time, self.tokens.as_ref()) {
            let theme = &tokens.theme;
            if theme.clock_show {
                format_time(&theme.clock_format, tm).hash(&mut h);
                // A card on its way is another picture every frame.
                if self.clock.drawn.moving(view.now_ms) {
                    view.now_ms.hash(&mut h);
                }
            }
            if theme.date_show {
                format_time(&theme.date_format, tm).hash(&mut h);
            }
            // What the forecast says, as it is drawn: the span, the skies and
            // whole degrees, of today, of the hours and of the days.
            if let (true, Some(w)) = (theme.weather_show, view.input.weather.as_ref()) {
                let whole = |t: f32| t.round() as i32;
                theme.weather_span.hash(&mut h);
                (w.code, w.day, w.today, w.now.map(whole)).hash(&mut h);
                (whole(w.low), whole(w.high)).hash(&mut h);
                for hour in &w.hours {
                    (hour.hour, hour.code, hour.day, whole(hour.temp)).hash(&mut h);
                }
                for day in &w.days {
                    (day.weekday, day.code, whole(day.low), whole(day.high)).hash(&mut h);
                }
                // The moving skies' frames at this moment; nothing while they
                // stand still.
                self.clock.forecast.stamp(view.now_ms, &mut h);
            }
        }
        h.finish()
    }
}

impl Overlay for Face {
    /// Nothing to draw while the player plays and the bar is away: the
    /// display is told so, and spares the copy of the picture a face draws
    /// on. And where the face would draw what it drew last, a clock that
    /// says the same minute, a bar nobody touches, the display is told
    /// that too, and leaves the screen as it is.
    fn covers(&mut self, view: &View) -> Cover {
        let (alpha, clock, picture, dark) = self.advance(view);
        if alpha == 0 && !clock && !picture && !dark && self.black(view.now_ms) == 0 {
            self.drawn = None;
            return Cover::Nothing;
        }
        // A cover read since the last frame changes what is drawn.
        if let Some(tokens) = self.tokens.as_ref() {
            if self
                .track
                .follow(&tokens.theme, &view.input.metadata.art_file)
            {
                self.accents = None;
            }
        }
        let time = clock.then_some(view.wall);
        // Black is one picture, whatever the bar would be under it; on its
        // way, in or out, it is another picture every frame.
        let black = self.black(view.now_ms);
        let stamp = if dark {
            self.stamp(view, 0, None)
        } else if black > 0 {
            let mut h = DefaultHasher::new();
            (self.stamp(view, alpha, time), view.now_ms).hash(&mut h);
            h.finish()
        } else {
            self.stamp(view, alpha, time)
        };
        if self.drawn.is_some() && self.drawn == Some(stamp) {
            Cover::Same
        } else {
            Cover::New
        }
    }

    fn draw(&mut self, frame: &mut Frame, view: &View) -> bool {
        let meta = &view.input.metadata;
        let (alpha, clock, picture, dark) = self.advance(view);
        let time = clock.then_some(view.wall);
        // Black, and nothing on it: the screen is off as far as its pixels go.
        if dark {
            ui::fill(frame, 0, 0, frame.width, frame.height, [0, 0, 0, 255]);
            self.drawn = Some(self.stamp(view, 0, None));
            return true;
        }
        let black = self.black(view.now_ms);
        // The tokens are this view's: `advance` saw to it.
        let Some(tokens) = self.tokens.as_ref() else {
            return false;
        };
        let theme = &tokens.theme;
        let frosted = tokens.frosted;
        let bar = Bar::measured(view.width, view.height, view.scale, theme.measure_bar);
        if alpha > 0 || clock {
            // The look of the track, asked for only while there is something to draw with it.
            if self.track.follow(theme, &meta.art_file) {
                self.accents = None;
            }
        }
        let look = self.track.look;
        let ink = theme.buttons_ink.unwrap_or(theme.ink);
        let mut drawn = false;
        // Nothing plays and a picture of the user's own is chosen: it stands
        // in the theme's place, under everything the face draws.
        if picture {
            if let Some(own) = self.idle.frame.as_ref() {
                ui::blit(frame, own, 0, 0, 255);
                drawn = true;
            }
        }
        // The clock and the date first, when the player stands still on the
        // display's own screen: the controls are drawn after and lie over them.
        if clock {
            if let Some(wall) = time {
                let mut glass = Glass {
                    look,
                    hairline: theme.hairline,
                    frost: frosted.then_some(&mut self.frost),
                };
                drawn |= self.clock.draw(frame, view, theme, &mut glass, bar.h, wall);
            }
        }
        if alpha > 0 {
            // The icons at half the bar's height, rastered when the size or the ink changes.
            let size = (bar.h / 2).max(16);
            let icons = match self.icons.take() {
                Some(set) if set.is(size, ink) => set,
                _ => icon::Set::new(size, ink),
            };
            let accents = match self.accents.take() {
                Some(set) if set.is(size, look.accent) => set,
                _ => icon::Set::new(size, look.accent),
            };
            let glass = |frame: &mut Frame, x: i32, y: i32, w: u32, h: u32, opacity: f32| {
                ui::fill(
                    frame,
                    x,
                    y,
                    w,
                    h,
                    [
                        look.tint[0],
                        look.tint[1],
                        look.tint[2],
                        share(opacity, alpha),
                    ],
                );
                ui::fill(
                    frame,
                    x,
                    y,
                    w,
                    1,
                    [255, 255, 255, share(theme.hairline, alpha)],
                );
            };
            let button_alpha = share(theme.buttons_opacity, alpha);
            // The bar: glass over the picture, frosted where that is on.
            if frosted {
                self.frost.apply(frame, bar.x, bar.y, bar.w, bar.h, alpha);
            }
            glass(frame, bar.x, bar.y, bar.w, bar.h, theme.bar);
            for (i, button) in BUTTONS.iter().enumerate() {
                let rect = bar.button_rect(i);
                if self.pressed == Some(*button) {
                    ui::fill(
                        frame,
                        rect.0,
                        rect.1,
                        rect.2,
                        rect.3,
                        [255, 255, 255, share(0.16, alpha)],
                    );
                }
                let open = *button == Button::More && self.sheet_open;
                if open {
                    ui::fill(
                        frame,
                        rect.0,
                        rect.1,
                        rect.2,
                        rect.3,
                        [
                            look.accent[0],
                            look.accent[1],
                            look.accent[2],
                            share(0.18, alpha),
                        ],
                    );
                }
                let set = if open { &accents } else { &icons };
                place(
                    frame,
                    set.get(button_icon(*button, meta)),
                    rect,
                    button_alpha,
                );
            }
            if self.sheet_open {
                let sheet = bar.sheet();
                if frosted {
                    self.frost
                        .apply(frame, sheet.x, sheet.y, sheet.w, sheet.h, alpha);
                }
                glass(frame, sheet.x, sheet.y, sheet.w, sheet.h, theme.sheet);
                for (i, tile) in TILES.iter().enumerate() {
                    let rect = sheet.tile_rect(i);
                    let lit = tile_lit(*tile, meta);
                    if lit {
                        ui::fill(
                            frame,
                            rect.0,
                            rect.1,
                            rect.2,
                            rect.3,
                            [
                                look.accent[0],
                                look.accent[1],
                                look.accent[2],
                                share(0.18, alpha),
                            ],
                        );
                    }
                    if self.pressed_tile == Some(*tile) {
                        ui::fill(
                            frame,
                            rect.0,
                            rect.1,
                            rect.2,
                            rect.3,
                            [255, 255, 255, share(0.16, alpha)],
                        );
                    }
                    // A mode that is on wears the accent; one that is off is there, and quieter.
                    let (set, a) = if lit {
                        (&accents, button_alpha)
                    } else {
                        (&icons, share(0.55, button_alpha))
                    };
                    place(frame, set.get(tile_icon(*tile, meta)), rect, a);
                }
            }
            self.icons = Some(icons);
            self.accents = Some(accents);
            drawn = true;
        }
        // On its way to black or back from it: black over everything, by
        // how far it has come.
        if black > 0 {
            ui::fill(frame, 0, 0, frame.width, frame.height, [0, 0, 0, black]);
            let mut h = DefaultHasher::new();
            (self.stamp(view, alpha, time), view.now_ms).hash(&mut h);
            self.drawn = Some(h.finish());
            return true;
        }
        self.drawn = drawn.then(|| self.stamp(view, alpha, time));
        drawn
    }

    fn pointer(&mut self, kind: PointerKind, x: i32, y: i32, view: &View) -> bool {
        let now = view.now_ms;
        // A touch keeps the screen on, and the press that wakes a black
        // screen is the wake and nothing else, up to its lift.
        self.touched_at = Some(now);
        if self.dark || self.waking {
            self.waking = kind != PointerKind::Up;
            if self.dark {
                self.dark = false;
                self.wake_since = Some(now);
                self.dark_since = None;
            }
            return true;
        }
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
        let units = self.tokens(view).theme.measure_bar;
        let bar = Bar::measured(view.width, view.height, view.scale, units);
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
                // A press beside an open sheet is the one that shuts it, and
                // is the face's own: the theme does not act on it.
                self.shutting = self.sheet_open && on_bar.is_none() && on_sheet.is_none();
                if on_bar.is_some() || on_sheet.is_some() || self.shutting {
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
                    || self.shutting
            }
            PointerKind::Up => {
                let was_tile = self.pressed_tile.take();
                let was = self.pressed.take();
                if std::mem::take(&mut self.shutting) {
                    self.sheet_open = false;
                    self.presence.kept(now);
                    return true;
                }
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
                            if let Some(command) = command_for(button, meta) {
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
                // A tap on the picture: the bar comes or goes; the theme
                // sees the tap too.
                self.presence.touched(now);
                false
            }
        }
    }

    fn name(&self) -> Option<String> {
        Some(banner())
    }

    /// Where the display built with this face is released: a remote that
    /// is the bundle brings itself up to date from glass-evo's releases,
    /// and so stays the bundle.
    fn origin(&self) -> Option<overlay::Origin> {
        Some(overlay::Origin {
            repository: "foonerd/glass-evo".to_string(),
            asset: "glass-evo-".to_string(),
            binary: "glass-evo".to_string(),
            version: VERSION.to_string(),
        })
    }

    fn commands(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use overlay::face::Input;

    static NO_SETTINGS: BTreeMap<String, String> = BTreeMap::new();

    fn settings(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

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

    /// Thursday 1 October 2026, five past one in the afternoon and nine
    /// seconds, an hour east of universal time.
    static A_THURSDAY: std::sync::LazyLock<Wall> =
        std::sync::LazyLock::new(|| Wall::at(1_790_856_309_000, 60, "BST"));

    fn view<'a>(input: &'a Input, fonts: &'a Fonts, now_ms: u64) -> View<'a> {
        View {
            input,
            fonts,
            width: 1280,
            height: 720,
            now_ms,
            wall: &A_THURSDAY,
            ours: true,
            scale: 1.0,
            settings: &NO_SETTINGS,
            theme_dir: "",
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
    fn the_face_says_where_its_display_is_released() {
        let origin = Face::new().origin().expect("an origin");
        assert_eq!(origin.repository, "foonerd/glass-evo");
        // An archive is glass-evo-<version>-<arch>.tar.gz, the display in
        // it bin/<arch>/glass-evo, and what it answers to --version ends in
        // the version a release is tagged with.
        assert_eq!(
            format!("{}{}-x64.tar.gz", origin.asset, origin.version),
            format!("glass-evo-{}-x64.tar.gz", env!("CARGO_PKG_VERSION"))
        );
        assert_eq!(origin.binary, "glass-evo");
        assert_eq!(
            banner().split_whitespace().last(),
            Some(origin.version.as_str())
        );
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
        assert_eq!(bar.button_rect(5), (1065, 648, 215, 72), "to the bar's end");
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
        let at = |volume: u32| Metadata {
            volume,
            ..Default::default()
        };
        assert_eq!(
            pair(command_for(Button::Toggle, &at(30)).unwrap()),
            ("toggle".to_string(), None)
        );
        assert_eq!(
            pair(command_for(Button::VolumeDown, &at(3)).unwrap()),
            ("volume".to_string(), Some(serde_json::json!(0))),
            "never below zero"
        );
        assert_eq!(
            pair(command_for(Button::VolumeUp, &at(98)).unwrap()),
            ("volume".to_string(), Some(serde_json::json!(100))),
            "never above a hundred"
        );
        assert_eq!(
            command_for(Button::Previous, &at(50)).unwrap().name,
            "previous"
        );
        assert_eq!(command_for(Button::Next, &at(50)).unwrap().name, "next");
        assert!(
            command_for(Button::More, &at(50)).is_none(),
            "More only opens the sheet"
        );
        let muted = Metadata {
            volume: 50,
            mute: true,
            ..Default::default()
        };
        assert_eq!(
            pair(command_for(Button::VolumeDown, &muted).unwrap()),
            ("volume".to_string(), Some(serde_json::json!("unmute"))),
            "volume down on a muted player unmutes and steps nothing"
        );
        assert_eq!(
            pair(command_for(Button::VolumeUp, &muted).unwrap()),
            ("volume".to_string(), Some(serde_json::json!(55)))
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
    fn a_change_of_track_is_not_the_player_standing_still() {
        let track = |status: &str, title: &str| Metadata {
            status: status.to_string(),
            title: title.to_string(),
            ..Default::default()
        };
        let mut playing = Playing::default();
        assert!(playing.settle(0, &track("play", "One")));
        // The player says stop between two tracks, naming the next already.
        assert!(playing.settle(1000, &track("stop", "One")));
        assert!(playing.settle(1200, &track("stop", "Two")));
        assert!(playing.settle(1250, &track("play", "Two")));
        // A stop that lasts is a stop, and stays one.
        assert!(playing.settle(2000, &track("stop", "Two")));
        assert!(playing.settle(2000 + TRACK_CHANGE_MS - 1, &track("stop", "Two")));
        assert!(!playing.settle(2000 + TRACK_CHANGE_MS, &track("stop", "Two")));
        assert!(!playing.settle(2001 + TRACK_CHANGE_MS, &track("stop", "Two")));
        // A pause is the listener's own, and the queue's end names no track.
        assert!(playing.settle(20000, &track("play", "Two")));
        assert!(!playing.settle(20100, &track("pause", "Two")));
        assert!(playing.settle(21000, &track("play", "Two")));
        assert!(!playing.settle(21100, &track("stop", "")));
        // Never having played, a stop is a stop at once.
        assert!(!Playing::default().settle(0, &track("stop", "One")));

        // On the screen: neither the clock nor the bar for the gap.
        let fonts = Fonts::default();
        let mut face = Face::new();
        let mut input = input("play");
        input.metadata.title = "One".to_string();
        let mut frame = blank();
        // Playing: the bar leaves after its moment, and once its fade is
        // done nothing is drawn.
        face.draw(&mut frame, &view(&input, &fonts, 0));
        face.draw(&mut frame, &view(&input, &fonts, AFTER_PLAY_MS));
        let settled = AFTER_PLAY_MS + FADE_MS + 100;
        assert!(!face.draw(&mut frame, &view(&input, &fonts, settled)));
        input.metadata.status = "stop".to_string();
        input.metadata.title = "Two".to_string();
        assert!(
            face.covers(&view(&input, &fonts, settled + 100)) == Cover::Nothing,
            "the gap between two tracks: the display is told there is nothing to draw"
        );
        assert!(
            !face.draw(&mut frame, &view(&input, &fonts, settled + 100)),
            "and nothing is drawn"
        );
        assert!(!face.presence().visible(), "and the bar stays away");
        input.metadata.status = "play".to_string();
        assert!(!face.draw(&mut frame, &view(&input, &fonts, settled + 300)));
        assert!(!face.draw(&mut frame, &view(&input, &fonts, settled + 3000)));
        // A pause brings the bar at once, through its fade; a face on a
        // screen of its own has its clock to draw from the first frame.
        input.metadata.status = "pause".to_string();
        assert!(face.covers(&view(&input, &fonts, settled + 4000)) != Cover::Nothing);
        face.draw(&mut frame, &view(&input, &fonts, settled + 4000));
        assert!(face.presence().visible());
        assert!(face.draw(&mut frame, &view(&input, &fonts, settled + 4000 + FADE_MS)));
    }

    #[test]
    fn the_face_says_when_it_would_draw_what_it_drew() {
        let fonts = Fonts::default();
        let mut face = Face::new();
        let input = input("pause");
        let mut frame = blank();
        // A clock that says the same whatever the hour, so the test does
        // not cross a minute.
        let set = settings(&[("clock.format", "standing"), ("date.show", "off")]);
        let at = |now_ms: u64| {
            let mut v = view(&input, &fonts, now_ms);
            v.settings = &set;
            v
        };
        // The bar is there from the start: something new, drawn.
        assert_eq!(face.covers(&at(0)), Cover::New);
        assert!(face.draw(&mut frame, &at(0)));
        // The next frames would be the same drawing.
        assert_eq!(face.covers(&at(16)), Cover::Same);
        assert_eq!(face.covers(&at(5000)), Cover::Same);
        // Drawn again all the same (the picture under it moved): still the same.
        assert!(face.draw(&mut frame, &at(5016)));
        assert_eq!(face.covers(&at(5032)), Cover::Same);
        // A finger down on a button: a new drawing, and the same again once drawn.
        let bar = Bar::for_picture(1280, 720, 1.0);
        let (x, y, w, h) = bar.button_rect(4);
        face.pointer(
            PointerKind::Down,
            x + w as i32 / 2,
            y + h as i32 / 2,
            &at(6000),
        );
        assert_eq!(face.covers(&at(6016)), Cover::New);
        assert!(face.draw(&mut frame, &at(6016)));
        assert_eq!(face.covers(&at(6032)), Cover::Same);
        face.pointer(
            PointerKind::Up,
            x + w as i32 / 2,
            y + h as i32 / 2,
            &at(6100),
        );
        assert_eq!(face.covers(&at(6116)), Cover::New);
        // A face that has drawn nothing has nothing to repeat.
        assert_eq!(Face::new().covers(&at(0)), Cover::New);
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
        // A tap beside the sheet shuts it and is the face's own: the theme
        // under it does not act.
        assert!(face.pointer(PointerKind::Down, 300, 300, &view(&input, &fonts, 4000)));
        assert!(face.pointer(PointerKind::Move, 302, 300, &view(&input, &fonts, 4020)));
        assert!(face.pointer(PointerKind::Up, 302, 300, &view(&input, &fonts, 4050)));
        assert!(!face.sheet_open());
        assert!(face.presence().visible(), "the bar stays");
        // The next tap there is the theme's again.
        assert!(!face.pointer(PointerKind::Down, 300, 300, &view(&input, &fonts, 5000)));
        assert!(!face.pointer(PointerKind::Up, 300, 300, &view(&input, &fonts, 5050)));
    }

    #[test]
    fn the_sheet_leaves_with_the_bar() {
        let input = input("play");
        let fonts = Fonts::default();
        let mut frame = blank();
        let mut face = Face::new();
        face.draw(&mut frame, &view(&input, &fonts, 0));
        assert!(face.pointer(PointerKind::Down, 1200, 700, &view(&input, &fonts, 500)));
        assert!(face.pointer(PointerKind::Up, 1200, 700, &view(&input, &fonts, 550)));
        assert!(face.sheet_open());
        face.draw(&mut frame, &view(&input, &fonts, 550 + LINGER_MS - 1));
        assert!(face.sheet_open(), "open while the bar lingers");
        face.draw(&mut frame, &view(&input, &fonts, 550 + LINGER_MS));
        face.draw(&mut frame, &view(&input, &fonts, 550 + LINGER_MS + FADE_MS));
        assert!(!face.presence().visible());
        assert!(!face.sheet_open(), "gone with the bar");
        // The bar brought back comes without the sheet.
        face.pointer(PointerKind::Down, 300, 300, &view(&input, &fonts, 20_000));
        face.pointer(PointerKind::Up, 300, 300, &view(&input, &fonts, 20_050));
        assert!(face.presence().visible() && !face.sheet_open());
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
        // On a muted player the rest unmutes, and so does a short press.
        let mut muted = input.clone();
        muted.metadata.mute = true;
        assert!(face.pointer(PointerKind::Down, 700, 700, &view(&muted, &fonts, 5000)));
        face.draw(&mut frame, &view(&muted, &fonts, 5000 + HOLD_MS));
        assert!(face.pointer(PointerKind::Up, 700, 700, &view(&muted, &fonts, 5900)));
        assert!(face.pointer(PointerKind::Down, 700, 700, &view(&muted, &fonts, 7000)));
        assert!(face.pointer(PointerKind::Up, 700, 700, &view(&muted, &fonts, 7100)));
        let sent = face.commands();
        assert_eq!(sent.len(), 2);
        for command in sent {
            assert_eq!(
                (command.name.as_str(), command.value),
                ("volume", Some(serde_json::json!("unmute")))
            );
        }
    }

    #[test]
    fn the_buttons_and_tiles_wear_the_players_state() {
        let mut meta = Metadata::default();
        assert_eq!(button_icon(Button::Toggle, &meta), Icon::Play);
        assert_eq!(button_icon(Button::VolumeDown, &meta), Icon::Minus);
        assert_eq!(tile_icon(Tile::Repeat, &meta), Icon::Repeat);
        assert_eq!(tile_icon(Tile::Random, &meta), Icon::Shuffle);
        assert_eq!(tile_icon(Tile::Mute, &meta), Icon::Speaker);
        meta.status = "play".to_string();
        meta.mute = true;
        meta.repeat = true;
        assert_eq!(button_icon(Button::Toggle, &meta), Icon::Pause);
        assert_eq!(button_icon(Button::VolumeDown, &meta), Icon::Muted);
        assert_eq!(button_icon(Button::VolumeUp, &meta), Icon::Plus);
        assert_eq!(tile_icon(Tile::Repeat, &meta), Icon::Repeat);
        assert_eq!(tile_icon(Tile::Mute, &meta), Icon::Muted);
        meta.repeat_single = true;
        assert_eq!(tile_icon(Tile::Repeat, &meta), Icon::RepeatOne);
    }

    #[test]
    fn the_bar_and_the_sheet_draw_and_the_icons_are_rastered_once_a_size() {
        let mut input = input("pause");
        input.metadata.mute = true;
        let fonts = Fonts::default();
        let mut frame = blank();
        let mut face = Face::new();
        face.sheet_open = true;
        assert!(face.draw(&mut frame, &view(&input, &fonts, 0)));
        let pixel = |frame: &Frame, x: usize, y: usize| frame.rgba[(y * 1280 + x) * 4 + 3];
        assert_ne!(pixel(&frame, 100, 700), 0, "the bar");
        assert_ne!(pixel(&frame, 960, 600), 0, "the sheet above its right end");
        assert_eq!(pixel(&frame, 300, 600), 0, "nothing beside the sheet");
        assert_eq!(face.icons.as_ref().map(|set| set.size()), Some(36));
        let first = face
            .icons
            .as_ref()
            .map(|set| set.get(Icon::Play).rgba.as_ptr());
        face.draw(&mut frame, &view(&input, &fonts, 16));
        let again = face
            .icons
            .as_ref()
            .map(|set| set.get(Icon::Play).rgba.as_ptr());
        assert_eq!(first, again, "the same raster on the next frame");
        let mut car = view(&input, &fonts, 32);
        car.scale = 2.0;
        face.draw(&mut frame, &car);
        assert_eq!(face.icons.as_ref().map(|set| set.size()), Some(72));
    }

    #[test]
    fn the_users_settings_reach_what_is_drawn_and_what_is_touched() {
        let input = input("pause");
        let fonts = Fonts::default();
        let mut frame = blank();
        let mut face = Face::new();
        // A bar twice as tall, in a red glass, with the buttons in yellow.
        let set = settings(&[
            ("measure.bar", "144"),
            ("colours.tint", "#400000"),
            ("glass.bar", "1"),
            ("buttons.ink", "#ffff00"),
        ]);
        let mut v = view(&input, &fonts, 0);
        v.settings = &set;
        assert!(face.draw(&mut frame, &v));
        let at = |frame: &Frame, x: usize, y: usize| {
            let i = (y * 1280 + x) * 4;
            [
                frame.rgba[i],
                frame.rgba[i + 1],
                frame.rgba[i + 2],
                frame.rgba[i + 3],
            ]
        };
        assert_eq!(
            at(&frame, 5, 600),
            [0x40, 0, 0, 255],
            "the glass is the user's, and reaches up to 576"
        );
        assert_eq!(at(&frame, 5, 570)[3], 0, "and no further");
        // Play's middle is inside its triangle: the buttons' own ink.
        let play = at(&frame, 213 + 106 + 4, 576 + 72);
        assert_eq!(
            [play[0], play[1], play[2]],
            [255, 255, 0],
            "the icon wears the buttons' ink"
        );
        // A touch where the taller bar is, is the bar's.
        assert!(face.pointer(PointerKind::Down, 320, 600, &v));
        assert!(face.pointer(PointerKind::Up, 320, 600, &v));
        assert_eq!(face.commands()[0].name, "toggle");
    }

    #[test]
    fn what_is_lit_wears_the_accent() {
        let mut input = input("pause");
        input.metadata.random = true;
        let fonts = Fonts::default();
        let mut frame = blank();
        let mut face = Face::new();
        face.sheet_open = true;
        let set = settings(&[
            ("colours.accent", "#00ff00"),
            ("colours.tint", "#000000"),
            ("glass.sheet", "1"),
        ]);
        let mut v = view(&input, &fonts, 0);
        v.settings = &set;
        face.draw(&mut frame, &v);
        let at = |x: usize, y: usize| {
            let i = (y * 1280 + x) * 4;
            [frame.rgba[i], frame.rgba[i + 1], frame.rgba[i + 2]]
        };
        // The random tile is lit: an accent wash on the black glass; repeat is not.
        let lit = at(860, 580);
        assert!(
            lit[1] > 30 && lit[0] == 0 && lit[2] == 0,
            "a green wash: {lit:?}"
        );
        assert_eq!(at(650, 580), [0, 0, 0], "an unlit tile is plain glass");
    }

    #[test]
    fn a_theme_is_one_folders_text_and_nothing_else() {
        let base = Path::new("/data/faces");
        assert_eq!(
            theme_file(base, "Midnight"),
            Some(PathBuf::from("/data/faces/Midnight/face.txt"))
        );
        assert_eq!(
            theme_file(base, " Warm Glow "),
            Some(PathBuf::from("/data/faces/Warm Glow/face.txt"))
        );
        for bad in ["", "..", ".hidden", "a/b", "../../etc", "a\\b"] {
            assert_eq!(theme_file(base, bad), None, "{bad:?} is no theme");
        }
    }

    #[test]
    fn a_picture_standing_still_under_the_glass_is_frosted_once() {
        let mut frame = blank();
        for (i, px) in frame.rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let v = if (i / 16) % 2 == 0 { 255 } else { 0 };
            px.copy_from_slice(&[v, v, v, 255]);
        }
        let clear = frame.rgba.clone();
        let mut frost = Frost::default();
        frost.apply(&mut frame, 0, 648, 1280, 72, 255);
        let band = |f: &Frame| f.rgba[648 * 1280 * 4..].to_vec();
        let frosted = band(&frame);
        assert_ne!(
            frosted,
            clear[648 * 1280 * 4..].to_vec(),
            "the stripes are blurred"
        );
        assert!(
            frosted
                .as_chunks::<4>()
                .0
                .iter()
                .all(|px| px[0] > 60 && px[0] < 200),
            "into greys"
        );
        assert_eq!(
            frame.rgba[..648 * 1280 * 4],
            clear[..648 * 1280 * 4],
            "and nothing above the band is touched"
        );
        // The same picture again: the band held is used, the same pixels land.
        let held = frost.held[0].2.as_ptr();
        frame.rgba.copy_from_slice(&clear);
        frost.apply(&mut frame, 0, 648, 1280, 72, 255);
        assert_eq!(frost.held[0].2.as_ptr(), held, "not frosted again");
        assert_eq!(band(&frame), frosted);
        // The picture moves under the glass: frosted afresh.
        frame.rgba.copy_from_slice(&clear);
        for px in frame.rgba[700 * 1280 * 4..701 * 1280 * 4]
            .as_chunks_mut::<4>()
            .0
        {
            px.copy_from_slice(&[255, 0, 0, 255]);
        }
        frost.apply(&mut frame, 0, 648, 1280, 72, 255);
        assert_ne!(band(&frame), frosted);
        // Half way through a fade the glass is half frosted.
        frame.rgba.copy_from_slice(&clear);
        frost.apply(&mut frame, 0, 648, 1280, 72, 0);
        assert_eq!(frame.rgba, clear, "none of the way is nothing");
    }

    #[test]
    fn the_look_follows_the_track_and_a_fixed_theme_reads_no_cover() {
        let mut track = TrackLook::default();
        let theme = Theme::default();
        assert!(
            !track.follow(&theme, ""),
            "no cover: the neutral look stands"
        );
        assert_eq!(track.look, look::NEUTRAL);
        // A cover that is not a picture: asked for off the frame's path, and
        // neutral when the answer is in, which may be at once or some frames on.
        assert!(
            !track.follow(&theme, "/nonexistent/cover.jpg"),
            "neutral before, neutral after: no change"
        );
        assert_eq!(track.cover.as_deref(), Some("/nonexistent/cover.jpg"));
        for _ in 0..400 {
            if track.pending.is_none() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
            assert!(!track.follow(&theme, "/nonexistent/cover.jpg"));
        }
        assert!(track.pending.is_none(), "the answer came");
        assert_eq!(track.look, look::NEUTRAL);
        // A theme that fixes both colours never asks for the cover.
        let mut fixed = Theme::default();
        fixed.apply(&settings(&[
            ("colours.tint", "#102030"),
            ("colours.accent", "#ff8800"),
        ]));
        let mut track = TrackLook::default();
        assert!(
            track.follow(&fixed, "/any/cover.jpg"),
            "the look changed to the theme's"
        );
        assert!(track.pending.is_none());
        assert_eq!(
            track.look,
            Look {
                tint: [16, 32, 48],
                accent: [255, 136, 0]
            }
        );
    }

    #[test]
    fn a_time_and_a_date_are_set_in_the_pattern_given() {
        let tm: &Wall = &A_THURSDAY;
        assert_eq!(format_time("%H:%M", tm), "13:05");
        assert_eq!(format_time("%H:%M:%S", tm), "13:05:09");
        assert_eq!(format_time("%-I:%M %p", tm), "1:05 PM");
        assert_eq!(format_time("%A %-d %B", tm), "Thursday 1 October");
        assert_eq!(format_time("%a %-d %b %Y", tm), "Thu 1 Oct 2026");
        assert_eq!(format_time("%d/%m/%Y", tm), "01/10/2026");
        assert_eq!(format_time("%m/%d/%Y", tm), "10/01/2026");
        assert_eq!(format_time("%Y-%m-%d", tm), "2026-10-01");
        assert_eq!(
            format_time("%B %-d, %A", tm),
            "October 1, Thursday",
            "any order"
        );
        assert_eq!(format_time("", tm), "");
        assert_eq!(
            format_time("a\0b", tm),
            "",
            "a pattern with a nul in it is no pattern"
        );
        let midnight = Wall::at(1_790_856_309_000 - 13 * 3_600_000, 60, "BST");
        assert_eq!(format_time("%-I:%M %p", &midnight), "12:05 AM");
    }

    #[test]
    fn the_frost_takes_the_part_of_a_band_that_lies_on_the_picture() {
        let mut frame = Frame {
            blend: Default::default(),
            width: 64,
            height: 32,
            rgba: vec![128; 64 * 32 * 4],
        };
        let mut frost = Frost::default();
        // A band that starts left of the picture and ends right of it, and
        // one that starts above it: neither reaches past the picture.
        frost.apply(&mut frame, -10, 4, 100, 8, 255);
        frost.apply(&mut frame, -10, -4, 30, 16, 255);
        frost.apply(&mut frame, 60, 28, 30, 30, 255);
        let rects: Vec<(u32, u32, u32, u32)> = frost.held.iter().map(|(r, _, _)| *r).collect();
        assert_eq!(
            rects,
            vec![(0, 4, 64, 8), (0, 0, 20, 12)],
            "the third is too small to frost"
        );
    }

    #[test]
    fn a_glass_gives_its_room_to_words_wanted_larger() {
        // A 1280 wide picture, the room designed 28, the least 6.
        assert_eq!(
            pad_about(1280, 600, 28, 6),
            28,
            "room to spare: as designed"
        );
        assert_eq!(
            pad_about(1280, 1224, 28, 6),
            28,
            "exactly the room designed"
        );
        assert_eq!(
            pad_about(1280, 1240, 28, 6),
            20,
            "the words take some of it"
        );
        assert_eq!(
            pad_about(1280, 1268, 28, 6),
            6,
            "the words fill the width: the least"
        );
        assert_eq!(pad_about(1280, 1280, 28, 6), 6, "never under the least");
        assert_eq!(pad_about(1280, 2000, 28, 6), 6);
        assert_eq!(
            pad_about(100, 10, 4, 6),
            4,
            "a room designed under the least stays as designed"
        );
    }

    #[test]
    fn a_line_is_set_as_large_as_wanted_or_as_fits() {
        assert_eq!(
            fitted_size(288, (700, 340), (1184, 360)),
            288,
            "it fits: as wanted"
        );
        assert_eq!(
            fitted_size(288, (1900, 340), (1184, 360)),
            179,
            "too wide: smaller by as much"
        );
        assert_eq!(
            fitted_size(288, (700, 340), (1184, 170)),
            144,
            "too tall: smaller by as much"
        );
        assert_eq!(
            fitted_size(288, (1900, 340), (1184, 100)),
            84,
            "the tighter of the two decides"
        );
        assert_eq!(
            fitted_size(40, (4000, 40), (100, 100)),
            12,
            "never under twelve pixels"
        );
    }

    #[test]
    fn the_grid_is_three_by_three_above_the_bar_and_the_clock_takes_a_block_of_it() {
        // 1280 by 720 with a bar of 72: thirds of 1280 across, of 648 down.
        let grid = Grid::new((1280, 720), 72);
        assert_eq!(grid.xs, [0, 427, 853, 1280]);
        assert_eq!(grid.ys, [0, 216, 432, 648]);
        let on = |text: &str| grid.area(theme::cells(text).unwrap());
        assert_eq!(on("middle right"), (853, 216, 427, 216), "one cell");
        assert_eq!(
            on("middle left-right"),
            (0, 216, 1280, 216),
            "a row: the flip clock's"
        );
        assert_eq!(
            on("middle-bottom centre-right"),
            (427, 216, 853, 432),
            "four cells: the dial's"
        );
        assert_eq!(
            on("top-bottom left-right"),
            (0, 0, 1280, 648),
            "all nine, the bar left out"
        );
    }

    #[test]
    fn the_clock_is_aligned_inside_what_it_occupies() {
        let area = (400, 200, 600, 300);
        let at = |text: &str| aligned(area, (200, 100), theme::align(text).unwrap(), (20, 10));
        assert_eq!(at(""), (600, 300), "in the middle unless said");
        assert_eq!(
            at("left top"),
            (420, 210),
            "a margin from the sides it is aligned to"
        );
        assert_eq!(at("right bottom"), (780, 390));
        assert_eq!(at("left"), (420, 300));
        assert_eq!(at("bottom"), (600, 390));
        // Larger than its cells and in the middle: over the edges by as
        // much on either side; aligned to a side, past that side, so that
        // "left" moves it left and "top" up.
        assert_eq!(
            aligned(area, (800, 100), Align::default(), (20, 10)),
            (300, 300)
        );
        let big = |text: &str| aligned(area, (800, 400), theme::align(text).unwrap(), (20, 10));
        assert_eq!(
            big("left top"),
            (180, 90),
            "its right edge and its foot the margin from the cells'"
        );
        assert_eq!(
            big("right bottom"),
            (420, 210),
            "its left edge and its top the margin from the cells'"
        );
        assert_eq!(big(""), (300, 150), "in the middle, over both sides alike");
        assert!(
            big("left").0 < big("").0 && big("right").0 > big("").0,
            "left of the middle, right of it"
        );
        assert!(
            big("top").1 < big("").1 && big("bottom").1 > big("").1,
            "above the middle, below it"
        );
    }

    #[test]
    fn a_date_on_the_grid_leaves_the_clock_the_middle_and_shares_a_glass_with_it() {
        let fonts = Fonts::default();
        let stopped = input("stop");
        let ink = |frame: &Frame, (x, y, w, h): (i32, i32, u32, u32)| {
            let mut inked = 0;
            for row in y.max(0) as u32..(y.max(0) as u32 + h).min(frame.height) {
                for column in x.max(0) as u32..(x.max(0) as u32 + w).min(frame.width) {
                    if frame.rgba[((row * frame.width + column) * 4 + 3) as usize] > 0 {
                        inked += 1;
                    }
                }
            }
            inked
        };
        let drawn = |pairs: &[(&str, &str)]| {
            let set = settings(pairs);
            let mut v = view(&stopped, &fonts, 0);
            v.settings = &set;
            let mut frame = blank();
            Face::new().draw(&mut frame, &v);
            frame
        };
        let dial = [
            ("clock.face", "dial"),
            ("glass.frost", "off"),
            ("date.show", "on"),
        ];
        // With no font to set the date in, the clock stands as a clock
        // alone does; a date on the grid leaves it there too.
        let alone = drawn(&[dial[0], dial[1]]);
        let on_grid = drawn(&[dial[0], dial[1], dial[2], ("date.place", "top right")]);
        let upper = (427, 0, 426, 200);
        assert_eq!(
            ink(&on_grid, upper),
            ink(&alone, upper),
            "on the grid the date leaves the clock where a clock alone stands"
        );
        // With no font the date sets no type; the clock on the same cells
        // still stands in them, and nothing is drawn where neither is.
        let shared = drawn(&[
            dial[0],
            dial[1],
            dial[2],
            ("clock.place", "bottom left"),
            ("date.place", "bottom left"),
        ]);
        assert!(
            ink(&shared, (0, 432, 427, 216)) > 2000,
            "the clock in the shared cell"
        );
        assert_eq!(
            ink(&shared, (427, 0, 853, 432)),
            0,
            "and nothing elsewhere above the bar"
        );
    }

    #[test]
    fn a_clock_on_the_grid_stands_in_its_cells_and_off_it_nothing_moves() {
        let fonts = Fonts::default();
        let stopped = input("stop");
        let ink = |frame: &Frame, (x, y, w, h): (i32, i32, u32, u32)| {
            let mut inked = 0;
            for row in y.max(0) as u32..(y.max(0) as u32 + h).min(frame.height) {
                for column in x.max(0) as u32..(x.max(0) as u32 + w).min(frame.width) {
                    if frame.rgba[((row * frame.width + column) * 4 + 3) as usize] > 0 {
                        inked += 1;
                    }
                }
            }
            inked
        };
        let drawn = |pairs: &[(&str, &str)]| {
            let set = settings(pairs);
            let mut v = view(&stopped, &fonts, 0);
            v.settings = &set;
            let mut frame = blank();
            Face::new().draw(&mut frame, &v);
            frame
        };
        // A dial, so there is something to see with no font to set type in.
        let dial = [("clock.face", "dial"), ("glass.frost", "off")];
        let before = drawn(&dial);
        let placed = drawn(&[dial[0], dial[1], ("clock.place", "top left")]);
        assert!(
            ink(&placed, (0, 0, 427, 216)) > 2000,
            "the dial stands in the top left cell"
        );
        assert_eq!(
            ink(&placed, (427, 0, 853, 648)) + ink(&placed, (0, 216, 427, 432)),
            0,
            "and nowhere else above the bar"
        );
        assert!(
            ink(&before, (427, 216, 426, 216)) > 2000,
            "with no place it is in the middle, as before the grid"
        );
        assert_eq!(
            drawn(&[dial[0], dial[1], ("clock.place", "")]).rgba,
            before.rgba,
            "an empty place: as it was, to the pixel"
        );
        // A size the user set is the user's: larger than its cell, the dial
        // runs over the cell's edges, as it would over the screen's.
        let large = drawn(&[
            dial[0],
            dial[1],
            ("clock.place", "top left"),
            ("measure.clock", "400"),
        ]);
        assert!(
            ink(&large, (427, 0, 853, 648)) + ink(&large, (0, 216, 427, 432)) > 2000,
            "ink outside the cell"
        );
        // The date placed with a clock that is on the grid has the middle to itself.
        let with_date = drawn(&[
            dial[0],
            dial[1],
            ("clock.place", "top left"),
            ("date.show", "on"),
            ("date.place", "below"),
        ]);
        assert!(
            ink(&with_date, (0, 0, 427, 216)) > 2000,
            "the dial where it was placed"
        );
        // No margin: the glass stands in the corner of its cell.
        let corner = drawn(&[
            dial[0],
            dial[1],
            ("clock.place", "top right"),
            ("clock.align", "right top"),
            ("clock.margin", "0"),
            ("clock.glass", "1"),
        ]);
        assert!(
            ink(&corner, (1270, 0, 10, 10)) > 50,
            "the glass touches the top right corner"
        );
        let kept = drawn(&[
            dial[0],
            dial[1],
            ("clock.place", "top right"),
            ("clock.align", "right top"),
            ("clock.glass", "1"),
        ]);
        assert_eq!(
            ink(&kept, (1270, 0, 10, 10)),
            0,
            "with the margin of 20 it stops short"
        );
    }

    #[test]
    fn the_date_stands_at_the_top_or_with_the_clock_and_alone_takes_the_middle() {
        let (clock, date) = (Some((400, 150)), Some((300, 40)));
        // 1280 by 720, a bar of 72; a margin of 20, 8 between clock and
        // date, a glass 28 by 12 about its words.
        let m = IdleMeasure {
            margin: 20,
            gap: 8,
            pad: (28, 12),
        };
        let lay = |c, d, place| idle_layout((1280, 720), 72, m, c, d, place);
        let top = lay(clock, date, DatePlace::Top);
        // Words larger than the picture stand in its middle all the same,
        // and run over its edges by as much on either side.
        let huge = lay(Some((1600, 800)), None, DatePlace::Top);
        assert_eq!(huge.middle, Some((-160, -76, 1600, 800)));
        assert_eq!(
            lay(None, Some((1500, 40)), DatePlace::Top).top,
            Some((-110, 32, 1500, 40))
        );
        assert_eq!(
            top.top,
            Some((490, 32, 300, 40)),
            "the date in the middle of the top"
        );
        // The date and its glass take 104 of the height; the clock stands
        // in the middle of what is left, 104 to 648.
        assert_eq!(top.middle, Some((440, 301, 400, 150)));
        assert_eq!((top.clock_y, top.date_y), (Some(301), None));
        assert!(
            top.middle.unwrap().1 - 12 >= 104,
            "the clock's glass starts below the date's"
        );
        let below = lay(clock, date, DatePlace::Below);
        assert_eq!(below.top, None);
        assert_eq!(
            below.middle,
            Some((440, 225, 400, 198)),
            "one glass for both"
        );
        assert_eq!(
            (below.clock_y, below.date_y),
            (Some(225), Some(383)),
            "the clock, then the date"
        );
        let above = lay(clock, date, DatePlace::Above);
        assert_eq!(
            (above.date_y, above.clock_y),
            (Some(225), Some(273)),
            "the date, then the clock"
        );
        // No clock: a date placed with it takes the middle alone; one at the top stays there.
        let alone = lay(None, date, DatePlace::Below);
        assert_eq!(alone.middle, Some((490, 304, 300, 40)));
        assert_eq!((alone.clock_y, alone.date_y), (None, Some(304)));
        assert_eq!(lay(None, date, DatePlace::Top).middle, None);
        // No date: the clock in the middle of what the bar leaves. Neither: nothing.
        assert_eq!(
            lay(clock, None, DatePlace::Top),
            IdleLayout {
                top: None,
                middle: Some((440, 249, 400, 150)),
                clock_y: Some(249),
                date_y: None
            }
        );
        assert_eq!(lay(None, None, DatePlace::Below), IdleLayout::default());
    }

    /// A folder that holds a `face.txt` as a meter theme may: here the
    /// one a shipped look is kept in, a taller bar on black glass.
    const BRINGS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../themes/Night Drive");

    /// A picture one pixel large, red, as a PNG file's bytes.
    const RED_PNG: [u8; 69] = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8,
        0xcf, 0xc0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xdd, 0x8d, 0xb0, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    /// A forecast as the player hands it on: partly cloudy and 13.6 now,
    /// rain today between 8.9 and 17.2.
    fn forecast() -> Weather {
        Weather {
            hours: Vec::new(),
            days: Vec::new(),
            place: "Krakow".into(),
            unit: "C".into(),
            now: Some(13.6),
            code: 2,
            day: true,
            today: 61,
            low: 8.9,
            high: 17.2,
            rain: Some(64),
            at: 1_760_000_000,
        }
    }

    #[test]
    fn a_forecast_is_said_in_whole_degrees_with_a_sky_before_each_half() {
        assert_eq!(degrees(13.6), "14\u{b0}");
        assert_eq!(degrees(-0.4), "0\u{b0}", "minus nought is nought");
        assert_eq!(degrees(-3.5), "-4\u{b0}");
        // Skies 40 high: 10 to their figures, 30 between now and the day;
        // the line as high as the skies, or as the numbers with their
        // captions where those stand taller.
        assert_eq!(ForecastLine::gaps(40), (10, 30));
        assert_eq!(ForecastLine::room(40, None, 120, 36), (40 + 10 + 120, 40));
        assert_eq!(
            ForecastLine::room(40, Some(60), 120, 44),
            ((40 + 10 + 60 + 30) + (40 + 10 + 120), 44),
            "now, then the day"
        );
    }

    #[test]
    fn a_forecast_in_hand_stands_still_with_the_date_or_alone_unless_the_look_hides_it() {
        let fonts = Fonts::default();
        let without = input("stop");
        let mut with = input("stop");
        with.weather = Some(forecast());
        let bare = settings(&[("clock.show", "off"), ("date.show", "off")]);
        let hidden = settings(&[
            ("clock.show", "off"),
            ("date.show", "off"),
            ("weather.show", "off"),
        ]);
        let stands = |input: &Input, set: &BTreeMap<String, String>| {
            let mut v = view(input, &fonts, 0);
            v.settings = set;
            Face::new().advance(&v).1
        };
        assert!(!stands(&without, &bare), "no clock, no date, no forecast");
        assert!(
            stands(&with, &bare),
            "a forecast alone is something to show"
        );
        assert!(!stands(&with, &hidden), "unless the look hides it");
        // Playing, the idle screen is away with or without one.
        let mut playing = input("play");
        playing.weather = Some(forecast());
        assert!(!stands(&playing, &bare));
    }

    #[test]
    fn a_span_is_so_many_columns_each_its_hour_or_weekday() {
        let mut weather = forecast();
        weather.hours = (0..24)
            .map(|i| overlay::face::Hour {
                hour: (15 + i) % 24,
                temp: 10.0 + i as f32,
                code: 61,
                day: (6..=19).contains(&((15 + i) % 24)),
            })
            .collect();
        weather.days = (0..7)
            .map(|i| overlay::face::Day {
                weekday: (3 + i) % 7,
                code: 3,
                low: 8.0,
                high: 15.0,
            })
            .collect();
        let labels = |span: Span, twelve: bool| -> Vec<String> {
            ForecastLine::entries(&weather, span, twelve)
                .into_iter()
                .map(|e| e.0)
                .collect()
        };
        assert_eq!(
            labels(Span::Today, false),
            Vec::<String>::new(),
            "today is the line"
        );
        assert_eq!(labels(Span::Hours(2), false).len(), 12);
        assert_eq!(labels(Span::Hours(3), false).len(), 8);
        assert_eq!(labels(Span::Hours(4), false).len(), 6);
        assert_eq!(labels(Span::Hours(6), false), ["15", "21", "03", "09"]);
        assert_eq!(labels(Span::Hours(6), true), ["3pm", "9pm", "3am", "9am"]);
        assert_eq!(
            labels(Span::Week, false),
            ["Wed", "Thu", "Fri", "Sat", "Sun", "Mon", "Tue"]
        );
        let figures: Vec<String> = ForecastLine::entries(&weather, Span::Week, false)
            .into_iter()
            .map(|e| e.1.iter().map(|p| p.0.as_str()).collect::<String>())
            .collect();
        assert_eq!(figures[0], "8° / 15°");
        assert!(twelve_hour("%-I:%M %p") && twelve_hour("%l:%M"));
        assert!(!twelve_hour("%H:%M:%S"));
        // With nothing to say the columns are nothing, as a line with no face.
        let mut empty = forecast();
        empty.hours.clear();
        assert!(ForecastLine::entries(&empty, Span::Hours(2), false).is_empty());
        // The room: columns side by side with half a line between, three rows high.
        assert_eq!(
            ForecastLine::columns_room(4, 40, 16, 22, 50),
            (4 * 50 + 3 * 20, 16 + 5 + 40 + 5 + 22),
            "columns side by side with half a sky between, the rows an eighth apart"
        );
        assert_eq!(
            part(48, TODAY_SKY),
            80,
            "today's skies from its numbers, as T21"
        );
        assert_eq!(
            (part(48, TODAY_CAPTION), share_of(48, TODAY_CAPTION_UP)),
            (18, 6)
        );
        assert_eq!(
            (part(80, FIGURE), part(80, LABEL)),
            (43, 32),
            "a span's figures and labels"
        );
    }

    #[test]
    fn the_heatmap_runs_from_cold_through_the_ink_to_warm_and_the_date_may_take_it() {
        let (cold, ink, warm) = ([0, 0, 255], [100, 100, 100], [255, 0, 0]);
        assert_eq!(heat_colour(-20.0, cold, ink, warm), cold, "held below -10");
        assert_eq!(heat_colour(-10.0, cold, ink, warm), cold);
        assert_eq!(heat_colour(12.0, cold, ink, warm), ink, "the ink at 12");
        assert_eq!(heat_colour(30.0, cold, ink, warm), warm);
        assert_eq!(heat_colour(40.0, cold, ink, warm), warm, "held above 30");
        assert_eq!(
            heat_colour(1.0, cold, ink, warm),
            [50, 50, 178],
            "halfway to the ink"
        );
        assert_eq!(
            heat_colour(21.0, cold, ink, warm),
            [178, 50, 50],
            "halfway to warm"
        );
        let mut theme = Theme {
            weather_cold: cold,
            weather_warm: warm,
            ..Theme::default()
        };
        let mut weather = forecast();
        weather.now = Some(86.0);
        weather.unit = "F".to_string();
        assert_eq!(
            date_ink(&theme, Some(&weather)),
            theme.ink,
            "the date keeps its ink with the heatmap off"
        );
        theme.weather_heat = true;
        assert_eq!(
            date_ink(&theme, Some(&weather)),
            theme.ink,
            "and with the date's switch off"
        );
        theme.weather_heat_date = true;
        assert_eq!(
            date_ink(&theme, Some(&weather)),
            warm,
            "86 °F is 30 °C: warm"
        );
        assert_eq!(
            date_ink(&theme, None),
            theme.ink,
            "no reading, the date's own"
        );
        theme.weather_ink = Some(ink);
        let heat = Heat::from(&theme, &weather).unwrap();
        assert_eq!(heat.of(53.6), ink, "53.6 °F is 12 °C: the forecast's ink");
    }

    #[test]
    fn the_skies_move_only_with_the_switch_and_the_stamp_follows_their_frames() {
        let ink = [242, 242, 245];
        let mut line = ForecastLine::default();
        let rain = Skies::key(Sky::Rain, true, 12.0, 61, 32, true, ink);
        line.skies.keep(rain, Sky::Rain);
        line.now_key = Some(rain);
        let (frames, ms) = icon::cycle(Sky::Rain, true, false).unwrap();
        let stamp = |line: &ForecastLine, t: u64| {
            let mut h = DefaultHasher::new();
            line.stamp(t, &mut h);
            h.finish()
        };
        line.drama(false, false, true);
        assert_eq!(
            stamp(&line, 0),
            stamp(&line, u64::from(ms)),
            "still skies stamp nothing of the moment"
        );
        assert_eq!(
            line.skies.index(rain, u64::from(ms) * 5, false, false),
            0,
            "and stand on their first frame"
        );
        line.drama(true, false, true);
        assert_ne!(
            stamp(&line, 0),
            stamp(&line, u64::from(ms)),
            "moving skies stamp their frame"
        );
        assert_eq!(
            stamp(&line, 0),
            stamp(&line, u64::from(ms * frames)),
            "and the same frame again a cycle on"
        );
        assert_eq!(
            line.skies.index(rain, u64::from(ms) * 5, true, false),
            5 % frames as usize
        );
        assert_eq!(
            line.skies.frames[&rain].1.len(),
            frames as usize,
            "the whole cycle is kept"
        );
        let thunder = Skies::key(Sky::Thunder, true, 12.0, 95, 32, true, ink);
        line.skies.keep(thunder, Sky::Thunder);
        let lit = (0..30_000u64)
            .step_by(10)
            .find(|t| icon::flash_at(*t))
            .unwrap();
        assert_eq!(
            line.skies.index(thunder, lit, true, false),
            0,
            "no flashes unless said"
        );
        assert_eq!(
            line.skies.index(thunder, lit, true, true),
            1,
            "lit in a flash"
        );
        assert!(
            Skies::key(Sky::Clear, true, 30.0, 0, 32, true, ink).2,
            "hot at 30 °C and above"
        );
        assert!(!Skies::key(Sky::Clear, true, 29.0, 0, 32, true, ink).2);
        assert!(
            Skies::key(Sky::Rain, true, 10.0, 65, 32, true, ink).3,
            "heavy rain is heavy"
        );
        assert_eq!(celsius(86.0, "F"), 30.0);
        line.skies.keep_only(&[thunder]);
        assert!(
            !line.skies.frames.contains_key(&rain),
            "a sky off show is let go"
        );
    }

    #[test]
    fn what_the_forecast_says_is_part_of_what_is_drawn() {
        let fonts = Fonts::default();
        let mut with = input("stop");
        with.weather = Some(forecast());
        let shown = settings(&[("date.show", "on")]);
        let hidden = settings(&[("date.show", "on"), ("weather.show", "off")]);
        let stamp_of = |input: &Input, set: &BTreeMap<String, String>| {
            let mut v = view(input, &fonts, 0);
            v.settings = set;
            let mut face = Face::new();
            face.advance(&v);
            face.stamp(&v, 0, Some(&A_THURSDAY))
        };
        let first = stamp_of(&with, &shown);
        let mut warmer = with.clone();
        warmer.weather.as_mut().unwrap().high = 21.0;
        assert_ne!(
            first,
            stamp_of(&warmer, &shown),
            "another high is another picture"
        );
        let mut night = with.clone();
        night.weather.as_mut().unwrap().day = false;
        assert_ne!(
            first,
            stamp_of(&night, &shown),
            "and so is the moon for the sun"
        );
        let mut nearly = with.clone();
        nearly.weather.as_mut().unwrap().high = 17.4;
        nearly.weather.as_mut().unwrap().at += 1800;
        assert_eq!(
            first,
            stamp_of(&nearly, &shown),
            "a reading that says the same in whole degrees is drawn alike"
        );
        assert_eq!(
            stamp_of(&with, &hidden),
            stamp_of(&warmer, &hidden),
            "hidden, it changes nothing"
        );
        assert_ne!(
            first,
            stamp_of(&input("stop"), &shown),
            "none in hand is another picture"
        );
    }

    #[test]
    fn the_screen_goes_black_after_the_minutes_named_and_a_tap_wakes_it() {
        let paused = input("pause");
        let fonts = Fonts::default();
        // At once, no fade: the timeout alone is under test here.
        let set = settings(&[("idle.off", "2"), ("idle.fade", "0"), ("clock.show", "on")]);
        let at = |ms: u64| {
            let mut v = view(&paused, &fonts, ms);
            v.settings = &set;
            v
        };
        let mut face = Face::new();
        let mut frame = blank();
        // Standing still from the start: the clock, until two minutes have passed.
        face.draw(&mut frame, &at(0));
        assert!(frame.rgba.iter().any(|b| *b != 0), "the clock is drawn");
        assert_ne!(face.covers(&at(119_000)), Cover::Nothing);
        assert!(!face.dark);
        assert_eq!(
            face.covers(&at(120_000)),
            Cover::New,
            "black comes as a new picture"
        );
        let mut frame = blank();
        assert!(face.draw(&mut frame, &at(120_000)));
        assert!(face.dark);
        assert!(
            frame
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p[0] == 0 && p[1] == 0 && p[2] == 0 && p[3] == 255),
            "all black"
        );
        assert_eq!(
            face.covers(&at(120_016)),
            Cover::Same,
            "and stays, costing nothing"
        );
        // A tap wakes it, and is the wake alone: no command, no bar brought up by it.
        assert!(face.pointer(PointerKind::Down, 640, 690, &at(200_000)));
        assert!(!face.dark);
        assert!(face.pointer(PointerKind::Up, 640, 690, &at(200_050)));
        assert!(
            face.commands().is_empty(),
            "the waking tap did not act on the bar"
        );
        assert_eq!(face.covers(&at(200_100)), Cover::New);
        // Two minutes from that touch it goes black again; a touch within keeps it on.
        assert!(!face.advance(&at(319_000)).3);
        // (A touch beside the bar is the theme's, as ever; it still counts as a touch.)
        let _ = face.pointer(PointerKind::Down, 10, 10, &at(319_000));
        let _ = face.pointer(PointerKind::Up, 10, 10, &at(319_020));
        assert!(!face.advance(&at(439_000)).3);
        assert!(face.advance(&at(439_100)).3);
        // Music wakes it, and while music plays it never goes black.
        let playing = input("play");
        let mut v = view(&playing, &fonts, 440_000);
        v.settings = &set;
        assert!(!face.advance(&v).3);
        assert!(!face.dark);
        let mut v = view(&playing, &fonts, 2_000_000);
        v.settings = &set;
        assert!(!face.advance(&v).3);
        // With a fade, black comes and goes by degrees: half way at half the fade.
        let faded = settings(&[
            ("idle.off", "1"),
            ("idle.fade", "1000"),
            ("clock.show", "off"),
        ]);
        let at_f = |ms: u64| {
            let mut v = view(&paused, &fonts, ms);
            v.settings = &faded;
            v
        };
        let mut face = Face::new();
        face.draw(&mut blank(), &at_f(0));
        face.advance(&at_f(60_000));
        assert_eq!(face.black(60_000), 0);
        assert_eq!(face.covers(&at_f(60_500)), Cover::New);
        let mut frame = blank();
        face.draw(&mut frame, &at_f(60_500));
        let half = frame.rgba[3];
        assert!((120..=135).contains(&half), "half way: {half}");
        face.draw(&mut frame, &at_f(61_000));
        assert_eq!(face.black(61_000), 255);
        assert_eq!(
            face.covers(&at_f(61_100)),
            Cover::Same,
            "black whole stands"
        );
        assert!(face.pointer(PointerKind::Down, 10, 10, &at_f(70_000)));
        assert!(face.pointer(PointerKind::Up, 10, 10, &at_f(70_020)));
        assert_eq!(face.black(70_500), 128);
        assert_eq!(face.black(71_000), 0);
        assert_eq!(
            face.covers(&at_f(70_500)),
            Cover::New,
            "on its way back it is drawn"
        );
        // Off by default: standing still for hours draws the clock.
        let mut plain = Face::new();
        let v = view(&paused, &fonts, 7_200_000);
        assert!(!plain.advance(&v).3);
    }

    #[test]
    fn a_picture_of_the_users_own_stands_in_the_themes_place_when_nothing_plays() {
        let dir = std::env::temp_dir().join(format!("glass-evo-idle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("red.png"), RED_PNG).unwrap();
        let folder = dir.to_string_lossy().into_owned();
        let fonts = Fonts::default();
        let at = |frame: &Frame, x: usize, y: usize| {
            let i = (y * 1280 + x) * 4;
            [
                frame.rgba[i],
                frame.rgba[i + 1],
                frame.rgba[i + 2],
                frame.rgba[i + 3],
            ]
        };
        // Draw until the picture, read off the frame loop, is in hand.
        let draw_idle = |pairs: &[(&str, &str)]| {
            let paused = input("pause");
            let set = settings(pairs);
            let mut face = Face::new();
            face.backgrounds = Some(folder.clone());
            let mut frame = blank();
            for step in 0..400u64 {
                let mut v = view(&paused, &fonts, step * 16);
                v.settings = &set;
                frame = blank();
                face.draw(&mut frame, &v);
                if at(&frame, 5, 5)[3] != 0 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            frame
        };
        // Chosen and not darkened: the whole picture is the user's, from the
        // top to the foot, with the bar's glass over it there.
        let plain = draw_idle(&[
            ("idle.picture", "red.png"),
            ("idle.dim", "0"),
            ("clock.show", "off"),
        ]);
        assert_eq!(at(&plain, 5, 5), [255, 0, 0, 255]);
        assert_eq!(at(&plain, 1270, 400), [255, 0, 0, 255]);
        // Darkened by half.
        let dark = draw_idle(&[
            ("idle.picture", "red.png"),
            ("idle.dim", "0.5"),
            ("clock.show", "off"),
        ]);
        let half = at(&dark, 5, 5);
        assert!(
            (120..=135).contains(&half[0]) && half[1] == 0 && half[2] == 0,
            "{half:?}"
        );
        // None chosen, a name that walks, a file that is no picture: the theme stays.
        for pairs in [
            vec![("clock.show", "off")],
            vec![("idle.picture", "../red.png"), ("clock.show", "off")],
            vec![("idle.picture", "absent.png"), ("clock.show", "off")],
        ] {
            assert_eq!(at(&draw_idle(&pairs), 5, 5)[3], 0, "{pairs:?}");
        }
        // While music plays the theme is the picture, whatever is chosen.
        let playing = input("play");
        let set = settings(&[("idle.picture", "red.png"), ("idle.dim", "0")]);
        let mut face = Face::new();
        face.backgrounds = Some(folder.clone());
        let mut v = view(&playing, &fonts, 60_000);
        v.settings = &set;
        let mut frame = blank();
        for _ in 0..50 {
            face.draw(&mut frame, &v);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(at(&frame, 5, 5)[3], 0);
        assert_eq!(idle_file(&folder, " red.png "), Some(dir.join("red.png")));
        assert_eq!(idle_file(&folder, ".hidden.png"), None);
        assert_eq!(idle_file("", "red.png"), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn what_the_theme_on_show_brings_lies_over_the_look_and_under_the_settings() {
        let shipped = concat!(env!("CARGO_MANIFEST_DIR"), "/../../themes");
        let input = input("pause");
        let fonts = Fonts::default();
        // The look chosen is Warm: amber on brown, at the built-in measures.
        let chosen = settings(&[("theme", "Warm")]);
        let mut v = view(&input, &fonts, 0);
        v.settings = &chosen;
        let warm = tokens_for(&v, Some(shipped)).theme;
        assert_eq!(warm.tint, theme::Paint::Fixed([0x28, 0x12, 0x06]));
        assert_eq!(warm.measure_bar, 72.0);
        // A theme on show that brings a look: what it says stands over
        // Warm's, and Warm's stands where it says nothing.
        v.theme_dir = BRINGS;
        let brought = tokens_for(&v, Some(shipped)).theme;
        assert_eq!(brought.tint, theme::Paint::Fixed([0, 0, 0]));
        assert_eq!(brought.ink, [255, 255, 255]);
        assert_eq!(brought.measure_bar, 100.0);
        assert_eq!(
            brought.accent,
            theme::Paint::Fixed([0xff, 0xb3, 0x47]),
            "the accent is Warm's: the theme on show names none"
        );
        // The user's own settings stand over both.
        let own = settings(&[
            ("theme", "Warm"),
            ("measure.bar", "144"),
            ("colours.ink", "#00ff00"),
        ]);
        v.settings = &own;
        let adjusted = tokens_for(&v, Some(shipped)).theme;
        assert_eq!(adjusted.measure_bar, 144.0);
        assert_eq!(adjusted.ink, [0, 255, 0]);
        assert_eq!(adjusted.tint, theme::Paint::Fixed([0, 0, 0]));
        // With no look chosen it lies over the built-in one; a theme that
        // brings nothing, and no theme at all, change nothing.
        v.settings = &NO_SETTINGS;
        assert_eq!(tokens_for(&v, Some(shipped)).theme.measure_bar, 100.0);
        v.theme_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../themes");
        assert_eq!(tokens_for(&v, Some(shipped)).theme, Theme::default());
        v.theme_dir = "";
        assert_eq!(tokens_for(&v, Some(shipped)).theme, Theme::default());
    }

    #[test]
    fn the_look_is_read_again_when_the_theme_or_the_settings_change() {
        let input = input("pause");
        let fonts = Fonts::default();
        let mut frame = blank();
        let mut face = Face::new();
        let mut v = view(&input, &fonts, 0);
        assert!(face.draw(&mut frame, &v));
        assert_eq!(face.icons.as_ref().map(|set| set.size()), Some(36));
        assert_eq!(face.covers(&v), Cover::Same);
        // Another theme comes on show under the same face, and brings a look.
        v.theme_dir = BRINGS;
        assert_eq!(
            face.covers(&v),
            Cover::New,
            "nothing drawn in the look before counts as drawn"
        );
        assert!(face.draw(&mut frame, &v));
        assert_eq!(
            face.icons.as_ref().map(|set| set.size()),
            Some(50),
            "the bar is as tall as the theme on show asks"
        );
        // A touch where only the taller bar is, is the bar's.
        assert!(face.pointer(PointerKind::Down, 320, 630, &v));
        assert!(face.pointer(PointerKind::Up, 320, 630, &v));
        assert_eq!(face.commands()[0].name, "toggle");
        face.draw(&mut frame, &v);
        assert_eq!(
            face.covers(&v),
            Cover::Same,
            "the look stands while nothing changes"
        );
        // The settings change under the same face and the same theme.
        let own = settings(&[("measure.bar", "144")]);
        v.settings = &own;
        assert_eq!(face.covers(&v), Cover::New);
        face.draw(&mut frame, &v);
        assert_eq!(face.icons.as_ref().map(|set| set.size()), Some(72));
        // And back to a theme that brings nothing.
        v.theme_dir = "";
        v.settings = &NO_SETTINGS;
        face.draw(&mut frame, &v);
        assert_eq!(face.icons.as_ref().map(|set| set.size()), Some(36));
    }

    #[test]
    fn a_theme_is_the_first_of_its_name_in_the_folders_named() {
        let shipped = concat!(env!("CARGO_MANIFEST_DIR"), "/../../themes");
        let folders = format!("/nonexistent/faces:{shipped}");
        let text = theme_text(&folders, "Warm").expect("found in the second folder");
        assert!(text.contains("name = Warm"));
        assert_eq!(theme_text(&folders, "No Such Look"), None);
        assert_eq!(
            theme_text(&folders, "../themes/Warm"),
            None,
            "a name is one folder's"
        );
        assert_eq!(theme_text("", "Warm"), None);
        // The first folder wins: the user's theme of a name stands before the shipped one.
        let both = format!("{shipped}:/nonexistent/faces");
        assert_eq!(theme_text(&both, "Warm"), Some(text));
    }

    #[test]
    fn the_looks_that_ship_say_only_what_the_face_reads() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
        let known = theme::keys(&std::fs::read_to_string(dir.join("Example/face.txt")).unwrap());
        let mut looks = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let folder = entry.unwrap().path();
            let name = folder.file_name().unwrap().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(folder.join("face.txt")).unwrap();
            let said = theme::keys(&text);
            for key in said.keys() {
                assert!(
                    known.contains_key(key),
                    "{name} says {key}, which the example does not document"
                );
            }
            assert_eq!(
                said.get("theme.name").map(String::as_str),
                Some(name.as_str()),
                "{name} is named as its folder"
            );
            // Every value is one the face reads: laid over the defaults and
            // taken off again by the defaults' own words, nothing is left.
            let mut theme = Theme::default();
            theme.apply(&said);
            if name != "Example" {
                assert_ne!(
                    theme,
                    Theme {
                        name: name.clone(),
                        ..Theme::default()
                    },
                    "{name} changes something"
                );
                looks += 1;
            }
        }
        assert_eq!(looks, 4, "Dark Glass, Clear, Warm and Night Drive");
    }

    #[test]
    fn the_clock_alone_is_drawn_for_a_page_from_the_looks_keys() {
        let mut drawn = clock::Drawn::default();
        let wall: &Wall = &A_THURSDAY;
        // Set in type, or not shown: nothing, the page sets its own.
        assert!(clock_preview(&mut drawn, &NO_SETTINGS, 100, wall, 0).is_none());
        let off = settings(&[("clock.face", "dial"), ("clock.show", "off")]);
        assert!(clock_preview(&mut drawn, &off, 100, wall, 0).is_none());
        // A dial twice the size across; segments as wide as the pattern's words.
        let dial = settings(&[("clock.face", "dial"), ("clock.dial", "roman")]);
        let picture = clock_preview(&mut drawn, &dial, 100, wall, 0).expect("a dial");
        assert_eq!((picture.width, picture.height), (200, 200));
        let seven = settings(&[("clock.face", "seven"), ("clock.format", "%H:%M:%S")]);
        let picture = clock_preview(&mut drawn, &seven, 100, wall, 0).expect("segments");
        assert_eq!(
            (picture.width, picture.height),
            clock::room(clock::ClockKind::Seven, "13:05:09", 100)
        );
        // The lit segments are in the clock's ink.
        let inked = settings(&[
            ("clock.face", "seven"),
            ("clock.ink", "#ff0000"),
            ("clock.unlit", "0"),
        ]);
        let picture = clock_preview(&mut drawn, &inked, 100, wall, 0).expect("segments");
        assert!(picture
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[3] == 255 && p[0] == 255 && p[1] == 0));
    }
}
