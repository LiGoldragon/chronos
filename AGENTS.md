# Agent instructions — chronos

## Repo role

Chronos is the **time-and-sky daemon**: a long-running user
service that publishes the current zodiacal time, sunrise /
sunset, civil twilight, and other ordinal-solar events for the
local observer. Chroma's schedule engine subscribes to chronos's
twilight events and reacts; humans query chronos via DOTOS-on-argv
(`chronos 'GetTime'`) for the current zodiacal time.

## Carve-outs worth knowing

- **Push, not poll.** Chronos pushes events at deadlines
  (`tokio::time::sleep_until`, timerfd-backed). Subscribers
  receive the current state on connect, then deltas at each
  event fire. The daemon never wakes on a clock to check
  "did anything change?" See the push-not-pull discipline.
- **Astronomy via SPICE-validated crates.** `anise` (Nyx Space)
  reads JPL DE440 directly and is validated to machine
  precision against SPICE. `hifitime` carries the time scales.
  `solar-positioning` runs NREL SPA for sunrise / sunset /
  twilight at any solar elevation angle. The prototype's
  Meeus + JPL Horizons calibration loop is **not** carried
  forward — DE440 *is* the ground truth.
- **rkyv on the wire, DOTOS at the human boundary.** Daemon ↔
  CLI / chroma is the canonical signal pattern (length-prefixed
  rkyv frames over UDS). DOTOS appears only on the CLI argv,
  the one disk record (`LocationSource` override), and the
  printed reply.
- **Group-gated UDS.** `/run/chronos/<uid>.sock`, `chronos`
  group at the directory level (mode 0770). Same shape as
  chroma. CriomOS owns the group + tmpfiles entry.
- **One persisted value.** Only `LocationSource` (manual
  override vs geoclue) is stored across restarts. Ephemeris
  data is read from a vendored DE440 file, not a live cache.
- **Subscription is a long-lived connection.** A subscriber
  invocation (or chroma) opens the UDS, sends one `Subscribe`
  request, and reads pushed events forever. The daemon does
  not write to a log; it pushes typed events to live consumers.

## Style

Per the Rust discipline:

- Methods on types, not free functions.
- Domain values are typed (newtypes; private fields).
- One object in, one object out at boundaries.
- Errors as a typed `Error` enum per crate via `thiserror`.
- Tests live in `tests/`, one file per module exercised.
- Full English words for identifiers (per
  the naming discipline).
- Type names do not carry the crate name (`Request`, not
  `ChronosRequest`). The crate name in `chronos::Request`
  is the namespace; the type name is the role.

Beauty is the criterion (per the design-quality discipline):
ugliness is a diagnostic reading; slow down and find the
structure that makes it beautiful.

## Version control

`jj` (Jujutsu), per the Jujutsu discipline. Standard flow:

```sh
jj commit -m '<short verb + scope>' \
  && jj bookmark set main -r @- \
  && jj git push --bookmark main
```

Push per logical commit; blanket authorisation. No editor
prompts (always `-m '<msg>'`).

## See also

- `chroma` — sibling daemon; subscribes to
  chronos's twilight events.
- the Rust discipline — Rust style and shape.
- the push-not-pull discipline — subscription discipline.
- the abstractions discipline — verb-belongs-to-noun.
- the design-quality discipline — beauty as criterion.
- the actor-system discipline — actor topology discipline.
- the Kameo discipline — Kameo actor runtime discipline.
- `lore/rust/rkyv.md` — wire format discipline.
- the `signal` repository — canonical signal pattern reference.
- the `lojix` repository — typed DOTOS client shape.
