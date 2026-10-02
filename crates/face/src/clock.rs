//! The clock's drawn faces: seven and sixteen segments, a flip clock, and a
//! dial with hands in four styles. Each is drawn from shapes given by
//! their distance from a point, as the icons are, so a face is sharp at
//! any size, takes any colours, and needs no font and no picture. What
//! does not change from one second to the next is rastered once and kept:
//! a segment's shape, a card, a dial with its marks; a second then costs
//! a few copies and the hands.

use crate::icon::{disc, polygon, segment};
use overlay::face::Frame;
use overlay::Wall;
use std::collections::BTreeMap;

/// How the clock is shown: set in the theme's type, or drawn.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum ClockKind {
    #[default]
    Type,
    Seven,
    Sixteen,
    Flip,
    Dial,
}

impl ClockKind {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "type" | "text" | "digits" => Some(Self::Type),
            "seven" | "7" | "7segment" | "seven-segment" => Some(Self::Seven),
            "sixteen" | "16" | "16segment" | "sixteen-segment" => Some(Self::Sixteen),
            "flip" => Some(Self::Flip),
            "dial" | "hands" | "analogue" | "analog" => Some(Self::Dial),
            _ => None,
        }
    }
}

/// A dial's style: bars and strong hands as on a station's clock, the
/// hours in numbers, in Roman numerals, or marks alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum DialStyle {
    #[default]
    Station,
    Numbers,
    Roman,
    Plain,
}

impl DialStyle {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "station" => Some(Self::Station),
            "numbers" | "digits" | "arabic" => Some(Self::Numbers),
            "roman" => Some(Self::Roman),
            "plain" | "simple" | "marks" => Some(Self::Plain),
            _ => None,
        }
    }
}

/// The colours a drawn clock is drawn in, the theme's and the user's
/// already laid over each face's own.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Paints {
    /// What is lit: the segments, the digits on a card.
    pub ink: [u8; 3],
    /// How much of an unlit segment shows, 0 for none.
    pub unlit: f32,
    /// The hour and the minute hand.
    pub hands: [u8; 3],
    /// The marks and the numerals of a dial.
    pub marks: [u8; 3],
    /// The second hand.
    pub second: [u8; 3],
    /// The dial's own disc behind the hands, where it has one.
    pub disc: Option<[u8; 3]>,
    /// A flip clock's cards.
    pub card: [u8; 3],
}

impl Paints {
    fn key(&self) -> String {
        format!(
            "{:?}{:.3}{:?}{:?}{:?}{:?}{:?}",
            self.ink, self.unlit, self.hands, self.marks, self.second, self.disc, self.card
        )
    }
}

/// A picture being drawn: shapes are laid over what is there, each in its
/// colour, its edge taken from the distance so it is smooth.
struct Canvas {
    frame: Frame,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        let (width, height) = (width.max(1), height.max(1));
        Self {
            frame: Frame {
                blend: Default::default(),
                width,
                height,
                rgba: vec![0; (width * height * 4) as usize],
            },
        }
    }

    /// A shape, given by its distance in pixels from a point, laid over
    /// the picture inside `bounds` (left, top, right, bottom).
    fn lay(
        &mut self,
        bounds: (f32, f32, f32, f32),
        colour: [u8; 3],
        alpha: f32,
        distance: impl Fn(f32, f32) -> f32,
    ) {
        let (w, h) = (self.frame.width as i32, self.frame.height as i32);
        let x0 = (bounds.0.floor() as i32 - 1).clamp(0, w);
        let y0 = (bounds.1.floor() as i32 - 1).clamp(0, h);
        let x1 = (bounds.2.ceil() as i32 + 1).clamp(0, w);
        let y1 = (bounds.3.ceil() as i32 + 1).clamp(0, h);
        for y in y0..y1 {
            for x in x0..x1 {
                // Half a pixel either side of the edge is the edge.
                let cover = (0.5 - distance(x as f32 + 0.5, y as f32 + 0.5)).clamp(0.0, 1.0);
                if cover > 0.0 {
                    let at = ((y * w + x) * 4) as usize;
                    over(&mut self.frame.rgba[at..at + 4], colour, cover * alpha);
                }
            }
        }
    }
}

/// A colour at an opacity over a pixel whose alpha is its own.
fn over(pixel: &mut [u8], colour: [u8; 3], alpha: f32) {
    let alpha = alpha.clamp(0.0, 1.0);
    let under = pixel[3] as f32 / 255.0;
    let out = alpha + under * (1.0 - alpha);
    if out <= 0.0 {
        return;
    }
    for c in 0..3 {
        let mixed = (colour[c] as f32 * alpha + pixel[c] as f32 * under * (1.0 - alpha)) / out;
        pixel[c] = mixed.round() as u8;
    }
    pixel[3] = (out * 255.0).round() as u8;
}

/// The bounds of a set of points, with room about them.
fn bounds(points: &[(f32, f32)], about: f32) -> (f32, f32, f32, f32) {
    let mut b = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for p in points {
        b = (b.0.min(p.0), b.1.min(p.1), b.2.max(p.0), b.3.max(p.1));
    }
    (b.0 - about, b.1 - about, b.2 + about, b.3 + about)
}

// ---- segments -------------------------------------------------------------

/// A segment display's cell, in parts of its height: the width of a
/// character, a segment's thickness, the gap where two segments meet, and
/// the room between characters.
struct Cell {
    width: f32,
    thick: f32,
    gap: f32,
    space: f32,
}

const SEVEN: Cell = Cell {
    width: 0.52,
    thick: 0.13,
    gap: 0.014,
    space: 0.16,
};
const SIXTEEN: Cell = Cell {
    width: 0.6,
    thick: 0.1,
    gap: 0.014,
    space: 0.16,
};

/// A segment lying across, pointed at both ends.
fn across(xa: f32, xb: f32, y: f32, c: &Cell) -> Vec<(f32, f32)> {
    let (t, g) = (c.thick / 2.0, c.gap);
    vec![
        (xa + g, y),
        (xa + g + t, y - t),
        (xb - g - t, y - t),
        (xb - g, y),
        (xb - g - t, y + t),
        (xa + g + t, y + t),
    ]
}

/// A segment standing upright, pointed at both ends.
fn upright(x: f32, ya: f32, yb: f32, c: &Cell) -> Vec<(f32, f32)> {
    let (t, g) = (c.thick / 2.0, c.gap);
    vec![
        (x, ya + g),
        (x + t, ya + g + t),
        (x + t, yb - g - t),
        (x, yb - g),
        (x - t, yb - g - t),
        (x - t, ya + g + t),
    ]
}

