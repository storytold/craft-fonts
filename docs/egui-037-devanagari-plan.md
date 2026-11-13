# Hindi regression fixture and fallback prerequisite

Status (2026-10-24): implemented locally and independently accepted with scope in review v3.
Managed `egui037-fonts-devanagari-20261024-tests-01` passed 4 embedded-library and 30 font tests
with all features and strict all-target/all-feature clippy. `egui037-fonts-devanagari-20261024-final-02`
passed formatting, default workspace tests (3 library plus 30 font tests) and the explicit embedded
configuration (4 library tests). PhotoCraft's required-font UI acceptance is tracked separately;
these results prove asset and glyph coverage, not native Devanagari shaping quality.

This follows local prerequisite `b4ce3ee090ea4b06791e9501ea2f8e6f746d5cd3` and uses its own
resource `egui037-fonts-devanagari-20261024`; the prior resource remains completed.
Plan and implementation reviews are retained in the migration evidence as
`egui037-fonts-devanagari-plan-review-20261024-root-v1.md` and
`egui037-fonts-devanagari-final-review-20261024-root-v3.md`. No publication is authorized.

## Failure evidence

PhotoCraft's Camera Raw language-switch integration test recorded `131 passed; 1 failed; 1 ignored`
because the strict font check cannot render Hindi `फ़ा` (U+092B U+093C U+093E):

```text
No glyph for "फ़ा" (U+092B) in Proportional.
Installed fonts: ["Inter", ..., "craft-font-0", … "craft-font-11"]
test result: FAILED. 131 passed; 1 failed; 1 ignored
```

The failing job is `egui037-photocraft-candidate-20261024-integration-tests-19`; its status JSON is
`/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/egui037-photocraft-candidate-20261024-integration-tests-19-status.json`.
The test exercises real translated labels and preserves document/filter state. Keep it strict,
keep every registered `i18n::Lang::all()` language (currently 22; the legacy test name
`all_seven_languages…` is misleading). PhotoCraft forbids vendored font files, so the font belongs
in this central repository and is consumed through `CRAFT_FONTS_DIR`.

## Font choice and provenance

