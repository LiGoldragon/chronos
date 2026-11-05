use std::path::Path;

use chronos::{EmbodyingPlace, Place, PlacingChronos};
use protos::Textualizable;

#[test]
fn startup_place_is_one_datom_carrying_runtime_and_memory() {
    let place = Place::from_text("{ main /runtime/nexus/main /state/nexus/main }").expect("embody Place");

    assert_eq!(place.ordinary_socket(), Path::new("/runtime/nexus/main/chronos.sock"));
    assert_eq!(place.state(), Path::new("/state/nexus/main"));
    assert_eq!(place.textualize(), "{ main /runtime/nexus/main /state/nexus/main }");
}

#[test]
fn cli_default_and_startup_place_share_the_nexus_layout() {
    let place = Place::main_from_anchors(Path::new("/run/user/42"), Path::new("/home/living"));

    assert_eq!(place.ordinary_socket(), Path::new("/run/user/42/nexus/main/chronos.sock"));
    assert_eq!(place.state(), Path::new("/home/living/.local/state/nexus/main"));
}
