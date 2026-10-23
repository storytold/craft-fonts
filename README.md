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
| `fonts/ui/` | CJK interface faces in TrueType outlines, which apps cut to their own catalogs at build time (see below). |
| `fonts/manifest.txt` | Machine-readable list of every font: family, style, file, scripts, licence, SHA-256, pinned source, and `ui` for the interface faces. |
| `tools/` | `ui-sources`, which regenerates `fonts/ui/` (Python, run with `uv`). |
| `ATTRIBUTION.md` | Authors, pinned sources and licences. |
| `crates/craft-fonts` | Rust: the manifest as a static table, finding a checkout (`CRAFT_FONTS_DIR`), loading it, and an `embed` feature that compiles the fonts in. |
| `crates/craft-fonts-tests` | Tests asserted on these fonts: checksums, licences, family names, Japanese coverage and outlines, vertical-text (`vert`/`vrt2`) support, and that each interface face is TrueType, covers its script's basic characters and keeps `vert` and `locl`. |
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
| Noto Sans Armenian | Regular (variable: wdth, wght) | Armenian UI and document text (sans) | OFL-1.1 |
| Noto Sans Ethiopic | Regular (variable: wdth, wght) | Ethiopic (Amharic, Tigrinya) UI and document text (sans) | OFL-1.1 |
| Noto Sans | Regular (variable: wdth, wght) | Latin, Cyrillic and Greek fallback (Vietnamese, Central European, Turkish, Ukrainian, Serbian, Kazakh, Greek) and Devanagari (sans) | OFL-1.1 |
| Noto Color Emoji | Regular (COLRv1 colour vector glyphs) | Colour emoji | OFL-1.1 |
| Noto Emoji | Regular (variable: wght) | Monochrome emoji, the last emoji fallback | OFL-1.1 |
| Noto Sans Symbols | Regular (variable: wght) | Key and UI symbols (⌃ ⌄ ⎋, arrows), with Symbols 2 and Math | OFL-1.1 |
| Noto Sans Symbols 2 | Regular | Key and UI symbols (⇧ ⌥ ⌘ ⌫ ⌦ ⏏ …) | OFL-1.1 |
| Noto Sans Math | Regular | Math symbols; the return, tab and page key arrows (↩ ↵ ⇥ ⇤ ⇞ ⇟) | OFL-1.1 |

### Interface faces

Each CJK script has one interface face, marked `ui` in the manifest. Apps embed it for interface
text through craft-fonts-build, which cuts it to the characters of the app's translation catalogs
at build time: about 4 to 5 MB per app for all five. Text outside the catalogs (what users type,
file names, font names) falls back to the system's CJK fonts. Latin comes from Noto Sans.

| Family | For | File | Source |
|---|---|---|---|
| Noto Sans SC UI | Simplified Chinese (`Hans`) | `fonts/ui/NotoSansSCUI-Regular.ttf`, 10.2 MB | Noto Sans SC Regular (`SubsetOTF`) |
| Noto Sans TC UI | Traditional Chinese, Taiwan forms (`Hant`) | `fonts/ui/NotoSansTCUI-Regular.ttf`, 6.9 MB | Noto Sans TC Regular (`SubsetOTF`) |
| Noto Sans HK UI | Traditional Chinese, Hong Kong forms (`Hant-HK`) | `fonts/ui/NotoSansHKUI-Regular.ttf`, 6.9 MB | Noto Sans HK Regular (`SubsetOTF`) |
| Noto Sans KR UI | Korean (`Kore`) | `fonts/ui/NotoSansKRUI-Regular.ttf`, 5.9 MB | Noto Sans KR Regular (`SubsetOTF`) |
| BIZ UDPGothic | Japanese (`Jpan`) | `fonts/biz-ud-pgothic/BIZUDPGothic-Regular.ttf` | the Regular above, unchanged |

The same Han character is drawn differently in mainland China, Taiwan, Hong Kong, Japan and Korea,
so each region has its own face; pick it from the UI language (zh-CN/SG: SC, zh-TW: TC, zh-HK/MO:
HK, ja: BIZ UDPGothic, ko: KR).

The four Noto faces are the noto-cjk regional Regulars with their CFF outlines converted to
TrueType quadratics (cu2qu, within 1 font unit), because the build-time subsetter cuts TrueType
faces correctly and CFF faces not. They keep every glyph and layout feature (vertical forms,
`locl`) and are renamed "<family> UI". To regenerate them and their manifest lines:

```sh
tools/ui-sources   # downloads the pinned sources to ~/.cache/craft-fonts-src; same bytes on every run
```

The interface faces are for interface text only. Full CJK faces for document text are a separate
matter; `Noto Sans CJK SC`, `Shippori Mincho` and `BIZ UDMincho` above are the ones shipped so far.

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
