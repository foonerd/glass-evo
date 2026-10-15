//! Lucide's icons, as data: the shapes of each on its grid of twenty-four,
//! for the face to draw at any size. Made by the journal's lucide-to-rust.py
//! from the icon files; nothing here is written by hand.
//!
//! ISC License
//!
//! Copyright (c) 2026 Lucide Icons and Contributors
//!
//! Permission to use, copy, modify, and/or distribute this software for any
//! purpose with or without fee is hereby granted, provided that the above
//! copyright notice and this permission notice appear in all copies.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
//! WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
//! MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
//! ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
//! WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
//! ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
//! OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//!
//! ---
//!
//! The following Lucide icons are derived from the Feather project:
//!
//! airplay, alert-circle, alert-octagon, alert-triangle, aperture, arrow-down-circle, arrow-down-left, arrow-down-right, arrow-down, arrow-left-circle, arrow-left, arrow-right-circle, arrow-right, arrow-up-circle, arrow-up-left, arrow-up-right, arrow-up, at-sign, calendar, cast, check, chevron-down, chevron-left, chevron-right, chevron-up, chevrons-down, chevrons-left, chevrons-right, chevrons-up, circle, clipboard, clock, code, columns, command, compass, corner-down-left, corner-down-right, corner-left-down, corner-left-up, corner-right-down, corner-right-up, corner-up-left, corner-up-right, crosshair, database, divide-circle, divide-square, dollar-sign, download, external-link, feather, frown, hash, headphones, help-circle, info, italic, key, layout, life-buoy, link-2, link, loader, lock, log-in, log-out, maximize, meh, minimize, minimize-2, minus-circle, minus-square, minus, monitor, moon, more-horizontal, more-vertical, move, music, navigation-2, navigation, octagon, pause-circle, percent, plus-circle, plus-square, plus, power, radio, rss, search, server, share, shopping-bag, sidebar, smartphone, smile, square, table-2, tablet, target, terminal, trash-2, trash, triangle, tv, type, upload, x-circle, x-octagon, x-square, x, zoom-in, zoom-out
//!
//! The MIT License (MIT) (for the icons listed above)
//!
//! Copyright (c) 2013-present Cole Bemis
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

