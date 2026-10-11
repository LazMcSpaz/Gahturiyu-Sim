# Working on Gahturiyu Sim

Laz is the designer; Claude writes the code. He playtests on a Windows desktop
and tweaks through conversation, not an editor. Explain game-dev terms plainly.
World lore (races, languages, architecture, pantheon) lives in the claude.ai
"Gahturiyu" Project, not in this repo.

## Where we are (Laz, 2026-10-09)

- **System freeze.** No new systems until the playable-MVP list is built
  (visual feedback, looting, recruiting, a money grind, robbery on defeat,
  placeholder buildings with interiors and containers, ruins and lairs,
  progress messages). Base building Stage 4 (raids, law) waits.
- **Every action the player orders has a visual cue** in the 3D view, not
  only a HUD line: a swing shows an arc, an archer's weapon points out to
  shoot and down to reload, a sneaker crouches, and so on. Placeholder
  shapes are fine; no animation rigs needed. New actions come with their
  cue.
- **Placeholder art only for now.** The final building models (Roduro GLBs,
  Horaro kit) stay switched off so the game's look doesn't mix; buildings
  are simple shapes in a few distinct variants.

## Rules that keep the bands honest

These are load-bearing. `tests/consistency.rs` enforces the first three.

1. **Outcomes never depend on band, step size, or squad position.** Bands decide
   how *often* something is looked at and how much *detail* exists — never
   *what happens*. If you add a system where the far band resolves something
   differently (e.g. strength-vs-strength fights), the band-1 version must be
   the same event at higher resolution, and the result must be reconciled back
   into the summary numbers.
2. **Randomness is keyed by what is decided, not when code runs.** Use
   `Rng::from_keys(&[world seed, entity, hour/leg/etc, tag])`. Never draw from a
   shared running RNG inside the step loop.
3. **Movement is scheduled, not integrated.** Groups carry legs with absolute
   depart/arrive times; position is looked up. Don't add per-tick velocity.
4. **Details, once built, are permanent.** `Person::ensure_detail` builds from the
   seed and the cached `might`; never rebuild or reroll an existing detail.
5. **`might` changes only via `recompute_might()`**, called when gear or traits change.
6. **`src/sim` has no graphics dependency.** The window reads the sim and sends
   orders (`order_squad`); it never contains world rules. Foliage, flicker,
   arrow flight and wind are drawing only; light that changes who sees whom
   (fires, windows, torches) comes from `World::lights` in `sim/torch.rs`, and
   the window draws those same sources.
7. **World events run on one timeline.** Hour departures, ambushes and fight
   endings are handled strictly in time order inside `World::step`
   (`encounters.rs`). Ambushes are found from the schedules when legs are
   written (two hours ahead), never by looking around each step. Fights away
   from the squad use the full combat rules, run to the end the moment they
   start — don't add a cruder far-band combat model; if one is ever needed for
   speed, a test must show it agrees with the full one.
   The one allowed exception to rule 1 is the player: when the squad's own
   fights start depends on how often it is looked at (like its walking).
8. **Condition is worked out in pieces.** A squad member's hunger, stamina,
   tiredness and healing are stored as values at one moment plus what they're
   doing; between changes everything moves at a steady rate. Every change
   (start walking, sleep, eat, cross a hunger or tiredness stage, wounds
   fully healed) *settles* the values and the wounds at that exact moment
   and starts a new piece (`condition.rs`). Stage changes and meals are
   found by solving for the moment they fall due, not by checking each
   step. Don't tick these per step. (Climbing is the one by-the-metre cost,
   because the squad's own walking is already step-by-step.) Everyone outside
   the squad heals at the constant `HEAL_PER_HOUR`.
9. **A stranger's kit and spells come from `kit_stats`**, their stats when
   their kit was chosen, never from their current (trained) stats — otherwise
   being met or not changes their might.
10. **Anything that changes what happens lives in the sim.** Terrain and roads
   decide travel times, so they are in `sim/terrain.rs` and `sim/routes.rs`,
   built from the seed plus the map's hand edits (`sim/mapedit.rs`: height,
   ground paint, plants, rocks — saved in map files and in every save; towns
   are placed before edits apply, so edits never move a town). The land
   editor (`view/editor.rs`, F10) calls the sim's brushes; it pauses the
   world, and `refit_land` finds the roads again when it closes. Land
   authored by the town forge (`assets/towns/demo/land.gmap`, the same
   layers) sits under the player's edits as `Terrain::authored`; the sea
   is wherever the land is below 0 (`Terrain::is_sea`), not the world's
   broad coastline, so authored bays and stacks count. Leg timing comes from `Leg::along`, which charges
   each stretch by its slope; keep using it so schedules stay analytic.
