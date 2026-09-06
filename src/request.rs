//! The typed Datomic request accepted at Chronos's CLI boundary.

use datom_codec::{Actualizable, Datom, Datomic, Headed, IncorporationBudget, Positional, Sited};
use protos::{Conceivable, Extent, Potential, Situated, Situation, Symbol};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::{
    error::{Error, Result},
    event::SolarEventKind,
    location::{Latitude, Longitude},
};

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq)]
pub enum Request {
    GetTime,
    GetSchedule,
    GetLocation,
    SetLocation { latitude: Latitude, longitude: Longitude },
    UseGeoclue,
    Subscribe { kinds: Vec<SolarEventKind> },
}

impl Request {
    pub fn from_text(source: &str) -> Result<Self> {
        Potential::<Self>::from(source)
            .actualize(IncorporationBudget::try_from(4096).expect("positive request budget"))
            .map_err(|fault| Error::from_fault("Request", fault))
    }

    pub fn to_text(&self) -> String {
        protos::Textualizable::textualize(self)
    }

    pub fn archive(&self) -> Result<Vec<u8>> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|error| Error::RkyvCodec(error.to_string()))
    }

    pub fn from_archive(bytes: &[u8]) -> Result<Self> {
        rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes).map_err(|error| Error::RkyvCodec(error.to_string()))
    }
}

impl Datomic for Request {
    fn incorporate(site: datom_codec::Site<'_>) -> core::result::Result<Self, datom_codec::Fault> {
        let variant = site.variant()?;
        match variant.name {
            "GetTime" => { Headed::nothing(variant)?; Ok(Self::GetTime) }
            "GetSchedule" => { Headed::nothing(variant)?; Ok(Self::GetSchedule) }
            "GetLocation" => { Headed::nothing(variant)?; Ok(Self::GetLocation) }
            "UseGeoclue" => { Headed::nothing(variant)?; Ok(Self::UseGeoclue) }
            "SetLocation" => { let mut p = Headed::positions(variant, 2)?; Ok(Self::SetLocation { latitude: p.position()?, longitude: p.position()? }) }
            "Subscribe" => { let mut p = Headed::positions(variant, 1)?; Ok(Self::Subscribe { kinds: p.position()? }) }
            name => Err(variant.reject(datom_codec::Problem::UnknownVariant(protos::Word::try_from(name).expect("variant word")))),
        }
    }
}

impl Conceivable<Datom> for Request {
    type Fault = core::convert::Infallible;
    fn conceive(&self) -> core::result::Result<Situated<Datom>, Self::Fault> {
        let body = |value: &dyn Conceivable<Datom, Fault = core::convert::Infallible>| value.conceive().expect("infallible Datom ascent").1;
        let datom = match self {
            Self::GetTime => Datom::Word(datom_codec::DatomWord::try_from("GetTime").expect("static word")),
            Self::GetSchedule => Datom::Word(datom_codec::DatomWord::try_from("GetSchedule").expect("static word")),
            Self::GetLocation => Datom::Word(datom_codec::DatomWord::try_from("GetLocation").expect("static word")),
            Self::UseGeoclue => Datom::Word(datom_codec::DatomWord::try_from("UseGeoclue").expect("static word")),
            Self::SetLocation { latitude, longitude } => Datom::Variant(Symbol::try_from("SetLocation").expect("static symbol"), Box::new(Datom::Struct(vec![body(latitude), body(longitude)]))),
            Self::Subscribe { kinds } => Datom::Variant(Symbol::try_from("Subscribe").expect("static symbol"), Box::new(Datom::Struct(vec![body(kinds)]))),
        };
        Ok(Situated(Situation { extent: Extent(0, 0), children: vec![] }, datom))
    }
}