/// A shape of an icon on Lucide's grid of twenty-four: a path in SVG's
/// own words, a circle, a rounded rectangle, a line, an open or a closed
/// run of points. Every shape is a stroke two units wide, round at its
/// ends and corners; nothing is filled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Path(&'static str),
    Circle(f32, f32, f32),
    Rect(f32, f32, f32, f32, f32),
    Line(f32, f32, f32, f32),
    Polyline(&'static [(f32, f32)]),
    Polygon(&'static [(f32, f32)]),
}

/// Lucide's `chevron-down`.
pub const CHEVRON_DOWN: &[Shape] = &[Shape::Path("m6 9 6 6 6-6")];

/// Lucide's `chevron-left`.
pub const CHEVRON_LEFT: &[Shape] = &[Shape::Path("m15 18-6-6 6-6")];

/// Lucide's `chevron-right`.
pub const CHEVRON_RIGHT: &[Shape] = &[Shape::Path("m9 18 6-6-6-6")];

/// Lucide's `disc-3`.
pub const DISC_3: &[Shape] = &[
    Shape::Circle(12.0, 12.0, 10.0),
    Shape::Path("M6 12c0-1.7.7-3.2 1.8-4.2"),
    Shape::Circle(12.0, 12.0, 2.0),
    Shape::Path("M18 12c0 1.7-.7 3.2-1.8 4.2"),
];

/// Lucide's `folder`.
pub const FOLDER: &[Shape] = &[
    Shape::Path("M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"),
];

/// Lucide's `hard-drive`.
pub const HARD_DRIVE: &[Shape] = &[
    Shape::Path("M10 16h.01"),
    Shape::Path("M2.212 11.577a2 2 0 0 0-.212.896V18a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5.527a2 2 0 0 0-.212-.896L18.55 5.11A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"),
    Shape::Path("M21.946 12.013H2.054"),
    Shape::Path("M6 16h.01"),
];

/// Lucide's `heart`.
pub const HEART: &[Shape] = &[
    Shape::Path("M2 9.5a5.5 5.5 0 0 1 9.591-3.676.56.56 0 0 0 .818 0A5.49 5.49 0 0 1 22 9.5c0 2.29-1.5 4-3 5.5l-5.492 5.313a2 2 0 0 1-3 .019L5 15c-1.5-1.5-3-3.2-3-5.5"),
];

/// Lucide's `library-big`.
pub const LIBRARY_BIG: &[Shape] = &[
    Shape::Rect(3.0, 3.0, 8.0, 18.0, 1.0),
    Shape::Path("M7 3v18"),
    Shape::Path("M20.4 18.9c.2.5-.1 1.1-.6 1.3l-1.9.7c-.5.2-1.1-.1-1.3-.6L11.1 5.1c-.2-.5.1-1.1.6-1.3l1.9-.7c.5-.2 1.1.1 1.3.6Z"),
];

/// Lucide's `list-end`.
pub const LIST_END: &[Shape] = &[
    Shape::Path("M16 5H3"),
    Shape::Path("M16 12H3"),
    Shape::Path("M9 19H3"),
    Shape::Path("m16 16-3 3 3 3"),
    Shape::Path("M21 5v12a2 2 0 0 1-2 2h-6"),
];

/// Lucide's `list-music`.
pub const LIST_MUSIC: &[Shape] = &[
    Shape::Path("M16 5H3"),
    Shape::Path("M11 12H3"),
    Shape::Path("M11 19H3"),
    Shape::Path("M21 16V5"),
    Shape::Circle(18.0, 16.0, 3.0),
];

/// Lucide's `list-plus`.
pub const LIST_PLUS: &[Shape] = &[
    Shape::Path("M16 5H3"),
    Shape::Path("M11 12H3"),
    Shape::Path("M16 19H3"),
    Shape::Path("M18 9v6"),
    Shape::Path("M21 12h-6"),
];

/// Lucide's `list`.
pub const LIST: &[Shape] = &[
    Shape::Path("M3 5h.01"),
    Shape::Path("M3 12h.01"),
    Shape::Path("M3 19h.01"),
    Shape::Path("M8 5h13"),
    Shape::Path("M8 12h13"),
    Shape::Path("M8 19h13"),
];

/// Lucide's `music`.
pub const MUSIC: &[Shape] = &[
    Shape::Path("M9 18V5l12-2v13"),
    Shape::Circle(6.0, 18.0, 3.0),
    Shape::Circle(18.0, 16.0, 3.0),
];

/// Lucide's `play`.
pub const PLAY: &[Shape] = &[Shape::Path(
    "M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z",
)];

/// Lucide's `plus`.
pub const PLUS: &[Shape] = &[Shape::Path("M5 12h14"), Shape::Path("M12 5v14")];

/// Lucide's `radio`.
pub const RADIO: &[Shape] = &[
    Shape::Path("M16.247 7.761a6 6 0 0 1 0 8.478"),
    Shape::Path("M19.075 4.933a10 10 0 0 1 0 14.134"),
    Shape::Path("M4.925 19.067a10 10 0 0 1 0-14.134"),
    Shape::Path("M7.753 16.239a6 6 0 0 1 0-8.478"),
    Shape::Circle(12.0, 12.0, 2.0),
];

/// Lucide's `search`.
pub const SEARCH: &[Shape] = &[
    Shape::Path("m21 21-4.34-4.34"),
    Shape::Circle(11.0, 11.0, 8.0),
];

/// Lucide's `star`.
pub const STAR: &[Shape] = &[
    Shape::Path("M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"),
];

/// Lucide's `usb`.
pub const USB: &[Shape] = &[
    Shape::Circle(10.0, 7.0, 1.0),
    Shape::Circle(4.0, 20.0, 1.0),
    Shape::Path("M4.7 19.3 19 5"),
    Shape::Path("m21 3-3 1 2 2Z"),
    Shape::Path("M9.26 7.68 5 12l2 5"),
    Shape::Path("m10 14 5 2 3.5-3.5"),
    Shape::Path("m18 12 1-1 1 1-1 1Z"),
];

/// Lucide's `x`.
pub const X: &[Shape] = &[Shape::Path("M18 6 6 18"), Shape::Path("m6 6 12 12")];

/// Every icon here by its Lucide name.
pub const ALL: &[(&str, &[Shape])] = &[
    ("chevron-down", CHEVRON_DOWN),
    ("chevron-left", CHEVRON_LEFT),
    ("chevron-right", CHEVRON_RIGHT),
    ("disc-3", DISC_3),
    ("folder", FOLDER),
    ("hard-drive", HARD_DRIVE),
    ("heart", HEART),
    ("library-big", LIBRARY_BIG),
    ("list-end", LIST_END),
    ("list-music", LIST_MUSIC),
    ("list-plus", LIST_PLUS),
    ("list", LIST),
    ("music", MUSIC),
    ("play", PLAY),
    ("plus", PLUS),
    ("radio", RADIO),
    ("search", SEARCH),
    ("star", STAR),
    ("usb", USB),
    ("x", X),
];