/// A segment lying aslant between two points, cut square across.
fn aslant(a: (f32, f32), b: (f32, f32), c: &Cell) -> Vec<(f32, f32)> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx.hypot(dy).max(1e-6);
    let (nx, ny) = (-dy / length * c.thick * 0.36, dx / length * c.thick * 0.36);
    vec![
        (a.0 + nx, a.1 + ny),
        (b.0 + nx, b.1 + ny),
        (b.0 - nx, b.1 - ny),
        (a.0 - nx, a.1 - ny),
    ]
}

/// The seven segments of a cell one high, in the order a b c d e f g:
/// the top, the right side down, the bottom, the left side up, the middle.
fn seven_shapes() -> Vec<Vec<(f32, f32)>> {
    let c = &SEVEN;
    let t = c.thick / 2.0;
    let (l, r, top, mid, bottom) = (t, c.width - t, t, 0.5, 1.0 - t);
    vec![
        across(l, r, top, c),
        upright(r, top, mid, c),
        upright(r, mid, bottom, c),
        across(l, r, bottom, c),
        upright(l, mid, bottom, c),
        upright(l, top, mid, c),
        across(l, r, mid, c),
    ]
}

/// The sixteen segments of a cell one high, in the order a1 a2 b c d1 d2
/// e f g1 g2 h i j k l m: the frame as for seven with the top, the bottom
/// and the middle each in two, then from the top left the three of the
/// upper half (aslant, upright, aslant) and the three of the lower.
fn sixteen_shapes() -> Vec<Vec<(f32, f32)>> {
    let c = &SIXTEEN;
    let t = c.thick / 2.0;
    let (l, r, top, mid, bottom) = (t, c.width - t, t, 0.5, 1.0 - t);
    let cx = c.width / 2.0;
    // The slanted ones keep clear of the frame and of the middle.
    let (inx, iny) = (c.thick * 0.85, c.thick * 0.8);
    vec![
        across(l, cx, top, c),
        across(cx, r, top, c),
        upright(r, top, mid, c),
        upright(r, mid, bottom, c),
        across(l, cx, bottom, c),
        across(cx, r, bottom, c),
        upright(l, mid, bottom, c),
        upright(l, top, mid, c),
        across(l, cx, mid, c),
        across(cx, r, mid, c),
        aslant((l + inx, top + iny), (cx - inx * 0.75, mid - iny * 0.9), c),
        upright(cx, top, mid, c),
        aslant((r - inx, top + iny), (cx + inx * 0.75, mid - iny * 0.9), c),
        aslant(
            (cx - inx * 0.75, mid + iny * 0.9),
            (l + inx, bottom - iny),
            c,
        ),
        upright(cx, mid, bottom, c),
        aslant(
            (cx + inx * 0.75, mid + iny * 0.9),
            (r - inx, bottom - iny),
            c,
        ),
    ]
}

/// Which of the seven segments a character lights, bit 0 for `a`; nothing
/// for a character seven segments cannot show.
fn seven_lit(ch: char) -> u32 {
    const A: u32 = 1;
    const B: u32 = 2;
    const C: u32 = 4;
    const D: u32 = 8;
    const E: u32 = 16;
    const F: u32 = 32;
    const G: u32 = 64;
    match ch.to_ascii_uppercase() {
        '0' | 'O' => A | B | C | D | E | F,
        '1' => B | C,
        '2' => A | B | D | E | G,
        '3' => A | B | C | D | G,
        '4' => B | C | F | G,
        '5' | 'S' => A | C | D | F | G,
        '6' => A | C | D | E | F | G,
        '7' => A | B | C,
        '8' => A | B | C | D | E | F | G,
        '9' => A | B | C | D | F | G,
        'A' => A | B | C | E | F | G,
        'P' => A | B | E | F | G,
        'C' => A | D | E | F,
        'E' => A | D | E | F | G,
        'F' => A | E | F | G,
        'H' => B | C | E | F | G,
        'L' => D | E | F,
        'U' => B | C | D | E | F,
        '-' => G,
        '_' => D,
        _ => 0,
    }
}

/// Which of the sixteen segments a character lights, bit 0 for `a1`.
fn sixteen_lit(ch: char) -> u32 {
    const A1: u32 = 1;
    const A2: u32 = 1 << 1;
    const B: u32 = 1 << 2;
    const C: u32 = 1 << 3;
    const D1: u32 = 1 << 4;
    const D2: u32 = 1 << 5;
    const E: u32 = 1 << 6;
    const F: u32 = 1 << 7;
    const G1: u32 = 1 << 8;
    const G2: u32 = 1 << 9;
    const H: u32 = 1 << 10;
    const I: u32 = 1 << 11;
    const J: u32 = 1 << 12;
    const K: u32 = 1 << 13;
    const L: u32 = 1 << 14;
    const M: u32 = 1 << 15;
    const TOP: u32 = A1 | A2;
    const FOOT: u32 = D1 | D2;
    const MID: u32 = G1 | G2;
    const LEFT: u32 = E | F;
    const RIGHT: u32 = B | C;
    match ch.to_ascii_uppercase() {
        '0' => TOP | RIGHT | FOOT | LEFT,
        '1' => RIGHT | J,
        '2' => TOP | B | MID | E | FOOT,
        '3' => TOP | RIGHT | FOOT | G2,
        '4' => F | MID | RIGHT,
        '5' => TOP | F | MID | C | FOOT,
        '6' => TOP | LEFT | MID | C | FOOT,
        '7' => TOP | RIGHT,
        '8' => TOP | RIGHT | FOOT | LEFT | MID,
        '9' => TOP | RIGHT | FOOT | F | MID,
        'A' => LEFT | TOP | RIGHT | MID,
        'B' => TOP | RIGHT | FOOT | I | L | G2,
        'C' => TOP | LEFT | FOOT,
        'D' => TOP | RIGHT | FOOT | I | L,
        'E' => TOP | LEFT | FOOT | G1,
        'F' => TOP | LEFT | G1,
        'G' => TOP | LEFT | FOOT | C | G2,
        'H' => LEFT | RIGHT | MID,
        'I' => TOP | FOOT | I | L,
        'J' => RIGHT | FOOT | E,
        'K' => LEFT | G1 | J | M,
        'L' => LEFT | FOOT,
        'M' => LEFT | RIGHT | H | J,
        'N' => LEFT | RIGHT | H | M,
        'O' => TOP | RIGHT | FOOT | LEFT,
        'P' => TOP | B | LEFT | MID,
        'Q' => TOP | RIGHT | FOOT | LEFT | M,
        'R' => TOP | B | LEFT | MID | M,
        'S' => TOP | F | MID | C | FOOT,
        'T' => TOP | I | L,
        'U' => LEFT | FOOT | RIGHT,
        'V' => LEFT | K | J,
        'W' => LEFT | RIGHT | K | M,
        'X' => H | J | K | M,
        'Y' => H | J | L,
        'Z' => TOP | J | K | FOOT,
        '-' => MID,
        '+' => MID | I | L,
        '/' => J | K,
        '\\' => H | M,
        '_' => FOOT,
        '*' => MID | H | I | J | K | L | M,
        _ => 0,
    }
}

