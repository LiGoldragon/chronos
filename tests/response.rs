//! Round-trip tests for the [`chronos::Response`] enum.
//!
//! Every variant must survive both wire formats:
//! DOTOS (the daemon's reply-rendering path) and rkyv (the
//! UDS frame path). The CLI's print loop renders responses
//! as DOTOS, so any breakage here breaks `chronos 'GetTime'`.

use chronos::{
    EclipticLongitude, EpochTaiNanos, Latitude, Location, LocationSource, Longitude, Response, SolarEvent,
    SolarEventKind, ZodiacalTime,
};

fn round_trip_dotos(response: &Response) -> Response {
    let text = response.to_dotos().expect("dotos encode");
    let again = Response::from_dotos(&text).expect("dotos decode");
    assert_eq!(*response, again, "dotos round-trip changed value via text {text:?}");
    again
}

fn round_trip_rkyv(response: &Response) {
    let bytes = response.archive().expect("archive");
    let restored = Response::from_archive(&bytes).expect("from_archive");
    assert_eq!(*response, restored, "rkyv round-trip changed value");
}

fn seattle() -> Location {
    Location { latitude: Latitude::try_new(47.6).unwrap(), longitude: Longitude::try_new(-122.3).unwrap() }
}

#[test]
fn acked() {
    let response = Response::Acked;
    round_trip_dotos(&response);
    round_trip_rkyv(&response);
}

#[test]
fn time() {
    let response =
        Response::Time { zodiacal_time: ZodiacalTime::from_longitude(EclipticLongitude::try_new(45.0).unwrap()) };
    round_trip_dotos(&response);
    round_trip_rkyv(&response);
}

#[test]
fn schedule_with_two_events() {
    let response = Response::Schedule {
        events: vec![
            SolarEvent {
                kind: SolarEventKind::Sunrise,
                when: EpochTaiNanos::from_epoch(hifitime::Epoch::from_tai_seconds(1_000_000_000.0)),
                location: seattle(),
            },
            SolarEvent {
                kind: SolarEventKind::Sunset,
                when: EpochTaiNanos::from_epoch(hifitime::Epoch::from_tai_seconds(1_000_050_000.0)),
                location: seattle(),
            },
        ],
    };
    round_trip_dotos(&response);
    round_trip_rkyv(&response);
}

#[test]
fn schedule_empty() {
    let response = Response::Schedule { events: vec![] };
    round_trip_dotos(&response);
    round_trip_rkyv(&response);
}

#[test]
fn location() {
    let response = Response::Location { location: seattle(), source: LocationSource::Manual };
    round_trip_dotos(&response);
    round_trip_rkyv(&response);
}

#[test]
fn event() {
    let response = Response::Event {
        event: SolarEvent {
            kind: SolarEventKind::CivilDawn,
            when: EpochTaiNanos::from_epoch(hifitime::Epoch::from_tai_seconds(1_000_000_000.0)),
            location: seattle(),
        },
    };
    round_trip_dotos(&response);
    round_trip_rkyv(&response);
}

#[test]
fn error_message() {
    let response = Response::Error { message: "ephemeris file not found".into() };
    round_trip_dotos(&response);
    round_trip_rkyv(&response);
}

#[test]
fn error_messages_with_apostrophes_do_not_require_quote_delimiters() {
    let response = Response::Error { message: "sky's ephemeris is missing".into() };
    let text = response.to_dotos().expect("dotos encode");
    assert_eq!(text, "Error.((sky's ephemeris is missing))");
    assert_eq!(Response::from_dotos(&text).expect("dotos decode"), response);
}
