# Weather: what exists and how to connect to it

Second agent's work, branch `agent2/weather`. This file grows with each stage;
this is the state after **stage 3** (stage 1: rules, regions, forecast, tests, numbers
panel; stage 2: light and sky, haze, ground fog, rain, wet surfaces, forced weather;
stage 3: shared weather that comes in off the sea, lightning, snow, wind, sound slots).

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
- **Wet surfaces reach only `Mats::lit`** (ground, roads, our own building meshes,
  people). The GLB building models and the foliage have their own materials; to opt
  them in, read `Res<WeatherView>().wet` and darken / lower the roughness the same way.
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
- Cost: about 0.2 to 0.6 ms a frame on the build machine's CPU (the panel shows it).

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

## Not built yet (later stages)

- Stage 4: `omens(region, day)`, `weather_effects(pos, time)` and the table of where to
  connect each effect; quality settings.

Known gaps to connect later: `talk.rs`'s `rain` piece (CLAUDE.md, Part 5 notes) can read
`world.weather(pos).rain`; `law.rs`'s `bad_omen` hook will get `omens` in stage 4.

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
| `src/bin/headless.rs` | the `weather` command (6 lines) and its usage line |
| `tests/consistency.rs` | `mod weather;` (the tests are in `tests/weather/mod.rs`) |

Not touched: `Cargo.toml`, `save.rs`, terrain generation, buildings, README, CLAUDE.md.
The help line at the bottom of the window doesn't list **F7** (the Weather panel); add
`F7: weather` there when merging.

## For CLAUDE.md, when merged (suggested rule)

> **Weather is looked up from the place and the time.** `weather_at(pos, t)` is a pure
> function of the seed, the region and `t` (`sim/weather/`): never tick it, never store
> it in `World`. Every number is in `data/weather/climate.ron`. Regions have their own
> climates and share the big weather, which reaches a place `lag(pos)` after the sea's
> edge. Anything on the timeline that reads the weather passes the event's time.

Keys: **F7** the Weather panel, **U** the next kind of forced weather (Shift+U back).

Screenshot flags: `GAHT_WEATHER=1` (the panel; `folded` for just its headline; `off` for
none), `GAHT_WEATHER_HOURS=h` (look h hours ahead or back),
`GAHT_PRESET=clear|overcast|drizzle|seafog|downpour|gale|thunderstorm|snow` and
`GAHT_PRESET_STRENGTH=0..1` (forced weather), `GAHT_FLASH=1` (a lightning strike timed
for the picture). With `GAHT_VIEW=map` the land is coloured
by its weather. Good views: `GAHT_PRESET=seafog GAHT_NUDGE=5200,1500 GAHT_ZOOM=1400
GAHT_PITCH=0.55` (fog lying below a hill), `GAHT_PRESET=seafog GAHT_ZOOM=45
GAHT_PITCH=0.14 GAHT_YAW=2.2` (in the fog at eye height).
