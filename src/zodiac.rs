//! Zodiacal projection — [`ZodiacSign`], [`EclipticLongitude`],
//! [`ZodiacDegree`], [`ZodiacMinute`], [`ZodiacalTime`].
//!
//! The sun's apparent ecliptic longitude is what the prototype
//! called "ordinal solar time"; chronos projects it onto the
//! twelve-sign zodiac for human-facing display.
//!
//! Every typed boundary is validated on decode. `EclipticLongitude`
//! is strict to `[0, 360)` — astronomical code that produces
//! unbounded degree values normalises explicitly via
//! [`EclipticLongitude::from_unnormalized_degrees`] before
//! constructing.

use core::fmt;

use nota_next::{Block, NotaDecode, NotaDecodeError, NotaEncode};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::error::{Error, Result};

/// One of the twelve zodiac signs, in source-declaration
/// order matching the standard zodiacal sequence (Aries first,
/// Pisces last).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaDecode, NotaEncode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
/// `[0.0, 360.0)`.
///
/// Construction is strict — [`EclipticLongitude::try_new`]
/// rejects out-of-range and non-finite inputs. Astronomical
/// code that computes raw degree values (which can land
/// outside `[0, 360)`) normalises through
/// [`EclipticLongitude::from_unnormalized_degrees`] before
/// constructing, so the wire form is always normalised.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct EclipticLongitude(f64);

impl EclipticLongitude {
    /// Construct from a finite degrees value in `[0.0, 360.0)`.
    pub fn try_new(degrees: f64) -> Result<Self> {
        if degrees.is_finite() && (0.0..360.0).contains(&degrees) {
            Ok(Self(degrees))
        } else {
            Err(Error::OutOfRange {
                type_name: "EclipticLongitude",
                valid_range: "[0, 360)",
                got: format!("{degrees:?}"),
            })
        }
    }

    /// Construct by wrapping any finite degrees value into
    /// `[0.0, 360.0)`. For astronomical computations whose
    /// output is unbounded; the wire-side strict constructor is
    /// [`Self::try_new`].
    pub fn from_unnormalized_degrees(degrees: f64) -> Self {
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

impl NotaDecode for EclipticLongitude {
    fn from_nota_block(block: &Block) -> core::result::Result<Self, NotaDecodeError> {
        let degrees = f64::from_nota_block(block)?;
        Self::try_new(degrees).map_err(|error| error.into_nota_invalid_value(degrees.to_string()))
    }
}

impl NotaEncode for EclipticLongitude {
    fn to_nota(&self) -> String {
        self.0.to_nota()
    }
}

/// Degrees within a zodiacal sign, in `[0, 30)`.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZodiacDegree(u8);

impl ZodiacDegree {
    /// Construct from a value in `[0, 30)`.
    pub fn try_new(degree: u8) -> Result<Self> {
        if degree < 30 {
            Ok(Self(degree))
        } else {
            Err(Error::OutOfRange { type_name: "ZodiacDegree", valid_range: "[0, 30)", got: format!("{degree}") })
        }
    }

    /// The degree as a `u8` in `[0, 30)`.
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl NotaDecode for ZodiacDegree {
    fn from_nota_block(block: &Block) -> core::result::Result<Self, NotaDecodeError> {
        let degree = u8::from_nota_block(block)?;
        Self::try_new(degree).map_err(|error| error.into_nota_invalid_value(degree.to_string()))
    }
}

impl NotaEncode for ZodiacDegree {
    fn to_nota(&self) -> String {
        self.0.to_nota()
    }
}

/// Minutes within a zodiacal degree, in `[0, 60)`.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZodiacMinute(u8);

impl ZodiacMinute {
    /// Construct from a value in `[0, 60)`.
    pub fn try_new(minute: u8) -> Result<Self> {
        if minute < 60 {
            Ok(Self(minute))
        } else {
            Err(Error::OutOfRange { type_name: "ZodiacMinute", valid_range: "[0, 60)", got: format!("{minute}") })
        }
    }

    /// The minute as a `u8` in `[0, 60)`.
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl NotaDecode for ZodiacMinute {
    fn from_nota_block(block: &Block) -> core::result::Result<Self, NotaDecodeError> {
        let minute = u8::from_nota_block(block)?;
        Self::try_new(minute).map_err(|error| error.into_nota_invalid_value(minute.to_string()))
    }
}

impl NotaEncode for ZodiacMinute {
    fn to_nota(&self) -> String {
        self.0.to_nota()
    }
}

/// A point on the zodiac: `sign` + `degree` (0..30) + `minute`
/// (0..60). Carried forward from the prototype's zodiacal-time
/// output formats.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaDecode, NotaEncode, Debug, Clone, Copy, PartialEq)]
pub struct ZodiacalTime {
    pub sign: ZodiacSign,
    pub degree: ZodiacDegree,
    pub minute: ZodiacMinute,
}

impl ZodiacalTime {
    /// Project an ecliptic longitude onto the zodiac.
    pub fn from_longitude(longitude: EclipticLongitude) -> Self {
        let sign = ZodiacSign::from_longitude(longitude);
        let into_sign = longitude.as_degrees().rem_euclid(30.0);
        let degree_raw = (into_sign.floor() as u8).min(29);
        let minute_raw = (((into_sign - degree_raw as f64) * 60.0).floor() as u8).min(59);
        Self {
            sign,
            degree: ZodiacDegree::try_new(degree_raw).expect("ZodiacDegree::try_new on a value clamped to [0, 30)"),
            minute: ZodiacMinute::try_new(minute_raw).expect("ZodiacMinute::try_new on a value clamped to [0, 60)"),
        }
    }
}

impl fmt::Display for ZodiacalTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {:02}°{:02}'", self.sign.symbol(), self.degree.as_u8(), self.minute.as_u8())
    }
}
