# egui 0.37 fallback-font prerequisite

## Status: central prerequisite validated locally; consumer qualification continues

The central asset/library change is validated atop `fff559f`. Managed
`egui037-fonts-candidate-20261023-chinese-05` passed all 29 all-feature tests and
strict clippy. `egui037-fonts-candidate-20261024-final-06` passed formatting,
28 default workspace tests, 4 explicit embed tests and strict all-feature clippy.
Fresh implementation review `egui037-fonts-upstream-final-review-20261024-root-v3`
confirmed all 13 font hashes, the five added font/licence pairs, metadata,
provenance and default/opt-in behavior. Consumer native GUI, font registration,
web size and performance checks remain in their separate app resources.

## Follow-up prerequisite: Hebrew consumer coverage

DeckCraft needs an actual Hebrew fallback for its mixed-script editing paths. The first
amendment incorrectly called the migration fixture's default instance Regular: the variable
font at google/fonts 8b0a1d0 defaults to Thin (`wght=100`) and enumerates nine named instances.
Keep that original fixture at `fixtures/noto-sans-hebrew/NotoSansHebrew.ttf` unchanged; it is
not added to this repository. Instead use the upstream **static Regular** release, which needs no
per-app variation-coordinate exception and registers one face.

### Provenance (verified on disk)

