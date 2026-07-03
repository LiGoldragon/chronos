//! [`Error`] — the crate's typed error enum.
//!
//! Every fallible boundary in the crate returns
//! `Result<T, Error>`. No `anyhow::Error` / `eyre::Report` /
//! `Box<dyn Error>` at any boundary; per
//! `~/primary/skills/rust-discipline.md` §"Errors: typed enum
//! per crate via thiserror".

use thiserror::Error as ThisError;

use nota::NotaDecodeError;

/// The crate's error type.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Failed to parse a NOTA document at the CLI boundary.
    #[error("nota parse failed: {0}")]
    NotaParse(#[from] NotaDecodeError),

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

    /// A typed newtype's `try_new` rejected an input that was
    /// outside its valid range. `type_name` names the newtype
    /// (`Latitude`, `EclipticLongitude`, `ZodiacDegree`, …);
    /// `valid_range` is a human-facing description of the
    /// constraint (`"[-90, 90]"`, `"[0, 360)"`, …); `got` is the
    /// rejected value's `Display` form.
    ///
    /// Wire-side, NOTA decode wraps this into
    /// `NotaDecodeError::InvalidValue` where the message renders
    /// this variant.
    #[error("`{type_name}` out of range {valid_range}, got {got}")]
    OutOfRange { type_name: &'static str, valid_range: &'static str, got: String },

    /// The daemon refused a request.
    #[error("daemon: {message}")]
    Daemon { message: String },
}

impl Error {
    pub fn into_nota_invalid_value(self, value: impl Into<String>) -> NotaDecodeError {
        let value = value.into();
        match self {
            Self::OutOfRange { type_name, valid_range, got } => NotaDecodeError::InvalidValue {
                type_name,
                value,
                reason: format!("out of range {valid_range}, got {got}"),
            },
            other => NotaDecodeError::InvalidValue { type_name: "ChronosValue", value, reason: other.to_string() },
        }
    }
}

/// Crate-local result alias.
pub type Result<T> = core::result::Result<T, Error>;
