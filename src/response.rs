//! [`Response`] — what the daemon replies with.
//!
//! Encodes both one-shot replies (`Time`, `Schedule`, `Location`,
//! `Acked`, `Error`) and subscription frames (`Event`). A
//! subscriber receives one `Event` frame per fire on a long-
//! lived connection.

use nota_next::{Block, Delimiter, NotaBlock, NotaDecode, NotaDecodeError, NotaEncode, NotaSource};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::error::{Error, Result};
use crate::event::SolarEvent;
use crate::location::{Location, LocationSource};
use crate::zodiac::ZodiacalTime;

/// What the daemon replies with.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq)]
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
        Ok(NotaSource::new(text).parse::<Self>()?)
    }

    /// Render this response as a NOTA record.
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

impl NotaDecode for Response {
    fn from_nota_block(block: &Block) -> core::result::Result<Self, NotaDecodeError> {
        if let Some(tag) = block.demote_to_string() {
            return match tag {
                "Acked" => Ok(Self::Acked),
                other => Err(NotaDecodeError::UnknownVariant { enum_name: "Response", variant: other.to_owned() }),
            };
        }

        let children = NotaBlock::new(block).expect_delimited(Delimiter::Parenthesis, "Response")?;
        let (tag, payload) = children.split_first().ok_or(NotaDecodeError::ExpectedRootCount {
            type_name: "Response",
            expected: 1,
            found: 0,
        })?;
        let tag = tag.demote_to_string().ok_or(NotaDecodeError::ExpectedAtom { type_name: "Response variant" })?;
        match tag {
            "Time" => {
                Self::expect_payload_count("Time", payload, 1)?;
                Ok(Self::Time { zodiacal_time: ZodiacalTime::from_nota_block(&payload[0])? })
            }
            "Schedule" => {
                Self::expect_payload_count("Schedule", payload, 1)?;
                Ok(Self::Schedule { events: Vec::<SolarEvent>::from_nota_block(&payload[0])? })
            }
            "Location" => {
                Self::expect_payload_count("Location", payload, 2)?;
                Ok(Self::Location {
                    location: Location::from_nota_block(&payload[0])?,
                    source: LocationSource::from_nota_block(&payload[1])?,
                })
            }
            "Event" => {
                Self::expect_payload_count("Event", payload, 1)?;
                Ok(Self::Event { event: SolarEvent::from_nota_block(&payload[0])? })
            }
            "Error" => {
                Self::expect_payload_count("Error", payload, 1)?;
                Ok(Self::Error { message: String::from_nota_block(&payload[0])? })
            }
            other => Err(NotaDecodeError::UnknownVariant { enum_name: "Response", variant: other.to_owned() }),
        }
    }
}

impl NotaEncode for Response {
    fn to_nota(&self) -> String {
        match self {
            Self::Acked => "Acked".to_owned(),
            Self::Time { zodiacal_time } => Delimiter::Parenthesis.wrap(["Time".to_owned(), zodiacal_time.to_nota()]),
            Self::Schedule { events } => Delimiter::Parenthesis.wrap(["Schedule".to_owned(), events.to_nota()]),
            Self::Location { location, source } => {
                Delimiter::Parenthesis.wrap(["Location".to_owned(), location.to_nota(), source.to_nota()])
            }
            Self::Event { event } => Delimiter::Parenthesis.wrap(["Event".to_owned(), event.to_nota()]),
            Self::Error { message } => Delimiter::Parenthesis.wrap(["Error".to_owned(), message.to_nota()]),
        }
    }
}