- Official release: [NotoSansHebrew-v3.001](https://github.com/notofonts/hebrew/releases/tag/NotoSansHebrew-v3.001),
  target commit `036f3206f67caac235cf8546a7751d3440771a7e` (2024-06-03).
- Archive `NotoSansHebrew-v3.001.zip`: 3,734,185 bytes, SHA-256
  `df0a71814b4e63644cf40fcc4529111b61266b7a2dafbe95068b29a7520cc3cb`
  (`archive-provenance.json`; internal timestamps `2024-06-03 13:44` match the release date).
- Archive member `NotoSansHebrew/full/ttf/NotoSansHebrew-Regular.ttf` is **byte-identical** to
  the extracted `full.ttf`: no rename, subset, generated instance or internal name edit.
- Font: 66,156 bytes, SHA-256
  `671951828bd5c95db818e5bb12dcea2d0c0dda00311888522be061ee6835125e`.
- Archive-root `OFL.txt`, extracted as `license.txt`, is byte-identical: 4,382 bytes, SHA-256
  `9b9fe028b5ba74d231659a1bbaf0ed09b11e759d1ca6a070999e16d151616b47`.
- Acquisition and all three upstream static variants (`full.ttf`, `hinted.ttf`, `unhinted.ttf`)
  are retained at
  `/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/font-acquisition/noto-sans-hebrew-static/`.
  The directory's `archive-provenance.json` records the URL, release commit, zip bytes and zip
  SHA-256. The top-level `acquisition.json` also records the Hebrew font and licence members,
  including their release URL, release commit, sizes and hashes. The evidence owner added
  those records before the fresh implementation review.
- Metadata: no `fvar`, `avar` or `STAT` table; sfnt version 1.0; 17 tables; 470 glyphs; OS/2
  `usWeightClass=400`, `usWidthClass=5`, `fsSelection=REGULAR|USE_TYPO_METRICS`, `macStyle=0`,
  `italicAngle=0`; names `Noto Sans Hebrew`, `Regular`, `Noto Sans Hebrew Regular`,
  `NotoSansHebrew-Regular`, version `3.001`.
- Coverage: all 27 letters `U+05D0..U+05EA` and all 24 assigned points/punctuation
  `U+05B0..U+05C7` map to non-empty outlines. 88 of the 112 code points in `U+0590..U+05FF` are
  mapped and every mapped glyph draws. The hinted/unhinted variants are smaller subsets (147
  cmap entries / 151 glyphs vs 464 / 470) with the same Hebrew coverage, so the full face is
  chosen to avoid assuming coverage from a subset.

### Repository addition (applied locally)

- `fonts/noto-sans-hebrew/NotoSansHebrew-Regular.ttf`, 66,156 bytes, SHA-256
  `67195182…5125e`, byte-identical to the verified upstream member.
- `fonts/noto-sans-hebrew/OFL.txt`, 4,382 bytes, SHA-256 `9b9fe028…616b47`, byte-identical to
  the archive-root `OFL.txt`. OFL-1.1 declares no Reserved Font Name, so shipping the unmodified
  file with the adjacent licence is compliant.
- Manifest row (eight fields; the `source` field names the release tag):

  ```text
  Noto Sans Hebrew | Regular | fonts/noto-sans-hebrew/NotoSansHebrew-Regular.ttf | Hebr | OFL-1.1 | fonts/noto-sans-hebrew/OFL.txt | 671951828bd5c95db818e5bb12dcea2d0c0dda00311888522be061ee6835125e | https://github.com/notofonts/hebrew/releases/tag/NotoSansHebrew-v3.001
  ```

- Matching `ATTRIBUTION.md` and README rows: family `Noto Sans Hebrew`, style `Regular`, script
  `Hebr`, author The Noto Project Authors, OFL-1.1, licence destination
  `fonts/noto-sans-hebrew/OFL.txt`.
- `docs/integration.md` lists the `Hebr` face in the central fallback set and states that it is
  native-only for this round.
- At the time of this Hebrew follow-up the `crates/craft-fonts/Cargo.toml` "about 43 MiB"
  embedded-size comment covered a manifest of 45,600,892 bytes (43.49 MiB) after +66,156.
  The later Devanagari follow-up raises the total to 45,878,572 bytes (43.75 MiB) and changes
  that comment to "about 43.8 MiB". This is the documented "embedded-size comment" result,
  not a silent omission.
- Every existing font, manifest line, attribution row, README row and the original baseline
  fixture stay byte-identical.

### Registration and size gates

- Exactly +66,156 font bytes and +4,382 licence bytes (70,538 total) in the central asset set.
- Exactly one additional native FontFace **file/face**, never nine instances. "One face" means
  one manifest row and one static face with no `fvar`/named instances; a consumer registers that
  face once per family set that includes it (egui's proportional family and, where present, its
  `CELL`/monospace family), so two families may reference the same face without duplicating bytes
  or creating instances.
- Native GridCraft embeds the new non-barred row under its existing policy; record its
  +66,156-byte font growth and the separate executable/bundle growth. GridCraft's five-face web
  allowlist stays unchanged, and DeckCraft's current web allowlist also excludes Hebrew. This
  amendment claims **native** required-font acceptance only; web support remains an explicit
  separate qualification gap.
- **Shaping qualification:** this range includes combining marks and punctuation; the marks
  need GPOS positioning. The storytold fork uses HarfRust and bidi layout (epaint
  `font_face.rs` shaper data and `text_layout.rs`), so the earlier review
  statement that this path has no shaping was incorrect. Font coverage alone does not prove
  correctly positioned or reordered vocalized Hebrew. The initial glyph acceptance uses
  unpointed mixed text such as `שלום deck 42` (and unpointed `שלום`
  alone); it must not claim correctly shaped niqqud.
- Benchmark identical native DeckCraft font-registration operations before/after, five
  alternating pairs with warm-up and at least 100 repeated registrations per sample; require
  additional median time <=1 ms per registration and within-run CV <=5%. No performance pass if
  measurement is too noisy. Register the one static face, not nine instances.

### Consumer acceptance

- DeckCraft native required-font acceptance must assert exact family `Noto Sans Hebrew`, static
  Regular weight (OS/2 400, no `fvar`), Hebrew coverage, and mixed Hebrew/Latin caret/edit
  behavior for `שלום deck 42` with system font discovery isolated (`CRAFT_FONTS_DIR` set, no
  system font provider). The no-font path must build/run without claiming bundled Hebrew. Strict
  glyph/accessibility checks remain enabled; no `allow_missing_glyphs`, tofu comparison or
  silent skip is acceptance.
- Run central `fmt`, `clippy`, default/embed tests and the hash/licence/metadata tests below,
  then the named DeckCraft required/no-font tests and actual mixed-direction editing. This
  revision received the fresh adversarial review in
  `egui037-font-hebrew-plan-review-20261023-root-v2.md`; the fresh implementation review
  and central validation are recorded above. Nothing is published without explicit approval.

GridCraft's strict egui_kittest run exposes missing UI symbols and Korean glyphs. This
prerequisite adds only the fonts that are actually missing, keeps every existing face and asset
byte-identical, and keeps `CRAFT_FONTS_DIR` optional. It does not copy font binaries into app
repositories, and it does not use the barred Adobe / Source Han / Noto CJK families in GridCraft.

Fresh remote state used for this plan:

- `craft-fonts` PR 23 is **closed, unmerged**, head
  `64b29ef11d9bfb2f50517ef659d4cfb410ad1afc`. Reuse only its selected, verified NanumGothic
  provenance below; do not merge the PR, its Latin manifest or its other faces.
- `craft-fonts` PR 46 is **open**, head `692413113844e5c621239acc429d621addfdb084`. Reuse only
  the selected Noto Sans Symbols provenance below; do not merge the whole PR or its CJK, emoji,
  Math and UI face set.

## Implementation status and verified assets

The exact upstream assets have been acquired, verified and installed locally. The acquisition
evidence is `/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/font-acquisition/`
with `acquisition.json` recording every URL, path, byte count and SHA-256 for the three faces
below. Every recomputed hash matches the recorded value and the pins below. The later Hebrew
face's structured record is in the follow-up section above and in
`font-acquisition/noto-sans-hebrew-static/archive-provenance.json`.

Read-only cmap and `glyf` inspection of the downloaded bytes established the actual coverage:

- NanumGothic Regular covers all 11,172 modern Hangul syllables (`U+AC00..U+D7A3`) and all 51
  compatibility Jamo (`U+3131..U+3163`), with drawn outlines for `한`, `글`, `각` and `힣`. It
  does not cover `✕`, `⌃`, `✓` or `📋`.
- Noto Sans Symbols Regular (Symbols 1) covers `⌃` U+2303, `↘` U+2198 and `↑` U+2191 with drawn
  outlines. It does **not** cover `✕` U+2715, `◀` U+25C0 or `✓` U+2713.
- Noto Sans Symbols 2 Regular covers `✕` U+2715, `◀` U+25C0 and `✓` U+2713 with drawn outlines.
  It does **not** cover `⌃`, `↘` or `↑`, so it cannot replace Symbols 1.

Symbols 2 is therefore **required**, not conditional: Symbols 1 lacks `✕` and `◀`, and no
existing permitted face covers `✕`. `📋` U+1F4CB is supplied by egui's bundled emoji faces and
is deliberately not claimed as craft-fonts coverage.

**Upstream filename correction:** the Symbols 1 file at the pinned upstream commit is
`NotoSansSymbols[wght].ttf`, not `NotoSansSymbols.ttf` as an earlier draft said. The local
manifest row and destination use `NotoSansSymbols.ttf`; this is a filename-only rename of the
byte-identical download. The internal family, style, PostScript and full names are unchanged
(`Noto Sans Symbols`, `Regular`, `NotoSansSymbols-Regular`, `Noto Sans Symbols Regular`) and the
SHA-256 is unchanged.

## Verified upstream assets

All files are upstream, unmodified, unsubsetted and unrenamed except for the documented Symbols 1
filename-only rename. All three fonts are OFL-1.1; their `OFL.txt` files are installed next to
them and are not copied into app repositories.

| Face | Upstream pin and file | Local destination | Bytes | SHA-256 |
|---|---|---|---|---|
| NanumGothic | [google/fonts@bd8f81d](https://github.com/google/fonts/tree/bd8f81ddb5c74d5c8897b36ad88b440266245103/ofl/nanumgothic) / `NanumGothic-Regular.ttf` | `fonts/nanum-gothic/NanumGothic-Regular.ttf` | 2,054,744 | `76f45ef4a6bcff344c837c95a7dcc26e017e38b5846d5ae0cdcb5b86be2e2d31` |
| Noto Sans Symbols | [google/fonts@47d54ca](https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/notosanssymbols) / `NotoSansSymbols[wght].ttf` | `fonts/noto-sans-symbols/NotoSansSymbols.ttf` | 372,684 | `f7e7e04b4a24b6c78893d50cbfd2b2f6cae49617ab047bfef668d252adb128f7` |
| Noto Sans Symbols 2 | [google/fonts@47d54ca](https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/notosanssymbols2) / `NotoSansSymbols2-Regular.ttf` | `fonts/noto-sans-symbols-2/NotoSansSymbols2-Regular.ttf` | 1,233,128 | `7d5fb73b7ca67a6798101741f5d280a3d016a56a197afcd4199dbb57b4b82a21` |

The licence files are:

- `fonts/nanum-gothic/OFL.txt`, 4,534 bytes,
  SHA-256 `eeacf16032901d0ed0456876ec77b8f0fda6b3fecec7d972f8543eb602e6c30f`.
- `fonts/noto-sans-symbols/OFL.txt` and `fonts/noto-sans-symbols-2/OFL.txt`, both 4,383 bytes,
  SHA-256 `b118dd41337806a5d4797052c77caf3bd096aed783e5eb21b4d11154351e1ac0`.

Exact acquisition commands (recomputed hashes must match before a manifest edit):

```sh
mkdir -p fonts/nanum-gothic fonts/noto-sans-symbols fonts/noto-sans-symbols-2
curl -fL -o fonts/nanum-gothic/NanumGothic-Regular.ttf \
  https://raw.githubusercontent.com/google/fonts/bd8f81ddb5c74d5c8897b36ad88b440266245103/ofl/nanumgothic/NanumGothic-Regular.ttf
curl -fL -o fonts/nanum-gothic/OFL.txt \
  https://raw.githubusercontent.com/google/fonts/bd8f81ddb5c74d5c8897b36ad88b440266245103/ofl/nanumgothic/OFL.txt
curl -fL -o 'fonts/noto-sans-symbols/NotoSansSymbols.ttf' \
  'https://raw.githubusercontent.com/google/fonts/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/notosanssymbols/NotoSansSymbols%5Bwght%5D.ttf'
curl -fL -o fonts/noto-sans-symbols/OFL.txt \
  https://raw.githubusercontent.com/google/fonts/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/notosanssymbols/OFL.txt
curl -fL -o fonts/noto-sans-symbols-2/NotoSansSymbols2-Regular.ttf \
  https://raw.githubusercontent.com/google/fonts/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/notosanssymbols2/NotoSansSymbols2-Regular.ttf
curl -fL -o fonts/noto-sans-symbols-2/OFL.txt \
  https://raw.githubusercontent.com/google/fonts/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/notosanssymbols2/OFL.txt
sha256sum fonts/nanum-gothic/NanumGothic-Regular.ttf \
  fonts/noto-sans-symbols/NotoSansSymbols.ttf \
  fonts/noto-sans-symbols-2/NotoSansSymbols2-Regular.ttf
```

### NanumGothic Regular (required)

- Family **`NanumGothic`** (exact spelling, no space), style `Regular`, scripts `Hang,Kore`.
- Licence header: `Copyright (c) 2010, NHN Corporation (http://www.nhncorp.com), with Reserved
  Font Name Nanum, Naver Nanum, NanumGothic, ...`. Authors recorded by the PR 23 evidence: NHN
  Corporation; Sandoll Communications.
- `METADATA.pb` was read for author/licence confirmation but is not copied into `fonts/`: this
  repository ships the font, `OFL.txt`, the manifest row and the attribution row only.
- Font: 13,297 glyphs (`maxp`), 11,827 cmap entries; 11,172/11,172 modern Hangul syllables; 51/51 compatibility Jamo
  `U+3131..U+3163`; 42 of the 43 archaic compatibility Jamo `U+3164..U+318E`; no conjoining
  Hangul Jamo blocks (`U+1100..U+11FF`, `U+A960..U+A97F`, `U+D7B0..U+D7FF`).
- Outlines: `한`, `글` and `각` are simple glyphs; `힣` is a composite whose two referenced
  components both have non-empty outlines.

### Noto Sans Symbols Regular (required)

- Family **`Noto Sans Symbols`**, style `Regular`, script `Zsym`. This is a variable font with a
  `wght` axis (`100..900`, default `400`); the name table's subfamily is `Regular` and the
  PostScript name is `NotoSansSymbols-Regular`.
- The upstream filename is `NotoSansSymbols[wght].ttf`; the local file is the same bytes under
  `NotoSansSymbols.ttf`.
- cmap/outline evidence: `⌃` U+2303, `↘` U+2198 and `↑` U+2191 map to non-empty outlines.
  `✕` U+2715, `◀` U+25C0, `✓` U+2713 and `📋` U+1F4CB are absent.

### Noto Sans Symbols 2 Regular (required)

- Family **`Noto Sans Symbols 2`**, style `Regular`, script `Zsym`.
- cmap/outline evidence: `✕` U+2715, `◀` U+25C0 and `✓` U+2713 map to non-empty outlines.
  `⌃`, `↘` and `↑` are absent, so Symbols 2 cannot replace Symbols 1. Symbols 2 also has
  `📋` U+1F4CB, but that glyph is supplied to GridCraft by egui's emoji faces and is not
  claimed as craft-fonts coverage.
- Added only because the actual cmap proves it is required, not because PR 46 proposed it.

### Faces that must not be added here

- `📋` U+1F4CB is supplied by egui's bundled emoji faces (`Noto Emoji`, `emoji-icon-font`). Do
  not add a craft emoji face or claim craft-fonts covers `📋`.
- Do not add Noto CJK, Source Han, Adobe, color emoji or other large CJK/emoji assets for this
  prerequisite.

## Actual cmap coverage, old and new faces

Independently checked with a read-only cmap reader against the current and downloaded files (not
a build). `yes` means the codepoint maps to a drawn outline at the default instance
(`glyf` for TrueType faces; CFF for Noto Sans CJK SC).

| Face | `↘` | `✕` | `◀` | `↑` | `📋` | `✓` | `⌃` |
|---|---|---|---|---|---|---|---|
| BIZ UDPGothic Regular / Bold | yes | no | yes | yes | no | yes | no |
| BIZ UDMincho Regular | yes | no | yes | yes | no | yes | no |
| BIZ Shippori Mincho Regular | no | no | no | yes | no | no | no |
| Noto Sans CJK SC Regular | yes | no | yes | yes | no | yes | no |
| egui Hack Regular | yes | no | yes | yes | no | no | no |
| egui Noto Emoji Regular | yes | no | yes | no | yes | no | no |
| egui emoji-icon-font | no | no | no | no | yes | no | no |
| egui Ubuntu Light / egui-icons | no | no | no | no | no | no | no |
| NanumGothic Regular | yes | no | yes | yes | no | no | no |
| Noto Sans Symbols Regular | yes | no | no | yes | no | no | yes |
| Noto Sans Symbols 2 Regular | no | yes | yes | no | yes | yes | no |

Consequences:

- The no-font path cannot promise all seven symbols. `✕` and `⌃` are real gaps, and `✓` is not
  in the bundled egui faces even though BIZ and Noto Sans CJK SC have it.
- With the selected craft faces, `⌃` comes from Symbols 1; `✕`, `◀` and `✓` come from Symbols 2;
  `↘` and `↑` come from Symbols 1 (NanumGothic also happens to have them); `📋` comes from egui.
- Noto Sans CJK SC happens to cover Korean and `↘ ◀ ↑ ✓`, but GridCraft bars it. It cannot be
  used as the Korean or symbol answer.

## Repository changes (applied locally)

These three manifest lines were added by the earlier prerequisite; the Hebrew follow-up adds
one more row (above). Every pre-existing line is unchanged:

```text
NanumGothic | Regular | fonts/nanum-gothic/NanumGothic-Regular.ttf | Hang,Kore | OFL-1.1 | fonts/nanum-gothic/OFL.txt | 76f45ef4a6bcff344c837c95a7dcc26e017e38b5846d5ae0cdcb5b86be2e2d31 | https://github.com/google/fonts/tree/bd8f81ddb5c74d5c8897b36ad88b440266245103/ofl/nanumgothic
Noto Sans Symbols | Regular | fonts/noto-sans-symbols/NotoSansSymbols.ttf | Zsym | OFL-1.1 | fonts/noto-sans-symbols/OFL.txt | f7e7e04b4a24b6c78893d50cbfd2b2f6cae49617ab047bfef668d252adb128f7 | https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/notosanssymbols
Noto Sans Symbols 2 | Regular | fonts/noto-sans-symbols-2/NotoSansSymbols2-Regular.ttf | Zsym | OFL-1.1 | fonts/noto-sans-symbols-2/OFL.txt | 7d5fb73b7ca67a6798101741f5d280a3d016a56a197afcd4199dbb57b4b82a21 | https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/notosanssymbols2
```

Matching `ATTRIBUTION.md` rows and README `Fonts` rows were added for the same three files; the
Hebrew follow-up adds matching rows for its one file. Every pre-existing font, asset, manifest,
attribution and README row is unchanged. The eight-field manifest format is kept; PR 46's
optional ninth `role` field and its `ui` faces are not copied.

## Consumer behavior

### No-font/default mode (no `CRAFT_FONTS_DIR`)

- The app must still build, test and run.
- Paint an explicit unavailable-icon state or fall back for UI icons that bundled/system faces
  do not provide (notably `✕` and `⌃`, and `✓` in an egui-only environment). Do not paint tofu,
  and do not weaken strict glyph checks, accessibility or `MissingGlyphPolicy` to make a test
  pass.
- Acceptance tests in this mode assert only glyphs that bundled/system faces really provide.
  They document `✓`/`⌃`/`✕` as optional-input or platform gaps rather than claiming all seven.
- Choosing an unavailable script must produce deliberate, tested fallback behavior. A hidden
  local fixture path is never a prerequisite.

### Extended acceptance mode (`CRAFT_FONTS_DIR` set, `CRAFT_FONTS_REQUIRED=1`)

- Require the exact union `↘ ✕ ◀ ↑ 📋 ✓ ⌃` in both proportional and `CELL` families.
  `📋` comes from egui's emoji faces; the other faces must be present in the installed
  fallback set.
- Require Korean UI strings such as `데이터` and `한글`.
- Fail, do not return early or skip, when a required face or codepoint is missing. The existing
  `symbol_glyphs.rs` early return when craft-fonts is absent is not acceptable acceptance
  behavior.
- No `allow_missing_glyphs()` / tofu comparison may substitute for the exact-codepoint
  assertion.

### Fallback ordering in GridCraft

Preserve the actual order, not the generic integration-doc order:

1. app/bundled faces;
2. Hack/emoji faces already added by `font_definitions`;
3. system script fallbacks;
4. allowed craft-fonts faces last.

Append NanumGothic, Noto Sans Symbols and Noto Sans Symbols 2 only when allowed and successfully
parsed. They must not displace system faces that already cover a glyph. Match exact family and
style: `NanumGothic` / `Regular`, `Noto Sans Symbols` / `Regular` and
`Noto Sans Symbols 2` / `Regular`; not `Nanum Gothic` and not a Symbols 2 row standing in for
Symbols 1. The manifest parser accepts arbitrary script strings, so exact `Hang,Kore` and `Zsym`
assertions belong in tests.

### Barred families in GridCraft

Keep the central repository's barred faces for other consumers, but filter them in GridCraft:

- `Source Han*`
- `Source Sans*`
- `Source Serif*`
- `Noto Sans CJK*`
- `Noto Serif CJK*`
- any Adobe-family face

Do not use a broad `Noto Sans` prefix that would also reject the permitted `Noto Sans Symbols`
and `Noto Sans Symbols 2`.

## Web allowlist

Before this migration, GridCraft's wasm `build.rs` embedded no craft-fonts. The local candidate
now uses the following explicit allowlist, retaining regular and bold Japanese UI faces.
The five font files total 12,970,840 raw bytes; this is not a measured wasm size or budget pass:

- `BIZ UDPGothic Regular`
- `BIZ UDPGothic Bold`
- `NanumGothic Regular`
- `Noto Sans Symbols Regular`
- `Noto Sans Symbols 2 Regular` (required by the verified cmap above)

Do not include Noto CJK or any barred family. Measure and record:

- raw bytes of each allowlisted font;
- raw wasm size before and after;
- compressed (gzip/brotli as used by the consumer) wasm size before and after;
- the consumer's actual web size gate.

Name the budget and add or use a real measurement gate; do not say "existing budget" without a
number and a check. If Korean web support is not accepted, remove Korean from the web language
picker and restored web preferences and record the gap honestly rather than shipping a claim
that cannot render.

## Chinese

Chinese stays documented as **system-only** in GridCraft while Noto CJK is barred. Keep Chinese
out of GridCraft's strict multilingual sample. Do not claim craft-fonts Chinese coverage from a
filtered face.

DeckCraft has a separate **native-only opt-in**, not a change to the default manifest:
`fonts/simplified-chinese-manifest.txt` lists the unmodified official LXGW WenKai Lite Regular
v1.530 (`Hani,Hans`, OFL-1.1, upstream commit
`b712c83d98acf977bacfb7fb010207802808bda9`). It is not in `fonts/manifest.txt`, `FONTS` or the
`embed` payload, and it must stay out of every wasm allowlist. It fixes document-derived UI
text such as `简体中文演示文稿` and U+7B80 `简` through the consumer's existing per-character
fallback after existing native families. It does not promise uniform Chinese typography,
system-font-first precedence, or static localized UI-label coverage. The central tests parse the
optional manifest with `parse_manifest` and check the exact row, hash, adjacent licence and RFN
text, one static weight-400 face, outlines for the full sample and `简`, and the 24 MiB ceiling.

## Tests in craft-fonts

`crates/craft-fonts-tests/tests/fonts.rs` now covers:

- exact style from the font `name` table for every manifest row;
- exact manifest metadata for all three new faces: `NanumGothic` / `Regular` / `Hang,Kore` and
  `Noto Sans Symbols` plus `Noto Sans Symbols 2`, both `Regular` / `Zsym`, all three OFL-1.1;
- Korean UI strings (`데이터`, `한글`, `파일`, `편집`, `보기`, `삽입`, `서식`, `저장`, `열기`,
  `닫기`, `인쇄`, `취소`, `확인`);
- the observed NanumGothic cmap: all 11,172 modern Hangul syllables and all 51 compatibility
  Jamo;
- outline checks (not cmap presence alone) for `한`, `글`, `각` and `힣`;
- exact per-face symbol assignment with outlines: Symbols 1 covers `⌃ ↘ ↑`; Symbols 2 covers
  `✕ ◀ ✓`; `📋` is not asserted against craft-fonts.
- the exact Hebrew manifest row and source, one Hebr face, and the one-face/static invariant:
  sfnt (not TTC), no `fvar`/`avar`/`STAT`, OS/2 weight 400/width 5 and upright `REGULAR`;
- all 27 Hebrew letters `U+05D0..U+05EA` and all 24 assigned points/punctuation
  `U+05B0..U+05C7` with drawn outlines (coverage, not GPOS shaping), plus the unpointed
  sample `שלום עברית …` used by the mixed acceptance string.
- the optional `fonts/simplified-chinese-manifest.txt` row for `LXGW WenKai Lite` / `Regular`
  (`Hani,Hans`), parsed with `parse_manifest` and kept out of `FONTS`/`fonts/manifest.txt`;
  exact SHA-256, adjacent OFL licence with the five upstream reserved names and Klee copyright,
  one static OS/2 weight-400 face with no `fvar`/`avar`/`STAT`, drawn outlines for
  `简体中文演示文稿` and U+7B80 `简`, and a 24 MiB file ceiling. The optional manifest is not
  read by craft-fonts' default build or embed payload; consumers must keep it out of wasm.

Every existing checksum, licence, family, Japanese, Chinese, Arabic, Thai, Georgian,
vertical-text and embed test is preserved. No test uses an `if present` skip or a tofu
comparison in place of coverage. These assertions passed the managed 05/06 checks recorded above.

## Regression sources

- PR 23 evidence:
  `/home/codex/work/task-evidence/ci-contributor-overlap-20261015/prs/craft-fonts/23/diff.txt`
  (NanumGothic manifest row, OFL header, destination paths).
- PR 46 evidence:
  `/home/codex/work/task-evidence/language-adversarial-review-20261021/sources/craft-fonts-46/fonts/manifest.txt`
  and `craft-fonts-46-metadata.json` (Symbols 1/2 rows, hashes, sizes and coverage notes).
- Acquisition evidence:
  `/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/font-acquisition/`
  (`acquisition.json`, `METADATA.pb`, `OFL.txt` and the downloaded fonts).
- Independent plan reviews:
  `/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/egui037-font-coverage-plan-review-20261023-root-v1.md`
  (overall plan) and
  `/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/egui037-font-hebrew-plan-review-20261023-root-v1.md`
  (which found the variable fixture's Thin default and nine named instances) and
  `/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/egui037-font-hebrew-plan-review-20261023-root-v2.md`
  (approval of this static-face revision, with the provenance and shaping nits).
- Hebrew acquisition evidence:
  `/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/font-acquisition/noto-sans-hebrew-static/`
  (`archive-provenance.json`, the release zip, `full.ttf`, `hinted.ttf`, `unhinted.ttf` and
  `license.txt`). The existing variable fixture at
  `/home/codex/work/CraftApps/egui037-migration-20261022/fixtures/noto-sans-hebrew/` is
  retained unchanged and is not a regression input for the new static face.
- DeckCraft LXGW opt-in evidence:
  `/home/codex/work/ssd2/task-evidence/egui037-migration-20261022/font-acquisition/lxgw-wenkai-lite/`
  (`LXGWWenKaiLite-Regular.ttf`, `OFL.txt`, the public v1.530 release ZIP and
  `metadata-inspection.json`). The reviewed TTF and OFL are copied unmodified into
  `fonts/lxgw-wenkai-lite/` and listed only in the opt-in Simplified Chinese manifest.

## Managed validation commands (completed as recorded above)

```sh
python3 /home/codex/work/maintenance/task.py run \
  --job-id <unique>-font-fmt-01 --kind build --task <task> \
  --source /home/codex/work/CraftApps/egui037-migration-20261022/craft-fonts-implementation \
  --evidence /home/codex/work/ssd2/task-evidence/<task> \
  -- cargo fmt --all --check
python3 /home/codex/work/maintenance/task.py run \
  --job-id <unique>-font-clippy-01 --kind build --task <task> \
  --source /home/codex/work/CraftApps/egui037-migration-20261022/craft-fonts-implementation \
  --evidence /home/codex/work/ssd2/task-evidence/<task> \
  -- cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
python3 /home/codex/work/maintenance/task.py run \
  --job-id <unique>-font-test-01 --kind build --task <task> \
  --source /home/codex/work/CraftApps/egui037-migration-20261022/craft-fonts-implementation \
  --evidence /home/codex/work/ssd2/task-evidence/<task> \
  -- cargo test --workspace --locked
python3 /home/codex/work/maintenance/task.py run \
  --job-id <unique>-font-embed-01 --kind build --task <task> \
  --source /home/codex/work/CraftApps/egui037-migration-20261022/craft-fonts-implementation \
  --evidence /home/codex/work/ssd2/task-evidence/<task> \
  -- cargo test -p craft-fonts --features embed --locked
```

## Completion gate

Locally satisfied by this implementation:

1. the exact downloaded files match the recomputed SHA-256 and immutable upstream pins;
2. cmap and outline inspection records which face covers `✕`, `⌃`, the arrows and the Korean
   strings, and the real Hangul/Jamo counts;
3. `OFL.txt`, manifest, `ATTRIBUTION.md`, README, `docs/integration.md` and the regression
   tests are updated for exactly the files added;
4. existing fonts and assets are byte-identical, and the variable Hebrew fixture is untouched;
5. the Hebrew add is exactly one static 400-weight face (70,538 bytes with its licence), with
   no `fvar`/`avar`/`STAT` and no nine-instance registration plan.

Central prerequisite complete:

6. managed fmt, clippy, workspace and embed checks passed in 05/06; fresh implementation
   review verified the tested source and asset bytes.

Downstream acceptance remains open in the separately owned app resources:
7. DeckCraft's native required/no-font Hebrew acceptance and mixed-direction editing are run
   with system discovery isolated;
8. GridCraft's no-font and extended-mode behavior, fallback order, barred-family filter, native
   Hebrew growth measurements and web allowlist/measurements are validated separately in the
   GridCraft checkout;
9. The fresh implementation review's consumer-acceptance gaps are closed. The Hebrew
   acquisition records are complete; the asset review verified their hashes and provenance.

Keep all changes local. No push, tag or PR publication without fresh explicit user approval of
the concrete diff and validation.
