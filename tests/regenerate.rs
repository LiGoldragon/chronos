//! Proves the committed generated module is fresh: identical to what
//! ethos-zero produces from the authored ethos source.

use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use ethos_zero::{Actualizing, File, Generating, Potential};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn format_rust(source: &str) -> String {
    let mut child = Command::new("rustfmt")
        .arg("--edition=2024")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("rustfmt");
    child.stdin.take().unwrap().write_all(source.as_bytes()).expect("write to rustfmt");
    let output = child.wait_with_output().expect("rustfmt output");
    assert!(output.status.success(), "rustfmt failed");
    String::from_utf8(output.stdout).expect("rustfmt output is UTF-8")
}

#[test]
fn committed_datom_library_is_the_current_ethos_projection() {
    let root = project_root();
    let source = fs::read_to_string(root.join(".ethos/data/chronos.ethos")).expect("read authored ethos source");
    let committed =
        fs::read_to_string(root.join(".ethos/data/datomic_library.rs")).expect("read committed generated module");

    let concept = Potential::<File>::from(source).actualize().unwrap_or_else(|_| panic!("actualize ethos source"));
    let emitted = concept.generate().unwrap_or_else(|_| panic!("generate Rust module"));
    let generated = format_rust(&emitted);

    assert_eq!(
        committed, generated,
        "committed .ethos/data/datomic_library.rs differs from the ethos-zero projection of the authored Ethos"
    );
}