11. **Light has one source of truth.** Outdoors it's `stealth::daylight(t)`;
   everything else adds through `torch::Light` / `light_from`. Fights carry
   the fixed lights round them (`Battle::lights`) plus fighters' torches, so
   a far fight in the dark is fought in the same dark. Torches burn down by
   the clock (`Flame::out_at`), never per step.
12. **Everything in `World` is saved.** New world state derives
   `Serialize, Deserialize` (`sim/save.rs`); only what's rebuilt from the
   seed (terrain, routes) is skipped. A fixed `&'static` name from a table
   uses `save::Name` and the helpers there. Bump `save::FORMAT` whenever the
   saved shape changes, and add the new state to `fingerprint` in
   `tests/save.rs`. Nothing may depend on a `HashMap`'s order (a loaded map
   iterates differently). The window's caches of the world reset when
   `Game::loads` changes.
13. **Magic is data.** Spells (`magic::SPELLS`), potions, scrolls and worn
   items all describe themselves with `effects::Effect` (what it does, how
   strong, how long, who or what it reaches). A new spell or item is a line
   of data; a new *kind* of effect is a `Does` variant plus its rule in the
   two appliers — `combat::Battle::affect` (fights) and
   `World::apply_effect` / `affect_object` (`casting.rs`, everywhere else).
   The fight AI (`ai.rs`) reads spells by their effects, never by name.
   Lasting effects outside fights are boons (on people) and wards (on the
   ground); they become statuses and zones in a fight and go back out
   after. Anything that changes a squad member's condition (load, hunger,
   tiredness) settles their condition when it starts and has its end on
   the condition timeline. Summoned creatures and raised dead are fighters
   whose `pid` is `combat::NOBODY`: never index `people` with a fighter's
   pid without checking `is_person()`. Bodies brought into a fight to be
   raised sit on `GRAVE_SIDE` and are never written back.
14. **Meeting someone never changes what they can do.** Felt spells come
   with use for the squad only; everyone else's spells are
   `starting_spells(kit_stats, seed)` (all felt spells their feel reaches,
   a seeded share of the rest), the same whether or not they've been met.
15. **Cultures are tendencies, not scripts.** Never write "Roduro towns do
   X". Each people's leanings live in one data table (`culture::PROFILES`);
   a community's customs are chosen from the head-count-weighted blend of
   its residents' leanings (plus a seeded nudge) by one fixed roll per
   community and custom, stored as state (`Community::customs`), and
   re-chosen at dawn when the population has shifted enough. Systems read
   the stored customs and people's own rolled habits (`Life::habits`) —
   never `race` directly. A person's own habits lean toward their people's
   profile but are rolled per person, like traits.
16. **Town life is looked up from the clock.** A day plan is a pure function
   of the person, their community's customs, the day and the hour
   (`World::day_plan`); positions, who's at work and what's open are read
   off it. The economy is settled on the world's timeline: each hour's
   work is worked out from the plans at the top of the hour and added as a
   rate (`society::Flow`), and the day's food, tax and changes are tallied
   at `DAWN`. Caravan arrivals and homecomings are timeline events like
   ambushes. Never step people through their day.
17. **Making and prices follow the same clock.** Town crafters pick their
   work at the top of the hour (one roll keyed to them and the hour, from
   what the store holds and lacks), their hours settle on the hour, and the
   finished thing goes to the store or the shelf (`making.rs`). One recipe
   table (`crafting::RECIPES`) serves the squad and the towns. Prices are
   worked out from the store when asked; anything on the world's timeline
   (caravans, the dawn tally) passes the event's time
   (`price_factor_at` / `worth_at`), never `self.time`. Wear is the squad's
   alone, like hunger. Grown things only grow while tended: orders and the
   squad's beds are checked at dawn, never per step.
   Likewise what a fight uses up (potions, scrolls, arrows) leaves only the
   squad's packs: strangers restock at home, so their kit never depends on
   whether they'd been met.
18. **Law is settled at dawn.** Governments, rites, townsfolk's disputes,
   bonds and unrest are worked out in `law::dawn_law` on the world's
   timeline, with the dawn's time passed in (never `self.time`). A
   townsperson's duel is fought out at once with the full combat rules
   (like far fights). Bonds end by the clock (`Bond::until`). The squad's
   own crimes are the player exception (`pursuit.rs`): a witness is found
   when it's done (`spotter`), tells the watch or not by one keyed roll
   (`report_chance`: friendship, scruples, the ring, bad blood with the
   victim), and a told crime sends a guard who is moved step by step and
   catches the thief (then `judge`) or loses them (then a bounty). Office eligibility reads race and sex directly — that's the
   canon rule (Laz), not a culture leaning; everything else reads customs.
   Sex is one reading of the seed shared with the names (`law::woman`/`man`
   are `sim::names::gender`), so a priestess always has a woman's name; a
   Qotiro priest by trade is always a woman (canon, Laz), and `Job::title`
   words the job for the person ("Priestess").
