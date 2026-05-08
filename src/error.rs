//! [`Error`] — the crate's typed error enum.
//!
//! Every fallible boundary in the crate returns
//! `Result<T, Error>`. No `anyhow::Error` / `eyre::Report` /
//! `Box<dyn Error>` at any boundary; per
//! `~/primary/skills/rust-discipline.md` §"Errors: typed enum
//! per crate via thiserror".

use thiserror::Error as ThisError;

/// The crate's error type.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Failed to parse a NOTA document at the CLI boundary.
    #[error("nota parse failed: {0}")]
    NotaParse(#[from] nota_codec::Error),

    /// Failed to encode or decode an rkyv archive on the wire.
    #[error("rkyv codec failed: {0}")]
    RkyvCodec(String),

    /// I/O error from the OS (UDS, file, process).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// DBus error from a `geoclue2` method call.
    #[error("dbus: {0}")]
    Dbus(#[from] zbus::Error),

    /// The DE440 ephemeris failed to load or query.
    #[error("ephemeris: {0}")]
    Ephemeris(String),

    /// NREL SPA solar-position computation failed.
    #[error("solar position: {0}")]
    SolarPosition(String),

    /// A latitude or longitude was outside the valid range.
    #[error("location out of range: {latitude}°, {longitude}°")]
    LocationOutOfRange { latitude: f64, longitude: f64 },

    /// The daemon refused a request.
    #[error("daemon: {message}")]
    Daemon { message: String },
}

/// Crate-local result alias.
pub type Result<T> = core::result::Result<T, Error>;
