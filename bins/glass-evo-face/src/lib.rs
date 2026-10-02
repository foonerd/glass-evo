//! The face for a browser: Glass's pipeline for a page, with the face
//! over it as on the player's own screen. The Glass Manager serves this
//! module to its Face tab and to Anymote in the place of Glass's own.
//!
//! A page keeps the face themes in its file table under the home's
//! `faces`, the user's own, and `faces-shipped`, the ones glass-evo comes
//! with, as the launcher names two folders on a player.
//!
//! Beside the pipeline's exports the module has `clock_preview`, the clock
//! alone as a look's keys draw it, for the Manager's look panel to show a
//! drawn clock face before it is saved.

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
