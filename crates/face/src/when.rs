//! A time of day set in a pattern, as `strftime` reads one in the C
//! locale: `%H:%M`, `%-I:%M %p`, `%A %-d %B`. Written out here, and not
//! asked of the system, so a pattern gives the same words on the player's
//! screen and in a browser, where there is no `strftime` to ask.
//!
//! Every conversion of the C library's is read as the library reads it,
//! with the flags `-` (no padding), `_` (spaces), `0` (zeros), `^`
//! (capitals) and `#` (the other case), a width, and the `E` and `O`
//! modifiers where the library takes them, which change nothing in this
//! locale. A conversion that is none stands as written. A test holds every
//! conversion in every form against the library itself; the one thing not
//! followed is its reading of a width on `%z`.

use overlay::Wall;

const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
/// A weekday's name, 0 Sunday to 6 Saturday, whole or its first three letters.
pub fn day_name(weekday: u8, short: bool) -> &'static str {
    let name = DAYS[(weekday as usize) % 7];
    if short {
        &name[..3]
    } else {
        name
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The flags and the width between a `%` and its conversion.
#[derive(Clone, Copy, Default)]
struct Spec {
    /// `-`, `_` or `0`, the last one given; none for the conversion's own.
    pad: Option<char>,
    upper: bool,
    swap: bool,
    width: Option<usize>,
}

fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// The ISO 8601 week a day lies in: the week's year and its number.
fn iso_week(wall: &Wall) -> (i32, u32) {
    // Monday is the first day of an ISO week, and week 1 holds the year's
    // first Thursday.
    let weekday = (wall.weekday + 6) % 7;
    let week = (wall.yearday as i32 - weekday as i32 + 10) / 7;
    let weeks_in = |year: i32| {
        // The weekday of the first of January, Monday 0.
        let first = {
            let y = i64::from(year) - 1;
            ((1 + 5 * y.rem_euclid(4) + 4 * y.rem_euclid(100) + 6 * y.rem_euclid(400)) % 7 + 6) % 7
        };
        if first == 3 || (first == 2 && leap(year)) {
            53i32
        } else {
            52
        }
    };
    if week < 1 {
        (wall.year - 1, weeks_in(wall.year - 1) as u32)
    } else if week > weeks_in(wall.year) {
        (wall.year + 1, 1)
    } else {
        (wall.year, week as u32)
    }
}

/// A number as a conversion writes it: at least `width` long, filled with
/// the conversion's own character unless a flag says another. A width
/// given in the pattern is always filled, with spaces where the flag took
/// the conversion's own filling away.
fn number(out: &mut String, value: i64, width: usize, fill: char, spec: Spec) {
    let digits = value.unsigned_abs().to_string();
    let sign = if value < 0 { "-" } else { "" };
    let fill = match (spec.pad, spec.width) {
        (Some('-'), None) => None,
        (Some('-') | Some('_'), _) => Some(' '),
        (Some('0'), _) => Some('0'),
        _ => Some(fill),
    };
    let width = spec.width.unwrap_or(width);
    let short = width.saturating_sub(digits.len() + sign.len());
    match fill {
        // Zeros go after the sign, spaces before it.
        Some('0') => {
            out.push_str(sign);
            out.extend(std::iter::repeat_n('0', short));
        }
        Some(fill) => {
            out.extend(std::iter::repeat_n(fill, short));
            out.push_str(sign);
        }
        None => out.push_str(sign),
    }
    out.push_str(&digits);
}

/// Words as a conversion writes them: in capitals where asked, and at
/// least `width` long where one is given.
fn words(out: &mut String, text: &str, upper: bool, spec: Spec) {
    let text = if upper || spec.upper {
        text.to_uppercase()
    } else {
        text.to_string()
    };
    let short = spec.width.unwrap_or(0).saturating_sub(text.chars().count());
    let fill = if spec.pad == Some('0') { '0' } else { ' ' };
    out.extend(std::iter::repeat_n(fill, short));
    out.push_str(&text);
}

/// One conversion into `out`; `false` when the letter is no conversion.
fn convert(out: &mut String, letter: char, spec: Spec, wall: &Wall) -> bool {
    let hour12 = match wall.hour % 12 {
        0 => 12,
        h => h,
    };
    let day = DAYS[(wall.weekday % 7) as usize];
    let month = MONTHS[((wall.month.clamp(1, 12)) - 1) as usize];
    // A conversion made of others: set by its own pattern, then as words.
    let made = |out: &mut String, pattern: &str| {
        let mut inner = String::new();
        set(&mut inner, pattern, wall);
        words(out, &inner, false, spec);
    };
    match letter {
        'a' => words(out, &day[..3], spec.swap, spec),
        'A' => words(out, day, spec.swap, spec),
        'b' | 'h' => words(out, &month[..3], spec.swap, spec),
        'B' => words(out, month, spec.swap, spec),
        'c' => made(out, "%a %b %e %H:%M:%S %Y"),
        'C' => number(out, i64::from(wall.year.div_euclid(100)), 2, '0', spec),
        'd' => number(out, i64::from(wall.day), 2, '0', spec),
        'D' | 'x' => made(out, "%m/%d/%y"),
        'e' => number(out, i64::from(wall.day), 2, ' ', spec),
        'F' => made(out, "%Y-%m-%d"),
        'g' => number(
            out,
            i64::from(iso_week(wall).0.rem_euclid(100)),
            2,
            '0',
            spec,
        ),
        'G' => number(out, i64::from(iso_week(wall).0), 1, '0', spec),
        'H' => number(out, i64::from(wall.hour), 2, '0', spec),
        'I' => number(out, i64::from(hour12), 2, '0', spec),
        'j' => number(out, i64::from(wall.yearday) + 1, 3, '0', spec),
        'k' => number(out, i64::from(wall.hour), 2, ' ', spec),
        'l' => number(out, i64::from(hour12), 2, ' ', spec),
        'm' => number(out, i64::from(wall.month), 2, '0', spec),
        'M' => number(out, i64::from(wall.minute), 2, '0', spec),
        'n' => words(out, "\n", false, spec),
        'p' | 'P' => {
            let half = if wall.hour < 12 { "AM" } else { "PM" };
            // `%P` is the small one whatever the flags, and `#` gives `%p`
            // the other case.
            if letter == 'P' {
                out.push_str(&{
                    let mut small = String::new();
                    words(
                        &mut small,
                        half,
                        false,
                        Spec {
                            upper: false,
                            ..spec
                        },
                    );
                    small.to_lowercase()
                });
            } else if spec.swap && !spec.upper {
                words(out, &half.to_lowercase(), false, spec);
            } else {
                words(out, half, false, spec);
            }
        }
        'r' => made(out, "%I:%M:%S %p"),
        'R' => made(out, "%H:%M"),
        's' => number(out, wall.epoch_s, 1, '0', spec),
        'S' => number(out, i64::from(wall.second), 2, '0', spec),
        't' => words(out, "\t", false, spec),
        'T' | 'X' => made(out, "%H:%M:%S"),
        'u' => number(out, i64::from((wall.weekday + 6) % 7 + 1), 1, '0', spec),
        'U' => number(
            out,
            i64::from((wall.yearday + 7 - wall.weekday) / 7),
            2,
            '0',
            spec,
        ),
        'V' => number(out, i64::from(iso_week(wall).1), 2, '0', spec),
        'w' => number(out, i64::from(wall.weekday), 1, '0', spec),
        'W' => number(
            out,
            i64::from((wall.yearday + 7 - (wall.weekday + 6) % 7) / 7),
            2,
            '0',
            spec,
        ),
        'y' => number(out, i64::from(wall.year.rem_euclid(100)), 2, '0', spec),
        'Y' => number(out, i64::from(wall.year), 1, '0', spec),
        'z' => {
            let minutes = wall.offset_minutes;
            let value = i64::from(minutes.abs() / 60 * 100 + minutes.abs() % 60);
            let mut digits = String::new();
            number(
                &mut digits,
                value,
                4,
                '0',
                Spec {
                    width: None,
                    ..spec
                },
            );
            let signed = format!("{}{digits}", if minutes < 0 { '-' } else { '+' });
            words(out, &signed, false, Spec { pad: None, ..spec });
        }
        'Z' => {
            if spec.swap && !spec.upper {
                words(out, &wall.zone.to_lowercase(), false, spec);
            } else {
                words(out, &wall.zone, false, spec);
            }
        }
        '%' => words(out, "%", false, spec),
        _ => return false,
    }
    true
}

/// The pattern into `out`, conversion by conversion.
fn set(out: &mut String, pattern: &str, wall: &Wall) {
    let mut rest = pattern.chars().peekable();
    while let Some(c) = rest.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        // What follows the percent sign, kept as written for a conversion
        // that turns out to be none.
        let mut written = String::from('%');
        let mut spec = Spec::default();
        while let Some(&flag) = rest.peek() {
            match flag {
                '-' | '_' | '0' => spec.pad = Some(flag),
                '^' => spec.upper = true,
                '#' => spec.swap = true,
                _ => break,
            }
            written.push(flag);
            rest.next();
        }
        while let Some(&digit) = rest.peek() {
            let Some(value) = digit.to_digit(10) else {
                break;
            };
            spec.width = Some(
                spec.width
                    .unwrap_or(0)
                    .saturating_mul(10)
                    .saturating_add(value as usize)
                    .min(64),
            );
            written.push(digit);
            rest.next();
        }
        let mut modifier = None;
        if let Some(&m) = rest.peek() {
            if m == 'E' || m == 'O' {
                modifier = Some(m);
                written.push(m);
                rest.next();
            }
        }
        match rest.next() {
            Some(letter) => {
                // The library takes each modifier with some conversions
                // only; with the others the whole stands as written.
                let taken = match modifier {
                    Some('E') => !"aAbBdDeFgGhHIjklmMSUVwW".contains(letter),
                    Some('O') => !"aAcDFxXY".contains(letter),
                    _ => true,
                };
                if !taken || !convert(out, letter, spec, wall) {
                    out.push_str(&written);
                    out.push(letter);
                }
            }
            None => out.push_str(&written),
        }
    }
}

