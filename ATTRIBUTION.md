# Font attribution

Every font in `fonts/`, with its source and licence. The files are byte-identical to the pinned
upstream sources (checked by `craft-fonts-tests`, which verifies each SHA-256 in
`fonts/manifest.txt`).

| File | Family / style | Author | Source (pinned) | Licence |
|---|---|---|---|---|
| `fonts/shippori-mincho/ShipporiMincho-Regular.ttf` | Shippori Mincho Regular | The Shippori Mincho Project Authors (FONTDASU) | [google/fonts@9710da1](https://github.com/google/fonts/tree/9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/shipporimincho) | OFL-1.1 (`fonts/shippori-mincho/OFL.txt`) |
| `fonts/biz-ud-pgothic/BIZUDPGothic-Regular.ttf` | BIZ UDPGothic Regular | The BIZ UDGothic Project Authors (Morisawa Inc.) | [googlefonts/morisawa-biz-ud-gothic@18934af](https://github.com/googlefonts/morisawa-biz-ud-gothic/tree/18934af56b9c003ca58c54bffbf226848cb11032) | OFL-1.1 (`fonts/biz-ud-pgothic/OFL.txt`) |
| `fonts/biz-ud-pgothic/BIZUDPGothic-Bold.ttf` | BIZ UDPGothic Bold | The BIZ UDGothic Project Authors (Morisawa Inc.) | [googlefonts/morisawa-biz-ud-gothic@18934af](https://github.com/googlefonts/morisawa-biz-ud-gothic/tree/18934af56b9c003ca58c54bffbf226848cb11032) | OFL-1.1 (`fonts/biz-ud-pgothic/OFL.txt`) |
| `fonts/biz-ud-mincho/BIZUDMincho-Regular.ttf` | BIZ UDMincho Regular | The BIZ UDMincho Project Authors (Morisawa Inc., design TypeBank Co., Ltd.) | [google/fonts@63833b7](https://github.com/google/fonts/tree/63833b7d10bb1f86a8f0b91cba2d3ae1f68d1aa3/ofl/bizudmincho) | OFL-1.1 (`fonts/biz-ud-mincho/OFL.txt`) |
| `fonts/noto-sans-cjk-sc/NotoSansCJKsc-Regular.otf` | Noto Sans CJK SC Regular | The Noto Project Authors (Adobe, Google) | [notofonts/noto-cjk@f8d1575](https://github.com/notofonts/noto-cjk/tree/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/OTF/SimplifiedChinese) | OFL-1.1 (`fonts/noto-sans-cjk-sc/OFL.txt`) |
| `fonts/noto-sans-arabic/NotoSansArabic.ttf` | Noto Sans Arabic (variable) | The Noto Project Authors | [google/fonts@7085eb8](https://github.com/google/fonts/tree/7085eb89a950e85db5b166b7a58d414544b4140c/ofl/notosansarabic) | OFL-1.1 (`fonts/noto-sans-arabic/OFL.txt`) |
| `fonts/droid-sans-fallback/DroidSansFallbackFull.ttf` | Droid Sans Fallback Regular | The Android Open Source Project (digitized data © Google, design Ascender Corporation) | [platform/frameworks/base@99b01a6](https://android.googlesource.com/platform/frameworks/base/+/99b01a65cc4c104933788b3143285ab6bae65827/data/fonts) (`android-16.0.0_r1`; the same file since Android 10) | Apache-2.0 (`fonts/droid-sans-fallback/NOTICE`) |

The OFL allows these fonts to be bundled, embedded and redistributed with software, provided the
licence text travels with them and the fonts are not sold on their own. The Apache License 2.0
(Droid Sans Fallback) allows the same, provided the licence and the upstream `NOTICE` travel with
the font: `fonts/droid-sans-fallback/NOTICE` is that file, byte-identical to upstream, and holds
both the copyright notice and the licence text. "Shippori Mincho", "BIZ UD", "Noto" and "Droid"
are the authors' names for their fonts ("Droid" is a trademark of Google); we use the files
unmodified.
