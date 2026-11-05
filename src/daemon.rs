//! Chronos's concrete parts for the standard Nexus entry point.

use std::path::{Path, PathBuf};

use nexus::{Admission, Changed, MemoryHandle, Nexus, Operating, Remembering, Signaling};
use redb::{Database, ReadableDatabase, TableDefinition};

use crate::location::{Latitude, Location, LocationSource, Longitude};
use crate::place::{Place, PlacingChronos, StartupPlace};
use crate::request::Request;
use crate::response::{ErrorMessage, Response};

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

/// The durable file names accepted while opening Chronos memory.
pub struct MemoryPaths {
    pub current: PathBuf,
    pub legacy_sema: PathBuf,
    pub legacy_redb: PathBuf,
}

/// More than one durable Chronos memory exists in one state directory.
#[derive(Debug, PartialEq, Eq)]
pub struct MemoryPathConflict {
    pub paths: Vec<PathBuf>,
}

/// A typed failure to open Chronos memory.
#[derive(Debug, thiserror::Error)]
pub enum MemoryOpenError {
    #[error("several Chronos memory files exist: {0:?}")]
    PathConflict(MemoryPathConflict),
    #[error("failed to migrate Chronos memory from {from:?} to {to:?}: {source}")]
    Migration { from: PathBuf, to: PathBuf, source: std::io::Error },
    #[error("failed to open Chronos memory at {path:?}: {source}")]
    Database { path: PathBuf, source: redb::DatabaseError },
}

/// Resolves the current and legacy durable names for Chronos memory.
pub trait LocatingMemory {
    fn memory_paths(&self) -> MemoryPaths;
}

impl LocatingMemory for Path {
    fn memory_paths(&self) -> MemoryPaths {
        MemoryPaths {
            current: self.join("state.memory"),
            legacy_sema: self.join("state.sema"),
            legacy_redb: self.join("state.redb"),
        }
    }
}

/// Opens Chronos memory while migrating a sole legacy durable name.
pub trait OpeningMemory: Sized {
    fn open_memory(directory: &Path) -> Result<Self, MemoryOpenError>;
}

impl OpeningMemory for ChronosMemory {
    fn open_memory(directory: &Path) -> Result<Self, MemoryOpenError> {
        let paths = directory.memory_paths();
        let present: Vec<PathBuf> = [&paths.current, &paths.legacy_sema, &paths.legacy_redb]
            .into_iter()
            .filter(|path| path.exists())
            .cloned()
            .collect();
        let path = match present.as_slice() {
            [] => paths.current,
            [path] if *path == paths.current => paths.current,
            [legacy] => {
                std::fs::rename(legacy, &paths.current).map_err(|source| MemoryOpenError::Migration {
                    from: legacy.clone(),
                    to: paths.current.clone(),
                    source,
                })?;
                paths.current
            }
            _ => return Err(MemoryOpenError::PathConflict(MemoryPathConflict { paths: present })),
        };
        Database::create(&path)
            .map(|database| Self { database })
            .map_err(|source| MemoryOpenError::Database { path, source })
    }
}

pub enum Change {
    SetLocation(Location),
}

pub struct CurrentLocation;

/// The result of reading the durable current-location slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RememberedLocation {
    Unset,
    Stored(Location),
    Failed,
}

impl Remembering for ChronosMemory {
    type Change = Change;
    type Reading = CurrentLocation;
    type Remembered = RememberedLocation;

    fn open(admission: Admission) -> Option<Self> {
        std::fs::create_dir_all(admission.directory()).ok()?;
        Self::open_memory(admission.directory()).ok()
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
        let Ok(transaction) = self.database.begin_read() else {
            return RememberedLocation::Failed;
        };
        let table = match transaction.open_table(LOCATIONS) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return RememberedLocation::Unset,
            Err(_) => return RememberedLocation::Failed,
        };
        let bytes = match table.get(CURRENT_LOCATION) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return RememberedLocation::Unset,
            Err(_) => return RememberedLocation::Failed,
        };
        let Ok(location) = rkyv::from_bytes::<Location, rkyv::rancor::Error>(bytes.value()) else {
            return RememberedLocation::Failed;
        };
        let (Ok(latitude), Ok(longitude)) =
            (Latitude::try_new(location.latitude.as_degrees()), Longitude::try_new(location.longitude.as_degrees()))
        else {
            return RememberedLocation::Failed;
        };
        RememberedLocation::Stored(Location { latitude, longitude })
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
                Some(RememberedLocation::Stored(location)) => Outcome::Location(location),
                Some(RememberedLocation::Unset) => Outcome::Error("location is not set".into()),
                Some(RememberedLocation::Failed) | None => Outcome::Error("location store refused the read".into()),
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
        Place::installed().expect("Chronos starts with a Place datom").state().to_owned()
    }

    fn socket_path() -> PathBuf {
        Place::installed().expect("Chronos starts with a Place datom").ordinary_socket()
    }
}
