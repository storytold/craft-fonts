//! Assertions on the font files themselves.

use craft_fonts::{Entry, FONTS, Font, for_script, parse_manifest};
use sha2::{Digest, Sha256};
use skrifa::attribute::{Style, Weight};
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::tables::os2::SelectionFlags;
use skrifa::raw::{ReadError, TableProvider};
use skrifa::{FontRef, MetadataProvider, string::StringId};

fn bytes(f: &Font) -> Vec<u8> {
    std::fs::read(craft_fonts_tests::repo_root().join(f.file))
        .unwrap_or_else(|e| panic!("{}: {e}", f.file))
}

#[test]
fn every_file_matches_its_manifest_checksum() {
    for f in FONTS {
        let digest = Sha256::digest(bytes(f));
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex, f.sha256,
            "{}: the file changed; update the manifest only after checking the new upstream source",
            f.file
        );
    }
}

#[test]
fn every_font_ships_its_licence() {
    for f in FONTS {
        let text = std::fs::read_to_string(craft_fonts_tests::repo_root().join(f.licence_file))
            .unwrap_or_else(|e| panic!("{}: {e}", f.licence_file));
        if f.licence == "OFL-1.1" {
            assert!(
                text.contains("SIL OPEN FONT LICENSE Version 1.1"),
                "{} is not the OFL 1.1 text",
                f.licence_file
            );
        }
    }
}

