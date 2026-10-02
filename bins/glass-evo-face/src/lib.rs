//! The face for a browser: Glass's pipeline for a page, with the face
//! over it as on the player's own screen. The Glass Manager serves this
//! module to its Face tab and to Anymote in the place of Glass's own.
//!
//! A page keeps the face themes in its file table under the home's
//! `faces`, the user's own, and `faces-shipped`, the ones glass-evo comes
//! with, as the launcher names two folders on a player.

page::exports!(|| Some(Box::new(face::Face::with_faces(
    "/glass/faces:/glass/faces-shipped"
))));
