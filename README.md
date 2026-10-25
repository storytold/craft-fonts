craft-fonts
===========

<a href="https://sonarcloud.io/project/overview?id=storytold_craft-fonts"><img alt="Scanned by SonarCloud" src="https://img.shields.io/badge/Scanned_by-SonarCloud-4b9fd5?style=flat-square&logo=sonarqubecloud&logoColor=white"></a>

Reusable fonts for the Crafting Apps.

This library is meant to have the following:

1. Reusable font assets. These may be large files, and since we're using a separate repo for storing them,
   We're not afraid of potentially having to do a git reset to purge stale assets. It won't impact the other
   repos with core app and library functionality.

2. Additional cross-cutting Rust libraries for dealing with fonts. It's uncertain whether it's better to
   have code packages as individual repos as one classically does, or whether we should take a
   more "biology-inspired" approach of copying innovations between repos as a form of lateral code/information
   exchange.

## What's here

| Path | What |
|---|---|
| `fonts/` | The font files, one directory per family, each with its licence. |
| `fonts/manifest.txt` | Machine-readable list of the default fonts: family, style, file, scripts, licence, SHA-256, pinned source. |
| `fonts/latin-manifest.txt` | Opt-in Latin presentation catalog, same eight-field format: 16 families and 51 faces (about 19.5 MiB). Not in the default `FONTS` or `embed` payload. |
| `fonts/hebrew-manifest.txt` | Opt-in Hebrew presentation and document catalog, same eight-field format: 29 families and 56 faces (about 12.9 MiB). Not in the default `FONTS` or `embed` payload. |
| `fonts/simplified-chinese-manifest.txt` | Opt-in native-only Simplified Chinese row, same eight-field format. Not in `FONTS` or the embed payload. |
| `ATTRIBUTION.md` | Authors, pinned sources and licences. |
| `crates/craft-fonts` | Rust: the default and opt-in Latin/Hebrew manifests as static tables, finding a checkout (`CRAFT_FONTS_DIR`), loading them, and `embed` / `embed-latin` / `embed-hebrew` features that compile the fonts in. |
| `crates/craft-fonts-tests` | Tests asserted on these fonts: checksums, licences, family/style names, Inter/Latin provenance and faces, Japanese/Korean/Hebrew/Devanagari/symbol coverage and outlines, static-face metadata and vertical-text (`vert`/`vrt2`) support. |
| `docs/integration.md` | How an app uses craft-fonts as a fully optional build input. |

### Fonts

| Family | Styles | For | Licence |
|---|---|---|---|
| BIZ UDPGothic | Regular, Bold | Japanese UI text (sans) | OFL-1.1 |
| Shippori Mincho | Regular | Japanese document text (serif) | OFL-1.1 |
| BIZ UDMincho | Regular | Japanese document text (serif) | OFL-1.1 |
| Noto Sans CJK SC | Regular | Simplified Chinese UI and document text (sans) | OFL-1.1 |
| Noto Sans Arabic | Regular (variable: wdth, wght) | Arabic UI and document text (sans) | OFL-1.1 |
| Noto Sans Thai | Regular (variable: wdth, wght) | Thai UI and document text (sans) | OFL-1.1 |
| Noto Sans Georgian | Regular (variable: wdth, wght) | Georgian UI and document text (sans) | OFL-1.1 |
| NanumGothic | Regular | Korean UI and document text | OFL-1.1 |
| Noto Sans Symbols | Regular (variable: wght) | UI key/modifier symbols (`⌃`, arrows) | OFL-1.1 |
| Noto Sans Symbols 2 | Regular | UI symbols (`✕`, `◀`, `✓`) | OFL-1.1 |
| Noto Sans Hebrew | Regular (static) | Hebrew UI and document text | OFL-1.1 |
| Noto Sans Devanagari | Regular (static) | Devanagari UI and document text | OFL-1.1 |
| LXGW WenKai Lite | Regular (static) | Native-only opt-in Simplified Chinese fallback (Kai) | OFL-1.1 |

### Latin presentation fonts (`fonts/latin-manifest.txt`, opt-in)

