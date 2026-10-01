//! The face's icons. Each is a shape given by its distance from a point,
//! on a grid of twenty-four with a margin of two, the way icon sets are
//! drawn; a set rasters them once for a size, the edge taken from the
//! distance so it is smooth at any size, and from then on an icon is a
//! picture to blit. No fonts and no files: any theme's fonts do, and a
//! face size is one more raster.

use glass::face::Frame;

/// What the face can show on a button or a tile.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Previous,
    Play,
    Pause,
    Next,
    Minus,
    Plus,
    More,
    Repeat,
    RepeatOne,
    Shuffle,
    Speaker,
    Muted,
}

const ICONS: [Icon; 12] = [
    Icon::Previous,
    Icon::Play,
    Icon::Pause,
    Icon::Next,
    Icon::Minus,
    Icon::Plus,
    Icon::More,
    Icon::Repeat,
    Icon::RepeatOne,
    Icon::Shuffle,
    Icon::Speaker,
    Icon::Muted,
];

type Point = (f32, f32);

/// The grid's span across an icon's box: 2 to 22 of the twenty-four.
const SPAN: f32 = 20.0;
/// A stroke's half width on the grid.
const STROKE: f32 = 1.0;

/// The distance to a stroke between two points, round at its ends.
fn segment(p: Point, a: Point, b: Point, half: f32) -> f32 {
    let (pa, ba) = ((p.0 - a.0, p.1 - a.1), (b.0 - a.0, b.1 - a.1));
    let along = ((pa.0 * ba.0 + pa.1 * ba.1) / (ba.0 * ba.0 + ba.1 * ba.1)).clamp(0.0, 1.0);
    (pa.0 - ba.0 * along).hypot(pa.1 - ba.1 * along) - half
}

/// The distance to a filled polygon, negative inside.
fn polygon(p: Point, v: &[Point]) -> f32 {
    let mut nearest = f32::MAX;
    let mut inside = false;
    let mut j = v.len() - 1;
    for i in 0..v.len() {
        let e = (v[j].0 - v[i].0, v[j].1 - v[i].1);
        let w = (p.0 - v[i].0, p.1 - v[i].1);
        let along = ((w.0 * e.0 + w.1 * e.1) / (e.0 * e.0 + e.1 * e.1)).clamp(0.0, 1.0);
        let b = (w.0 - e.0 * along, w.1 - e.1 * along);
        nearest = nearest.min(b.0 * b.0 + b.1 * b.1);
        // An edge crossed by the ray to the right of the point turns the side.
        if (v[i].1 > p.1) != (v[j].1 > p.1) && e.0 * w.1 / e.1 > w.0 {
            inside = !inside;
        }
        j = i;
    }
    if inside {
        -nearest.sqrt()
    } else {
        nearest.sqrt()
    }
}

/// The distance to a disc.
fn disc(p: Point, c: Point, radius: f32) -> f32 {
    (p.0 - c.0).hypot(p.1 - c.1) - radius
}

/// The distance to a stroke along a circle's arc that opens to the right,
/// `spread` radians either side of the horizontal, round at its ends.
fn arc(p: Point, c: Point, radius: f32, spread: f32, half: f32) -> f32 {
    let (dx, dy) = (p.0 - c.0, (p.1 - c.1).abs());
    if dy.atan2(dx) <= spread {
        (dx.hypot(dy) - radius).abs() - half
    } else {
        (dx - radius * spread.cos()).hypot(dy - radius * spread.sin()) - half
    }
}

/// The speaker's body: the box and the cone, facing right.
fn speaker(p: Point) -> f32 {
    polygon(
        p,
        &[
            (3.0, 9.0),
            (7.0, 9.0),
            (12.0, 4.0),
            (12.0, 20.0),
            (7.0, 15.0),
            (3.0, 15.0),
        ],
    )
}

