#![allow(dead_code)]
#![allow(clippy::redundant_closure)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    GetTime,
    GetSchedule,
    GetLocation,
    SetLocation(RequestSetLocation),
    UseGeoclue,
    Subscribe(RequestSubscribe),
}
impl datom_codec::Datomic for Request {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "GetTime" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::GetTime)
            }
            "GetSchedule" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::GetSchedule)
            }
            "GetLocation" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::GetLocation)
            }
            "SetLocation" => {
                std::result::Result::Ok(
                    Self::SetLocation(datom_codec::Carrying::body(v)?),
                )
            }
            "UseGeoclue" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::UseGeoclue)
            }
            "Subscribe" => {
                std::result::Result::Ok(Self::Subscribe(datom_codec::Carrying::body(v)?))
            }
            _ => {
                std::result::Result::Err(
                    datom_codec::Headed::reject(
                        &v,
                        datom_codec::Problem::UnknownVariant(
                            protos::Word::try_from(v.name).expect("variant name"),
                        ),
                    ),
                )
            }
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for Request {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::GetTime => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("GetTime").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::GetSchedule => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("GetSchedule")
                                        .expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::GetLocation => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("GetLocation")
                                        .expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::SetLocation(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("SetLocation")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::UseGeoclue => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("UseGeoclue")
                                        .expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Subscribe(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Subscribe")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                },
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestSetLocation(pub super::Latitude, pub super::Longitude);
impl datom_codec::Datomic for RequestSetLocation {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: super::Latitude = datom_codec::Positional::position(&mut p)?;
        let p1: super::Longitude = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for RequestSetLocation {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.1)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestSubscribe(pub std::vec::Vec<super::SolarEventKind>);
impl datom_codec::Datomic for RequestSubscribe {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 1)?;
        let p0: std::vec::Vec<super::SolarEventKind> = datom_codec::Positional::position(
            &mut p,
        )?;
        std::result::Result::Ok(Self(p0))
    }
}
impl protos::Conceivable<datom_codec::Datom> for RequestSubscribe {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Response {
    Acked,
    Time(ResponseTime),
    Schedule(ResponseSchedule),
    Location(ResponseLocation),
    Event(ResponseEvent),
    Error(ResponseError),
}
impl datom_codec::Datomic for Response {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Acked" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Acked)
            }
            "Time" => {
                std::result::Result::Ok(Self::Time(datom_codec::Carrying::body(v)?))
            }
            "Schedule" => {
                std::result::Result::Ok(Self::Schedule(datom_codec::Carrying::body(v)?))
            }
            "Location" => {
                std::result::Result::Ok(Self::Location(datom_codec::Carrying::body(v)?))
            }
            "Event" => {
                std::result::Result::Ok(Self::Event(datom_codec::Carrying::body(v)?))
            }
            "Error" => {
                std::result::Result::Ok(Self::Error(datom_codec::Carrying::body(v)?))
            }
            _ => {
                std::result::Result::Err(
                    datom_codec::Headed::reject(
                        &v,
                        datom_codec::Problem::UnknownVariant(
                            protos::Word::try_from(v.name).expect("variant name"),
                        ),
                    ),
                )
            }
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for Response {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::Acked => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Acked").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Time(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Time").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Schedule(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Schedule")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Location(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Location")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Event(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Event").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Error(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Error").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                },
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseTime(pub super::ZodiacalTime);
impl datom_codec::Datomic for ResponseTime {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 1)?;
        let p0: super::ZodiacalTime = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0))
    }
}
impl protos::Conceivable<datom_codec::Datom> for ResponseTime {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseSchedule(pub std::vec::Vec<super::SolarEvent>);
impl datom_codec::Datomic for ResponseSchedule {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 1)?;
        let p0: std::vec::Vec<super::SolarEvent> = datom_codec::Positional::position(
            &mut p,
        )?;
        std::result::Result::Ok(Self(p0))
    }
}
impl protos::Conceivable<datom_codec::Datom> for ResponseSchedule {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseLocation(pub super::Location, pub super::LocationSource);
impl datom_codec::Datomic for ResponseLocation {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: super::Location = datom_codec::Positional::position(&mut p)?;
        let p1: super::LocationSource = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for ResponseLocation {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.1)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseEvent(pub super::SolarEvent);
impl datom_codec::Datomic for ResponseEvent {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 1)?;
        let p0: super::SolarEvent = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0))
    }
}
impl protos::Conceivable<datom_codec::Datom> for ResponseEvent {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseError(pub super::ErrorMessage);
impl datom_codec::Datomic for ResponseError {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 1)?;
        let p0: super::ErrorMessage = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0))
    }
}
impl protos::Conceivable<datom_codec::Datom> for ResponseError {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
