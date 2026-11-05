//! The typed place in which one Chronos Nexus starts.

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use datom_codec::{Actualizable, Datom, Datomic, IncorporationBudget, Positional, Potential, Sited};
use protos::{Conceivable, Extent, Situated, Situation};

use crate::{Error, Result};

const NEXUS_DIRECTORY: &str = "nexus";
const ORDINARY_SOCKET: &str = "chronos.sock";

static STARTUP_PLACE: OnceLock<Place> = OnceLock::new();

/// The name distinguishing one running Chronos instance from another.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceName(String);

/// The runtime and durable state directories supplied to Chronos at startup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    name: PlaceName,
    runtime: PathBuf,
    state: PathBuf,
}

/// Builds and reads the paths belonging to a typed Nexus place.
pub trait PlacingChronos: Sized {
    fn from_paths(name: String, runtime: PathBuf, state: PathBuf) -> Self;
    fn main_from_anchors(runtime_root: &Path, home: &Path) -> Self;
    fn ordinary_socket(&self) -> PathBuf;
    fn runtime(&self) -> &Path;
    fn state(&self) -> &Path;
}

impl PlacingChronos for Place {
    fn from_paths(name: String, runtime: PathBuf, state: PathBuf) -> Self {
        Self { name: PlaceName(name), runtime, state }
    }

    fn main_from_anchors(runtime_root: &Path, home: &Path) -> Self {
        Self::from_paths(
            "main".into(),
            runtime_root.join(NEXUS_DIRECTORY).join("main"),
            home.join(".local/state").join(NEXUS_DIRECTORY).join("main"),
        )
    }

    fn ordinary_socket(&self) -> PathBuf {
        self.runtime.join(ORDINARY_SOCKET)
    }

    fn runtime(&self) -> &Path {
        &self.runtime
    }

    fn state(&self) -> &Path {
        &self.state
    }
}

/// Resolves the CLI's fixed default place from standard XDG anchors.
pub trait DefaultPlace: Sized {
    fn main_for_process() -> Self;
}

impl DefaultPlace for Place {
    fn main_for_process() -> Self {
        let uid =
            std::fs::metadata("/proc/self").map(|metadata| std::os::unix::fs::MetadataExt::uid(&metadata)).unwrap_or(0);
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| PathBuf::from(format!("/run/user/{uid}")));
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| PathBuf::from("/"));
        Self::main_from_anchors(&runtime, &home)
    }
}

/// Embodies the startup datom as a typed place.
pub trait EmbodyingPlace: Sized {
    fn from_text(source: &str) -> Result<Self>;
}

impl EmbodyingPlace for Place {
    fn from_text(source: &str) -> Result<Self> {
        Potential::<Self>::from(source)
            .actualize(IncorporationBudget::try_from(1024).expect("positive startup budget"))
            .map_err(|fault| Error::from_fault("Place", fault))
    }
}

/// Installs and reads the process's single startup place.
pub trait StartupPlace: Sized {
    fn install(self) -> core::result::Result<(), Self>;
    fn installed() -> Option<&'static Self>;
}

impl StartupPlace for Place {
    fn install(self) -> core::result::Result<(), Self> {
        STARTUP_PLACE.set(self)
    }

    fn installed() -> Option<&'static Self> {
        STARTUP_PLACE.get()
    }
}

impl Datomic for Place {
    fn incorporate(site: datom_codec::Site<'_>) -> core::result::Result<Self, datom_codec::Fault> {
        let mut positions = site.positions(3)?;
        let name: protos::Text = positions.position()?;
        let runtime: protos::Text = positions.position()?;
        let state: protos::Text = positions.position()?;
        Ok(Self::from_paths(name.as_ref().to_owned(), PathBuf::from(runtime.as_ref()), PathBuf::from(state.as_ref())))
    }
}

impl Conceivable<Datom> for Place {
    type Fault = core::convert::Infallible;

    fn conceive(&self) -> core::result::Result<Situated<Datom>, Self::Fault> {
        let text = |value: &str| {
            protos::Text::try_from(value.to_owned())
                .expect("startup place paths are representable")
                .conceive()
                .expect("infallible datom ascent")
                .1
        };
        Ok(Situated(
            Situation { extent: Extent(0, 0), children: vec![] },
            Datom::Struct(vec![
                text(&self.name.0),
                text(self.runtime.to_str().expect("startup runtime path is UTF-8")),
                text(self.state.to_str().expect("startup state path is UTF-8")),
            ]),
        ))
    }
}
