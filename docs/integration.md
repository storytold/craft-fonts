# Using craft-fonts in a Crafting App

craft-fonts is a **fully optional** source of fonts. An app must build, test and run without it;
with it, the app embeds the fonts listed in `fonts/manifest.txt` and uses them (today: Japanese,
Korean, Hebrew, Indian-script and UI-symbol fallback). A native consumer may additionally opt in to
`fonts/simplified-chinese-manifest.txt` for the LXGW WenKai Lite Simplified Chinese fallback; that
manifest is never part of the default set or a wasm build.

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
    let required = std::env::var_os("CRAFT_FONTS_REQUIRED").is_some();
    if let Some(dir) = std::env::var_os("CRAFT_FONTS_DIR").map(PathBuf::from) {
        match craft_fonts(&dir) {
            Ok(entries) => src.push_str(&entries),
            Err(e) if required => {
                println!("cargo::error=CRAFT_FONTS_DIR={}: {e}", dir.display());
            }
            Err(e) => println!("cargo::warning=building without craft-fonts: CRAFT_FONTS_DIR={}: {e}", dir.display()),
        }
    } else if required {
        println!("cargo::error=CRAFT_FONTS_REQUIRED is set but CRAFT_FONTS_DIR is missing");
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
    // Web builds use an explicit allowlist. Never embed the whole manifest or Noto CJK.
    // GridCraft's selected faces are BIZ UDPGothic Regular/Bold, NanumGothic Regular,
    // Noto Sans Symbols Regular and Noto Sans Symbols 2 Regular (verified cmap coverage requires
    // both symbol faces). Measure raw font bytes, raw wasm and compressed wasm before/after.
    const WEB_FONTS: &[(&str, &str)] = &[
        ("BIZ UDPGothic", "Regular"),
        ("BIZ UDPGothic", "Bold"),
        ("NanumGothic", "Regular"),
        ("Noto Sans Symbols", "Regular"),
        ("Noto Sans Symbols 2", "Regular"),
    ];
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").is_ok_and(|a| a == "wasm32");
    let mut out = String::new();
    let mut included = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let f: Vec<&str> = line.split(" | ").map(str::trim).collect();
        let [family, style, file, scripts, ..] = f.as_slice() else {
            return Err(format!("malformed manifest line: {line}"));
        };
        if wasm && !WEB_FONTS.contains(&(*family, *style)) {
            continue;
        }
        let path = dir.join(file).canonicalize().map_err(|e| format!("{file}: {e}"))?;
        included.push((*family, *style));
        println!("cargo::rerun-if-changed={}", path.display());
        let scripts: Vec<String> = scripts.split(',').map(|s| format!("{:?}", s.trim())).collect();
        let _ = writeln!(
            out,
            "    CraftFont {{ family: {family:?}, style: {style:?}, scripts: &[{}], bytes: include_bytes!({:?}) }},",
            scripts.join(", "),
            path.display().to_string(),
        );
    }
    if std::env::var_os("CRAFT_FONTS_REQUIRED").is_some() {
        for &(family, style) in WEB_FONTS {
            if !included.contains(&(family, style)) {
                return Err(format!("required fallback face is missing: {family} {style}"));
            }
        }
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

## Optional: the Latin presentation fonts (`fonts/latin-manifest.txt`)

`fonts/latin-manifest.txt` is a **second, opt-in manifest** in exactly the same format as
`fonts/manifest.txt` (same 8 fields, same parser in `crates/craft-fonts/src/parse.rs`). It lists
the Latin presentation catalog: Inter, the metric-compatible Office substitutes (Carlito for
Calibri, Caladea for Cambria, Liberation Sans / Serif / Mono for Arial / Times New Roman /
Courier New), Source Sans 3, Source Serif 4, Montserrat, Lato, Open Sans, Roboto, Merriweather,
Playfair Display, Poppins and Nunito Sans. The central catalog is **16 families, 51 faces, about
19.5 MiB** (20,417,364 bytes of font files). The fonts are listed in `README.md` and
`ATTRIBUTION.md`.

The recipe above reads only `fonts/manifest.txt`, so **apps that don't opt in are unchanged**. An
app that wants these fonts reads the second manifest too, in addition to the first. Generalise
`craft_fonts` to take the manifest path and its web allowlist, and call it once per manifest:

```rust
/// Manifests this app embeds, each with the (family, style) pairs its wasm32 build keeps.
const MANIFESTS: &[(&str, &[(&str, &str)])] = &[
    ("fonts/manifest.txt", &[("BIZ UDPGothic", "Regular")]),
    // Opt in to the Latin catalog; on the web keep only Inter Regular + Bold.
    ("fonts/latin-manifest.txt", &[("Inter", "Regular"), ("Inter", "Bold")]),
];

// In main(), match on this instead of on craft_fonts(&dir) (same Ok/Err handling):
//     MANIFESTS.iter().map(|(rel, web)| craft_fonts(&dir, rel, web)).collect::<Result<String, String>>()
// and in craft_fonts(dir, rel, web): read dir.join(rel) instead of "fonts/manifest.txt",
// and use `web` instead of WEB_FONTS.
```

- **Consumer filter (required for some apps).** Eight faces are Adobe-authored: Source Sans 3
  (4) and Source Serif 4 (4), 2,594,420 bytes. They remain in the central catalog and are valid
  OFL-1.1 assets, but a consumer that must not embed Adobe-authored faces skips those two
  families, leaving **43** permitted Latin faces. DeckCraft already excludes them; FigCraft adds
  the same family filter in its own `build.rs`. The main manifest is unaffected.
- **Styles.** Most families are static files (`Regular`, `Bold`, `Italic`, `Bold Italic`,
  `SemiBold` / `Semibold` as each family names it). Montserrat, Open Sans, Roboto, Playfair
  Display and Nunito Sans are variable fonts, one file per upright/italic, listed with style
  `Variable` or `Italic Variable`: register them once and select weights through the `wght` axis
  (set it explicitly, e.g. 400 / 700: a variable font's default instance isn't always Regular).
  The tests check that exactly these styles have a `wght` axis with a real range containing 400.
- **Scripts.** Every line is tagged `Latn` (many also cover Greek and Cyrillic, but nothing
  asserts that, so don't rely on it).
- **Office documents.** When a document asks for Calibri, Cambria, Arial, Times New Roman or
  Courier New and the system doesn't have it, substitute Carlito, Caladea, Liberation Sans,
  Liberation Serif or Liberation Mono: same advance widths, so line breaks and text boxes match.
- **Web (wasm32).** Embed only **Inter Regular and Inter Bold** (~0.8 MB) from this manifest, as
  in the allowlist above; the full catalog would push a web build past typical per-file hosting
  limits together with the main manifest's fonts.
- **Licences.** All are OFL-1.1 and each family directory has its `OFL.txt`, so the packaging rule
  below (copy `fonts/*/OFL.txt` for each embedded font) covers them unchanged.
- From Rust tools that can depend on this repo, the same table is `craft_fonts::LATIN_FONTS`
  (`load_latin`, feature `embed-latin`). The default `FONTS` table and `embed` feature remain
  exactly `fonts/manifest.txt`.

## Optional native-only manifest

`fonts/simplified-chinese-manifest.txt` uses the same eight-field format and parser, but it is a
separate opt-in input. Read it only for non-wasm builds, after the main manifest has been parsed:

- **Native builds only.** Never read it from a wasm branch and never add its row to `WEB_FONTS`.
  Native builds that do not opt in embed no added bytes.
- **Missing file is compatible.** An older craft-fonts checkout without the optional manifest is
  not an error; continue with the main manifest's fonts.
- **Malformed or unreadable when present.** Under `CRAFT_FONTS_REQUIRED=1`, fail the build. In
  optional mode, emit a `cargo::warning` and keep every successfully parsed font from the main
  manifest. An optional-manifest error must never discard the main fonts.
- **Required mode still needs a directory.** `CRAFT_FONTS_REQUIRED=1` with no `CRAFT_FONTS_DIR`
  is a build error, as in the recipe above.
- **Not in `FONTS`/embed.** `craft-fonts`' default `FONTS` table and `embed` feature cover
  only `fonts/manifest.txt`; the optional row appears in neither that table nor
  `LATIN_FONTS`/`embed-latin`. Apps supply their own static initializer from the parsed
  optional file.
- **Native fallback order.** Append `LXGW WenKai Lite` / `Regular` after the existing native
  families, so it fills characters they lack without taking precedence over BIZ for characters
  BIZ already covers. It is a per-character fallback, not a promise of uniform Chinese
  typography or of system-font-first precedence. It does not make static localized UI labels
  render in Chinese unless the consumer installs the face for those labels too.

The central tests in `crates/craft-fonts-tests/tests/fonts.rs` parse the optional file with
`craft_fonts::parse_manifest` and verify the exact row, SHA-256, adjacent licence and RFN text,
one static weight-400 face without `fvar`/`avar`/`STAT`, outlines for `简体中文演示文稿` and
`简`, the 24 MiB ceiling, and that the face is absent from `FONTS`/`fonts/manifest.txt`.

## Using the fonts

- **UI (egui):** append permitted craft faces at the end of each interface font family, after
  bundled fonts, system fonts and existing script fallbacks. Register Japanese (`Jpan`) faces,
  `NanumGothic` / `Regular` for `Hang,Kore`, `Noto Sans Hebrew` / `Regular` for `Hebr`,
  `Noto Sans Devanagari` / `Regular` for `Deva`, and both
  `Noto Sans Symbols` / `Regular` and `Noto Sans Symbols 2` / `Regular` for `Zsym`. The Hebrew
  file is one static face: add it once per family set that includes it (for example the
  proportional family and, where the consumer has one, the `CELL`/monospace family). Do not
  expand it into the nine named instances of the old variable fixture. A native consumer that
  opts in to the Simplified Chinese manifest adds `LXGW WenKai Lite` / `Regular` for `Hani,Hans`
  after those existing families. Match the exact family/style spelling; in particular,
  `NanumGothic` has no space. Verify the installed
  production set with `CRAFT_FONTS_REQUIRED=1` and strict glyph assertions for each advertised
  script and symbol. Hebrew coverage here means each codepoint has a drawn outline. The
  storytold egui fork uses HarfRust shaping and bidi layout, but font-only tests do not
  qualify correctly positioned vocalized (pointed) Hebrew in an app. The initial glyph
  acceptance text is unpointed, e.g. `שלום deck 42`; shaping needs separate app-level checks.
  Devanagari coverage likewise means each codepoint has a drawn outline: the font-only tests
  do not qualify conjunct formation or mark positioning, which need app-level shaping checks
  such as PhotoCraft's Camera Raw language-switch test with `फ़ा` (U+092B U+093C U+093E).
- **Document text engines:** register the same permitted script and symbol faces after the
  document's requested font and the app's existing fallback faces. Preserve the app's asset
  exclusions, and do not treat a script tag as proof that every character in that script exists.
- **Web (wasm32):** the recipe uses a fixed allowlist. Never embed the whole manifest: the
  current faces together are roughly 47.9 MiB (50.3 MB), over typical per-file hosting limits.
  Measure the raw bytes of each allowlisted font, plus raw and compressed wasm before and after,
  against the consumer's actual web gate. Do not call the result "within budget" without a named
  number and a real check. GridCraft's decision is documented in
  [`egui-037-font-coverage-plan.md`](egui-037-font-coverage-plan.md): BIZ UDPGothic Regular/Bold,
  NanumGothic Regular, Noto Sans Symbols Regular and Noto Sans Symbols 2 Regular (the verified
  cmap shows Symbols 1 lacks `✕`/`◀` and Symbols 2 supplies them), and no Noto CJK. Hebrew is
  deliberately **not** in that web allowlist: `Noto Sans Hebrew` is a native-only addition in
  this round, and web Hebrew remains an unqualified gap. `LXGW WenKai Lite` is likewise
  native-only and must stay out of every wasm allowlist. No existing web allowlist adds
  `Noto Sans Devanagari`; keep every wasm allowlist explicit.
- **Indian scripts:** eight more Noto Sans faces (`Beng`, `Guru`, `Gujr`, `Orya`, `Taml`, `Telu`,
  `Knda`, `Mlym`) join `Noto Sans Devanagari` (`Deva`, above), 0.3–0.9 MB each. Register them like
  Devanagari, after the system faces. Windows ships faces for all of them (Nirmala UI) and macOS
  does too (Kohinoor, the Sangam MN families, Gurmukhi MN), but a minimal Linux install may have
  none, so keep the craft faces as the last fallback there. A web build that wants them lists each
  one in its allowlist explicitly and measures it against its web gate, as above. Their tests check
  codepoint coverage (each character maps to a glyph), not drawn outlines as for Devanagari;
  conjuncts and mark positioning need app-level shaping checks either way.
- Prefer `BIZ UDPGothic` for UI text and `Shippori Mincho` / `BIZ UDMincho` for serif document
  text.
- Optional-input diagnostics may skip when `CRAFT_FONTS` is empty. Acceptance tests that
  validate advertised scripts must run with `CRAFT_FONTS_DIR` set and `CRAFT_FONTS_REQUIRED=1`,
  and must fail (not skip) when a required face or codepoint is missing. Never weaken strict
  glyph checks, substitute tofu comparisons, or use `allow_missing_glyphs()` to make acceptance
  pass.

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
2. Add a manifest line (with its SHA-256) and an `ATTRIBUTION.md` row. A Latin presentation face
   belongs in `fonts/latin-manifest.txt`; a face that only some native consumers should embed
   belongs in `fonts/simplified-chinese-manifest.txt`; neither belongs in the default
   `fonts/manifest.txt`.
3. `cargo test --workspace`: `craft-fonts-tests` checks the checksum, licence, family and style
   names, and, for Japanese fonts, coverage and vertical-text support. Add exact ISO 15924
   script assertions for any new face. Coverage tests may claim only glyphs the face really
   contains: check outlines as well as cmap entries, do not claim a codepoint supplied by
   another face (egui's emoji faces supply `📋`), and do not claim full-script coverage from a
   sample string. Static faces also get metadata assertions (no `fvar`/`STAT`, weight 400),
   so a variable font with a Thin default cannot pass as a one-face Regular.
