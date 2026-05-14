# Skill — chronos

*The time-and-sky daemon. Zodiacal time, sunrise/sunset,
twilight events. Push subscriptions for chroma; one-shot
CLI for humans.*

---

## What this repo is

Chronos is the **canonical owner** of the desktop's
relationship to celestial time at the local observer's
position: which zodiacal time it is, where the sun is, when
the next civil dawn fires. Anything that schedules or
publishes solar events belongs in this repo; anything that
only *consumes* them subscribes to the daemon.

The capability is *publishing the local sky as typed
events*. The pipeline is JPL DE440 (`anise`) + NREL SPA
(`solar-positioning`) + UTC↔TT↔TDB (`hifitime`) — no Meeus,
no live JPL Horizons fetch, no calibration loop.

---

## Invariants

These are non-negotiable; an edit that breaks them needs a
report, not a pull request.

1. **DE440 is the ground truth.** No hand-rolled Meeus
   approximations, no live Horizons fetch as a calibration
   step. The vendored DE440 ephemeris is the canonical
   source for solar position (and, in Phase 3, for moon and
   planet positions). `anise` reads it; chronos consumes
   `anise`.

2. **rkyv on the wire, NOTA at the human boundary.** Daemon
   ↔ CLI / chroma is the signal pattern (length-prefixed
   rkyv frames over UDS). NOTA appears only on the CLI argv
   and the printed reply. The daemon never re-parses NOTA
   from a request frame.

3. **Push, not poll.** Event fires are
   `tokio::time::sleep_until` deadlines (timerfd-backed).
   Geoclue is a `zbus` signal subscription. There is no
   loop-and-check anywhere. The carve-outs in
   `~/primary/skills/push-not-pull.md` are the only
   exceptions.

4. **One object in, one object out — and the object is a
   typed record.** Methods on `Sky`, `Observer`, etc. take
   one explicit object alongside `self` and return one
   object. No anonymous tuples at type boundaries.

5. **The runtime root is the only raw `spawn` site.** Every other
   actor is supervised from its parent with Kameo's
   `supervise(&parent, ...).spawn().await` shape. Failures escalate.

6. **Type names don't carry the crate name.** Use
   `chronos::Request`, not `chronos::ChronosRequest`. The
   namespace is the crate; the type name is the role.

7. **Subscribers receive current state on connect.** Per
   `~/primary/skills/push-not-pull.md` §"Subscription
   contract" — `Subscribe` always returns at least one
   frame describing the current schedule before any
   future-event deltas.

---

## What this repo does NOT own

- The `geoclue2` service. Subscribed, not embedded.
- The DE440 file's distribution. `anise` + flake-input
  handle vendoring; chronos is read-only on it.
- Chroma's schedule engine. Chronos *publishes* twilight
  events; chroma's `EventScheduler` subscribes and reacts.
- Planetary positions beyond the sun (Phase 1). Moon,
  ascendant, midheaven, houses, planetary aspects are
  Phase 3 work behind Swiss Ephemeris.

If a change touches one of these, it goes upstream
(`anise`, `geoclue`, chroma, `swisseph`), not into chronos.

---

## How to work in this repo

- **Domain types first, actors last.** Land
  `ZodiacSign`, `EclipticLongitude`, `Location`,
  `SolarEventKind`, `ZodiacalTime`, `AmYear` (with tests)
  before wiring the actors that compute them.
- **Astronomical bodies are stubbed in the skeleton.** The
  `Sky::zodiacal_time_at`, `Sky::next_civil_dawn`, etc.
  bodies are `todo!()` until the implementation phase.
  The type signatures are the design.
- **Tests in `tests/`, not in `#[cfg(test)] mod tests`.**
  Per `~/primary/skills/rust-discipline.md` §"Tests live
  in separate files".
- **Use existing trait domains.** `FromStr` over inherent
  `parse`. `Display` over inherent `to_string`. `From` /
  `TryFrom` for conversions. `AsRef<str>` for newtype peeks.
- **`nix flake check` is the canonical pre-commit runner.**
  Per `~/primary/skills/nix-discipline.md`.
- **Push per logical commit.** Per `~/primary/skills/jj.md`.

---

## See also

- `ARCHITECTURE.md` — what the system IS.
- `AGENTS.md` — agent contract for this repo.
- `~/primary/skills/rust-discipline.md` — Rust style and
  shape; methods on types, domain newtypes, errors, redb + rkyv.
- `~/primary/skills/actor-systems.md` — actor topology discipline.
- `~/primary/skills/kameo.md` — Kameo actor runtime discipline.
- `~/primary/skills/push-not-pull.md` — subscription
  discipline.
- `~/primary/skills/abstractions.md` — verb belongs to
  noun.
- `~/primary/skills/micro-components.md` — one capability
  per crate per repo.
- `~/primary/skills/jj.md` — version-control discipline.
- `~/primary/skills/nix-discipline.md` — flake hygiene.
- `~/primary/repos/chroma` — sibling daemon; subscribes
  to chronos's twilight events.
- `~/primary/repos/signal` — canonical signal pattern.
- `~/primary/repos/lojix-cli` — canonical NOTA-on-argv CLI.
- `~/primary/repos/lore/rust/rkyv.md` — wire-format
  discipline, feature pinning.
