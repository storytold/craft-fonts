# egui 0.37 DeckCraft font gaps: Chinese UI fallback and table-style fixture

Status: central font assets, the opt-in manifest, and DeckCraft's native build/UI/table-fixture
changes are implemented locally. Central managed job `egui037-fonts-candidate-20261023-chinese-05`
passed 4 embed tests, 25 font tests and strict all-features clippy. Deck's fresh review found and
repaired tofu-permissive assertions and a Chinese sample that was not visible at startup;
managed `egui037-deckcraft-candidate-20261023-font-input-14` passed the two targeted font
regressions but failed its full UI suite on seven other tests. Later jobs
`egui037-deckcraft-candidate-20261024-upstream-tests-24` and `-25` passed 307 UI
tests with 4 ignored (serial and parallel), and `-upstream-lints-26` passed strict
linting. The earlier parallel SIGSEGV remains an open diagnostic, not a fixed issue.
Central `egui037-fonts-candidate-20261024-final-06` also passed default/embed tests,
formatting and strict linting after the CI-only upstream fast-forward to `fff559f`.
Native GUI, web-size and performance qualification remain open. The planning observations below
are retained as history; the acquisition and implementation sections supersede the speculative GB choice.

## Observed failures

### 1. Chinese interface text

The managed candidate run
`/home/codex/work/ssd2/task-evidence/egui037-deckcraft-candidate-20261023/task-jobs/egui037-deckcraft-candidate-20261023-ime-tests-11/output.log`
reports:

```text
No glyph for "简" (U+7B80) in Proportional.
Installed fonts: ["craft-1-BIZ UDPGothic-Regular", "craft-4-Noto Sans Arabic-Regular",
"craft-7-NanumGothic-Regular", "craft-8-Noto Sans Symbols-Regular",
"craft-9-Noto Sans Symbols 2-Regular", "craft-10-Noto Sans Hebrew-Regular",
"Ubuntu-Light", "egui-icons", "NotoEmoji-Regular", "emoji-icon-font", "Hack"]
```

This is the DeckCraft UI test
`crates/ui-egui/src/ime_tests.rs::cjk_text_in_the_interface_is_drawn_with_real_glyphs` (lines
182-220). The test's Chinese sample is `简体中文演示文稿` (line 189) and the missing character is
`简` U+7B80. Noto Sans CJK is present in the central repository but DeckCraft's
`crates/fonts/build.rs` excludes `Noto Sans CJK SC` under its asset policy, so it is correctly
absent from the candidate's installed fonts.

The existing `system_has(text)` skip at lines 200-204 is not a safe fix. The test builds the app
with `harness(s)` before it reaches that loop, and the app paints a UI surface containing the
Chinese section name. Under egui 0.37's strict missing-glyph policy the paint panics before the
skip can run. Even if the skip were reachable, skipping Chinese would violate the requirement that
the real UI regression remain covered. Do not remove the Chinese assertion or compare against
tofu.

The current central plan says:

- `docs/egui-037-font-coverage-plan.md` lines 353-358: Chinese stays system-only in GridCraft
  while Noto CJK is barred; a permitted Chinese face may be added later with the same provenance,
  cmap/outline and measurement evidence.
- `docs/integration.md`, web allowlist section: the web allowlist is fixed and excludes Noto CJK. A new
  native Chinese face must not be added to that allowlist implicitly.

### 2. Table style Regular/Bold fixture

The same managed run reports:

```text
tests::tables_with_a_file_defined_style_edit_and_resize
panicked at crates/engine/src/tests.rs:3070:
the header is laid out in the file style's (regular) weight:
(177.70600000000002, 114.4) vs (177.70600000000002, 114.4)
```

The test uses `Session::with_new()` (lines 5-7), whose default theme names `Inter`. It takes the
built-in header position with the built-in style's bold weight, then applies a file style with
`bold: Some(false)` and expects a different caret x. In the candidate checkout there is no
`fonts/latin-manifest.txt` and no `Inter` face in the central manifest. `FontDb::face` therefore
falls through its family/substitute list to the first loaded face for both Regular and Bold
(`crates/fonts/src/fontdb.rs` lines 1006-1025). Both requests resolve to the same static face, so
the two positions are identical.

