//! Current Datom witnesses for Chronos's checked domain values: every text
//! entry admits only what the value's one constructor, `try_new`, admits.

mod support;
use chronos::{
    AmYear, EclipticLongitude, EpochTaiNanos, ErrorMessage, Latitude, Location, LocationSource, Longitude,
    OrdinalSolarTime, Request, Response, SolarEvent, SolarEventKind, ZodiacDegree, ZodiacMinute, ZodiacSign,
    ZodiacalTime,
};
use datom_codec::{Composing, Datomizable, ErrorKind};

fn round_trip<T: Composing + Datomizable + std::fmt::Debug + PartialEq>(source: &str) {
    let value = support::read::<T>(source).unwrap_or_else(|error| panic!("fixture reads ({source}): {error:?}"));
    let rendered = support::print(&value);
    assert_eq!(support::read::<T>(&rendered).expect("the rendering reads back"), value, "{source}");
}

fn refused<T: Composing + std::fmt::Debug>(source: &str) -> datom_codec::Error {
    support::read::<T>(source).expect_err(source)
}

fn value_refusal<T: Composing + std::fmt::Debug>(source: &str, expected: &str) {
    let error = refused::<T>(source);
    match error.kind {
        ErrorKind::Value { expected: named, .. } => {
            assert_eq!(named, expected, "{source}");
        }
        other => panic!("{source}: expected a Value refusal, got {other:?}"),
    }
}

#[test]
fn domain_anatomies_round_trip_current_datom() {
    round_trip::<AmYear>("5863");
    round_trip::<OrdinalSolarTime>("0.5");
    round_trip::<Latitude>("47.6");
    round_trip::<Longitude>("-122.3");
    round_trip::<Location>("{ 47.6 -122.3 }");
    round_trip::<LocationSource>("Geoclue");
    round_trip::<SolarEventKind>("Sunrise");
    round_trip::<EpochTaiNanos>("1000000000");
    round_trip::<SolarEvent>("{ Sunrise 1000000000 { 47.6 -122.3 } }");
    round_trip::<EclipticLongitude>("45.5");
    round_trip::<ZodiacDegree>("15");
    round_trip::<ZodiacMinute>("30");
    round_trip::<ZodiacSign>("Taurus");
    round_trip::<ZodiacalTime>("{ Taurus 15 30 }");
}

#[test]
fn request_and_response_keep_the_cli_data_boundary() {
    round_trip::<Request>("GetTime");
    round_trip::<Request>("SetLocation.{ 47.6 -122.3 }");
    round_trip::<Request>("Subscribe.{ [ CivilDawn CivilDusk ] }");
    round_trip::<Response>("Acked");
    round_trip::<Response>("Time.{ { Taurus 15 30 } }");
    round_trip::<Response>("Schedule.{ [ { Sunrise 1000000000 { 47.6 -122.3 } } ] }");
    round_trip::<Response>("Location.{ { 47.6 -122.3 } Manual }");
    round_trip::<Response>("Event.{ { CivilDawn 1000000000 { 47.6 -122.3 } } }");
    round_trip::<Response>("Error.{ «sky missing» }");
}

#[test]
fn wrong_shapes_are_refused() {
    refused::<Location>("{ 47.6 }");
    refused::<Request>("SetLocation.{ 47.6 }");
    refused::<Request>("Subscribe.[ CivilDawn ]");
    refused::<Response>("Time.{ Taurus 15 30 }");
}

#[test]
fn every_ranged_scalar_admits_its_boundaries_and_refuses_beyond() {
    support::read::<Latitude>("-90.0").unwrap();
    support::read::<Latitude>("90.0").unwrap();
    value_refusal::<Latitude>("-90.000001", "Latitude");
    value_refusal::<Latitude>("90.000001", "Latitude");
    support::read::<Longitude>("-180.0").unwrap();
    support::read::<Longitude>("180.0").unwrap();
    value_refusal::<Longitude>("-180.000001", "Longitude");
    value_refusal::<Longitude>("180.000001", "Longitude");
    support::read::<OrdinalSolarTime>("0.0").unwrap();
    support::read::<OrdinalSolarTime>("0.999999").unwrap();
    value_refusal::<OrdinalSolarTime>("1.0", "OrdinalSolarTime");
    value_refusal::<OrdinalSolarTime>("-0.000001", "OrdinalSolarTime");
    support::read::<EclipticLongitude>("0.0").unwrap();
    support::read::<EclipticLongitude>("359.999999").unwrap();
    value_refusal::<EclipticLongitude>("360.0", "EclipticLongitude");
    value_refusal::<EclipticLongitude>("-0.000001", "EclipticLongitude");
    support::read::<ZodiacDegree>("0").unwrap();
    support::read::<ZodiacDegree>("29").unwrap();
    value_refusal::<ZodiacDegree>("30", "ZodiacDegree");
    value_refusal::<ZodiacDegree>("-1", "ZodiacDegree");
    value_refusal::<ZodiacDegree>("256", "ZodiacDegree");
    support::read::<ZodiacMinute>("0").unwrap();
    support::read::<ZodiacMinute>("59").unwrap();
    value_refusal::<ZodiacMinute>("60", "ZodiacMinute");
    value_refusal::<ZodiacMinute>("-1", "ZodiacMinute");
    support::read::<AmYear>("2147483647").unwrap();
    support::read::<AmYear>("-2147483648").unwrap();
    value_refusal::<AmYear>("2147483648", "AmYear");
    value_refusal::<AmYear>("-2147483649", "AmYear");
}

#[test]
fn non_finite_and_non_numeric_inputs_are_refused() {
    for source in ["NaN", "inf", "-inf", "Infinity", "latitude", "«47.6»"] {
        refused::<Latitude>(source);
        refused::<Longitude>(source);
        refused::<OrdinalSolarTime>(source);
        refused::<EclipticLongitude>(source);
    }
    refused::<AmYear>("1.5");
    refused::<ZodiacDegree>("1.5");
}

#[test]
fn a_refusal_is_positioned_at_the_nested_value() {
    let error = refused::<Location>("{ 47.6 -200.0 }");
    assert_eq!(error.path, vec![1], "the longitude is the second position");
    assert!(matches!(error.kind, ErrorKind::Value { .. }));
    let error = refused::<Request>("SetLocation.{ 91.0 -122.3 }");
    assert_eq!(error.path, vec![1, 0], "the body of the variant is child one; the latitude is position zero");
    let error = refused::<Response>("Event.{ { CivilDawn 1000000000 { 47.6 181.0 } } }");
    assert!(matches!(error.kind, ErrorKind::Value { .. }));
    let error = refused::<Response>("Time.{ { Taurus 30 0 } }");
    assert!(matches!(error.kind, ErrorKind::Value { .. }));
    let error = refused::<Response>("Schedule.{ [ { Sunrise 1 { 91.0 0.0 } } ] }");
    assert!(matches!(error.kind, ErrorKind::Value { .. }));
}

#[test]
fn outbound_error_text_is_datom_text() {
    let response =
        Response::Error(chronos::ResponseError { message: ErrorMessage::new("sky's ephemeris is missing".to_owned()) });
    assert_eq!(support::print(&response), "Error.{ «sky's ephemeris is missing» }");
}

#[test]
fn error_response_requires_a_text_payload() {
    let error = support::read::<Response>("Error.{ sky missing }");
    assert!(error.is_err(), "two bare words cannot stand in for one text payload");
}
