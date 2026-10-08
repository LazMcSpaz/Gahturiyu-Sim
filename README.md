# Gahturiyu Sim

A Kenshi-style world for Gahturiyu, built simulation-first. Right now it is a
21 × 21 km coast with 30 towns and 5,000 people who travel, visit and wander
whether or not you are watching — plus a plain window to watch them in.

There is no combat, trade or player interaction yet. This first slice proves the
foundation: **the far-away world runs cheaply, the nearby world runs in full
detail, and nothing changes or blinks when things cross between the two.**

## Running it

One-time setup on Windows:

1. Install Rust from <https://rustup.rs>. If it asks to install the Visual Studio
   C++ build tools, say yes.
2. Clone this repo (GitHub Desktop is fine).

Then, in a terminal inside the repo folder:

```
cargo run --release
```

The first build downloads and compiles a few libraries and takes a minute or
two. After that it starts in seconds. `cargo run --release -- 42` builds a
different world from seed 42.

### Controls

| | |
|---|---|
| Left-click | Send your squad there |
| Right-drag, or WASD | Pan the map |
| Mouse wheel | Zoom |
| C | Snap back to following the squad |
| Space | Pause |
| 1 – 5 | Speed: real time, 10×, 1 minute/s, 10 minutes/s, 1 hour/s |
| R | Show / hide the band rings |

Hover over anyone or anything for details.

## What you are looking at

- **Colours are races.** Stone = Roduro, ember = Qotiro, sea blue = Horaro,
  pale violet = Ṭaḍoro.
- **Towns** are the large circles, tinted by who founded them. Every town is
  mixed. Coastal towns have a Horaro stilt community just offshore (the blue
  patch in the water).
- **The rings around your squad are the bands.**
  - Inside the inner ring (500 m) is **band 1**: everyone is drawn as a person
    with a name, temperament and gear.
  - Between the rings (out to 2.5 km) is **band 2**: travelling groups are one
    dot each, updated every few game seconds.
  - Beyond is **band 3**: groups are small faint dots, updated once a game-minute.
- The panel's **"Named so far"** count only goes up when something comes close.
  Walk somewhere new and watch it climb. Hover a far-off group and it will say
  nobody in it has been named yet.

## How it works, in plain words

**Everyone exists, but cheaply.** Each person is a seed, a race, four temperament
traits and one cached "might" score. That is all the far-away world reads.

**Details appear only when needed, and then stay.** A name and gear are built
from the seed the first time a person comes within band 1, chosen so they add up
to the might score the world was already using. Once built they are kept forever.
Someone you have met never comes back as a stranger.

**Races set the average, not the person.** Each trait is drawn around the race's
average with a wide spread. Most Roduro are homebodies; some are restless and
travel far. Most Ṭaḍoro roam; some settle in a town.

**Journeys are schedules.** When a group sets out, its whole trip is written
down: leave at 09:12, reach the next town at 11:40, stay until evening, walk
home. Its position at any moment is just looked up from the schedule. A group
in band 3 is checked once a minute and a group beside you sixty times a second,
and both are in exactly the right place. This is the same rule as the old NPC
routine plan — read the clock, set the state.

**Randomness is keyed, not rolled.** "Does anyone leave this town this hour?" is
answered from the world seed plus that town plus that hour. So the answer is the
same no matter how coarsely the world was being stepped or where your squad was.

## Proving it

```
cargo test --release
```

The important tests are in `tests/consistency.rs`. They run the same world
several ways — one-second steps vs one-hour steps, squad here vs squad in the
far corner — and check that every journey, route and position comes out
identical. They also check that details, once built, never change.

```
cargo run --release --bin headless -- 3
```

runs three game days with no window and prints what the world is doing and how
long it took. Three days takes under a second.

## Dials

The numbers most worth tuning, all named constants:

| What | Where |
|---|---|
| Population, town counts, who becomes a wanderer | `src/sim/worldgen.rs` |
| Which races favour which towns | `affinity()` in `src/sim/worldgen.rs` |
| Race temperaments, build, walking speed | `src/sim/race.rs` |
| How busy the roads are, day vs night | `DEPARTURE_RATE`, `NIGHT_FACTOR` in `src/sim/world.rs` |
| Band sizes and update rates | `src/sim/bands.rs` |
| Name sounds per race | `src/sim/names.rs` |

## Layout

```
src/sim/      the simulation — no graphics, fully testable
  world.rs      the world and its step loop
  worldgen.rs   building a world from a seed
  group.rs      travelling groups and their schedules
  person.rs     people: cheap summary + lazy details
  bands.rs      chunk grid and band assignment
  race.rs       the four races
  names.rs      per-race name generators
  geo.rs        positions and the coastline
  rng.rs        deterministic randomness
src/main.rs   the playtest window (reads the sim, never changes its rules)
tests/        the consistency checks
```
