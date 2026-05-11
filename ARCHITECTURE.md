# ARCHITECTURE — chronos

Chronos is a single user-service Rust daemon that owns the
**local sky**: which zodiacal time it is, where the sun is,
when the next twilight fires. It publishes typed events to
subscribers and answers one-shot queries through the canonical
signal pattern.

The full design lives in
`~/primary/reports/system-specialist/49-chronos-daemon-design.md`.
This file describes the system shape that ships; the report
describes trade-offs and the library research grounding the
choices.

## Capability boundary

Chronos owns:

- the observer's typed `Location` (latitude/longitude with a
  `LocationSource` — geoclue or manual override)
- the JPL **DE440** ephemeris (vendored; read by `anise`)
- the **time pipeline** — UTC ↔ TT ↔ TDB via `hifitime`, sun
  position via NREL SPA (`solar-positioning`), zodiacal
  projection via DE440 ecliptic longitude
- the typed `SolarEvent` set (`CivilDawn`, `Sunrise`,
  `SolarNoon`, `Sunset`, `CivilDusk`; nautical/astronomical
  twilights are an extension)
- the **subscription primitive** — long-lived UDS connections
  that receive the current state on connect, then push events
  as they fire (timerfd-backed deadlines, no polling)
- the typed CLI request grammar (`Request` / `Response`)
- the persisted `LocationSource` override (one redb row)

Chronos does **not** own:

- chroma's schedule engine — chronos *publishes*; chroma
  *subscribes*
- the geolocation source — `geoclue2` is the upstream signal;
  chronos subscribes via `zbus`
- planetary positions beyond the sun — moon, ascendant,
  midheaven, houses are Phase 3 (Swiss Ephemeris)
- audit logging — pushed events are ephemeral; subscribers
  who miss events do not get replay

## The three layers of state

| Layer | Where | What |
|---|---|---|
| Static | DE440 file (`/usr/share/chronos/de440s.bsp` or vendored) | Ephemeris bytes; never written |
| Persistent | `$XDG_STATE_HOME/chronos/state.redb` | One row: `LocationSource` (manual override or `Geoclue`) |
| Live | In-process | The `Sky` (loaded ephemeris), the `Observer` (sky + location), the active subscriber set, the next-event deadlines |

Most state is *live*. The daemon recomputes from DE440 + the
current location; the only durable bit is whether the user
overrode location.

## Actor topology (skeleton)

```
Supervisor
├── StateStore                       (redb handle; one row: LocationSource)
├── EphemerisLoader                  (loads DE440 once; produces a Sky)
├── LocationTracker                  (zbus to geoclue2 OR persisted override)
├── EventScheduler                   (next-fire deadline per kind; timerfd)
│   └── …pushes SolarEvent to SubscriptionHub
├── SubscriptionHub                  (live subscribers + their cursors)
└── SocketServer                     (UDS at /run/chronos/<uid>.sock)
```

Per workspace Kameo discipline: each actor is a data-bearing noun,
message types are specific to their requests, state is owned rather
than shared, failures escalate, and raw `spawn` belongs only at the
runtime root. Children spawn through supervised parent builders.

## IPC shape — the canonical signal pattern

Daemon ↔ CLI / chroma is the **signal pattern** documented in
`~/primary/repos/signal`:

- **Transport:** Unix domain socket at
  `/run/chronos/<uid>.sock`. CriomOS owns the directory + the
  `chronos` group at mode 0770 (same shape as chroma).
- **Framing:** 4-byte big-endian length, then the rkyv archive.
- **Request:** `Request` enum (one variant per CLI verb).
- **Reply:** `Response` enum (`Time`, `Location`, `Schedule`,
  `Event`, `Acked`, `Error`).
- **Subscription:** a `Subscribe` request opens a long-lived
  reply stream; each reply is one `Event` frame.

The CLI is a thin signal client for one-shot verbs; chroma
opens the same socket but holds the connection open for the
event stream.

## Configuration

There is no NOTA config file. The two configurable values are:

- **Location** — set via `chronos '(SetLocation <lat> <lon>)'`
  (persisted) or auto-detected via `geoclue2` (default).
- **Ephemeris path** — defaults to `de440s.bsp` shipped with
  the package; overridable via `CHRONOS_EPHEMERIS=<path>`.

Most chronos behaviour is observer-driven: location +
current time + DE440 ⇒ everything else.

## Persistence

`$XDG_STATE_HOME/chronos/state.redb` — one redb file. Tables:

| Table | Key | Value |
|---|---|---|
| `location` | fixed slot `source` | rkyv archive of `LocationSource` |
| `meta` | fixed slot `version` | `(schema_version, wire_version)` |

The version-skew guard at boot hard-fails on schema mismatch.

## Boundary contracts

| Boundary | Format |
|---|---|
| In-process: actor ↔ actor | typed Rust values |
| Daemon ↔ CLI / chroma | rkyv-archived `Request` / `Response`, length-prefixed |
| Daemon ↔ disk (state) | rkyv values inside redb tables |
| Daemon ↔ disk (ephemeris) | DE440 SPK bytes (read-only, `anise`) |
| Daemon ↔ geoclue2 | `zbus` signal subscription |
| Daemon ↔ human (audit) | NOTA reply printed by the CLI |

JSON / `serde` appears nowhere. The only text format is
NOTA (CLI argv + printed reply).

## Out of scope (Phase 1)

- Moon, ascendant, midheaven, houses, planetary aspects
  (Phase 3 — Swiss Ephemeris)
- Per-observer ascendant tracking (Phase 3)
- Cross-machine event push (Phase 4 — Persona fabric)
- Astronomical twilight beyond civil + nautical (Phase 1.1)
- Event replay for late-joining subscribers (out — pushed
  events are ephemeral)

See the design report for the rationale behind each.
