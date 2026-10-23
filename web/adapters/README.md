# Native editor integration references

The `.patch` files retain the font adapters already developed in
[mmmbeer/starcraft](https://github.com/mmmbeer/starcraft). They are supplied for review and porting,
not automatically applied. Their source snapshot is launcher commit
`0dd12269d7b72780022efc64c92e4a6b8319bb8e`. That launcher includes other customization patches
(session/preset hosts, clipboard and UI changes), so these diffs can require context adaptation.
The original underlying app revisions were:

| App | Underlying source commit |
|---|---|
| photocraft | `67e34a1d4cea9bfa66056a5a1ca7466a4684c827` |
| vectorcraft | `2ddbd9293f5c65d95f072e5ab00584491e62aa65` |
| pdfcraft | `b8ddf71e30959dc8a5ca4b66c40b8fcaea68cf76` |
| lightcraft | `964d9338bafac524146ecf24a1fab54bd22b052e` |

| App | Registration path and intended scope |
|---|---|
| PhotoCraft | WebShell drains the byte bridge into served_fonts::install and checks the shared text FontDb. Editable type and raster exports. |
| VectorCraft | Register font bytes in the global FontDb used by document text, outlines and exporters. |
| PDFCraft | Added page text selects a stored face ID. HarfRust/bidi layout and bounded Type3 outlines with ToUnicode. Existing arbitrary PDF text and independent appearance generators are outside scope. |
| LightCraft | Select a stored face ID for export watermarks and contact-sheet captions. Existing raster text behavior remains, without new complex-script shaping. |

For a minimal PhotoCraft/VectorCraft adoption, port only the take/ack/wake functions and native
registration calls. PDFCraft/LightCraft need face selectors and persistence too. The bridge is
optional: absent CraftFonts means an empty batch, preserving existing builds and fonts.
Check bridge version 1 at the app boundary when porting these references. Bound incoming buffers
and family/ID fields. Use a listener tied to the runner lifecycle where available rather than
leaking listeners across repeated runner starts.

The patches are evidence and reusable source, not app-side PRs. This repository's tests do not
compile them. Before submitting to each app, rebase on its current source, run its native/WASM
checks, then test upload in one editor, use in another, refresh recovery, selected styles, save,
export and reopening. Test every script/format claimed by that app. Removal requires editor reload.

Do not treat the reference patches as proof of current upstream support. The portable package and
its storage/bridge are tested separately in `web/tests`.
