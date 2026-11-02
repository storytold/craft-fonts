//! Provenance and minimum shaping inputs for the three unmodified Syriac faces.
use sha2::{Digest, Sha256};
use skrifa::{FontRef, MetadataProvider, raw::TableProvider, string::StringId};

#[test]
fn syriac_faces_have_pinned_bytes_licenses_and_layout_tables() {
    let faces: Vec<_> = craft_fonts::for_script("Syrc").collect();
    assert_eq!(
        faces.iter().map(|face| face.family).collect::<Vec<_>>(),
        [
            "Noto Sans Syriac",
            "Noto Sans Syriac Western",
            "Noto Sans Syriac Eastern"
        ]
    );
    let root = craft_fonts_tests::repo_root();
    for face in faces {
        let bytes = std::fs::read(root.join(face.file)).unwrap();
        let digest: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(digest, face.sha256);
        assert!(
            face.source
                .starts_with("https://github.com/notofonts/syriac/releases/tag/")
        );
        assert_eq!(face.licence, "OFL-1.1");
        let license = std::fs::read_to_string(root.join(face.licence_file)).unwrap();
        assert!(license.contains("SIL OPEN FONT LICENSE Version 1.1"));
        assert!(license.contains("The Noto Project Authors"));
        let font = FontRef::new(&bytes).unwrap();
        assert_eq!(face.style, "Regular");
        assert!(
            [StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME]
                .into_iter()
                .flat_map(|id| font.localized_strings(id))
                .any(|name| name.to_string() == face.family),
            "{} must match its font's family name",
            face.file
        );
        assert!(
            font.fvar().is_err(),
            "{} must be a static face",
            face.family
        );
        assert!(
            font.gsub().is_ok() && font.gpos().is_ok(),
            "{} must carry shaping and mark positioning",
            face.family
        );
        for ch in "ܫܠܡܐܒܓܕܐ݂ܰܳܶܺܽ݁".chars() {
            assert!(
                font.charmap().map(ch).is_some_and(|gid| gid.to_u32() != 0),
                "{} lacks {ch}",
                face.family
            );
        }
    }
}
