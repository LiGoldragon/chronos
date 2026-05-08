//! chronos — a daemon publishing zodiacal time, sunrise/sunset,
//! and twilight events for the local observer.
//!
//! Per the design report at
//! `~/primary/reports/system-specialist/49-chronos-daemon-design.md`:
//! a single user-service daemon that owns the local sky — DE440
//! ephemeris (`anise`) + NREL SPA (`solar-positioning`) +
//! `hifitime` time scales — and publishes typed [`SolarEvent`]s
//! to subscribers at deadline fires.
//!
//! See `ARCHITECTURE.md` for the system shape, `AGENTS.md` for
//! the agent contract, and `skills.md` for project-specific
//! intent and invariants.

pub mod calendar;
pub mod client;
pub mod daemon;
pub mod error;
pub mod event;
pub mod location;
pub mod request;
pub mod response;
pub mod sky;
pub mod wire;
pub mod zodiac;

pub use calendar::{AmYear, OrdinalSolarTime};
pub use error::{Error, Result};
pub use event::{SolarEvent, SolarEventKind};
pub use location::{Latitude, Location, LocationSource, Longitude};
pub use request::Request;
pub use response::Response;
pub use sky::{Observer, Sky, SkyState};
pub use zodiac::{EclipticLongitude, ZodiacSign, ZodiacalTime};
