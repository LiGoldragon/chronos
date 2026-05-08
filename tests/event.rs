//! Round-trip tests for `SolarEventKind` and `SolarEvent`.

use chronos::{Location, SolarEvent, SolarEventKind};

#[test]
fn every_solar_event_kind_is_unique() {
    let kinds = [
        SolarEventKind::CivilDawn,
        SolarEventKind::Sunrise,
        SolarEventKind::SolarNoon,
        SolarEventKind::Sunset,
        SolarEventKind::CivilDusk,
    ];
    let mut sorted: Vec<_> = kinds.iter().map(|kind| format!("{kind:?}")).collect();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 5);
}

#[test]
fn solar_event_archives_round_trip() {
    let location = Location::from_degrees(47.6, -122.3).expect("valid location");
    let when = chronos::event::EpochTaiNanos::from_epoch(hifitime::Epoch::from_tai_seconds(1_000_000_000.0));
    let event = SolarEvent {
        kind: SolarEventKind::CivilDawn,
        when,
        at: location,
    };
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&event).expect("archive");
    let restored: SolarEvent = rkyv::from_bytes::<SolarEvent, rkyv::rancor::Error>(&bytes).expect("from_bytes");
    assert_eq!(restored, event);
}