/// How far a character moves the pen along, in parts of the height: a
/// cell and its room for a character of the display, less for the marks
/// between the numbers.
fn advance(ch: char, cell: &Cell) -> f32 {
    match ch {
        ':' | '.' => 0.3,
        ' ' => 0.32,
        _ => cell.width + cell.space,
    }
}

/// The characters a segment display shows of a text: all of them on
/// sixteen segments; on seven, the ones seven segments can show, a letter
/// they cannot (the M of PM) left out and not shown as a dead cell.
fn shown(text: &str, sixteen: bool) -> String {
    if sixteen {
        return text.to_string();
    }
    let kept: String = text
        .chars()
        .filter(|ch| matches!(ch, ' ' | ':' | '.') || seven_lit(*ch) != 0)
        .collect();
    kept.trim().to_string()
}

/// The room a line of segments takes at a height: its width and height.
fn segments_room(text: &str, sixteen: bool, height: u32) -> (u32, u32) {
    let cell = if sixteen { &SIXTEEN } else { &SEVEN };
    let h = height.max(1) as f32;
    let units: f32 = shown(text, sixteen)
        .chars()
        .map(|ch| advance(ch, cell))
        .sum::<f32>()
        - cell.space;
    ((units.max(0.1) * h).ceil() as u32, height.max(1))
}

/// One character of a segment display, `height` high: every segment at
/// the share an unlit one shows, the lit ones in full.
fn glyph(ch: char, sixteen: bool, height: u32, paints: &Paints) -> Frame {
    let cell = if sixteen { &SIXTEEN } else { &SEVEN };
    let shapes = if sixteen {
        sixteen_shapes()
    } else {
        seven_shapes()
    };
    let h = height.max(1) as f32;
    let mut canvas = Canvas::new((cell.width * h).ceil() as u32, height.max(1));
    let lit = if sixteen {
        sixteen_lit(ch)
    } else {
        seven_lit(ch)
    };
    for (bit, shape) in shapes.iter().enumerate() {
        let on = lit & (1 << bit) != 0;
        if !on && paints.unlit <= 0.0 {
            continue;
        }
        let points: Vec<(f32, f32)> = shape.iter().map(|p| (p.0 * h, p.1 * h)).collect();
        let alpha = if on { 1.0 } else { paints.unlit };
        canvas.lay(bounds(&points, 1.0), paints.ink, alpha, |px, py| {
            polygon((px, py), &points)
        });
    }
    canvas.frame
}

/// A picture copied into another with its top left at a point, what falls
/// outside left out.
fn put(onto: &mut Frame, picture: &Frame, x0: usize, y0: usize) {
    let (w, h) = (onto.width as usize, onto.height as usize);
    let (pw, ph) = (picture.width as usize, picture.height as usize);
    for y in 0..ph.min(h.saturating_sub(y0)) {
        let take = pw.min(w.saturating_sub(x0));
        let from = y * pw * 4;
        let to = ((y0 + y) * w + x0) * 4;
        onto.rgba[to..to + take * 4].copy_from_slice(&picture.rgba[from..from + take * 4]);
    }
}

/// A line of segments, each character rastered once and kept in `glyphs`.
fn segments(
    text: &str,
    sixteen: bool,
    height: u32,
    paints: &Paints,
    glyphs: &mut BTreeMap<char, Frame>,
) -> Frame {
    let cell = if sixteen { &SIXTEEN } else { &SEVEN };
    let (width, height) = segments_room(text, sixteen, height);
    let h = height as f32;
    let mut canvas = Canvas::new(width, height);
    let mut pen = 0.0f32;
    for ch in shown(text, sixteen).chars() {
        match ch {
            ':' | '.' => {
                let x = (pen + (0.3 - cell.space) / 2.0) * h;
                let radius = cell.thick * 0.55 * h;
                let dots: &[f32] = if ch == ':' { &[0.32, 0.68] } else { &[0.93] };
                for at in dots {
                    let (cx, cy) = (x, at * h);
                    canvas.lay(
                        (cx - radius, cy - radius, cx + radius, cy + radius),
                        paints.ink,
                        1.0,
                        |px, py| disc((px, py), (cx, cy), radius),
                    );
                }
            }
            ' ' => {}
            _ => {
                let made = glyphs
                    .entry(ch.to_ascii_uppercase())
                    .or_insert_with(|| glyph(ch, sixteen, height, paints));
                put(&mut canvas.frame, made, (pen * h).round() as usize, 0);
            }
        }
        pen += advance(ch, cell);
    }
    canvas.frame
}

// ---- strokes: the numerals of a dial and of a card -------------------------

