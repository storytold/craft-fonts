//! Korean UI font coverage, independent of an application's installed fonts.
use craft_fonts::for_script;
use skrifa::{FontRef, MetadataProvider};

#[test]
fn korean_fonts_cover_every_modern_hangul_syllable() {
    let fonts: Vec<_> = for_script("Kore").collect();
    assert!(!fonts.is_empty(), "the manifest lists no Korean UI font");
    for font in fonts {
        let data = std::fs::read(craft_fonts_tests::repo_root().join(font.file))
            .unwrap_or_else(|error| panic!("{}: {error}", font.file));
        let face = FontRef::new(&data)
            .unwrap_or_else(|error| panic!("{}: {error}", font.file));
        let charmap = face.charmap();
        for ch in '\u{ac00}'..='\u{d7a3}' {
            assert!(
                charmap.map(ch).is_some(),
                "{} {}: missing U+{:04X}",
                font.family,
                font.style,
                ch as u32
            );
        }
        for ch in "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".chars() {
            assert!(charmap.map(ch).is_some(), "{} {}: missing {ch}", font.family, font.style);
        }
    }
}
