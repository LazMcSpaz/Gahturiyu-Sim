# Weather: what exists and how to connect to it

Second agent's work, branch `agent2/weather`. This file grows with each stage;
this is the state after **stage 2** (stage 1: rules, regions, forecast, tests, numbers
panel; stage 2: light and sky, haze, ground fog, rain, wet surfaces, forced weather).

## What weather is

- **Looked up, never run.** `weather_at(place, time)` is worked out from the world
  seed, the place and the time. There is no weather state in `World`, nothing in
  `World::step`, nothing saved, and `save::FORMAT` is untouched.
- **By region.** Six kinds of country, read off the land (`sim/weather/region.rs`):
  open sea, exposed coast, sheltered coast and lowland, upland, mountain, plateau.
  Each has its own weather; borders fade over about 200 m.
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
| `weather::skies(seed, t)` + `weather::weather_with(&skies, &mix, height, floor)` | the quick way to ask about many places at one moment (same answers as `weather_at`) |

`Weather` holds: `kind` (and `label()`, `describe()`), `cloud`, `rain`, `snow`, `wind` (m/s),
`wind_to`, `gust`, `fog`, `fog_height`, `visibility` (m), `temperature`, `wetness`, `storm`,
`lightning` (flashes a minute), `sea`, `snowline`, `cloud_base`, `region`.

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
- **The sea has no shader input**, so sea state isn't drawn yet (stage 3 will note
  what it needs).
- Cloud shadows drifting over the land were left out (not cheap without a custom shader).
- Cost: about 0.2 to 0.5 ms a frame on the build machine's CPU (the panel shows it).

## Not built yet (later stages)

- Stage 3: storms' looks, lightning strikes, falling snow and snow on the ground, wind
  for foliage, sound slots.
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
| `src/bin/headless.rs` | the `weather` command (6 lines) and its usage line |
| `tests/consistency.rs` | `mod weather;` (the tests are in `tests/weather/mod.rs`) |

Not touched: `Cargo.toml`, `save.rs`, terrain generation, buildings, README, CLAUDE.md.
The help line at the bottom of the window doesn't list **F7** (the Weather panel); add
`F7: weather` there when merging.

## For CLAUDE.md, when merged (suggested rule)

> **Weather is looked up from the place and the time.** `weather_at(pos, t)` is a pure
> function of the seed, the region and `t` (`sim/weather/`): never tick it, never store
> it in `World`. Every number is in `data/weather/climate.ron`. Regions get their own
> weather; nothing moves between them. Anything on the timeline that reads the weather
> passes the event's time.

Keys: **F7** the Weather panel, **U** the next kind of forced weather (Shift+U back).

Screenshot flags: `GAHT_WEATHER=1` (the panel; `folded` for just its headline; `off` for
none), `GAHT_WEATHER_HOURS=h` (look h hours ahead or back),
`GAHT_PRESET=clear|overcast|drizzle|seafog|downpour|gale|thunderstorm|snow` and
`GAHT_PRESET_STRENGTH=0..1` (forced weather). With `GAHT_VIEW=map` the land is coloured
by its weather. Good views: `GAHT_PRESET=seafog GAHT_NUDGE=5200,1500 GAHT_ZOOM=1400
GAHT_PITCH=0.55` (fog lying below a hill), `GAHT_PRESET=seafog GAHT_ZOOM=45
GAHT_PITCH=0.14 GAHT_YAW=2.2` (in the fog at eye height).
