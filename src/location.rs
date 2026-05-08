//! Observer location — [`Latitude`], [`Longitude`], [`Location`],
//! [`LocationSource`].
//!
//! Latitudes and longitudes carry typed identity beyond their
//! bits (per `~/primary/skills/rust-discipline.md` §"Domain
//! values are types, not primitives"). Construction validates
//! the range; raw access goes through [`Latitude::as_degrees`]
//! and [`Longitude::as_degrees`].

use core::fmt;

use nota_codec::{NotaEnum, NotaRecord, NotaTransparent};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::error::{Error, Result};

/// Latitude in degrees, in `[-90.0, 90.0]`.
///
/// The wire form is the bare degrees value; validation runs
/// on construction through [`Latitude::new`]. Wire decoders
/// that bypass `new` and feed an out-of-range float produce
/// a record that violates the type's invariant — see the
/// schema-discipline note in `skills.md`.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, Copy, PartialEq)]
pub struct Latitude(f64);

impl Latitude {
    /// Construct from a degrees value in `[-90.0, 90.0]`.
    pub fn new(degrees: f64) -> Result<Self> {
        if !degrees.is_finite() || !(-90.0..=90.0).contains(&degrees) {
            return Err(Error::LocationOutOfRange { latitude: degrees, longitude: 0.0 });
        }
        Ok(Self(degrees))
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

/// Longitude in degrees, in `[-180.0, 180.0]`. East is positive.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, Copy, PartialEq)]
pub struct Longitude(f64);

impl Longitude {
    /// Construct from a degrees value in `[-180.0, 180.0]`.
    pub fn new(degrees: f64) -> Result<Self> {
        if !degrees.is_finite() || !(-180.0..=180.0).contains(&degrees) {
            return Err(Error::LocationOutOfRange { latitude: 0.0, longitude: degrees });
        }
        Ok(Self(degrees))
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

/// A geographic location — latitude + longitude.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq)]
pub struct Location {
    pub latitude: Latitude,
    pub longitude: Longitude,
}

impl Location {
    /// Construct from raw degree values, validating both
    /// components.
    pub fn from_degrees(latitude: f64, longitude: f64) -> Result<Self> {
        Ok(Self {
            latitude: Latitude::new(latitude)?,
            longitude: Longitude::new(longitude)?,
        })
    }
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
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq)]
pub enum LocationSource {
    /// Subscribe to `geoclue2`.
    Geoclue,
    /// Use the persisted manual override.
    Manual,
}