/// A character as strokes on a grid four wide and six high, each stroke a
/// line through its points: the digits, and the letters a twelve hour
/// clock adds.
fn strokes(ch: char) -> &'static [&'static [(f32, f32)]] {
    match ch.to_ascii_uppercase() {
        '0' => &[&[
            (1.0, 0.0),
            (3.0, 0.0),
            (4.0, 1.0),
            (4.0, 5.0),
            (3.0, 6.0),
            (1.0, 6.0),
            (0.0, 5.0),
            (0.0, 1.0),
            (1.0, 0.0),
        ]],
        '1' => &[&[(0.9, 1.3), (2.4, 0.0), (2.4, 6.0)]],
        '2' => &[&[
            (0.0, 1.0),
            (1.0, 0.0),
            (3.0, 0.0),
            (4.0, 1.0),
            (4.0, 2.3),
            (0.0, 6.0),
            (4.0, 6.0),
        ]],
        '3' => &[
            &[
                (0.0, 1.0),
                (1.0, 0.0),
                (3.0, 0.0),
                (4.0, 1.0),
                (4.0, 2.0),
                (3.0, 3.0),
                (1.6, 3.0),
            ],
            &[
                (3.0, 3.0),
                (4.0, 4.0),
                (4.0, 5.0),
                (3.0, 6.0),
                (1.0, 6.0),
                (0.0, 5.0),
            ],
        ],
        '4' => &[&[(3.2, 6.0), (3.2, 0.0), (0.0, 4.2), (4.0, 4.2)]],
        '5' => &[&[
            (3.8, 0.0),
            (0.5, 0.0),
            (0.2, 2.7),
            (1.2, 2.3),
            (3.0, 2.3),
            (4.0, 3.3),
            (4.0, 5.0),
            (3.0, 6.0),
            (1.0, 6.0),
            (0.0, 5.0),
        ]],
        '6' => &[&[
            (3.7, 0.5),
            (3.0, 0.0),
            (1.0, 0.0),
            (0.0, 1.0),
            (0.0, 5.0),
            (1.0, 6.0),
            (3.0, 6.0),
            (4.0, 5.0),
            (4.0, 3.6),
            (3.0, 2.6),
            (1.0, 2.6),
            (0.0, 3.6),
        ]],
        '7' => &[&[(0.0, 0.0), (4.0, 0.0), (1.5, 6.0)]],
        '8' => &[
            &[
                (1.0, 0.0),
                (3.0, 0.0),
                (4.0, 1.0),
                (4.0, 2.0),
                (3.0, 3.0),
                (1.0, 3.0),
                (0.0, 2.0),
                (0.0, 1.0),
                (1.0, 0.0),
            ],
            &[
                (1.0, 3.0),
                (0.0, 4.0),
                (0.0, 5.0),
                (1.0, 6.0),
                (3.0, 6.0),
                (4.0, 5.0),
                (4.0, 4.0),
                (3.0, 3.0),
            ],
        ],
        '9' => &[&[
            (0.3, 5.5),
            (1.0, 6.0),
            (3.0, 6.0),
            (4.0, 5.0),
            (4.0, 1.0),
            (3.0, 0.0),
            (1.0, 0.0),
            (0.0, 1.0),
            (0.0, 2.4),
            (1.0, 3.4),
            (3.0, 3.4),
            (4.0, 2.4),
        ]],
        'A' => &[
            &[(0.0, 6.0), (2.0, 0.0), (4.0, 6.0)],
            &[(0.8, 4.0), (3.2, 4.0)],
        ],
        'P' => &[&[
            (0.0, 6.0),
            (0.0, 0.0),
            (3.0, 0.0),
            (4.0, 1.0),
            (4.0, 2.3),
            (3.0, 3.3),
            (0.0, 3.3),
        ]],
        'M' => &[&[(0.0, 6.0), (0.0, 0.0), (2.0, 3.6), (4.0, 0.0), (4.0, 6.0)]],
        'I' => &[&[(2.0, 0.0), (2.0, 6.0)]],
        'V' => &[&[(0.0, 0.0), (2.0, 6.0), (4.0, 0.0)]],
        'X' => &[&[(0.0, 0.0), (4.0, 6.0)], &[(4.0, 0.0), (0.0, 6.0)]],
        _ => &[],
    }
}

/// How wide a stroked character is on its grid, a Roman `I` being narrow.
fn stroke_width(ch: char) -> f32 {
    match ch.to_ascii_uppercase() {
        'I' => 0.0,
        '1' => 2.6,
        _ => 4.0,
    }
}

/// A line of stroked characters `height` high with its middle at a point,
/// each stroke `half` thick either side, upright.
fn stroke_text(
    canvas: &mut Canvas,
    text: &str,
    middle: (f32, f32),
    height: f32,
    half: f32,
    colour: [u8; 3],
) {
    let unit = height / 6.0;
    let gap = 1.3;
    let widths: Vec<f32> = text.chars().map(stroke_width).collect();
    let total: f32 = widths.iter().sum::<f32>() + gap * (widths.len().saturating_sub(1)) as f32;
    let mut pen = middle.0 - total * unit / 2.0;
    let top = middle.1 - height / 2.0;
    for (ch, width) in text.chars().zip(widths) {
        // A narrow character stands in the middle of its own width.
        let shift = if ch.eq_ignore_ascii_case(&'I') {
            -2.0
        } else if ch == '1' {
            -0.7
        } else {
            0.0
        };
        for stroke in strokes(ch) {
            let points: Vec<(f32, f32)> = stroke
                .iter()
                .map(|p| (pen + (p.0 + shift) * unit, top + p.1 * unit))
                .collect();
            canvas.lay(bounds(&points, half + 1.0), colour, 1.0, |px, py| {
                points
                    .windows(2)
                    .map(|w| segment((px, py), w[0], w[1], half))
                    .fold(f32::MAX, f32::min)
            });
        }
        pen += (width + gap) * unit;
    }
}

// ---- the dial ---------------------------------------------------------------

/// A point on a dial: `turn` of a full turn clockwise from twelve, `r`
/// from the middle.
fn on_dial(centre: (f32, f32), turn: f32, r: f32) -> (f32, f32) {
    let angle = turn * std::f32::consts::TAU;
    (centre.0 + r * angle.sin(), centre.1 - r * angle.cos())
}

/// A bar along a ray of the dial from `from` to `to` of the radius, as wide
/// as `near` at its inner end and `far` at its outer.
fn bar(centre: (f32, f32), turn: f32, from: f32, to: f32, near: f32, far: f32) -> Vec<(f32, f32)> {
    let angle = turn * std::f32::consts::TAU;
    let (dx, dy) = (angle.sin(), -angle.cos());
    let (nx, ny) = (-dy, dx);
    let a = (centre.0 + dx * from, centre.1 + dy * from);
    let b = (centre.0 + dx * to, centre.1 + dy * to);
    vec![
        (a.0 + nx * near / 2.0, a.1 + ny * near / 2.0),
        (b.0 + nx * far / 2.0, b.1 + ny * far / 2.0),
        (b.0 - nx * far / 2.0, b.1 - ny * far / 2.0),
        (a.0 - nx * near / 2.0, a.1 - ny * near / 2.0),
    ]
}

const ROMAN: [&str; 12] = [
    "XII", "I", "II", "III", "IIII", "V", "VI", "VII", "VIII", "IX", "X", "XI",
];

