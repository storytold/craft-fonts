//! The Crafting Apps' shared font assets.
//!
//! The fonts live in this repository's `fonts/` directory and are listed in `fonts/manifest.txt`.
//! This crate exposes that manifest ([`FONTS`]), finds a craft-fonts checkout at run time
//! ([`dir_from_env`]), loads fonts from one ([`load`]), and with the `embed` feature compiles them
//! into the binary ([`embedded`]). `fonts/simplified-chinese-manifest.txt` is a separate opt-in
//! native-only manifest; it is not part of [`FONTS`] or the `embed` payload, and consumers or
//! tests parse it with [`parse_manifest`].
//!
//! The apps do **not** depend on this crate: Cargo resolves even optional git dependencies, so a
//! dependency would make craft-fonts required for every build. Apps read `fonts/manifest.txt`
//! from their own `build.rs` when the `CRAFT_FONTS_DIR` build option is set; see
//! `docs/integration.md`. This crate is for tools and tests that can afford the dependency, and it
//! keeps the manifest format ([`parse_manifest`]) defined in one place.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

mod parse;

use std::path::{Path, PathBuf};

pub use parse::{Entry, ManifestError, parse_manifest};

/// One font file from `fonts/manifest.txt`, known at compile time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Font {
    /// Family name, as the font's own `name` table spells it.
    pub family: &'static str,
    /// Style within the family (`Regular`, `Bold`, …).
    pub style: &'static str,
    /// Path of the font file, relative to the repository root.
    pub file: &'static str,
    /// ISO 15924 script codes the font is meant for.
    pub scripts: &'static [&'static str],
    /// SPDX licence identifier.
    pub licence: &'static str,
    /// Path of the licence text, relative to the repository root.
    pub licence_file: &'static str,
    /// Lower-case hex SHA-256 of the file.
    pub sha256: &'static str,
    /// Pinned upstream source.
    pub source: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/manifest.rs"));

/// The fonts meant for an ISO 15924 script (e.g. `"Jpan"`), in manifest order.
pub fn for_script(script: &str) -> impl Iterator<Item = &'static Font> + '_ {
    FONTS.iter().filter(move |f| f.scripts.contains(&script))
}

/// The compiled-in bytes of a font from [`FONTS`] (`embed` feature). `None` for any other value.
#[cfg(feature = "embed")]
pub fn embedded(font: &Font) -> Option<&'static [u8]> {
    FONTS
        .iter()
        .position(|f| f == font)
        .and_then(|i| EMBEDDED.get(i).copied())
}

/// The environment variable (and build option) naming a craft-fonts checkout.
pub const DIR_ENV: &str = "CRAFT_FONTS_DIR";

/// The craft-fonts checkout named by `CRAFT_FONTS_DIR`, if it is set and holds a manifest.
pub fn dir_from_env() -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os(DIR_ENV)?);
    dir.join("fonts/manifest.txt").is_file().then_some(dir)
}

/// Why fonts could not be loaded from a checkout.
#[derive(Debug)]
pub enum LoadError {
    /// A file could not be read.
    Io {
        path: PathBuf,
        error: std::io::Error,
    },
    /// The manifest is malformed.
    Manifest(ManifestError),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, error } => write!(f, "reading {}: {error}", path.display()),
            Self::Manifest(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for LoadError {}

/// Read the manifest of the checkout at `dir` and every font it lists, in manifest order.
pub fn load(dir: &Path) -> Result<Vec<(Entry, Vec<u8>)>, LoadError> {
    let manifest = dir.join("fonts/manifest.txt");
    let text = std::fs::read_to_string(&manifest).map_err(|error| LoadError::Io {
        path: manifest,
        error,
    })?;
    let entries = parse_manifest(&text).map_err(LoadError::Manifest)?;
    entries
        .into_iter()
        .map(|e| {
            let path = dir.join(&e.file);
            std::fs::read(&path)
                .map(|bytes| (e, bytes))
                .map_err(|error| LoadError::Io { path, error })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_manifest_parses_into_the_static_table() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let text = std::fs::read_to_string(root.join("fonts/manifest.txt")).unwrap();
        let entries = parse_manifest(&text).unwrap();
        assert_eq!(entries.len(), FONTS.len());
        for (e, f) in entries.iter().zip(FONTS) {
            assert_eq!(
                (e.family.as_str(), e.style.as_str(), e.file.as_str()),
                (f.family, f.style, f.file)
            );
        }
        assert!(for_script("Jpan").count() >= 1);
    }

    #[test]
    fn malformed_lines_are_errors_not_panics() {
        assert_eq!(
            parse_manifest("a | b"),
            Err(ManifestError::FieldCount { line: 1, found: 2 })
        );
        let blank_style = format!("A |  | f | Jpan | OFL-1.1 | l | {} | s", "0".repeat(64));
        assert_eq!(
            parse_manifest(&blank_style),
            Err(ManifestError::EmptyField {
                line: 1,
                field: "style"
            })
        );
        assert_eq!(
            parse_manifest("A | B | f | Jpan | OFL-1.1 | l | XYZ | s"),
            Err(ManifestError::BadSha256 { line: 1 })
        );
        assert_eq!(parse_manifest("# comment\n\n"), Ok(Vec::new()));
    }

    #[test]
    fn a_missing_checkout_is_an_error() {
        assert!(matches!(
            load(Path::new("/nonexistent/craft-fonts")),
            Err(LoadError::Io { .. })
        ));
    }

    #[cfg(feature = "embed")]
    #[test]
    fn embedded_bytes_match_the_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for f in FONTS {
            assert_eq!(
                embedded(f).unwrap(),
                std::fs::read(root.join(f.file)).unwrap().as_slice(),
                "{}",
                f.file
            );
        }
    }
}
