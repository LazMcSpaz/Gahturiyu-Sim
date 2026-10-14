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

## Mobile first

44 px targets; the tray a bottom sheet that folds to a tab; brush sliders in
a pill above the tray; two-finger pinch stays available only with the Hand
tool (a picked brush pauses all movement, as asked); undo; a thing picked
shows four handles (move, rotate, scale, remove) big enough for a thumb;
contour lines and ground colours readable in daylight.
