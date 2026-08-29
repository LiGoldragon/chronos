//! Round-trip tests for the [`chronos::Request`] enum.
//!
//! Every variant must survive both wire formats:
//! Datomic (the CLI's argv-embody path) and rkyv (the daemon's
//! UDS frame).

use chronos::{Latitude, Longitude, Request, SolarEventKind};

fn round_trip_datomic(text: &str) -> Request {
    let request = Request::from_text(text).expect("Datomic embody");
    let rendered = request.to_text();
    let again = Request::from_text(&rendered).expect("Datomic re-embody");
    assert_eq!(request, again, "Datomic round-trip changed value");
    request
}

fn round_trip_rkyv(request: &Request) {
    let bytes = request.archive().expect("archive");
    let restored = Request::from_archive(&bytes).expect("from_archive");
    assert_eq!(*request, restored, "rkyv round-trip changed value");
}

#[test]
fn get_time() {
    let request = round_trip_datomic("GetTime");
    assert_eq!(request, Request::GetTime);
    round_trip_rkyv(&request);
}

#[test]
fn get_schedule() {
    let request = round_trip_datomic("GetSchedule");
    assert_eq!(request, Request::GetSchedule);
    round_trip_rkyv(&request);
}

#[test]
fn get_location() {
    let request = round_trip_datomic("GetLocation");
    assert_eq!(request, Request::GetLocation);
    round_trip_rkyv(&request);
}

#[test]
fn set_location_seattle() {
    let request = round_trip_datomic("SetLocation.{47.6 -122.3}");
    assert_eq!(
        request,
        Request::SetLocation {
            latitude: Latitude::try_new(47.6).unwrap(),
            longitude: Longitude::try_new(-122.3).unwrap(),
        }
    );
    round_trip_rkyv(&request);
}

#[test]
fn use_geoclue() {
    let request = round_trip_datomic("UseGeoclue");
    assert_eq!(request, Request::UseGeoclue);
    round_trip_rkyv(&request);
}

#[test]
fn subscribe_civil_dawn_and_dusk() {
    let request = round_trip_datomic("Subscribe.{[CivilDawn CivilDusk]}");
    assert_eq!(request, Request::Subscribe { kinds: vec![SolarEventKind::CivilDawn, SolarEventKind::CivilDusk] });
    round_trip_rkyv(&request);
}
