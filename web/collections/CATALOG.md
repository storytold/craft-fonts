# Shared *craft font catalog

This catalog pins unmodified TrueType/OpenType files from `google/fonts` at
commit `bd8f81ddb5c74d5c8897b36ad88b440266245103`. The manifest is the acquisition
allowlist. Every downloaded font and license must match the pinned Git blob
SHA-1 and exact size. The generated catalog adds a SHA-256 digest, font metadata,
script coverage, embedding flags and variation axes for each face.

The initial catalog has 96 families and 254 faces (173.4 MB): 47 sans, 24 serif,
8 monospace, 10 display and 7 handwriting families. There are 106 variable font
files. Every bundled face has unrestricted installable embedding flags
(`OS/2.fsType == 0`). Zilla Slab was excluded because several upstream faces had
preview/print-only flags. All individual files fit below a 24 MiB asset limit.

## Reproduce

Install Python 3.10+ and dependencies from `requirements.txt`, then run:

```sh
python acquire_fonts.py --output assets/fonts --catalog catalog.json
python acquire_fonts.py --output assets/fonts --catalog catalog.json --verify-only
```

The first command downloads only missing files and verifies existing ones.
Corrupt or modified cached files are rejected. The second command requires all
assets to exist and never uses the network. SHA-256 is locked for all 350 files
(254 fonts and 96 licenses). Font binaries are cached outside
version control. Use `web/tools/acquire-fonts.py` to stage the collection under the package asset directory
directory and make the catalog available to the common font provider. Resolve
all catalog paths against `assetBasePath`, including each family's OFL file.

Retain each `OFL.txt` next to its fonts. The catalog license record provides the
SPDX identifier, upstream source, digest and path. Font acquisition verifies the
presence of the SIL Open Font License 1.1. Full license text is shipped intact.
No font modifications, renaming, subsetting or font-only resale is performed.

## Integration contract

Use family IDs as stable keys. Treat the catalog family name as a display name
and preserve the original names inside the binaries. Each face records its
original family, subfamily and PostScript names so an app can register a stable
CSS alias without modifying font data.

- Lazy-load selected fonts. Do not preload the entire catalog or every CJK face.
- Register all styles an application needs. Variable weight ranges are explicit.
- Use the same original bytes for native browser text and document export.
- Resolve font fallback by actual character coverage. `scripts` is an inventory,
  not a guarantee that every character in that script is supported.
- Preserve face selection, style, variation axes and font IDs in project files.
- User imports belong in shared same-origin IndexedDB with their original bytes,
  not localStorage. They need a separate inventory and must not mutate this pinned
  bundled catalog. Browser storage remains device/profile/origin specific.
- Preserve original imported font licenses. A successful parse is not proof of
  permission to distribute or embed a user's font.
- Check embedding flags and format support before export. A static-only export
  engine must instantiate a variable face before embedding or clearly report
  unsupported variable font use. `fvar` axis support alone does not ensure that
  the application renderer supports those axes.

## Coverage boundaries

The collection spans sans, serif, monospace, display and handwriting. Noto
families add Arabic, Hebrew, major Indic scripts, Thai, Lao, Khmer, Myanmar,
Japanese, Korean, simplified Chinese and traditional Chinese. Individual fonts
also carry Greek, Cyrillic or Vietnamese characters where upstream includes them.

This is a broad curated collection, not every typeface or Unicode character.
Emoji/color fonts, symbol-only fonts, musical notation, historic scripts,
specialist mathematics and many minority scripts are outside this first catalog.
Font availability does not provide Arabic/Indic shaping, bidirectional layout,
vertical CJK typography, ruby text, line-breaking or PDF embedding by itself.
Each app must test its own text layout and export paths.

The catalog contains full fonts, including large CJK files. A service worker or
user-triggered offline installation can cache selected families. Initial loading
must remain independent of this complete asset set.

## Updates

An upstream application sync must leave this catalog, manifest, acquisition
script, shared font provider and user IndexedDB schema intact. Catalog updates
are separate reviewed changes: pin a new source commit, update the manifest's
exact paths and blob digests, regenerate, and verify licenses and runtime/export
compatibility. Do not silently replace bundled or user-uploaded font versions.