This is not evidence that the table-style implementation is wrong. The fixture is asking for two
metrics from a family that is not present. Adding Inter solely to satisfy this test would solve
the symptom but would also add a new, unreviewed Latin asset and change the default document font
for every candidate. The smaller, honest fixture fix is to make the test use a family with a real
Regular and Bold face that is already part of the candidate.

## Decision summary

1. **Chinese UI:** add one native-only, permitted Simplified Chinese face, supplied by
   craft-fonts. Preferred candidate is **LXGW WenKai GB Regular** because it targets Simplified
   Chinese, is derived from Fontworks Klee One under OFL-1.1, and should be smaller than the full
   LXGW WenKai family. Keep it out of every wasm allowlist. Do not use Noto CJK.
2. **Table fixture:** use the existing `BIZ UDPGothic` Regular/Bold pair in the test, not a new
   Inter/Latin asset. A read-only hmtx check of the existing bytes gives 12,767 font units for
   `Hello world` in Regular and 13,237 in Bold at 2,048 upem (112.21 pt versus 116.34 pt at 18 pt;
   a 4.13 pt difference). That is sufficient to make the assertion meaningful. The test may
   choose another installed family with a real Regular/Bold pair only when the candidate is
   absent; under `CRAFT_FONTS_REQUIRED=1` a missing pair must fail, not skip.
3. **System-only Chinese with an owned test fixture:** retain as a fallback only if the LXGW
   candidate cannot pass provenance, licence/RFN or byte gates. It is not the preferred answer:
   it proves the fallback machinery with a fixture but leaves Linux users without a shipped
   Chinese face. If used, the fixture must be contributor-original or openly licensed, attributed,
   and the production limitation must be stated explicitly. It must never be presented as
   production Chinese coverage.

No Chinese font addition or production fallback change should land until the provenance gate in this document is complete. The table fixture fix can be reviewed independently because it uses existing BIZ bytes and does not add an asset.

## Chinese candidate research

### Policy fit

DeckCraft's asset policy (repository `AGENTS.md` §1.1-1.4) bars Microsoft, Adobe, Avid and
Autodesk visual assets, including Adobe's Source Han and Noto CJK. It permits OFL-1.1 assets from
their original source with attribution. Fontworks Klee One is not one of the barred vendors, and
LXGW WenKai is a derivative of Klee One under OFL-1.1, so the family is a policy candidate. This is
a licence/policy finding, not a provenance or quality pass.

`LXGW` is the project abbreviation for the Pinyin `Luo Xia Gu Wu`; the Debian package description
also identifies the upstream homepage and the Klee One derivation. Record the derivative author
as **LXGW / Luo Xia Gu Wu**, and record the original design/copyright holder as **Fontworks Inc.
(Klee One)**. Do not attribute the derivative to Adobe or Google.

### Release and provenance evidence available locally

The server has no working DNS for `github.com`, `raw.githubusercontent.com` or
`api.github.com`; attempts to read the two official repositories failed before any content was
fetched. This document therefore does **not** claim an upstream release tag, commit or asset hash
that it has not inspected.

The strongest local evidence is Debian trixie's package metadata for the base family:

- Debian source package: `fonts-lxgw-wenkai` version `1.511+repack-1`.
- Upstream homepage: `https://github.com/lxgw/LxgwWenKai`.
- Description: an open-source Chinese font derived from Fontworks' Klee One; `LXGW` is the
  abbreviation for `Luo Xia Gu Wu`.
- Package contains six faces: `LXGWWenKai-{Light,Regular,Bold}.ttf` and
  `LXGWWenKaiMono-{Light,Regular,Bold}.ttf`.
- Binary package size: 60,097,836 bytes compressed; installed size: 116,871 KiB (119,675,904
  bytes) for all six files. The average is 19,945,984 bytes (19.02 MiB) per face. These are Debian
  package figures, not a measured upstream Regular file.

