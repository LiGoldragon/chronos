//! The text edge for tests: read one datom text as a `T`, print a `T`.
#![allow(dead_code)]

use datom_codec::{Actualizing, Budget, Composing, Datomizable, Error, Path, Potential};
use protos::{Compactable, Protosizable, ReaderBudget};

fn budget() -> Budget {
    Budget {
        remaining: 4096,
        reader: ReaderBudget::try_from(4096).expect("the reader budget is within range"),
        depth: 0,
        maximum_depth: 4096,
    }
}

pub fn read<T: Composing>(source: &str) -> Result<T, Error> {
    Potential::<T>::from(source).actualize(&mut budget())
}

pub fn print<T: Datomizable>(value: &T) -> String {
    value.datomize(Path::new()).protosize().compact()
}
