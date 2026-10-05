//! Chronos's concrete parts for the standard Nexus entry point.

use std::path::PathBuf;

use nexus::{Admission, Changed, MemoryHandle, Nexus, Operating, Remembering, Signaling};
use redb::{Database, ReadableDatabase, TableDefinition};

use crate::location::{Location, LocationSource};
use crate::request::Request;
use crate::response::{ErrorMessage, Response};
use crate::wire::socket_path;

const LOCATIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("location");
const CURRENT_LOCATION: &str = "current";

/// Chronos's whole Nexus.
///
/// Signal code cannot construct the admission that opens memory:
///
/// ```compile_fail,E0451
/// let _ = nexus::Admission { directory: "/tmp".into() };
/// ```
///
/// It cannot construct a memory handle:
///
/// ```compile_fail,E0451
/// fn reach<M: nexus::Remembering>() -> nexus::MemoryHandle<M> {
///     nexus::MemoryHandle { actor: todo!() }
/// }
/// ```
///
/// Nor can it follow its operation handle to the operation actor:
///
/// ```compile_fail,E0616
/// fn reach<N: nexus::Nexus>(operation: &nexus::OperationHandle<N>) {
///     let _ = &operation.actor;
/// }
/// ```
pub struct Chronos;

/// The signal implementation selected by [`Chronos`].
pub struct ChronosSignal;

impl Signaling for ChronosSignal {
    type Query = Request;
    type Response = Response;
    type Operation = Operation;
    type Outcome = Outcome;

    fn decode(&self, frame: &[u8]) -> Option<Self::Query> {
        Request::from_archive(frame).ok()
    }

    fn encode(&self, response: Self::Response) -> Vec<u8> {
        response.archive().unwrap_or_default()
    }

    fn intend(&self, query: Self::Query) -> Self::Operation {
        match query {
            Request::SetLocation { latitude, longitude } => Operation::SetLocation(Location { latitude, longitude }),
            Request::GetLocation => Operation::GetLocation,
            _ => Operation::Unsupported,
        }
    }

    fn answer(&self, outcome: Self::Outcome) -> Self::Response {
        match outcome {
            Outcome::Set => Response::Acked,
            Outcome::Location(location) => Response::Location { location, source: LocationSource::Manual },
            Outcome::Error(message) => Response::Error {
                message: ErrorMessage::try_new(message).expect("operation error message is representable"),
            },
        }
    }

    fn undecodable(&self) -> Self::Response {
        Response::Error {
            message: ErrorMessage::try_new("undecodable request".into())
                .expect("static error message is representable"),
        }
    }
}

/// The redb-backed memory implementation selected by [`Chronos`].
pub struct ChronosMemory {
    database: Database,
}

pub enum Change {
    SetLocation(Location),
}

pub struct CurrentLocation;

impl Remembering for ChronosMemory {
    type Change = Change;
    type Reading = CurrentLocation;
    type Remembered = Option<Location>;

    fn open(admission: Admission) -> Option<Self> {
        std::fs::create_dir_all(admission.directory()).ok()?;
        Database::create(admission.directory().join("state.redb")).ok().map(|database| Self { database })
    }

    fn change(&mut self, change: Self::Change) -> Changed {
        let Change::SetLocation(location) = change;
        let Ok(bytes) = rkyv::to_bytes::<rkyv::rancor::Error>(&location) else {
            return Changed::Failed;
        };
        let written = self.database.begin_write().ok().and_then(|transaction| {
            transaction.open_table(LOCATIONS).ok()?.insert(CURRENT_LOCATION, bytes.as_slice()).ok()?;
            transaction.commit().ok()
        });
        if written.is_some() { Changed::Succeeded } else { Changed::Failed }
    }

    fn read(&self, _: Self::Reading) -> Self::Remembered {
        let transaction = self.database.begin_read().ok()?;
        let table = transaction.open_table(LOCATIONS).ok()?;
        let bytes = table.get(CURRENT_LOCATION).ok()??;
        rkyv::from_bytes::<Location, rkyv::rancor::Error>(bytes.value()).ok()
    }
}

pub enum Operation {
    SetLocation(Location),
    GetLocation,
    Unsupported,
}

pub enum Outcome {
    Set,
    Location(Location),
    Error(String),
}

/// The operation implementation selected by [`Chronos`].
pub struct ChronosOperation;

impl Operating<ChronosMemory> for ChronosOperation {
    type Operation = Operation;
    type Outcome = Outcome;

    async fn perform(&mut self, operation: Self::Operation, memory: &MemoryHandle<ChronosMemory>) -> Self::Outcome {
        match operation {
            Operation::SetLocation(location) => match memory.change(Change::SetLocation(location)).await {
                Changed::Succeeded => Outcome::Set,
                Changed::Failed => Outcome::Error("location store refused the change".into()),
            },
            Operation::GetLocation => match memory.read(CurrentLocation).await {
                Some(Some(location)) => Outcome::Location(location),
                Some(None) => Outcome::Error("location is not set".into()),
                None => Outcome::Error("location store refused the read".into()),
            },
            Operation::Unsupported => Outcome::Error("request is outside the entry experiment".into()),
        }
    }
}

impl Nexus for Chronos {
    type Memory = ChronosMemory;
    type Operation = ChronosOperation;
    type Signal = ChronosSignal;

    fn signal() -> Self::Signal {
        ChronosSignal
    }
    fn operation() -> Self::Operation {
        ChronosOperation
    }

    fn directory() -> PathBuf {
        std::env::var_os("CHRONOS_STATE_DIRECTORY")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("chronos"))
    }

    fn socket_path() -> PathBuf {
        socket_path()
    }
}