These are not part of the default `FONTS` table or `embed` payload. Apps opt in by reading the
second manifest in their own `build.rs` (see [`docs/integration.md`](docs/integration.md)).
The central catalog is 16 families, 51 faces, about 19.5 MiB (20,417,364 bytes of font files).
"Variable" means one variable-font file (weight axis, some also width or optical size) per
upright/italic.

| Family | Styles | For | Licence |
|---|---|---|---|
| Inter | Regular, Italic, SemiBold, Bold | UI and document text (sans) | OFL-1.1 |
| Carlito | Regular, Bold, Italic, Bold Italic | Metric-compatible with Calibri | OFL-1.1 |
| Caladea | Regular, Bold, Italic, Bold Italic | Metric-compatible with Cambria | OFL-1.1 |
| Liberation Sans | Regular, Bold, Italic, Bold Italic | Metric-compatible with Arial | OFL-1.1 |
| Liberation Serif | Regular, Bold, Italic, Bold Italic | Metric-compatible with Times New Roman | OFL-1.1 |
| Liberation Mono | Regular, Bold, Italic, Bold Italic | Metric-compatible with Courier New | OFL-1.1 |
| Source Sans 3 | Regular, Italic, Semibold, Bold | Presentation text (sans; Adobe-authored, catalog-only for these consumers) | OFL-1.1 |
| Source Serif 4 | Regular, Italic, Semibold, Bold | Presentation text (serif; Adobe-authored, catalog-only for these consumers) | OFL-1.1 |
| Montserrat | Variable, Italic Variable | Headings (geometric sans) | OFL-1.1 |
| Lato | Regular, Bold, Italic | Presentation text (sans) | OFL-1.1 |
| Open Sans | Variable, Italic Variable | Presentation text (sans) | OFL-1.1 |
| Roboto | Variable, Italic Variable | Presentation text (sans) | OFL-1.1 |
| Merriweather | Regular, Bold, Italic | Presentation text (serif) | OFL-1.1 |
| Playfair Display | Variable, Italic Variable | Headings (display serif) | OFL-1.1 |
| Poppins | Regular, Bold, Italic | Headings (geometric sans) | OFL-1.1 |
| Nunito Sans | Variable, Italic Variable | Presentation text (sans) | OFL-1.1 |

The eight Adobe-authored Source Sans 3 / Source Serif 4 faces stay in this central catalog for
provenance and reuse. Native consumers that must not embed Adobe fonts filter those two families,
leaving the 43 permitted Latin faces; DeckCraft already filters them and FigCraft adds the same
family filter in its own build. The catalog and its hashes do not depend on any consumer filter.

### Hebrew presentation and document fonts (`fonts/hebrew-manifest.txt`, opt-in)

These are not part of the default `FONTS` table or `embed` payload. `Noto Sans Hebrew` Regular
remains in the default `fonts/manifest.txt` so every app gets a baseline Hebrew UI fallback out of
the box; apps that want a full Hebrew typographic catalog (UI, documents, presentations, Rashi
script, handwriting and code) opt in by reading `fonts/hebrew-manifest.txt` in their `build.rs` or
via `craft_fonts::HEBREW_FONTS` (`load_hebrew`, feature `embed-hebrew`). The catalog is 29 families,
56 faces, about 12.9 MiB (13,565,048 bytes of font files). Every face is dual-script (`Hebr,Latn`)
and OFL-1.1.

