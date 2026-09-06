# Upgrades

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