/// What stands still of a dial `2 * radius` across: its disc, its rim, its
/// marks and its numerals.
fn dial_still(style: DialStyle, radius: u32, paints: &Paints) -> Frame {
    let size = radius.max(8) * 2;
    let r = size as f32 / 2.0;
    let c = (r, r);
    let mut canvas = Canvas::new(size, size);
    let whole = (0.0, 0.0, size as f32, size as f32);
    if let Some(colour) = paints.disc {
        canvas.lay(whole, colour, 1.0, |px, py| disc((px, py), c, r * 0.985));
        // A rim about the disc, in the marks' colour.
        canvas.lay(whole, paints.marks, 1.0, |px, py| {
            (disc((px, py), c, r * 0.97).abs()) - r * 0.014
        });
    }
    let mark = |canvas: &mut Canvas, turn: f32, from: f32, to: f32, width: f32| {
        let points = bar(c, turn, r * from, r * to, r * width, r * width);
        canvas.lay(bounds(&points, 1.0), paints.marks, 1.0, |px, py| {
            polygon((px, py), &points)
        });
    };
    match style {
        DialStyle::Station => {
            for minute in 0..60 {
                let turn = minute as f32 / 60.0;
                if minute % 5 == 0 {
                    mark(&mut canvas, turn, 0.70, 0.92, 0.062);
                } else {
                    mark(&mut canvas, turn, 0.855, 0.92, 0.022);
                }
            }
        }
        DialStyle::Plain => {
            for minute in 0..60 {
                let turn = minute as f32 / 60.0;
                if minute % 15 == 0 {
                    mark(&mut canvas, turn, 0.76, 0.93, 0.04);
                } else if minute % 5 == 0 {
                    mark(&mut canvas, turn, 0.82, 0.93, 0.026);
                } else {
                    mark(&mut canvas, turn, 0.895, 0.93, 0.012);
                }
            }
        }
        DialStyle::Numbers | DialStyle::Roman => {
            for minute in 0..60 {
                let turn = minute as f32 / 60.0;
                if minute % 5 == 0 {
                    mark(&mut canvas, turn, 0.895, 0.945, 0.024);
                } else {
                    mark(&mut canvas, turn, 0.915, 0.945, 0.01);
                }
            }
            for (hour, roman) in ROMAN.iter().enumerate() {
                let middle = on_dial(c, hour as f32 / 12.0, r * 0.72);
                if style == DialStyle::Roman {
                    stroke_text(&mut canvas, roman, middle, r * 0.2, r * 0.016, paints.marks);
                } else {
                    let text = if hour == 0 {
                        "12".to_string()
                    } else {
                        hour.to_string()
                    };
                    stroke_text(&mut canvas, &text, middle, r * 0.21, r * 0.02, paints.marks);
                }
            }
        }
    }
    canvas.frame
}

/// The hands over a dial at a time of day; the second hand where the
/// clock shows seconds.
fn dial_hands(
    still: &Frame,
    style: DialStyle,
    wall: &Wall,
    seconds: bool,
    paints: &Paints,
) -> Frame {
    let mut canvas = Canvas {
        frame: still.clone(),
    };
    let r = still.width as f32 / 2.0;
    let c = (r, r);
    let second = if seconds { wall.second as f32 } else { 0.0 };
    let minute = wall.minute as f32 + second / 60.0;
    let hour = (wall.hour % 12) as f32 + minute / 60.0;
    let station = style == DialStyle::Station;
    let hand = |canvas: &mut Canvas,
                turn: f32,
                from: f32,
                to: f32,
                near: f32,
                far: f32,
                colour: [u8; 3]| {
        let points = bar(c, turn, r * from, r * to, r * near, r * far);
        canvas.lay(bounds(&points, 1.0), colour, 1.0, |px, py| {
            polygon((px, py), &points)
        });
    };
    if station {
        hand(
            &mut canvas,
            hour / 12.0,
            -0.2,
            0.6,
            0.105,
            0.085,
            paints.hands,
        );
        hand(
            &mut canvas,
            minute / 60.0,
            -0.2,
            0.9,
            0.085,
            0.062,
            paints.hands,
        );
    } else {
        // Round at their ends: a line with its thickness about it.
        for (turn, to, half) in [(hour / 12.0, 0.52, 0.034), (minute / 60.0, 0.8, 0.024)] {
            let (a, b) = (on_dial(c, turn, -r * 0.12), on_dial(c, turn, r * to));
            canvas.lay(
                bounds(&[a, b], r * half + 1.0),
                paints.hands,
                1.0,
                |px, py| segment((px, py), a, b, r * half),
            );
        }
    }
    if seconds {
        let turn = second / 60.0;
        if station {
            hand(&mut canvas, turn, -0.28, 0.62, 0.024, 0.024, paints.second);
            let at = on_dial(c, turn, r * 0.62);
            let radius = r * 0.088;
            canvas.lay(
                (at.0 - radius, at.1 - radius, at.0 + radius, at.1 + radius),
                paints.second,
                1.0,
                |px, py| disc((px, py), at, radius),
            );
        } else {
            let (a, b) = (on_dial(c, turn, -r * 0.2), on_dial(c, turn, r * 0.86));
            canvas.lay(
                bounds(&[a, b], r * 0.008 + 1.0),
                paints.second,
                1.0,
                |px, py| segment((px, py), a, b, r * 0.008),
            );
        }
    }
    // The cap over where the hands meet.
    let (cap, colour) = if seconds {
        (r * 0.04, paints.second)
    } else {
        (r * 0.05, paints.hands)
    };
    canvas.lay(
        (c.0 - cap, c.1 - cap, c.0 + cap, c.1 + cap),
        colour,
        1.0,
        |px, py| disc((px, py), c, cap),
    );
    canvas.frame
}

// ---- the flip clock ---------------------------------------------------------

/// How long a card takes to fall, in milliseconds.
pub const FLIP_MS: u64 = 450;

/// A card's width, the room between two cards, and the room a colon
/// takes, in parts of the card's height.
const CARD: (f32, f32, f32) = (0.64, 0.07, 0.26);

fn flip_advance(ch: char) -> f32 {
    match ch {
        ':' | '.' => CARD.2,
        ' ' => 0.2,
        _ => CARD.0 + CARD.1,
    }
}

fn flip_room(text: &str, height: u32) -> (u32, u32) {
    let h = height.max(1) as f32;
    let units: f32 = text.chars().map(flip_advance).sum::<f32>() - CARD.1;
    ((units.max(0.1) * h).ceil() as u32, height.max(1))
}

/// The distance to a box with rounded corners.
fn rounded(p: (f32, f32), min: (f32, f32), max: (f32, f32), radius: f32) -> f32 {
    let centre = ((min.0 + max.0) / 2.0, (min.1 + max.1) / 2.0);
    let half = (
        (max.0 - min.0) / 2.0 - radius,
        (max.1 - min.1) / 2.0 - radius,
    );
    let q = (
        (p.0 - centre.0).abs() - half.0,
        (p.1 - centre.1).abs() - half.1,
    );
    q.0.max(0.0).hypot(q.1.max(0.0)) + q.0.max(q.1).min(0.0) - radius
}

