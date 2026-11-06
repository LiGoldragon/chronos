//! Checked composition for the invariant-bearing scalars.
//!
//! A domain value is admitted by its one constructor, `try_new`. These macros
//! compose the inner scalar from a datom, hand it to that constructor, and
//! refuse with a positioned `ErrorKind::Value` when the constructor refuses, so
//! no datom entry admits a value `try_new` would not. The paired `Datomizable`
//! writes the valid value back as its inner scalar.

use datom_codec::{Datom, Error, ErrorKind, ErrorRaising};

/// The refusal for the value a datom carried at its own path.
pub(crate) fn refusal(datom: &Datom, expected: &'static str, value: String) -> Error {
    Error::composition(datom.path.clone(), ErrorKind::Value { expected: expected.into(), value })
}

/// A scalar over a finite `f64`, built by `Self::try_new(f64)`.
macro_rules! decimal_scalar {
    ($type:ident) => {
        impl datom_codec::Composing for $type {
            fn compose(
                datom: &datom_codec::Datom,
                budget: &mut datom_codec::Budget,
            ) -> core::result::Result<Self, datom_codec::Error> {
                let decimal = <datom_codec::Decimal as datom_codec::Composing>::compose(datom, budget)?;
                let value = datom_codec::Decimating::float(&decimal);
                Self::try_new(value).map_err(|_| $crate::scalar::refusal(datom, stringify!($type), value.to_string()))
            }
        }
        impl datom_codec::Datomizable for $type {
            fn datomize(&self, at: datom_codec::Path) -> datom_codec::Datom {
                datom_codec::Datomizable::datomize(
                    &datom_codec::Decimal::try_from(self.0).expect("a valid value is finite"),
                    at,
                )
            }
        }
    };
}

/// A scalar over a small integer, built by `Self::try_new(inner)`.
macro_rules! integer_scalar {
    ($type:ident, $inner:ty) => {
        impl datom_codec::Composing for $type {
            fn compose(
                datom: &datom_codec::Datom,
                budget: &mut datom_codec::Budget,
            ) -> core::result::Result<Self, datom_codec::Error> {
                let value = <i64 as datom_codec::Composing>::compose(datom, budget)?;
                <$inner>::try_from(value)
                    .ok()
                    .and_then(|inner| Self::try_new(inner).ok())
                    .ok_or_else(|| $crate::scalar::refusal(datom, stringify!($type), value.to_string()))
            }
        }
        impl datom_codec::Datomizable for $type {
            fn datomize(&self, at: datom_codec::Path) -> datom_codec::Datom {
                datom_codec::Datomizable::datomize(&i64::from(self.0), at)
            }
        }
    };
}

pub(crate) use decimal_scalar;
pub(crate) use integer_scalar;