#[test]
fn every_font_is_the_family_the_manifest_says() {
    for f in FONTS {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        let names: Vec<String> = [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME]
            .into_iter()
            .flat_map(|id| {
                font.localized_strings(id)
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(
            names.iter().any(|n| n == f.family),
            "{}: name table has {names:?}, manifest says {:?}",
            f.file,
            f.family
        );
    }
}

#[test]
fn every_font_is_the_style_the_manifest_says() {
    for f in FONTS {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        let names: Vec<String> = [
            StringId::SUBFAMILY_NAME,
            StringId::TYPOGRAPHIC_SUBFAMILY_NAME,
        ]
        .into_iter()
        .flat_map(|id| {
            font.localized_strings(id)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
        assert!(
            names.iter().any(|n| n == f.style),
            "{}: name table styles are {names:?}, manifest says {:?}",
            f.file,
            f.style
        );
    }
}

/// The exact manifest rows for the faces this prerequisite adds. The rows are pinned to the
/// verified upstream hashes above; a missing row or a drifted path/script/licence fails here
/// rather than silently skipping the Korean and symbol coverage below.
#[test]
fn prerequisite_faces_have_the_documented_manifest_rows() {
    fn exact(family: &str, style: &str, file: &str, scripts: &[&str]) -> &'static Font {
        let found: Vec<&'static Font> = FONTS.iter().filter(|f| f.family == family).collect();
        assert_eq!(
            found.len(),
            1,
            "manifest lists {} rows for {family}; expected exactly one",
            found.len()
        );
        let f = found[0];
        assert_eq!(f.style, style, "{}: manifest style", f.file);
        assert_eq!(f.file, file, "{family}: manifest file");
        assert_eq!(f.scripts, scripts, "{family}: manifest scripts");
        assert_eq!(f.licence, "OFL-1.1", "{family}: licence");
        f
    }

    exact(
        "NanumGothic",
        "Regular",
        "fonts/nanum-gothic/NanumGothic-Regular.ttf",
        &["Hang", "Kore"],
    );
    exact(
        "Noto Sans Symbols",
        "Regular",
        "fonts/noto-sans-symbols/NotoSansSymbols.ttf",
        &["Zsym"],
    );

    // Symbols 2 is required, not optional: the pinned Symbols 1 bytes lack U+2715 `✕` and
    // U+25C0 `◀`, which Symbols 2 provides.
    exact(
        "Noto Sans Symbols 2",
        "Regular",
        "fonts/noto-sans-symbols-2/NotoSansSymbols2-Regular.ttf",
        &["Zsym"],
    );
}

/// An outline pen that counts drawing commands, so a cmap entry with an empty outline cannot
/// pass as coverage.
struct OutlineCount(usize);

impl OutlinePen for OutlineCount {
    fn move_to(&mut self, _: f32, _: f32) {}
    fn line_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn close(&mut self) {}
}

/// Assert that `c` maps to a glyph that actually draws something in `font`.
fn assert_glyph_outline(font: &FontRef<'_>, file: &str, c: char) {
    let gid = font
        .charmap()
        .map(c)
        .unwrap_or_else(|| panic!("{file}: no glyph for {c}"));
    let glyph = font
        .outline_glyphs()
        .get(gid)
        .unwrap_or_else(|| panic!("{file}: no outline for {c}"));
    let mut pen = OutlineCount(0);
    glyph
        .draw(
            DrawSettings::unhinted(Size::new(64.0), LocationRef::default()),
            &mut pen,
        )
        .unwrap_or_else(|e| panic!("{file}: drawing {c}: {e}"));
    assert!(pen.0 > 0, "{file}: {c} maps to a glyph that draws nothing");
}

/// Assert that `f` itself (not merely some other face) maps `c` to a drawn outline.
fn assert_face_glyph_outline(f: &Font, c: char) {
    let data = bytes(f);
    let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
    assert_glyph_outline(&font, f.file, c);
}

/// The one manifest row for `family`.
fn face(family: &str) -> &'static Font {
    let mut found = FONTS.iter().filter(|f| f.family == family);
    let f = found
        .next()
        .unwrap_or_else(|| panic!("the manifest lists no {family} row"));
    assert!(
        found.next().is_none(),
        "the manifest lists more than one {family} row"
    );
    f
}

/// Text every Japanese font must cover: kana, common kanji, Japanese punctuation, full-width
/// forms and basic Latin (UI strings mix them).
const JAPANESE: &str = concat!(
    "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんがぎぐげござじずぜぞだぢづでどばびぶべぼぱぴぷぺぽぁぃぅぇぉっゃゅょ",
    "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲンガギグゲゴヴァィゥェォッャュョー",
    "日本語漢字東京都新見書読学校会社電話開閉保存編集表示画像文字色線形層効果時間写真映画印刷設定終了取消削除選択複製",
    "、。「」『』・…！？（）：；〜",
    "０１２３４５６７８９ＡＢＣＤＥＦＧＨＩＪＫＬＭＮＯＰＱＲＳＴＵＶＷＸＹＺ",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
);

#[test]
fn japanese_fonts_cover_japanese_text() {
    let jpan: Vec<&Font> = for_script("Jpan").collect();
    assert!(!jpan.is_empty(), "the manifest lists no Japanese font");
    for f in jpan {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        let charmap = font.charmap();
        let missing: String = JAPANESE
            .chars()
            .filter(|c| charmap.map(*c).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "{} {} has no glyph for {missing:?}",
            f.family,
            f.style
        );
    }
}

#[test]
fn japanese_glyphs_have_outlines() {
    for f in for_script("Jpan") {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        for c in ['漢', 'あ', 'ア', '。'] {
            assert_glyph_outline(&font, f.file, c);
        }
    }
}

#[test]
fn japanese_fonts_support_vertical_text() {
    // Vertical type (tategaki) needs the vertical alternates for punctuation and long vowels.
    for f in for_script("Jpan") {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        let gsub = font
            .gsub()
            .unwrap_or_else(|e| panic!("{}: no GSUB: {e}", f.file));
        let features = gsub
            .feature_list()
            .unwrap_or_else(|e| panic!("{}: GSUB features: {e}", f.file));
        let tags: Vec<String> = features
            .feature_records()
            .iter()
            .map(|r| r.feature_tag().to_string())
            .collect();
        assert!(
            tags.iter().any(|t| t == "vert" || t == "vrt2"),
            "{} {}: no vert/vrt2 GSUB feature (has {tags:?})",
            f.family,
            f.style
        );
    }
}

/// Coverage of `text` by every font tagged with `script`.
fn covers(script: &str, text: &str) {
    let fonts: Vec<&Font> = for_script(script).collect();
    assert!(!fonts.is_empty(), "the manifest lists no {script} font");
    for f in fonts {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        let charmap = font.charmap();
        let missing: String = text.chars().filter(|c| charmap.map(*c).is_none()).collect();
        assert!(
            missing.is_empty(),
            "{} {} has no glyph for {missing:?}",
            f.family,
            f.style
        );
    }
}

#[test]
fn simplified_chinese_fonts_cover_chinese_text() {
    covers(
        "Hans",
        concat!(
            "中文简体字文件编辑视图窗口帮助新建打开保存关闭撤销重做复制粘贴删除选择图层页面样式颜色导出打印设置",
            "，。、：；？！（）《》“”…",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
        ),
    );
}

#[test]
fn arabic_fonts_cover_arabic_text() {
    covers(
        "Arab",
        concat!(
            "ابتثجحخدذرزسشصضطظعغفقكلمنهوي",
            "ءآأؤإئةى",
            "٠١٢٣٤٥٦٧٨٩",
            "،؛؟",
        ),
    );
}

#[test]
fn thai_fonts_cover_thai_text() {
    covers(
        "Thai",
        concat!(
            "กขฃคฅฆงจฉชซฌญฎฏฐฑฒณดตถทธนบปผฝพฟภมยรฤลฦวศษสหฬอฮ",
            "ะัาำิีึืุูเแโใไๅ็่้๊๋์",
            "๐๑๒๓๔๕๖๗๘๙",
            "ฯๆ฿",
        ),
    );
}

#[test]
fn georgian_fonts_cover_georgian_text() {
    covers(
        "Geor",
        concat!(
            "აბგდევზთიკლმნოპჟრსტუფქღყშჩცძწჭხჯჰ",
            "ᲐᲑᲒᲓᲔᲕᲖᲗᲘᲙᲚᲛᲜᲝᲞᲟᲠᲡᲢᲣᲤᲥᲦᲧᲨᲩᲪᲫᲬᲭᲮᲯᲰ",
            "„“–№",
        ),
    );
}

/// Korean interface labels used by the GridCraft UI, not a single sample character.
const KOREAN_UI: &str = concat!(
    "데이터 한글 파일 편집 보기 삽입 서식 저장 열기 닫기 인쇄 취소 확인 ",
    "가나다라마바사아자차카타파하",
);

#[test]
fn korean_fonts_cover_korean_ui_text() {
    covers("Hang", KOREAN_UI);
    covers("Kore", KOREAN_UI);
}

/// The Korean `Hang` face really covers all modern Hangul syllables and the 51 compatibility
/// Jamo. These exact counts come from the pinned NanumGothic cmap, not from a sample string.
#[test]
fn korean_fonts_cover_every_modern_syllable_and_compatibility_jamo() {
    let syllables = '\u{ac00}'..='\u{d7a3}';
    let jamo = '\u{3131}'..='\u{3163}';
    assert_eq!(syllables.clone().count(), 11_172);
    assert_eq!(jamo.clone().count(), 51);

    let fonts: Vec<&Font> = for_script("Hang").collect();
    assert!(!fonts.is_empty(), "the manifest lists no Hang font");
    for f in fonts {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        let charmap = font.charmap();
        let missing_syllables = syllables
            .clone()
            .filter(|c| charmap.map(*c).is_none())
            .count();
        let missing_jamo = jamo.clone().filter(|c| charmap.map(*c).is_none()).count();
        assert_eq!(
            missing_syllables, 0,
            "{}: {missing_syllables} of 11,172 modern Hangul syllables are missing",
            f.file
        );
        assert_eq!(
            missing_jamo, 0,
            "{}: {missing_jamo} of the 51 compatibility Jamo are missing",
            f.file
        );
    }
}

#[test]
fn korean_glyphs_have_outlines() {
    let fonts: Vec<&Font> = for_script("Hang").collect();
    assert!(
        !fonts.is_empty(),
        "the manifest lists no Hang font; add NanumGothic Regular from the pinned source in docs/egui-037-font-coverage-plan.md"
    );
    for f in fonts {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        for c in ['한', '글', '각', '힣'] {
            assert_glyph_outline(&font, f.file, c);
        }
    }
}

/// Exact face assignment for the observed symbol gaps. Symbols 1 covers `⌃` and two of the
/// arrows but not `✕`, `◀` or `✓`; Symbols 2 covers `✕`, `◀` and `✓` but not `⌃`. `📋` is
/// supplied by egui's emoji faces, so it is intentionally not asserted against craft-fonts.
#[test]
fn zsym_faces_cover_the_documented_symbol_gaps() {
    let symbols = face("Noto Sans Symbols");
    let symbols2 = face("Noto Sans Symbols 2");
    assert_eq!(
        symbols.scripts,
        ["Zsym"].as_slice(),
        "{}: scripts",
        symbols.file
    );
    assert_eq!(
        symbols2.scripts,
        ["Zsym"].as_slice(),
        "{}: scripts",
        symbols2.file
    );

    for c in ['⌃', '↘', '↑'] {
        assert_face_glyph_outline(symbols, c);
    }
    for c in ['✕', '◀', '✓'] {
        assert_face_glyph_outline(symbols2, c);
    }
}

/// The Hebrew face is the verified upstream static Regular release, not the old variable
/// fixture. It must remain one face/file: no `fvar` (and no named instances), no `avar`, no
/// `STAT`, OS/2 weight 400 and an upright `REGULAR` selection. A consumer registers this one
/// face once per family set that includes it, never the nine instances of the variable font.
#[test]
fn hebrew_face_is_a_single_static_regular_face() {
    const HEBREW_FILE: &str = "fonts/noto-sans-hebrew/NotoSansHebrew-Regular.ttf";
    const HEBREW_LICENCE_FILE: &str = "fonts/noto-sans-hebrew/OFL.txt";
    const HEBREW_SOURCE: &str =
        "https://github.com/notofonts/hebrew/releases/tag/NotoSansHebrew-v3.001";
    const HEBREW_SHA256: &str = "671951828bd5c95db818e5bb12dcea2d0c0dda00311888522be061ee6835125e";

    let f = face("Noto Sans Hebrew");
    assert_eq!(for_script("Hebr").count(), 1);
    assert_eq!(f.style, "Regular");
    assert_eq!(f.file, HEBREW_FILE);
    assert_eq!(f.scripts, ["Hebr"].as_slice());
    assert_eq!(f.licence, "OFL-1.1");
    assert_eq!(f.licence_file, HEBREW_LICENCE_FILE);
    assert_eq!(f.source, HEBREW_SOURCE);
    assert_eq!(f.sha256, HEBREW_SHA256);

    let data = bytes(f);
    // A TrueType sfnt, not a TrueType Collection: one face in the file.
    assert_eq!(&data[..4], &[0x00, 0x01, 0x00, 0x00]);
    let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
    for (name, table) in [
        ("fvar", font.fvar().map(|_| ())),
        ("avar", font.avar().map(|_| ())),
        ("STAT", font.stat().map(|_| ())),
    ] {
        assert!(
            matches!(table, Err(ReadError::TableIsMissing(_))),
            "{}: expected no {name} table; the Hebrew file must stay a single static face",
            f.file
        );
    }

    let os2 = font
        .os2()
        .unwrap_or_else(|e| panic!("{}: OS/2: {e}", f.file));
    assert_eq!(os2.us_weight_class(), 400, "{}: OS/2 weight", f.file);
    assert_eq!(os2.us_width_class(), 5, "{}: OS/2 width", f.file);
    let selection = os2.fs_selection();
    assert!(
        selection.contains(SelectionFlags::REGULAR),
        "{}: fsSelection lacks REGULAR",
        f.file
    );
    for flag in [
        SelectionFlags::ITALIC,
        SelectionFlags::BOLD,
        SelectionFlags::OBLIQUE,
    ] {
        assert!(
            !selection.contains(flag),
            "{}: fsSelection unexpectedly contains {flag:?}",
            f.file
        );
    }

    let attributes = font.attributes();
    assert_eq!(attributes.weight, Weight::NORMAL, "{}: weight", f.file);
    assert_eq!(attributes.style, Style::Normal, "{}: style", f.file);
}

/// All 27 Hebrew letters `U+05D0..U+05EA` map to drawn outlines, not merely cmap entries.
#[test]
fn hebrew_letters_have_drawn_outlines() {
    let f = face("Noto Sans Hebrew");
    let letters = '\u{05d0}'..='\u{05ea}';
    assert_eq!(letters.clone().count(), 27);
    for c in letters {
        assert_face_glyph_outline(f, c);
    }
}

/// All 24 assigned points, marks and punctuation in `U+05B0..U+05C7` map to drawn outlines.
/// The combining marks require GPOS positioning in a shaping engine. This font-only test
/// asserts coverage, not application-level niqqud positioning or bidi order.
#[test]
fn hebrew_points_have_drawn_outlines() {
    let f = face("Noto Sans Hebrew");
    let points = '\u{05b0}'..='\u{05c7}';
    assert_eq!(points.clone().count(), 24);
    for c in points {
        assert_face_glyph_outline(f, c);
    }
}

/// The unpointed Hebrew sample used by mixed acceptance text such as `שלום deck 42`.
/// craft-fonts only claims the Hebrew part here, which must map to drawn outlines.
#[test]
fn hebrew_fonts_cover_unpointed_hebrew_text() {
    const HEBREW_SAMPLE: &str = "שלום עברית אבגדהוזחטיכלמנסעפצקרשת";
    covers("Hebr", HEBREW_SAMPLE);
    let f = face("Noto Sans Hebrew");
    for c in "שלום".chars() {
        assert_face_glyph_outline(f, c);
    }
}

/// The Devanagari face is the verified upstream static Regular release, added after Hebrew so
/// existing manifest rows and consumer `craft-font-N` indices do not move. It is the single
/// `Deva` face in the default manifest; the optional Chinese manifest stays independent.
const DEVANAGARI_FILE: &str = "fonts/noto-sans-devanagari/NotoSansDevanagari-Regular.ttf";
const DEVANAGARI_LICENCE_FILE: &str = "fonts/noto-sans-devanagari/OFL.txt";
const DEVANAGARI_SOURCE: &str =
    "https://github.com/notofonts/devanagari/releases/tag/NotoSansDevanagari-v2.007";
const DEVANAGARI_SHA256: &str = "573e525d9f4923d34645fe752475419f45f8dec5a8e43f3e83c5c5f43d1f2ff9";
const DEVANAGARI_LICENCE_SHA256: &str =
    "a216f6f8d85c7228093e0ee5e258d9d377e6671f68acb4db1930b29583d0f331";

#[test]
fn devanagari_row_is_exact_and_independent_of_the_optional_chinese_manifest() {
    let f = face("Noto Sans Devanagari");
    assert_eq!(
        for_script("Deva").count(),
        1,
        "the default manifest must list exactly one Deva face"
    );
    assert_eq!(f.style, "Regular");
    assert_eq!(f.file, DEVANAGARI_FILE);
    assert_eq!(f.scripts, ["Deva"].as_slice());
    assert_eq!(f.licence, "OFL-1.1");
    assert_eq!(f.licence_file, DEVANAGARI_LICENCE_FILE);
    assert_eq!(f.sha256, DEVANAGARI_SHA256);
    assert_eq!(f.source, DEVANAGARI_SOURCE);

    let optional = optional_entries(SIMPLIFIED_CHINESE_MANIFEST);
    assert!(
        !optional
            .iter()
            .any(|e| e.family == "Noto Sans Devanagari" || e.file == DEVANAGARI_FILE),
        "the opt-in Chinese manifest must not gain the default Devanagari face"
    );
}

#[test]
fn devanagari_file_and_licence_match_hashes_and_sizes() {
    let f = face("Noto Sans Devanagari");
    let data = bytes(f);
    assert_eq!(
        data.len(),
        277_680,
        "{}: the pinned Devanagari font is 277,680 bytes",
        f.file
    );
    let digest = Sha256::digest(&data);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(hex, DEVANAGARI_SHA256, "{}: font SHA-256", f.file);

    let licence = std::fs::read(craft_fonts_tests::repo_root().join(f.licence_file))
        .unwrap_or_else(|e| panic!("{}: {e}", f.licence_file));
    assert_eq!(
        licence.len(),
        4_386,
        "{}: the archive-root OFL text is 4,386 bytes",
        f.licence_file
    );
    let digest = Sha256::digest(&licence);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex, DEVANAGARI_LICENCE_SHA256,
        "{}: licence SHA-256",
        f.licence_file
    );
    let text = String::from_utf8_lossy(&licence);
    assert!(
        text.contains("SIL OPEN FONT LICENSE Version 1.1"),
        "{} is not the OFL 1.1 text",
        f.licence_file
    );
}

#[test]
fn devanagari_face_is_a_single_static_regular_face() {
    let f = face("Noto Sans Devanagari");
    let data = bytes(f);
    // A TrueType sfnt, not a TrueType Collection: one face in the file.
    assert_eq!(&data[..4], &[0x00, 0x01, 0x00, 0x00]);
    let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
    for (name, table) in [
        ("fvar", font.fvar().map(|_| ())),
        ("avar", font.avar().map(|_| ())),
        ("STAT", font.stat().map(|_| ())),
    ] {
        assert!(
            matches!(table, Err(ReadError::TableIsMissing(_))),
            "{}: expected no {name} table; the Devanagari file must stay a single static face",
            f.file
        );
    }

    let versions: Vec<String> = font
        .localized_strings(StringId::VERSION_STRING)
        .map(|s| s.to_string())
        .collect();
    assert!(
        versions.iter().any(|v| v.contains("Version 2.007")),
        "{}: expected Version 2.007, found {versions:?}",
        f.file
    );

    let os2 = font
        .os2()
        .unwrap_or_else(|e| panic!("{}: OS/2: {e}", f.file));
    assert_eq!(os2.us_weight_class(), 400, "{}: OS/2 weight", f.file);
    assert_eq!(os2.us_width_class(), 5, "{}: OS/2 width", f.file);
    let selection = os2.fs_selection();
    assert!(
        selection.contains(SelectionFlags::REGULAR),
        "{}: fsSelection lacks REGULAR",
        f.file
    );
    for flag in [
        SelectionFlags::ITALIC,
        SelectionFlags::BOLD,
        SelectionFlags::OBLIQUE,
    ] {
        assert!(
            !selection.contains(flag),
            "{}: fsSelection unexpectedly contains {flag:?}",
            f.file
        );
    }

    let attributes = font.attributes();
    assert_eq!(attributes.weight, Weight::NORMAL, "{}: weight", f.file);
    assert_eq!(attributes.style, Style::Normal, "{}: style", f.file);
}

/// The observed failing sequence `फ़ा` (U+092B U+093C U+093E), the Hindi Camera Raw sample
/// and the individual letter/mark pieces must all map to drawn outlines. This is glyph
/// coverage, not proof of Devanagari shaping, conjunct formation or mark positioning; the
/// PhotoCraft Camera Raw test exercises the real shaping path.
#[test]
fn devanagari_glyphs_and_hindi_sample_have_drawn_outlines() {
    let f = face("Noto Sans Devanagari");
    let failing_sequence = "फ़ा";
    assert_eq!(
        failing_sequence.chars().map(u32::from).collect::<Vec<_>>(),
        [0x092b, 0x093c, 0x093e],
        "the regression sequence is U+092B U+093C U+093E"
    );
    for c in ['फ', '़', 'ा'] {
        assert_face_glyph_outline(f, c);
    }
    for c in failing_sequence.chars() {
        assert_face_glyph_outline(f, c);
    }

    const HINDI_SAMPLE: &str = "हिन्दी भाषा क्षत्रिय श्री फ़ाइल फ़िल्टर";
    let data = bytes(f);
    let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
    assert!(
        font.charmap().map(' ').is_some(),
        "{}: no space glyph in the Hindi sample",
        f.file
    );
    for c in HINDI_SAMPLE.chars() {
        if c.is_whitespace() {
            continue;
        }
        assert_face_glyph_outline(f, c);
    }
}

/// Every code point in the Devanagari block `U+0900..=U+097F` maps to a glyph that draws
/// something in the pinned file. Again this is coverage, not shaping/positioning.
#[test]
fn devanagari_full_block_maps_to_drawn_outlines() {
    let f = face("Noto Sans Devanagari");
    let block = '\u{0900}'..='\u{097f}';
    assert_eq!(block.clone().count(), 128);
    let data = bytes(f);
    let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
    for c in block {
        assert_glyph_outline(&font, f.file, c);
    }
}

/// The opt-in Simplified Chinese manifest. It is deliberately separate from
/// `fonts/manifest.txt`: consumers that do not opt in (including every wasm build) must not
/// embed this face, and it is not part of craft-fonts' `FONTS`/embed payload.
const SIMPLIFIED_CHINESE_MANIFEST: &str = "fonts/simplified-chinese-manifest.txt";
const LXGW_FILE: &str = "fonts/lxgw-wenkai-lite/LXGWWenKaiLite-Regular.ttf";
const LXGW_LICENCE_FILE: &str = "fonts/lxgw-wenkai-lite/OFL.txt";
const LXGW_SOURCE: &str = "https://github.com/lxgw/LxgwWenKai-Lite/releases/tag/v1.530";
const LXGW_SHA256: &str = "f799a538a3355877e597a0ae04050e897932c9ae7689eb9227c2953c5dd549ea";
/// The plan's hard ceiling for one added font file.
const LXGW_MAX_BYTES: usize = 24 * 1024 * 1024;
/// The DeckCraft UI sample whose missing `简` U+7B80 triggered this opt-in face.
const DECK_CHINESE_SAMPLE: &str = "简体中文演示文稿";

fn optional_entries(relative: &str) -> Vec<Entry> {
    let path = craft_fonts_tests::repo_root().join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    parse_manifest(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn lxgw_entry() -> Entry {
    let entries = optional_entries(SIMPLIFIED_CHINESE_MANIFEST);
    assert_eq!(
        entries.len(),
        1,
        "{SIMPLIFIED_CHINESE_MANIFEST} must hold exactly the one reviewed LXGW row"
    );
    entries.into_iter().next().unwrap()
}

#[test]
fn simplified_chinese_opt_in_row_is_exact_and_not_in_the_default_manifest() {
    let e = lxgw_entry();
    assert_eq!(e.family, "LXGW WenKai Lite");
    assert_eq!(e.style, "Regular");
    assert_eq!(e.file, LXGW_FILE);
    assert_eq!(
        e.scripts.iter().map(String::as_str).collect::<Vec<_>>(),
        ["Hani", "Hans"],
        "DeckCraft's script classifier emits Hani; Hans names the Simplified Chinese face"
    );
    assert_eq!(e.licence, "OFL-1.1");
    assert_eq!(e.licence_file, LXGW_LICENCE_FILE);
    assert_eq!(e.sha256, LXGW_SHA256);
    assert_eq!(e.source, LXGW_SOURCE);

    assert!(
        !FONTS.iter().any(|f| f.family == e.family.as_str()),
        "the opt-in LXGW face must not be in the default FONTS table or embed payload"
    );
    let default =
        std::fs::read_to_string(craft_fonts_tests::repo_root().join("fonts/manifest.txt")).unwrap();
    assert!(
        !default.contains("LXGW") && !default.contains("simplified-chinese"),
        "the default font manifest must not list the opt-in Chinese face"
    );
}

#[test]
fn simplified_chinese_face_matches_its_hash_licence_and_size_ceiling() {
    let e = lxgw_entry();
    let data = bytes_from_entry(&e);
    let digest = Sha256::digest(&data);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex, e.sha256,
        "{}: the file changed; update the opt-in manifest only after checking the new upstream source",
        e.file
    );
    assert!(
        data.len() <= LXGW_MAX_BYTES,
        "{}: {} bytes exceeds the 24 MiB single-face ceiling",
        e.file,
        data.len()
    );
    assert_eq!(
        std::path::Path::new(&e.file).parent(),
        std::path::Path::new(&e.licence_file).parent(),
        "the licence must sit next to the font it covers"
    );
    let licence = std::fs::read_to_string(craft_fonts_tests::repo_root().join(&e.licence_file))
        .unwrap_or_else(|err| panic!("{}: {err}", e.licence_file));
    assert!(licence.contains("SIL OPEN FONT LICENSE Version 1.1"));
}

#[test]
fn simplified_chinese_licence_retains_klee_copyright_and_reserved_font_names() {
    let e = lxgw_entry();
    let licence = std::fs::read_to_string(craft_fonts_tests::repo_root().join(&e.licence_file))
        .unwrap_or_else(|err| panic!("{}: {err}", e.licence_file));
    assert!(licence.contains("Reserved Font Name"));
    assert!(licence.contains("Copyright 2020 The Klee Project Authors"));
    for name in ["霞鹜", "霞鶩", "落霞孤鹜", "落霞孤鶩", "LXGW"] {
        assert!(
            licence.contains(name),
            "{}: the upstream Reserved Font Name {name} is missing",
            e.licence_file
        );
    }
}

#[test]
fn simplified_chinese_face_is_one_static_regular_400_face() {
    let e = lxgw_entry();
    let data = bytes_from_entry(&e);
    assert_eq!(
        &data[..4],
        &[0x00, 0x01, 0x00, 0x00],
        "{}: expected a single TrueType sfnt, not a collection or variable font",
        e.file
    );
    let font = FontRef::new(&data).unwrap_or_else(|err| panic!("{}: {err}", e.file));
    for (name, table) in [
        ("fvar", font.fvar().map(|_| ())),
        ("avar", font.avar().map(|_| ())),
        ("STAT", font.stat().map(|_| ())),
    ] {
        assert!(
            matches!(table, Err(ReadError::TableIsMissing(_))),
            "{}: expected no {name} table; the opt-in file must stay a single static face",
            e.file
        );
    }

    let families: Vec<String> = [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME]
        .into_iter()
        .flat_map(|id| {
            font.localized_strings(id)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        families.iter().any(|n| n.as_str() == e.family),
        "{}: name table has {families:?}, manifest says {:?}",
        e.file,
        e.family
    );
    let styles: Vec<String> = [
        StringId::SUBFAMILY_NAME,
        StringId::TYPOGRAPHIC_SUBFAMILY_NAME,
    ]
    .into_iter()
    .flat_map(|id| {
        font.localized_strings(id)
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    })
    .collect();
    assert!(
        styles.iter().any(|n| n.as_str() == e.style),
        "{}: name table styles are {styles:?}, manifest says {:?}",
        e.file,
        e.style
    );
    let versions: Vec<String> = font
        .localized_strings(StringId::VERSION_STRING)
        .map(|s| s.to_string())
        .collect();
    assert!(
        versions.iter().any(|v| v.contains("Version 1.530")),
        "{}: expected Version 1.530, found {versions:?}",
        e.file
    );

    let os2 = font
        .os2()
        .unwrap_or_else(|err| panic!("{}: OS/2: {err}", e.file));
    assert_eq!(os2.us_weight_class(), 400, "{}: OS/2 weight", e.file);
    let selection = os2.fs_selection();
    assert!(
        selection.contains(SelectionFlags::REGULAR),
        "{}: fsSelection lacks REGULAR",
        e.file
    );
    for flag in [
        SelectionFlags::ITALIC,
        SelectionFlags::BOLD,
        SelectionFlags::OBLIQUE,
    ] {
        assert!(
            !selection.contains(flag),
            "{}: fsSelection unexpectedly contains {flag:?}",
            e.file
        );
    }
    let attributes = font.attributes();
    assert_eq!(attributes.weight, Weight::NORMAL, "{}: weight", e.file);
    assert_eq!(attributes.style, Style::Normal, "{}: style", e.file);
}

#[test]
fn simplified_chinese_face_draws_the_deckcraft_sample_and_jian() {
    let e = lxgw_entry();
    let data = bytes_from_entry(&e);
    let font = FontRef::new(&data).unwrap_or_else(|err| panic!("{}: {err}", e.file));
    assert_eq!(DECK_CHINESE_SAMPLE.chars().count(), 8);
    assert!(DECK_CHINESE_SAMPLE.contains('简'));
    for c in DECK_CHINESE_SAMPLE.chars() {
        assert_glyph_outline(&font, &e.file, c);
    }
    assert_glyph_outline(&font, &e.file, '简');
}

fn bytes_from_entry(e: &Entry) -> Vec<u8> {
    std::fs::read(craft_fonts_tests::repo_root().join(&e.file))
        .unwrap_or_else(|err| panic!("{}: {err}", e.file))
}