/// One card with its character, `height` high.
fn card(ch: char, height: u32, paints: &Paints) -> Frame {
    let h = height.max(1) as f32;
    let width = (CARD.0 * h).ceil() as u32;
    let mut canvas = Canvas::new(width, height.max(1));
    let (w, radius) = (width as f32, h * 0.07);
    canvas.lay((0.0, 0.0, w, h), paints.card, 1.0, |px, py| {
        rounded((px, py), (0.0, 0.0), (w, h), radius)
    });
    stroke_text(
        &mut canvas,
        &ch.to_string(),
        (w / 2.0, h / 2.0),
        h * 0.62,
        h * 0.055,
        paints.ink,
    );
    canvas.frame
}

/// A row of a card, cut through at the hinge: the line across the middle
/// where the two halves meet is left clear.
fn hinge(frame: &mut Frame) {
    let (w, h) = (frame.width as usize, frame.height as usize);
    let thick = (h / 90).max(1);
    for y in (h / 2).saturating_sub(thick / 2)..(h / 2 + thick.div_ceil(2)).min(h) {
        for x in 0..w {
            frame.rgba[(y * w + x) * 4 + 3] = 0;
        }
    }
}

/// A card on its way from one character to the next, `phase` of the way
/// from 0 to 1: the old upper half falls away over the new one, then the
/// new lower half falls into place over the old.
fn falling(old: &Frame, new: &Frame, phase: f32) -> Frame {
    let (w, h) = (new.width as usize, new.height as usize);
    if old.width != new.width || old.height != new.height {
        return new.clone();
    }
    let mut out = new.clone();
    let mid = h / 2;
    fn row(frame: &Frame, y: usize) -> &[u8] {
        let w = frame.width as usize;
        &frame.rgba[y * w * 4..(y + 1) * w * 4]
    }
    let fold = (phase.clamp(0.0, 1.0) * std::f32::consts::PI).cos();
    // The lower half: the old one, until the flap comes down over it.
    for y in mid..h {
        out.rgba[y * w * 4..(y + 1) * w * 4].copy_from_slice(row(old, y));
    }
    let flap = |out: &mut Frame, source: &Frame, share: f32, upper: bool| {
        // The flap seen at an angle is its half made shorter, and darker
        // the more it is turned away.
        let rows = (mid as f32 * share).round() as usize;
        let shade = 0.45 + 0.55 * share;
        for k in 0..rows {
            let from = ((k as f32 + 0.5) / share).floor() as usize;
            let (y, sy) = if upper {
                (mid - 1 - k, mid.saturating_sub(1 + from.min(mid - 1)))
            } else {
                (mid + k, (mid + from).min(h - 1))
            };
            let line = row(source, sy).to_vec();
            let target = &mut out.rgba[y * w * 4..(y + 1) * w * 4];
            for (pixel, src) in target
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(line.as_chunks::<4>().0.iter())
            {
                pixel[0] = (src[0] as f32 * shade) as u8;
                pixel[1] = (src[1] as f32 * shade) as u8;
                pixel[2] = (src[2] as f32 * shade) as u8;
                pixel[3] = src[3];
            }
        }
    };
    if fold > 0.0 {
        flap(&mut out, old, fold, true);
    } else {
        flap(&mut out, new, -fold, false);
    }
    out
}

/// A flip clock's line: a card for every character, each on its way from
/// the character before where it has just changed.
fn flip(
    text: &str,
    before: &[(char, f32)],
    height: u32,
    paints: &Paints,
    cards: &mut BTreeMap<char, Frame>,
) -> Frame {
    let (width, height) = flip_room(text, height);
    let h = height as f32;
    let mut canvas = Canvas::new(width, height);
    let mut pen = 0.0f32;
    for (at, ch) in text.chars().enumerate() {
        match ch {
            ':' | '.' => {
                let x = (pen + (CARD.2 - CARD.1) / 2.0) * h;
                let radius = h * 0.04;
                let dots: &[f32] = if ch == ':' { &[0.34, 0.66] } else { &[0.9] };
                for dot in dots {
                    let (cx, cy) = (x, dot * h);
                    canvas.lay(
                        (cx - radius, cy - radius, cx + radius, cy + radius),
                        paints.ink,
                        1.0,
                        |px, py| disc((px, py), (cx, cy), radius),
                    );
                }
            }
            ' ' => {}
            _ => {
                let mut made = |ch: char| {
                    cards
                        .entry(ch)
                        .or_insert_with(|| card(ch, height, paints))
                        .clone()
                };
                let new = made(ch);
                let mut shown = match before.get(at) {
                    Some(&(old, phase))
                        if old != ch && phase < 1.0 && !matches!(old, ':' | '.' | ' ') =>
                    {
                        falling(&made(old), &new, phase)
                    }
                    _ => new,
                };
                hinge(&mut shown);
                put(&mut canvas.frame, &shown, (pen * h).round() as usize, 0);
            }
        }
        pen += flip_advance(ch);
    }
    canvas.frame
}

// ---- the clock as drawn, kept between frames -----------------------------------

/// What a drawn clock is asked to show.
pub struct Asked<'a> {
    pub kind: ClockKind,
    pub dial: DialStyle,
    pub paints: Paints,
    /// The time set in the clock's pattern, for the faces that show characters.
    pub text: &'a str,
    pub wall: &'a Wall,
    /// Whether the pattern shows seconds: a dial then has its second hand.
    pub seconds: bool,
    /// The display's clock, for a card that falls.
    pub now_ms: u64,
}

/// The room a drawn clock takes at a size: a line of characters `size`
/// high, a dial twice that across.
pub fn room(kind: ClockKind, text: &str, size: u32) -> (u32, u32) {
    match kind {
        ClockKind::Seven => segments_room(text, false, size),
        ClockKind::Sixteen => segments_room(text, true, size),
        ClockKind::Flip => flip_room(text, size),
        ClockKind::Dial => (size.max(8) * 2, size.max(8) * 2),
        ClockKind::Type => (0, 0),
    }
}

/// A drawn clock, kept between frames: the picture as last drawn and what
/// it was drawn of, the parts that stand still, and for a flip clock the
/// characters on their way.
#[derive(Default)]
pub struct Drawn {
    key: String,
    frame: Option<Frame>,
    still: Option<(String, Frame)>,
    /// The characters as rastered, cards or segments, and what they are of.
    cards: BTreeMap<char, Frame>,
    cards_for: String,
    /// The characters shown, the ones before them, and when each changed.
    shown: Vec<char>,
    before: Vec<char>,
    changed_at: Vec<u64>,
}

impl Drawn {
    /// Whether a card is on its way at `now_ms`: the clock is then drawn
    /// anew every frame.
    pub fn moving(&self, now_ms: u64) -> bool {
        self.changed_at
            .iter()
            .any(|at| *at != 0 && now_ms.saturating_sub(*at) < FLIP_MS)
    }

