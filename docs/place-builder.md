# The place builder

Laz, 14 Oct 2026: no more world building by Claude. He hand-authors places,
people and land; the game simulates what he authored. The place builder is
the authoring tool for a place: a mode of the Map Marks page (phone and iPad
first) opened from a mark on the world map ("Build it out"), top-down 2D.

## What it does

- **Brushes** (one finger paints; pan and zoom are off while a brush is
  picked, the Hand tool puts them back): ground paint (grass, scrub, dirt,
  sand, mud, gravel, rock, snow, water), trees / bushes / rocks (stamps, by
  density), raise / lower ground (contour lines every metre; brush size and
  softness sliders), erase.
- **Roads and paths**: tap points, finish; width by kind.
- **A tray** of buildings and props drawn to scale (not icons), by people and
  group, that minimises; drag one onto the ground, then move / rotate / scale
  it with handles, or remove it.
- **Info**: the place's capacity (beds in its housing) and the roles its
  workplaces give (a smithy: a smith; a block with a loom: weavers), read
  from the catalogue, shown live.
- **Save**: to the page's store as you go; **Save to GitHub** writes the
  place file into the repo (a key kept in the browser, never in a file) when
  Laz clicks it; Export hands the file over by hand until then.

## The place file: `assets/world/places/<town key>.json`

Metres from the place's centre, x east and y south (the sim's convention);
rotation in degrees clockwise seen from above, 0 = the door looks east.

```json
{
  "format": 1,
  "key": "last_wave_outpost_southwest",
  "title": "Small Last Wave outpost",
  "notes": "...",
  "cell": 2.0,
  "size": [120, 90],
  "origin": [60, 45],
  "ground": "gggggffff...",
  "height": "AAAABBBC...",
  "foliage": [{ "k": "tree_broad", "x": 12.5, "y": -30.2, "s": 1.0 }],
  "roads": [{ "k": "road", "w": 3.0, "pts": [[0, 0], [20, -5]] }],
  "things": [{ "id": "t1", "k": "qotiro_court", "x": -20.0, "y": -2.0, "rot": 5, "s": 1.0, "label": "Chieftain's quarters", "room": 12, "note": "" }],
  "saved": "2026-10-14T12:00:00Z"
}
```

- `cell`, `size`, `origin`: the rasters' grid; `origin` is the cell at the
  centre. Rasters are one character a cell, row-major from the top left.
