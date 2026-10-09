# Using craft-fonts in a Crafting App

craft-fonts is a **fully optional** source of fonts. An app must build, test and run without it;
with it, the app embeds the fonts listed in `fonts/manifest.txt` and uses them (today: Japanese
UI and document fallback).

## Why not a Cargo dependency

Cargo resolves every dependency in `Cargo.toml`, optional ones included, even when the feature
that enables them is off. A git dependency on craft-fonts would make every build (offline, in a
fork, on a mirror) need this repository. So apps never list craft-fonts in `Cargo.toml`. The build
option is an environment variable instead.

## The build option: `CRAFT_FONTS_DIR`

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR=$PWD/../craft-fonts cargo build --release
```

- **Unset** (the default): nothing changes. The app uses its own bundled fonts and system fonts.
- **Set to a craft-fonts checkout:** the app's `build.rs` reads `fonts/manifest.txt` and embeds
  every font it lists.
- **Set but not a checkout** (no manifest, missing file): the build prints a `cargo::warning` and
  carries on without craft-fonts, unless `CRAFT_FONTS_REQUIRED=1` is also set, which turns it
  into a build error. Release workflows set both, so a release can't silently ship without them.

Use an **absolute** path (e.g. `CRAFT_FONTS_DIR=$PWD/../craft-fonts`): Cargo runs build scripts in
the crate's own directory, so a relative path would resolve from there. `CRAFT_FONTS_DIR` is read by
`build.rs`, so changing it rebuilds only the crate that embeds the fonts.

## The `build.rs` recipe

Put this in the build script of the one crate that installs fonts (usually the UI or text crate).
It has no dependencies; the manifest format is defined in `crates/craft-fonts/src/parse.rs`.

```rust
use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_DIR");
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_REQUIRED");
    let mut src = String::from("pub static CRAFT_FONTS: &[CraftFont] = &[\n");
    if let Some(dir) = std::env::var_os("CRAFT_FONTS_DIR").map(PathBuf::from) {
        match craft_fonts(&dir) {
            Ok(entries) => src.push_str(&entries),
            Err(e) if std::env::var_os("CRAFT_FONTS_REQUIRED").is_some() => {
                println!("cargo::error=CRAFT_FONTS_DIR={}: {e}", dir.display());
            }
            Err(e) => println!("cargo::warning=building without craft-fonts: CRAFT_FONTS_DIR={}: {e}", dir.display()),
        }
    }
    src.push_str("];\n");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("craft_fonts.rs");
    if let Err(e) = std::fs::write(&out, src) {
        println!("cargo::error=writing {}: {e}", out.display());
    }
}

/// One `CraftFont { .. }` initialiser per manifest line.
fn craft_fonts(dir: &std::path::Path) -> Result<String, String> {
    let manifest = dir.join("fonts/manifest.txt");
    println!("cargo::rerun-if-changed={}", manifest.display());
    let text = std::fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
    // Web builds have a size budget (e.g. Cloudflare's 25 MiB per file): embed only the UI font there.
    const WEB_FONTS: &[(&str, &str)] = &[("BIZ UDPGothic", "Regular")];
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").is_ok_and(|a| a == "wasm32");
    let mut out = String::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let f: Vec<&str> = line.split(" | ").map(str::trim).collect();
        let [family, style, file, scripts, ..] = f.as_slice() else {
            return Err(format!("malformed manifest line: {line}"));
        };
        if wasm && !WEB_FONTS.contains(&(*family, *style)) {
            continue;
        }
        let path = dir.join(file).canonicalize().map_err(|e| format!("{file}: {e}"))?;
        println!("cargo::rerun-if-changed={}", path.display());
        let scripts: Vec<String> = scripts.split(',').map(|s| format!("{:?}", s.trim())).collect();
        let _ = writeln!(
            out,
            "    CraftFont {{ family: {family:?}, style: {style:?}, scripts: &[{}], bytes: include_bytes!({:?}) }},",
            scripts.join(", "),
            path.display().to_string(),
        );
    }
    Ok(out)
}
```

And in the crate:

```rust
/// A font from the optional craft-fonts build input (empty unless built with `CRAFT_FONTS_DIR`).
pub struct CraftFont {
    pub family: &'static str,
    pub style: &'static str,
    /// ISO 15924 scripts the font is for, e.g. `"Jpan"`.
    pub scripts: &'static [&'static str],
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/craft_fonts.rs"));
```

`CRAFT_FONTS` is always defined, so there is no `cfg` to thread through: an empty slice means the
app was built without craft-fonts. Code that uses it must work when it is empty.

## Using the fonts

- **UI (egui):** add each `Jpan` font as a fallback at the end of every font family, after the
  app's own fonts and before system-font discovery, so Japanese text renders the same everywhere
  the app was built with craft-fonts.
- **Document text engines:** register them as fallback faces for Japanese, after the document's
  requested font and the app's bundled fonts.
- **Arabic (`Arab`):** `Noto Sans Arabic` is the first Arabic font in the manifest and is meant as
  the fallback for Arabic text. The other Arabic families are for users to choose by name; register
  them in the document font database so they appear in font menus. Each is complete on its own
  (Arabic plus basic Latin) and joins letters (GSUB `init`/`medi`/`fina`, checked by the tests).
- **Web (wasm32):** the recipe embeds only `BIZ UDPGothic Regular` (~4.5 MB); all four fonts are
  ~24 MB, over typical per-file hosting limits. If the app's web build has a size check, measure
  with `CRAFT_FONTS_DIR` set and shrink `WEB_FONTS` (even to empty) if it fails. Fonts that don't
  fit can be served as separate files next to the wasm and fetched at startup instead of embedded
  (VectorCraft and PhotoCraft do this for the Arabic fonts; see their `crates/text/web-fonts.txt`).
- Prefer `BIZ UDPGothic` for UI text and `Shippori Mincho` / `BIZ UDMincho` for serif document
  text.
- Tests that assert on these fonts' glyphs must skip (not fail) when `CRAFT_FONTS` is empty, and
  CI should run them with `CRAFT_FONTS_DIR` set so they are exercised.

## Releases and licences

The fonts are OFL-1.1: they can be embedded and redistributed, but the licence text must go with
them. In `release.yml`, check out craft-fonts at a pinned commit and build with it:

```yaml
- uses: actions/checkout@v4
  with:
    repository: storytold/craft-fonts
    ref: <full commit SHA>        # bump deliberately
    path: craft-fonts
# then, on the build steps:
env:
  CRAFT_FONTS_DIR: ${{ github.workspace }}/craft-fonts
  CRAFT_FONTS_REQUIRED: "1"
```

Packages built with craft-fonts must include each embedded font's licence file (copy
`$CRAFT_FONTS_DIR/fonts/*/OFL.txt` into the package's licences, named after the family) and the
app's `NOTICE` must say that builds made with `CRAFT_FONTS_DIR` embed the fonts listed in
craft-fonts' `ATTRIBUTION.md`.

## Adding a font

1. Put the file and its licence under `fonts/<family>/`, from a pinned upstream commit.
2. Add a manifest line (with its SHA-256) and an `ATTRIBUTION.md` row.
3. `cargo test --workspace`: `craft-fonts-tests` checks the checksum, licence, family name and,
   for Japanese fonts, coverage and vertical-text support. Add assertions for any new script.
