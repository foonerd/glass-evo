//! The face's library: what glass-evo draws and does, shared by the device
//! screen, the remotes and the browser. This is the scaffold: the crate
//! names itself and nothing more yet.

/// The workspace version, as Cargo knows it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The line the binary prints to say what it is.
pub fn banner() -> String {
    format!("glass-evo {VERSION}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_banner_names_the_face_and_its_version() {
        assert_eq!(banner(), format!("glass-evo {}", env!("CARGO_PKG_VERSION")));
    }
}
