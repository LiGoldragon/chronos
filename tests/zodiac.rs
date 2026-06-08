//! Round-trip + projection + validation tests for the zodiac
//! module.

use chronos::{EclipticLongitude, ZodiacDegree, ZodiacMinute, ZodiacSign, ZodiacalTime};
use nota_next::{NotaDecodeError, NotaEncode, NotaSource};

// ─── ZodiacSign ────────────────────────────────────────────

#[test]
fn aries_at_zero_degrees() {
    let longitude = EclipticLongitude::try_new(0.0).unwrap();
    assert_eq!(ZodiacSign::from_longitude(longitude), ZodiacSign::Aries);
}

#[test]
fn taurus_at_thirty_degrees() {
    let longitude = EclipticLongitude::try_new(30.0).unwrap();
    assert_eq!(ZodiacSign::from_longitude(longitude), ZodiacSign::Taurus);
}

#[test]
fn pisces_at_359_degrees() {
    let longitude = EclipticLongitude::try_new(359.0).unwrap();
    assert_eq!(ZodiacSign::from_longitude(longitude), ZodiacSign::Pisces);
}

#[test]
fn every_sign_has_a_unique_symbol() {
    let signs = [
        ZodiacSign::Aries,
        ZodiacSign::Taurus,
        ZodiacSign::Gemini,
        ZodiacSign::Cancer,
        ZodiacSign::Leo,
        ZodiacSign::Virgo,
        ZodiacSign::Libra,
        ZodiacSign::Scorpio,
        ZodiacSign::Sagittarius,
        ZodiacSign::Capricorn,
        ZodiacSign::Aquarius,
        ZodiacSign::Pisces,
    ];
    let mut symbols: Vec<char> = signs.iter().map(|sign| sign.symbol()).collect();
    symbols.sort_unstable();
    symbols.dedup();
    assert_eq!(symbols.len(), 12);
}

#[test]
fn zodiac_sign_nota_round_trips_for_every_variant() {
    for sign in [
        ZodiacSign::Aries,
        ZodiacSign::Taurus,
        ZodiacSign::Gemini,
        ZodiacSign::Cancer,
        ZodiacSign::Leo,
        ZodiacSign::Virgo,
        ZodiacSign::Libra,
        ZodiacSign::Scorpio,
        ZodiacSign::Sagittarius,
        ZodiacSign::Capricorn,
        ZodiacSign::Aquarius,
        ZodiacSign::Pisces,
    ] {
        let text = sign.to_nota();
        let decoded = NotaSource::new(&text).parse::<ZodiacSign>().expect("decode");
        assert_eq!(decoded, sign, "round-trip changed {sign:?} via text {text:?}");
    }
}

// ─── EclipticLongitude ─────────────────────────────────────

#[test]
fn ecliptic_longitude_strict_in_range() {
    assert!(EclipticLongitude::try_new(0.0).is_ok());
    assert!(EclipticLongitude::try_new(180.0).is_ok());
    assert!(EclipticLongitude::try_new(359.999).is_ok());
}

#[test]
fn ecliptic_longitude_360_rejected() {
    // Half-open [0, 360) — 360 is the same point as 0 and
    // must normalise via from_unnormalized_degrees first.
    assert!(EclipticLongitude::try_new(360.0).is_err());
}

#[test]
fn ecliptic_longitude_negative_rejected() {
    assert!(EclipticLongitude::try_new(-0.5).is_err());
}

#[test]
fn ecliptic_longitude_from_unnormalized_wraps() {
    assert_eq!(EclipticLongitude::from_unnormalized_degrees(360.0).as_degrees(), 0.0);
    assert_eq!(EclipticLongitude::from_unnormalized_degrees(720.5).as_degrees(), 0.5);
    assert_eq!(EclipticLongitude::from_unnormalized_degrees(-1.0).as_degrees(), 359.0);
}

#[test]
fn ecliptic_longitude_wire_validation() {
    let error = NotaSource::new("400.0").parse::<EclipticLongitude>().unwrap_err();
    match error {
        NotaDecodeError::InvalidValue { type_name, reason, .. } => {
            assert_eq!(type_name, "EclipticLongitude");
            assert!(reason.contains("[0, 360)"), "message was: {reason}");
        }
        other => panic!("expected InvalidValue error, got {other:?}"),
    }
}

// ─── ZodiacDegree / ZodiacMinute ───────────────────────────

#[test]
fn zodiac_degree_in_range() {
    assert!(ZodiacDegree::try_new(0).is_ok());
    assert!(ZodiacDegree::try_new(29).is_ok());
}

#[test]
fn zodiac_degree_30_rejected() {
    assert!(ZodiacDegree::try_new(30).is_err());
}

#[test]
fn zodiac_degree_wire_validation() {
    let error = NotaSource::new("30").parse::<ZodiacDegree>().unwrap_err();
    match error {
        NotaDecodeError::InvalidValue { type_name, reason, .. } => {
            assert_eq!(type_name, "ZodiacDegree");
            assert!(reason.contains("[0, 30)"), "message was: {reason}");
        }
        other => panic!("expected InvalidValue error, got {other:?}"),
    }
}

#[test]
fn zodiac_minute_in_range() {
    assert!(ZodiacMinute::try_new(0).is_ok());
    assert!(ZodiacMinute::try_new(59).is_ok());
}

#[test]
fn zodiac_minute_60_rejected() {
    assert!(ZodiacMinute::try_new(60).is_err());
}

// ─── ZodiacalTime ──────────────────────────────────────────

#[test]
fn zodiacal_time_at_15_30_aries() {
    let zt = ZodiacalTime::from_longitude(EclipticLongitude::try_new(15.5).unwrap());
    assert_eq!(zt.sign, ZodiacSign::Aries);
    assert_eq!(zt.degree.as_u8(), 15);
    assert_eq!(zt.minute.as_u8(), 30);
}

#[test]
fn zodiacal_time_display_uses_unicode_symbol() {
    let zt = ZodiacalTime::from_longitude(EclipticLongitude::try_new(45.0).unwrap());
    let rendered = format!("{zt}");
    assert!(rendered.contains('\u{2649}'), "expected ♉ in {rendered:?}");
}

#[test]
fn zodiacal_time_nota_round_trip() {
    let original = ZodiacalTime::from_longitude(EclipticLongitude::try_new(135.5).unwrap());
    let text = original.to_nota();
    let decoded = NotaSource::new(&text).parse::<ZodiacalTime>().expect("decode");
    assert_eq!(decoded, original);
}