- `ground`: a letter a cell: `g` grass, `s` scrub, `d` dirt, `n` sand,
  `m` mud, `v` gravel, `r` rock, `w` snow, `~` water, `.` untouched (the
  land's own).
- `height`: a letter a cell, `A`..`Z` then `a`..`z` = -26..+25 metres from
  the land's own height at the centre, `.` untouched.
- `foliage`: stamps, each a catalogue key (`tree_broad`, `tree_conifer`,
  `bush`, `rock_small`, `rock_large`, …), position and scale.
- `roads`: polylines, kind `road` | `path` | `cobbles` | `stairs`, width in m.
- `things`: buildings and props by catalogue key; `room` is the beds the
  builder worked out (editable); `label` and `note` are Laz's.

## The catalogue: `assets/world/catalogue.json`

Written by the game (`headless catalogue`) so the tool and the sim agree on
every shape and size. One entry per thing:

```json
{ "key": "qotiro_court", "name": "Courtyard house", "group": "building", "people": "qotiro",
  "kind": "QotiroBlock", "shape": "rect", "depth": 15.0, "width": 12.0, "size_range": [15, 30],
  "storeys": 2, "beds": 5, "jobs": ["Weaver"], "stations": [], "scalable": true, "rotatable": true,
  "glyph": "qotiro_block", "door": 0.0, "blurb": "..." }
```

- `group`: `building` (a `layout::VARIANTS` entry), `base` (a
  `base::BUILDINGS` entry: huts, palisades, sheds), `workplace` (a
  `jobs::PlaceKind` footprint: fields, a smithy, a market…), `pen` (by
  species), `prop` (rocks, a timber stack, a garden plot, a bedroll, a tent,
  a campfire, a standing torch, a fence run, a well…), `foliage` (trees and
  bushes), `ground` (the paint classes).
- `depth` runs along the facing (the door end), `width` across; both at
  scale 1. `beds` counts the variant's beds and bedrolls. `jobs` are what
  `layout::keeps` says the building keeps; `stations` its crafting stations.
- `glyph` names how the page draws it (a drawing routine per family, not an
  icon: tiers, courtyards, decks, fences at the real size).

## Who does what

- **Primary** (the map page): the builder mode, its gestures, the tray, the
  handles, capacity and roles from the catalogue, save / export / GitHub.
- **Second** (the game): `headless catalogue` → `assets/world/catalogue.json`
  (every variant, base building, workplace, pen species, prop, foliage kind,
  ground class, with real footprints and a `glyph` family); the place-file
  importer (`sim/place.rs`): ground and height rasters under the town as
  terrain layers (`Terrain::authored`), the foliage stamps as the drawn
  trees (replacing the plan's ink raster), roads as `forge::Way`s, things as
  the plan's buildings / workplaces / pens / solid props; capacity and roles
  checked against the sim (`tests/places.rs`); a review of the catalogue for
  anything a place needs that the game has no shape for (then a variant or a
  prop). The first file to import is the outpost, rebuilt in the tool.

## The importer (built: `sim/placefile.rs`)

A place file for a town (its key is the town mark's) is read at world
creation and turned into a town plan (`plan.rs`), so it is laid out as
placed: buildings by variant at their spot, facing and scale (`room`
decides how many sleep there; the catalogue's beds if left out),
workplaces, pens (a pen's scale changes how many it holds, by area), and
the land (`PlaceLand`, kept on the plan and saved with the world): ground
paint and height laid under the town (each height letter metres above or
below the land's own height at the centre), rocks (stamped or placed) as
rocks of the land, trees and bushes drawn exactly where stamped with no
others growing inside the built area, roads painted on the ground. A place
file overrides a plan read from a sketch for the same town. What the game
can't stand up yet goes to the plan's `later`, with where it was put.
`FORMAT`: `Plan::land` is new saved state.

## The land under a place (built: `sim/grounds.rs`)

The builder opens on Laz's map round the mark and glides down into the
place, over the land as the game makes it there: `headless grounds <dir>
[seed]` writes `<dir>/<key>.json` for every mark (towns and landmarks) and
`<dir>/index.json` (mark id → the game's key; the map page reads it, so a
relabelled or duplicate mark opens the right place). Each file has the
place's own grid (the builder's 2 m cells, 160 × 120, centred on the town's
spot) and a ring round it (8 m cells, 1280 × 960 m): a ground letter a cell
(grass, scrub, sand, rock, road, river or lake, sea...) and the height (dm
from the centre's), plus the trees and bushes the land itself grows in the
place (`sim/plants.rs`, the one rule the window's foliage uses too) and the
map's own rocks. They're published beside the page as `grounds/`.

- Untouched cells show the land's own ground and height; contours and
  shading follow the real slopes, and raise / lower start from them.
- A new place starts with the land's own trees and bushes as stamps (inside
  a built place only stamps grow, so these are what will stand there):
  erase what you don't want; "Land's trees" puts back any that are missing.
- A town with a plan or place file stands exactly on its mark's spot
  (`worldgen::town_spot`: ashore and off any river) in every world, so the
  layout sits on the land it was drawn on. (A town nobody laid out still
  moves to the levellest ground near its mark, which the seed decides.)
- The land's broad shape is the map's and the same in every world; its
  small relief (a few metres), scrub patches and woods follow the world's
  seed. The files are from one world (the file says which); inside a place,
  painted ground, set heights and stamped trees are the same in every world.

Re-run the export and republish `grounds/` whenever the map, the marks or
the land's rules change.

## Mobile first

44 px targets; the tray a bottom sheet that folds to a tab; brush sliders in
a pill above the tray; two-finger pinch stays available only with the Hand
tool (a picked brush pauses all movement, as asked); undo; a thing picked
shows four handles (move, rotate, scale, remove) big enough for a thumb;
contour lines and ground colours readable in daylight.

## What a place needs that the game has no shape for yet (the second's review, 14 Oct)

The catalogue (`assets/world/catalogue.json`) lists everything the game can
stand up today, with `missing: true` on the props it can show in the tray
but not build. A place that uses a missing thing still imports: the thing
goes to the plan's `later` list with where it was put, so nothing is lost,
and it appears in the game once its shape exists. Here is what the outpost
sketch asks for that has no shape, and what each would take. None is built
yet.

**Buildings.** A building in the game is a *variant*: its outline, inner
walls, furniture and containers in one table (`layout::VARIANTS`), drawn
from the same numbers it's solid by. A new building is one new line there
plus its glyph on the page. The game has grown Roduro round houses, stepped
Qotiro blocks, a temple and a hall, and Horaro stilt homes; it has no
timber building at all, and nothing with more than one floor you can be on.

| Asked for | What it would take |
|---|---|
| **Timber two-tier house, a workshop below and the living above** (the shield maker, the bowyer, the leatherworker, the herbalist's stillroom) | A timber people-agnostic kind (`BuildingKind::Timber`), its look in `view/interiors.rs`, and one variant per trade (shield bench, bow bench, leather bench, stillroom with a desk or alchemy table). The two tiers can be shown as storeys from outside, as Qotiro blocks are now; walking upstairs would be new (the game has one floor per building). |
| **Timber hall, long and low, with an antechamber you pass through** | A variant whose inner wall makes a small front room with a second door into the hall. The walls and doors system already does inner walls; it needs an inner *door* (today inner walls are walls with gaps). |
| **Stone house with a large open courtyard and the forge in it** | A Qotiro-style variant with an open courtyard (the `qotiro_court` has one) and the forge's furniture (forge, anvil, workbench) standing in the yard instead of a room. Mostly furniture placement; the forge's fire is already drawn. |
| **Three-tier stone quarters for twelve priestesses, closed to outsiders** | The shape: a taller quarters variant (twelve beds). The closing is a rule, not a shape: a building flag the trespass check (`law.rs` `mind_the_guests`) reads, so walking in by day is trespass too. |
| **Two-tier smokehouse below, living above, a chimney smoking** | As the timber house, with a smoking-rack piece of furniture and smoke drawn from the chimney (the flame system has the fire; smoke would be new drawing). It could also be a station for preserving food (no smoking recipe exists yet). |

**Props.** A prop that stands in the world has to be in the solid list
(`solid.rs`: what blocks a walker and what's drawn come from the same
numbers). Each of these is a new kind there, plus its drawing:

| Prop | What it would take |
|---|---|
| **Timber stack** | A solid pile (the woodlot already draws one as part of the workplace); on its own it needs a place in the town's layout. Could hold timber the town's store counts. |
| **Garden plots** | Town gardens already exist in town life (`TownLife::gardens`, a gardener tends them) but aren't placed things; a plot from the place file would become one of those gardens at that spot, fenced as drawn. |
| **Tanner's vats and stretching frames** | A solid outdoor yard prop; tanning is done in a workshop now, so the vats would be the tanner's workplace (a new place kind beside the workshop) or just a prop. |
| **Fence run** | Fences exist only round pens, yards and palisades. A free fence line would be a solid line with an optional gate, like the palisade's segments. |
| **Well, stall, cart** | A town well (the squad's base well exists: the same shape could stand in a town); a market stall on its own (stalls exist only as a market's counters); a cart (nothing like it yet; it could be the start of carts that move). |
| **Young tree** | Foliage only has full-grown broadleaf and conifer trees; a small one is the broadleaf drawn smaller (the importer already draws `tree_small` that way). |
| **Painted water** | The ground has no fresh water yet: a pond or stream painted in the builder stays the land's own ground. This waits for the rivers-and-bog work (water as ground that stops a walker). |
| **Tents, campfires, bedrolls, standing torches in a town** | These exist, but as the squad's own things (they burn down, get packed up). A town's would need a fixed kind that just stands there (a fire that's always lit at night, a tent someone lives in). |

**Roads.** A road drawn in the builder is painted onto the ground (dirt,
or gravel for cobbles) and kept with the place; the game's own roads (the
ones travellers walk between towns, and the faster pace on them) don't read
it yet. Joining them would make a drawn road count as road underfoot.

**Squad base buildings in a town** (huts, sheds, palisades) are a squad's
outpost things with their own building and upkeep rules; placing one in a
town is listed as `later`.

## One map: zoom to build (Laz, 14 Oct evening)

Laz: "all I have to do is zoom and I can build, no need to click the build
button", and "I want to be able to work without an internet connection too."

**The design.** One page, one canvas, one continuous zoom from the whole
continent down to a doorway:

- Far out it is the map page as now: Laz's map, the marks, the roads, the
  people screen. Zoom in past about 1 m a screen pixel and the view becomes
  the builder: the game's land under the paint, the place's things, the
  tray and the tools. No button. Zoom out and it is the map again.
- **The place under the view** is the active one: the mark nearest the
  middle of the screen whose area holds the centre. Its name and capacity
  sit in the corner; paint and things go into its file. Between places the
  tools are greyed and the corner says "zoom in on a mark to build, or add
  one here" (a tap adds a mark, which is then editable at once).
- Areas by kind (hamlet 120 m to capital 600 m across), and a place's land
  comes with it (the game writes the land under every mark at that size).
- **Local first.** Every change is kept in the browser (IndexedDB) the
  moment it is made; nothing needs the connection. **Offline** by a service
  worker that caches the page, the map, the catalogue and every mark's land
  on first visit, so the page opens with no signal at all. That needs a real
  web origin: the page moves from the claude.ai artifact to **GitHub Pages**
  (`lazmcspaz.github.io/Gahturiyu-Sim/map/`), served from the repo. The
  artifact stays as a mirror until the Pages site is up.
- **Save** (a button, when online) commits the changed place files, marks
  and rosters to the repo through GitHub's API with a token kept only in
  that browser (a github.io page may call GitHub; the artifact viewer may
  not). Export / Import of a single file stays for the days with no signal.
- One world file for the marks and roads (`assets/world/marks-export.json`),
  one file a place (`assets/world/places/<key>.json`), one roster a place:
  the same files the game reads, so Save is the whole hand-over.

**Who does what.**
- *Primary:* the unified page (`tools/map/`): continuous zoom, the active
  place, local-first storage, the service worker, Save to GitHub, Import /
  Export; the Pages deployment (a workflow that publishes `tools/map/` with
  the data beside it).
- *Second:* the game side: `headless grounds` at the new scale and with
  the area by kind (hamlet 120 × 90 m … capital 600 × 450 m), written into
  `tools/map/grounds/` in the repo so the site carries them; `headless
  site` (or a script) that refreshes everything the site needs
  (catalogue, grounds, marks) in one go; the importer reading areas of any
  size; the three weather tests that read the map's size
  (`tests/weather/mod.rs`) and `everyone_goes_in_when_the_weather_drives_them_in`
  after the change to 26.5 m a pixel.

**Built (second, 14 Oct late):**
- `headless site [seed]` refreshes everything the site serves beside its
  page, the same bytes every run: `tools/map/catalogue.json`,
  `tools/map/grounds/` (every mark's land, `index.json` mark id → key,
  stale files cleared) and copies of `assets/world/marks-export.json` and
  `assets/world/places/*.json` in `tools/map/data/`; it reads every roster in
  `assets/world/people/` and exits 1 if one is bad or names no mark.
  `headless grounds [dir] [seed]` writes only the land (to the site unless
  told).
- Areas by kind (`grounds::AREAS`): hamlet 120 × 90 m, outpost 160 × 120,
  village 200 × 150, town and fort 320 × 240, city 450 × 340, capital
  600 × 450, any other mark 160 × 120; 2 m cells, origin at the middle cell;
  the ring round it has as many cells, each 8 m. The importer already reads a
  place file of any `size`/`origin` (`tests/grounds.rs` checks a hamlet's).
- `python tools/map/pull.py [EXPORT.txt] [--no-site]`: after a save lands,
  `marks_to_towns.py` then `headless site`; given a page export, each
  `// ---- assets/world/people/<key>.ron` section is written to its file
  first. `tests/grounds.rs` fails if `tools/map/grounds/index.json` doesn't
  match the marks.

## The game's land, photographed (built: second, 14 Oct evening)

Laz, from the site on his phone: the game's land showed only round each mark
("it should cover the whole screen"), it didn't match his map ("look again"),
and it was blocky; "generate the map in game with no cities and such, then use
a high resolution image of that as the detail level of the map".

**The land now reads his map faithfully** (`sim/atlas.rs`, `sim/terrain.rs`,
`sim/plants.rs`, `view/palette.rs`):
- The sea is flooded in from the map's edge across any blue, stopped by his
  dark shore line (`sea_from_the_edge`): the pale shallows had read as land and
  the line as a river, so every coast stood ~300 m out to sea.
- Edges between kinds of ground curve: each class's share is softened over a
  few pixels once (`Atlas::soft`, `SOFT_PASSES`) and read back with a fixed
  wobble (`EDGE_WOBBLE`); `class_at` is the biggest share, `share`/`blend`
  give the soft values. The relief's strength blends across edges (no little
  cliffs at a forest's edge). The shore is blurred a pixel (`SHORE_SMOOTH_PX`)
  and land rises over its first `SHORE_RISE_M` (25 m), so a cliff's top follows
  the shore's curve, not the map's 26.5 m pixels.
- His forest grows woods (`plants::wood` follows `Class::Forest`; elsewhere at
  most `OPEN_WOOD`), and the ground's colour follows his classes in the game
  itself (forest floor, Overgrowth, bog, rock, snow, `palette::ground`).

**The pictures** (`view/maprender.rs`, `tools/map/render.py`): the game window
in a map-render mode (`GAHT_MAPRENDER=<dir>`) photographs the land from
straight above with an orthographic camera, no towns, people or panels, a
fixed north-west sun: the whole map at 4 m a pixel (open sea skipped) and every
mark at 1 m, its place left without plants (only stamps grow in a built place;
the site draws them). `render.py` runs it and packs the PNGs as WebP into
`tools/map/render/` with `index.json` (name, level, metres a pixel, the world
box). The page (`drawRender`) draws them in world metres: the 4 m level from
`RENDER_LAND_Z`, the 1 m level from `RENDER_MARKS_Z`; over a rendered place only
the paint is drawn (untouched cells clear) and the ring isn't. It needs a
graphics card, so it runs on Laz's PC (a window open for about half an hour);
re-run it when the land changes.

## The editor in the game (built: second, 14 Oct evening)

Laz: the same authoring on the PC, in 3D with the game's placeholder
buildings; one authored world every new game loads; a building's use set by
hanging a sign on it.

- **F10, then Build** (`view/build.rs`): a tray of the catalogue (Roduro,
  Qotiro and Horaro buildings, workplaces, pens, props); what's in hand stands
  on the ground under the mouse, its outline red outside every place or at
  sea; click to put it down, click a placed thing to pick it, drag it, Q / E
  to turn it (Shift finer), [ ] to size it, Delete to take it away,
  right-click or Escape to let go. A place is a mark's area (its box); a
  town stands hidden while its place is open, drawn as the place file
  stands. Ctrl+S or Save places writes `assets/world/places/<key>.json`;
  Rebuild the world starts the world again from the saved places.
- **One authored world:** the Land tab's edits save to
  `assets/world/land.gmap` (`MapEdits::world_path`), loaded under every new
  game whatever its seed, and under the site's export and the render.
- **Signs** (`catalogue::SIGNS`, written for the site as
  `tools/map/signs.json`): a building thing's `use` is a sign key (smith,
  weaver, bench, trader, tender, food_shop, goods_shop, materials_shop, inn,
  healer, healing_house, teacher, exchange, letters, alchemist, scribe,
  workshop, shrine, guard, hall, kitchen, mess), or absent for a home. Each
  sign: `{key, name, variants, place}`. Where the building's people (its
  catalogue `people`) have one of the sign's `variants`, the building becomes
  that variant on its own footprint (a Roduro cottage signed `smith` is the
  Roduro forge). Otherwise the sign's `place` (a workplace key) is kept in
  the building: the importer puts that workplace at the building's middle,
  and a workplace inside a building is the building's (the game already
  knew: `solid::place_in_building`, nothing of its own stands or is solid;
  its workers work there); its signpost stands by the door unless the
  building hangs its own sign. No saved-shape change.

## Grow a draft (second, 17 Oct)

Laz asked for town layouts grown rather than spiralled, tried in the 2-D builder first
(the research and Laz's rulings: https://claude.ai/artifact/7CUVyK4p43TRjynH24Ewj2).
The place card's **Grow a draft** lays a town out in the place under the view, from the
mark's notes and label and the land (`tools/map/src/towngrow.js`, inlined by
`assemble.py`). It replaces the place's things and paths (Undo puts them back); each
press is another roll. What it does:

- **What the town grew on** comes from words in the notes or label ("fishing", "port",
  "lake" a harbour; "ford", "river", "bridge", "crossing" a ford; "hill", "cliffs" a
  hill; "crossroads", "market"; "road"; "fort", "keep", "castle"); with none, or when
  the land can't hold it (a harbour with no sea near), the land decides. "Small",
  "remote" shrink it; "spread out", "marshy" loosen it.
- **Where it began**: the quay, the higher bank of the crossing, a hill's shoulder, the
  road junction, or the flattest ground; a market square there, a well beside it. The
  landmark (a Qotiro temple in a Qotiro town, else a great house) on the highest flat
  ground near it, the top on a hill.
- **Streets** from the heart out along the land's roads (and to the place's edge), to the
  landmark and to the shore, by a path search that charges for climbing; **lanes** off
  them to good ground. Paths `road` 5 m and `path` 2.5 m.
- **Homes** on plots fronting every street, door to the street, packed near the square
  and thinning (some plots left empty) toward the edge. Each plot goes to the people
  who lean to it most (founders count 2.5×): Roduro the core, Qotiro the high and steep
  ground (they terrace), Horaro ashore near the water, Ṭaḍoro tents at the edges; trade
  houses (forge, loom room, bench room, trader…) on the main streets near the square.
  A coast gets stilt homes on the water near the quay, on a boardwalk (`path` 2 m).
- **Fields** (`field_plot`) along the roads beyond the last house ("farming" doubles them).
- **Walls** round towns, cities, capitals and forts, not villages and smaller unless the
  notes say "walled" / "palisade" ("unwalled" turns it off): stone pieces for a Roduro
  or Qotiro town (`rubble_wall`, noted "grown stone wall" for Roduro until the game has
  that piece), a palisade else; a gate where each main street crosses the wall or ends
  at it, watchtowers by a city's or fort's gates.

Homes by kind: hamlet 8, outpost 6, village 22, port 14, fort 12, town 50, city 110,
capital 160 (before "small"). Everything stays inside the place's area; the ring of
land round it (from `grounds/`) only tells it where the sea, rivers and roads are.
