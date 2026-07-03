//! Solar events — [`SolarEvent`], [`SolarEventKind`],
//! [`EpochTaiNanos`].
//!
//! These are the pushed records subscribers receive on each
//! daily fire. Per `~/primary/skills/push-not-pull.md`
//! §"Subscription contract", a subscriber receives the
//! current state on connect (the upcoming events for today)
//! and then deltas as deadlines fire.
//!
//! Chroma's schedule engine subscribes to a chosen subset
//! of these kinds and reacts (e.g. start a 60-minute warmth
//! ramp at `CivilDusk`).

use hifitime::Epoch;
use nota::{NotaDecode, NotaEncode};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::location::Location;

/// One of the named solar events chronos publishes.
///
/// The set is deliberately small: civil-twilight bookends, the
/// rise/set pair, and solar noon. Nautical and astronomical
/// twilights are an extension; the set may grow but the variant
/// names already in use must not change without a coordinated
/// schema upgrade (per `~/primary/skills/rust-discipline.md`
/// §"Schema discipline").
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaDecode, NotaEncode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SolarEventKind {
    /// Sun's centre 6° below horizon, rising.
    CivilDawn,
    /// Sun's upper limb crosses horizon, rising.
    Sunrise,
    /// Sun crosses local meridian.
    SolarNoon,
    /// Sun's upper limb crosses horizon, setting.
    Sunset,
    /// Sun's centre 6° below horizon, setting.
    CivilDusk,
}

/// An instant in TAI nanoseconds since the J2000 epoch, wire-
/// stable and zero-copy in rkyv. Constructed from a `hifitime`
/// [`Epoch`] for human-side use.
///
/// Any signed `i64` is a valid offset, so this is
/// transparent NOTA encoding — no validation gap.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaDecode, NotaEncode, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EpochTaiNanos(i64);

impl EpochTaiNanos {
    /// Construct from a [`hifitime::Epoch`].
    pub fn from_epoch(epoch: Epoch) -> Self {
        Self(epoch.to_tai_duration().total_nanoseconds() as i64)
    }

    /// Reconstruct a [`hifitime::Epoch`].
    pub fn to_epoch(&self) -> Epoch {
        Epoch::from_tai_duration(hifitime::Duration::from_total_nanoseconds(i128::from(self.0)))
    }

    /// The raw TAI nanoseconds since J2000.
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

/// A pushed event — what fires, when, and where the observer
/// was when it was scheduled.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaDecode, NotaEncode, Debug, Clone, Copy, PartialEq)]
pub struct SolarEvent {
    pub kind: SolarEventKind,
    pub when: EpochTaiNanos,
    pub location: Location,
}
