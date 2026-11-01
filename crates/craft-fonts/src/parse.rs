//! The `fonts/manifest.txt` format, shared by `build.rs` and the library (`#[path]`-included by
//! both, so it has no dependencies). The same parser reads the opt-in `fonts/latin-manifest.txt`
//! and `fonts/simplified-chinese-manifest.txt`. Apps that read a manifest from their own
//! `build.rs` can copy this file.

/// One line of the manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Family name, as the font's own `name` table spells it (e.g. `Shippori Mincho`).
    pub family: String,
    /// Style within the family (e.g. `Regular`, `Bold`).
    pub style: String,
    /// Path of the font file, relative to the repository root.
    pub file: String,
    /// ISO 15924 script codes the font is meant for (e.g. `Jpan`, `Latn`).
    pub scripts: Vec<String>,
    /// SPDX licence identifier.
    pub licence: String,
    /// Path of the licence text, relative to the repository root.
    pub licence_file: String,
    /// Lower-case hex SHA-256 of the font file.
    pub sha256: String,
    /// Where the file came from (a pinned upstream URL).
    pub source: String,
}

/// Why a manifest line could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestError {
    /// The line does not have the 8 `" | "`-separated fields.
    FieldCount { line: usize, found: usize },
    /// A field is empty.
    EmptyField { line: usize, field: &'static str },
    /// The SHA-256 is not 64 lower-case hex digits.
    BadSha256 { line: usize },
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldCount { line, found } => write!(
                f,
                "manifest line {line}: expected 8 fields separated by \" | \", found {found}"
            ),
            Self::EmptyField { line, field } => {
                write!(f, "manifest line {line}: the {field} field is empty")
            }
            Self::BadSha256 { line } => write!(
                f,
                "manifest line {line}: sha256 must be 64 lower-case hex digits"
            ),
        }
    }
}

impl std::error::Error for ManifestError {}

const FIELDS: [&str; 8] = [
    "family",
    "style",
    "file",
    "scripts",
    "licence",
    "licence file",
    "sha256",
    "source",
];

/// Parse the whole manifest. Blank lines and `#` comments are skipped; line numbers are 1-based.
pub fn parse_manifest(text: &str) -> Result<Vec<Entry>, ManifestError> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = trimmed.split(" | ").map(str::trim).collect();
        let [
            family,
            style,
            file,
            scripts,
            licence,
            licence_file,
            sha256,
            source,
        ] = fields.as_slice()
        else {
            return Err(ManifestError::FieldCount {
                line,
                found: fields.len(),
            });
        };
        for (name, value) in FIELDS.iter().zip([
            family,
            style,
            file,
            scripts,
            licence,
            licence_file,
            sha256,
            source,
        ]) {
            if value.is_empty() {
                return Err(ManifestError::EmptyField { line, field: name });
            }
        }
        if sha256.len() != 64
            || !sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(ManifestError::BadSha256 { line });
        }
        out.push(Entry {
            family: (*family).to_string(),
            style: (*style).to_string(),
            file: (*file).to_string(),
            scripts: scripts
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            licence: (*licence).to_string(),
            licence_file: (*licence_file).to_string(),
            sha256: (*sha256).to_string(),
            source: (*source).to_string(),
        });
    }
    Ok(out)
}
