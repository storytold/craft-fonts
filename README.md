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
| `fonts/manifest.txt` | Machine-readable list of every font: family, style, file, scripts, licence, SHA-256, pinned source. |
| `ATTRIBUTION.md` | Authors, pinned sources and licences. |
| `crates/craft-fonts` | Rust: the manifest as a static table, finding a checkout (`CRAFT_FONTS_DIR`), loading it, and an `embed` feature that compiles the fonts in. |
| `crates/craft-fonts-tests` | Tests asserted on these fonts: checksums, licences, family names, Japanese coverage and outlines, vertical-text (`vert`/`vrt2`) support. |
| `docs/integration.md` | How an app uses craft-fonts as a fully optional build input. |

### Fonts

| Family | Styles | For | Licence |
|---|---|---|---|
| BIZ UDPGothic | Regular, Bold | Japanese UI text (sans) | OFL-1.1 |
| Shippori Mincho | Regular | Japanese document text (serif) | OFL-1.1 |
| BIZ UDMincho | Regular | Japanese document text (serif) | OFL-1.1 |
| Noto Sans CJK SC | Regular | Simplified Chinese UI and document text (sans) | OFL-1.1 |
| Droid Sans Fallback | Regular | Simplified and Traditional Chinese UI fallback (sans; no Latin glyphs) | Apache-2.0 |
| Noto Sans Arabic | Regular (variable: wdth, wght) | Arabic UI and document text (sans) | OFL-1.1 |

## How the apps use it

**Fully optional, never required.** The apps don't list craft-fonts in `Cargo.toml` (Cargo would
fetch it for every build). Building with `CRAFT_FONTS_DIR=<path to this checkout>` makes an app
embed the fonts in `fonts/manifest.txt`; without it, the app builds and runs as before. Release
builds check out a pinned commit of this repo and set the option. See
[`docs/integration.md`](docs/integration.md).

**Font assets for the Crafting Apps belong here**, not in the app repos: add the file, its licence,
a manifest line and an attribution row, and the tests check it.

## Develop

```sh
cargo test --workspace                       # all font assertions
cargo test -p craft-fonts --features embed   # the embedded bytes match the files
```

Code is MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`); each font keeps its own licence.
