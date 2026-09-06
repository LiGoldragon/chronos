//! The typed Datomic response rendered by the Chronos CLI.

use datom_codec::{Actualizable, Datom, Datomic, Headed, IncorporationBudget, Positional, Potential, Sited};
use protos::{Conceivable, Extent, Situated, Situation, Symbol};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::{
    error::{Error as ChronosError, Result},
    event::SolarEvent,
    location::{Location, LocationSource},
    zodiac::ZodiacalTime,
};

/// A response message representable at every canonical Datomic outbound edge.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct ErrorMessage(String);

impl ErrorMessage {
    /// Validate text before it becomes an outbound response payload.
    pub fn try_new(message: String) -> Result<Self> {
        protos::Text::try_from(message.clone())
            .map(|_| Self(message))
            .map_err(|_| ChronosError::Datomic { type_name: "ErrorMessage", problem: "unrepresentable string" })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Datomic for ErrorMessage {
    fn incorporate(site: datom_codec::Site<'_>) -> core::result::Result<Self, datom_codec::Fault> {
        Ok(Self(protos::Text::incorporate(site)?.as_ref().to_owned()))
    }
}
impl Conceivable<Datom> for ErrorMessage {
    type Fault = core::convert::Infallible;
    fn conceive(&self) -> core::result::Result<Situated<Datom>, Self::Fault> {
        protos::Text::try_from(self.0.clone()).expect("validated text").conceive()
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq)]
pub enum Response {
    Acked,
    Time { zodiacal_time: ZodiacalTime },
    Schedule { events: Vec<SolarEvent> },
    Location { location: Location, source: LocationSource },
    Event { event: SolarEvent },
    Error { message: ErrorMessage },
}

impl Response {
    pub fn from_text(source: &str) -> Result<Self> {
        Potential::<Self>::from(source)
            .actualize(IncorporationBudget::try_from(4096).expect("positive reply budget"))
            .map_err(|fault| ChronosError::from_fault("Response", fault))
    }

    pub fn to_text(&self) -> String {
        protos::Textualizable::textualize(self)
    }

    pub fn archive(&self) -> Result<Vec<u8>> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|error| ChronosError::RkyvCodec(error.to_string()))
    }

    pub fn from_archive(bytes: &[u8]) -> Result<Self> {
        rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes).map_err(|error| ChronosError::RkyvCodec(error.to_string()))
    }
}

impl Datomic for Response {
    fn incorporate(site: datom_codec::Site<'_>) -> core::result::Result<Self, datom_codec::Fault> {
        let variant = site.variant()?;
        match variant.name {
            "Acked" => {
                Headed::nothing(variant)?;
                Ok(Self::Acked)
            }
            "Time" => {
                let mut p = Headed::positions(variant, 1)?;
                Ok(Self::Time { zodiacal_time: p.position()? })
            }
            "Schedule" => {
                let mut p = Headed::positions(variant, 1)?;
                Ok(Self::Schedule { events: p.position()? })
            }
            "Location" => {
                let mut p = Headed::positions(variant, 2)?;
                Ok(Self::Location { location: p.position()?, source: p.position()? })
            }
            "Event" => {
                let mut p = Headed::positions(variant, 1)?;
                Ok(Self::Event { event: p.position()? })
            }
            "Error" => {
                let mut p = Headed::positions(variant, 1)?;
                Ok(Self::Error { message: p.position()? })
            }
            name => Err(variant
                .reject(datom_codec::Problem::UnknownVariant(protos::Word::try_from(name).expect("variant word")))),
        }
    }
}
impl Conceivable<Datom> for Response {
    type Fault = core::convert::Infallible;
    fn conceive(&self) -> core::result::Result<Situated<Datom>, Self::Fault> {
        let variant = |name, values: Vec<Datom>| {
            Datom::Variant(Symbol::try_from(name).expect("static symbol"), Box::new(Datom::Struct(values)))
        };
        let datom = match self {
            Self::Acked => Datom::Word(datom_codec::DatomWord::try_from("Acked").expect("static word")),
            Self::Time { zodiacal_time } => variant("Time", vec![zodiacal_time.conceive()?.1]),
            Self::Schedule { events } => variant("Schedule", vec![events.conceive()?.1]),
            Self::Location { location, source } => {
                variant("Location", vec![location.conceive()?.1, source.conceive()?.1])
            }
            Self::Event { event } => variant("Event", vec![event.conceive()?.1]),
            Self::Error { message } => variant("Error", vec![message.conceive()?.1]),
        };
        Ok(Situated(Situation { extent: Extent(0, 0), children: vec![] }, datom))
    }
}
