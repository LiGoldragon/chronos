//! Round-trip + projection tests for the zodiac module.

use chronos::{EclipticLongitude, ZodiacSign, ZodiacalTime};

#[test]
fn aries_at_zero_degrees() {
    let longitude = EclipticLongitude::new(0.0);
    assert_eq!(ZodiacSign::from_longitude(longitude), ZodiacSign::Aries);
}

#[test]
fn taurus_at_thirty_degrees() {
    let longitude = EclipticLongitude::new(30.0);
    assert_eq!(ZodiacSign::from_longitude(longitude), ZodiacSign::Taurus);
}

#[test]
fn pisces_at_359_degrees() {
    let longitude = EclipticLongitude::new(359.0);
    assert_eq!(ZodiacSign::from_longitude(longitude), ZodiacSign::Pisces);
}

#[test]
fn ecliptic_longitude_wraps_into_range() {
    assert_eq!(EclipticLongitude::new(360.0).as_degrees(), 0.0);
    assert_eq!(EclipticLongitude::new(720.5).as_degrees(), 0.5);
    assert_eq!(EclipticLongitude::new(-1.0).as_degrees(), 359.0);
}

#[test]
fn zodiacal_time_at_15_degrees_aries() {
    let zt = ZodiacalTime::from_longitude(EclipticLongitude::new(15.5));
    assert_eq!(zt.sign, ZodiacSign::Aries);
    assert_eq!(zt.degree, 15);
    assert_eq!(zt.minute, 30);
}

#[test]
fn zodiacal_time_display_uses_unicode_symbol() {
    let zt = ZodiacalTime::from_longitude(EclipticLongitude::new(45.0));
    let rendered = format!("{zt}");
    assert!(rendered.contains('\u{2649}'), "expected ♉ in {rendered:?}");
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
