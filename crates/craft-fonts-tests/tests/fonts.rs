//! Assertions on the font files themselves.

use craft_fonts::{FONTS, Font, ROLE_UI, for_script};
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
    covered_by(script, for_script(script), text);
}

/// Coverage of `text` by each of `fonts` (`what` names them in the failure message).
fn covered_by<'a>(what: &str, fonts: impl Iterator<Item = &'a Font>, text: &str) {
    let fonts: Vec<&Font> = fonts.collect();
    assert!(!fonts.is_empty(), "the manifest lists no {what} font");
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

#[test]
fn armenian_fonts_cover_armenian_text() {
    covers(
        "Armn",
        concat!(
            "ԱԲԳԴԵԶԷԸԹԺԻԼԽԾԿՀՁՂՃՄՅՆՇՈՉՊՋՌՍՎՏՐՑՒՓՔՕՖ",
            "աբգդեզէըթժիլխծկհձղճմյնշոչպջռսվտրցւփքօֆև",
            "։՝՛՞֊«»",
        ),
    );
}

#[test]
fn ethiopic_fonts_cover_ethiopic_text() {
    covers(
        "Ethi",
        concat!(
            "ሀሁሂሃሄህሆለሉሊላሌልሎሐመሙሚማሜምሞሠረሩሪራሬርሮሰሱሲሳሴስሶሸቀቁቂቃቄቅቆበቡቢባ",
            "ቤብቦቨተቱቲታቴትቶቸኀነኑኒናኔንኖኘአኡኢኣኤእኦከኸወዐዘዠየደጀገጠጨጰጸፀፈፐ",
            "።፣፤፥፦፧፨፩፪፫፬፭፮፯፰፱፲፻",
        ),
    );
}

#[test]
fn cyrillic_fonts_cover_cyrillic_text() {
    covers(
        "Cyrl",
        concat!(
            "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя",
            "ЄІЇҐєіїґ",         // Ukrainian
            "ЂЈЉЊЋЏђјљњћџ",     // Serbian
            "ѐѝ",               // Bulgarian
            "ӘҒҚҢӨҰҮҺәғқңөұүһ", // Kazakh
            "№«»„“₴₽₸",
        ),
    );
}

#[test]
fn greek_fonts_cover_greek_text() {
    covers(
        "Grek",
        concat!(
            "ΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩαβγδεζηθικλμνξοπρσςτυφχψω",
            "άέήίόύώΐΰϊϋΆΈΉΊΌΎΏ·;",
        ),
    );
}

#[test]
fn devanagari_fonts_cover_devanagari_text() {
    covers(
        "Deva",
        concat!(
            "अआइईउऊऋएऐओऔकखगघङचछजझञटठडढणतथदधनपफबभमयरलवशषसह",
            "ािीुूृेैोौंःँ़्",
            "।॥०१२३४५६७८९₹",
        ),
    );
}

/// The UI translations' Latin beyond Western Europe: Vietnamese, Central European, Turkish,
/// Baltic. The Japanese faces lack Vietnamese and the CJK faces lack the rest, so the Latin
/// fallback is Noto Sans.
#[test]
fn noto_sans_covers_extended_latin() {
    covered_by(
        "Noto Sans",
        FONTS.iter().filter(|f| f.family == "Noto Sans"),
        concat!(
            "ĂăÂâĐđÊêÔôƠơƯưẠạẢảẤấẦầẨẩẪẫẬậẮắẰằẲẳẴẵẶặẸẹẺẻẼẽẾếỀềỂểỄễỆệỈỉỊịỌọỎỏỐốỒồỔổỖỗỘộ",
            "ỚớỜờỞởỠỡỢợỤụỦủỨứỪừỬửỮữỰựỲỳỴỵỶỷỸỹ₫",
            "ąćęłńóśźżĄĆĘŁŃÓŚŹŻčďěňřšťůžČĎĚŇŘŠŤŮŽőűŐŰ",
            "ğışĞİŞõäöüßÕÄÖÜẞėįųūĖĮŲŪāēīĀĒĪņļķģŅĻĶĢⁿ€",
        ),
    );
}

#[test]
fn emoji_fonts_cover_emoji() {
    covers(
        "Zsye",
        concat!(
            "😀😂😍🥺🫠👍👋👩💻🧑🎨🎉🔥⭐✅❌⚠❤🌍🪿📁📂🔍🏳🏴",
            "🇯🇵🇺🇳",       // regional indicators (flags)
            "\u{200d}🏽", // ZWJ sequences, skin tone
        ),
    );
}

#[test]
fn colour_emoji_is_colrv1() {
    // egui and the web renderers draw COLRv1 outlines; the CBDT bitmap build would draw nothing.
    let f = FONTS
        .iter()
        .find(|f| f.family == "Noto Color Emoji")
        .unwrap_or_else(|| panic!("the manifest lists no Noto Color Emoji"));
    let data = bytes(f);
    let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
    let colr = font
        .colr()
        .unwrap_or_else(|e| panic!("{}: no COLR table: {e}", f.file));
    assert_eq!(colr.version(), 1, "{}: COLR version", f.file);
    assert!(font.cbdt().is_err(), "{}: has bitmap (CBDT) glyphs", f.file);
}

/// Modifier and key symbols in shortcut labels and the UI translations. No one Noto face has
/// them all: Symbols 2 has the modifiers, Symbols has ⌃ ⌄ ⎋, Math has the return, tab and page
/// arrows.
#[test]
fn symbol_fonts_together_cover_key_symbols() {
    let fonts: Vec<&Font> = for_script("Zsym").collect();
    assert!(!fonts.is_empty(), "the manifest lists no Zsym font");
    let data: Vec<Vec<u8>> = fonts.iter().map(|f| bytes(f)).collect();
    let refs: Vec<FontRef> = data
        .iter()
        .zip(&fonts)
        .map(|(d, f)| FontRef::new(d).unwrap_or_else(|e| panic!("{}: {e}", f.file)))
        .collect();
    let missing: String = "⇧⇪⌃⌄⌥⌘⌫⌦⌤⌧⎋↩↵⏎⇥⇤⇞⇟↖↘←↑→↓↔↕↗⏏"
        .chars()
        .filter(|c| refs.iter().all(|r| r.charmap().map(*c).is_none()))
        .collect();
    assert!(missing.is_empty(), "no Zsym font has {missing:?}");
}

/// The basic character set an interface face for `script` must cover before an app's build cuts
/// it to the app's catalogs. The Han and Hangul lists are the first 3,500 Han of GB 2312 level 1,
/// the first 3,500 Han of Big5 level 1 and the 2,350 KS X 1001 Hangul syllables, in code order
/// (decoded from their legacy encodings with Python's codecs).
fn ui_basic_set(script: &str) -> Option<String> {
    match script {
        "Jpan" => Some(
            ('\u{3041}'..='\u{3096}')
                .chain('\u{30A1}'..='\u{30FA}')
                .collect(),
        ),
        "Hans" => Some(
            include_str!("../data/gb2312-han.txt")
                .trim_end()
                .to_string(),
        ),
        "Hant" | "Hant-HK" => Some(include_str!("../data/big5-han.txt").trim_end().to_string()),
        "Kore" => Some(
            include_str!("../data/ksx1001-hangul.txt")
                .trim_end()
                .to_string(),
        ),
        _ => None,
    }
}

/// Each interface face (`role` `ui`) is a TrueType-outline face, because the apps' build-time
/// subsetter cuts CFF faces wrongly; covers its script's basic set; and keeps the layout features
/// interface text needs. Every CJK script has one.
#[test]
fn ui_faces_are_glyf_sources_for_their_script() {
    let ui: Vec<&Font> = FONTS.iter().filter(|f| f.role == Some(ROLE_UI)).collect();
    let mut scripts = std::collections::BTreeSet::new();
    for f in ui {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        assert!(
            font.table_data(skrifa::Tag::new(b"CFF ")).is_none() && font.glyf().is_ok(),
            "{}: not a glyf face",
            f.file
        );
        let charmap = font.charmap();
        let mut sets = 0;
        for script in f.scripts {
            let Some(set) = ui_basic_set(script) else {
                continue;
            };
            let missing: String = set.chars().filter(|c| charmap.map(*c).is_none()).collect();
            assert!(
                missing.is_empty(),
                "{} lacks {} {script} basic characters: {missing:?}",
                f.family,
                missing.chars().count()
            );
            scripts.insert(*script);
            sets += 1;
        }
        assert!(
            sets > 0,
            "{}: no basic set for scripts {:?}",
            f.file,
            f.scripts
        );
        let tags: Vec<String> = font
            .gsub()
            .and_then(|g| g.feature_list())
            .map(|l| {
                l.feature_records()
                    .iter()
                    .map(|r| r.feature_tag().to_string())
                    .collect()
            })
            .unwrap_or_default();
        // BIZ UDPGothic is Japanese-only and has no locl; the Noto faces share code points across
        // languages and need theirs.
        let needed: &[&str] = if f.scripts.contains(&"Jpan") {
            &["vert"]
        } else {
            &["vert", "locl"]
        };
        for tag in needed {
            assert!(
                tags.iter().any(|t| t == tag),
                "{}: no {tag} GSUB feature (has {tags:?})",
                f.family
            );
        }
    }
    for script in ["Hans", "Hant", "Hant-HK", "Jpan", "Kore"] {
        assert!(scripts.contains(script), "no interface face for {script}");
    }
}
