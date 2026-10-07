//! The face for a browser: Glass's pipeline for a page, with the face
//! over it as on the player's own screen. The Glass Manager serves this
//! module to its Face tab and to Anymote in the place of Glass's own.
//!
//! A page keeps the face themes in its file table under the home's
//! `faces`, the user's own, and `faces-shipped`, the ones glass-evo comes
//! with, as the launcher names two folders on a player.
//!
//! Beside the pipeline's exports the module has `clock_preview`, the clock
//! alone as a look's keys draw it, and `line_preview`, the date or a clock
//! in type set in the look's font, for the Manager's look panel to show the
//! idle screen before it is saved, drawn as the face draws it.

page::exports!(|| Some(Box::new(face::Face::with_faces(
    "/glass/faces:/glass/faces-shipped"
))));

/// The clock alone, for a page: `clock_preview(json, len, size, epoch_ms,
/// offset_minutes)` draws the clock a look's keys describe (a JSON object
/// of the face's keys, `{"clock.face": "dial", ...}`) `size` pixels high at
/// a moment in a zone, and answers a pointer to its RGBA, straight alpha,
/// `clock_preview_width` by `clock_preview_height`; null where the clock
/// is set in type or not shown. The bytes stand until the next call.
#[cfg(target_arch = "wasm32")]
mod preview {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    thread_local! {
        static DRAWN: RefCell<face::clock::Drawn> = RefCell::new(face::clock::Drawn::default());
        static SIZE: RefCell<(u32, u32)> = const { RefCell::new((0, 0)) };
        static LINE: RefCell<Option<Frame>> = const { RefCell::new(None) };
        static LINE_SIZE: RefCell<(u32, u32)> = const { RefCell::new((0, 0)) };
        static FORECAST: RefCell<Option<Frame>> = const { RefCell::new(None) };
        static FORECAST_SIZE: RefCell<(u32, u32)> = const { RefCell::new((0, 0)) };
        static FONTS: RefCell<Option<Fonts>> = const { RefCell::new(None) };
    }

    use face::{FaceFonts as Fonts, FaceFrame as Frame, FontFiles};

    /// The look's bold font, as the page put it at `fonts/bold.ttf` under
    /// the module's home before asking for a line; loaded once, for every
    /// style.
    fn bold_font() -> String {
        format!("{}/fonts/bold.ttf", page::HOME)
    }

