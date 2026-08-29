# Upgrades

## 0.2.0 — Datomic and Ethos boundary

Version 0.2.0 is a breaking textual-boundary release.

- Chronos accepts one typed Datomic `Request` on argv and prints one canonical
  Datomic `Response`. The legacy Dotos codec is removed completely; do not add
  a translation path or a second parser.
- The complete contract is authored in `.ethos/data/chronos.ethos`; its exact
  `ethos-zero` Datomic library emission is committed beside it. Range-bearing
  numeric domain values retain hand D3 implementations because their invariant
  checks are not a generated scalar shape.
- Request payloads are braced, for example
  `SetLocation.{47.6 -122.3}` and `Subscribe.{[CivilDawn CivilDusk]}`.
- `Response::Error` now requires a checked `ErrorMessage`. Construct it with
  `ErrorMessage::try_new` before returning an error; unrepresentable text is
  refused before it can reach the outbound Datomic edge.
- The rkyv UDS frame contract remains unchanged.
