//! Turns `fonts/manifest.txt` and `fonts/latin-manifest.txt` into static tables (and, with the
//! `embed` / `embed-latin` features, the font bytes).

#[path = "src/parse.rs"]
#[allow(dead_code)]
mod parse;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default());
    let root = manifest_dir.join("../..");
    let mut src = String::new();
    table(
        &root,
        "fonts/manifest.txt",
        "FONTS",
        "EMBEDDED",
        std::env::var_os("CARGO_FEATURE_EMBED").is_some(),
        &mut src,
    );
    table(
        &root,
        "fonts/latin-manifest.txt",
        "LATIN_FONTS",
        "LATIN_EMBEDDED",
        std::env::var_os("CARGO_FEATURE_EMBED_LATIN").is_some(),
        &mut src,
    );
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("manifest.rs");
    if let Err(e) = std::fs::write(&out, src) {
        fail(&format!("writing {}: {e}", out.display()));
    }
}

/// Append `pub static <name>: &[Font]` for the manifest at `rel`, and with `embed` the bytes as
/// `static <embedded>: &[&[u8]]`.
fn table(root: &Path, rel: &str, name: &str, embedded: &str, embed: bool, src: &mut String) {
    let manifest = root.join(rel);
    println!("cargo::rerun-if-changed={}", manifest.display());
    let text = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|e| fail(&format!("reading {}: {e}", manifest.display())));
    let entries = parse::parse_manifest(&text).unwrap_or_else(|e| fail(&format!("{rel}: {e}")));

    let _ = writeln!(
        src,
        "/// Every font in `{rel}`, in manifest order.\npub static {name}: &[Font] = &["
    );
    for e in &entries {
        let scripts: Vec<String> = e.scripts.iter().map(|s| format!("{s:?}")).collect();
        let _ = writeln!(
            src,
            "    Font {{ family: {:?}, style: {:?}, file: {:?}, scripts: &[{}], licence: {:?}, licence_file: {:?}, sha256: {:?}, source: {:?} }},",
            e.family,
            e.style,
            e.file,
            scripts.join(", "),
            e.licence,
            e.licence_file,
            e.sha256,
            e.source
        );
    }
    src.push_str("];\n");
    if embed {
        let _ = writeln!(
            src,
            "/// The bytes of each font in [`{name}`], same order.\nstatic {embedded}: &[&[u8]] = &["
        );
        for e in &entries {
            let path = root.join(&e.file);
            println!("cargo::rerun-if-changed={}", path.display());
            if !path.is_file() {
                fail(&format!(
                    "{} is listed in {rel} but missing",
                    path.display()
                ));
            }
            let _ = writeln!(src, "    include_bytes!({:?}),", path.display().to_string());
        }
        src.push_str("];\n");
    }
}

/// Build scripts report errors by failing; this keeps the message readable.
#[allow(clippy::panic)]
fn fail(msg: &str) -> ! {
    panic!("craft-fonts: {msg}")
}