The Debian version is **evidence of a release number, not an acceptable pin for the font bytes**:
the archive is a Debian `repack`, and the exact upstream tag, commit, release archive SHA-256 and
member hash still need to be recorded. `LxgwWenkaiGB` has no local package metadata at all, so its
release must be verified independently before it can be selected.

### Candidate table

| Field | Base candidate | GB candidate | Required evidence before adoption |
|---|---|---|---|
| Project | `lxgw/LxgwWenKai` | `lxgw/LxgwWenkaiGB` | Official release page and immutable tag/commit |
| Candidate release | `1.511` (Debian-observed; upstream tag/commit unverified) | not locally verified | Release tag, commit, release date, archive URL and SHA-256 |
| Expected file | `LXGWWenKai-Regular.ttf` | `LXGWWenKaiGB-Regular.ttf` (exact name to verify) | Byte-identical upstream member; no rename/subset unless reviewed |
| Derivative author | LXGW / Luo Xia Gu Wu | LXGW / Luo Xia Gu Wu | `name` table and upstream licence header |
| Original face | Fontworks Klee One | Fontworks Klee One | Upstream OFL and copyright notice |
| Licence | OFL-1.1 | OFL-1.1 expected; verify | Byte-identical `OFL.txt`, hash and destination |
| RFN | Not proven locally | Not proven locally | Read the full OFL header and record every Reserved Font Name; ship unmodified bytes |
| Script | `Hans` (plus exact inherited coverage) | `Hans` expected | Actual cmap/script data, not a guess from the family name |
| Coverage | Full family expected to be broader | GB-oriented; verify exact set | Outline checks for `简体中文演示文稿` and `简` U+7B80, plus documented limits |
| Native-only | Yes | Yes | No wasm allowlist entry |

**RFN handling:** the plan must not state "no Reserved Font Name" until the upstream OFL text has
been read. Regardless of the result, ship the release member unmodified and retain the upstream
family name. Do not subset, rename or create a "DeckCraft Chinese" face. If the upstream licence
does contain an RFN, the unmodified redistribution is still the safe path; any later modified
face needs a separate OFL review.

### Coverage acceptance

The bundled face is accepted for the current regression only when it draws every character of
`简体中文演示文稿` and the exact failing codepoint `简` U+7B80 with non-empty outlines. Do not
infer coverage from `scripts.contains("Hans")`. Also record which GB repertoire the chosen release
claims (for example GB 2312/GBK/GB18030) and do not advertise broader coverage without a cmap
check.

For mixed UI text, assert both proportional and bold interface families, as the current test does.
Fallback shaping is not the hard part here: these are ordinary Han characters with no combining
marks. Still, a real UI screenshot and an edit/reopen workflow are required, not just a glyph
lookup.

## Native byte budget and WASM exclusion

### Current bytes

The central `fonts/manifest.txt` currently names 45,878,572 font bytes (43.75 MiB; 13 rows,
after the Devanagari follow-up added 277,680 bytes). The 16,437,364-byte `Noto Sans CJK SC`
file is still in the repository for other consumers but is excluded from DeckCraft native
builds by `crates/fonts/build.rs`.

DeckCraft's web branch embeds only:

- `BIZ UDPGothic` Regular
- `Inter` Regular, Bold or Variable

The wasm branch skips every other manifest row (`crates/fonts/build.rs` lines 49-62). A new
Chinese row in a native-only manifest therefore has a 0-byte wasm delta by construction. Do not
edit that allowlist and do not add the Chinese face to GridCraft's `WEB_FONTS`.

### Budget gates

Use these as hard gates until exact upstream bytes are available:

| Gate | Limit | Measurement / action |
|---|---|---|
| One font file | <= 24 MiB | Upstream file size; abort if larger |
| Licence file | <= 8 KiB | Byte hash and destination under `fonts/<family>/` |
| Native executable/bundle growth | <= 26 MiB | Release build before/after, recorded per app |
| Additional runtime font storage | <= 50 MiB peak | Startup and first Chinese paint; egui keeps a copy of each face |
| Wasm raw and compressed delta | 0 bytes | Build with the unchanged allowlist and assert no LXGW family in `CRAFT_FONTS` |
| Number of added faces | exactly 1 | One static Regular row, no Light/Bold/Mono sibling files |

