//! Round-trip + validation tests for `SolarEventKind` and
//! `SolarEvent` — both wire formats (NOTA + rkyv).

use chronos::{EpochTaiNanos, Latitude, Location, Longitude, SolarEvent, SolarEventKind};
use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};

fn seattle() -> Location {
    Location {
        latitude: Latitude::try_new(47.6).unwrap(),
        longitude: Longitude::try_new(-122.3).unwrap(),
    }
}

fn sample_event(kind: SolarEventKind) -> SolarEvent {
    SolarEvent {
        kind,
        when: EpochTaiNanos::from_epoch(hifitime::Epoch::from_tai_seconds(1_000_000_000.0)),
        location: seattle(),
    }
}

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
fn solar_event_kind_nota_round_trips_for_every_variant() {
    for kind in [
        SolarEventKind::CivilDawn,
        SolarEventKind::Sunrise,
        SolarEventKind::SolarNoon,
        SolarEventKind::Sunset,
        SolarEventKind::CivilDusk,
    ] {
        let mut encoder = Encoder::new();
        kind.encode(&mut encoder).expect("encode");
        let text = encoder.into_string();
        let mut decoder = Decoder::new(&text);
        let decoded = SolarEventKind::decode(&mut decoder).expect("decode");
        assert_eq!(decoded, kind, "round-trip changed {kind:?} via text {text:?}");
    }
}

#[test]
fn solar_event_archives_round_trip() {
    let event = sample_event(SolarEventKind::CivilDawn);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&event).expect("archive");
    let restored: SolarEvent = rkyv::from_bytes::<SolarEvent, rkyv::rancor::Error>(&bytes).expect("from_bytes");
    assert_eq!(restored, event);
}

#[test]
fn solar_event_nota_round_trip() {
    let event = sample_event(SolarEventKind::CivilDusk);
    let mut encoder = Encoder::new();
    event.encode(&mut encoder).expect("encode");
    let text = encoder.into_string();
    let mut decoder = Decoder::new(&text);
    let decoded = SolarEvent::decode(&mut decoder).expect("decode");
    assert_eq!(decoded, event);
}

#[test]
fn epoch_tai_nanos_round_trips_through_hifitime() {
    let original_epoch = hifitime::Epoch::from_tai_seconds(1_234_567_890.5);
    let nanos = EpochTaiNanos::from_epoch(original_epoch);
    let restored_epoch = nanos.to_epoch();
    assert_eq!(restored_epoch, original_epoch);
}
