//! Solar calendar — [`AmYear`], [`OrdinalSolarTime`].
//!
//! The "AM" calendar is anchored to the vernal equinox: year
//! zero is the vernal-equinox crossing closest to a project-
//! defined epoch, and each year begins at the next vernal
//! equinox. Carried forward from the prototype's `am`,
//! `version`, `numeric`, `unicode`, and `json` output formats.
//!
//! The five output formats of the prototype project all
//! flow from the same underlying [`OrdinalSolarTime`].
//!
//! Like [`crate::zodiac::EclipticLongitude`], `OrdinalSolarTime`
//! is strict on construction; astronomical code that produces
//! raw fractions outside `[0, 1)` normalises explicitly via
//! [`OrdinalSolarTime::from_unnormalized_fraction`] before
//! constructing.

use core::fmt;

use dotos::{Block, DotosDecode, DotosDecodeError, DotosEncode};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::error::{Error, Result};

/// A vernal-equinox-anchored year number, signed.
///
/// AM-year 0 is the project-defined epoch; positive years
/// follow chronologically. Any signed integer is a valid
/// year, so [`AmYear`] is transparently encoded — no validation
/// gap exists.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, DotosDecode, DotosEncode, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub struct AmYear(i32);

impl AmYear {
    /// Construct from a raw signed integer.
    pub fn new(year: i32) -> Self {
        Self(year)
    }

    /// The year as a signed integer.
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for AmYear {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// The sun's ecliptic longitude expressed as a fractional
/// position in the year, in `[0.0, 1.0)` where `0.0` is
/// the vernal equinox and `0.5` the autumnal equinox.
///
/// Construction is strict — [`OrdinalSolarTime::try_new`]
/// rejects out-of-range and non-finite inputs.
/// [`OrdinalSolarTime::from_unnormalized_fraction`] wraps any
/// finite input into `[0, 1)` for astronomical-pipeline use.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct OrdinalSolarTime(f64);

impl OrdinalSolarTime {
    /// Construct from a finite fraction in `[0.0, 1.0)`.
    pub fn try_new(fraction: f64) -> Result<Self> {
        if fraction.is_finite() && (0.0..1.0).contains(&fraction) {
            Ok(Self(fraction))
        } else {
            Err(Error::OutOfRange {
                type_name: "OrdinalSolarTime",
                valid_range: "[0, 1)",
                got: format!("{fraction:?}"),
            })
        }
    }

    /// Construct by wrapping any finite fraction into `[0.0, 1.0)`.
    pub fn from_unnormalized_fraction(fraction: f64) -> Self {
        Self(fraction.rem_euclid(1.0))
    }

    /// Construct from a year fraction in degrees.
    pub fn from_unnormalized_degrees(degrees: f64) -> Self {
        Self::from_unnormalized_fraction(degrees / 360.0)
    }

    /// The fractional position in `[0.0, 1.0)`.
    pub fn as_fraction(&self) -> f64 {
        self.0
    }

    /// The position expressed in degrees `[0.0, 360.0)`.
    pub fn as_degrees(&self) -> f64 {
        self.0 * 360.0
    }
}

impl fmt::Display for OrdinalSolarTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.6}", self.0)
    }
}

impl DotosDecode for OrdinalSolarTime {
    fn from_dotos_block(block: &Block) -> core::result::Result<Self, DotosDecodeError> {
        let fraction = f64::from_dotos_block(block)?;
        Self::try_new(fraction).map_err(|error| error.into_dotos_invalid_value(fraction.to_string()))
    }
}

impl DotosEncode for OrdinalSolarTime {
    fn to_dotos(&self) -> String {
        self.0.to_dotos()
    }
}