The 24 MiB font ceiling is intentionally above the 19.02 MiB Debian-package average. It gives room
for a single full base Regular while rejecting an accidental six-face package. If the verified GB
Regular is smaller, record its actual size; do not replace the measurement with an estimate.

`crates/ui-egui/src/uifonts.rs` already accepts non-colour faces up to 128 MiB and an aggregate of
384 MiB. One 20-24 MiB face is within that policy, but the user-visible startup and memory cost
must still be measured rather than inferred from that constant.

### Packaging choice

Keep the existing native consumers unchanged unless they explicitly opt in. Add the Chinese face
under a separate opt-in manifest, for example `fonts/simplified-chinese-manifest.txt`, and have
DeckCraft's native build read it after the Latin and general manifests. This avoids silently
adding ~20 MiB to GridCraft and every other native consumer, and preserves GridCraft's documented
system-only Chinese policy. If the maintainers instead decide that all native consumers want the
face, a single row in `fonts/manifest.txt` is simpler, but then every consumer's native growth
must be reviewed and documented.

The wasm build must ignore the opt-in manifest entirely. Add a build-time or test assertion that
the Chinese family/file is absent from the wasm `CRAFT_FONTS` slice even when
`CRAFT_FONTS_REQUIRED=1`.

## Table-style test fixture

### Minimal fix

Do not add Inter and do not change production default fonts for this test. Before taking
`built_in`, set the first-row text run's font to a family whose Regular and Bold faces are both
present. Prefer `BIZ UDPGothic`, because it is already in the central manifest and is required by
the current candidate.

The read-only hmtx check above proves the existing BIZ files are not metric clones:

- `BIZUDPGothic-Regular.ttf`: `Hello world` = 12,767 units / 2,048 upem
- `BIZUDPGothic-Bold.ttf`: `Hello world` = 13,237 units / 2,048 upem
- At 18 pt: 112.21 pt vs 116.34 pt (4.13 pt difference)

That is enough for the test's `> 0.5 pt` threshold. The check is a static hmtx measurement, not a
layout pass; the managed test still has to prove the caret actually moves.

For a no-candidate run, choose another available family with genuinely different Regular/Bold
faces (for example DejaVu Sans or Liberation Sans if installed), or skip only when
`CRAFT_FONTS_REQUIRED` is unset. Under `CRAFT_FONTS_REQUIRED=1`, missing BIZ Regular/Bold must be a
failure, not a skip. The test must not silently return to the current "both weights are the same
fallback face" condition.

### Separate Latin-manifest gap

The absence of `fonts/latin-manifest.txt` and `Inter` is still a real production gap for DeckCraft's
default theme and document metrics. It is not necessary to solve it inside this table fixture.
Do not hide the gap by making the test pass with a CJK face. Record it as a separate prerequisite:
pin and verify the Latin manifest, add the reviewed Inter faces, and measure native/web growth.
The table test can proceed with BIZ in the meantime.

## Alternatives considered

### A. Native-only LXGW WenKai GB Regular (recommended)

- Pros: one real permitted face; fixes the Chinese UI regression on Linux and any platform without
  a CJK system font; deterministic test; no wasm growth; keeps Noto CJK banned.
- Cons: ~10-24 MiB native cost; exact GB release/provenance not yet verified; Kai style may be
  less UI-neutral than a sans, so it should remain a fallback behind system CJK when present.
- Decision: adopt only after the provenance, licence/RFN, coverage and byte gates pass.

### B. Native-only full LXGW WenKai Regular

- Pros: broader coverage and the release number is corroborated by Debian 1.511; same licence
  family.
- Cons: likely ~19 MiB or more per face; larger than the GB variant; more glyphs than the UI
  regression needs.