    /// The page's reading as the face reads it: the `weather` line's JSON,
    /// or nothing for an empty or broken one.
    ///
    /// # Safety
    /// The pointer names a buffer of the module's `alloc` of the given
    /// length, or the length is zero.
    unsafe fn reading_of(reading: *const u8, len: usize) -> Option<face::FaceWeather> {
        if len == 0 || reading.is_null() {
            return None;
        }
        let text = String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(reading, len) });
        serde_json::from_str::<face::FaceWeather>(&text).ok()
    }

    /// The page's keys as the face reads them.
    fn keys_of(json: *const u8, len: usize) -> BTreeMap<String, String> {
        let text = if len == 0 {
            String::new()
        } else {
            // SAFETY: the caller names a buffer of the module's `alloc`.
            String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(json, len) }).into_owned()
        };
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|value| value.as_object().cloned())
            .map(|object| {
                object
                    .into_iter()
                    .map(|(key, value)| {
                        let said = value
                            .as_str()
                            .map_or_else(|| value.to_string(), str::to_string);
                        (key, said)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// A line of type, for a page: `line_preview(json, len, reading,
    /// reading_len, which, size, epoch_ms, offset_minutes)` sets the date
    /// (`which` 0) or the clock in type (1) as a look's keys describe it,
    /// `size` pixels high, in the font the page put at `fonts/bold.ttf`
    /// under its home; the reading (the `weather` line's JSON, or nothing)
    /// colours the date where the look's heatmap reaches it. Answers a pointer
    /// to its RGBA, straight alpha, `line_preview_width` by
    /// `line_preview_height`; null where there is no font, the piece is not
    /// shown, or the clock is drawn and not set. The bytes stand until the
    /// next call.
    ///
    /// # Safety
    /// The pointer names a buffer of the module's `alloc` of the given length.
    #[no_mangle]
    pub unsafe extern "C" fn line_preview(
        json: *const u8,
        len: usize,
        reading: *const u8,
        reading_len: usize,
        which: u32,
        size: u32,
        epoch_ms: u64,
        offset_minutes: i32,
    ) -> *const u8 {
        let keys = keys_of(json, len);
        // SAFETY: the caller names a buffer of the module's `alloc`, or none.
        let weather = unsafe { reading_of(reading, reading_len) };
        let wall = face::Wall::at(epoch_ms as i64, offset_minutes, "");
        let which = if which == 1 { "clock" } else { "date" };
        FONTS.with(|fonts| {
            let mut fonts = fonts.borrow_mut();
            let fonts = fonts.get_or_insert_with(|| {
                Fonts::load(&FontFiles {
                    light: bold_font(),
                    regular: bold_font(),
                    bold: bold_font(),
                    italic: bold_font(),
                    digi: bold_font(),
                    fallback: String::new(),
                })
            });
            LINE.with(|line| {
                let mut line = line.borrow_mut();
                *line = face::line_preview(
                    fonts,
                    &keys,
                    which,
                    size.clamp(8, 2000),
                    &wall,
                    weather.as_ref(),
                );
                match line.as_ref() {
                    Some(picture) => {
                        LINE_SIZE.with(|slot| *slot.borrow_mut() = (picture.width, picture.height));
                        picture.rgba.as_ptr()
                    }
                    None => {
                        LINE_SIZE.with(|slot| *slot.borrow_mut() = (0, 0));
                        std::ptr::null()
                    }
                }
            })
        })
    }

    #[no_mangle]
    pub extern "C" fn line_preview_width() -> u32 {
        LINE_SIZE.with(|slot| slot.borrow().0)
    }

    #[no_mangle]
    pub extern "C" fn line_preview_height() -> u32 {
        LINE_SIZE.with(|slot| slot.borrow().1)
    }

    /// Today's forecast, for a page: `forecast_preview(json, len, reading,
    /// reading_len, size, epoch_ms, offset_minutes)` sets it as a look's
    /// keys describe it from the player's reading (the `weather` line's
    /// JSON), `size` pixels high, in the font the page put at
    /// `fonts/bold.ttf` under its home, and answers a pointer to its RGBA,
    /// straight alpha, `forecast_preview_width` by `forecast_preview_height`;
    /// null where there is no font, no reading, or the look hides it. The
    /// bytes stand until the next call.
    ///
    /// # Safety
    /// Each pointer names a buffer of the module's `alloc` of the given length.
    #[no_mangle]
    pub unsafe extern "C" fn forecast_preview(
        json: *const u8,
        len: usize,
        reading: *const u8,
        reading_len: usize,
        size: u32,
        epoch_ms: u64,
        offset_minutes: i32,
    ) -> *const u8 {
        let keys = keys_of(json, len);
        // SAFETY: the caller names a buffer of the module's `alloc`, or none.
        let weather = unsafe { reading_of(reading, reading_len) };
        let wall = face::Wall::at(epoch_ms as i64, offset_minutes, "");
        FONTS.with(|fonts| {
            let mut fonts = fonts.borrow_mut();
            let fonts = fonts.get_or_insert_with(|| {
                Fonts::load(&FontFiles {
                    light: bold_font(),
                    regular: bold_font(),
                    bold: bold_font(),
                    italic: bold_font(),
                    digi: bold_font(),
                    fallback: String::new(),
                })
            });
            FORECAST.with(|line| {
                let mut line = line.borrow_mut();
                *line = weather.as_ref().and_then(|weather| {
                    face::forecast_preview(
                        fonts,
                        &keys,
                        weather,
                        size.clamp(8, 2000),
                        &wall,
                        epoch_ms,
                        "/glass/faces:/glass/faces-shipped",
                    )
                });
                match line.as_ref() {
                    Some(picture) => {
                        FORECAST_SIZE
                            .with(|slot| *slot.borrow_mut() = (picture.width, picture.height));
                        picture.rgba.as_ptr()
                    }
                    None => {
                        FORECAST_SIZE.with(|slot| *slot.borrow_mut() = (0, 0));
                        std::ptr::null()
                    }
                }
            })
        })
    }

    #[no_mangle]
    pub extern "C" fn forecast_preview_width() -> u32 {
        FORECAST_SIZE.with(|slot| slot.borrow().0)
    }

    #[no_mangle]
    pub extern "C" fn forecast_preview_height() -> u32 {
        FORECAST_SIZE.with(|slot| slot.borrow().1)
    }

    /// # Safety
    /// The pointer names a buffer of the module's `alloc` of the given length.
    #[no_mangle]
    pub unsafe extern "C" fn clock_preview(
        json: *const u8,
        len: usize,
        size: u32,
        epoch_ms: u64,
        offset_minutes: i32,
    ) -> *const u8 {
        let text = if len == 0 {
            String::new()
        } else {
            String::from_utf8_lossy(std::slice::from_raw_parts(json, len)).into_owned()
        };
        let keys: BTreeMap<String, String> = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|value| value.as_object().cloned())
            .map(|object| {
                object
                    .into_iter()
                    .map(|(key, value)| {
                        let said = value
                            .as_str()
                            .map_or_else(|| value.to_string(), str::to_string);
                        (key, said)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let wall = face::Wall::at(epoch_ms as i64, offset_minutes, "");
        DRAWN.with(|drawn| {
            let mut drawn = drawn.borrow_mut();
            match face::clock_preview(&mut drawn, &keys, size.clamp(8, 2000), &wall, epoch_ms) {
                Some(picture) => {
                    SIZE.with(|slot| *slot.borrow_mut() = (picture.width, picture.height));
                    picture.rgba.as_ptr()
                }
                None => {
                    SIZE.with(|slot| *slot.borrow_mut() = (0, 0));
                    std::ptr::null()
                }
            }
        })
    }

    #[no_mangle]
    pub extern "C" fn clock_preview_width() -> u32 {
        SIZE.with(|slot| slot.borrow().0)
    }

    #[no_mangle]
    pub extern "C" fn clock_preview_height() -> u32 {
        SIZE.with(|slot| slot.borrow().1)
    }
}
