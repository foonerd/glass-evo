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

/// Lucide's `cast`.
pub const CAST: &[Shape] = &[
    Shape::Path("M2 8V6a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-6"),
    Shape::Path("M2 12a9 9 0 0 1 8 8"),
    Shape::Path("M2 16a5 5 0 0 1 4 4"),
    Shape::Line(0.0, 0.0, 0.0, 0.0),
];

/// Lucide's `chevron-down`.
pub const CHEVRON_DOWN: &[Shape] = &[Shape::Path("m6 9 6 6 6-6")];

/// Lucide's `chevron-left`.
pub const CHEVRON_LEFT: &[Shape] = &[Shape::Path("m15 18-6-6 6-6")];

/// Lucide's `chevron-right`.
pub const CHEVRON_RIGHT: &[Shape] = &[Shape::Path("m9 18 6-6-6-6")];

/// Lucide's `cloud`.
pub const CLOUD: &[Shape] = &[Shape::Path(
    "M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z",
)];

/// Lucide's `cpu`.
pub const CPU: &[Shape] = &[
    Shape::Rect(4.0, 4.0, 16.0, 16.0, 2.0),
    Shape::Rect(9.0, 9.0, 6.0, 6.0, 1.0),
    Shape::Path("M15 2v2"),
    Shape::Path("M15 20v2"),
    Shape::Path("M2 15h2"),
    Shape::Path("M2 9h2"),
    Shape::Path("M20 15h2"),
    Shape::Path("M20 9h2"),
    Shape::Path("M9 2v2"),
    Shape::Path("M9 20v2"),
];

/// Lucide's `disc-3`.
pub const DISC_3: &[Shape] = &[
    Shape::Circle(12.0, 12.0, 10.0),
    Shape::Path("M6 12c0-1.7.7-3.2 1.8-4.2"),
    Shape::Circle(12.0, 12.0, 2.0),
    Shape::Path("M18 12c0 1.7-.7 3.2-1.8 4.2"),
];

/// Lucide's `folder-open`.
pub const FOLDER_OPEN: &[Shape] = &[
    Shape::Path("m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"),
];

/// Lucide's `folder`.
pub const FOLDER: &[Shape] = &[
    Shape::Path("M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"),
];

/// Lucide's `globe`.
pub const GLOBE: &[Shape] = &[
    Shape::Circle(12.0, 12.0, 10.0),
    Shape::Path("M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"),
    Shape::Path("M2 12h20"),
];

/// Lucide's `hard-drive`.
pub const HARD_DRIVE: &[Shape] = &[
    Shape::Path("M10 16h.01"),
    Shape::Path("M2.212 11.577a2 2 0 0 0-.212.896V18a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5.527a2 2 0 0 0-.212-.896L18.55 5.11A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"),
    Shape::Path("M21.946 12.013H2.054"),
    Shape::Path("M6 16h.01"),
];

/// Lucide's `heart-pulse`.
pub const HEART_PULSE: &[Shape] = &[
    Shape::Path("M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"),
    Shape::Path("M3.22 12H9.5l.5-1 2 4.5 2-7 1.5 3.5h5.27"),
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

/// Lucide's `list-ordered`.
pub const LIST_ORDERED: &[Shape] = &[
    Shape::Path("M10 12h11"),
    Shape::Path("M10 18h11"),
    Shape::Path("M10 6h11"),
    Shape::Path("M4 10h2"),
    Shape::Path("M4 6h1v4"),
    Shape::Path("M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"),
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

/// Lucide's `map-pin`.
pub const MAP_PIN: &[Shape] = &[
    Shape::Path("M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0"),
    Shape::Circle(12.0, 10.0, 3.0),
];

/// Lucide's `mic`.
pub const MIC: &[Shape] = &[
    Shape::Path("M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"),
    Shape::Path("M19 10v2a7 7 0 0 1-14 0v-2"),
    Shape::Line(0.0, 0.0, 0.0, 0.0),
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

/// Lucide's `server`.
pub const SERVER: &[Shape] = &[
    Shape::Rect(2.0, 2.0, 20.0, 8.0, 2.0),
    Shape::Rect(2.0, 14.0, 20.0, 8.0, 2.0),
    Shape::Line(0.0, 0.0, 0.0, 0.0),
    Shape::Line(0.0, 0.0, 0.0, 0.0),
];

/// Lucide's `star`.
pub const STAR: &[Shape] = &[
    Shape::Path("M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"),
];

/// Lucide's `tag`.
pub const TAG: &[Shape] = &[
    Shape::Path("M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z"),
    Shape::Circle(7.5, 7.5, 0.5),
];

/// Lucide's `thumbs-up`.
pub const THUMBS_UP: &[Shape] = &[
    Shape::Path("M7 10v12"),
    Shape::Path("M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2a3.13 3.13 0 0 1 3 3.88Z"),
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

/// Lucide's `user-round`.
pub const USER_ROUND: &[Shape] = &[
    Shape::Circle(12.0, 8.0, 5.0),
    Shape::Path("M20 21a8 8 0 0 0-16 0"),
];

/// Lucide's `x`.
pub const X: &[Shape] = &[Shape::Path("M18 6 6 18"), Shape::Path("m6 6 12 12")];

/// Every icon here by its Lucide name.
pub const ALL: &[(&str, &[Shape])] = &[
    ("cast", CAST),
    ("chevron-down", CHEVRON_DOWN),
    ("chevron-left", CHEVRON_LEFT),
    ("chevron-right", CHEVRON_RIGHT),
    ("cloud", CLOUD),
    ("cpu", CPU),
    ("disc-3", DISC_3),
    ("folder-open", FOLDER_OPEN),
    ("folder", FOLDER),
    ("globe", GLOBE),
    ("hard-drive", HARD_DRIVE),
    ("heart-pulse", HEART_PULSE),
    ("heart", HEART),
    ("library-big", LIBRARY_BIG),
    ("list-end", LIST_END),
    ("list-music", LIST_MUSIC),
    ("list-ordered", LIST_ORDERED),
    ("list-plus", LIST_PLUS),
    ("list", LIST),
    ("map-pin", MAP_PIN),
    ("mic", MIC),
    ("music", MUSIC),
    ("play", PLAY),
    ("plus", PLUS),
    ("radio", RADIO),
    ("search", SEARCH),
    ("server", SERVER),
    ("star", STAR),
    ("tag", TAG),
    ("thumbs-up", THUMBS_UP),
    ("usb", USB),
    ("user-round", USER_ROUND),
    ("x", X),
];
