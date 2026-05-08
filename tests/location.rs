//! Pins the wire-validation contract for [`Latitude`] and
//! [`Longitude`] — decode rejects out-of-range values with
//! `nota_codec::Error::Validation { type_name, message }`,
//! same shape as `repos/nota-codec/tests/nota_try_transparent_round_trip.rs`
//! pins for the canonical `ShortHex` example.

use chronos::{Latitude, Location, Longitude};
use nota_codec::{Decoder, Error as NotaError, NotaDecode};

#[test]
fn valid_latitude_decodes_through_try_new() {
    let mut decoder = Decoder::new("47.6");
    let value = Latitude::decode(&mut decoder).unwrap();
    assert_eq!(value, Latitude::try_new(47.6).unwrap());
}

#[test]
fn out_of_range_latitude_rejected_at_wire() {
    let mut decoder = Decoder::new("200.0");
    let error = Latitude::decode(&mut decoder).unwrap_err();
    match error {
        NotaError::Validation { type_name, message } => {
            assert_eq!(type_name, "Latitude");
            assert!(message.contains("[-90, 90]"), "message was: {message}");
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn out_of_range_longitude_rejected_at_wire() {
    let mut decoder = Decoder::new("-400.0");
    let error = Longitude::decode(&mut decoder).unwrap_err();
    match error {
        NotaError::Validation { type_name, message } => {
            assert_eq!(type_name, "Longitude");
            assert!(message.contains("[-180, 180]"), "message was: {message}");
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn latitude_boundary_90_accepted() {
    assert!(Latitude::try_new(90.0).is_ok());
    assert!(Latitude::try_new(-90.0).is_ok());
}

#[test]
fn latitude_just_past_boundary_rejected() {
    assert!(Latitude::try_new(90.000_001).is_err());
    assert!(Latitude::try_new(-90.000_001).is_err());
}

#[test]
fn longitude_boundary_180_accepted() {
    assert!(Longitude::try_new(180.0).is_ok());
    assert!(Longitude::try_new(-180.0).is_ok());
}

#[test]
fn longitude_just_past_boundary_rejected() {
    assert!(Longitude::try_new(180.000_001).is_err());
    assert!(Longitude::try_new(-180.000_001).is_err());
}

#[test]
fn nan_latitude_rejected_in_constructor() {
    assert!(Latitude::try_new(f64::NAN).is_err());
}

#[test]
fn infinite_longitude_rejected_in_constructor() {
    assert!(Longitude::try_new(f64::INFINITY).is_err());
    assert!(Longitude::try_new(f64::NEG_INFINITY).is_err());
}

#[test]
fn from_self_for_inner_is_emitted_for_latitude() {
    let value = Latitude::try_new(47.6).unwrap();
    let inner: f64 = value.into();
    assert_eq!(inner, 47.6);
}

#[test]
fn location_decode_propagates_field_validation() {
    // (Location 200.0 0.0) — invalid latitude inside an
    // otherwise-valid Location record. NotaRecord delegates
    // per-field decode, so the latitude field's NotaTryTransparent
    // validation surfaces before any Location is constructed.
    let mut decoder = Decoder::new("(Location 200.0 0.0)");
    let error = Location::decode(&mut decoder).unwrap_err();
    match error {
        NotaError::Validation { type_name, .. } => {
            assert_eq!(type_name, "Latitude");
        }
        other => panic!("expected Validation error from latitude field, got {other:?}"),
    }
}

#[test]
fn location_field_init_with_validated_newtypes() {
    // Programmatic construction goes through field-init with
    // pre-validated newtypes — there is no two-arg
    // `Location::try_from_degrees` (per skills/rust-discipline.md
    // §"One object in, one object out").
    let location = Location {
        latitude: Latitude::try_new(47.6).unwrap(),
        longitude: Longitude::try_new(-122.3).unwrap(),
    };
    assert_eq!(location.latitude.as_degrees(), 47.6);
    assert_eq!(location.longitude.as_degrees(), -122.3);
}
