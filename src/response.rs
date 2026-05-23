//! [`Response`] — what the daemon replies with.
//!
//! Encodes both one-shot replies (`Time`, `Schedule`, `Location`,
//! `Acked`, `Error`) and subscription frames (`Event`). A
//! subscriber receives one `Event` frame per fire on a long-
//! lived connection.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::error::{Error, Result};
use crate::event::SolarEvent;
use crate::location::{Location, LocationSource};
use crate::zodiac::ZodiacalTime;

/// What the daemon replies with.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq)]
pub enum Response {
    /// The request was accepted and produced no reply payload.
    Acked,

    /// The current zodiacal time (reply to `GetTime`).
    Time { zodiacal_time: ZodiacalTime },

    /// Today's solar schedule (reply to `GetSchedule`).
    Schedule { events: Vec<SolarEvent> },

    /// The daemon's current location (reply to `GetLocation`).
    Location { location: Location, source: LocationSource },

    /// One pushed event on a `Subscribe` stream.
    Event { event: SolarEvent },

    /// The daemon refused or could not handle the request.
    Error { message: String },
}

impl Response {
    /// Parse a single NOTA record into a typed response.
    pub fn from_nota(text: &str) -> Result<Self> {
        let mut decoder = Decoder::new(text);
        let response = <Self as NotaDecode>::decode(&mut decoder)?;
        Ok(response)
    }

    /// Render this response as a NOTA record.
    pub fn to_nota(&self) -> Result<String> {
        let mut encoder = Encoder::new();
        <Self as NotaEncode>::encode(self, &mut encoder)?;
        Ok(encoder.into_string())
    }

    /// Archive into rkyv bytes for the wire.
    pub fn archive(&self) -> Result<Vec<u8>> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|err| Error::RkyvCodec(err.to_string()))
    }

    /// Reconstruct from an rkyv archive coming off the wire.
    pub fn from_archive(bytes: &[u8]) -> Result<Self> {
        rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes).map_err(|err| Error::RkyvCodec(err.to_string()))
    }
}
