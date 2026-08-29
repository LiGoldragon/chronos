//! The typed Datomic request accepted at Chronos's CLI boundary.

use datomic::{Datomic, Fault, FaultProblem, PortionBuilding, PortionViewing, Text, TextEdge};
use protos::{Portion, Separator, StructuralEnclosure};
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
        Text::<Self>::from(source).embody().map_err(|fault| Error::from_fault("Request", fault))
    }

    pub fn to_text(&self) -> String {
        self.textualize().as_ref().to_owned()
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
    fn embody(portion: &Portion) -> core::result::Result<Self, Fault> {
        match portion.bare_symbol() {
            Some("GetTime") => return Ok(Self::GetTime),
            Some("GetSchedule") => return Ok(Self::GetSchedule),
            Some("GetLocation") => return Ok(Self::GetLocation),
            Some("UseGeoclue") => return Ok(Self::UseGeoclue),
            _ => {}
        }
        let Some(headed) = portion.headed() else {
            return Err(portion.fault(FaultProblem::Shape));
        };
        if headed.separator != Separator::Period {
            return Err(portion.fault(FaultProblem::Shape));
        }
        match headed.head.as_ref() {
            "SetLocation" => {
                let Some(parts) = headed.body.structural(StructuralEnclosure::Braced) else {
                    return Err(headed.body.fault(FaultProblem::Shape));
                };
                let [latitude, longitude] = parts else {
                    return Err(headed.body.fault(FaultProblem::Arity));
                };
                Ok(Self::SetLocation {
                    latitude: Latitude::embody(latitude)?,
                    longitude: Longitude::embody(longitude)?,
                })
            }
            "Subscribe" => {
                let Some(parts) = headed.body.structural(StructuralEnclosure::Braced) else {
                    return Err(headed.body.fault(FaultProblem::Shape));
                };
                let [kinds] = parts else {
                    return Err(headed.body.fault(FaultProblem::Arity));
                };
                Ok(Self::Subscribe { kinds: Vec::<SolarEventKind>::embody(kinds)? })
            }
            _ => Err(portion.fault(FaultProblem::Shape)),
        }
    }

    fn portion(&self) -> Portion {
        match self {
            Self::GetTime => "GetTime".bare(),
            Self::GetSchedule => "GetSchedule".bare(),
            Self::GetLocation => "GetLocation".bare(),
            Self::UseGeoclue => "UseGeoclue".bare(),
            Self::SetLocation { latitude, longitude } => "SetLocation".headed(
                Separator::Period,
                "".structural(StructuralEnclosure::Braced, vec![latitude.portion(), longitude.portion()]),
            ),
            Self::Subscribe { kinds } => {
                "Subscribe".headed(Separator::Period, "".structural(StructuralEnclosure::Braced, vec![kinds.portion()]))
            }
        }
    }
}