19. **Lives are settled on the clock, and few people act.** Purses, work
   status, needs, dealings, gossip, grudges' first rung, the ring's choice
   and the storyteller all run at dawn (`lives.rs`, `memory.rs`,
   `history.rs`, `ring.rs`, `stories.rs`), with the dawn's `t`; storylines
   are acted out on the hour they were given (`story_hour`), never per
   step. Only storylines the storyteller picks act — keep new behaviour a
   kind of storyline, under its cap — and it never reads the squad's
   position. Per-person lists are inline and capped (`few::Few`); keep a
   person within about 200 bytes (`tests/budget.rs`). What people say is
   data in `data/lines/` (format in `data/lines/FORMAT.md`), chosen by
   conditions; the code gathers facts, it doesn't write lines.

20. **Animals are looked up, not stepped.** A herd is a small record that
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

21. **Weather is looked up from the place and the time.** `weather_at(pos, t)`
   is a pure function of the seed, the region and `t` (`sim/weather/`): never
   tick it, never store it in `World`. Every number is in
   `data/weather/climate.ron`. Regions have their own climates and share the
   big weather, which reaches a place `lag(pos)` after the sea's edge.
   Anything on the timeline that reads the weather passes the event's time.
   What the weather does to people and things is `weather_effects(pos, t)`,
   and unusual days are `omens(region, day)`: both looked up the same way. A
   rule that reads them asks once per decision (a fight's start, a leg's
   departure, an hour of a day plan). Nothing reads it yet
   (`docs/weather-hooks.md` says where each piece is meant to connect).

22. **Bases are built on the clock.** A construction site stores its labour
   at one moment plus a rate (the builders on it now); its finish is solved
   for (`Site::done_at`) and handled on the world's timeline
   (`World::base_events`, one call in the event loop). Anything that changes
   the rate (someone arriving or leaving, materials delivered, a site
   finishing) settles the base at that moment and solves again
   (`base_changed_at`). Who of the squad is at a base is checked at the start
   of each step (the player exception, like walking). Buildings are data
   (`base::BUILDINGS`, each with a `Method`, so culture methods are new
   lines); the window places them only through `check_place` /
   `place_building` / `place_wall`. A base keeps cached `wealth` and
   `defence`, recomputed only on change. Squad members left at a base
   (`baselife.rs`) leave `Squad` but stay `in_squad`: they keep their
   condition timeline (eating from the base's store, sleeping in its beds)
   and work in rounds with fixed ends on the same timeline (`start_cycle` /
   `finish_cycle`); a building's health is stored the same way (value at a
   moment plus a rate: rot, mending), and its fall or mend is solved for.
   Hired hands are residents with a `Hire`: they leave their town's roll,
   community and household (restored when they go home) and keep their
   trade (`trade_of`, `work_skill`). Their wages, food, beds and loyalty
   are tallied once a day at `DAWN` on the base's timeline (`base_dawn`),
   paying from the base's coin then the squad's; quits and theft are keyed
   rolls and are noted in the home town's history (gossip).

## Verifying visual changes

The window can screenshot itself headlessly:

```
GAHT_SHOT=out.png GAHT_FRAMES=60 xvfb-run -a -s "-screen 0 1600x1000x24" ./target/release/gahturiyu
```

