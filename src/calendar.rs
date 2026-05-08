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

use core::fmt;

use nota_codec::NotaTransparent;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

/// A vernal-equinox-anchored year number, signed.
///
/// AM-year 0 is the project-defined epoch; positive years
/// follow chronologically.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
/// Resolution is the underlying f64; the prototype's five
/// output formats are all `Display` projections of this one
/// value.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, Copy, PartialEq)]
pub struct OrdinalSolarTime(f64);

impl OrdinalSolarTime {
    /// Construct from a fractional value; wraps into `[0.0, 1.0)`.
    pub fn new(fraction: f64) -> Self {
        Self(fraction.rem_euclid(1.0))
    }

    /// Construct from a year fraction in degrees `[0.0, 360.0)`.
    pub fn from_degrees(degrees: f64) -> Self {
        Self::new(degrees / 360.0)
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
