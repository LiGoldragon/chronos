# chronos

A user-service daemon publishing **zodiacal time, sunrise / sunset,
and civil twilight events** for the local observer. Backed by
JPL DE440 (via [`anise`](https://crates.io/crates/anise)), NREL SPA
(via [`solar-positioning`](https://crates.io/crates/solar-positioning)),
and [`hifitime`](https://crates.io/crates/hifitime) — no Meeus,
no calibration loop, no live JPL Horizons fetch.

## What chronos answers

- *What zodiacal time is it?* — the sun's apparent ecliptic
  longitude projected onto the twelve-sign zodiac, rendered
  in the prototype's five output formats (`am`, `unicode`,
  `version`, `numeric`, `json`).
- *When is the next civil dawn / sunrise / sunset / civil dusk?*
  — at the configured location, accurate to NREL SPA's
  ±0.0003° / ~30 s on twilight times.
- *What's the AM-calendar position right now?* — vernal-equinox-
  anchored year + ordinal solar time.

## Two surfaces

**One-shot CLI** for humans:

```sh
chronos 'GetTime'                    # current zodiacal time
chronos 'GetSchedule'                # today's sunrise/sunset/twilights
chronos '(SetLocation 47.6 -122.3)'  # manual location override
```

**Long-lived subscription** for chroma + future consumers:
open the UDS, send

```dotos
Subscribe.([CivilDawn Sunrise Sunset CivilDusk])
```

and receive each event as it fires. The producer pushes;
the consumer waits. Per `~/primary/skills/push-not-pull.md`,
no polling.

## Status

Phase 1 skeleton — types declared, daemon structure laid out,
astronomical bodies behind `todo!()`. Implementation lands
behind the agent contract in [`AGENTS.md`](AGENTS.md). The
agent contract, system shape, and project intent live in
[`AGENTS.md`](AGENTS.md), [`ARCHITECTURE.md`](ARCHITECTURE.md),
and [`skills.md`](skills.md) respectively.
