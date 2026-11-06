//! The text edge: one budget, one reading, one printing, for every datom type.

use datom_codec::{Actualizing, Budget, Composing, Datomizable, Path, Potential};
use protos::{Compactable, Protosizable, ReaderBudget};

use crate::error::{Error, Result};

const REQUEST_BUDGET: i64 = 4096;
const READER_BUDGET: usize = 4096;
const DEPTH_BUDGET: i64 = 4096;

fn budget() -> Budget {
    Budget {
        remaining: REQUEST_BUDGET,
        reader: ReaderBudget::try_from(READER_BUDGET).expect("the reader budget is within range"),
        depth: 0,
        maximum_depth: DEPTH_BUDGET,
    }
}

/// Read one datom text as a `T`, every ranged value checked by its own composition.
pub(crate) fn read<T: Composing>(type_name: &'static str, source: &str) -> Result<T> {
    Potential::<T>::from(source).actualize(&mut budget()).map_err(|error| Error::from_datom(type_name, error))
}

/// Print a `T` as one-line datom text.
pub(crate) fn print<T: Datomizable>(value: &T) -> String {
    value.datomize(Path::new()).protosize().compact()
}