- Decision: fallback if GB cannot be pinned or does not cover the sample. Do not ship both.

### C. System-only Chinese plus owned fixture

- Pros: no shipped Chinese font, no native size growth, no new font licence.
- Cons: Linux/bare CI still cannot render Chinese; the UI test uses a fixture rather than
  production coverage; a strict-glyph paint can panic before a runtime skip. It is honest only if
  the product explicitly accepts that Chinese requires an installed system font.
- Decision: contingency only. The fixture must be contributor-original or openly licensed,
  attributed, and clearly separated from production acceptance. Do not call this "real Chinese
  coverage".

### D. Noto Sans CJK SC

- Rejected by DeckCraft policy. It remains in the central repository for other consumers but is
  filtered out of DeckCraft's native build and must stay out of its wasm allowlist.

### E. Remove or skip the Chinese assertion

- Rejected. It would hide a real missing-glyph panic and contradict the requirement to keep the
  UI regression meaningful. Tofu comparison or `allow_missing_glyphs` is not acceptance.

## Required tests and evidence before implementation

### Acquisition and provenance (must run before code changes)

1. Read the official release page for the selected LXGW project. Record the release tag, commit
   SHA, release date, archive URL, archive byte size and archive SHA-256.
2. Download only the reviewed release archive into the durable acquisition evidence directory.
   Record every release asset name and size. Do not use a package mirror's repacked binary as the
   font source.
3. Extract the single Regular TTF member without renaming, subsetting or regenerating it. Record
   its byte size and SHA-256, and the member's byte identity against the archive.
4. Extract the upstream `OFL.txt`; record byte size, SHA-256 and the complete copyright/RFN
   language. Confirm the original Fontworks Klee One notice is retained.
5. Read the font's `name` table, OS/2 weight/width, sfnt/collection status and `fvar`/`STAT`
   presence. Require one static Regular face if that is what is selected.
6. Read cmap and outlines for every character of `简体中文演示文稿`, especially `简` U+7B80.
   Record the claimed GB repertoire and do not generalize beyond it.
7. Store the acquisition record, licence, hashes and inspection output under the existing
   `/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/font-acquisition/` structure.

If any of these fail, stop. Do not add the bytes and do not switch to "system-only" without
recording the product limitation.

### craft-fonts tests

Add, but do not run outside the managed workflow:

- exact manifest row for family/style/file/script/licence/hash/source;
- SHA-256 and licence adjacency checks;
- one static face, exact `name` table family/style, weight 400, no accidental variable instances;
- exact script `Hans` (unless the verified metadata justifies an additional script tag);
- outline checks for the full sample and U+7B80;
- file-size ceiling;
- a test that the opt-in manifest is not read by a wasm consumer, or an equivalent build-time
  assertion that the wasm `CRAFT_FONTS` slice has no LXGW entry.

### DeckCraft tests

Run with the exact candidate and `CRAFT_FONTS_REQUIRED=1`:

```text
cargo test --config <reviewed-local-config> -p deckcraft-ui-egui --lib \
  ime_tests::cjk_text_in_the_interface_is_drawn_with_real_glyphs
cargo test --config <reviewed-local-config> -p deckcraft-engine --lib \
  tests::tables_with_a_file_defined_style_edit_and_resize
cargo test --config <reviewed-local-config> -p deckcraft-ui-egui --lib
cargo test --config <reviewed-local-config> -p deckcraft-engine --lib
```

The Chinese test must assert all four samples are checked on every platform where the bundled
face is present, and must not reach the missing-glyph panic before the assertion. The table test
must use the BIZ pair and show a caret delta greater than 0.5 pt.

Also run the no-candidate path separately. It may skip the table-metric assertion only when
`CRAFT_FONTS_REQUIRED` is unset; it must not claim Chinese coverage without a real face.

### Wasm and size checks

- Run the repository's wasm gate with the unchanged allowlist.
- Assert the generated wasm `CRAFT_FONTS` contains no LXGW family and no Chinese font bytes.
- Record raw and compressed wasm sizes before/after; require a 0-byte delta for this font.
- Record release native binary/bundle size before/after and enforce the <=26 MiB gate.
- Record peak memory before/after first Chinese paint.