    /// The clock at `size`, drawn if what it shows has changed; its room.
    pub fn set(&mut self, asked: &Asked, size: u32) -> Option<(u32, u32)> {
        let size = size.max(8);
        let paints = asked.paints.key();
        if asked.kind == ClockKind::Flip {
            // A character that changes starts to fall; the first ones shown do not.
            let chars: Vec<char> = asked.text.chars().collect();
            if self.shown.len() != chars.len() {
                self.before = chars.clone();
                self.changed_at = vec![0; chars.len()];
            } else {
                for (at, ch) in chars.iter().enumerate() {
                    if self.shown[at] != *ch {
                        self.before[at] = self.shown[at];
                        self.changed_at[at] = asked.now_ms.max(1);
                    }
                }
            }
            self.shown = chars;
        }
        let moving = asked.kind == ClockKind::Flip && self.moving(asked.now_ms);
        let key = match asked.kind {
            ClockKind::Dial => format!(
                "dial {:?} {size} {paints} {}:{}:{}",
                asked.dial,
                asked.wall.hour,
                asked.wall.minute,
                if asked.seconds { asked.wall.second } else { 0 }
            ),
            kind => format!(
                "{kind:?} {size} {paints} {} {}",
                asked.text,
                if moving { asked.now_ms } else { 0 }
            ),
        };
        if self.key != key || self.frame.is_none() {
            let frame = match asked.kind {
                ClockKind::Type => return None,
                ClockKind::Seven | ClockKind::Sixteen => {
                    self.keep_for(format!("{:?} {size} {paints}", asked.kind));
                    let sixteen = asked.kind == ClockKind::Sixteen;
                    segments(asked.text, sixteen, size, &asked.paints, &mut self.cards)
                }
                ClockKind::Flip => {
                    self.keep_for(format!("flip {size} {paints}"));
                    let before: Vec<(char, f32)> = self
                        .before
                        .iter()
                        .zip(self.changed_at.iter())
                        .map(|(ch, at)| {
                            let phase = if *at == 0 {
                                1.0
                            } else {
                                asked.now_ms.saturating_sub(*at) as f32 / FLIP_MS as f32
                            };
                            (*ch, phase)
                        })
                        .collect();
                    flip(asked.text, &before, size, &asked.paints, &mut self.cards)
                }
                ClockKind::Dial => {
                    let still_for = format!("{:?} {size} {paints}", asked.dial);
                    if self.still.as_ref().is_none_or(|(k, _)| *k != still_for) {
                        self.still = Some((still_for, dial_still(asked.dial, size, &asked.paints)));
                    }
                    let still = &self.still.as_ref()?.1;
                    dial_hands(still, asked.dial, asked.wall, asked.seconds, &asked.paints)
                }
            };
            self.frame = Some(frame);
            self.key = key;
        }
        self.frame.as_ref().map(|f| (f.width, f.height))
    }

    /// The characters kept are those of one face at one size in one set
    /// of colours: anything else, and they are made anew.
    fn keep_for(&mut self, what: String) {
        if self.cards_for != what {
            self.cards.clear();
            self.cards_for = what;
        }
    }

