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

use datom_codec::{Datom, Datomic, Headed, Positional, Sited};
use hifitime::Epoch;
use protos::{Conceivable, Extent, Situated, Situation};
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
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Datomic for SolarEventKind {
    fn incorporate(site: datom_codec::Site<'_>) -> core::result::Result<Self, datom_codec::Fault> {
        let variant = site.variant()?;
        let value = match variant.name { "CivilDawn" => Self::CivilDawn, "Sunrise" => Self::Sunrise, "SolarNoon" => Self::SolarNoon, "Sunset" => Self::Sunset, "CivilDusk" => Self::CivilDusk, name => return Err(variant.reject(datom_codec::Problem::UnknownVariant(protos::Word::try_from(name).expect("variant word")))) };
        Headed::nothing(variant)?;
        Ok(value)
    }
}

impl Conceivable<Datom> for SolarEventKind {
    type Fault = core::convert::Infallible;
    fn conceive(&self) -> core::result::Result<Situated<Datom>, Self::Fault> {
        let word = match self { Self::CivilDawn => "CivilDawn", Self::Sunrise => "Sunrise", Self::SolarNoon => "SolarNoon", Self::Sunset => "Sunset", Self::CivilDusk => "CivilDusk" };
        Ok(Situated(Situation { extent: Extent(0, 0), children: vec![] }, Datom::Word(datom_codec::DatomWord::try_from(word).expect("static word"))))
    }
}

/// An instant in TAI nanoseconds since the J2000 epoch, wire-
/// stable and zero-copy in rkyv. Constructed from a `hifitime`
/// [`Epoch`] for human-side use.
///
/// Any signed `i64` is a valid offset, so this is
/// transparent Datomic integer encoding — no validation gap.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Datomic for EpochTaiNanos { fn incorporate(site: datom_codec::Site<'_>) -> core::result::Result<Self, datom_codec::Fault> { Ok(Self(i64::from(protos::Integer::incorporate(site)?))) } }
impl Conceivable<Datom> for EpochTaiNanos { type Fault = core::convert::Infallible; fn conceive(&self) -> core::result::Result<Situated<Datom>, Self::Fault> { protos::Integer::from(self.0).conceive() } }

/// A pushed event — what fires, when, and where the observer
/// was when it was scheduled.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct SolarEvent {
    pub kind: SolarEventKind,
    pub when: EpochTaiNanos,
    pub location: Location,
}

impl Datomic for SolarEvent { fn incorporate(site: datom_codec::Site<'_>) -> core::result::Result<Self, datom_codec::Fault> { let mut p = site.positions(3)?; Ok(Self { kind: p.position()?, when: p.position()?, location: p.position()? }) } }
impl Conceivable<Datom> for SolarEvent { type Fault = core::convert::Infallible; fn conceive(&self) -> core::result::Result<Situated<Datom>, Self::Fault> { Ok(Situated(Situation { extent: Extent(0, 0), children: vec![] }, Datom::Struct(vec![self.kind.conceive()?.1, self.when.conceive()?.1, self.location.conceive()?.1]))) } }
