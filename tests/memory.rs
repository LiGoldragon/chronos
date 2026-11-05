use std::path::{Path, PathBuf};

use chronos::daemon::{
    Change, ChronosMemory, CurrentLocation, LocatingMemory, MemoryOpenError, MemoryPathConflict, OpeningMemory,
    RememberedLocation,
};
use chronos::{Latitude, Location, Longitude};
use nexus::{Changed, Remembering};
use redb::{Database, TableDefinition};

const LOCATIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("location");

fn location() -> Location {
    Location {
        latitude: Latitude::try_new(19.4326).expect("valid latitude"),
        longitude: Longitude::try_new(-99.1332).expect("valid longitude"),
    }
}

fn stored_memory(directory: &Path) {
    let mut memory = ChronosMemory::open_memory(directory).expect("open fresh memory");
    assert_eq!(memory.change(Change::SetLocation(location())), Changed::Succeeded);
}

fn assert_legacy_migrates(legacy_path: impl Fn(&chronos::daemon::MemoryPaths) -> &PathBuf) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let paths = directory.path().memory_paths();
    stored_memory(directory.path());
    std::fs::rename(&paths.current, legacy_path(&paths)).expect("prepare legacy memory name");

    let memory = ChronosMemory::open_memory(directory.path()).expect("migrate legacy memory");

    assert!(paths.current.exists());
    assert!(!legacy_path(&paths).exists());
    assert_eq!(memory.read(CurrentLocation), RememberedLocation::Stored(location()));
}

#[test]
fn fresh_open_creates_a_memory_file() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let paths = directory.path().memory_paths();

    ChronosMemory::open_memory(directory.path()).expect("open fresh memory");

    assert!(paths.current.exists());
    assert!(!paths.legacy_sema.exists());
    assert!(!paths.legacy_redb.exists());
}

#[test]
fn current_memory_opens_without_renaming() {
    let directory = tempfile::tempdir().expect("temporary directory");
    stored_memory(directory.path());

    let memory = ChronosMemory::open_memory(directory.path()).expect("reopen current memory");

    assert_eq!(memory.read(CurrentLocation), RememberedLocation::Stored(location()));
}

#[test]
fn absent_location_is_distinct_from_a_failed_read() {
    let fresh_directory = tempfile::tempdir().expect("fresh temporary directory");
    let fresh = ChronosMemory::open_memory(fresh_directory.path()).expect("open fresh memory");
    assert_eq!(fresh.read(CurrentLocation), RememberedLocation::Unset);

    let corrupt_directory = tempfile::tempdir().expect("corrupt temporary directory");
    let paths = corrupt_directory.path().memory_paths();
    let database = Database::create(&paths.current).expect("create corrupt fixture database");
    let transaction = database.begin_write().expect("begin corrupt fixture write");
    transaction
        .open_table(LOCATIONS)
        .expect("open location table")
        .insert("current", &[0xff, 0x00][..])
        .expect("insert invalid archive");
    transaction.commit().expect("commit invalid archive");
    drop(database);
    let corrupt = ChronosMemory::open_memory(corrupt_directory.path()).expect("open corrupt memory");
    assert_eq!(corrupt.read(CurrentLocation), RememberedLocation::Failed);
}

#[test]
fn archived_location_is_revalidated_after_loading() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let paths = directory.path().memory_paths();
    let mut archive = rkyv::to_bytes::<rkyv::rancor::Error>(&location()).expect("archive valid location");
    let valid_latitude = location().latitude.as_degrees().to_le_bytes();
    let invalid_latitude = 91.0_f64.to_le_bytes();
    let latitude_offset = archive
        .windows(valid_latitude.len())
        .position(|window| window == valid_latitude)
        .expect("find archived latitude");
    archive[latitude_offset..latitude_offset + invalid_latitude.len()].copy_from_slice(&invalid_latitude);

    let database = Database::create(&paths.current).expect("create semantic corruption fixture");
    let transaction = database.begin_write().expect("begin semantic corruption fixture write");
    transaction
        .open_table(LOCATIONS)
        .expect("open location table")
        .insert("current", archive.as_slice())
        .expect("insert semantically invalid archive");
    transaction.commit().expect("commit semantically invalid archive");
    drop(database);

    let memory = ChronosMemory::open_memory(directory.path()).expect("open semantic corruption fixture");
    assert_eq!(memory.read(CurrentLocation), RememberedLocation::Failed);
}

#[test]
fn sole_sema_memory_migrates_with_its_records() {
    assert_legacy_migrates(|paths| &paths.legacy_sema);
}

#[test]
fn sole_redb_memory_migrates_with_its_records() {
    assert_legacy_migrates(|paths| &paths.legacy_redb);
}

#[test]
fn several_memory_names_are_a_typed_conflict_and_stay_in_place() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let paths = directory.path().memory_paths();
    std::fs::write(&paths.current, []).expect("current marker");
    std::fs::write(&paths.legacy_sema, []).expect("sema marker");

    let conflict = match ChronosMemory::open_memory(directory.path()) {
        Err(MemoryOpenError::PathConflict(conflict)) => conflict,
        _ => panic!("conflicting memory names must return a path conflict"),
    };

    assert_eq!(conflict, MemoryPathConflict { paths: vec![paths.current.clone(), paths.legacy_sema.clone()] });
    assert!(paths.current.exists());
    assert!(paths.legacy_sema.exists());
}

#[test]
fn two_legacy_memory_names_are_a_typed_conflict_and_stay_in_place() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let paths = directory.path().memory_paths();
    std::fs::write(&paths.legacy_sema, []).expect("sema marker");
    std::fs::write(&paths.legacy_redb, []).expect("redb marker");

    let conflict = match ChronosMemory::open_memory(directory.path()) {
        Err(MemoryOpenError::PathConflict(conflict)) => conflict,
        _ => panic!("conflicting legacy names must return a path conflict"),
    };

    assert_eq!(conflict, MemoryPathConflict { paths: vec![paths.legacy_sema.clone(), paths.legacy_redb.clone()] });
    assert!(paths.legacy_sema.exists());
    assert!(paths.legacy_redb.exists());
}
