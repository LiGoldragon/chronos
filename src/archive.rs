//! The error rkyv reads archives with here: a structural failure, or the typed refusal
//! of a value whose constructor, `try_new`, refused it.

use core::any::Any;

use rkyv::rancor::{Source, Trace};
use thiserror::Error as ThisError;

use crate::error::Error;

/// Why an archive was not admitted.
#[derive(Debug, ThisError)]
pub enum ArchiveFault {
    /// The archive's bytes are not a valid archive of the type.
    #[error("archive structure: {0}")]
    Structure(String),
    /// The archive holds a value its constructor refuses.
    #[error(transparent)]
    Refused(Error),
}

impl Trace for ArchiveFault {
    fn trace<R>(self, trace: R) -> Self
    where
        R: core::fmt::Debug + core::fmt::Display + Send + Sync + 'static,
    {
        match self {
            Self::Structure(text) => Self::Structure(format!("{text}: {trace}")),
            refused => refused,
        }
    }
}

impl Source for ArchiveFault {
    fn new<T: core::error::Error + Send + Sync + 'static>(source: T) -> Self {
        let text = source.to_string();
        let boxed: Box<dyn Any> = Box::new(source);
        match boxed.downcast::<Error>() {
            Ok(refusal) => Self::Refused(*refusal),
            Err(_) => Self::Structure(text),
        }
    }
}

impl From<ArchiveFault> for Error {
    fn from(fault: ArchiveFault) -> Self {
        match fault {
            ArchiveFault::Refused(refusal) => refusal,
            ArchiveFault::Structure(text) => Error::RkyvCodec(text),
        }
    }
}
