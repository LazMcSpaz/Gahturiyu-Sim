# Gahturiyu Sim

A Kenshi-style world for Gahturiyu, built simulation-first. Right now it is a
21 × 21 km coast with 30 towns and 5,000 people who travel, visit and wander
whether or not you are watching — plus a plain window to watch them in.

Your squad of four can fight, sneak, pick locks, go indoors, gather and craft,
talk to people and take on work. Bandit camps by the roads ambush travellers
across the whole map whether you're there or not. The foundation underneath:
**the far-away world runs cheaply, the nearby world runs in full detail, and
nothing that happens depends on how closely it was being watched.**

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

The window opens in **3D**. Press **V** to flip to the top-down map and back.

| | |
|---|---|
| Left-click ground | Send the selected squad members there (everyone, if none selected) |
| Left-click a squad member, or F1–F4 | Select them (hold Shift to add or remove) |
| ` or Esc | Select everyone again (Esc also leaves a conversation) |
| Left-click a bandit | Attack (a sneak attack if they haven't noticed you) |
| Left-click anyone else | Walk over and talk to them |
| Left-click something on the ground | Pick it up |
| Left-click a door | Go in, or pick the lock if it's locked (needs a lockpick) |
| Left-click a plant, rock or log | Gather it |
| Z | Selected members sneak / stop sneaking |
| I, or right-click a squad card | Pack and gear |
| K | Crafting |
| J | Journal (jobs) |
| Right-drag, Q / E | Turn the camera (map: pan) |
| Middle-drag, or WASD | Pan |
| Mouse wheel | Zoom |
| C | Snap back to following the squad |
| Space | Pause |
| 1 – 5 | Speed: real time, 10×, 1 minute/s, 10 minutes/s, 1 hour/s |
| R | Show / hide the band rings |
| B | (testing) Drop a band of bandits next to the squad |

Hover over anyone or anything for details. A fight breaking out near you drops
the speed to real time.

In the **pack** panel: click something worn to take it off, click something in
the pack to put it on (or drink it, for a potion), right-click to drop it.

## Playing

- **Your squad**: a Roduro brawler with a stone maul, a Horaro hunter with a
  spear and lockpicks, a Qotiro shield-fighter, and a Ṭaḍoro mage (paralyze,
  fireball, lightning, blind, mage armour, haste, heal) with a mortar and
  pestle and some herbs. Each walks at their own pace: hills, a hurt leg or an
  overloaded pack all slow them down.
- **Fights** are Kenshi-style: six body parts, each with its own health;
  cuts and blunt blows; armour that covers some parts some of the time. A
  ruined arm drops the weapon, a ruined leg slows you, a head or torso at zero
  knocks you out, and only a much worse beating kills. Wounds heal on their
  own over hours. Hits, dodges, blocks and spells train the skills used,
  Morrowind-style. Hurt fighters drink healing draughts if they have them.
- **Magic** costs mana, takes a moment to cast and can fizzle (the mana is
  still spent). Scrolls cast once with no mana and never fizzle.
- **Bandit camps** sit beside the roads, lit by a campfire. Their lookouts
  notice you by sight (worse in the dark, against someone sneaking) and by
  sound (louder when moving, fighting or in heavy armour). Once they notice,
  they attack. They also attack travellers they think they can beat — all
  across the map.
- **Doors** lock from 20:00 to 06:00. A picked lock stays open until the next
  night. Inside, the walls and roof are cut away; there are things to take,
  but they belong to someone. Being seen picking a lock or stealing earns a
  bounty in that town (pay it off by talking to a local).
- **Crafting**: potions (anywhere with a mortar and pestle), scrolls (scribe's
  desk), weapons and armour (forge, armourer's bench). Every town has the four
  stations round its hearth. Materials grow or lie around the land and come
  back a day after you take them.
- **Talking**: people answer from who they are and what's true right now. Some
  have work: break a bandit camp, fetch materials, carry a letter.

## What you are looking at

- **Colours are races.** Stone = Roduro, ember = Qotiro, sea blue = Horaro,
  pale violet = Ṭaḍoro.
- **Towns** follow `architecture.md`: Roduro grow rounded, banded homes of dark
  stone with a lit window; Qotiro towns are stepped sandstone blocks around a
  three-tier temple; Qotiro living elsewhere keep one small dark-stone hall with
  a gold crown; every coastal town has a Horaro stilt village just offshore —
  woven domes on stone pillars with timber decks. Every town has a hearth at its
  centre. Ṭaḍoro build nothing; a resting wanderer pitches a tent.
- **The rings around your squad are the bands.**
  - Inside the inner ring (500 m) is **band 1**: everyone is a person with a
    name, temperament and gear. This is decided **person by person**, by where
    each one stands, so the ring cuts through a town rather than switching the
    whole town on or off.
  - Between the rings (out to 2.5 km) is **band 2**: travelling groups are one
    marker each, updated every few game seconds.
  - Beyond is **band 3**: groups updated once a game-minute.
- In 3D, people are true size up close and drawn larger as you zoom out, so you
  can still pick them out. Buildings always stay true size.
- The panel's **"Named so far"** count only goes up when something comes close.

### The land

- **The coast** runs down the west side, mostly low beaches with some stretches
  of low sea cliff. Harbour towns only stand where there's a decent beach.
- **Rolling hills** rise slowly inland.
- **Mountains** wall in the north and east edges, with one big massif in the
  middle of the map.
- **The Qotiro plateau** in the south-east is a flat-topped, cliff-edged
  tableland of dry scrub — their homeland.
- **Roads** link every town to its nearest neighbours. They were found by
  searching the land for the easiest walk, so they follow valleys, climb
  escarpments where the slope eases, and go around mountains. Travellers use
  them, and walk slower uphill and a little faster downhill (Tobler's hiking
  rule), so the slope shapes how long every journey takes.
- Your squad also slows on climbs. The sea is off-limits.

In 3D, the land within a few kilometres is detailed and the rest is coarser
out to the horizon. The map view shows the whole world as shaded relief.

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

**Fights far away are the same fights.** When a traveller's route is planned
(a couple of hours ahead), it's checked against every bandit camp: the exact
moment the road brings them into sight is worked out like a line crossing a
circle. Bandits weigh their combined might against the travellers' and decide
whether to attack. If they do, the fight is run blow by blow on the same rules
as your own fights, there and then — it takes well under a millisecond. Nearby,
you watch a copy of it play out in real time; the outcome was already fixed.
Survivors wait until they can stand, then head home. So walking closer never
changes how a fight went.

## Proving it

```
cargo test --release
```

The important tests are in `tests/consistency.rs`. They run the same world
several ways — one-second steps vs one-hour steps, squad here vs squad in the
far corner, squad watching a roadside ambush vs far away — and check that every
journey, route, position, wound, death and ambush comes out identical. They
also check that details, once built, never change. The other test files check
each system does what it says: `combat.rs`, `gear.rs` (every enchantment),
`squad.rs`, `stealth.rs`, `indoors.rs`, `crafting.rs`, `talk.rs`, `terrain.rs`.

```
cargo run --release --bin headless -- 3
```

runs three game days with no window and prints what the world is doing and how
long it took (a few seconds, stepping one game second at a time).
`headless fight 3` prints a squad-vs-bandits fight blow by blow.

## Dials

The numbers most worth tuning, all named constants:

| What | Where |
|---|---|
| Population, town counts, who becomes a wanderer | `src/sim/worldgen.rs` |
| Which races favour which towns | `affinity()` in `src/sim/worldgen.rs` |
| Race temperaments, build, walking speed | `src/sim/race.rs` |
| How busy the roads are, day vs night | `DEPARTURE_RATE`, `NIGHT_FACTOR` in `src/sim/world.rs` |
| Band sizes and update rates | `src/sim/bands.rs` |
| Where mountains, the plateau and cliffs are; how tall | `src/sim/terrain.rs` |
| How roads are chosen (steepness limit, how many links per town) | `src/sim/routes.rs` |
| Ground colours | `src/view/palette.rs` |
| Stats, skills, how fast they train | `src/sim/stats.rs` |
| Weapons, armour, enchantments, materials, potions, scrolls | `src/sim/items.rs` |
| Spells | `src/sim/magic.rs` |
| Hit chances, damage, sneak attacks | `src/sim/combat.rs` |
| Bandit camps: how many, how far they see, rest between attacks | `src/sim/encounters.rs` |
| Sight, hearing, light, sneaking | `src/sim/stealth.rs` |
| Locks, lockpicking, what's inside homes | `src/sim/buildings.rs` |
| Recipes, stations, gathering | `src/sim/crafting.rs` |
| Jobs and dialogue | `src/sim/quests.rs`, `src/sim/dialogue.rs` |
| Name sounds per race | `src/sim/names.rs` |

## Layout

```
src/sim/      the simulation — no graphics, fully testable
  world.rs      the world and its step loop
  worldgen.rs   building a world from a seed
  group.rs      travelling groups and their schedules
  person.rs     people: cheap summary + lazy details
  bands.rs      band assignment by distance from the squad
  race.rs       the four races
  names.rs      per-race name generators
  geo.rs        positions and the coastline
  terrain.rs    the height of the land, and walking speed on slopes
  routes.rs     the road network and every town-to-town route
  rng.rs        deterministic randomness
  stats.rs      attributes, skills, callings
  body.rs       body parts and wounds that heal
  items.rs      the item catalogue
  inventory.rs  gear slots, packs, weight
  magic.rs      spells
  combat.rs     a fight, tick by tick
  ai.rs         what fighters decide
  fights.rs     where fights meet the world (the squad's fights)
  encounters.rs bandit camps and ambushes on the world's timeline
  squad.rs      squad members, walking, picking things up
  stealth.rs    being seen and heard
  buildings.rs  doors, locks, interiors
  crafting.rs   recipes, stations, gathering, potions
  quests.rs     jobs
  dialogue.rs   conversations
src/main.rs   the playtest window: input, timing, switching views
src/view/     drawing only — never changes the world's rules
  scene.rs      the 3D view
  mesh.rs       mesh building with baked-in lighting
  palette.rs    ground colours, shared by both views
  map.rs        the top-down map
  ui.rs         panel, tooltips, colours
  squadui.rs    squad cards, pack, crafting, conversation, journal
tests/        the consistency checks
```
