# Weather: what exists and how to connect to it

Second agent's work, branch `agent2/weather`. All four stages are done (stage 1:
rules, regions, forecast, tests, numbers panel; stage 2: light and sky, haze, ground
fog, rain, wet surfaces, forced weather; stage 3: shared weather that comes in off the
sea, lightning, snow, wind, sound slots; stage 4: omens, `weather_effects`, quality
settings, this file).

**Nothing in the game reads the weather yet.** It is drawn and it can be asked about;
no rule depends on it. The two sections "What the weather does" and "Omens" below say
where each thing is meant to be connected.

## What weather is

- **Looked up, never run.** `weather_at(place, time)` is worked out from the world
  seed, the place and the time. There is no weather state in `World`, nothing in
  `World::step`, nothing saved, and `save::FORMAT` is untouched.
- **A character by region.** Six kinds of country, read off the land
  (`sim/weather/region.rs`): open sea, exposed coast, sheltered coast and lowland,
  upland, mountain, plateau. Each has its own climate table; borders fade over about 200 m.
- **The big weather is shared, and comes in off the sea** (Laz asked for this after
  stage 2; it replaces "by region only"). One set of slow curves and one run of storms
  belong to the whole country. Each region takes its share (`shared` in the climate
  file) and catches a storm or not by how stormy it is, so what reaches the sheltered
  country has crossed the exposed country first. Everything arrives later further east
  (`lag(pos)`, up to `crossing_hours`). Still nothing is run: a place's weather is its
  regions' skies read at `t - lag(place)`.
- **All the numbers** are in `data/weather/climate.ron` (compiled in, like `data/lines`).
  `headless weather [years] [seed]` prints what the tables actually produce.

## Asking about the weather (`gahturiyu_sim::sim::weather`)

