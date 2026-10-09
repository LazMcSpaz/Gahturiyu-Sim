# Animals — notes for the merge

Written by the second session (branch `agent2/animals`). `README.md` and
`CLAUDE.md` were off-limits to this branch, so the sections that belong in
them are below, ready to paste. Delete this file once they're in.

Contents: [for whoever merges](#for-whoever-merges) ·
[hooks for society](#hooks-for-the-society-side) ·
[README section](#readme-section-paste-under-magic-or-society) ·
[CLAUDE.md section](#claudemd-additions) ·
[what to tune first](#what-to-tune-first) · [known limits](#known-limits)

---

## For whoever merges

### Shared files touched (every line)

| File | Line added |
|---|---|
| `src/sim/mod.rs` | `pub mod animals;` |
| `src/sim/world.rs` | field in `World`: `pub animals: super::animals::Animals,` |
| `src/sim/world.rs` | in `World::assemble`: `animals: Default::default(),` |
| `src/sim/world.rs` | first line of the event loop in `World::step`: `if self.animal_events(hour_t) { continue; }` |
| `src/sim/worldgen.rs` | after `w.place_crafting();`: `w.place_animals();` |
| `src/view/mod.rs` | `pub mod animals;` |
| `src/view/scene.rs` | `super::animals::draw(w, &mut b, &mut gl, &on_ground, oc.target, radius, k, eye);` |
| `src/view/map.rs` | `super::animals::draw_map(c, cam, w);` |
| `src/view/app.rs` | `super::animals::overlay(&c, game, &scene, &mut panels);` (hover, panel, debug keys) |
| `src/view/shot.rs` | end of `Shot::prepare`: `super::animals::prepare(world);` (the `GAHT_ANIMALS` flag) |

Ten lines in eight files. Two are beyond the six the ground rules named
(`app.rs` and `shot.rs`; neither is on the don't-edit list): the hover and
panel need a place in the window's panel pass, and the screenshot flag needs
a place in the screenshot set-up. `world.rs` needed a second line because a
new field has to be given a starting value.

Everything else is new files: `src/sim/animals.rs`, `src/sim/animals/*.rs`
(`species`, `region`, `herd`, `attack`, `livestock`, `hooks`, `place`),
`src/view/animals.rs`, `tests/animals.rs`.

### To do at merge

- **Needs FORMAT bump.** `World::animals` is new saved state (it derives
  `Serialize`/`Deserialize` throughout). `save::FORMAT` was left alone as
  instructed. Old saves won't load.
- **`tests/save.rs` `fingerprint`** doesn't include the animals (that file
  wasn't mine to edit). Suggested line: `s += &format!("{:?}", w.animals);`.
  Until then, save/load of animals is covered in `tests/animals.rs`
  (an ordinary moment, mid-stalk and mid-fight).
- Paste the two sections below into `README.md` and `CLAUDE.md`.

### What I'd want from `combat.rs` / `fights.rs` / `scene.rs`

Animals fight under the ordinary rules with no change to them. Where the
rules don't fit an animal, the difference is applied from outside, in
`animals/attack.rs::advance_fray`. Each of these would be cleaner inside:

1. **A fighter kind for beasts.** An animal is built with
   `Fighter::creature(Summon::Swarmling, …)` and then given its own numbers,
   because that is the only constructor for a fighter that isn't a person.
   The borrowed kind leaks: `scene.rs` draws every `summon` fighter as a
   glowing dome with a ring and a health bar, so **an animal in a fight is
   drawn twice** (its own shape from `view/animals.rs`, plus the spirit-beast
   glow). Wanted: a `Summon::Beast` (or a `beast: bool`) that `scene.rs`
   skips and `Dispel` ignores.
2. **Death by the fighter's own health.** `Battle::wound` calls
   `body::dead(&hp, &stats)`, which measures against a *person's* maximum
   for those stats, not `fighter.max_hp`. Applied from outside for now
   (`flesh_and_blood`): an animal dies at minus its own maximum.
3. **Nerve for creatures.** `ai.rs` only lets people break and run. Animal
   nerve (`Body::nerve`, `Body::odds`) is checked once a fight-second from
   outside.
4. **Dispel** marks any `summon` fighter `fled` ("is sent back"). Undone
   from outside each tick; if the Dispel lands on the fight's very last
   tick the animal counts as having run off rather than stood.
5. **`fights::start_battle` is private**, so `squad_fray_shell` repeats it
   (squad fighters, lights, wards → zones, graves, waiting guardians). If
   it's opened up (`pub(super)`, returning the `Battle` before enemies are
   added) that copy should go.
6. **`finish_npc_fight` robs any beaten travellers.** Animals take nothing,
   so the caravan's goods are saved when the fight starts and put back
   straight after the world ends it. A `camp != NO_CAMP` check there would
   replace that.
7. **A `Hover` variant** for animals (`hud.rs`): the animal tooltip is its
   own small box drawn in `view/animals.rs` for now.

### A bug found on the way (not animals; not fixed here)

`fights::write_back` only takes used potions, scrolls and arrows off a
person **if their detail has been built** (`if let Some(d) =
p.detail.as_mut()`). A stranger's detail is built when the squad sees them
up close. So whether a traveller has been *seen* changes what they carry
into their next fight — which breaks CLAUDE.md rule 1.

Reproduced on seed 1: stand the squad 35 m from six far-off animal fights
(never fighting itself) and one traveller ends the run with different
wounds than if the squad had stayed away. Adding one line before that
`if let` — `p.ensure_detail();` (it's built from the seed, so it's safe) —
makes the two runs identical. I tried it, confirmed it, and took it back
out, since `fights.rs` isn't this branch's to change.

---

## Hooks for the society side

All plain functions on `World`; none of them touch jobs, stockpiles, goods
or law. Owners are opaque `u64`s; yields are `(&'static str, f32)` names and
amounts for whoever turns them into goods.

Anything that changes state has an `_at(t)` twin. **Call the `_at` form
from anything on the world's timeline** (an hourly or dawn tally) with that
moment, not the plain form, or the result will depend on step size.

| Question / action | Function |
|---|---|
| What game is near this point | `game_near(p, radius) -> Vec<Game>` (herd, species, count, where, `protected`, yields per animal) |
| A hunter's day's work, no fight | `take_animals_at(herd, n, t)` → yields |
| How many of a species in a region / the most it holds | `population(r, sp)`, `population_at(r, sp, t)`, `capacity(r, sp)`, `wildlife_census()` |
| Which region is this | `region_at(p)`, `animals::region_of(p)` |
| Which herds belong to this town / this owner | `pens_of_town(town)`, `pens_of_owner(owner)` |
| A pen's head count, hunger, feed | `pen_now(id)`, `pen_at(id, t)`, `pen(id)` |
| Yield since time T (eggs, milk, oil) | `pen_yield_since(id, since)`, `pen_yield_between(id, since, until)`, `animal_yield_since(id, since)` |
| Feed this pen | `feed_pen_at(id, amount, t)` (1 unit = one Turiyu for a day) |
| Cull one | `cull_pen_at(id, t)` → yields |
| Give a pen an owner / a size | `set_pen_owner(id, owner)`, `set_pen_limit_at(id, limit, t)`, `add_pen(…)` |
| Pack animals | `lead_plodder(pen, leader)`, `release_plodders(leader)`, `plodder_capacity()` (kg), `Led::load` |
| Raftbacks as platforms | `floating_platforms()` → where and how wide |
| Fish | `fish_stock(r, sp, t)`, `catch_fish_at(r, sp, want, t)`, `fishing_region(p)` |
| Dive danger (Deepcoil) | `dive_danger(p)` → 0..1, fixed per spot |
| Overgrowth (stub) | `overgrowth(r)` — **the one function animals read it through**; `set_overgrowth_at(r, level, t)` |
| Grazing pressure | `grazing_pressure_at(r, t)` (wild + pastured Turiyu) |
| Cragmaw territory covers this point | `cragmaw_territory(p) -> Option<herd>` |
| Silk | `colonies()`, `cocoons(colony)`, `mother_guarding(colony)`, `take_cocoons(who, colony) -> Take` |
| Body cleared at time T | `body_cleared_at(pid)`, `body_there(pid)` |
| Carcasses | `carcasses_near(p, radius)`, `butcher(id)` → yields |
| Taming | `tameable(herd, slot)`, `tame(who, herd, slot)`; **the check to tie to a skill is `animals::taming_check`** (a keyed true/false) |
| Tamed hounds | `tamed()`, `hound_hunger(id)`, `feed_hound(id, amount)`, `hound_pos(id)` |
| Squad hunting | `hunt(who, herd)`, `herd_near(p, radius)`, `finish_off(who, herd, slot)` |
| Protected species (for law) | `Species::protected` (Wild Turiyu) |
| What happened | `animals.attacks` (last 48), `animals.stats` |

---

## README section (paste under Magic or Society)

### Animals

The world has wildlife and livestock: about 2,000 wild animals in ~250
herds and ~950 kept ones in ~95 pens on a new map. None of them is stepped
through its day. English placeholder names; no real-world animals.

**One table.** Every animal is a row in `animals::SPECIES`
(`src/sim/animals/species.rs`): where it lives, group size, when it's up,
what it eats, how it treats people, speed, its fight numbers, what a carcass
gives, what it gives daily, how thick on the ground, how fast it breeds.
Adding or retuning one is a data edit.

- **Kept:** Turiyu (graze), Shellhen (eggs), Mossback (milk), Raftback
  (oil; a floating platform), Plodder (carries 120 kg).
- **Wild prey:** Wild Turiyu (flagged `protected`), Brushleaper,
  Wallowback, Crag grazer, Dustrunner, Tidepicker (out at low tide),
  Silkcrawler.
- **Predators:** Ridgehound (packs; tameable), Chasm lurker and Mirejaw
  (ambushers), Cragmaw (one beast, one territory), Silk Mother (guards a
  colony), Bonepicker (carrion), Briarback (only where the Overgrowth
  stands high).
- **Sea:** Silverling, Slatefin, Ribbonback, Gulpjaw as stocks per coastal
  region; Deepcoil as a "dive danger" number per spot.

**Three levels of detail, one set of facts.** The map is cut into 1.5 km
regions. Far away, wildlife is read as a number per species per region;
nearer, as herds on their rounds; close up, as individual animals whose
size, age and wounds come from the herd's seed. All three are the same
herd records — the region's number is the sum of its herds — so nothing
changes when you walk up.

**Where a herd is comes from the clock.** Each has a home range and hours
it's up (read off `stealth::daylight`, and the tide for Tidepickers). Its
position at any moment is looked up, not walked.

**Numbers grow by formula.** A herd grows toward what its ground can carry
along an S-curve worked out in one go for any stretch of time (the
"carrying capacity": the most the land feeds). Predators in the region slow
prey and thin it; kills lower the count and it recovers on its own; a
wiped-out herd is restocked from the region after a while.

**Attacks are on the world's timeline.** Each game-hour every hunter's
round is laid against every traveller's schedule. A pack that picks people
up weighs them — its strength against theirs with a keyed roll, like a
bandit camp — and either leaves them, or shadows them for a few minutes and
strikes. The fight is fought at once under the ordinary rules and ended by
the same code that ends a bandit ambush, so a pack attack on the far side
of the map goes exactly as it would beside you. Packs go for the lone and
weak; ambushers for small parties; a Cragmaw for anyone on its ground.

**Livestock** live in pens by each town: an opaque owner number, a home
spot, a feed store, hunger, and what they've produced, all worked out from
the clock. Unfed animals get hungry over three days and give far less.

**Also:** Silkcrawler colonies hang cocoons a Silk Mother guards (she
leaves to hunt for a couple of hours a day); Bonepickers come down on a
body by day and clear it in about an hour, so it can't be found or raised
afterwards; a Ridgehound that is down or young can be tamed, then follows
its owner, fights beside them, and leaves if starved.

**Looking at it.** Hover any animal for species, what it's doing and
whether it's wild or whose it is. `F7` opens the wildlife panel (the
region's numbers per species against what it holds, grazing pressure,
Overgrowth, recent attacks). Debug keys: `H` hunt the nearest herd,
`Y` tame, `U` take cocoons. `headless` output is unchanged.

**Not built (on purpose):** herding and hunting as jobs, hides and meat as
goods, prices, law about protected animals, riding, breeding lines,
fishing as something the squad does. The functions those will call exist
(see `ANIMALS.md`/the hook list).

---

## CLAUDE.md additions

### New rule (number 17)

17. **Animals are looked up, not stepped.** A herd is a small record that
   exists everywhere, whatever band it's in; its place is a pure function
   of the clock (`Herd::round_pos`), its numbers a closed-form curve settled
   at `DAWN` and on kills (`Herd::settle`), a pen's hunger and produce
   piecewise from its last change (`Pen::now`). A region's population is
   the sum of its herds — never keep a second count. Hunters find
   travellers by laying the hour's rounds against the hour's schedules
   (`watch_the_roads`), and `Notice`, `Strike` and `FrayEnd` are events on
   the world's timeline, entered through the one call at the top of the
   loop in `World::step` (`animal_events`). Far fights with animals use the
   full fight rules, run to the end at once, and are filed in
   `World::npc_fights` with `camp: NO_CAMP` so `finish_npc_fight` handles
   the travellers. Every copy of a fight with animals in it is advanced
   through `animals::advance_fray` and nowhere else (it applies animals'
   nerve and death from outside the rules). Only the squad's own run-ins
   depend on being looked at (`animals_by_the_squad`); what the squad
   causes there (`Herd::away`, `met`, `Animals::near`) must never feed back
   into far events — the hourly scan reads `round_pos`, which ignores them.
   All numbers live in `animals::SPECIES` and the named constants beside
   each system; no species is special-cased by name outside its data row
   except where the scope asked for it (Silk Mother, Cragmaw, Briarback,
   Bonepicker, Tidepicker's tide). Society reaches animals only through the
   plain functions in `animals/hooks.rs`, `livestock.rs`, `region.rs` and
   `herd.rs`; a caller on the timeline uses the `_at(t)` form. Animals read
   the Overgrowth only through `World::overgrowth(region)`.

### Verifying visual changes (add to the flag list)

`GAHT_ANIMALS=<scene>[,<scene>…]` — `hounds` (run on until a pack is
stalking travellers at dusk, and stand beside it), `tide` (Tidepickers out
at the next daylight low water), `silk` (a Silk Mother at home on her
colony at midday), `bones` (a fight with a death in it, then wait for the
Bonepickers), `pens` (the nearest town's livestock), `parade` (one of every
species in a row), `panel` (open the wildlife panel), `see:<species key>`
(go and look at the nearest of that species at an hour it's up, e.g.
`see:wallowback`, `see:cragmaw`). Combine with `GAHT_ZOOM`/`GAHT_PITCH`,
`GAHT_SPEED=0` so the moment holds, and `GAHT_VIEW=map` for the map dots.

### Drawing notes (add)

- Animals are drawn by `view/animals.rs` into the per-frame mesh: block
  shapes from the species' `Looks` (build, length, height, colour), a full
  shape per animal near, one marker per herd beyond `PERSON_SIMPLE`-ish
  distances, dots on the map. Pens get a fence ring. No new materials.
- The animal hover and the wildlife panel are drawn there too (their own
  small boxes on `hud::Canvas`), because `hud.rs` wasn't this branch's to
  edit. Fold the hover into `Hover` when convenient.

### Canon notes used so far (add)

- Animals follow the Part 4 prompt (Laz): the species list and their roles
  are his; English placeholder names are final for now; no real-world
  animals. All numbers are first guesses in data tables.
- The Overgrowth doesn't exist yet: a stored 0 per region behind
  `World::overgrowth`. Briarbacks appear above 0.6.
- Wild Turiyu carry a `protected` flag and nothing acts on it yet.
- Taming works 60% of the time by a keyed roll until it's tied to a skill.
- Hounds, hunting, taming and cocoon-taking have debug keys only; no real
  orders or UI yet.

---

## What to tune first

Everything is in `animals::SPECIES` or a named constant.

1. **How dangerous the roads are.** Five to nine animal attacks a day
   world-wide depending on the map, about two people knocked down a day,
   nobody killed in ten days — and the animals win nearly every time (no
   animal died in any far-off fight across four maps). Dials: `PACK_CAUTION` (how
   much stronger a pack wants to be: 1.25), `PACK_BOLD_NIGHTS` (share of
   nights a pack bothers people: 0.5), `AMBUSH_BOLD`, each predator's
   `density` and `sense`.
2. **How much game there is.** `density` per species (animals per km² of
   its country) and `growth` (how fast it comes back).
3. **How hard each animal is to kill**: the `Body` numbers per species
   (`hp`, `cut`, `skill`, `might`, and `nerve`/`odds` for when it runs).
4. **Livestock yields and appetite**: `daily`, `feed`, `forage` per
   species; `STARVE_DAYS`, `HUNGER_LOSS`.
5. **Bonepickers**: `BONE_ARRIVE` (20–60 min) and `BONE_PICK` (40 min)
   against the two hours a body lies anyway.

## Known limits

- An animal in a fight is drawn with the spirit-beast glow on top of its
  own shape (item 1 above).
- Bonepickers only clear bodies; they don't attack the downed.
- Animals knocked down in a fight get up again, as people do. Prey the
  squad hunted and beat is finished off automatically; anything else left
  down needs `finish_off`.
- Far herds don't fight each other: predators thin prey through the
  population numbers only.
- Tidepickers, Silkcrawlers and Bonepickers never fight; Deepcoil and the
  fish are numbers only (the sea is off-limits to the squad).
- Hunting, taming and cocoons are debug keys (`H`, `Y`, `U`), not orders.
