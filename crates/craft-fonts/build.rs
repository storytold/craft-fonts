//! Turns `fonts/manifest.txt` into a static table (and, with the `embed` feature, the font bytes).

#[path = "src/parse.rs"]
#[allow(dead_code)]
mod parse;

use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default());
    let root = manifest_dir.join("../..");
    let manifest = root.join("fonts/manifest.txt");
    println!("cargo::rerun-if-changed={}", manifest.display());
    let text = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|e| fail(&format!("reading {}: {e}", manifest.display())));
    let entries = parse::parse_manifest(&text).unwrap_or_else(|e| fail(&e.to_string()));
    let embed = std::env::var_os("CARGO_FEATURE_EMBED").is_some();

    let mut src = String::from(
        "/// Every font in `fonts/manifest.txt`, in manifest order.\npub static FONTS: &[Font] = &[\n",
    );
    for e in &entries {
        let scripts: Vec<String> = e.scripts.iter().map(|s| format!("{s:?}")).collect();
        let _ = writeln!(
            src,
            "    Font {{ family: {:?}, style: {:?}, file: {:?}, scripts: &[{}], licence: {:?}, licence_file: {:?}, sha256: {:?}, source: {:?}, role: {:?} }},",
            e.family,
            e.style,
            e.file,
            scripts.join(", "),
            e.licence,
            e.licence_file,
            e.sha256,
            e.source,
            e.role
        );
    }
    src.push_str("];\n");
    if embed {
        src.push_str("/// The bytes of each font in [`FONTS`], same order.\nstatic EMBEDDED: &[&[u8]] = &[\n");
        for e in &entries {
            let path = root.join(&e.file);
            println!("cargo::rerun-if-changed={}", path.display());
            if !path.is_file() {
                fail(&format!(
                    "{} is listed in the manifest but missing",
                    path.display()
                ));
            }
            let _ = writeln!(src, "    include_bytes!({:?}),", path.display().to_string());
        }
        src.push_str("];\n");
    }
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("manifest.rs");
    if let Err(e) = std::fs::write(&out, src) {
        fail(&format!("writing {}: {e}", out.display()));
    }
}

/// Build scripts report errors by failing; this keeps the message readable.
#[allow(clippy::panic)]
fn fail(msg: &str) -> ! {
    panic!("craft-fonts: {msg}")
}
