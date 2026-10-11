//! The new interface faces must contain letters, combining marks and shaping tables.
use craft_fonts::for_script;
use skrifa::raw::TableProvider;
use skrifa::{FontRef, MetadataProvider};

fn covers(script: &str, text: &str, needs_shaping: bool) {
    let fonts: Vec<_> = for_script(script).collect();
    assert!(!fonts.is_empty(), "no font tagged {script}");
    for face in fonts {
        let data = std::fs::read(craft_fonts_tests::repo_root().join(face.file)).unwrap();
        let font = FontRef::new(&data).unwrap();
        let missing: String = text
            .chars()
            .filter(|c| !c.is_whitespace() && font.charmap().map(*c).is_none())
            .collect();
        assert!(missing.is_empty(), "{} misses {missing:?}", face.family);
        if needs_shaping {
            assert!(font.gsub().is_ok(), "{} needs GSUB", face.family);
            assert!(font.gpos().is_ok(), "{} needs GPOS", face.family);
        }
    }
}

#[test]
fn devanagari_letters_marks_and_shaping_tables() {
    covers(
        "Deva",
        "हिन्दी फ़ाइल खोलें क्ष त्र ज्ञ श्र प्रक्रिया ०१२३४५६७८९ ॐ । ॥",
        true,
    );
}

#[test]
fn bengali_letters_marks_and_shaping_tables() {
    covers(
        "Beng",
        "বাংলা ফাইল খুলুন ক্ষ জ্ঞ শ্র ত্র ক্রিয়া ০১২৩৪৫৬৭৮৯ ৎ য় ড় ঢ়",
        true,
    );
}

#[test]
fn korean_syllables_and_jamo() {
    covers(
        "Hang",
        "한국어 파일 열기 저장 문서 인쇄 ㄱㄴㄷㄹㅁㅂㅅㅇㅈㅊㅋㅌㅍㅎ ㅏㅑㅓㅕㅗㅛㅜㅠㅡㅣ",
        false,
    );
}

#[test]
fn existing_arabic_face_also_covers_urdu() {
    covers(
        "Arab",
        "اردو فائل کھولیں دستاویز محفوظ کریں ٹ ڈ ڑ ں ھ ے ء ئ ؤ",
        true,
    );
}
