//! Every archive entry admits only what `try_new` admits: the six ranged scalars refuse an
//! out-of-range, non-finite or otherwise invalid value when rkyv reconstructs them, directly,
//! nested in a container, and through the Request and Response frame entry points.
//! Each unvalidated mirror below has the layout of the type it stands for.

use chronos::{
    ArchiveFault, EclipticLongitude, EpochTaiNanos, Error, ErrorMessage, Latitude, Location, LocationSource, Longitude,
    OrdinalSolarTime, Request, Response, ResponseError, ResponseLocation, SolarEvent, SolarEventKind, ZodiacDegree,
    ZodiacMinute, ZodiacSign, ZodiacalTime,
};
use rkyv::{Archive, Serialize as RkyvSerialize};

#[derive(Archive, RkyvSerialize)]
struct RawFloat(f64);
#[derive(Archive, RkyvSerialize)]
struct RawByte(u8);
#[derive(Archive, RkyvSerialize)]
struct RawLocation {
    latitude: RawFloat,
    longitude: RawFloat,
}
#[derive(Archive, RkyvSerialize)]
struct RawZodiacalTime {
    sign: ZodiacSign,
    degree: RawByte,
    minute: RawByte,
}
#[derive(Archive, RkyvSerialize)]
struct RawSolarEvent {
    kind: SolarEventKind,
    when: EpochTaiNanos,
    location: RawLocation,
}
#[derive(Archive, RkyvSerialize)]
struct RawSetLocation {
    latitude: RawFloat,
    longitude: RawFloat,
}
#[derive(Archive, RkyvSerialize)]
struct RawSubscribe {
    kinds: Vec<SolarEventKind>,
}
#[derive(Archive, RkyvSerialize)]
#[expect(dead_code, reason = "the unconstructed variants fix the discriminants of the layout mirrored")]
enum RawRequest {
    GetTime,
    GetSchedule,
    GetLocation,
    SetLocation(RawSetLocation),
    UseGeoclue,
    Subscribe(RawSubscribe),
}
#[derive(Archive, RkyvSerialize)]
struct RawTime {
    zodiacal_time: RawZodiacalTime,
}
#[derive(Archive, RkyvSerialize)]
struct RawSchedule {
    events: Vec<RawSolarEvent>,
}
#[derive(Archive, RkyvSerialize)]
#[expect(dead_code, reason = "the unconstructed variants fix the discriminants of the layout mirrored")]
enum RawResponse {
    Acked,
    Time(RawTime),
    Schedule(RawSchedule),
    Location(ResponseLocation),
    Event(chronos::ResponseEvent),
    Error(ResponseError),
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

fn read<T>(bytes: &[u8]) -> Result<T, ArchiveFault>
where
    T: Archive,
    T::Archived: rkyv::Deserialize<T, rkyv::rancor::Strategy<rkyv::de::Pool, ArchiveFault>>
        + for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, ArchiveFault>>,
{
    rkyv::from_bytes::<T, ArchiveFault>(bytes)
}

fn refused<T>(bytes: &[u8], type_name: &str)
where
    T: Archive + std::fmt::Debug,
    T::Archived: rkyv::Deserialize<T, rkyv::rancor::Strategy<rkyv::de::Pool, ArchiveFault>>
        + for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, ArchiveFault>>,
{
    match read::<T>(bytes) {
        Err(ArchiveFault::Refused(Error::OutOfRange { type_name: named, .. })) => assert_eq!(named, type_name),
        other => panic!("expected a typed {type_name} refusal, got {other:?}"),
    }
}

const FLOATS_BEYOND: [f64; 5] = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1.0e308, -1.0e308];

macro_rules! float_scalar_is_admitted_only_by_try_new {
    ($test:ident, $type:ident, $admitted:expr, $beyond:expr) => {
        #[test]
        fn $test() {
            for value in $admitted {
                let admitted: $type =
                    read(&bytes(&RawFloat(value))).unwrap_or_else(|fault| panic!("{value}: {fault:?}"));
                assert_eq!(admitted, $type::try_new(value).unwrap());
            }
            for value in $beyond.into_iter().chain(FLOATS_BEYOND) {
                refused::<$type>(&bytes(&RawFloat(value)), stringify!($type));
            }
        }
    };
}

