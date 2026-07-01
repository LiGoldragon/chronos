# Agent instructions — chronos

You **MUST** read `~/primary/AGENTS.md` and
`~/primary/repos/lore/AGENTS.md` — the canonical workspace
agent contract.

## Repo role

Chronos is the **time-and-sky daemon**: a long-running user
service that publishes the current zodiacal time, sunrise /
sunset, civil twilight, and other ordinal-solar events for the
local observer. Chroma's schedule engine subscribes to chronos's
twilight events and reacts; humans query chronos via NOTA-on-argv
(`chronos 'GetTime'`) for the current zodiacal time.

## Carve-outs worth knowing

- **Push, not poll.** Chronos pushes events at deadlines
  (`tokio::time::sleep_until`, timerfd-backed). Subscribers
  receive the current state on connect, then deltas at each
  event fire. The daemon never wakes on a clock to check
  "did anything change?" See `~/primary/skills/push-not-pull.md`.
- **Astronomy via SPICE-validated crates.** `anise` (Nyx Space)
  reads JPL DE440 directly and is validated to machine
  precision against SPICE. `hifitime` carries the time scales.
  `solar-positioning` runs NREL SPA for sunrise / sunset /
  twilight at any solar elevation angle. The prototype's
  Meeus + JPL Horizons calibration loop is **not** carried
  forward — DE440 *is* the ground truth.
- **rkyv on the wire, NOTA at the human boundary.** Daemon ↔
  CLI / chroma is the canonical signal pattern (length-prefixed
  rkyv frames over UDS). NOTA appears only on the CLI argv,
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

Per `~/primary/skills/rust-discipline.md`:

- Methods on types, not free functions.
- Domain values are typed (newtypes; private fields).
- One object in, one object out at boundaries.
- Errors as a typed `Error` enum per crate via `thiserror`.
- Tests live in `tests/`, one file per module exercised.
- Full English words for identifiers (per
  `~/primary/skills/naming.md`).
- Type names do not carry the crate name (`Request`, not
  `ChronosRequest`). The crate name in `chronos::Request`
  is the namespace; the type name is the role.

Beauty is the criterion (per `~/primary/skills/beauty.md`):
ugliness is a diagnostic reading; slow down and find the
structure that makes it beautiful.

## Version control

`jj` (Jujutsu), per `~/primary/skills/jj.md`. Standard flow:

```sh
jj commit -m '<short verb + scope>' \
  && jj bookmark set main -r @- \
  && jj git push --bookmark main
```

Push per logical commit; blanket authorisation. No editor
prompts (always `-m '<msg>'`).

## See also

- `~/primary/AGENTS.md` — workspace agent contract.
- `~/primary/repos/lore/AGENTS.md` — canonical (cross-workspace)
  agent contract.
- `~/primary/repos/chroma` — sibling daemon; subscribes to
  chronos's twilight events.
- `~/primary/skills/rust-discipline.md` — Rust style and shape.
- `~/primary/skills/push-not-pull.md` — subscription discipline.
- `~/primary/skills/abstractions.md` — verb-belongs-to-noun.
- `~/primary/skills/beauty.md` — beauty as criterion.
- `~/primary/skills/actor-systems.md` — actor topology discipline.
- `~/primary/skills/kameo.md` — Kameo actor runtime discipline.
- `~/primary/repos/lore/rust/rkyv.md` — wire format discipline.
- `~/primary/repos/signal` — canonical signal pattern reference.
- `~/primary/repos/lojix` — typed NOTA client shape.