| Call | Gives |
|---|---|
| `world.weather(pos)` / `world.weather_at(pos, t)` | `Weather` at a spot, now or at any time |
| `weather::weather_at(&terrain, seed, pos, t)` | the same without a `World` |
| `world.forecast(region, from, hours)` | `Vec<Weather>`, one per hour, at the region's typical spot |
| `weather::weather_in(seed, region, t)` | one region's weather at its typical spot |
| `world.climate_region(pos)` / `weather::mix(&terrain, pos)` | the region, or the shares near a border |
| `world.forecast_at(pos, from, hours)` | `Vec<Weather>`, one per hour, at that very spot |
| `weather::lag(pos)` | seconds the weather takes to reach a spot from the sea's edge |
| `weather::skies(seed, t, lag)` + `weather::weather_with(&skies, &mix, height, floor)` | the quick way to ask about many places about as far east as each other (with a spot's own `lag`, the same answers as `weather_at`) |
| `world.strikes(from, to)` | every lightning strike on the map between two moments: when, where, how big |
| `world.weather_effects(pos, t)` | `Effects`: what the weather there does to sight, hearing, travel, boats, crops ... (below) |
| `weather::effects_of(&weather, t)` | the same for a `Weather` already in hand (a forecast hour, say) |
| `weather::crop_growth_over(&terrain, seed, pos, from, to)` | a stretch of time's crop growth, averaged hour by hour |
| `world.omens(region, day)` / `world.omens_at(pos, day)` | the omens in a region on a day, or as seen from a spot (below) |

`Weather` holds: `kind` (and `label()`, `describe()`), `cloud`, `rain`, `snow`, `wind` (m/s),
`wind_to`, `gust`, `fog`, `fog_height`, `visibility` (m), `temperature`, `wetness`, `storm`,
`lightning` (flashes a minute), `sea`, `snowline` (what falls above it is snow),
`snow_lies_above` and `snow_cover` (snow on the ground), `cloud_base`, `region`.

On the world's timeline, pass the event's own time (`weather_at(pos, t)`), never `self.time`,
as with prices (rule 17).

## What the window draws (`src/view/weather/`)

Everything reads one resource, `WeatherView`, filled once a frame by `observe`: every
region's sky (`skies`), the weather where the camera looks (`here`) and at the camera
itself (`eye`), `wet`, and a `clock` for things that move. `view.at(&game, pos)` gives
the weather shown at any spot. Forced weather (the panel, the U key, `GAHT_PRESET`)
replaces the skies, so every part of the drawing follows it.

| What | How | Where |
|---|---|---|
| Sun, moon, sky light | scaled after `light::update` has set the clear-day values | `look.rs` |
| Sky colour and haze | `ClearColor` and the camera's `DistanceFog` are set again after `light::update` | `look.rs` |
| Low fog | two soft sheets draped round the camera; thick where the ground is under the fog's top (the same rule as `sim/weather/local.rs`) | `groundfog.rs` |
| Rain | one mesh of streaks in a box of air round the camera, placed from the clock | `rain.rs` |
| Wet surfaces | `WeatherView::wet` (0..1), applied to `scene::Mats::lit` (darker, shinier) | `look.rs` |
| Snow falling | the rain's drops made slow, small and soft | `rain.rs` |
| Snow on the ground | `palette::set_snow_line`: the ground's own colours whiten above it (see below) | `look.rs` |
| Lightning | a flash over the whole scene, a jagged bolt, and thunder handed to the sound slots after the sound's travel time | `lightning.rs` |
| Wind | `WeatherView::wind`: this instant's wind with its gusts (direction times m/s), for anything that sways | `mod.rs` |
| Sounds | `WeatherView::sound`: each loop's volume and the one-shots to start (nothing is played) | `sound.rs` |

Things to know when merging:

- **`look::apply` must run after `light::update`** (it multiplies what that sets, and
  its `DistanceFog` insert must land second). The plugin orders it so.
- **Wet surfaces reach `Mats::lit` and `Mats::ground`** (the land, roads, our own
  building meshes, people): darker, less rough, more reflective. The land's shader
  (`view/ground.rs`) wrote its own colour over the material's, so one line there now
  multiplies by the material's `base_color` (white unless wet). The GLB building models
  and the foliage have their own materials; to opt them in, read
  `Res<WeatherView>().wet` (0..1) and darken / lower the roughness the same way.
- **Thick fog is drawn thinner than it is.** The simulation's sight can be 30 m; the
  drawing never hides the spot the camera looks at (haze is held off to 2.2 camera
  distances and the fog sheets are thinned over that spot), or the game couldn't be
  played from the squad camera. Rules should read `weather_at(..).visibility`.
- **Snow on the ground goes through the ground's own colours.** `palette::ground`
  already whitened the high flats above a fixed 700 m; that height is now
  `palette::snow_line()`, which the weather sets each frame (in 20 m steps), and the
  ground mesh's key includes `palette::snow_step()` so it is redrawn when the line
  moves. The map's relief picture is made once and keeps the 700 m line. Trees and
  building models don't whiten.
- **The lightning drawn is not the simulation's.** `world.strikes(..)` is for rules and
  omens and runs on game time. The window makes its own flashes on its own clock, as
  often as the storm overhead says, because the game can run at an hour a second.
- **The foliage shader has its own sway.** To make grass and trees follow the weather,
  feed `WeatherView::wind` (and `here.gust`) into `foliage.rs`'s sway uniforms.
- **The sea has no shader input**, so sea state isn't drawn: there is nothing to make
  rougher. It needs a sea material with a wave-height input; `WeatherView::here.sea`
  (0 calm .. 1 heavy surf) is the number to feed it.
- Cloud shadows drifting over the land were left out (not cheap without a custom shader).

### Quality settings and cost

The graphics panel (O) has a new row, **Rain, snow and fog**: off / low / medium /
high, saved in `settings.txt` as `weather = ...` (default medium). It maps to
`view::weather::Quality`:

| Setting | Drops in the heaviest fall | Fog sheets | Points across a sheet | Sheets redraped | Bolts |
|---|---|---|---|---|---|
| off | none | none | - | - | no (the flash stays) |
| low | 1,500 | 1 | 41 | once a second | yes |
| medium | 4,200 | 2 | 65 | twice a second | yes |
| high | 6,500 | 2 | 97 | four times a second | yes |

The light, the sky, the haze, wet ground and the snow line cost nothing and are on at
every setting, so the weather always reads. Medium is the look of the stage 2 and 3
screenshots.

Cost, measured as the time the weather's own systems take on the CPU each frame
(the Weather panel's last line shows the running average and the worst recent frame):

| Weather shown | low | medium | high |
|---|---|---|---|
| Sea fog, on average | about 0.02 ms | 0.06 ms | 0.4 ms |
| Sea fog, a frame that does a slice of redraping | about 0.3 ms | 0.4 ms | about 0.8 ms |
| Downpour (every drop is placed afresh each frame) | 0.2 ms | 0.5 ms | 1.4 ms |
| Clear, overcast, anything with nothing falling and no fog | under 0.05 ms | under 0.05 ms | under 0.05 ms |

These are from the build machine: two slow cores, which the software renderer is
also using, so single frames there are sometimes held up for a few milliseconds by
the renderer itself (the "worst recent frame" figure shows that as 2 to 4 ms). A
desktop PC should be several times quicker. At the default (medium) the weather's own
work comes to about half a millisecond a frame at its heaviest here, against the 2 ms
a frame the brief allows.

The fog sheets are the only part that was lumpy: redraping them means asking, for
every point of the sheet, where the lowest ground round about is. That answer is now
kept while the sheets stay put, and a redraping is spread over several frames
(`groundfog::Job`), so no one frame pays for a whole sheet.

What this does not measure is the graphics card's side (two see-through sheets over
the view, a few thousand small see-through quads). The build machine has no graphics
card, so that has to be read off Laz's PC: the fps readout with the weather forced to
sea fog or a downpour (U), against the same view with "Rain, snow and fog" off.

## Sound slots (no files yet)

`WeatherView::sound` is a `sound::Mixer`: `volumes` (one per loop, 0..1, in
`sound::LOOPS` order), `muffled` (the listener is under a roof), `played` (one-shots to
start this frame, with volume) and `due` (thunder waiting for its sound to arrive).
To play them: load each slot's file, keep the loops running at their volumes, start
each one-shot in `played`. Bevy's audio feature is not switched on in `Cargo.toml`.

| Slot | File under `assets/` | Kind | Suggested length | When it is heard |
|---|---|---|---|---|
| light rain | `sounds/weather/rain_light.ogg` | loop | 30 s | drizzle to steady rain; gives way to heavy |
| heavy rain | `sounds/weather/rain_heavy.ogg` | loop | 30 s | from steady rain up to a downpour |
| low wind | `sounds/weather/wind_low.ogg` | loop | 40 s | a breeze, 2 to 12 m/s |
| high wind | `sounds/weather/wind_high.ogg` | loop | 40 s | a strong wind, 8 to 20 m/s |
| gale | `sounds/weather/gale.ogg` | loop | 40 s | 15 m/s and up |
| calm surf | `sounds/weather/surf_calm.ogg` | loop | 45 s | within about 800 m of the shore, quiet sea |
| heavy surf | `sounds/weather/surf_heavy.ogg` | loop | 45 s | within about 800 m of the shore, rough sea |
| snow wind | `sounds/weather/snow_wind.ogg` | loop | 40 s | snow falling in a wind |
| thunder, near | `sounds/weather/thunder_near.ogg` | one-shot | 6 s | a strike within 1.5 km, after its travel time |
| thunder, far | `sounds/weather/thunder_far.ogg` | one-shot | 10 s | a strike further off |
| rain on a roof | `sounds/weather/rain_on_roof.ogg` | one-shot, restarted | 20 s | indoors while it rains |

Indoors (the camera following a squad that has all gone inside, read from
`Squad::inside`) every loop drops to 35% and `muffled` is set. From far overhead
everything fades to 30%.

## What the weather does (`weather_effects`) and where to connect it

`world.weather_effects(pos, t)` returns an `Effects` for someone **standing in the
open** at that spot and moment (under a roof, ignore it). Like the weather it is a
plain lookup: ask about any place and time, in any order. Every number behind it is in
`data/weather/climate.ron` under `effects`, each with a comment. The Weather panel's
"What it does" tab shows them live.

| Field | Means | Suggested place to connect |
|---|---|---|
| `sight_mult` (0.08..1) | How far people see: closed in by fog, rain and snow (full sight while the air is clear to 150 m), dimmed a little by a storm's gloom and by cloud on a night with no moon. The hour's own light is **not** in it. | `stealth.rs` where `sight = SIGHT * self.visibility_of(pid) * sharp` (and `buildings.rs:322`): multiply by it, using the watcher's spot. `Battle::light_at` for fights is separate (rule 11): a fight could carry the multiplier taken at its start. |
| `hearing_mult` (0.2..1) | How far people hear: less in rain, wind and falling snow. | `stealth.rs` where `hearing = HEARING * self.noise_of(pid) * ...`: multiply by it. |
| `ranged_accuracy_mult` (0.2..1) | Bows and thrown things: less in wind (gusts count), rain, snow. | `combat.rs`, the hit chance: `p_hit = ... * far * dark`: multiply by it when `shot`. Take it once when the fight starts (`weather_effects(fight.pos, fight.start)`) and keep it on the `Battle`, so a far fight and a near one are the same fight (rule 1). |
| `travel_speed_mult` (about 0.45..1) | Walking pace: soaked ground, lying snow, a gale on open ground (less in the sheltered low country). | `Leg::along` (`group.rs`): divide each stretch's effort by the multiplier at the stretch's midpoint **at the leg's departure time**, so the schedule stays worked out in advance (rules 3 and 10). On a paved road the mud part should not count: the caller knows the ground (`Terrain::ground`). Wind direction is not in it; `Weather::wind_to` is there if head and tail winds are wanted. |
| `slip_risk` (0..1) | How treacherous stone and steps are: wet, snowed on, iced (wet and below freezing). Only meaningful where the footing is rock or stairs. | Climbing in `condition.rs` (the by-the-metre cost), stairs and cliff paths, a fall chance in fights fought on wet stone. Roll it keyed (`Rng::from_keys`), never from a running generator. |
| `exposure` (0..1) and `feels_like` (deg C) | Cold and wet together: 0 at 10 deg C and up, 1 at -15 and below, after wind chill and rain. | A future warmth need in `condition.rs`: a rate that changes when the weather does, so settle the squad's condition at the forecast hours where it crosses a stage (the forecast is free: solve for the hour, as with hunger). Tents, fires and roofs cancel it. |
| `fire_spread_mult` (0.03..1.75) | How readily fire spreads: almost none on soaked ground in rain, more in a dry wind. | Base-building fire when it exists. Until then: how long `Does::Burning` lasts out of fights, whether a campfire or standing torch stays lit in a downpour (`torch::Flame::out_at` could end early when `rain > 0.5`), whether a wet person can light one (`elements.rs`). |
| `boats_can_sail` / `sea_danger` (0..1) | Whether small boats put out, and how dangerous the water is: surf, wind, fog, lightning. `boats_can_sail` is exactly `sea_danger < 0.5`. | The dawn boats in `society.rs`: at `DAWN`, ask at the stilt village (`Settlement::stilts`) and scale that day's `catch` rate, or keep them in. The forecast lets sailors decide the night before (`world.forecast_at(stilts, t, 12)`). Ask at a spot on the shore or the water: inland the sea number fades out. |
| `crop_growth_mult` (0..about 1.2) | How fast crops grow this hour: season (spring 1, summer 1.15, autumn 0.6, winter 0.1), warmth (none at a frost), water (less on dry ground, a little less in a downpour), none under snow. | Gardens in `society.rs` (`food_plan`, the gardens' share of the day's food) at the `DAWN` tally: use `weather::crop_growth_over(&terrain, seed, town.pos, yesterday_dawn, dawn)`. Seasons so far only move working hours, so this would be the first thing to make the harvest follow the year. |
| `outdoor_work_ok` | Whether people carry on working outside: not in heavy rain, a gale, heavy snow, bitter cold, thick fog or lightning. | `World::day_plan` (`routine.rs`) and `at_work`: an outdoor job's hours are lost while it is false. Day plans are worked out from the clock (rule 16), and so is this: the plan for a day can read the day's weather hour by hour up front. |
| `shelter_seeking` (0..1) | The chance someone with no pressing reason to be out heads indoors: nobody for drizzle, everybody for a downpour, a gale or a storm. | `day_plan`: for each person and hour, one keyed roll (`Rng::from_keys(&[seed, pid, hour, tag])`) against it moves them from the street to home or the hall. A people's leaning could scale it (rule 15: a number in `culture::PROFILES`, not "Horaro don't mind rain" in code). |

Things to keep right when connecting:

- **Pass the event's own time**, never `self.time`, for anything on the timeline.
- **Ask once per decision.** A fight, a leg or a day's work should read the weather at
  one stated moment (its start, its departure, each hour of the plan), not whenever the
  code happens to run, or the bands stop agreeing (rule 1).
- **`effects_of(&weather, t)`** gives the same answers for a `Weather` you already have.
  Only the walking pace differs slightly from `weather_effects` near a region border
  (it uses the main region's openness to the wind, not the blend).
- Sight in thick fog: the *rules* number can be as low as 0.08 (about 6 m of the usual
  75). The window never draws it that thick where the camera looks (see above).

## Omens

Weather unusual enough that people might read something into it. An omen is real
weather, found by looking the day up, not something rolled on the side: on a day with
"a dead calm" the wind from `weather_at` really is under 1.5 m/s for six hours.
**Nothing posts or reads them.** The panel shows the day's.

| Omen (`OmenKind`) | What counts | Where | Rarity | Measured, times a year |
|---|---|---|---|---|
| `WinterThunder` | a thunderstorm whose thunder begins within 5 days of midwinter | every region | very rare | 0.09 to 0.14 |
| `NoonFog` | fog 0.6 thick or more still lying at noon, within 4 days of midsummer | every region (hardly ever the plateau) | rare | 0.47 to 0.52 |
| `ShoreSnow` | snow falling at sea level for three hours together; counted on the first day of a spell | open sea, exposed coast, lowland | very rare | 0.15 to 0.17 |
| `StormRun` | three storm days in a row; counted on the third | every region | very rare | 0.13 to 0.18 |
| `DeadCalm` | wind under 1.5 m/s for six hours together between 06:00 and 20:00 | open sea, exposed coast | rare | 0.54 to 0.71 |
| `LandmarkStrike` | a lightning strike of some size within 150 m of a town's centre | wherever a town is | uncommon | 2.4 to 2.6 (whole country) |

- **Rarity bands** (in the data file): uncommon 1 to 4 times a year, rare 0.25 to 1,
  very rare 0.05 to 0.25. The year is the placeholder 48 days, so "very rare" is
  roughly once in four to twenty years. For the first five the count is for one
  region where the omen can happen (the average of those regions); for lightning on a
  landmark it is for the whole country (any one town is struck far less often). The
  test `each_omen_comes_as_often_as_its_rarity_says` holds every omen inside its band;
  `headless omens [years] [seed]` prints the rates and the first date of each.
- **If the year's length changes**, the rates per year change with it: re-run
  `headless omens` and retune the thresholds in `climate.ron` (`omens`).
- **To make the wind able to drop to nothing** the wind got one new ingredient: a lull
  that comes over the whole country now and then (`lull_days`, `lull_share`,
  `lull_floor` in the climate file). Winter thunder's share was raised from 4% to 12%
  of winter storms so that thunder in midwinter is possible without being common.
- **Landmarks** are the towns (`world.landmarks()`: name and centre). Anything else
  with a name and a place can be added to that list when it exists (shrines, the
  massif's peak).

An `Omen` has `kind`, `rarity`, `region`, `at` (the moment it shows itself, inside the
day asked about), `pos` and `landmark` (for a strike: where, and which landmark in
`world.landmarks()` order), and `text` (a plain sentence: "Snow fell right down to the
shore.", "Lightning struck <town>.").

How to connect them (suggestions; none of this is built):

- **When to look.** Omens can be known ahead, so treat them like ambushes (rule 7): at
  each `DAWN` tally, for each town, ask `world.omens_at(town.pos, day)` for the day
  just starting and put each on the timeline at its `at`. `omens_at` gives the omens of
  the town's own region, timed as the weather reaches that spot (`World::day_of` and
  these days agree: day 0 starts at midnight). Cost: next to nothing on an ordinary
  day; on a day of thunder, looking through the day's lightning takes some
  milliseconds, once per call, so ask once a day per region and share the answer
  between that region's towns.
- **Rites** (`law.rs`): `bad_omen(town, t)` is the hook ("a failed rite, or (later)
  one reported or faked by others"). Whether a given omen is *bad* is the priestesses'
  reading, which is Part 3's to decide: one keyed roll per town, day and omen kind,
  leaning on the town's customs (rule 15), with rarer omens weighing more
  (`omen.rarity`). The weather only says what happened.
- **Gossip and talk**: `data/lines/FORMAT.md` already lists a `rain` tag that never
  fires; it can read `world.weather(pos).rain > 0.03`. An `omen` tag (plus the kind)
  could be true in a town for a few days after one, and `news.rs` could carry it to the
  next town as it carries crimes.
- **The journal**: `self.say(omen.at, omen.text)` when the squad is in that region.

## Lines added to shared files

| File | Line |
|---|---|
| `src/sim/mod.rs` | `pub mod weather;` |
| `src/view/mod.rs` | `pub mod weather;` |
| `src/view/app.rs` | `.add_plugins(super::weather::WeatherPlugin)` |
| `src/view/app.rs` | `ui` takes `mut weather: ResMut<super::weather::WeatherView>` |
| `src/view/app.rs` | `super::weather::map_colours(&c, ctx, game, &mut weather);` after the map is drawn |
| `src/view/app.rs` | `panels.extend(super::weather::panel(&c, game, &mut weather, click));` after the settings panel |
| `src/view/palette.rs` | `SNOW_LINE` with `snow_line()`, `snow_step()`, `set_snow_line()`; `ground()` reads `snow_line()` where it had 640..760 |
| `src/view/scene.rs` | the ground mesh's key includes `palette::snow_step()` |
| `src/view/settings.rs` | `Settings::weather: Level` (default medium): the field, its `settings.txt` line, and the "Rain, snow and fog" row in the graphics panel (12 lines) |
| `src/view/ground.rs` | the land's shader multiplies its colour by the material's `base_color` (one line; white unless the weather wets it) |
| `src/bin/headless.rs` | the `weather` and `omens` commands and the usage line |
| `tests/consistency.rs` | `mod weather;` (the tests are in `tests/weather/mod.rs`) |

Not touched: `Cargo.toml`, `save.rs` (`save::FORMAT` is unchanged: weather keeps no
state), terrain generation, buildings, README, CLAUDE.md. The help line at the bottom
of the window doesn't list **F7** (the Weather panel) or **U**; add `F7: weather` there
when merging.

**Found while merging main in (not weather's doing):** the main branch's commit "The
land is made of the buildings' stone and grass" loads `assets/textures/roduro_grass.png`,
`roduro_grass_n.png` and `roduro_stone.png`, but they are not in the repository:
`.gitignore` has `*.png` and only lets `assets/*.png` back in, so `assets/textures/*.png`
were never committed. On a fresh checkout the land does not draw at all (the material
waits for its textures). Add `!assets/textures/*.png` to `.gitignore` and commit the
three files from the machine that has them.

## For CLAUDE.md, when merged (suggested rule)

> **Weather is looked up from the place and the time.** `weather_at(pos, t)` is a pure
> function of the seed, the region and `t` (`sim/weather/`): never tick it, never store
> it in `World`. Every number is in `data/weather/climate.ron`. Regions have their own
> climates and share the big weather, which reaches a place `lag(pos)` after the sea's
> edge. Anything on the timeline that reads the weather passes the event's time.
> What the weather does to people and things is `weather_effects(pos, t)`, and unusual
> days are `omens(region, day)`: both looked up the same way. A rule that reads them
> asks once per decision (a fight's start, a leg's departure, an hour of a day plan).

Keys: **F7** the Weather panel, **U** the next kind of forced weather (Shift+U back).

Screenshot flags: `GAHT_WEATHER=1` (the panel; `folded` for just its headline; `off` for
none), `GAHT_WEATHER_HOURS=h` (look h hours ahead or back),
`GAHT_PRESET=clear|overcast|drizzle|seafog|downpour|gale|thunderstorm|snow` and
`GAHT_PRESET_STRENGTH=0..1` (forced weather), `GAHT_FLASH=1` (a lightning strike timed
for the picture), `GAHT_WEATHER_TAB=effects` (the panel's "What it does" tab),
`GAHT_WEATHER_QUALITY=off|low|medium|high` (draw at that setting whatever
`settings.txt` says), `GAHT_WEATHER_COST=1` (print the cost every 150 frames). With `GAHT_VIEW=map` the land is coloured
by its weather. Good views: `GAHT_PRESET=seafog GAHT_NUDGE=5200,1500 GAHT_ZOOM=1400
GAHT_PITCH=0.55` (fog lying below a hill), `GAHT_PRESET=seafog GAHT_ZOOM=45
GAHT_PITCH=0.14 GAHT_YAW=2.2` (in the fog at eye height).