/// Whether a pattern shows the seconds: whether a second later it reads
/// otherwise.
pub fn shows_seconds(pattern: &str) -> bool {
    format(pattern, &Wall::at(0, 0, "")) != format(pattern, &Wall::at(1000, 0, ""))
}

/// A time set in a pattern. Nothing for a pattern that holds a nul or
/// gives more than a line can hold, as the C library's gave nothing.
pub fn format(pattern: &str, wall: &Wall) -> String {
    if pattern.contains('\0') {
        return String::new();
    }
    let mut out = String::new();
    set(&mut out, pattern, wall);
    if out.len() >= 128 {
        return String::new();
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Thursday 1 October 2026, 13:05:09, an hour east of universal time.
    fn a_thursday() -> Wall {
        Wall::at(1_790_856_309_000, 60, "BST")
    }

    #[test]
    fn the_patterns_the_manager_offers() {
        let wall = a_thursday();
        assert_eq!(
            (wall.year, wall.month, wall.day, wall.weekday),
            (2026, 10, 1, 4)
        );
        assert_eq!(format("%H:%M", &wall), "13:05");
        assert_eq!(format("%H:%M:%S", &wall), "13:05:09");
        assert_eq!(format("%-I:%M %p", &wall), "1:05 PM");
        assert_eq!(format("%-I:%M:%S %p", &wall), "1:05:09 PM");
        assert_eq!(format("%A %-d %B", &wall), "Thursday 1 October");
        assert_eq!(format("%A, %B %-d", &wall), "Thursday, October 1");
        assert_eq!(format("%a %-d %b %Y", &wall), "Thu 1 Oct 2026");
        assert_eq!(format("%d/%m/%Y", &wall), "01/10/2026");
        assert_eq!(format("%m/%d/%Y", &wall), "10/01/2026");
        assert_eq!(format("%d.%m.%Y", &wall), "01.10.2026");
        assert_eq!(format("%Y-%m-%d", &wall), "2026-10-01");
    }

    #[test]
    fn flags_widths_and_what_is_no_conversion() {
        let wall = a_thursday();
        assert_eq!(
            format("%e|%_d|%-d|%0e", &wall),
            "1| 1|1|01",
            "the line's own ends are trimmed"
        );
        assert_eq!(format("x%e", &wall), "x 1");
        assert_eq!(
            format("%^A %^b %#p %P %#Z", &wall),
            "THURSDAY OCT pm pm bst"
        );
        assert_eq!(
            format("%5d|%-5d|%_5d|%10A|%-10A|", &wall),
            "00001|    1|    1|  Thursday|  Thursday|"
        );
        assert_eq!(
            format("%j %U %W %V %G %g %u %w %C %y", &wall),
            "274 39 39 40 2026 26 4 4 20 26"
        );
        assert_eq!(format("%z %Z %s", &wall), "+0100 BST 1790856309");
        assert_eq!(
            format("%D %F %T %R %r", &wall),
            "10/01/26 2026-10-01 13:05:09 13:05 01:05:09 PM"
        );
        assert_eq!(format("%c", &wall), "Thu Oct  1 13:05:09 2026");
        assert_eq!(
            format("%Ey %Od %EY %Ed %OY", &wall),
            "26 01 2026 %Ed %OY",
            "each modifier where the library takes it"
        );
        assert_eq!(format("100%% %Q %", &wall), "100% %Q %");
        assert_eq!(format("%-", &wall), "%-");
        assert_eq!(format("", &wall), "");
        assert_eq!(
            format("a\0b", &wall),
            "",
            "a pattern that cannot be read gives nothing"
        );
        assert_eq!(
            format(&"%A ".repeat(40), &wall),
            "",
            "more than a line holds gives nothing"
        );
        // West of universal time, and midnight and noon on a twelve hour clock.
        let west = Wall::at(1_790_856_309_000, -(9 * 60 + 30), "MART");
        assert_eq!(format("%z %H:%M", &west), "-0930 02:35");
        let midnight = Wall::at(1_790_812_800_000, 0, "UTC");
        assert_eq!(format("%I %l %p %k", &midnight), "12 12 AM  0");
        assert_eq!(
            format("%I %p", &Wall::at(1_790_856_000_000, 0, "UTC")),
            "12 PM"
        );
    }

    #[test]
    fn a_pattern_says_whether_it_shows_seconds() {
        for with in ["%H:%M:%S", "%T", "%-I:%M:%S %p", "%r", "%c", "%s"] {
            assert!(shows_seconds(with), "{with}");
        }
        for without in ["%H:%M", "%-I:%M %p", "%R", "%A %-d %B", ""] {
            assert!(!shows_seconds(without), "{without}");
        }
    }

    #[test]
    fn the_weeks_at_the_turn_of_a_year() {
        // Friday 1 January 2027 is in the last ISO week of 2026.
        let new_year = Wall::at(1_798_761_600_000, 0, "UTC");
        assert_eq!(
            format("%F %a %V %G %g %U %W", &new_year),
            "2027-01-01 Fri 53 2026 26 00 00"
        );
        // Monday 31 December 2029 is in the first ISO week of 2030.
        let eve = Wall::at(1_893_369_600_000, 0, "UTC");
        assert_eq!(
            format("%F %a %V %G %U %W %j", &eve),
            "2029-12-31 Mon 01 2030 52 53 365"
        );
    }

    /// The C library's own answer for the same time, where there is one.
    #[cfg(unix)]
    fn by_the_library(pattern: &str, wall: &Wall) -> String {
        let zone = std::ffi::CString::new(wall.zone.as_str()).unwrap();
        // SAFETY: a tm is plain numbers and a pointer to a name that
        // outlives the call; strftime writes at most the buffer's length.
        unsafe {
            let mut tm: libc::tm = std::mem::zeroed();
            tm.tm_year = wall.year - 1900;
            tm.tm_mon = wall.month as i32 - 1;
            tm.tm_mday = wall.day as i32;
            tm.tm_hour = wall.hour as i32;
            tm.tm_min = wall.minute as i32;
            tm.tm_sec = wall.second as i32;
            tm.tm_wday = wall.weekday as i32;
            tm.tm_yday = wall.yearday as i32;
            tm.tm_gmtoff = i64::from(wall.offset_minutes) * 60;
            tm.tm_zone = zone.as_ptr();
            let pattern = std::ffi::CString::new(pattern).unwrap();
            let mut text = [0u8; 128];
            let written =
                libc::strftime(text.as_mut_ptr().cast(), text.len(), pattern.as_ptr(), &tm);
            String::from_utf8_lossy(&text[..written]).trim().to_string()
        }
    }

    /// Every conversion, bare and with every flag and a width, over days
    /// spread across twelve years and four zones, reads as the C library
    /// reads it: what the face showed before it set its own time.
    #[cfg(unix)]
    #[test]
    fn as_the_c_library_reads_them() {
        let letters = "aAbBcCdDeFgGhHIjklmMnpPrRStTuUVwWxXyYzZ%";
        let forms = [
            "", "-", "_", "0", "^", "#", "5", "-5", "_5", "05", "^5", "E", "O",
        ];
        let zones = [(0, "UTC"), (60, "BST"), (-480, "PST"), (345, "+0545")];
        let mut compared = 0;
        let mut seen = std::collections::BTreeSet::new();
        for step in 0..400i64 {
            // Eleven days and some hours apart, from the first days of 2020.
            let epoch_ms = (1_577_836_800 + step * 987_654) * 1000;
            let (offset, zone) = zones[step as usize % zones.len()];
            let wall = Wall::at(epoch_ms, offset, zone);
            for letter in letters.chars() {
                for form in forms {
                    // The library pads the sign and the digits of `%z`
                    // apart, to twice the width: not followed.
                    if letter == 'z' && form.ends_with('5') {
                        continue;
                    }
                    let pattern = format!("x%{form}{letter}x");
                    let (ours, theirs) = (format(&pattern, &wall), by_the_library(&pattern, &wall));
                    if ours != theirs && seen.insert(pattern.clone()) {
                        println!("DIFF {pattern}: ours {ours:?} theirs {theirs:?}");
                    }
                    compared += 1;
                }
            }
        }
        assert!(compared > 190_000);
        assert!(
            seen.is_empty(),
            "{} patterns read otherwise than the C library reads them",
            seen.len()
        );
    }
}
