//! Test-only crate: see `tests/`. It asserts things about the font files in `fonts/`, so a font
//! that is replaced, truncated or swapped for one without Japanese coverage fails CI here, not in
//! an app.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

/// The repository root (where `fonts/` lives).
pub fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