| Family | Styles | For | Licence |
|---|---|---|---|
| Noto Sans Hebrew | Bold | Hebrew UI and document text (pairs with Regular in `fonts/manifest.txt`) | OFL-1.1 |
| Noto Serif Hebrew | Regular, Bold | Hebrew document text (serif) | OFL-1.1 |
| Noto Rashi Hebrew | Regular, Bold | Traditional Rashi semi-cursive commentary script | OFL-1.1 |
| Heebo | Variable | Hebrew and Latin UI and document text (sans) | OFL-1.1 |
| Assistant | Variable | Hebrew and Latin UI and document text (sans) | OFL-1.1 |
| Rubik | Variable, Italic Variable | Hebrew and Latin rounded geometric sans | OFL-1.1 |
| Alef | Regular, Bold | Multi-script screen and document sans | OFL-1.1 |
| Frank Ruhl Libre | Variable | Classic Hebrew book and newspaper serif | OFL-1.1 |
| David Libre | Regular, Medium, Bold | Classic Hebrew book and document serif | OFL-1.1 |
| Miriam Libre | Variable | Classic Hebrew monolinear sans / slab | OFL-1.1 |
| IBM Plex Sans Hebrew | Thin, ExtraLight, Light, Regular, Text, Medium, SemiBold, Bold | Corporate UI and technical document sans | OFL-1.1 |
| Secular One | Regular | Display and heading sans | OFL-1.1 |
| Varela Round | Regular | Rounded UI and presentation sans | OFL-1.1 |
| Suez One | Regular | Display and heading serif | OFL-1.1 |
| Fredoka | Variable | Rounded display and heading sans | OFL-1.1 |
| Karantina | Light, Regular, Bold | Condensed display and poster sans | OFL-1.1 |
| Amatic SC | Regular, Bold | Hand-drawn condensed display | OFL-1.1 |
| Bellefair | Regular | Classic high-contrast display serif | OFL-1.1 |
| Gveret Levin | Regular | Everyday Israeli cursive handwriting | OFL-1.1 |
| Solitreo | Regular | Sephardic cursive script | OFL-1.1 |
| Playpen Sans Hebrew | Variable | Casual handwriting and educational text | OFL-1.1 |
| Handjet | Variable | Pixel and grid display | OFL-1.1 |
| Lunasima | Regular, Bold | Unmodulated document sans | OFL-1.1 |
| Bona Nova | Regular, Bold, Italic | Traditional book serif | OFL-1.1 |
| Bona Nova SC | Regular, Bold, Italic | Small-caps book serif | OFL-1.1 |
| Cardo | Regular, Bold, Italic | Classical and biblical scholarship serif | OFL-1.1 |
| Libertinus Serif | Regular, SemiBold, Bold, Italic, SemiBold Italic, Bold Italic | Academic and technical publishing serif | OFL-1.1 |
| Cascadia Code | Variable | Monospace code with programming ligatures | OFL-1.1 |
| Cascadia Mono | Variable | Monospace code and terminal text | OFL-1.1 |

## How the apps use it

**Fully optional, never required.** The apps don't list craft-fonts in `Cargo.toml` (Cargo would
fetch it for every build). Building with `CRAFT_FONTS_DIR=<path to this checkout>` makes an app
embed the fonts in `fonts/manifest.txt`; without it, the app builds and runs as before. Release
builds check out a pinned commit of this repo and set the option. Apps that explicitly opt in may
also read `fonts/latin-manifest.txt` (the full Latin catalog, filtered by the consumer),
`fonts/hebrew-manifest.txt` (the opt-in Hebrew catalog) and, on native builds only,
`fonts/simplified-chinese-manifest.txt`. None of those is in the default set; only the Chinese
manifest is native-only and must never enter a wasm allowlist. A consumer that may not embed
Adobe-authored faces filters `Source Sans 3` and `Source Serif 4` from the Latin rows.
See [`docs/integration.md`](docs/integration.md).

**Font assets for the Crafting Apps belong here**, not in the app repos: add the file, its licence,
a manifest line and an attribution row, and the tests check it. Put a Latin presentation face in
`fonts/latin-manifest.txt`, a Hebrew presentation/document face in `fonts/hebrew-manifest.txt`, and
a face that only some native consumers should embed in `fonts/simplified-chinese-manifest.txt`;
none of those belongs in the default `fonts/manifest.txt`.

## Develop

```sh
cargo test --workspace                        # all font assertions
cargo test -p craft-fonts --features embed    # the embedded bytes match the files
cargo test -p craft-fonts --features embed-latin   # same, for the opt-in Latin manifest
cargo test -p craft-fonts --features embed-hebrew  # same, for the opt-in Hebrew manifest
```

Code is MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`); each font keeps its own licence.