/// The two arrows of repeat: the top one running right from a stem at the
/// left, the bottom one running left from a stem at the right.
fn repeat(p: Point) -> f32 {
    segment(p, (6.0, 6.5), (17.0, 6.5), STROKE)
        .min(segment(p, (6.0, 6.5), (6.0, 11.0), STROKE))
        .min(polygon(p, &[(17.0, 2.5), (21.5, 6.5), (17.0, 10.5)]))
        .min(segment(p, (7.0, 17.5), (18.0, 17.5), STROKE))
        .min(segment(p, (18.0, 17.5), (18.0, 13.0), STROKE))
        .min(polygon(p, &[(7.0, 13.5), (2.5, 17.5), (7.0, 21.5)]))
}

/// An icon's distance from a point on the grid, negative inside.
fn distance(icon: Icon, p: Point) -> f32 {
    // The mirror of a point about the grid's middle, for the icons that
    // are another's reflection.
    let mirrored = (24.0 - p.0, p.1);
    match icon {
        Icon::Previous => distance(Icon::Next, mirrored),
        Icon::Next => polygon(p, &[(5.5, 5.0), (15.5, 12.0), (5.5, 19.0)]).min(polygon(
            p,
            &[(16.5, 5.0), (19.0, 5.0), (19.0, 19.0), (16.5, 19.0)],
        )),
        Icon::Play => polygon(p, &[(7.0, 4.0), (20.0, 12.0), (7.0, 20.0)]),
        Icon::Pause => polygon(p, &[(6.0, 4.5), (10.0, 4.5), (10.0, 19.5), (6.0, 19.5)]).min(
            polygon(p, &[(14.0, 4.5), (18.0, 4.5), (18.0, 19.5), (14.0, 19.5)]),
        ),
        Icon::Minus => segment(p, (5.0, 12.0), (19.0, 12.0), STROKE * 1.2),
        Icon::Plus => segment(p, (5.0, 12.0), (19.0, 12.0), STROKE * 1.2).min(segment(
            p,
            (12.0, 5.0),
            (12.0, 19.0),
            STROKE * 1.2,
        )),
        Icon::More => disc(p, (5.0, 12.0), 2.0)
            .min(disc(p, (12.0, 12.0), 2.0))
            .min(disc(p, (19.0, 12.0), 2.0)),
        Icon::Repeat => repeat(p),
        Icon::RepeatOne => repeat(p)
            .min(segment(p, (12.3, 9.6), (12.3, 14.4), STROKE * 0.75))
            .min(segment(p, (10.7, 10.7), (12.3, 9.6), STROKE * 0.75)),
        Icon::Shuffle => segment(p, (4.7, 19.3), (17.0, 7.0), STROKE)
            .min(polygon(p, &[(14.0, 3.5), (20.5, 3.5), (20.5, 10.0)]))
            .min(segment(p, (4.7, 4.7), (9.4, 9.4), STROKE))
            .min(segment(p, (14.6, 14.6), (17.0, 17.0), STROKE))
            .min(polygon(p, &[(14.0, 20.5), (20.5, 20.5), (20.5, 14.0)])),
        Icon::Speaker => speaker(p)
            .min(arc(p, (12.0, 12.0), 3.6, 0.9, STROKE * 0.9))
            .min(arc(p, (12.0, 12.0), 7.6, 0.9, STROKE * 0.9)),
        Icon::Muted => speaker(p)
            .min(segment(p, (15.5, 9.3), (20.9, 14.7), STROKE))
            .min(segment(p, (15.5, 14.7), (20.9, 9.3), STROKE)),
    }
}

/// One icon rastered at a size in an ink: the ink everywhere, the shape in
/// the alpha.
fn raster(icon: Icon, size: u32, ink: [u8; 3]) -> Frame {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let step = SPAN / size as f32;
    for y in 0..size {
        for x in 0..size {
            let p = (2.0 + (x as f32 + 0.5) * step, 2.0 + (y as f32 + 0.5) * step);
            // Half a pixel either side of the edge is the edge.
            let cover = (0.5 - distance(icon, p) / step).clamp(0.0, 1.0);
            rgba.extend_from_slice(&[ink[0], ink[1], ink[2], (cover * 255.0).round() as u8]);
        }
    }
    Frame {
        blend: Default::default(),
        width: size,
        height: size,
        rgba,
    }
}