Optional: `GAHT_VIEW=map`, `GAHT_ZOOM` (map px/m, or 3D camera distance in m),
`GAHT_PITCH` / `GAHT_YAW` (3D camera, radians), `GAHT_SPEED=0..4`,
`GAHT_HOVER=x,y` (fake mouse, for tooltips), `GAHT_NUDGE=dx,dy` (move the
squad's start, e.g. to put a town on the band edge). For the newer systems:
`GAHT_HOURS=h` (run h hours first), `GAHT_CAMP=k` (start 70 m from bandit camp
k), `GAHT_WAIT=h` (run until a fight is on nearby), `GAHT_BANDITS=n`,
`GAHT_SNEAK=1`, `GAHT_SELECT=k`, `GAHT_INV=k`, `GAHT_CRAFT=k`,
`GAHT_MAKE=forge|bench|loom|workbench|desk|bed|mortar` (the squad at the nearest such station
with the craft and some materials, its making screen open), `GAHT_DROP=k`
(member k drops some gear), `GAHT_ENTER=1` (member 0 walks into a home),
`GAHT_TALK=1` (talk to the nearest local; `GAHT_TALK=roduro|qotiro|horaro|tadoro` the nearest of that people),
`GAHT_NAMES=native|english` (the Names setting), `GAHT_STARVE=1`, `GAHT_EXHAUST=1`,
`GAHT_CARRY=1` (member 0 carrying a downed member 2), `GAHT_LIMB=1` (member 0
loses the left arm), `GAHT_RANGED=1` (bandit archers open up), `GAHT_TORCH=1`
(members 0 and 1 light torches; the hunter sets a standing torch),
`GAHT_DEBUG=1` (the detail readout), `GAHT_FOREST=1` (camera far out over the
nearest big wood), `GAHT_SETTINGS=1` (the graphics panel), `GAHT_LOAD=path`
(start from a save; `GAHT_SAVE=path ./target/release/headless 0.5` makes one),
`GAHT_BOOK=k` (member k's spell book), `GAHT_HELD=1` (the squad's mage holds
Restore ready), `GAHT_SUMMON=1` (a fight where the mage calls up a spirit
beast and raises a fallen bandit; try `GAHT_ZOOM=16 GAHT_PITCH=0.45`),
`GAHT_TOWN=1` (the town panel for the nearest town; `GAHT_TOWN=roduro|qotiro|horaro|mixed`
goes to that kind of town first), `GAHT_DUEL=1` (member 0 judged by duel),
`GAHT_SHUN=1` (the nearest stilt village withdraws; runs past the next dawn), `GAHT_TRADE=1` (trading
with the nearest merchant at work), `GAHT_CONVO=1` (a local robbed last night talks about it,
assembled from `data/lines`), `GAHT_GUARD=1` (member 0 on a guard contract at a merchant's stall;
the journal shows it), `GAHT_EDIT=1` (the land editor open beside a few demo strokes: a terraced
snow-capped hill, sand and mud, more trees, rocks; screenshots ignore
`maps/` unless `GAHT_MAP=1`), `GAHT_FEUD=1` (two households fall out over 16 days; the town panels show
the log; slow to set up), `GAHT_SOCIETY=runners|boats|tides` (go and
watch the midday meal run, the dawn boats, or a stilt village; for tides
compare two days, e.g. `GAHT_HOURS=27` and `123`; add `GAHT_SPEED=0` so the
moment holds), `GAHT_FORGE=1` (start at the forged demo town; the land, homes
and ways come from `assets/towns/demo`, written by
`cargo run --release --bin town_forge -- step N`; `GAHT_NUDGE` moves the
squad from there). Combine with `GAHT_HOURS=17` for night, `13.6` for dusk.
`GAHT_ANIMALS=<scene>[,<scene>…]` — `hounds` (run on until a pack is
stalking travellers at dusk, and stand beside it), `tide` (Tidepickers out
at the next daylight low water), `silk` (a Silk Mother at home on her
colony at midday), `bones` (a fight with a death in it, then wait for the
Bonepickers), `pens` (the nearest town's livestock), `parade` (one of every
species in a row), `panel` (open the wildlife panel), `see:<species key>`
(go and look at the nearest of that species at an hour it's up, e.g.
`see:wallowback`, `see:cragmaw`). Combine with `GAHT_ZOOM`/`GAHT_PITCH`,
`GAHT_SPEED=0` so the moment holds, and `GAHT_VIEW=map` for the map dots.
`GAHT_DRAG=x,y` (a selection box held from there to the `GAHT_HOVER` point).
`GAHT_CHASE=1` (member 0 seen stealing in a town with its watch out; the guard on their way).
`GAHT_LOOT=1` (two bandits lie beaten beside the squad; member 0 goes through the
first one's things: the loot panel).
`GAHT_GRIND=1` (two of the squad at work at the nearest woodlot, mid-morning; `GAHT_GRIND=mine`
for the nearest iron seam).
`GAHT_MENU=1` (the right-click menu for whatever is under `GAHT_HOVER`).
`GAHT_ALT=1` (Alt held: labels on everything hoverable, long tooltips).
`GAHT_RUIN=k` (midday, the squad 45 m from ruin or lair k, the camera on the ruin and its cache; ruins come first, then lairs).
`GAHT_HINT=<id>` (show that first-hour tip: welcome, town, work, fight, loot, full, hungry, night,
beaten; screenshots otherwise show none and never touch `hints.txt`).
`GAHT_INTERIOR=1|tadoro|crowded|slope|<variant key>` (member 0 walks into the nearest open workshop or
shop and opens a container: the cut-open rooms and the container panel; camera close unless
`GAHT_ZOOM`/`GAHT_PITCH`. `tadoro`: a building a Ṭaḍoro lodges in, framed on their corner;
`crowded`: the most residents per sleeping place (extra bedrolls); `slope`: a building whose
floor stands well above the ground on one side; a variant key such as `roduro_benchroom`:
the nearest open building of that variant).
`GAHT_VARIANTS=1|roduro|qotiro|horaro` (every building variant, or one people's, in rows
on open ground near the squad; add `,open` to see them cut open). `GAHT_CUTAWAY=1` (every
building near the camera drawn cut open). `GAHT_MODELS=1` (the final building models back on;
off by default).
`GAHT_SIGNS=1|roduro|qotiro` (frame the spot in the nearest town, or the most Roduro / Qotiro one,
where the most signs stand: workplace signposts and service plaques; add
`GAHT_HOURS=15` for night, `GAHT_ZOOM=60 GAHT_PITCH=0.6` for the usual town camera).
`GAHT_RECRUIT=n` (n willing townsfolk join the squad, fees covered; the last is asked in
conversation, which stays open; try `GAHT_HOURS=30` so the first dawn has sorted out who's jobless).
`GAHT_REST=1` (the squad beds down where it stands, after `GAHT_HOURS`, so the tent in the hunter's pack goes up),
`GAHT_BUILD=1` (a demo outpost in the wilds near the start: huts and a lean-to up,
a palisade and gate under way, the Build panel open and a hut's ghost on the cursor;
put the cursor with `GAHT_HOVER`; `GAHT_BUILD=base` leaves a farmer and a builder there,
hires a hauler from town and opens the Base tab a day later; `GAHT_BUILD=store` does the same
and has the first member go through the base's store).
`GAHT_WEATHER=1` (the weather panel; in play it's F7 with the detail switch L on; `folded` for its headline, `off` for none),
`GAHT_WEATHER_HOURS=h`, `GAHT_PRESET=clear|overcast|drizzle|seafog|downpour|gale|thunderstorm|snow`
with `GAHT_PRESET_STRENGTH=0..1` (forced weather; U cycles it in the window), `GAHT_FLASH=1`
(a lightning strike), `GAHT_WEATHER_TAB=effects`, `GAHT_WEATHER_QUALITY=off|low|medium|high`,
`GAHT_WEATHER_COST=1`; more in `docs/weather-hooks.md`.
`headless society [days] [seed]` prints every town's customs, jobs, food and money.

Under Xvfb, Bevy renders in software (Mesa's lavapipe Vulkan driver, package
`mesa-vulkan-drivers`): about 5 fps, so the fps readout means nothing there.
Screenshots compile every shader before the first frame
(`synchronous_pipeline_compilation`) and step the world at a fixed 1/30 s
per frame, so a given flag set gives the same picture. The first Bevy build
takes ~20 minutes on this container's 2 cores; later ones under a minute.

## Voices (a tool, not a system)

`tools/voice/` invents accents and makes spoken clips for NPC lines ahead of
time (Python, not part of the game build; `tools/voice/README.md`).
- Laz invents accents and speakers by ear in the panel
  (`tools/voice/studio.bat`, or `python tools/voice/voice.py studio`). It
  saves `accents.toml` (shared by a people or region) and `voices.toml`
  (one speaker each). Those two files are his; don't edit them unasked.
- An accent starts from a tongue's own habits (`tongues.toml`, derived from
  the naming system's borrow rules) and is a setting of the knobs in
  `sounds.toml`. Dials and emotions are numbers in `dials.toml`; invented
  names are spelled out in `lexicon.txt`.
- Claude can't hear: use `--check` (a listener model writes down what it
  hears), `accents NAME` (prints the bent sounds) and `selftest`, and leave
  how anything *sounds* to Laz.
- The game plays no voices yet: wiring clips in waits for the system freeze
  to lift.

## Drawing notes

- The HUD follows Laz's mockup (2026-10-10): dark umber and brass, serif
  type (Alegreya for text, Alegreya SC for names, Cinzel for headings, in
  `assets/fonts`, OFL; `hud::Face`). `view/frame.rs` draws the squad list
  (top left), the tracked job and news (top right), the place name, and the
  bottom band (buttons, the lead member's plate with a body showing each
  part's wounds, orders, day and hour, speed, a north-up little map). Every
  panel's ground is `Canvas::frame_box`; a tooltip whose first line is
  `GOLD` gets it as a Cinzel title. The old side panel (counts, timings,
  full log) shows with L; the keys list is the Keys button.
- The window is Bevy 0.19.1 (pinned) with bevy_egui 0.42.0 for the panels.
  The panels are drawn with egui's painter through `hud::Canvas`, in pixels
  from the top left with text placed by its baseline (the old layout carried
  over). Clicks on panels are hit-tested against the boxes drawn last frame.
- Two cameras: the 3D one (order 0, HDR, bloom) and a 2D one on top for egui
  (order 1, also HDR, no tonemapping, no clear). If the 2D camera isn't HDR
  it paints its own black picture over the 3D one.
- Meshes are built in code with `view/mesh.rs` (positions, facings, linear
  vertex colours). Materials don't cull or flip facings (`cull_mode: None`,
  `double_sided: false`): our meshes aren't consistently wound, so the facing
  we give each vertex is what's lit.
- Three materials in `scene::Mats`: `lit` (sun, moon, fires), `glow` (unlit;
  brightened at night so windows and flames bloom), `flat` (unlit ground
  markings: rings, order lines).
- Three lifetimes: the ground and roads are rebuilt only when the camera
  moves a coarse cell or zooms; each town near the camera is its own entity,
  rebuilt only when someone goes in or out of a building; everything that
  moves is one mesh rebuilt every frame. All three carry `NoFrustumCulling`
  (their bounds change when rebuilt).
- Anything laid on or standing on the ground uses the cached `Grid::height`
  (the drawn surface), not `Terrain::height` (the true surface) — between
  coarse mesh vertices the two differ by tens of metres on mountainsides.
- Thin ribbons are widened with distance from the camera, or they alias into
  dashes when seen edge-on.
- Graphics settings (`view/settings.rs`, key O) scale foliage density and
  reach, shadow reach, the lamp count and bloom. Saved to `settings.txt`
  (git-ignored). Bumping `Settings::version` makes foliage rebuild.
- Lights (`view/light.rs`): sun and moon follow `stealth::daylight` and the
  hour; the sun casts shadows out to 3× the camera distance. Point lights are a
  pool of `MAX_LAMPS`, filled each frame with the nearest of
  `World::lights()` plus a few lit windows; flicker uses real time (drawing
  only). `NIGHT_BRIGHTNESS` is the one dial for how dark night is.
- Detail levels use Bevy's `VisibilityRange` (distance from the camera, with
  a dithered crossfade where two ranges overlap). Models: `models.rs`, levels
  by file name, missing ones simplified with meshopt. Foliage: `foliage.rs`,
  chunks round the camera, instanced (one mesh + one material per kind, many
  entities). Far tree billboards are merged per 512 m chunk and turned to the
  camera in the vertex shader.
- Custom shading is one small vertex shader (`foliage.rs`, built from a Rust
  string so its distances come from the named constants) added to
  `StandardMaterial` through `ExtendedMaterial`: wind sway weighted by vertex
  colour alpha, and billboard corners in UV_1. Shadows use the standard
  shadow pass, so trees' shadows don't sway.
- People: full figure near, a plain shape beyond `PERSON_SIMPLE`, a shape per
  traveller for band-2 groups, one marker beyond band 2.

- Names (`view/lexicon.rs`, the O panel's "Names" row): towns, jobs and
  items in common English, or as the people around you say them (the
  nearest town's founders' tongue). Tooltips give the other form. In
  dialogue, `{w:root}` in `data/lines/` puts in the speaker's own word,
  marked `⟦native|meaning⟧` (`sim/speech.rs`); the talk panel draws it in
  `NATIVE` with a dotted underline and gives its meaning along the bottom
  when the mouse is over it. Text that can't hover uses `speech::plain`.

- Floating words over heads (`view/floaters.rs`: "+2 Timber", hurt and
  healing, "?"/"!" as suspicion builds, a skill rising) come from comparing
  the world with what the window saw last frame — drawing only. Left-drag
  draws a box that selects the squad members inside it.

- Sound (`view/sound.rs`, drawing only): our own placeholder effects in
  `assets/sounds` (`manifest.json` lists name, file and group: interface,
  screens, world), made by `tools/sounds/make_sounds.py`. Ask with
  `game.sounds.ui(name)` / `.at(name, pos)` (base name; a take `_1..3` is
  picked at random); `sound::update` plays the queue, adds panel, selection
  and nearby-fight sounds by comparing with last frame, and rate-limits each
  name to once per 50 ms. World sounds fade with distance from a listener
  near the camera's focus, silent beyond 120 m. The O panel's Sound row is
  the master volume (`settings.txt`). Screenshots are silent (the audio
  plugin is off with `GAHT_SHOT`).

- Buildings are placeholder shapes by variant (`view/interiors.rs`, from
  `sim/layout.rs`): outside in the town mesh; a building a squad member is
  in is drawn cut open (floor, low walls, inner walls, furniture). Its
  containers are drawn every frame (lids open while searched). The final
  models load only with `GAHT_MODELS=1` (`models::use_final_models`).
- What stands in a room is placed by pure geometry in `sim/layout.rs`
  (`footprints`, `sleeping_plan` for extra bedrolls and the Ṭaḍoro lodger's
  corner, `loose_spots` for belongings); the window only draws it, and
  `tests/buildings.rs` checks nothing overlaps in every building. Floors are
  level at the highest ground under the outline (`interiors::floor_height`,
  cached per town rebuild); people and things indoors stand on it.
- Signs are for players finding services (Laz): in-town workplaces carry a
  signpost, and only buildings that are themselves service places
  (`layout::serves`: the temple, the island hall) a sign by the door — a
  dark plaque with a gold symbol and an iron lantern (`view/signs.rs`, one
  block pictogram per `layout::Sign`). Trade homes hang none; the door hover
  still names the trade and its keeper (`World::keeper`).

- Animals are drawn by `view/animals.rs` into the per-frame mesh: block
  shapes from the species' `Looks` (build, length, height, colour), a full
  shape per animal near, one marker per herd beyond `PERSON_SIMPLE`-ish
  distances, dots on the map. Pens get a fence ring. No new materials.
- The animal hover and the wildlife panel are drawn there too (their own
  small boxes on `hud::Canvas`), because `hud.rs` wasn't this branch's to
  edit. Fold the hover into `Hover` when convenient.

## Canon notes used so far

- Races coexist Elder Scrolls style: every town is mixed; race tilts trait
  averages with a wide spread per person.
- Horaro live on stilts just off any coast, so every coastal town depends on them.
- Ṭaḍoro don't found towns or build; they lodge in others' homes, or wander and pitch a tent.
- Buildings follow `architecture.md` (grown Roduro stone, Horaro stilts on Roduro
  pillars, quarried Qotiro steps, diaspora Qotiro hall in local dark stone).
  Qotiro build in the island's dark stone everywhere, never sandstone (Laz,
  2026-10-09): their stepped, quarried shape and gold sun discs set them apart.
- A building's variant is chosen once, at world creation, from who lives
  there (`World::style_buildings`, `layout::TRADE_STYLES`, stored by key in
  `Settlement::styles`): a Roduro home is a workshop only if its head of
  house (the eldest) has a trade; a Qotiro block if about a sixth of its
  residents share one; plain homes go by head count. Residents are whoever
  sleeps there (`dwelling`), not `Household::home`. It never changes after.
  Keepers and trade marks use the same table (`layout::keeps`, `sign_of`).
- The south-east inland is Qotiro country — a raised arid plateau; elsewhere
  inland is mostly Roduro. Mountains wall the north and east, plus one massif.
- Population is 5,000 split evenly by race (a starting point, Laz's call).
  Towns are mixed: founders count 2.5× in their own town, Horaro lean coastal
  (Laz: very present on the coast but rarely the majority). Bandits are extra.
- Farewells name gods from `pantheon.md` by people (Horahìda, Dodìṭo, Qotisho,
  Rìthaduya) and night is "Hiyaḍote's hours" — placeholder flavour, not canon
  ties between races and gods.
- Deaths are rare, Kenshi-style: a head or torso at zero knocks you out; only
  falling to minus its maximum kills. Limbs at minus their maximum are lost.
- No diseases, no aging; starvation knocks out but never kills (this slice).
- Tents are bought items anyone can carry (Laz's call); there's no shop yet,
  so the squad's hunter starts with one.
- The sea stays off-limits to the squad for now.
- Torches are ordinary bought/carried items anyone can use (no racial tie);
  the squad starts with two each and the hunter with two standing torches.
- Magic follows the Project doc `claude/magic-system.md` (Laz's decisions):
  three styles are the skills (Felt, Structured, Ritual), eight domains are
  tags, magic is equally strong everywhere, no levitation, no diseases,
  stone-tending is a craft not a spell, raised dead are mindless and
  necromancy never brings back a fallen squadmate. "Energy" is the mana pool.
- Shrines for rituals are the Qotiro temples and halls for now (placeholder
  until shrines exist).
- Society Part 1 follows the Project doc `claude/society-economy.md`
  (Laz's decisions). English placeholder names only; no new place names.
  A coastal town is two communities (land and the stilt village), each with
  its own blend and customs. Ṭaḍoro have no cooking leaning (hosts feed
  them). Dawn (the boats, the day's tally) is 06:00. One coin (struck by
  Ṭaḍoro presses) plus 50-coin notes. Seasons only move seasonal working
  hours (a placeholder 48-day year); daylight doesn't follow them yet.
  Parts 2 and 3 build on it (below).
- Society Part 2 (materials and crafting) follows Laz's brief: placeholder
  English names, no new place names. Its scope-downs (Laz asked to keep it
  lean): stations are free to use (no rent); lessons and manual study run
  while you carry on (no staying put); an order takes its feedstock from the
  Tender's town store; only the squad's gear wears; caravans carry bulk goods,
  not shelf pieces; pitch burning, paper and water, and fishskin's wet grip
  aren't modelled (no fire or wet damage yet); each crafter teaches one way
  (deep or drilled), rolled from their people's leaning.
- Fire, water and cold are lasting conditions in fights (`elements.rs`):
  burning (hurts, gives light through `spell_light`), wet, chilled, frozen,
  and their meetings. Laz asked for what's carried to be affected too: the
  squad's paper burns or soaks, pitch-sealed gear scorches. Laz: "might be
  too realistic — we can roll it back." No rain or wading yet.
- Society Part 3 (government, law, bondage) follows Laz's brief and
  `claude/society-economy.md`. Scope-downs: no leagues yet (they'd be led
  as their largest town); bound townsfolk keep their job (their output isn't
  routed to the holder); buying land is a standing step with no purchase yet;
  carrying stolen goods is a `Wrong` with no detection yet; omens reported
  or faked by others are a hook (`bad_omen`); fights in the street on a
  revolt are left out; the squad can't hold bonded workers.
- Part 5 (emergent lives, opportunities, dialogue) follows Laz's brief. Money is
  per household (a stilt village living as one shares a purse); squad members
  keep their own coin. Scope-downs: escort and champion jobs exist as kinds but
  aren't offered yet; no recruitment (only `would_join`); a ring can't be
  "turned" yet; ring members are found out by finding out one of their deeds;
  no weather, so the `rain` piece never fires; fencing pays ring and thief out
  of the wider world (as caravans do). A dialogue "voice" is the speaker's
  people's way of speaking (presentation only, not a world rule).
- Homes are open to visitors by day. Bedded down in someone else's home at
  any hour, or inside one after dark, a squad member is told to go by
  whoever of the house is up; staying on (ten minutes), or doing it again
  that day, is trespass, reported like any other crime (Laz, N5;
  `law.rs` `mind_the_guests`). A bed paid for at an inn is theirs.
- The Overgrowth is still a stand-in: three wild regions far from any town
  start overgrown, with a Briarback group each (Laz, N3; `animals/hooks.rs`).
- Taming, cocoons, pens and pack beasts have no orders in this version
  (Laz, N6): the wildlife and weather panels and their keys (F7, F12, H, Y,
  F11, U) work only with the detail switch (L) on.
- Shunning falls on the thief alone; what was stolen in a town has no buyer
  there until it's settled (Laz, N4).
- Skipped spells and why are listed in README (Magic). Far sight is skipped
  because the map shows everything; it needs fog of war first.
- Trade (Laz, N7): buying low in one town and selling high in the next
  should make money, so merchants sell at 115% of a thing's worth and buy at
  75% (`economy::BUY_MARKUP`, `SELL_SHARE`; `headless trade` counts the
  routes that pay). The squad doesn't run caravans of its own. Being hired
  to guard a caravan is wanted but not built (after the freeze).
- Making is done at the station (`view/makeui.rs`, mockup 4): the screen is
  a view of `crafting::making_list` and `makers`; a maker draws on the packs
  of squad members standing with them (`craft_givers`); skill at a thing is
  shown in words (`crafting::Hand`), never as a chance.
- Animals follow the Part 4 prompt (Laz): the species list and their roles
  are his; English placeholder names are final for now; no real-world
  animals. All numbers are first guesses in data tables.
- The Overgrowth doesn't exist yet: a stored 0 per region behind
  `World::overgrowth`. Briarbacks appear above 0.6.
- Wild Turiyu carry a `protected` flag and nothing acts on it yet.
- Taming works 60% of the time by a keyed roll until it's tied to a skill.
- Hounds, hunting, taming and cocoon-taking have debug keys only; no real
  orders or UI yet.
