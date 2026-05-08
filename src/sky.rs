//! [`Sky`], [`Observer`], [`SkyState`] — the astronomy boundary.
//!
//! `Sky` owns the loaded JPL DE440 ephemeris (via `anise`).
//! `Observer` binds a `Sky` to a [`Location`] for queries that
//! need both. `SkyState` is the answer to "where is the sun
//! right now, from where I'm standing?"
//!
//! Astronomical bodies are stubbed (`todo!()`) in the skeleton;
//! the type signatures are the design. See the design report
//! `~/primary/reports/system-specialist/49-chronos-daemon-design.md`
//! for the pipeline (DE440 → ecliptic longitude → zodiac;
//! NREL SPA → twilight crossings).

use hifitime::Epoch;

use crate::error::Result;
use crate::event::{SolarEvent, SolarEventKind};
use crate::location::Location;
use crate::zodiac::{EclipticLongitude, ZodiacalTime};

/// The loaded ephemeris — currently a `()` stub; the
/// implementation phase wires `anise::almanac::Almanac`
/// here.
pub struct Sky {
    // TODO(chronos-impl): hold an `anise::almanac::Almanac`.
    _ephemeris: (),
}

impl Sky {
    /// Load a [`Sky`] from a DE440 SPK file path.
    pub fn from_path(_path: &std::path::Path) -> Result<Self> {
        todo!("load DE440 via anise::almanac::Almanac::new")
    }

    /// The sun's apparent ecliptic longitude at `when`.
    pub fn ecliptic_longitude_at(&self, _when: Epoch) -> Result<EclipticLongitude> {
        todo!("query DE440 for the sun's geocentric apparent longitude")
    }

    /// The zodiacal time at `when` — i.e. `ZodiacalTime` for
    /// the sun's apparent ecliptic longitude.
    pub fn zodiacal_time_at(&self, when: Epoch) -> Result<ZodiacalTime> {
        let longitude = self.ecliptic_longitude_at(when)?;
        Ok(ZodiacalTime::from_longitude(longitude))
    }
}

/// A bound observer — `Sky` plus `Location`. Owns the queries
/// that need both ephemeris and place: rise / set / twilight
/// times.
pub struct Observer<'sky> {
    sky: &'sky Sky,
    location: Location,
}

impl<'sky> Observer<'sky> {
    /// Construct from a [`Sky`] reference and a [`Location`].
    pub fn new(sky: &'sky Sky, location: Location) -> Self {
        Self { sky, location }
    }

    /// The observer's location.
    pub fn location(&self) -> Location {
        self.location
    }

    /// The full astronomical state of the sky at this
    /// observer's place at `when`.
    pub fn state_at(&self, _when: Epoch) -> Result<SkyState> {
        todo!("compute zodiacal time + sun-above-horizon + altitude")
    }

    /// The next [`SolarEvent`] of `kind` strictly after
    /// `after`. Computed via NREL SPA in `solar-positioning`.
    pub fn next_event(&self, _kind: SolarEventKind, _after: Epoch) -> Result<SolarEvent> {
        todo!("call solar_positioning twilight-at-angle / sunrise-sunset")
    }

    /// All five named events for the local civil day
    /// containing `now`, in chronological order.
    pub fn day_schedule(&self, _now: Epoch) -> Result<Vec<SolarEvent>> {
        todo!("CivilDawn → Sunrise → SolarNoon → Sunset → CivilDusk")
    }

    /// The [`Sky`] this observer is bound to. Useful when a
    /// caller wants to issue a sky-only query (one that doesn't
    /// need the location) without re-loading the ephemeris.
    pub fn sky(&self) -> &Sky {
        self.sky
    }
}

/// The astronomical state of the local sky at one moment.
pub struct SkyState {
    /// The sun's zodiacal time.
    pub zodiacal_time: ZodiacalTime,
    /// True if the sun's centre is above the local horizon.
    pub sun_above_horizon: bool,
    /// Solar altitude above the local horizon, in degrees.
    pub solar_altitude_degrees: f64,
}