float_scalar_is_admitted_only_by_try_new!(latitude, Latitude, [-90.0, 0.0, 47.6, 90.0], [90.000001, -90.000001]);
float_scalar_is_admitted_only_by_try_new!(longitude, Longitude, [-180.0, 0.0, 180.0], [180.000001, -180.000001]);
float_scalar_is_admitted_only_by_try_new!(ordinal_solar_time, OrdinalSolarTime, [0.0, 0.5, 0.999999], [1.0, -0.000001]);
float_scalar_is_admitted_only_by_try_new!(
    ecliptic_longitude,
    EclipticLongitude,
    [0.0, 180.0, 359.999999],
    [360.0, -0.000001]
);

#[test]
fn zodiac_degree_and_minute_admit_only_their_ranges() {
    for value in [0u8, 15, 29] {
        assert_eq!(read::<ZodiacDegree>(&bytes(&RawByte(value))).unwrap(), ZodiacDegree::try_new(value).unwrap());
    }
    for value in [30u8, 59, 255] {
        refused::<ZodiacDegree>(&bytes(&RawByte(value)), "ZodiacDegree");
    }
    for value in [0u8, 30, 59] {
        assert_eq!(read::<ZodiacMinute>(&bytes(&RawByte(value))).unwrap(), ZodiacMinute::try_new(value).unwrap());
    }
    for value in [60u8, 100, 255] {
        refused::<ZodiacMinute>(&bytes(&RawByte(value)), "ZodiacMinute");
    }
}

#[test]
fn a_nested_invalid_value_is_refused_in_every_container() {
    let invalid = || RawLocation { latitude: RawFloat(47.6), longitude: RawFloat(181.0) };
    refused::<Location>(&bytes(&invalid()), "Longitude");
    refused::<SolarEvent>(
        &bytes(&RawSolarEvent {
            kind: SolarEventKind::Sunrise,
            when: EpochTaiNanos::from_epoch(hifitime::Epoch::from_tai_seconds(1.0)),
            location: invalid(),
        }),
        "Longitude",
    );
    refused::<ZodiacalTime>(
        &bytes(&RawZodiacalTime { sign: ZodiacSign::Taurus, degree: RawByte(30), minute: RawByte(0) }),
        "ZodiacDegree",
    );
    let admitted: Location =
        read(&bytes(&RawLocation { latitude: RawFloat(47.6), longitude: RawFloat(-122.3) })).unwrap();
    assert_eq!(admitted.latitude, Latitude::try_new(47.6).unwrap());
}

#[test]
fn the_frame_entry_points_refuse_with_the_typed_error() {
    let set = bytes(&RawRequest::SetLocation(RawSetLocation { latitude: RawFloat(200.0), longitude: RawFloat(0.0) }));
    assert!(matches!(Request::from_archive(&set), Err(Error::OutOfRange { type_name: "Latitude", .. })));
    let nan =
        bytes(&RawRequest::SetLocation(RawSetLocation { latitude: RawFloat(f64::NAN), longitude: RawFloat(0.0) }));
    assert!(matches!(Request::from_archive(&nan), Err(Error::OutOfRange { type_name: "Latitude", .. })));
    let time = bytes(&RawResponse::Time(RawTime {
        zodiacal_time: RawZodiacalTime { sign: ZodiacSign::Aries, degree: RawByte(99), minute: RawByte(0) },
    }));
    assert!(matches!(Response::from_archive(&time), Err(Error::OutOfRange { type_name: "ZodiacDegree", .. })));
    let schedule = bytes(&RawResponse::Schedule(RawSchedule {
        events: vec![RawSolarEvent {
            kind: SolarEventKind::Sunset,
            when: EpochTaiNanos::from_epoch(hifitime::Epoch::from_tai_seconds(1.0)),
            location: RawLocation { latitude: RawFloat(91.0), longitude: RawFloat(0.0) },
        }],
    }));
    assert!(matches!(Response::from_archive(&schedule), Err(Error::OutOfRange { type_name: "Latitude", .. })));
}

#[test]
fn valid_frames_and_garbage_still_behave() {
    let good =
        bytes(&RawRequest::SetLocation(RawSetLocation { latitude: RawFloat(47.6), longitude: RawFloat(-122.3) }));
    assert!(matches!(Request::from_archive(&good), Ok(Request::SetLocation(_))));
    let message = bytes(&RawResponse::Error(ResponseError { message: ErrorMessage::new("sky missing".into()) }));
    assert!(Response::from_archive(&message).is_ok());
    assert!(matches!(Response::from_archive(&[1, 2, 3]), Err(Error::RkyvCodec(_))));
    let _ = LocationSource::Manual;
}