### Performance

Use the existing font-registration benchmark protocol rather than a new ad-hoc timer:

- five alternating before/after pairs, warm-up, at least 100 repeated registrations per sample;
- additional median registration time <= 1 ms;
- within-run CV <= 5%; if noisier, do not claim a pass;
- register one face per family set that includes it, not multiple font copies per character;
- record the exact font bytes, build profile, commit and command.

Also record startup-to-usable and first-Chinese-paint timings, plus the UI screenshot. A glyph
lookup or unit test alone is not a UI acceptance result.

## Implementation sequence (after this plan is approved)

1. Complete acquisition/provenance and freeze the exact release and hashes.
2. Add the single face, `OFL.txt`, opt-in manifest row, attribution/README rows and integration
   documentation.
3. Add craft-fonts metadata/coverage/size tests.
4. Teach DeckCraft's native build to read the opt-in manifest; leave the wasm branch unchanged.
5. Add the exact family to DeckCraft's native interface fallback only if the existing dynamic
   `UiFonts` path does not deterministically select it. Prefer the existing fallback path; do not
   make the Chinese face a primary Latin/UI face.
6. Change the table-style fixture to use BIZ UDPGothic Regular/Bold. Do not add Inter here.
7. Run the managed checks above, capture screenshots and measurements, and preserve all evidence.
8. Update the central Chinese section to distinguish GridCraft's system-only policy from
   DeckCraft's native bundled fallback. Do not add Chinese to any web allowlist.

## Open issues and limitations

- Official GitHub release metadata, exact archive hashes and the upstream `OFL.txt`/RFN text could
  not be read in this environment because DNS/network access failed. The only local version
  evidence is Debian's `1.511+repack-1` packaging of the base family; that is not a font-byte pin.
- The GB release, exact file name, size and coverage are therefore unverified. The plan must not
  be treated as authorization to fetch or commit a font.
- No native or wasm size, startup, memory or registration benchmark was measured here. The table
  BIZ advance check is a read-only hmtx measurement, not a layout or performance result.
- Native-only Chinese fixes the strict-glyph UI failure only when the build actually includes the
  reviewed face. The default no-font path remains a documented limitation and must not claim
  Chinese coverage.
- The table fixture change is test-scoped. The missing `latin-manifest.txt`/Inter issue remains a
  separate production prerequisite; it must not be papered over by selecting a CJK family for
  the default theme.


## Acquisition and v2 review dispositions (2026-10-23)

The independent v2 review is saved in the migration evidence. Implement these corrections with
the original plan; they supersede the speculative GB choice and any broader coverage claim.

- The GB v1.530 Regular release is 26,170,696 bytes, above the planned 24 MiB font ceiling.
  Select the official **LXGW WenKai Lite Regular v1.530**, commit
  `b712c83d98acf977bacfb7fb010207802808bda9`, instead. The unmodified release TTF is
  13,949,472 bytes, SHA256 `f799a538a3355877e597a0ae04050e897932c9ae7689eb9227c2953c5dd549ea`.
  The official release ZIP is 51,990,239 bytes, SHA256
  `7fa084dfdb76f36a40c183dbb50cb5d2f92b2ea26553454988e08c889f8ed5fa`.
  Its extracted OFL.txt is 5,176 bytes, SHA256
  `0333a85fe077b6849f8e417acc44e8bd879442ceb681e271acf93e2d57fd4e2e`.
  All bytes and metadata are retained under the migration evidence's
  `font-acquisition/lxgw-wenkai-lite/`. Static one-face sfnt, weight400, no fvar, actual
  family `LXGW WenKai Lite`, style `Regular`, version1.530; no renaming/subsetting.
  Official source: https://github.com/lxgw/LxgwWenKai-Lite/releases/tag/v1.530 .
  RFNs are 霞鹜, 霞鶩, 落霞孤鹜, 落霞孤鶩, LXGW; retain full licence and Klee copyright.
