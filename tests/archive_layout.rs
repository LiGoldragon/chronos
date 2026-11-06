//! The 0.4.0 payload-struct variants archive to the same bytes as the 0.3.0
//! struct variants, and the 0.3.0 datom texts read to the same values.

mod support;
use chronos::{
    EclipticLongitude, EpochTaiNanos, ErrorMessage, Latitude, Location, LocationSource, Longitude, Request,
    RequestSetLocation, RequestSubscribe, Response, ResponseError, ResponseEvent, ResponseLocation, ResponseSchedule,
    ResponseTime, SolarEvent, SolarEventKind, ZodiacalTime,
};
use rkyv::{Archive, Serialize as RkyvSerialize};

/// `Request` as 0.3.0 declared it, with struct variants.
#[derive(Archive, RkyvSerialize)]
enum RequestZeroThree {
    GetTime,
    GetSchedule,
    GetLocation,
    SetLocation { latitude: Latitude, longitude: Longitude },
    UseGeoclue,
    Subscribe { kinds: Vec<SolarEventKind> },
}

/// `Response` as 0.3.0 declared it, with struct variants.
#[derive(Archive, RkyvSerialize)]
enum ResponseZeroThree {
    Acked,
    Time { zodiacal_time: ZodiacalTime },
    Schedule { events: Vec<SolarEvent> },
    Location { location: Location, source: LocationSource },
    Event { event: SolarEvent },
    Error { message: ErrorMessage },
}

fn bytes<T>(value: &T) -> Vec<u8>
where
    T: for<'a> RkyvSerialize<
        rkyv::api::high::HighSerializer<
            rkyv::util::AlignedVec,
            rkyv::ser::allocator::ArenaHandle<'a>,
            rkyv::rancor::Error,
        >,
    >,
{
    rkyv::to_bytes::<rkyv::rancor::Error>(value).expect("archive").to_vec()
}

fn seattle() -> Location {
    Location { latitude: Latitude::try_new(47.6).unwrap(), longitude: Longitude::try_new(-122.3).unwrap() }
}

fn event() -> SolarEvent {
    SolarEvent {
        kind: SolarEventKind::Sunrise,
        when: EpochTaiNanos::from_epoch(hifitime::Epoch::from_tai_seconds(1.0e9)),
        location: seattle(),
    }
}

#[test]
fn request_archives_to_the_same_bytes_as_zero_three() {
    let latitude = Latitude::try_new(47.6).unwrap();
    let longitude = Longitude::try_new(-122.3).unwrap();
    let kinds = vec![SolarEventKind::CivilDawn, SolarEventKind::CivilDusk];
    assert_eq!(
        bytes(&Request::SetLocation(RequestSetLocation { latitude, longitude })),
        bytes(&RequestZeroThree::SetLocation { latitude, longitude })
    );
    assert_eq!(
        bytes(&Request::Subscribe(RequestSubscribe { kinds: kinds.clone() })),
        bytes(&RequestZeroThree::Subscribe { kinds })
    );
    assert_eq!(bytes(&Request::GetTime), bytes(&RequestZeroThree::GetTime));
    assert_eq!(bytes(&Request::GetSchedule), bytes(&RequestZeroThree::GetSchedule));
    assert_eq!(bytes(&Request::GetLocation), bytes(&RequestZeroThree::GetLocation));
    assert_eq!(bytes(&Request::UseGeoclue), bytes(&RequestZeroThree::UseGeoclue));
}

#[test]
fn response_archives_to_the_same_bytes_as_zero_three() {
    let zodiacal_time = ZodiacalTime::from_longitude(EclipticLongitude::try_new(45.0).unwrap());
    assert_eq!(
        bytes(&Response::Time(ResponseTime { zodiacal_time })),
        bytes(&ResponseZeroThree::Time { zodiacal_time })
    );
    assert_eq!(
        bytes(&Response::Schedule(ResponseSchedule { events: vec![event()] })),
        bytes(&ResponseZeroThree::Schedule { events: vec![event()] })
    );
    assert_eq!(
        bytes(&Response::Location(ResponseLocation { location: seattle(), source: LocationSource::Manual })),
        bytes(&ResponseZeroThree::Location { location: seattle(), source: LocationSource::Manual })
    );
    assert_eq!(
        bytes(&Response::Event(ResponseEvent { event: event() })),
        bytes(&ResponseZeroThree::Event { event: event() })
    );
    assert_eq!(
        bytes(&Response::Error(ResponseError { message: ErrorMessage::new("sky missing".into()) })),
        bytes(&ResponseZeroThree::Error { message: ErrorMessage::new("sky missing".into()) })
    );
    assert_eq!(bytes(&Response::Acked), bytes(&ResponseZeroThree::Acked));
}

/// Each 0.3.0 text, as its own tests wrote it, reads to the same value as the 0.4.0 spelling.
#[test]
fn zero_three_texts_read_to_the_same_values() {
    for (old, current) in [
        ("SetLocation.{47.6 -122.3}", "SetLocation.{ 47.6 -122.3 }"),
        ("Subscribe.{[CivilDawn CivilDusk]}", "Subscribe.{ [ CivilDawn CivilDusk ] }"),
    ] {
        assert_eq!(support::read::<Request>(old).unwrap(), support::read::<Request>(current).unwrap(), "{old}");
    }
    for (old, current) in [
        ("Time.{{Taurus 15 30}}", "Time.{ { Taurus 15 30 } }"),
        ("Location.{{47.6 -122.3} Manual}", "Location.{ { 47.6 -122.3 } Manual }"),
        ("Schedule.{[{Sunrise 1000000000 {47.6 -122.3}}]}", "Schedule.{ [ { Sunrise 1000000000 { 47.6 -122.3 } } ] }"),
        ("Event.{{CivilDawn 1000000000 {47.6 -122.3}}}", "Event.{ { CivilDawn 1000000000 { 47.6 -122.3 } } }"),
    ] {
        assert_eq!(support::read::<Response>(old).unwrap(), support::read::<Response>(current).unwrap(), "{old}");
    }
}
