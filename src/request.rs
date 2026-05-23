//! [`Request`] — what the CLI / chroma sends to the daemon.
//!
//! Parses from a single NOTA record on argv (the CLI's one
//! positional arg). Travels on the wire as a length-prefixed
//! rkyv archive over the daemon's UDS.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::error::{Error, Result};
use crate::event::SolarEventKind;
use crate::location::{Latitude, Longitude};

/// What the CLI / a subscriber sends to the daemon.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, PartialEq)]
pub enum Request {
    /// Read the current zodiacal time.
    GetTime,

    /// Read today's full solar schedule (CivilDawn → CivilDusk).
    GetSchedule,

    /// Read the daemon's current authoritative location.
    GetLocation,

    /// Override location manually (persists across restarts).
    /// Switches `LocationSource` to `Manual`.
    SetLocation { latitude: Latitude, longitude: Longitude },

    /// Switch back to `Geoclue`-driven location.
    UseGeoclue,

    /// Open a long-lived event stream. The daemon replies
    /// with one frame per event of the requested kinds, in
    /// arrival order. The stream begins with the upcoming
    /// fires of each requested kind for the current civil
    /// day (the "current state" required by
    /// `~/primary/skills/push-not-pull.md` §"Subscription
    /// contract").
    Subscribe { kinds: Vec<SolarEventKind> },
}

impl Request {
    /// Parse a single NOTA record into a typed request.
    pub fn from_nota(text: &str) -> Result<Self> {
        let mut decoder = Decoder::new(text);
        let request = <Self as NotaDecode>::decode(&mut decoder)?;
        Ok(request)
    }

    /// Render this request as a NOTA record.
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
