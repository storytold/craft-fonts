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
| `fonts/simplified-chinese-manifest.txt` | Opt-in native-only Simplified Chinese row, same eight-field format. Not in `FONTS` or the embed payload. |
| `ATTRIBUTION.md` | Authors, pinned sources and licences. |
| `crates/craft-fonts` | Rust: the default and opt-in Latin manifests as static tables, finding a checkout (`CRAFT_FONTS_DIR`), loading them, and `embed` / `embed-latin` features that compile the fonts in. |
| `crates/craft-fonts-tests` | Tests asserted on these fonts: checksums, licences, family/style names, Inter/Latin provenance and faces, Japanese/Korean/Hebrew/Devanagari/symbol coverage and outlines, codepoint coverage of the other Indian scripts, static-face metadata and vertical-text (`vert`/`vrt2`) support. |
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
| Noto Sans Bengali | Regular (variable: wdth, wght) | Bengali UI and document text (sans) | OFL-1.1 |
| Noto Sans Gurmukhi | Regular (variable: wdth, wght) | Punjabi UI and document text (sans) | OFL-1.1 |
| Noto Sans Gujarati | Regular (variable: wdth, wght) | Gujarati UI and document text (sans) | OFL-1.1 |
| Noto Sans Oriya | Regular (variable: wdth, wght) | Odia UI and document text (sans) | OFL-1.1 |
| Noto Sans Tamil | Regular (variable: wdth, wght) | Tamil UI and document text (sans) | OFL-1.1 |
| Noto Sans Telugu | Regular (variable: wdth, wght) | Telugu UI and document text (sans) | OFL-1.1 |
| Noto Sans Kannada | Regular (variable: wdth, wght) | Kannada UI and document text (sans) | OFL-1.1 |
| Noto Sans Malayalam | Regular (variable: wdth, wght) | Malayalam UI and document text (sans) | OFL-1.1 |
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

## How the apps use it

**Fully optional, never required.** The apps don't list craft-fonts in `Cargo.toml` (Cargo would
fetch it for every build). Building with `CRAFT_FONTS_DIR=<path to this checkout>` makes an app
embed the fonts in `fonts/manifest.txt`; without it, the app builds and runs as before. Release
builds check out a pinned commit of this repo and set the option. Apps that explicitly opt in may
also read `fonts/latin-manifest.txt` (the full Latin catalog, filtered by the consumer) and, on
native builds only, `fonts/simplified-chinese-manifest.txt`. Neither is in the default set; only
the Chinese manifest is native-only and must never enter a wasm allowlist. A consumer that may
not embed Adobe-authored faces filters `Source Sans 3` and `Source Serif 4` from the Latin rows.
See [`docs/integration.md`](docs/integration.md).

**Font assets for the Crafting Apps belong here**, not in the app repos: add the file, its licence,
a manifest line and an attribution row, and the tests check it. Put a Latin presentation face in
`fonts/latin-manifest.txt` and a face that only some native consumers should embed in
`fonts/simplified-chinese-manifest.txt`; neither belongs in the default `fonts/manifest.txt`.

## Develop

```sh
cargo test --workspace                       # all font assertions
cargo test -p craft-fonts --features embed   # the embedded bytes match the files
cargo test -p craft-fonts --features embed-latin  # same, for the opt-in Latin manifest
```

Code is MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`); each font keeps its own licence.
