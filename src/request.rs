//! The typed Datomic request accepted at Chronos's CLI boundary.

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::{
    error::{Error, Result},
    event::SolarEventKind,
    location::{Latitude, Longitude},
};

#[derive(
    datom_codec::Composing, datom_codec::Datomizable, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq,
)]
pub enum Request {
    GetTime,
    GetSchedule,
    GetLocation,
    SetLocation(RequestSetLocation),
    UseGeoclue,
    Subscribe(RequestSubscribe),
}

/// The payload of [`Request::SetLocation`]: the observer's new position, two struct positions.
#[derive(
    datom_codec::Composing, datom_codec::Datomizable, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq,
)]
pub struct RequestSetLocation {
    pub latitude: Latitude,
    pub longitude: Longitude,
}

/// The payload of [`Request::Subscribe`]: the kinds to receive, one struct position.
#[derive(
    datom_codec::Composing, datom_codec::Datomizable, Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq,
)]
pub struct RequestSubscribe {
    pub kinds: Vec<SolarEventKind>,
}

impl Request {
    pub fn from_text(source: &str) -> Result<Self> {
        crate::text::read("Request", source)
    }

    pub fn to_text(&self) -> String {
        crate::text::print(self)
    }

    pub fn archive(&self) -> Result<Vec<u8>> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|error| Error::RkyvCodec(error.to_string()))
    }

    pub fn from_archive(bytes: &[u8]) -> Result<Self> {
        rkyv::from_bytes::<Self, crate::archive::ArchiveFault>(bytes).map_err(Into::into)
    }
}
