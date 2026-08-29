//! Parsed-Protos D3 witnesses for Chronos's hand-checked domain values.

use chronos::{
    AmYear, EclipticLongitude, EpochTaiNanos, ErrorMessage, Latitude, Location, LocationSource, Longitude,
    OrdinalSolarTime, Request, Response, SolarEvent, SolarEventKind, ZodiacDegree, ZodiacMinute, ZodiacSign,
    ZodiacalTime,
};
use datomic::{Datomic, FaultProblem, Text, TextEdge};

fn embody<T: Datomic>(source: &str) -> T {
    Text::<T>::from(source).embody().expect("the fixture embodies through one parsed Portion")
}

fn round_trip<T: Datomic>(source: &str) {
    let value = embody::<T>(source);
    assert_eq!(value.textualize().as_ref(), source, "canonical Datomic text survives its checked D3 anatomy");
}

#[test]
fn hand_d3_domain_anatomies_round_trip_parsed_portions() {
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
fn request_and_response_keep_the_canonical_cli_grammar() {
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
fn hand_d3_refuses_wrong_shapes_and_out_of_range_values() {
    refused::<Location>("{47.6}", FaultProblem::Arity);
    refused::<Request>("SetLocation.{47.6}", FaultProblem::Arity);
    refused::<Request>("Subscribe.[CivilDawn]", FaultProblem::Shape);
    refused::<Response>("Time.{Taurus 15 30}", FaultProblem::Arity);
    refused::<Latitude>("91.0", FaultProblem::Value);
    refused::<EclipticLongitude>("360.0", FaultProblem::Value);
    refused::<ZodiacDegree>("30", FaultProblem::Value);
}

fn refused<T: Datomic + std::fmt::Debug>(source: &str, expected: FaultProblem) {
    let fault = Text::<T>::from(source).embody().expect_err("invalid D3 anatomy is refused");
    assert!(
        matches!(
            (fault.problem, expected),
            (FaultProblem::Shape, FaultProblem::Shape)
                | (FaultProblem::Arity, FaultProblem::Arity)
                | (FaultProblem::Value, FaultProblem::Value)
        ),
        "{source} was refused as a different fault"
    );
}

#[test]
fn outbound_error_text_is_checked_as_a_datomic_string() {
    let response = Response::Error { message: ErrorMessage::try_new("sky's ephemeris is missing".to_owned()).unwrap() };
    assert_eq!(response.textualize().as_ref(), "Error.{“sky's ephemeris is missing”}");
}

#[test]
fn error_message_rejects_unrepresentable_text_before_outbound_datomic() {
    assert!(ErrorMessage::try_new("unbalanced “ curly".to_owned()).is_err());
}
