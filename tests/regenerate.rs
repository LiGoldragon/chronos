//! Authored Ethos is the only source of the generated contract projection.

use std::{fs, path::Path};

use ethos_zero::Generating;
use protos::{Actualizable, Potential};

#[test]
fn committed_datom_library_is_the_current_ethos_projection() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(root.join(".ethos/data/chronos.ethos")).expect("read Chronos Ethos source");
    let file = Potential::<ethos_zero::File>::from(source.as_str()).actualize(()).expect("parse Chronos Ethos source");
    let generated = file.generate().expect("generate current Datom projection");
    let committed = fs::read_to_string(root.join(".ethos/data/datomic_library.rs")).expect("read committed projection");
    assert_eq!(committed, generated);
}
