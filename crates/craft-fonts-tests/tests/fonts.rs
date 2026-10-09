//! Assertions on the font files themselves.

use craft_fonts::{FONTS, Font, for_script};
use sha2::{Digest, Sha256};
use skrifa::raw::TableProvider;
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
    use skrifa::instance::{LocationRef, Size};
    use skrifa::outline::{DrawSettings, OutlinePen};
    struct Count(usize);
    impl OutlinePen for Count {
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
    for f in for_script("Jpan") {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        let outlines = font.outline_glyphs();
        for c in ['漢', 'あ', 'ア', '。'] {
            let gid = font
                .charmap()
                .map(c)
                .unwrap_or_else(|| panic!("{}: no glyph for {c}", f.file));
            let glyph = outlines
                .get(gid)
                .unwrap_or_else(|| panic!("{}: no outline for {c}", f.file));
            let mut pen = Count(0);
            glyph
                .draw(
                    DrawSettings::unhinted(Size::new(64.0), LocationRef::default()),
                    &mut pen,
                )
                .unwrap_or_else(|e| panic!("{}: drawing {c}: {e}", f.file));
            assert!(pen.0 > 0, "{} {}: {c} draws nothing", f.family, f.style);
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
fn arabic_fonts_join_letters() {
    // Arabic letters take initial, medial and final forms; a font without these GSUB features
    // draws every letter isolated, which reads as broken text.
    let arab: Vec<&Font> = for_script("Arab").collect();
    assert!(!arab.is_empty(), "the manifest lists no Arabic font");
    for f in arab {
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
        for needed in ["init", "medi", "fina"] {
            assert!(
                tags.iter().any(|t| t == needed),
                "{} {}: no {needed} GSUB feature (has {tags:?})",
                f.family,
                f.style
            );
        }
    }
}

#[test]
fn noto_sans_arabic_is_the_first_arabic_font() {
    // The apps take the first Arabic font in manifest order as the fallback for Arabic text.
    let first = for_script("Arab")
        .next()
        .unwrap_or_else(|| panic!("the manifest lists no Arabic font"));
    assert_eq!(
        (first.family, first.style),
        ("Noto Sans Arabic", "Regular"),
        "keep Noto Sans Arabic as the first Arab line in fonts/manifest.txt"
    );
}