    /// The picture as last drawn.
    pub fn frame(&self) -> Option<&Frame> {
        self.frame.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAINTS: Paints = Paints {
        ink: [240, 240, 240],
        unlit: 0.1,
        hands: [20, 20, 20],
        marks: [20, 20, 20],
        second: [214, 42, 30],
        disc: Some([245, 245, 242]),
        card: [23, 23, 26],
    };

    /// How much of a picture's part is covered, as a share of full cover.
    fn covered(frame: &Frame, part: (u32, u32, u32, u32)) -> f32 {
        let mut sum = 0u64;
        for y in part.1..part.3 {
            for x in part.0..part.2 {
                sum += frame.rgba[((y * frame.width + x) * 4 + 3) as usize] as u64;
            }
        }
        sum as f32 / (255.0 * ((part.2 - part.0) * (part.3 - part.1)) as f32)
    }

    #[test]
    fn the_names_of_the_faces_and_the_styles() {
        assert_eq!(ClockKind::parse(" Seven "), Some(ClockKind::Seven));
        assert_eq!(ClockKind::parse("16"), Some(ClockKind::Sixteen));
        assert_eq!(ClockKind::parse("flip"), Some(ClockKind::Flip));
        assert_eq!(ClockKind::parse("DIAL"), Some(ClockKind::Dial));
        assert_eq!(ClockKind::parse("type"), Some(ClockKind::Type));
        assert_eq!(ClockKind::parse("sundial"), None);
        assert_eq!(DialStyle::parse("Roman"), Some(DialStyle::Roman));
        assert_eq!(DialStyle::parse("numbers"), Some(DialStyle::Numbers));
        assert_eq!(DialStyle::parse("plain"), Some(DialStyle::Plain));
        assert_eq!(DialStyle::parse("station"), Some(DialStyle::Station));
        assert_eq!(DialStyle::parse("cuckoo"), None);
    }

    #[test]
    fn every_digit_lights_its_own_segments() {
        // Seven segments: each digit differs from every other, an eight
        // lights them all, and a one the two at the right.
        let sevens: Vec<u32> = ('0'..='9').map(seven_lit).collect();
        for (i, a) in sevens.iter().enumerate() {
            for b in &sevens[i + 1..] {
                assert_ne!(a, b);
            }
        }
        assert_eq!(seven_lit('8'), 0b111_1111);
        assert_eq!(seven_lit('1'), 0b000_0110);
        assert_eq!(seven_lit(' '), 0);
        assert_eq!(seven_shapes().len(), 7);
        // Sixteen: every digit and letter lights something, no two alike
        // but the pairs that are one shape (O and 0 differ by the slash).
        assert_eq!(sixteen_shapes().len(), 16);
        let mut seen = BTreeMap::new();
        for ch in ('0'..='9').chain('A'..='Z') {
            let lit = sixteen_lit(ch);
            assert_ne!(lit, 0, "{ch} lights nothing");
            if let Some(other) = seen.insert(lit, ch) {
                assert!(
                    matches!((other, ch), ('5', 'S') | ('0', 'O')),
                    "{other} and {ch} light the same segments"
                );
            }
        }
        assert_eq!(
            sixteen_lit('m'),
            sixteen_lit('M'),
            "small letters as capitals"
        );
    }

    #[test]
    fn a_line_of_segments_is_as_wide_as_its_characters() {
        let paints = PAINTS;
        let room = room(ClockKind::Seven, "12:34", 100);
        assert_eq!(room.1, 100);
        let mut kept = BTreeMap::new();
        let frame = segments("12:34", false, 100, &paints, &mut kept);
        assert_eq!(kept.len(), 4, "each character rastered once");
        assert_eq!((frame.width, frame.height), room);
        // The same room whatever the digits: the glass stands still.
        assert_eq!(room, super::room(ClockKind::Seven, "88:88", 100));
        // An eight covers more than a one, and an unlit segment shows faintly.
        let eight = segments("8", false, 100, &paints, &mut BTreeMap::new());
        let one = segments("1", false, 100, &paints, &mut BTreeMap::new());
        let all = (0, 0, eight.width, eight.height);
        assert!(covered(&eight, all) > covered(&one, all) * 1.5);
        let dark = Paints {
            unlit: 0.0,
            ..paints
        };
        assert!(
            covered(&one, all)
                > covered(&segments("1", false, 100, &dark, &mut BTreeMap::new()), all)
        );
        // Sixteen segments are wider, and take letters.
        assert!(super::room(ClockKind::Sixteen, "12:34", 100).0 > room.0);
        let word = segments("PM", true, 60, &dark, &mut BTreeMap::new());
        assert!(covered(&word, (0, 0, word.width, word.height)) > 0.1);
        // Seven segments leave out what they cannot show, and keep no dead cell for it.
        assert_eq!(shown("1:05 PM", false), "1:05 P");
        assert_eq!(shown("1:05 PM", true), "1:05 PM");
        assert_eq!(
            super::room(ClockKind::Seven, "1:05 PM", 100),
            super::room(ClockKind::Seven, "1:05 P", 100)
        );
    }

    #[test]
    fn a_dial_stands_still_but_for_its_hands() {
        let paints = PAINTS;
        let still = dial_still(DialStyle::Station, 100, &paints);
        assert_eq!((still.width, still.height), (200, 200));
        assert_eq!(room(ClockKind::Dial, "", 100), (200, 200));
        // The disc is there in the middle, and not in the corner.
        assert_eq!(still.rgba[(100 * 200 + 100) * 4 + 3], 255);
        assert_eq!(still.rgba[3], 0);
        // Three o'clock: the minute hand up, the hour hand to the right.
        let three = Wall::at(1_790_780_400_000, 0, "UTC");
        assert_eq!((three.hour, three.minute, three.second), (15, 0, 0));
        let hands = dial_hands(&still, DialStyle::Station, &three, false, &paints);
        let dark = |frame: &Frame, x: u32, y: u32| frame.rgba[((y * 200 + x) * 4) as usize] < 80;
        assert!(dark(&hands, 100, 40), "the minute hand at twelve");
        assert!(dark(&hands, 140, 100), "the hour hand at three");
        assert!(
            !dark(&hands, 100, 150) && !dark(&hands, 60, 100),
            "and nowhere else"
        );
        // The second hand only where the clock shows seconds, in its own colour.
        let later = Wall::at(1_790_780_430_000, 0, "UTC");
        let with = dial_hands(&still, DialStyle::Station, &later, true, &paints);
        let at = ((162 * 200 + 100) * 4) as usize;
        assert_eq!(
            &with.rgba[at..at + 3],
            &[214, 42, 30],
            "thirty seconds: straight down"
        );
        let without = dial_hands(&still, DialStyle::Station, &later, false, &paints);
        assert_ne!(&without.rgba[at..at + 3], &[214, 42, 30]);
        // Every style draws, with and without a disc of its own.
        for style in [DialStyle::Numbers, DialStyle::Roman, DialStyle::Plain] {
            let bare = Paints {
                disc: None,
                ..paints
            };
            let frame = dial_still(style, 120, &bare);
            assert!(
                covered(&frame, (0, 0, 240, 240)) > 0.01,
                "{style:?} has its marks"
            );
            assert_eq!(
                frame.rgba[(120 * 240 + 120) * 4 + 3],
                0,
                "{style:?} has no disc"
            );
        }
    }

    #[test]
    fn a_card_falls_from_one_character_to_the_next() {
        let paints = PAINTS;
        let mut drawn = Drawn::default();
        let wall = Wall::default();
        let ask = |drawn: &mut Drawn, text: &str, now_ms: u64| {
            drawn.set(
                &Asked {
                    kind: ClockKind::Flip,
                    dial: DialStyle::Station,
                    paints,
                    text,
                    wall: &wall,
                    seconds: true,
                    now_ms,
                },
                120,
            )
        };
        let room = ask(&mut drawn, "12:07", 1000).expect("a clock");
        assert_eq!(room, super::room(ClockKind::Flip, "12:07", 120));
        assert!(
            !drawn.moving(1000),
            "the first characters shown do not fall"
        );
        let first = drawn.frame().unwrap().clone();
        // The same again: nothing is drawn anew.
        ask(&mut drawn, "12:07", 1100);
        assert_eq!(drawn.frame().unwrap().rgba, first.rgba);
        // The minute turns: its card is on its way, and the picture changes
        // from frame to frame until it has fallen.
        ask(&mut drawn, "12:08", 2000);
        assert!(drawn.moving(2000) && drawn.moving(2000 + FLIP_MS - 1));
        let started = drawn.frame().unwrap().clone();
        ask(&mut drawn, "12:08", 2000 + FLIP_MS / 2);
        let half = drawn.frame().unwrap().clone();
        assert_ne!(started.rgba, half.rgba);
        ask(&mut drawn, "12:08", 2000 + FLIP_MS);
        assert!(!drawn.moving(2000 + FLIP_MS));
        let fallen = drawn.frame().unwrap().clone();
        assert_ne!(half.rgba, fallen.rgba);
        ask(&mut drawn, "12:08", 2000 + FLIP_MS + 500);
        assert_eq!(
            drawn.frame().unwrap().rgba,
            fallen.rgba,
            "and then it stands"
        );
        // The cards that did not change are as they were.
        let card_w = (CARD.0 * 120.0).ceil() as usize;
        for y in [10usize, 60, 110] {
            let row = y * first.width as usize * 4;
            assert_eq!(
                first.rgba[row..row + card_w * 4],
                fallen.rgba[row..row + card_w * 4]
            );
        }
    }

    #[test]
    fn a_colour_over_a_pixel_keeps_what_shows_of_both() {
        let mut pixel = [0u8, 0, 0, 0];
        over(&mut pixel, [200, 100, 50], 0.5);
        assert_eq!(
            pixel,
            [200, 100, 50, 128],
            "over nothing: the colour at its share"
        );
        over(&mut pixel, [0, 0, 0], 1.0);
        assert_eq!(
            pixel,
            [0, 0, 0, 255],
            "a solid colour over anything is itself"
        );
        let mut solid = [255u8, 255, 255, 255];
        over(&mut solid, [0, 0, 0], 0.5);
        assert_eq!(solid, [128, 128, 128, 255]);
    }
}
