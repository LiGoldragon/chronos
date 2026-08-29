//! The typed Datomic response rendered by the Chronos CLI.

use datomic::{Datomic, DatomicString, Fault, FaultProblem, PortionBuilding, PortionViewing, Text, TextEdge};
use protos::{Portion, Separator, StructuralEnclosure};
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
        DatomicString::try_from(message.clone())
            .map(|_| Self(message))
            .map_err(|_| ChronosError::Datomic { type_name: "ErrorMessage", problem: "unrepresentable string" })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Datomic for ErrorMessage {
    fn embody(portion: &Portion) -> core::result::Result<Self, Fault> {
        Ok(Self(DatomicString::embody(portion)?.as_ref().to_owned()))
    }

    fn portion(&self) -> Portion {
        DatomicString::try_from(self.0.clone()).expect("ErrorMessage construction proved representability").portion()
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
        Text::<Self>::from(source).embody().map_err(|fault| ChronosError::from_fault("Response", fault))
    }

    pub fn to_text(&self) -> String {
        self.textualize().as_ref().to_owned()
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
    fn embody(portion: &Portion) -> core::result::Result<Self, Fault> {
        if portion.bare_symbol() == Some("Acked") {
            return Ok(Self::Acked);
        }
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if headed.separator != Separator::Period {
            return Err(portion.fault(FaultProblem::Shape));
        }
        let Some(parts) = headed.body.structural(StructuralEnclosure::Braced) else {
            return Err(headed.body.fault(FaultProblem::Shape));
        };
        match headed.head.as_ref() {
            "Time" => {
                let [time] = parts else {
                    return Err(headed.body.fault(FaultProblem::Arity));
                };
                Ok(Self::Time { zodiacal_time: ZodiacalTime::embody(time)? })
            }
            "Schedule" => {
                let [events] = parts else {
                    return Err(headed.body.fault(FaultProblem::Arity));
                };
                Ok(Self::Schedule { events: Vec::<SolarEvent>::embody(events)? })
            }
            "Location" => {
                let [location, source] = parts else {
                    return Err(headed.body.fault(FaultProblem::Arity));
                };
                Ok(Self::Location { location: Location::embody(location)?, source: LocationSource::embody(source)? })
            }
            "Event" => {
                let [event] = parts else {
                    return Err(headed.body.fault(FaultProblem::Arity));
                };
                Ok(Self::Event { event: SolarEvent::embody(event)? })
            }
            "Error" => {
                let [message] = parts else {
                    return Err(headed.body.fault(FaultProblem::Arity));
                };
                Ok(Self::Error { message: ErrorMessage::embody(message)? })
            }
            _ => Err(portion.fault(FaultProblem::Shape)),
        }
    }

    fn portion(&self) -> Portion {
        let braced = |parts| "".structural(StructuralEnclosure::Braced, parts);
        match self {
            Self::Acked => "Acked".bare(),
            Self::Time { zodiacal_time } => "Time".headed(Separator::Period, braced(vec![zodiacal_time.portion()])),
            Self::Schedule { events } => "Schedule".headed(Separator::Period, braced(vec![events.portion()])),
            Self::Location { location, source } => {
                "Location".headed(Separator::Period, braced(vec![location.portion(), source.portion()]))
            }
            Self::Event { event } => "Event".headed(Separator::Period, braced(vec![event.portion()])),
            Self::Error { message } => "Error".headed(Separator::Period, braced(vec![message.portion()])),
        }
    }
}
