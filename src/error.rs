//! The crate's typed boundary failures.

use thiserror::Error as ThisError;

/// Every fallible Chronos boundary returns this error.
#[derive(Debug, ThisError)]
pub enum Error {
    /// A Datomic value does not embody as the expected Chronos type.
    #[error("Datomic {type_name} value is invalid: {problem}")]
    Datomic { type_name: &'static str, problem: &'static str },
    /// Failed to encode or decode an rkyv archive on the wire.
    #[error("rkyv codec failed: {0}")]
    RkyvCodec(String),
    /// I/O error from the OS (UDS, file, process).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// DBus error from a `geoclue2` method call.
    #[error("dbus: {0}")]
    Dbus(#[from] zbus::Error),
    #[error("ephemeris: {0}")]
    Ephemeris(String),
    #[error("solar position: {0}")]
    SolarPosition(String),
    /// A numeric domain value is outside its representable range.
    #[error("`{type_name}` out of range {valid_range}, got {got}")]
    OutOfRange { type_name: &'static str, valid_range: &'static str, got: String },
    #[error("daemon: {message}")]
    Daemon { message: String },
}

impl Error {
    pub fn from_fault(type_name: &'static str, fault: datom_codec::Fault) -> Self {
        let problem = match fault {
            datom_codec::Fault::Structural(_) => "structural",
            datom_codec::Fault::Conceptual(_, _) => "conceptual",
            datom_codec::Fault::Corporate(_, _) => "corporate",
        };
        Self::Datomic { type_name, problem }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