/// The icons rastered at one size.
pub struct Set {
    size: u32,
    ink: [u8; 3],
    frames: Vec<Frame>,
}

impl Set {
    /// Every icon at `size` pixels square, in `ink`.
    pub fn new(size: u32, ink: [u8; 3]) -> Self {
        let size = size.max(1);
        Self {
            size,
            ink,
            frames: ICONS.iter().map(|icon| raster(*icon, size, ink)).collect(),
        }
    }

    /// Whether the set is the one for a size and an ink.
    pub fn is(&self, size: u32, ink: [u8; 3]) -> bool {
        self.size == size.max(1) && self.ink == ink
    }

    /// The size the set was rastered at.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// An icon's picture.
    pub fn get(&self, icon: Icon) -> &Frame {
        &self.frames[icon as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(frame: &Frame, x: u32, y: u32) -> u8 {
        frame.rgba[((y * frame.width + x) * 4 + 3) as usize]
    }

    #[test]
    fn the_set_holds_every_icon_in_the_order_of_its_names() {
        for (i, icon) in ICONS.iter().enumerate() {
            assert_eq!(*icon as usize, i, "{icon:?} is where the set looks for it");
        }
        let set = Set::new(36, [235, 235, 240]);
        assert_eq!(set.size(), 36);
        for icon in ICONS {
            let frame = set.get(icon);
            assert_eq!((frame.width, frame.height), (36, 36));
            assert_eq!(frame.rgba.len(), 36 * 36 * 4);
            let inked = (0..36 * 36)
                .filter(|i| frame.rgba[i * 4 + 3] == 255)
                .count();
            assert!(inked > 40, "{icon:?} has a body: {inked} full pixels");
            assert!(inked < 36 * 36 / 2, "{icon:?} is a shape, not a block");
            for (x, y) in [(0, 0), (35, 0), (0, 35), (35, 35)] {
                assert_eq!(alpha(frame, x, y), 0, "{icon:?} leaves its corners clear");
            }
            assert_eq!(&frame.rgba[..3], &[235, 235, 240], "the ink is the set's");
        }
    }

    #[test]
    fn previous_is_next_in_a_mirror_and_the_edge_is_smooth() {
        let set = Set::new(48, [255; 3]);
        let (previous, next) = (set.get(Icon::Previous), set.get(Icon::Next));
        let mut partial = 0;
        for y in 0..48 {
            for x in 0..48 {
                let (a, b) = (alpha(previous, x, y), alpha(next, 47 - x, y));
                assert!(a.abs_diff(b) <= 1, "at {x},{y}: {a} against {b}");
                if a > 0 && a < 255 {
                    partial += 1;
                }
            }
        }
        assert!(
            partial > 20,
            "the slopes carry part cover: {partial} pixels"
        );
    }

    #[test]
    fn the_shapes_are_where_the_grid_puts_them() {
        // Play's middle is inside, its left margin outside.
        assert!(distance(Icon::Play, (12.0, 12.0)) < 0.0);
        assert!(distance(Icon::Play, (4.0, 12.0)) > 0.0);
        // The speaker's cone is inside both speakers; the waves only in one,
        // the cross only in the other.
        assert!(distance(Icon::Speaker, (9.0, 12.0)) < 0.0);
        assert!(distance(Icon::Muted, (9.0, 12.0)) < 0.0);
        assert!(distance(Icon::Speaker, (19.6, 12.0)) < 0.0);
        assert!(distance(Icon::Muted, (18.2, 12.0)) < 0.0);
        assert!(distance(Icon::Muted, (18.2, 6.0)) > 0.0);
        // Repeat one is repeat with a figure in the middle.
        assert!(distance(Icon::Repeat, (12.3, 12.0)) > 0.0);
        assert!(distance(Icon::RepeatOne, (12.3, 12.0)) < 0.0);
        // More is three dots, clear between them.
        assert!(distance(Icon::More, (12.0, 12.0)) < 0.0);
        assert!(distance(Icon::More, (8.5, 12.0)) > 0.0);
    }
}
