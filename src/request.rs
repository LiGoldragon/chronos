//! [`Request`] — what the CLI / chroma sends to the daemon.
//!
//! Parses from a single NOTA record on argv (the CLI's one
//! positional arg). Travels on the wire as a length-prefixed
//! rkyv archive over the daemon's UDS.

use nota_next::{Block, Delimiter, NotaBlock, NotaDecode, NotaDecodeError, NotaEncode, NotaSource};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::error::{Error, Result};
use crate::event::SolarEventKind;
use crate::location::{Latitude, Longitude};

/// What the CLI / a subscriber sends to the daemon.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq)]
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
        Ok(NotaSource::new(text).parse::<Self>()?)
    }

    /// Render this request as a NOTA record.
    pub fn to_nota(&self) -> Result<String> {
        Ok(NotaEncode::to_nota(self))
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

    fn expect_payload_count(
        tag: &'static str,
        payload: &[Block],
        expected: usize,
    ) -> core::result::Result<(), NotaDecodeError> {
        if payload.len() == expected {
            Ok(())
        } else {
            Err(NotaDecodeError::ExpectedRootCount { type_name: tag, expected, found: payload.len() })
        }
    }
}

impl NotaDecode for Request {
    fn from_nota_block(block: &Block) -> core::result::Result<Self, NotaDecodeError> {
        if let Some(tag) = block.demote_to_string() {
            return match tag {
                "GetTime" => Ok(Self::GetTime),
                "GetSchedule" => Ok(Self::GetSchedule),
                "GetLocation" => Ok(Self::GetLocation),
                "UseGeoclue" => Ok(Self::UseGeoclue),
                other => Err(NotaDecodeError::UnknownVariant { enum_name: "Request", variant: other.to_owned() }),
            };
        }

        let children = NotaBlock::new(block).expect_delimited(Delimiter::Parenthesis, "Request")?;
        let (tag, payload) = children.split_first().ok_or(NotaDecodeError::ExpectedRootCount {
            type_name: "Request",
            expected: 1,
            found: 0,
        })?;
        let tag = tag.demote_to_string().ok_or(NotaDecodeError::ExpectedAtom { type_name: "Request variant" })?;
        match tag {
            "SetLocation" => {
                Self::expect_payload_count("SetLocation", payload, 2)?;
                Ok(Self::SetLocation {
                    latitude: Latitude::from_nota_block(&payload[0])?,
                    longitude: Longitude::from_nota_block(&payload[1])?,
                })
            }
            "Subscribe" => {
                Self::expect_payload_count("Subscribe", payload, 1)?;
                Ok(Self::Subscribe { kinds: Vec::<SolarEventKind>::from_nota_block(&payload[0])? })
            }
            other => Err(NotaDecodeError::UnknownVariant { enum_name: "Request", variant: other.to_owned() }),
        }
    }
}

impl NotaEncode for Request {
    fn to_nota(&self) -> String {
        match self {
            Self::GetTime => "GetTime".to_owned(),
            Self::GetSchedule => "GetSchedule".to_owned(),
            Self::GetLocation => "GetLocation".to_owned(),
            Self::SetLocation { latitude, longitude } => {
                Delimiter::Parenthesis.wrap(["SetLocation".to_owned(), latitude.to_nota(), longitude.to_nota()])
            }
            Self::UseGeoclue => "UseGeoclue".to_owned(),
            Self::Subscribe { kinds } => Delimiter::Parenthesis.wrap(["Subscribe".to_owned(), kinds.to_nota()]),
        }
    }
}
