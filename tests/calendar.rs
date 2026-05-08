//! Round-trip + validation tests for the calendar module
//! (`AmYear`, `OrdinalSolarTime`).

use chronos::{AmYear, OrdinalSolarTime};
use nota_codec::{Decoder, Error as NotaError, NotaDecode};

// ─── AmYear ────────────────────────────────────────────────

#[test]
fn am_year_round_trips_through_inner() {
    let year = AmYear::new(5863);
    assert_eq!(year.as_i32(), 5863);
}

#[test]
fn am_year_handles_negative_and_zero() {
    assert_eq!(AmYear::new(-1).as_i32(), -1);
    assert_eq!(AmYear::new(0).as_i32(), 0);
}

#[test]
fn am_year_display_is_the_integer() {
    assert_eq!(format!("{}", AmYear::new(5863)), "5863");
    assert_eq!(format!("{}", AmYear::new(-1)), "-1");
}

// ─── OrdinalSolarTime ──────────────────────────────────────

#[test]
fn ordinal_solar_time_strict_in_range() {
    assert!(OrdinalSolarTime::try_new(0.0).is_ok());
    assert!(OrdinalSolarTime::try_new(0.5).is_ok());
    assert!(OrdinalSolarTime::try_new(0.999_999).is_ok());
}

#[test]
fn ordinal_solar_time_one_rejected() {
    assert!(OrdinalSolarTime::try_new(1.0).is_err());
}

#[test]
fn ordinal_solar_time_negative_rejected() {
    assert!(OrdinalSolarTime::try_new(-0.001).is_err());
}

#[test]
fn ordinal_solar_time_from_unnormalized_wraps() {
    assert_eq!(OrdinalSolarTime::from_unnormalized_fraction(1.0).as_fraction(), 0.0);
    assert_eq!(OrdinalSolarTime::from_unnormalized_fraction(2.5).as_fraction(), 0.5);
    assert_eq!(OrdinalSolarTime::from_unnormalized_fraction(-0.25).as_fraction(), 0.75);
}

#[test]
fn ordinal_solar_time_from_unnormalized_degrees_converts() {
    assert_eq!(OrdinalSolarTime::from_unnormalized_degrees(0.0).as_fraction(), 0.0);
    assert_eq!(OrdinalSolarTime::from_unnormalized_degrees(180.0).as_fraction(), 0.5);
    assert_eq!(OrdinalSolarTime::from_unnormalized_degrees(360.0).as_fraction(), 0.0);
}

#[test]
fn ordinal_solar_time_wire_validation() {
    let mut decoder = Decoder::new("1.5");
    let error = OrdinalSolarTime::decode(&mut decoder).unwrap_err();
    match error {
        NotaError::Validation { type_name, message } => {
            assert_eq!(type_name, "OrdinalSolarTime");
            assert!(message.contains("[0, 1)"), "message was: {message}");
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn ordinal_solar_time_as_degrees_is_consistent() {
    let value = OrdinalSolarTime::try_new(0.25).unwrap();
    assert_eq!(value.as_degrees(), 90.0);
}
