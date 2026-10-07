craft-fonts
===========

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
| `fonts/manifest.txt` | Machine-readable list of the fonts every app embeds: family, style, file, scripts, licence, SHA-256, pinned source. |
| `fonts/latin-manifest.txt` | Opt-in second manifest, same format: Latin presentation fonts (for SlideCraft and other document apps). Apps that read only `manifest.txt` don't get them. |
| `ATTRIBUTION.md` | Authors, pinned sources and licences. |
| `crates/craft-fonts` | Rust: the manifest as a static table, finding a checkout (`CRAFT_FONTS_DIR`), loading it, and an `embed` feature that compiles the fonts in. |
| `crates/craft-fonts-tests` | Tests asserted on these fonts (both manifests): checksums, licences, family names, Japanese coverage and outlines, vertical-text (`vert`/`vrt2`) support, Latin coverage and outlines, variable vs static styles. |
| `docs/integration.md` | How an app uses craft-fonts as a fully optional build input. |

### Fonts (`fonts/manifest.txt`)

| Family | Styles | For | Licence |
|---|---|---|---|
| BIZ UDPGothic | Regular, Bold | Japanese UI text (sans) | OFL-1.1 |
| Shippori Mincho | Regular | Japanese document text (serif) | OFL-1.1 |
| BIZ UDMincho | Regular | Japanese document text (serif) | OFL-1.1 |
| Noto Sans CJK SC | Regular | Simplified Chinese UI and document text (sans) | OFL-1.1 |
| Noto Sans Arabic | Regular (variable: wdth, wght) | Arabic UI and document text (sans) | OFL-1.1 |

### Latin presentation fonts (`fonts/latin-manifest.txt`, opt-in)

About 19.5 MB in all. "Variable" means one variable-font file (weight axis, some also width or
optical size) per upright/italic.

| Family | Styles | For | Licence |
|---|---|---|---|
| Inter | Regular, Italic, SemiBold, Bold | UI and slide text (sans) | OFL-1.1 |
| Carlito | Regular, Bold, Italic, Bold Italic | Metric-compatible with Calibri | OFL-1.1 |
| Caladea | Regular, Bold, Italic, Bold Italic | Metric-compatible with Cambria | OFL-1.1 |
| Liberation Sans | Regular, Bold, Italic, Bold Italic | Metric-compatible with Arial | OFL-1.1 |
| Liberation Serif | Regular, Bold, Italic, Bold Italic | Metric-compatible with Times New Roman | OFL-1.1 |
| Liberation Mono | Regular, Bold, Italic, Bold Italic | Metric-compatible with Courier New | OFL-1.1 |
| Source Sans 3 | Regular, Italic, Semibold, Bold | Presentation text (sans) | OFL-1.1 |
| Source Serif 4 | Regular, Italic, Semibold, Bold | Presentation text (serif) | OFL-1.1 |
| Montserrat | Variable, Italic Variable | Headings (geometric sans) | OFL-1.1 |
| Lato | Regular, Bold, Italic | Presentation text (sans) | OFL-1.1 |
| Open Sans | Variable, Italic Variable | Presentation text (sans) | OFL-1.1 |
| Roboto | Variable, Italic Variable | Presentation text (sans) | OFL-1.1 |
| Merriweather | Regular, Bold, Italic | Presentation text (serif) | OFL-1.1 |
| Playfair Display | Variable, Italic Variable | Headings (display serif) | OFL-1.1 |
| Poppins | Regular, Bold, Italic | Headings (geometric sans) | OFL-1.1 |
| Nunito Sans | Variable, Italic Variable | Presentation text (sans) | OFL-1.1 |

## How the apps use it

**Fully optional, never required.** The apps don't list craft-fonts in `Cargo.toml` (Cargo would
fetch it for every build). Building with `CRAFT_FONTS_DIR=<path to this checkout>` makes an app
embed the fonts in `fonts/manifest.txt` (and, for apps that opt in, `fonts/latin-manifest.txt`); without it, the app builds and runs as before. Release
builds check out a pinned commit of this repo and set the option. See
[`docs/integration.md`](docs/integration.md).

**Font assets for the Crafting Apps belong here**, not in the app repos: add the file, its licence,
a manifest line and an attribution row, and the tests check it.

## Develop

```sh
cargo test --workspace                       # all font assertions
cargo test -p craft-fonts --features embed   # the embedded bytes match the files
cargo test -p craft-fonts --features embed-latin   # same, for the Latin manifest
```

Code is MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`); each font keeps its own licence.