- Add only `fonts/simplified-chinese-manifest.txt`, with the existing eight-field parser format.
  Use `Hani,Hans` because Deck's script classifier emits Hani, and restrict coverage claims to
  actually checked text. Tests explicitly parse this optional manifest, then verify hashes,
  licence, family/style, weight400, no fvar, outlines and <=24MiB size. Do not add it to FONTS
  or the default embedded manifest. Update central attribution to cover both manifests.
- Accept the existing bundled-first, per-character fallback order. This fixes missing glyphs
  for document-derived UI text and adds a native static-label fallback; it does not promise
  uniform Chinese typography or system-first precedence. BIZ remains ahead for characters it
  already covers. Add LXGW after existing native families in theme::font_definitions.
- The observed panic is kittest's strict MissingGlyphPolicy::Panic; production defaults to
  Tofu. Describe production missing glyphs, not an unproven production crash.
- Deck reads the opt-in manifest only for native builds. Missing file is compatible with older
  craft-fonts checkouts. If present but malformed/unreadable, required-font mode fails; optional
  mode warns and retains the successfully parsed main fonts. Required-font mode with no
  CRAFT_FONTS_DIR is an error. No Chinese font is added to any wasm allowlist.
- The initial-document regression must paint Chinese before any user event. With the LXGW face
  bundled or required-font mode enabled, require all four CJK samples on Linux too; no skips.
  Verify actual proportional/bold glyph ink, including 简, rather than only a cmap entry.
- The table fixture sets explicit BIZ UDPGothic on the first-row runs before either caret
  measurement, leaves bold unset so table style controls it, and requires distinct actual
  regular/bold faces. Required-font mode fails when the pair is absent. In optional mode the
  metric-only assertion may skip with a visible reason; preserve independent edit/resize checks.
- Update Deck attribution, localization-parity and central integration docs. Native size,
  startup/registration measurements, screenshot and wasm zero-byte delta remain acceptance
  gates, not claims established by this acquisition.

The release metadata was obtained through GitHub's read-only connector. Because it cannot
return binary assets, the exact public release ZIP was downloaded through the normal host
network path and verified against GitHub's published SHA256. No repository asset has been
modified or published by acquisition alone.

### Central craft-fonts implementation status (2026-10-24)

The central side is implemented locally, with no DeckCraft source edited here:

- Copied the reviewed upstream `LXGWWenKaiLite-Regular.ttf` (13,949,472 bytes, SHA-256
  `f799a538a3355877e597a0ae04050e897932c9ae7689eb9227c2953c5dd549ea`) and `OFL.txt`
  (5,176 bytes, SHA-256 `0333a85fe077b6849f8e417acc44e8bd879442ceb681e271acf93e2d57fd4e2e`)
  unmodified to `fonts/lxgw-wenkai-lite/`.
- Added one row to the opt-in `fonts/simplified-chinese-manifest.txt`: `LXGW WenKai Lite`,
  `Regular`, `Hani,Hans`, OFL-1.1, adjacent licence path, the TTF hash and the v1.530 release
  source. The default `fonts/manifest.txt`, `FONTS` and the `embed` payload are unchanged.
- Added central tests that parse the optional manifest with `parse_manifest` and check the exact
  row, hash, adjacent licence and reserved-font-name/copyright text, one static weight-400 face
  without `fvar`/`avar`/`STAT`, drawn outlines for `简体中文演示文稿` and `简`, the 24 MiB
  ceiling, and absence from the default manifest/`FONTS`.
- Updated `ATTRIBUTION.md`, `README.md`, `docs/integration.md` and the Chinese section of
  `docs/egui-037-font-coverage-plan.md` to distinguish GridCraft's system-only policy from
  DeckCraft's native-only opt-in.
- DeckCraft build-script/UI integration and the BIZ Regular/Bold table fixture are implemented;
  current automated and central checks are recorded above. Native/wasm size measurements,
  visible edit/save/reopen screenshots and performance remain consumer acceptance gates.
  This central font resource does not own the downstream app/VM resources.
