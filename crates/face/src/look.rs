//! The look a track gives the face: the theme's colours, with those it
//! leaves to the artwork read from the cover. Read once per track, from
//! the cover at thirty-two by thirty-two.

use crate::theme::{Paint, Theme};
use glass::face::{fit_art, Frame};

/// The colours in use for a track.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Look {
    /// The colour of glass.
    pub tint: [u8; 3],
    /// What is lit.
    pub accent: [u8; 3],
}

/// With no cover and nothing fixed: a dark neutral glass, a near white accent.
pub const NEUTRAL: Look = Look {
    tint: [14, 14, 16],
    accent: [235, 235, 240],
};

/// How light a colour reads, 0 to 1.
pub fn luma(c: [u8; 3]) -> f32 {
    (0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32) / 255.0
}

/// A colour brought to a lightness, its hue kept: scaled, and where a
/// channel would clip, moved toward white for the rest.
pub fn at_luma(c: [u8; 3], want: f32) -> [u8; 3] {
    let k = want / luma(c).max(0.01);
    let mut out = [0u8; 3];
    for i in 0..3 {
        out[i] = (c[i] as f32 * k).min(255.0) as u8;
    }
    let have = luma(out);
    if have < want - 0.04 {
        let t = ((want - have) / (1.0 - have).max(0.01)).clamp(0.0, 1.0);
        for v in out.iter_mut() {
            *v = (*v as f32 + (255.0 - *v as f32) * t) as u8;
        }
    }
    out
}

/// What a cover gives: its mean colour taken dark for the glass, and its
/// most vivid colour lifted to read on that glass; a cover with no colour
/// to speak of gives the neutral accent.
pub fn of_cover(cover: &Frame) -> Look {
    let small = fit_art(cover, 32, 32);
    let (mut sum, mut best, mut best_score) = ([0f32; 3], NEUTRAL.accent, -1.0f32);
    let mut count = 0f32;
    for px in small.rgba.as_chunks::<4>().0 {
        let c = [px[0], px[1], px[2]];
        for i in 0..3 {
            sum[i] += c[i] as f32;
        }
        count += 1.0;
        let max = c.iter().copied().max().unwrap_or(0) as f32;
        let min = c.iter().copied().min().unwrap_or(0) as f32;
        let saturation = if max > 0.0 { (max - min) / max } else { 0.0 };
        // Vivid, and neither black nor white.
        let score = saturation * (1.0 - (luma(c) - 0.5).abs());
        if score > best_score {
            best_score = score;
            best = c;
        }
    }
    if count == 0.0 {
        return NEUTRAL;
    }
    let mean = [
        (sum[0] / count) as u8,
        (sum[1] / count) as u8,
        (sum[2] / count) as u8,
    ];
    Look {
        tint: at_luma(mean, 0.07),
        accent: if best_score < 0.3 {
            NEUTRAL.accent
        } else {
            at_luma(best, 0.62)
        },
    }
}

/// The look for a theme and a cover: what the theme fixes stands, what it
/// leaves to the artwork comes from the cover, or is neutral without one.
pub fn resolve(theme: &Theme, cover: Option<&Frame>) -> Look {
    let from_cover = match (theme.tint, theme.accent) {
        (Paint::Fixed(_), Paint::Fixed(_)) => NEUTRAL,
        _ => cover.map(of_cover).unwrap_or(NEUTRAL),
    };
    Look {
        tint: match theme.tint {
            Paint::Fixed(c) => c,
            Paint::Artwork => from_cover.tint,
        },
        accent: match theme.accent {
            Paint::Fixed(c) => c,
            Paint::Artwork => from_cover.accent,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cover(pixels: &[([u8; 3], usize)]) -> Frame {
        let mut rgba = Vec::new();
        for (c, n) in pixels {
            for _ in 0..*n {
                rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
        let side = ((rgba.len() / 4) as f32).sqrt() as u32;
        Frame {
            blend: Default::default(),
            width: side,
            height: side,
            rgba,
        }
    }

    #[test]
    fn a_cover_gives_a_dark_glass_of_its_colour_and_an_accent_that_reads() {
        // A deep blue cover with a band of orange.
        let look = of_cover(&cover(&[([10, 40, 160], 768), ([240, 130, 20], 256)]));
        assert!(luma(look.tint) < 0.12, "the glass is dark: {:?}", look.tint);
        assert!(
            look.tint[2] > look.tint[0],
            "and keeps the cover's blue: {:?}",
            look.tint
        );
        assert!(
            luma(look.accent) > 0.5,
            "the accent reads on it: {:?}",
            look.accent
        );
        assert!(
            look.accent[0] > look.accent[2],
            "and is the cover's most vivid colour, the orange: {:?}",
            look.accent
        );
    }

    #[test]
    fn a_cover_with_no_colour_gives_the_neutral_accent() {
        let look = of_cover(&cover(&[([200, 200, 200], 512), ([30, 30, 30], 512)]));
        assert_eq!(look.accent, NEUTRAL.accent);
        assert!(luma(look.tint) < 0.12);
        // A faint cast is still no colour to speak of.
        assert_eq!(
            of_cover(&cover(&[([150, 155, 165], 1024)])).accent,
            NEUTRAL.accent
        );
    }

    #[test]
    fn what_the_theme_fixes_stands_and_the_rest_comes_from_the_cover() {
        let blue = cover(&[([10, 40, 160], 1024)]);
        let mut theme = Theme::default();
        assert_eq!(resolve(&theme, None), NEUTRAL, "no cover, nothing fixed");
        assert_eq!(resolve(&theme, Some(&blue)), of_cover(&blue));
        theme.accent = Paint::Fixed([0, 255, 0]);
        let look = resolve(&theme, Some(&blue));
        assert_eq!(look.accent, [0, 255, 0]);
        assert_eq!(look.tint, of_cover(&blue).tint);
        theme.tint = Paint::Fixed([20, 0, 0]);
        assert_eq!(
            resolve(&theme, Some(&blue)),
            Look {
                tint: [20, 0, 0],
                accent: [0, 255, 0]
            }
        );
    }

    #[test]
    fn a_colour_is_brought_to_a_lightness_with_its_hue() {
        let dark = at_luma([200, 100, 50], 0.07);
        assert!((luma(dark) - 0.07).abs() < 0.02);
        assert!(dark[0] > dark[1] && dark[1] > dark[2]);
        let lifted = at_luma([0, 0, 120], 0.62);
        assert!(
            (luma(lifted) - 0.62).abs() < 0.06,
            "a dark blue lifted toward white: {lifted:?}"
        );
        assert!(lifted[2] >= lifted[0]);
    }
}