Use one unmodified static Regular font from the official Noto Sans Devanagari 2.007 release,
target commit `e123d230c160ebe949d731cc19017cdb354180d1`:
[release](https://github.com/notofonts/devanagari/releases/tag/NotoSansDevanagari-v2.007). That
commit is recorded in the acquisition metadata; no local git object independently proves the
tag-to-commit mapping, so it is recorded provenance, not an independently rechecked fact.

- Release ZIP `NotoSansDevanagari-v2.007.zip`: **18,449,254 bytes**, SHA-256
  `820c7da45b1e63562cb41c0a8cac5d9a4202312043a3a040ed1325857ef469b1`. `acquisition.json` records
  this as `zip_sha256`; it did not capture a GitHub release-API asset digest, so this plan does not
  claim an independently rechecked GitHub digest. The local ZIP is retained as downloaded.
- Selected member `NotoSansDevanagari/full/ttf/NotoSansDevanagari-Regular.ttf`: **277,680 bytes**,
  SHA-256 `573e525d9f4923d34645fe752475419f45f8dec5a8e43f3e83c5c5f43d1f2ff9`. The extracted file is
  byte-identical to the ZIP member.
- Archive-root `OFL.txt` (not adjacent to the `full/ttf` member), copied next to the installed
  font: **4,386 bytes**, SHA-256
  `a216f6f8d85c7228093e0ee5e258d9d377e6671f68acb4db1930b29583d0f331`. It is byte-identical to
  the archive member, contains “SIL OPEN FONT LICENSE Version 1.1” and declares no Reserved Font
  Name.
- Font metadata: sfnt `0x00010000`, 17 tables, 1,117 glyphs, no
  `fvar`/`avar`/`STAT`/`gvar`/`HVAR`/`MVAR`; OS/2 weight **400**, width **5**, `fsSelection=0x01c0`
  (`REGULAR|USE_TYPO_METRICS|WWS`); names `Noto Sans Devanagari`, `Regular`; version string contains
  `Version 2.007`.
- Coverage: all 128 codepoints `U+0900..U+097F` are mapped and every mapped glyph draws
  recursively. The failing sequence `फ़ा` maps to drawn glyphs: `फ` U+092B → gid 77, `़` U+093C →
  gid 104, `ा` U+093E → gid 31. The Hindi sample
  `हिन्दी भाषा क्षत्रिय श्री फ़ाइल फ़िल्टर` has no missing or undrawn glyphs.

Retained ZIP and selected members are in the migration evidence's
`font-acquisition/noto-sans-devanagari/`. Its per-directory `acquisition.json` is the authoritative
record for this font; the aggregate top-level `font-acquisition/acquisition.json` does not list
Devanagari.

## Repository addition

Copy only the selected static Regular TTF and the archive-root OFL into
`fonts/noto-sans-devanagari/`. Append one `Deva` Regular row after the Hebrew row in the default
manifest, preserving every existing row and its order:

```text
Noto Sans Devanagari | Regular | fonts/noto-sans-devanagari/NotoSansDevanagari-Regular.ttf | Deva | OFL-1.1 | fonts/noto-sans-devanagari/OFL.txt | 573e525d9f4923d34645fe752475419f45f8dec5a8e43f3e83c5c5f43d1f2ff9 | https://github.com/notofonts/devanagari/releases/tag/NotoSansDevanagari-v2.007
```

`Deva` is the correct scripts field; the font's family is exactly `Noto Sans Devanagari`, matching
PhotoCraft's `DEVANAGARI_FAMILIES`. Appending after Hebrew preserves existing consumer
`craft-font-N` indices; the new face becomes `craft-font-12` in PhotoCraft's eager registration.

This follows the existing Arabic/Thai/Georgian/Hebrew script fallbacks: consumers that embed the
default set gain 277,680 font bytes. Explicit app web allowlists stay explicit. Do not broaden
another app's web allowlist implicitly. Keep the optional Chinese manifest independent: the
Devanagari face must not appear in `fonts/simplified-chinese-manifest.txt`, `FONTS`, or any wasm
allowlist by accident.

### Payload accounting and documentation

- Before this row: **12 rows**, **45,600,892 bytes** (43.49 MiB).
- After this row: **13 rows**, **45,878,572 bytes** (43.75 MiB; 45.88 MB).
- Added: **+277,680 font bytes**; **+4,386 licence bytes**, not embedded.
- Update `docs/integration.md` from “roughly 43 MiB (45.5 MB)” to “roughly 43.8 MiB (45.9 MB)”.
- Update the `crates/craft-fonts/Cargo.toml` embedded-size comment to “about 43.8 MiB”.
- Update the current total in `docs/egui-037-deck-font-gaps-plan.md` (formerly 45,600,892) and
  mark the older total in `docs/egui-037-font-coverage-plan.md` as historical for the Hebrew
  follow-up, noting the new 45,878,572-byte total.
- Add matching `ATTRIBUTION.md` and README rows: family `Noto Sans Devanagari`, style `Regular`,
  script `Deva`, author The Noto Project Authors, OFL-1.1, licence destination
  `fonts/noto-sans-devanagari/OFL.txt`.

## Central test requirements

Add these assertions to `crates/craft-fonts-tests/tests/fonts.rs`, with a `devanagari_` test-name
prefix so the managed filter can select them:

- Exact row: family `Noto Sans Devanagari`, style `Regular`, file
  `fonts/noto-sans-devanagari/NotoSansDevanagari-Regular.ttf`, scripts `["Deva"]`, licence
  `OFL-1.1`, licence path `fonts/noto-sans-devanagari/OFL.txt`, font SHA-256
  `573e525d…1f2ff9` and source URL
  `https://github.com/notofonts/devanagari/releases/tag/NotoSansDevanagari-v2.007`;
  `for_script("Deva").count() == 1`.
- File size exactly `277_680` and licence size exactly `4_386`; licence SHA-256 exactly
  `a216f6f8…83d0f331` (the existing generic licence test checks only OFL text, not the licence
  hash).
- Single static face: sfnt version `0x00010000`; no `fvar`/`avar`/`STAT`; OS/2 weight 400, width 5,
  `fsSelection` contains `REGULAR` and not `ITALIC`/`BOLD`/`OBLIQUE`; skrifa attributes
  `Weight::NORMAL`/`Style::Normal`; version string contains `Version 2.007`.
- Outlines: `assert_face_glyph_outline` for `फ`, `़`, `ा`, for every character of `फ़ा`, and for
  the Hindi sample `हिन्दी भाषा क्षत्रिय श्री फ़ाइल फ़िल्टर` (whitespace maps to a space glyph, which
  draws no outline, so skip whitespace for the outline assertions). Optionally and additionally
  assert all `U+0900..=U+097F` map to drawn outlines; this is true for the pinned file.
- State explicitly that these are glyph-coverage assertions, not proof of Devanagari
  shaping/conjunct positioning; the PhotoCraft Camera Raw test exercises the real shaping path.
- Assert `Noto Sans Devanagari` is absent from `fonts/simplified-chinese-manifest.txt`, keeping the
  optional Chinese manifest independent.

## PhotoCraft production-wiring check

Add a small assertion to the existing Camera Raw language-switch test, not a new broad test suite:

- Assert `photocraft_text::craft_fonts::CRAFT_FONTS` contains exactly one
  `Noto Sans Devanagari` / `Regular` row with `Deva` scripts and non-empty bytes. This exact-row
  assertion prevents a system Devanagari font from masking a missing central row.
- Assert the production UI stack has glyphs for `फ़ा` (for example with the context's font
  collection `has_glyph` check) before the language loop.
- Keep the existing `MissingGlyphPolicy::Panic`; add no `Tofu`, `allow_missing_glyphs`, synthetic
  Devanagari fixture or test-only font registration.
- Run with `CRAFT_FONTS_DIR` pointing at the central checkout and `CRAFT_FONTS_REQUIRED=1`, so the
  test cannot pass by silently omitting the central input.

## Consumer pins

PhotoCraft currently pins `craft-fonts-ref: abb83316d96aa59c1cf64784289e378fe9fa5695` in `ci.yml`,
`release.yml` and `freebsd.yml`. Do not change those to an unpushed local SHA. After the reviewed
central commit exists and is published with explicit approval, update all three pins to the
published full SHA. The exact pin remains a publication gate; do not guess a path-only lock or
dependency pin.

## No-font and system-provider separation

State explicitly that the acceptance test runs only in the required central-font configuration.
No-font and system-provider tests remain separate and must not be weakened, changed to tofu
comparisons, or made to depend on a host-installed Devanagari font. The central exact-row check is
what proves the bundled face is present; the installed production set is what the Camera Raw test
checks.

## Proposed managed verification filters

Run through the existing `task.py` resource workflow; do not run these directly:

```sh
# Central repository
cargo test -p craft-fonts-tests --test fonts devanagari_ --locked
cargo test --workspace --locked
cargo test -p craft-fonts --features embed --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo fmt --all --check

# PhotoCraft, with the central checkout and required input
CRAFT_FONTS_DIR=<central-checkout> CRAFT_FONTS_REQUIRED=1 \
  cargo test -p photocraft-ui-egui --test it \
  camera_raw_localization::all_seven_languages_render_camera_raw_and_switch_without_resetting_view_or_filter -- --exact
CRAFT_FONTS_DIR=<central-checkout> CRAFT_FONTS_REQUIRED=1 \
  cargo test -p photocraft-ui-egui --test it
CRAFT_FONTS_DIR=<central-checkout> CRAFT_FONTS_REQUIRED=1 \
  cargo clippy -p photocraft-ui-egui --all-targets -- -D warnings
```

## Limitations

- The central default manifest previously had no exact-row count assertion beyond
  `entries.len() == FONTS.len()`, so the explicit `for_script("Deva").count() == 1` and exact-row
  tests are necessary to prevent silent omission.
- Font-only outline checks do not prove Devanagari shaping, conjunct formation or mark
  positioning. The Camera Raw test must exercise the real shaping path.
- The tag commit and a GitHub release-API digest could not be independently rechecked offline; the
  local ZIP/member/OFL hashes and metadata were rechecked directly.
- No Cargo, rustc, test, fmt, clippy, metadata-inspection or VM/resource commands were run for
  this implementation; the filters above are for the parent to run through `task.py`.
