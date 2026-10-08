# Working on Gahturiyu Sim

Laz is the designer; Claude writes the code. He playtests on a Windows desktop
and tweaks through conversation, not an editor. Explain game-dev terms plainly.
World lore (races, languages, architecture, pantheon) lives in the claude.ai
"Gahturiyu" Project, not in this repo.

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
   built once from the seed. Leg timing comes from `Leg::along`, which charges
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
`GAHT_SNEAK=1`, `GAHT_SELECT=k`, `GAHT_INV=k`, `GAHT_CRAFT=k`, `GAHT_DROP=k`
(member k drops some gear), `GAHT_ENTER=1` (member 0 walks into a home),
`GAHT_TALK=1` (talk to the nearest local), `GAHT_STARVE=1`, `GAHT_EXHAUST=1`,
`GAHT_CARRY=1` (member 0 carrying a downed member 2), `GAHT_LIMB=1` (member 0
loses the left arm), `GAHT_RANGED=1` (bandit archers open up), `GAHT_TORCH=1`
(members 0 and 1 light torches; the hunter sets a standing torch),
`GAHT_DEBUG=1` (the detail readout), `GAHT_FOREST=1` (camera far out over the
nearest big wood), `GAHT_SETTINGS=1` (the graphics panel), `GAHT_LOAD=path`
(start from a save; `GAHT_SAVE=path ./target/release/headless 0.5` makes one). Combine with `GAHT_HOURS=17` for night, `13.6` for dusk.

Under Xvfb, Bevy renders in software (Mesa's lavapipe Vulkan driver, package
`mesa-vulkan-drivers`): about 5 fps, so the fps readout means nothing there.
Screenshots compile every shader before the first frame
(`synchronous_pipeline_compilation`) and step the world at a fixed 1/30 s
per frame, so a given flag set gives the same picture. The first Bevy build
takes ~20 minutes on this container's 2 cores; later ones under a minute.

## Drawing notes

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

## Canon notes used so far

- Races coexist Elder Scrolls style: every town is mixed; race tilts trait
  averages with a wide spread per person.
- Horaro live on stilts just off any coast, so every coastal town depends on them.
- Ṭaḍoro don't found towns or build; they lodge in others' homes, or wander and pitch a tent.
- Buildings follow `architecture.md` (grown Roduro stone, Horaro stilts on Roduro
  pillars, quarried Qotiro steps, diaspora Qotiro hall in local dark stone).
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
