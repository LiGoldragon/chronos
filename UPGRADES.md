# Upgrades

## 0.4.0 — Datom-codec 0.33.0

Repins datom-codec 0.33.0 (eabe), protos 0.33.0 (3c7a) and the dev dependency ethos-zero 19.2.0 (5a5d), spelled as ethos-zero spells them so the lock holds one source of each. The Rust toolchain is selected by version, `1.99.1`, in place of the floating `stable`, and the flake's toolchain hash is that release's manifest hash.

What breaks in the library:

- The hand `Datomic`/`Conceivable` codec is gone. Enums and structs derive `Composing` and `Datomizable`; the invariant-bearing scalars (`Latitude`, `Longitude`, `OrdinalSolarTime`, `AmYear`, `EclipticLongitude`, `ZodiacDegree`, `ZodiacMinute`) implement them by hand through one checked composition: the inner scalar is composed, `try_new` is called, and a refusal is a positioned `ErrorKind::Value` naming the type. Every text entry, nested paths included, admits only what `try_new` admits.
- `Request` and `Response` variants carry the payload structs the authored Ethos names: `Request::SetLocation(RequestSetLocation)`, `Request::Subscribe(RequestSubscribe)`, `Response::Time(ResponseTime)`, `Schedule(ResponseSchedule)`, `Location(ResponseLocation)`, `Event(ResponseEvent)`, `Error(ResponseError)`. The datom text is unchanged, for example `SetLocation.{ 47.6 -122.3 }` and `Subscribe.{ [ CivilDawn CivilDusk ] }`.
- `ErrorMessage::try_new` is `ErrorMessage::new`: every string is representable datom text. Strings print in guillemets, `Error.{ «sky missing» }`.
- `chronos::Error::Datomic` carries the codec's positioned error as text; `Error::from_fault` is `Error::from_datom`.
- `.ethos/data/chronos.ethos` is in the current Ethos syntax and its projection is regenerated.

Archive admission: the rkyv archive of the six ranged scalars (`Latitude`, `Longitude`, `OrdinalSolarTime`, `EclipticLongitude`, `ZodiacDegree`, `ZodiacMinute`) is reconstructed through the same `try_new`, by a hand `rkyv::Deserialize` for each archived type; the archived representation and every byte layout are unchanged (`AmYear` admits any `i32`, so its archive needs no check). A direct `rkyv::from_bytes`, a nested one through `Location`, `SolarEvent`, `ZodiacalTime`, `Request` or `Response`, and the frame entry points `Request::from_archive` and `Response::from_archive` refuse an out-of-range or non-finite value; the frame entry points return `Error::OutOfRange`, and `ArchiveFault` is the typed rkyv error that carries it. Bytecheck still validates structure only. A value already archived out of range is refused on read, never clamped or migrated.

How to deploy: build chronos 0.4.0 and replace the binaries. No stored state changes.

## 0.3.0 — current Datom and Ethos boundary

Version 0.3.0 is a breaking textual-boundary release.

- Chronos accepts one typed `datom-codec` `Request` on argv and prints one
  canonical `Response`. The retired `datomic`/Dotos codec is removed
  completely; do not add a translation path or a second parser.
- The complete contract is authored in `.ethos/data/chronos.ethos`; its exact
  current `ethos-zero` projection is committed beside it. Range-bearing numeric
  domain values retain hand implementations because their invariant checks are
  not a generated scalar shape.
- Request payloads use the current braced form, for example
  `SetLocation.{ 47.6 -122.3 }` and
  `Subscribe.{ [ CivilDawn CivilDusk ] }`. Canonical output includes spaces;
  compact accepted input has no separate compatibility parser.
- `Response::Error` carries typed Datom text. Bare words are refused where a
  text payload is required.
- The rkyv UDS frame contract remains unchanged.
