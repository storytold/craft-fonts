# Optional shared font library for web launchers

This is a portable version of the browser font system developed in
[mmmbeer/starcraft](https://github.com/mmmbeer/starcraft). It adds an optional static browser
package beside the existing Rust library. It does not change `fonts/manifest.txt`, the Cargo
workspace, native builds or the `CRAFT_FONTS_DIR` recipe.

## Build and try it

Node 22 or newer is required for development. No JavaScript dependencies are shipped at runtime.

```sh
node web/tools/build.mjs                       # output: .cache/web-fonts
python3 -m http.server 8000 --directory .cache/web-fonts
# open http://localhost:8000
cd web
npm ci
npm test
```

The builder reads the existing eight-field manifest. It checks each original font's SHA-256,
copies its licence, generates `catalog.json`, and copies the standalone UI and browser modules.
The default package has the repository's six families / seven faces. New manifest entries are
included on the next build. Serve over HTTPS in production (localhost works for development).
Nothing is added to WebAssembly, so the font collection does not increase the WASM file size.
The example is a working search/import/storage UI, not a compiled Craft application.

## Host it with multiple editors

Copy the generated directory to an origin shared by the launcher and editors, such as
`/tools/fonts/`. Add the stylesheet and import the manager on each page. Paths work under
subdirectories and need no server API.

```html
<link rel="stylesheet" href="/tools/fonts/library.css">
<button data-open-fonts>Font library</button>
<script>
window.CraftFontsConfig = {
  baseURL: new URL('/tools/fonts/', location.origin).href,
  databaseName: 'craft-font-library',
  autoActivate: false
};
</script>
<script type="module" src="/tools/fonts/manager.mjs"></script>
```

Configuration must precede imports. `baseURL` defaults to the modules' directory.
`assetBaseURL` defaults to `baseURL`. `catalogURL`, `discoveryURL`, `supportURL` and
`databaseName` can be overridden. Assets must remain within the configured same-origin
asset directory. The manager adds an Aa button on canvas pages and handles `craft:fonts`.
You can also import and call `openFonts()` yourself. No host styles or branding are required.

All pages use the same `databaseName` and origin to share font bytes, activation, removal and
favorite preferences. Browser profiles and different origins do not share storage. This is not
an account-based sync service. Same-origin iframes must load the package inside the editor
frame so the bridge exists in the same window as WASM. Cross-origin sharing is not implemented.

Automatic downloads are **off by default**. Set `autoActivate: true` or call `activateDefaults()`
to install the included catalog progressively (two downloads at a time). Deactivated and removed
faces stay that way. Failed downloads can be retried. Stored fonts remain usable offline after
loading the application itself; this package does not install a service worker.

## Native engine bridge: required for actual editor use

`FontFace` loads browser previews only. To make a font selectable in a Craft editor and usable
for rendering/export, its WASM host must register the original bytes in the app's own font database.
Adding only the HTML snippets above does **not** update an unmodified editor's font selector.

The per-window `window.CraftFonts` version-1 API is deliberately small:

| API | Meaning |
|---|---|
| `version` | Check for `1` before consuming. |
| `take()` | Up to two `{id, family, style, bytes: Uint8Array}` records. Bytes are copies. |
| `ack(id, ok, message)` | Acknowledge a delivered face after native registration. Unknown IDs are ignored. |
| `retry(id)` | Explicitly requeue an active face after rejection or a lost acknowledgement. |
| `status()` | Ready state, active/acknowledged counts, default-download progress/errors. |
| `craft:font-change` event | Request an app repaint and drain another batch. |

Register each style's bytes using the app's existing font installer. Do not acknowledge success
just because a browser preview decoded. Validate bounds again at the WASM boundary. Attach the
wake listener once per runner and also drain during normal updates. Idle applications must request
a repaint when new bytes arrive. A successful acknowledgement stops later duplicate delivery.
Failed faces are retried explicitly, not in an endless repaint loop.

[`web/adapters/README.md`](../web/adapters/README.md) documents the four existing engine integrations
and contains the source patches as reference material. These patches were developed against a
customized web launcher and **are not promised to apply directly to current upstream app trees**.
They are not compiled or applied by this repository's build. Each app needs a separate reviewed
integration PR and browser render/export testing before claiming support in its own releases.

Deactivation/removal updates storage immediately. Existing native engine caches require saving
work and reloading the editor to release those fonts. Exported documents are not modified by
removing a font from browser storage.

## Discovery, uploads and backup

The bundled Font Library snapshot contains descriptions, licensing and source links, not font
binaries. Search text and preview text stay local. Previews fetch original TTF/OTF files from
HTTPS `fontlibrary.org/assets/fonts/` only, with omitted credentials/referrers and no redirects.
No external HTML, CSS or script is executed. Preview caches are bounded to 48 MiB and 48 decoded
faces. Imports preserve source/designer/licence metadata and use the same validation as uploads.
Automatic import filters recognized open licences. Metadata and OS/2 permission bits do not
replace the full licence terms. Keep original packages and notices for external fonts.

Uploads accept SFNT TTF/OTF files up to 24 MiB. WOFF/WOFF2 and collections need conversion.
Bounded table parsing, browser decoding and SHA-256 identity precede persistent installation.
Restricted, bitmap-only, preview/print-only and no-subsetting files are rejected because supported
editable document paths need editable embedding and subsetting. Active font bytes are capped at
256 MiB. IndexedDB write transactions enforce the budget across simultaneous imports.

Backup v1 stores original bytes and provenance in JSON. Restore validates every face before an
atomic merge. Existing live records win. Backup generation is bounded at 256 MiB of bytes and
restore at 360 MiB of JSON. Legacy `starcraft-font-library` backups are accepted, but the original
database is not renamed or automatically migrated. To reuse an existing launcher database, set
`databaseName: 'starcraft-font-library'`; schema version 2 preserves version-1 records. Requesting
persistent storage is an explicit user action. A browser can still clear storage: keep backups.

## Optional larger collection

`web/collections/catalog.lock.json` retains the previously built collection: 96 families and
254 faces from a pinned Google Fonts commit, each with file hashes, original licence and provenance.
It is separate from the default upstream manifest and adds no Cargo inputs or binaries to this PR.
Its full download is about 173 MB. Hosts choose whether to distribute or auto-activate it.

```sh
python3 -m pip install fonttools==4.61.1
node web/tools/build.mjs
python3 web/tools/acquire-fonts.py
# same .cache/web-fonts directory, catalog.json now selects the larger collection
# offline check, without downloading:
python3 web/tools/acquire-fonts.py --verify-only
```

Acquisition verifies pinned Git blob IDs, SHA-256, sizes, embedding permissions and licence text.
Never silently substitute a CDN file. `web/collections/CATALOG.md` records the family list.
Refresh discovery metadata with `python3 web/tools/refresh-fontlibrary.py --refresh`. Review new
licences and counts before committing. Interrupted metadata refreshes retain the prior snapshot.

## Validation and limits

Node tests cover discovery filters/download limits, DOM interactions, upload deduplication,
activation/favorites, cross-module recovery, backup/restore atomicity, embedding restrictions,
default opt-in/retries, subdirectory URL containment and byte-bridge delivery. They use the actual
fonts already in this checkout. Happy DOM and mocked FontFace test behavior, not font rasterization.

The standalone package does not prove multilingual shaping, UI fallback, variable axes, document
reopening on another device or every export format. Native editor verification remains an app-side
requirement. In particular, PDF forms/signatures/appearance generators are separate from added page
text. Font availability does not establish glyph coverage or correct shaping for every language.

All browser and tool code is MIT OR Apache-2.0, like the repository. Font binaries retain their own
licences. Build output ships the existing attribution and licence files. External imports stay on
the user's device. No telemetry or upload service is included.
