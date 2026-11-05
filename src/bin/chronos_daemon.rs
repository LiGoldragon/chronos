//! `chronos-daemon` — Chronos started in one typed Nexus place.

use chronos::{EmbodyingPlace, Place, PlacingChronos, StartupPlace};
use nexus::Entering;

fn main() -> std::process::ExitCode {
    let mut arguments = std::env::args();
    let _program = arguments.next();
    let Some(source) = arguments.next() else {
        eprintln!("usage: chronos-daemon '{{ main /run/user/1000/nexus/main /home/user/.local/state/nexus/main }}'");
        return std::process::ExitCode::FAILURE;
    };
    if arguments.next().is_some() {
        eprintln!("chronos-daemon accepts one Place datom");
        return std::process::ExitCode::FAILURE;
    }
    let Ok(place) = Place::from_text(&source) else {
        eprintln!("chronos-daemon requires a Place datom");
        return std::process::ExitCode::FAILURE;
    };
    if let Err(error) = std::fs::create_dir_all(place.runtime()) {
        eprintln!("Chronos could not prepare its runtime place: {error}");
        return std::process::ExitCode::FAILURE;
    }
    if place.install().is_err() {
        eprintln!("Chronos already has a startup place");
        return std::process::ExitCode::FAILURE;
    }
    chronos::daemon::Chronos::enter()
}
