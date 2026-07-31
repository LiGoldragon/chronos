//! [`Response`] — what the daemon replies with.
//!
//! Encodes both one-shot replies (`Time`, `Schedule`, `Location`,
//! `Acked`, `Error`) and subscription frames (`Event`). A
//! subscriber receives one `Event` frame per fire on a long-
//! lived connection.

use dotos::{Block, Delimiter, DotosBlock, DotosDecode, DotosDecodeError, DotosEncode, DotosSource};
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
    /// Parse a single DOTOS record into a typed response.
    pub fn from_dotos(text: &str) -> Result<Self> {
        Ok(DotosSource::new(text).parse::<Self>()?)
    }

    /// Render this response as a DOTOS record.
    pub fn to_dotos(&self) -> Result<String> {
        Ok(DotosEncode::to_dotos(self))
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
    ) -> core::result::Result<(), DotosDecodeError> {
        if payload.len() == expected {
            Ok(())
        } else {
            Err(DotosDecodeError::ExpectedRootCount { type_name: tag, expected, found: payload.len() })
        }
    }
}

impl DotosDecode for Response {
    fn from_dotos_block(block: &Block) -> core::result::Result<Self, DotosDecodeError> {
        if let Some(tag) = block.demote_to_string() {
            return match tag {
                "Acked" => Ok(Self::Acked),
                other => Err(DotosDecodeError::UnknownVariant { enum_name: "Response", variant: other.to_owned() }),
            };
        }

        let (head, payload) = block.as_application().ok_or(DotosDecodeError::ExpectedDelimited {
            type_name: "Response",
            delimiter: "Response.(payload) application",
        })?;
        let tag = head.demote_to_string().ok_or(DotosDecodeError::ExpectedAtom { type_name: "Response variant" })?;
        let payload = DotosBlock::new(payload).expect_delimited(Delimiter::Parenthesis, "Response")?;
        match tag {
            "Time" => {
                Self::expect_payload_count("Time", payload, 1)?;
                Ok(Self::Time { zodiacal_time: ZodiacalTime::from_dotos_block(&payload[0])? })
            }
            "Schedule" => {
                Self::expect_payload_count("Schedule", payload, 1)?;
                Ok(Self::Schedule { events: Vec::<SolarEvent>::from_dotos_block(&payload[0])? })
            }
            "Location" => {
                Self::expect_payload_count("Location", payload, 2)?;
                Ok(Self::Location {
                    location: Location::from_dotos_block(&payload[0])?,
                    source: LocationSource::from_dotos_block(&payload[1])?,
                })
            }
            "Event" => {
                Self::expect_payload_count("Event", payload, 1)?;
                Ok(Self::Event { event: SolarEvent::from_dotos_block(&payload[0])? })
            }
            "Error" => {
                Self::expect_payload_count("Error", payload, 1)?;
                Ok(Self::Error { message: String::from_dotos_block(&payload[0])? })
            }
            other => Err(DotosDecodeError::UnknownVariant { enum_name: "Response", variant: other.to_owned() }),
        }
    }
}

impl DotosEncode for Response {
    fn to_dotos(&self) -> String {
        match self {
            Self::Acked => "Acked".to_owned(),
            Self::Time { zodiacal_time } => format!("Time.{}", Delimiter::Parenthesis.wrap([zodiacal_time.to_dotos()])),
            Self::Schedule { events } => format!("Schedule.{}", Delimiter::Parenthesis.wrap([events.to_dotos()])),
            Self::Location { location, source } => {
                format!("Location.{}", Delimiter::Parenthesis.wrap([location.to_dotos(), source.to_dotos()]))
            }
            Self::Event { event } => format!("Event.{}", Delimiter::Parenthesis.wrap([event.to_dotos()])),
            Self::Error { message } => format!("Error.{}", Delimiter::Parenthesis.wrap([message.to_dotos()])),
        }
    }
}
