//! Observer location — [`Latitude`], [`Longitude`], [`Location`],
//! [`LocationSource`].
//!
//! Latitudes and longitudes carry typed identity beyond their
//! bits (per `~/primary/skills/rust-discipline.md` §"Domain
//! values are types, not primitives"). Construction validates
//! the range; raw access goes through [`Latitude::as_degrees`]
//! and [`Longitude::as_degrees`].
//!
//! Wire decoding routes through [`Latitude::try_new`] /
//! [`Longitude::try_new`]. An out-of-range latitude or
//! longitude on the wire surfaces as `DotosDecodeError::InvalidValue`
//! and never reaches the daemon.

use core::fmt;

use dotos::{Block, DotosDecode, DotosDecodeError, DotosEncode};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::error::{Error, Result};

/// Latitude in degrees, in `[-90.0, 90.0]`.
///
/// Construction is fallible — [`Latitude::try_new`] rejects
/// out-of-range and non-finite inputs. The wire form is the
/// bare degrees value; decode routes through `try_new`, so an
/// invalid wire latitude is rejected at the parser boundary.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct Latitude(f64);

impl Latitude {
    /// Construct from a degrees value in `[-90.0, 90.0]`.
    pub fn try_new(degrees: f64) -> Result<Self> {
        if degrees.is_finite() && (-90.0..=90.0).contains(&degrees) {
            Ok(Self(degrees))
        } else {
            Err(Error::OutOfRange { type_name: "Latitude", valid_range: "[-90, 90]", got: format!("{degrees:?}") })
        }
    }

    /// The latitude in degrees.
    pub fn as_degrees(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Latitude {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.4}°", self.0)
    }
}

impl From<Latitude> for f64 {
    fn from(value: Latitude) -> Self {
        value.0
    }
}

impl DotosDecode for Latitude {
    fn from_dotos_block(block: &Block) -> core::result::Result<Self, DotosDecodeError> {
        let degrees = f64::from_dotos_block(block)?;
        Self::try_new(degrees).map_err(|error| error.into_dotos_invalid_value(degrees.to_string()))
    }
}

impl DotosEncode for Latitude {
    fn to_dotos(&self) -> String {
        self.0.to_dotos()
    }
}

/// Longitude in degrees, in `[-180.0, 180.0]`. East is positive.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct Longitude(f64);

impl Longitude {
    /// Construct from a degrees value in `[-180.0, 180.0]`.
    pub fn try_new(degrees: f64) -> Result<Self> {
        if degrees.is_finite() && (-180.0..=180.0).contains(&degrees) {
            Ok(Self(degrees))
        } else {
            Err(Error::OutOfRange { type_name: "Longitude", valid_range: "[-180, 180]", got: format!("{degrees:?}") })
        }
    }

    /// The longitude in degrees.
    pub fn as_degrees(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Longitude {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.4}°", self.0)
    }
}

impl From<Longitude> for f64 {
    fn from(value: Longitude) -> Self {
        value.0
    }
}

impl DotosDecode for Longitude {
    fn from_dotos_block(block: &Block) -> core::result::Result<Self, DotosDecodeError> {
        let degrees = f64::from_dotos_block(block)?;
        Self::try_new(degrees).map_err(|error| error.into_dotos_invalid_value(degrees.to_string()))
    }
}

impl DotosEncode for Longitude {
    fn to_dotos(&self) -> String {
        self.0.to_dotos()
    }
}

/// A geographic location — latitude + longitude.
///
/// `Location` derives `DotosDecode`, whose per-field decode
/// delegates to each field's `DotosDecode`. With `Latitude`
/// and `Longitude` validated on decode, `Location` inherits
/// the validation: a `(200 -400)` frame is rejected
/// at the latitude field before any `Location` is constructed.
///
/// Programmatic construction goes through field-init with
/// pre-validated newtypes:
///
/// ```ignore
/// let location = Location {
///     latitude: Latitude::try_new(47.6)?,
///     longitude: Longitude::try_new(-122.3)?,
/// };
/// ```
///
/// There is no `Location::try_from_degrees(f64, f64)`
/// constructor — that would take two explicit objects at the
/// boundary, against `~/primary/skills/rust-discipline.md`
/// §"One object in, one object out".
#[derive(Archive, RkyvSerialize, RkyvDeserialize, DotosDecode, DotosEncode, Debug, Clone, Copy, PartialEq)]
pub struct Location {
    pub latitude: Latitude,
    pub longitude: Longitude,
}

impl fmt::Display for Location {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}, {}", self.latitude, self.longitude)
    }
}

/// Where the daemon's authoritative [`Location`] comes from.
///
/// `Geoclue` is the default — the daemon subscribes to
/// `geoclue2` and updates as fixes arrive. `Manual` is set
/// by `chronos '(SetLocation …)'` and persists across
/// restarts in the redb store.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, DotosDecode, DotosEncode, Debug, Clone, Copy, PartialEq)]
pub enum LocationSource {
    /// Subscribe to `geoclue2`.
    Geoclue,
    /// Use the persisted manual override.
    Manual,
}
