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
| `fonts/simplified-chinese-manifest.txt` | Opt-in native-only Simplified Chinese row, same eight-field format. Not in `FONTS` or the embed payload. |
| `ATTRIBUTION.md` | Authors, pinned sources and licences. |
| `crates/craft-fonts` | Rust: the manifest as a static table, finding a checkout (`CRAFT_FONTS_DIR`), loading it, and an `embed` feature that compiles the fonts in. |
| `crates/craft-fonts-tests` | Tests asserted on these fonts: checksums, licences, family/style names, Japanese/Korean/Hebrew/Devanagari/symbol coverage and outlines, static-face metadata and vertical-text (`vert`/`vrt2`) support. |
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

## How the apps use it

**Fully optional, never required.** The apps don't list craft-fonts in `Cargo.toml` (Cargo would
fetch it for every build). Building with `CRAFT_FONTS_DIR=<path to this checkout>` makes an app
embed the fonts in `fonts/manifest.txt`; without it, the app builds and runs as before. Release
builds check out a pinned commit of this repo and set the option. Apps that explicitly opt in may
also read `fonts/simplified-chinese-manifest.txt` on native builds only; it is never part of the
default set or a wasm allowlist. See [`docs/integration.md`](docs/integration.md).

**Font assets for the Crafting Apps belong here**, not in the app repos: add the file, its licence,
a manifest line and an attribution row, and the tests check it. Put a face that only some native
consumers should embed in the opt-in manifest described in `docs/integration.md`, not in the
default `fonts/manifest.txt`.

## Develop

```sh
cargo test --workspace                       # all font assertions
cargo test -p craft-fonts --features embed   # the embedded bytes match the files
```

Code is MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`); each font keeps its own licence.
