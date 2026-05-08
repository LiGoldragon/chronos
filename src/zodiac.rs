//! Zodiacal projection — [`ZodiacSign`], [`EclipticLongitude`],
//! [`ZodiacalTime`].
//!
//! The sun's apparent ecliptic longitude is what the prototype
//! called "ordinal solar time"; chronos projects it onto the
//! twelve-sign zodiac for human-facing display.

use core::fmt;

use nota_codec::{NotaEnum, NotaRecord, NotaTransparent};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

/// One of the twelve zodiac signs, in source-declaration
/// order matching the standard zodiacal sequence (Aries first,
/// Pisces last).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl ZodiacSign {
    /// The Unicode symbol for this sign (♈ … ♓).
    pub fn symbol(&self) -> char {
        match self {
            Self::Aries => '\u{2648}',
            Self::Taurus => '\u{2649}',
            Self::Gemini => '\u{264A}',
            Self::Cancer => '\u{264B}',
            Self::Leo => '\u{264C}',
            Self::Virgo => '\u{264D}',
            Self::Libra => '\u{264E}',
            Self::Scorpio => '\u{264F}',
            Self::Sagittarius => '\u{2650}',
            Self::Capricorn => '\u{2651}',
            Self::Aquarius => '\u{2652}',
            Self::Pisces => '\u{2653}',
        }
    }

    /// Construct from an ecliptic longitude; each sign owns
    /// 30° in standard order, starting at 0° = Aries.
    pub fn from_longitude(longitude: EclipticLongitude) -> Self {
        let band = (longitude.as_degrees() / 30.0).floor() as usize % 12;
        match band {
            0 => Self::Aries,
            1 => Self::Taurus,
            2 => Self::Gemini,
            3 => Self::Cancer,
            4 => Self::Leo,
            5 => Self::Virgo,
            6 => Self::Libra,
            7 => Self::Scorpio,
            8 => Self::Sagittarius,
            9 => Self::Capricorn,
            10 => Self::Aquarius,
            _ => Self::Pisces,
        }
    }
}

impl fmt::Display for ZodiacSign {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.symbol())
    }
}

/// The sun's apparent ecliptic longitude in degrees, in
/// `[0.0, 360.0)`. Construction wraps into range.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, Copy, PartialEq)]
pub struct EclipticLongitude(f64);

impl EclipticLongitude {
    /// Construct from any finite degrees value; wraps into
    /// `[0.0, 360.0)`.
    pub fn new(degrees: f64) -> Self {
        Self(degrees.rem_euclid(360.0))
    }

    /// The longitude in degrees, in `[0.0, 360.0)`.
    pub fn as_degrees(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for EclipticLongitude {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.4}°", self.0)
    }
}

/// A point on the zodiac: `sign` + `degree` (0..30) + `minute`
/// (0..60). Carried forward from the prototype's zodiacal-time
/// output formats.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq)]
pub struct ZodiacalTime {
    pub sign: ZodiacSign,
    pub degree: u8,
    pub minute: u8,
}

impl ZodiacalTime {
    /// Project an ecliptic longitude onto the zodiac.
    pub fn from_longitude(longitude: EclipticLongitude) -> Self {
        let total = longitude.as_degrees();
        let sign = ZodiacSign::from_longitude(longitude);
        let into_sign = total.rem_euclid(30.0);
        let degree = into_sign.floor() as u8;
        let minute = ((into_sign - degree as f64) * 60.0).floor() as u8;
        Self { sign, degree, minute }
    }
}

impl fmt::Display for ZodiacalTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {:02}°{:02}'", self.sign.symbol(), self.degree, self.minute)
    }
}
