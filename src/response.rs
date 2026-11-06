//! The typed Datomic response rendered by the Chronos CLI.

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::{
    error::{Error as ChronosError, Result},
    event::SolarEvent,
    location::{Location, LocationSource},
    zodiac::ZodiacalTime,
};

/// A response message. Every string is representable as datom text, so any text is admitted.
#[derive(
    datom_codec::Composing,
    datom_codec::Datomizable,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
)]
pub struct ErrorMessage(String);

impl ErrorMessage {
    pub fn new(message: String) -> Self {
        Self(message)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    datom_codec::Composing, datom_codec::Datomizable, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq,
)]
pub enum Response {
    Acked,
    Time(ResponseTime),
    Schedule(ResponseSchedule),
    Location(ResponseLocation),
    Event(ResponseEvent),
    Error(ResponseError),
}

/// The payload of [`Response::Location`], two struct positions.
#[derive(
    datom_codec::Composing, datom_codec::Datomizable, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq,
)]
pub struct ResponseLocation {
    pub location: Location,
    pub source: LocationSource,
}

/// The payload of [`Response::Time`], one struct position.
#[derive(
    datom_codec::Composing, datom_codec::Datomizable, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq,
)]
pub struct ResponseTime {
    pub zodiacal_time: ZodiacalTime,
}

/// The payload of [`Response::Schedule`], one struct position.
#[derive(
    datom_codec::Composing, datom_codec::Datomizable, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq,
)]
pub struct ResponseSchedule {
    pub events: Vec<SolarEvent>,
}

/// The payload of [`Response::Event`], one struct position.
#[derive(
    datom_codec::Composing, datom_codec::Datomizable, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq,
)]
pub struct ResponseEvent {
    pub event: SolarEvent,
}

/// The payload of [`Response::Error`], one struct position.
#[derive(
    datom_codec::Composing, datom_codec::Datomizable, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq,
)]
pub struct ResponseError {
    pub message: ErrorMessage,
}

impl Response {
    pub fn from_text(source: &str) -> Result<Self> {
        crate::text::read("Response", source)
    }

    pub fn to_text(&self) -> String {
        crate::text::print(self)
    }

    pub fn archive(&self) -> Result<Vec<u8>> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|error| ChronosError::RkyvCodec(error.to_string()))
    }

    pub fn from_archive(bytes: &[u8]) -> Result<Self> {
        rkyv::from_bytes::<Self, crate::archive::ArchiveFault>(bytes).map_err(Into::into)
    }
}
