//! The Android shell of the face: Glass's remote app with the face in it.
//! SDL's activity loads `libSDL2.so` and this library, `libmain.so`, and
//! calls [`SDL_main`] with the arguments the activity gives. Nothing is
//! decided here: Glass's own Android entry runs the display as a remote,
//! with the face over it.

use std::ffi::{c_char, c_int};

/// The entry point SDL's Java side calls once the activity is up. The
/// return value is only logged by SDL.
///
/// # Safety
///
/// `argv` holds `argc` C strings, as SDL passes them.
#[no_mangle]
pub unsafe extern "C" fn SDL_main(argc: c_int, argv: *const *const c_char) -> c_int {
    #[cfg(target_os = "android")]
    {
        glass::android::enter(argc, argv, Some(Box::new(face::Face::new())))
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (argc, argv);
        0
    }
}
