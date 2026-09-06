//! Current Datom witnesses for Chronos's hand-checked domain values.

use chronos::{
    AmYear, EclipticLongitude, EpochTaiNanos, ErrorMessage, Latitude, Location, LocationSource, Longitude,
    OrdinalSolarTime, Request, Response, SolarEvent, SolarEventKind, ZodiacDegree, ZodiacMinute, ZodiacSign,
    ZodiacalTime,
};
use datom_codec::{Actualizable, Datomic, IncorporationBudget, Potential, Problem, Textualizable};

fn incorporate<T: Datomic>(source: &str) -> T {
    Potential::<T>::from(source)
        .actualize(IncorporationBudget::try_from(4096).expect("positive budget"))
        .unwrap_or_else(|fault| panic!("fixture incorporates as one Datom ({source}): {fault:?}"))
}

fn round_trip<T: Datomic + std::fmt::Debug + PartialEq>(source: &str) {
    let value = incorporate::<T>(source);
    let rendered = value.textualize();
    assert_eq!(incorporate::<T>(&rendered), value, "canonical Datom changes value");
}

#[test]
fn domain_anatomies_round_trip_current_datom() {
    round_trip::<AmYear>("5863");
    round_trip::<OrdinalSolarTime>("0.5");
    round_trip::<Latitude>("47.6");
    round_trip::<Longitude>("-122.3");
    round_trip::<Location>("{47.6 -122.3}");
    round_trip::<LocationSource>("Geoclue");
    round_trip::<SolarEventKind>("Sunrise");
    round_trip::<EpochTaiNanos>("1000000000");
    round_trip::<SolarEvent>("{Sunrise 1000000000 {47.6 -122.3}}");
    round_trip::<EclipticLongitude>("45.5");
    round_trip::<ZodiacDegree>("15");
    round_trip::<ZodiacMinute>("30");
    round_trip::<ZodiacSign>("Taurus");
    round_trip::<ZodiacalTime>("{Taurus 15 30}");
}

#[test]
fn request_and_response_keep_the_cli_data_boundary() {
    round_trip::<Request>("GetTime");
    round_trip::<Request>("SetLocation.{47.6 -122.3}");
    round_trip::<Request>("Subscribe.{[CivilDawn CivilDusk]}");
    round_trip::<Response>("Acked");
    round_trip::<Response>("Time.{{Taurus 15 30}}");
    round_trip::<Response>("Schedule.{[{Sunrise 1000000000 {47.6 -122.3}}]}");
    round_trip::<Response>("Location.{{47.6 -122.3} Manual}");
    round_trip::<Response>("Event.{{CivilDawn 1000000000 {47.6 -122.3}}}");
    round_trip::<Response>("Error.{“sky missing”}");
}

#[test]
fn domain_readers_refuse_wrong_shapes_and_out_of_range_values() {
    refused::<Location>("{47.6}", false);
    refused::<Request>("SetLocation.{47.6}", false);
    refused::<Request>("Subscribe.[CivilDawn]", false);
    refused::<Response>("Time.{Taurus 15 30}", false);
    refused::<Latitude>("91.0", true);
    refused::<EclipticLongitude>("360.0", true);
    refused::<ZodiacDegree>("30", true);
}

fn refused<T: Datomic + std::fmt::Debug>(source: &str, value: bool) {
    let fault = Potential::<T>::from(source)
        .actualize(IncorporationBudget::try_from(4096).expect("positive budget"))
        .expect_err("invalid Datom is refused");
    if value {
        assert!(matches!(fault, datom_codec::Fault::Corporate(_, Problem::Value(_))), "{source}");
    }
}

#[test]
fn outbound_error_text_is_checked_as_datom_text() {
    let response = Response::Error { message: ErrorMessage::try_new("sky's ephemeris is missing".to_owned()).unwrap() };
    assert_eq!(response.textualize(), "Error.{ “sky's ephemeris is missing” }");
}

#[test]
fn error_response_requires_a_text_payload() {
    let result = Potential::<Response>::from("Error.{ sky missing }")
        .actualize(IncorporationBudget::try_from(4096).expect("positive budget"))
        .is_err();
    assert!(result, "bare words cannot stand in for a typed error text payload");
}
