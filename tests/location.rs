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
fn non_finite_latitude_rejected() {
    let mut decoder = Decoder::new("nan");
    // NOTA may or may not parse `nan` as a float; if it doesn't,
    // the parser layer rejects before reaching try_new. Either
    // way, the wire never produces an in-band non-finite Latitude.
    let result = Latitude::decode(&mut decoder);
    assert!(result.is_err(), "expected error for non-finite input");
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
fn try_from_degrees_validates_both_axes() {
    assert!(Location::try_from_degrees(47.6, -122.3).is_ok());
    assert!(Location::try_from_degrees(91.0, 0.0).is_err()); // bad lat
    assert!(Location::try_from_degrees(0.0, 200.0).is_err()); // bad lon
}
