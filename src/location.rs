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
//! [`Longitude::try_new`] via the `NotaTryTransparent` derive
//! (same shape as horizon-rs's `SshPubKey`/`NixPubKey`/
//! `WireguardPubKey`/`CriomeDomainName`). An out-of-range
//! latitude or longitude on the wire surfaces as
//! `nota_codec::Error::Validation { type_name, message }`,
//! never reaches the daemon.

use core::fmt;

use nota_codec::{NotaEnum, NotaRecord, NotaTryTransparent};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::error::{Error, Result};

/// Latitude in degrees, in `[-90.0, 90.0]`.
///
/// Construction is fallible — [`Latitude::try_new`] rejects
/// out-of-range and non-finite inputs. The wire form is the
/// bare degrees value; the `NotaTryTransparent` derive routes
/// decode through `try_new`, so an invalid wire latitude is
/// rejected at the parser boundary.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTryTransparent, Debug, Clone, Copy, PartialEq)]
pub struct Latitude(f64);

impl Latitude {
    /// Construct from a degrees value in `[-90.0, 90.0]`.
    pub fn try_new(degrees: f64) -> Result<Self> {
        if degrees.is_finite() && (-90.0..=90.0).contains(&degrees) {
            Ok(Self(degrees))
        } else {
            Err(Error::LatitudeOutOfRange { got: degrees })
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

/// Longitude in degrees, in `[-180.0, 180.0]`. East is positive.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTryTransparent, Debug, Clone, Copy, PartialEq)]
pub struct Longitude(f64);

impl Longitude {
    /// Construct from a degrees value in `[-180.0, 180.0]`.
    pub fn try_new(degrees: f64) -> Result<Self> {
        if degrees.is_finite() && (-180.0..=180.0).contains(&degrees) {
            Ok(Self(degrees))
        } else {
            Err(Error::LongitudeOutOfRange { got: degrees })
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

/// A geographic location — latitude + longitude.
///
/// `Location` derives `NotaRecord`, whose per-field decode
/// delegates to each field's `NotaDecode`. With `Latitude`
/// and `Longitude` validated on decode, `Location` inherits
/// the validation: a `(Location 200 -400)` frame is rejected
/// at the latitude field before any `Location` is constructed.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq)]
pub struct Location {
    pub latitude: Latitude,
    pub longitude: Longitude,
}

impl Location {
    /// Construct from raw degree values, validating both
    /// components.
    pub fn try_from_degrees(latitude: f64, longitude: f64) -> Result<Self> {
        Ok(Self {
            latitude: Latitude::try_new(latitude)?,
            longitude: Longitude::try_new(longitude)?,
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
