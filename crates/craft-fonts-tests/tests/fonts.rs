//! Assertions on the font files themselves.

use craft_fonts::{Entry, FONTS, Font, LATIN_FONTS, for_script, parse_manifest};
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

/// Every font in both the default and opt-in Latin manifests.
fn all() -> impl Iterator<Item = &'static Font> {
    FONTS.iter().chain(LATIN_FONTS)
}

#[test]
fn every_file_matches_its_manifest_checksum() {
    for f in all() {
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
    for f in all() {
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
    for f in all() {
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
    for f in all() {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        // A `Variable` / `Italic Variable` row names the fvar file, not a name-table style.
        if f.style.ends_with("Variable") {
            let axes: Vec<String> = font.axes().iter().map(|a| a.tag().to_string()).collect();
            assert!(
                axes.iter().any(|t| t == "wght"),
                "{}: style {:?} but axes are {axes:?}",
                f.file,
                f.style
            );
            let os2 = font
                .os2()
                .unwrap_or_else(|e| panic!("{}: OS/2: {e}", f.file));
            assert_eq!(
                os2.fs_selection().contains(SelectionFlags::ITALIC),
                f.style.contains("Italic"),
                "{}: style {:?} disagrees with the OS/2 italic bit",
                f.file,
                f.style
            );
            continue;
        }
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
fn bengali_fonts_cover_bengali_text() {
    // Bengali: letters, vowel signs and virama, digits, danda.
    covers(
        "Beng",
        concat!(
            "ঀঅআইঈউঊঋঌএঐওঔকখগঘঙচছজঝঞটঠডঢণতথদধনপফবভমযরলশষসহঽৎড়ঢ়য়ৠৡৰৱ",
            "ঁংঃ়ািীুূৃৄেৈোৌ্ৗৢৣ৾",
            "০১২৩৪৫৬৭৮৯",
            "।॥",
        ),
    );
}

#[test]
fn gurmukhi_fonts_cover_gurmukhi_text() {
    // Gurmukhi (Punjabi): letters, vowel signs and virama, digits, danda.
    covers(
        "Guru",
        concat!(
            "ਅਆਇਈਉਊਏਐਓਔਕਖਗਘਙਚਛਜਝਞਟਠਡਢਣਤਥਦਧਨਪਫਬਭਮਯਰਲਲ਼ਵਸ਼ਸਹਖ਼ਗ਼ਜ਼ੜਫ਼ੲੳੴ",
            "ਁਂਃ਼ਾਿੀੁੂੇੈੋੌ੍ੑੰੱੵ",
            "੦੧੨੩੪੫੬੭੮੯",
            "।॥",
        ),
    );
}

#[test]
fn gujarati_fonts_cover_gujarati_text() {
    // Gujarati: letters, vowel signs and virama, digits, danda.
    covers(
        "Gujr",
        concat!(
            "અઆઇઈઉઊઋઌઍએઐઑઓઔકખગઘઙચછજઝઞટઠડઢણતથદધનપફબભમયરલળવશષસહઽૐૠૡૹ",
            "ઁંઃ઼ાિીુૂૃૄૅેૈૉોૌ્ૢૣૺૻૼ૽૾૿",
            "૦૧૨૩૪૫૬૭૮૯",
            "।॥",
        ),
    );
}

#[test]
fn oriya_fonts_cover_oriya_text() {
    // Odia: letters, vowel signs and virama, digits, danda.
    covers(
        "Orya",
        concat!(
            "ଅଆଇଈଉଊଋଌଏଐଓଔକଖଗଘଙଚଛଜଝଞଟଠଡଢଣତଥଦଧନପଫବଭମଯରଲଳଵଶଷସହଽଡ଼ଢ଼ୟୠୡୱ",
            "ଁଂଃ଼ାିୀୁୂୃୄେୈୋୌ୍୕ୖୗୢୣ",
            "୦୧୨୩୪୫୬୭୮୯",
            "।॥",
        ),
    );
}

#[test]
fn tamil_fonts_cover_tamil_text() {
    // Tamil: letters, vowel signs and virama, digits, danda.
    covers(
        "Taml",
        concat!(
            "ஃஅஆஇஈஉஊஎஏஐஒஓஔகஙசஜஞடணதநனபமயரறலளழவஶஷஸஹௐ",
            "ஂாிீுூெேைொோௌ்ௗ",
            "௦௧௨௩௪௫௬௭௮௯",
            "।॥",
        ),
    );
}

#[test]
fn telugu_fonts_cover_telugu_text() {
    // Telugu: letters, vowel signs and virama, digits, danda.
    covers(
        "Telu",
        concat!(
            "అఆఇఈఉఊఋఌఎఏఐఒఓఔకఖగఘఙచఛజఝఞటఠడఢణతథదధనపఫబభమయరఱలళఴవశషసహఽౘౙౚౠౡ",
            "ఀఁంఃఄాిీుూృౄెేైొోౌ్ౕౖౢౣ",
            "౦౧౨౩౪౫౬౭౮౯",
            "।॥",
        ),
    );
}

#[test]
fn kannada_fonts_cover_kannada_text() {
    // Kannada: letters, vowel signs and virama, digits, danda.
    covers(
        "Knda",
        concat!(
            "ಀಅಆಇಈಉಊಋಌಎಏಐಒಓಔಕಖಗಘಙಚಛಜಝಞಟಠಡಢಣತಥದಧನಪಫಬಭಮಯರಱಲಳವಶಷಸಹಽೞೠೡೱೲ",
            "ಁಂಃ಼ಾಿೀುೂೃೄೆೇೈೊೋೌ್ೕೖೢೣ",
            "೦೧೨೩೪೫೬೭೮೯",
            "।॥",
        ),
    );
}

#[test]
fn malayalam_fonts_cover_malayalam_text() {
    // Malayalam: letters, vowel signs and virama, digits, danda.
    covers(
        "Mlym",
        concat!(
            "അആഇഈഉഊഋഌഎഏഐഒഓഔകഖഗഘങചഛജഝഞടഠഡഢണതഥദധനഩപഫബഭമയരറലളഴവശഷസഹഺഽൎൔൕൖൠൡൺൻർൽൾൿ",
            "ഀഁംഃ഻഼ാിീുൂൃൄെേൈൊോൌ്ൗൢൣ",
            "൦൧൨൩൪൫൬൭൮൯",
            "।॥",
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

/// The opt-in Latin presentation catalog. FigCraft and DeckCraft read it from their own
/// `build.rs`; consumers that read only `fonts/manifest.txt` must not receive these faces, and it
/// is deliberately not part of the default `FONTS`/`embed` payload.
const LATIN_MANIFEST: &str = "fonts/latin-manifest.txt";
const LATIN_FACE_COUNT: usize = 51;
const LATIN_FAMILY_COUNT: usize = 16;
const INTER_LICENCE_FILE: &str = "fonts/inter/OFL.txt";
const INTER_LICENCE_SHA256: &str =
    "262481e844521b326f5ecd053e59b98c8b2da78c8ee1bdbb6e8174305e54935a";

/// The exact reviewed catalog, in manifest order: family, style, file and SHA-256. Every row is a
/// `Latn` OFL-1.1 face and the licence file sits next to the font. The eight Adobe-authored
/// Source Sans 3 / Source Serif 4 faces stay in this central catalog; consumers that must not
/// embed Adobe fonts filter those families themselves.
const LATIN_FACES: &[(&str, &str, &str, &str)] = &[
    (
        "Inter",
        "Regular",
        "fonts/inter/Inter-Regular.ttf",
        "40d692fce188e4471e2b3cba937be967878f631ad3ebbbdcd587687c7ebe0c82",
    ),
    (
        "Inter",
        "Italic",
        "fonts/inter/Inter-Italic.ttf",
        "bbc051dd204b5019a1aa0bc0ae2aa8a05ab13e7a3f979fa357631dc7feb6833a",
    ),
    (
        "Inter",
        "SemiBold",
        "fonts/inter/Inter-SemiBold.ttf",
        "78a843fade9d4612a5567302fb595b56976eb5fcebf4fea5a5912d638bafcde3",
    ),
    (
        "Inter",
        "Bold",
        "fonts/inter/Inter-Bold.ttf",
        "288316099b1e0a47a4716d159098005eef7c0066921f34e3200393dbdb01947f",
    ),
    (
        "Carlito",
        "Regular",
        "fonts/carlito/Carlito-Regular.ttf",
        "f6418f708baede9789daef5d458c0f53d2a888af9820e8062934e504fedc6595",
    ),
    (
        "Carlito",
        "Bold",
        "fonts/carlito/Carlito-Bold.ttf",
        "bb5d20f79b82599ec72983597437373a80f2d2085fa91fc144fd74e876a594db",
    ),
    (
        "Carlito",
        "Italic",
        "fonts/carlito/Carlito-Italic.ttf",
        "0b019225e58d702bfedcbd35c21696769f8ee115cb6343f84c2f240312450d1c",
    ),
    (
        "Carlito",
        "Bold Italic",
        "fonts/carlito/Carlito-BoldItalic.ttf",
        "b32928186c119599e03ca6a1ffc680fdcb7fac95772f4b95d989cf6cd3861517",
    ),
    (
        "Caladea",
        "Regular",
        "fonts/caladea/Caladea-Regular.ttf",
        "f1e899278b7b4491aba5b6a8253c4b04c050cc59b21865be5c37559a775153cd",
    ),
    (
        "Caladea",
        "Bold",
        "fonts/caladea/Caladea-Bold.ttf",
        "ae3cb2dcbc925809dd29d2a44e9802211cab66be541bacbfc9c08c74b27c3742",
    ),
    (
        "Caladea",
        "Italic",
        "fonts/caladea/Caladea-Italic.ttf",
        "4359a8e24f748b6447b1ff6d7a174febe70961d29f8bb8634b56dacd740a3deb",
    ),
    (
        "Caladea",
        "Bold Italic",
        "fonts/caladea/Caladea-BoldItalic.ttf",
        "ccabaa7b7e2fdf253d2b1a5fa699dd8a3df8d835a9eb285ad82631a677eb76c0",
    ),
    (
        "Liberation Sans",
        "Regular",
        "fonts/liberation-sans/LiberationSans-Regular.ttf",
        "76d04c18ea243f426b7de1f3ad208e927008f961dc5945e5aad352d0dfde8ee8",
    ),
    (
        "Liberation Sans",
        "Bold",
        "fonts/liberation-sans/LiberationSans-Bold.ttf",
        "788abee4c806d660e8aee46689dd8540cd4bb98da03dcc9d171ce3efd99a9173",
    ),
    (
        "Liberation Sans",
        "Italic",
        "fonts/liberation-sans/LiberationSans-Italic.ttf",
        "e5bae5c4cde31f22142753855f4f8fb86da6ff39955ed3c0a11248b0d16948b0",
    ),
    (
        "Liberation Sans",
        "Bold Italic",
        "fonts/liberation-sans/LiberationSans-BoldItalic.ttf",
        "698da70fc191cc5f33ad4d6d3fe830fe4624b898ea2e3169955928b7c491f1ee",
    ),
    (
        "Liberation Serif",
        "Regular",
        "fonts/liberation-serif/LiberationSerif-Regular.ttf",
        "058ea80864aef09a23f45cbec2bb5400bc3dfbdea01c3f10538a21fcb497fb74",
    ),
    (
        "Liberation Serif",
        "Bold",
        "fonts/liberation-serif/LiberationSerif-Bold.ttf",
        "d754ba427cfe0bca54ae052384baa8f842da5bd6550ad4da024ac441e7a7d5ce",
    ),
    (
        "Liberation Serif",
        "Italic",
        "fonts/liberation-serif/LiberationSerif-Italic.ttf",
        "0e3dea9f8d613e006ccfa62201f33e265d19167bd0907725c3e145368b04fc2e",
    ),
    (
        "Liberation Serif",
        "Bold Italic",
        "fonts/liberation-serif/LiberationSerif-BoldItalic.ttf",
        "f17db8af71e24d2066b587546021d4f0b296be389512b658dec3c09affeb11a7",
    ),
    (
        "Liberation Mono",
        "Regular",
        "fonts/liberation-mono/LiberationMono-Regular.ttf",
        "f2b83c763e8afd21709333370bed4774337fae82267937e2b5aea7e2fbd922c1",
    ),
    (
        "Liberation Mono",
        "Bold",
        "fonts/liberation-mono/LiberationMono-Bold.ttf",
        "bd62a0672d0b9b6710b01df434c80ad54fa5f0835207eb7b17b7a761463067bb",
    ),
    (
        "Liberation Mono",
        "Italic",
        "fonts/liberation-mono/LiberationMono-Italic.ttf",
        "605c01c711b44480a7508d349dfbf3264e81fa43d69e61cfa7d10b86e764c4d1",
    ),
    (
        "Liberation Mono",
        "Bold Italic",
        "fonts/liberation-mono/LiberationMono-BoldItalic.ttf",
        "79451f3c09fe25116098853b7a2ca6e2436220ccc11af022979adbcf195be130",
    ),
    (
        "Source Sans 3",
        "Regular",
        "fonts/source-sans-3/SourceSans3-Regular.ttf",
        "4644c81b86ec9caaa76b634889968ed3c4f4f52f054855933acc7c2b21e53b0f",
    ),
    (
        "Source Sans 3",
        "Italic",
        "fonts/source-sans-3/SourceSans3-It.ttf",
        "192afd78f0f54a3c69eaf02d43f4d9a821e9d6110e41d3d25d61a7385cd580e4",
    ),
    (
        "Source Sans 3",
        "Semibold",
        "fonts/source-sans-3/SourceSans3-Semibold.ttf",
        "a3f4f8dcf343a8f24dc61951de93f3ba1558b15cd250ba24af8a40e957081b7d",
    ),
    (
        "Source Sans 3",
        "Bold",
        "fonts/source-sans-3/SourceSans3-Bold.ttf",
        "9214b9d95e4231c609802815c2646c98174e2102d0d37f88978a7f8e71006e6a",
    ),
    (
        "Source Serif 4",
        "Regular",
        "fonts/source-serif-4/SourceSerif4-Regular.ttf",
        "e5a4ee6a3d87bb9024796be390c6771e2a0eb1883dae25effaf57ca01668e24b",
    ),
    (
        "Source Serif 4",
        "Italic",
        "fonts/source-serif-4/SourceSerif4-It.ttf",
        "9d2950a8f1da66e21502c35d646a1d2148e79f9ea43fd2158cf02f5232e7f430",
    ),
    (
        "Source Serif 4",
        "Semibold",
        "fonts/source-serif-4/SourceSerif4-Semibold.ttf",
        "36db62940cb5728b12b1802476dc7fcf4c6c519a7bdd476ba23a4e555fc4655f",
    ),
    (
        "Source Serif 4",
        "Bold",
        "fonts/source-serif-4/SourceSerif4-Bold.ttf",
        "7cf4f4e1ad74f45058d5bc61716b82560442fbdcd9d3654d2dea96bf6c683d86",
    ),
    (
        "Montserrat",
        "Variable",
        "fonts/montserrat/Montserrat.ttf",
        "0f7b311b2f3279e4eef9b2f968bcdbab6e28f4daeb1f049f4f278a902bcd82f7",
    ),
    (
        "Montserrat",
        "Italic Variable",
        "fonts/montserrat/Montserrat-Italic.ttf",
        "51607f316bc020e59f03cbf51543eecffbea501c0b31d73e5b82927c5cca442c",
    ),
    (
        "Lato",
        "Regular",
        "fonts/lato/Lato-Regular.ttf",
        "d636e4683231f931eda222d588e944d082bfd3bdba02f928bee461c0f185b251",
    ),
    (
        "Lato",
        "Bold",
        "fonts/lato/Lato-Bold.ttf",
        "8a0aace75d33794eece4b28187bfc1df0bbd2888b5d8a56e01788c8d65d16be1",
    ),
    (
        "Lato",
        "Italic",
        "fonts/lato/Lato-Italic.ttf",
        "e399c44efe1387100531d26c7e4800c5d12251b890d6654a3098c7c679cb1786",
    ),
    (
        "Open Sans",
        "Variable",
        "fonts/open-sans/OpenSans.ttf",
        "36643644f318a812aab2d2ed3bb98f8cf0872527f835fe9398d95fe6b9adb878",
    ),
    (
        "Open Sans",
        "Italic Variable",
        "fonts/open-sans/OpenSans-Italic.ttf",
        "fe269381e992f32e135801740998544d6235061e37c93ec067ad2be3edd5b17b",
    ),
    (
        "Roboto",
        "Variable",
        "fonts/roboto/Roboto.ttf",
        "d7598e12c5dbef095ff8272cfc55da0250bd07fbdecbac8a530b9b277872a134",
    ),
    (
        "Roboto",
        "Italic Variable",
        "fonts/roboto/Roboto-Italic.ttf",
        "9725a847af6b460ffca162ae66d20dad48b01876137947180b42d7dcd7887182",
    ),
    (
        "Merriweather",
        "Regular",
        "fonts/merriweather/Merriweather-Regular.ttf",
        "10ec10d1f7a9b5f0eb1a964e06348ce20587d478c661ea36728cce7b57f3de1c",
    ),
    (
        "Merriweather",
        "Bold",
        "fonts/merriweather/Merriweather-Bold.ttf",
        "a2fa4f57946c62036f79d9e8b4b642515001d65af1e6a98afc0b94bf669e3f11",
    ),
    (
        "Merriweather",
        "Italic",
        "fonts/merriweather/Merriweather-Italic.ttf",
        "1f73096441117e56bd867a437188e98bcf8ae7dcc7f676980da2ed2b6b51554d",
    ),
    (
        "Playfair Display",
        "Variable",
        "fonts/playfair-display/PlayfairDisplay.ttf",
        "c40f2293766a503bc70cce9e512ef844a4ccb7cbcde792fe2ea31d191917d8d6",
    ),
    (
        "Playfair Display",
        "Italic Variable",
        "fonts/playfair-display/PlayfairDisplay-Italic.ttf",
        "a5e26dc5e2e77fb2803a0bf02fd4f81ee136ec8dea863ccdb0c59a263b21378b",
    ),
    (
        "Poppins",
        "Regular",
        "fonts/poppins/Poppins-Regular.ttf",
        "7e65201e9b79159e2300267cc885e16c8dcef2424cdfa09a29bfb0980a94a7ba",
    ),
    (
        "Poppins",
        "Bold",
        "fonts/poppins/Poppins-Bold.ttf",
        "983676516167748b74de6f4771fb384c664fd913acb8b471122ecacf5da5ea6c",
    ),
    (
        "Poppins",
        "Italic",
        "fonts/poppins/Poppins-Italic.ttf",
        "4fa76ae75b40f926420514044722cb97f32186cafd3b38263cc34dad7174d46d",
    ),
    (
        "Nunito Sans",
        "Variable",
        "fonts/nunito-sans/NunitoSans.ttf",
        "f934d7142fb4784bf828da485b7dcbd90c0c80d514e9d49a5da0ed3a1ae2491d",
    ),
    (
        "Nunito Sans",
        "Italic Variable",
        "fonts/nunito-sans/NunitoSans-Italic.ttf",
        "d9d5db18f3c11221a4fbb553cbc709391c1179964c7eaa4466ef43c78aa4492f",
    ),
];

#[test]
fn latin_opt_in_manifest_holds_the_reviewed_catalog() {
    let entries = optional_entries(LATIN_MANIFEST);
    assert_eq!(
        entries.len(),
        LATIN_FACE_COUNT,
        "{LATIN_MANIFEST} must list exactly the {LATIN_FACE_COUNT} reviewed Latin faces"
    );
    assert_eq!(
        LATIN_FONTS.len(),
        LATIN_FACE_COUNT,
        "the static LATIN_FONTS table must match the manifest"
    );
    for (e, f) in entries.iter().zip(LATIN_FONTS) {
        assert_eq!(
            (e.family.as_str(), e.style.as_str(), e.file.as_str()),
            (f.family, f.style, f.file),
            "{LATIN_MANIFEST}: static table order"
        );
    }

    assert_eq!(
        LATIN_FACES.len(),
        LATIN_FACE_COUNT,
        "review inventory must cover every face"
    );
    for (e, &(family, style, file, sha256)) in entries.iter().zip(LATIN_FACES) {
        assert_eq!(e.family, family, "{file}: family");
        assert_eq!(e.style.as_str(), style, "{file}: style");
        assert_eq!(e.file.as_str(), file, "{family} {style}: file");
        assert_eq!(
            e.scripts.iter().map(String::as_str).collect::<Vec<_>>(),
            ["Latn"],
            "{file}: scripts"
        );
        assert_eq!(e.licence, "OFL-1.1", "{file}: licence");
        assert_eq!(e.sha256.as_str(), sha256, "{file}: manifest SHA-256");
        assert!(
            !e.source.is_empty() && e.source.starts_with("https://"),
            "{file}: pinned source"
        );
        assert_eq!(
            std::path::Path::new(&e.file).parent(),
            std::path::Path::new(&e.licence_file).parent(),
            "the licence must sit next to the font it covers"
        );
        assert!(
            FONTS.iter().all(|f| f.file != e.file.as_str()),
            "{file}: an opt-in Latin face must not appear in the default FONTS table"
        );
    }

    let mut files: Vec<&str> = entries.iter().map(|e| e.file.as_str()).collect();
    files.sort_unstable();
    let listed = files.len();
    files.dedup();
    assert_eq!(files.len(), listed, "{LATIN_MANIFEST}: duplicate file rows");

    let mut families: Vec<&str> = entries.iter().map(|e| e.family.as_str()).collect();
    families.sort_unstable();
    families.dedup();
    assert_eq!(
        families.len(),
        LATIN_FAMILY_COUNT,
        "{LATIN_MANIFEST}: families {families:?}"
    );
    let variable = LATIN_FONTS
        .iter()
        .filter(|f| f.style.ends_with("Variable"))
        .count();
    assert_eq!(
        variable, 10,
        "five families each contribute an upright and an italic variable file"
    );

    let default =
        std::fs::read_to_string(craft_fonts_tests::repo_root().join("fonts/manifest.txt")).unwrap();
    assert!(
        !default.contains("latin-manifest"),
        "the default font manifest must not list the opt-in Latin faces"
    );
    for e in &entries {
        assert!(
            !default.contains(e.file.as_str()),
            "{}: the default manifest contains a Latin-only file",
            e.file
        );
    }
}

#[test]
fn latin_inter_files_and_their_upstream_licence_match_their_hashes() {
    let entries: Vec<Entry> = optional_entries(LATIN_MANIFEST)
        .into_iter()
        .filter(|e| e.family == "Inter")
        .collect();
    assert_eq!(entries.len(), 4, "{LATIN_MANIFEST}: Inter faces");
    for e in &entries {
        assert_eq!(e.licence, "OFL-1.1", "{}: licence", e.file);
        assert_eq!(
            e.licence_file, INTER_LICENCE_FILE,
            "{}: licence file",
            e.file
        );
        let data = bytes_from_entry(e);
        let digest = Sha256::digest(&data);
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex, e.sha256,
            "{}: the file changed; update the opt-in manifest only after checking the pinned upstream source",
            e.file
        );
    }

    let licence = std::fs::read(craft_fonts_tests::repo_root().join(INTER_LICENCE_FILE))
        .unwrap_or_else(|e| panic!("{INTER_LICENCE_FILE}: {e}"));
    assert_eq!(licence.len(), 4_380, "{INTER_LICENCE_FILE}: size");
    let digest = Sha256::digest(&licence);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(hex, INTER_LICENCE_SHA256, "{INTER_LICENCE_FILE}: SHA-256");
    let text = String::from_utf8_lossy(&licence);
    assert!(
        text.contains("Copyright (c) 2016 The Inter Project Authors"),
        "{INTER_LICENCE_FILE}: the upstream Inter copyright notice is missing"
    );
    assert!(
        text.contains("SIL OPEN FONT LICENSE Version 1.1"),
        "{INTER_LICENCE_FILE} is not the OFL 1.1 text"
    );
}

#[test]
fn latin_inter_faces_are_the_static_styles_and_glyphs_the_manifest_claims() {
    const SAMPLE: &str = "Inter FigCraft ÀÉÎÕÜ çñ œŒ “quotes” – — … • €";
    let entries: Vec<Entry> = optional_entries(LATIN_MANIFEST)
        .into_iter()
        .filter(|e| e.family == "Inter")
        .collect();
    assert_eq!(entries.len(), 4, "{LATIN_MANIFEST}: Inter faces");
    for e in &entries {
        let data = bytes_from_entry(e);
        assert_eq!(
            &data[..4],
            &[0x00, 0x01, 0x00, 0x00],
            "{}: expected a single TrueType sfnt, not a collection",
            e.file
        );
        let font = FontRef::new(&data).unwrap_or_else(|err| panic!("{}: {err}", e.file));

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
        assert!(
            font.axes().is_empty(),
            "{}: the reviewed Inter faces are static, not variable",
            e.file
        );

        let expected_weight = match e.style.as_str() {
            "Regular" | "Italic" => 400,
            "SemiBold" => 600,
            "Bold" => 700,
            other => panic!("{}: unexpected Inter style {other:?}", e.file),
        };
        let os2 = font
            .os2()
            .unwrap_or_else(|err| panic!("{}: OS/2: {err}", e.file));
        assert_eq!(
            os2.us_weight_class(),
            expected_weight,
            "{}: OS/2 weight",
            e.file
        );
        let selection = os2.fs_selection();
        assert_eq!(
            selection.contains(SelectionFlags::ITALIC),
            e.style == "Italic",
            "{}: style {:?} disagrees with the OS/2 italic bit",
            e.file,
            e.style
        );

        for c in SAMPLE.chars().filter(|c| !c.is_whitespace()) {
            assert_glyph_outline(&font, &e.file, c);
        }
    }
}

/// Text every Latin presentation font must cover: ASCII, Western European accents and the
/// typographic punctuation document and slide text uses.
const LATIN: &str = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
    " !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~",
    "ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÑÒÓÔÕÖØÙÚÛÜÝßàáâãäåæçèéêëìíîïñòóôõöøùúûüýÿŒœŸ",
    "‘’“”–—…•€£¥©®™°±×÷",
);

#[test]
fn latin_fonts_cover_western_european_text() {
    assert_eq!(LATIN_FONTS.len(), LATIN_FACE_COUNT);
    for f in LATIN_FONTS {
        assert_eq!(
            f.scripts,
            ["Latn"],
            "{}: the Latin catalog is for Latn fonts",
            f.file
        );
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        let charmap = font.charmap();
        let missing: String = LATIN
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
fn latin_glyphs_have_outlines() {
    for f in LATIN_FONTS {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        for c in ['A', 'g', 'é', '€', 'Œ', '—'] {
            assert_glyph_outline(&font, f.file, c);
        }
    }
}

#[test]
fn latin_variable_styles_are_variable_fonts() {
    // A style named "... Variable" must be a variable font with a real weight range; any other
    // style must be a static instance, so apps can trust the style field.
    for f in LATIN_FONTS {
        let data = bytes(f);
        let font = FontRef::new(&data).unwrap_or_else(|e| panic!("{}: {e}", f.file));
        let axes = font.axes();
        let tags: Vec<String> = axes.iter().map(|a| a.tag().to_string()).collect();
        if f.style.ends_with("Variable") {
            let weight = axes
                .iter()
                .find(|a| a.tag() == "wght")
                .unwrap_or_else(|| panic!("{}: style {:?} but axes {tags:?}", f.file, f.style));
            assert!(
                weight.min_value() < weight.max_value(),
                "{}: wght axis {:?}..{:?} is not a range",
                f.file,
                weight.min_value(),
                weight.max_value()
            );
            assert!(
                weight.min_value() <= 400.0 && 400.0 <= weight.max_value(),
                "{}: wght axis {:?}..{:?} does not include Regular 400",
                f.file,
                weight.min_value(),
                weight.max_value()
            );
        } else {
            assert!(
                axes.is_empty(),
                "{}: style {:?} but it is variable ({tags:?})",
                f.file,
                f.style
            );
        }
        let os2 = font
            .os2()
            .unwrap_or_else(|e| panic!("{}: OS/2: {e}", f.file));
        assert_eq!(
            os2.fs_selection().contains(SelectionFlags::ITALIC),
            f.style.contains("Italic"),
            "{}: style {:?} disagrees with the OS/2 italic bit",
            f.file,
            f.style
        );
    }
}

/// Preserve every complete upstream copyright/licence text, not only the generic OFL terms.
#[test]
fn all_latin_licences_match_the_reviewed_upstream_catalog() {
    // Hashes of the unchanged 76ba5a0496503c0e2841abd626295038cbe07cde licence blobs.
    let expected = [
        (
            "fonts/caladea/OFL.txt",
            "ccdab61d371d8c8683a128a92cd7d498dbdb1d37689f7cb21f1bf6b16658d213",
        ),
        (
            "fonts/carlito/OFL.txt",
            "58402f82a7c332a700294988fe7554fbb0a63a8d27ccc1ee3bbc640311990a00",
        ),
        (
            "fonts/inter/OFL.txt",
            "262481e844521b326f5ecd053e59b98c8b2da78c8ee1bdbb6e8174305e54935a",
        ),
        (
            "fonts/lato/OFL.txt",
            "74ba064d03f1f1c4a952da936c3eb71866c34404916734de3cae73b34357e59e",
        ),
        (
            "fonts/liberation-mono/OFL.txt",
            "93fed46019c38bbe566b479d22148e2e8a1e85ada614accb0211c37b2c61c19b",
        ),
        (
            "fonts/liberation-sans/OFL.txt",
            "93fed46019c38bbe566b479d22148e2e8a1e85ada614accb0211c37b2c61c19b",
        ),
        (
            "fonts/liberation-serif/OFL.txt",
            "93fed46019c38bbe566b479d22148e2e8a1e85ada614accb0211c37b2c61c19b",
        ),
        (
            "fonts/merriweather/OFL.txt",
            "a7ba785ad99b6eae0444ab89910e512d4af9865937fddbcb5c72c4f59f55d6ad",
        ),
        (
            "fonts/montserrat/OFL.txt",
            "8b7141c03fa4f8d44e6345d5d4931709290f0f67875e452e95ac1fd3a027802e",
        ),
        (
            "fonts/nunito-sans/OFL.txt",
            "efbb0c9e864cef973982d9a17567e6be5c3d1759695574586f3f18c7ecca064b",
        ),
        (
            "fonts/open-sans/OFL.txt",
            "fbbbcfef55318de350562559b671360de6d597112ecc5c73881b05092db89602",
        ),
        (
            "fonts/playfair-display/OFL.txt",
            "566be814f8e96e93dfa16101331557eb6b5467e9e03f627c0910fe93ca12300e",
        ),
        (
            "fonts/poppins/OFL.txt",
            "6be04893d770899a015649c7aa3b582f871b272f8747a92b78b17c3e5c8b2573",
        ),
        (
            "fonts/roboto/OFL.txt",
            "061402327a96aadb0bfb694a960ed289ecd38d383e396243831ab81feb109c41",
        ),
        (
            "fonts/source-sans-3/OFL.txt",
            "89ad2c4f66dd29127527493e729c31e731f111cf10faf5774c3db9275ed0c22c",
        ),
        (
            "fonts/source-serif-4/OFL.txt",
            "75784a295293a8992f5a8d99210566e0064a012e6dab6731305e3787f15896c7",
        ),
    ];
    let actual: std::collections::BTreeSet<_> =
        LATIN_FONTS.iter().map(|font| font.licence_file).collect();
    assert_eq!(
        actual.len(),
        expected.len(),
        "every Latin family needs a reviewed licence"
    );
    for (path, hash) in expected {
        assert!(actual.contains(path), "missing licence {path}");
        let data = std::fs::read(craft_fonts_tests::repo_root().join(path)).unwrap();
        let digest = Sha256::digest(&data);
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, hash, "{path}: complete upstream licence changed");
    }
}
