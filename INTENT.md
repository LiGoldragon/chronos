# INTENT — chronos

*What the psyche has explicitly intended for this project.
Synthesised from psyche statements and applicable workspace
constraints; not embellished. `ARCHITECTURE.md` says what chronos
IS; this file says what the psyche wants it to BE.*

## Purpose

`chronos` is one Rust user-service daemon that owns the **local
sky**: which zodiacal time it is, where the sun is, and when the
next twilight fires. It answers one-shot queries and publishes
typed `SolarEvent`s to subscribers (chroma today; other consumers
later) through the canonical signal pattern. It is the publisher;
chroma is a subscriber.

## Constraints

- **Astronomical truth comes from vendored data, not a
  calibration loop.** The JPL DE440 ephemeris (read by `anise`),
  NREL SPA solar position (`solar-positioning`), and `hifitime`
  for UTC↔TT↔TDB are the source of truth — no Meeus, no live JPL
  Horizons fetch, no calibration loop.
- **NOTA is the only text format.** The CLI argument and the
  printed reply are NOTA; JSON and `serde` appear nowhere. Per the
  workspace NOTA discipline (`primary/ESSENCE.md`).
- **The CLI takes exactly one NOTA record on argv and signals the
  daemon.** `chronos 'GetTime'`, `chronos '(SetLocation 47.6
  -122.3)'`. The CLI is a thin signal client for one-shot verbs;
  consumers like chroma hold the connection open for the event
  stream. Per the single-argument rule
  (`primary/skills/component-triad.md`).
- **Daemon ↔ client is the canonical signal pattern.** Unix domain
  socket, 4-byte big-endian length prefix, then the rkyv archive;
  closed `Request` / `Response` enums. A `Subscribe` request opens
  a long-lived reply stream; each reply is one `Event` frame. The
  version-skew guard hard-fails at boot on schema mismatch.
- **Push, not poll.** The producer pushes events as they fire
  (timerfd-backed deadlines); the consumer waits. There is no
  polling loop. Per `primary/skills/push-not-pull.md`.
- **State is actor-owned.** Each actor is a data-bearing noun with
  request-specific message types; raw `spawn` belongs only at the
  runtime root. Per `primary/skills/actor-systems.md`.

## Anti-patterns — explicitly not to do

- **No event replay for late-joining subscribers.** Pushed events
  are ephemeral; a subscriber that misses an event does not get a
  replay. Subscribers receive the current state on connect, then
  the live push stream.

## Scope — today, not eventually

chronos Phase 1 owns the sun: civil dawn, sunrise, solar noon,
sunset, civil dusk, plus zodiacal time. Moon, ascendant,
midheaven, houses, and planetary aspects are a later phase (Swiss
Ephemeris); cross-machine event push is later still (Persona
fabric). Built rightly for the scope it serves today. Per
`primary/ESSENCE.md` §"Today and eventually".

*Source statements live in Spirit intent records and the
project's `ARCHITECTURE.md`. Workspace-shape intent stays in
`primary/INTENT.md` and the named skills above.*
