# Town plans: a town from a drawing

Laz draws a place; Claude transcribes the drawing into a **plan file** and the
game lays the town out from it. The first one is the small Last Wave outpost
south-west of the Open Hand capital (`docs/plans/last_wave_outpost_southwest.png`,
14 Oct 2026).

## The method

1. **Laz draws** the place: buildings as rectangles with a note each (what it
   is, how many tiers, timber or stone, room for how many, what's below),
   outdoor work (a tanner's pits, storage, a timber stack), pens, gardens,
   the woods round it, and a line saying where in the world it goes.
2. **Claude measures it**, never eyeballs it: `python tools/plans/read_sketch.py
   --grid` writes a copy of the drawing with a labelled 100 px grid and four
   quadrant crops; the four corners of every drawn building are read off
   those and written into the script's table with the kind, the variant's
   depth:width ratio, the head count and the nearest variant key. The script
   then works out each building's centre, long axis, size and facing (the
   door on the short end that looks toward the middle, the long side running
   that way), **draws the reading over the sketch** (`plan_overlay.png`), and
   rasters the trees from the black ink with the drawn things masked out
   (`woods_preview.png`, one digit a cell). Scale: a "room for 12" house is
   15 m along its long side. The overlay is checked against the drawing
   before anything else; the first attempt skipped this and turned every
   house 90° (the door was put on the long side).
3. **The plan file** `assets/world/plans/<town key>.ron` holds the result:
   every building's offset in metres, facing in degrees, size, who it houses,
   the nearest variant the game has and a `wants` line for what the drawing
   asks that the variant doesn't give; workplaces and pens the same; the
   `woods` raster; `later` for what nothing stands for yet. The drawing is
   kept beside it in `docs/plans/`.
4. **The town needs a mark** in `assets/world/towns.ron` with the same key
   (its spot on the map; the Map Marks page can move it).
5. **Laz checks it** in the game: `GAHT_GOTO=<town key> GAHT_YAW=1.5708
   GAHT_PITCH=1.5 GAHT_ZOOM=230` is the drawing's view (north up, straight
   down); or he walks there. Corrections are edits to the plan file, in
   words he can read ("facing: 90" is a door looking south).
6. **The shapes come after.** A `wants` line is a brief for a new building
   variant (`layout::VARIANTS`: a data line plus its drawing), made without
   going back to the picture. When it exists the plan names it.

## What the game does with a plan

- `Settlement::build_from_plan` (`sim/plan.rs`): the hearth at the centre
  (building 0, as in every town), then the drawn buildings in the file's
  order, so a roster's `home: Some(n)` counts them from 1. Residents fill the
  homes in order, each to its `room_for`; more people than room goes round
  again (the roster sets the real head count).
- `World::plan_styles`: a named variant is used over the one the residents'
  trades would choose.
- `society::lay_out`: the plan's workplaces stand where drawn (one on a
  building is reached by its door, as the island's hall is); every kind the
  town would want anyway is counted as given, and the rest are spiralled in
  among them.
- `animals/place.rs`: a plan with pens keeps only those.
- `view/foliage.rs` (drawing only): inside the `woods` raster the trees are
  the drawing's, thick where the ink is thick (a finer tree grid,
  `DRAWN_GRID`) and none on the open ground, whatever the land's own class
  (the first outpost stands on rock by the map, and is wooded anyway); the
  usual clearing round a town's middle is skipped for a drawn town. Outside
  the raster the land grows as usual.
- The plan is kept in the world (`World::plans`, saved) because a town is
  laid out again when its customs shift.

## Conventions

- Offsets are metres east and south of the centre (the simulation's y grows
  southward).
- Facing is degrees: 0 east, 90 south, 180 west, 270 north. It is the way the
  door looks, and the building's long side (its depth) runs that way: the
  door is on a short end.
- Size is metres along the long side; the variant's `half` ratios give the
  depth and width (a Qotiro block is 1.0 deep by 0.8 wide).

## Not yet

- No new shapes: timber two-tier houses with a workshop below, an incense hall
  with an antechamber, a forge courtyard, and a closed temple are `wants` and
  `later` lines in the first plan.
- Gardens and a timber stack have nothing to stand for them.
- The woods end where the drawing ends; past its edge the land's own class
  decides, so an outpost drawn in forest on rocky map ground has a hard edge.
  The mark should sit on forest in the map, or the map painted so.
- Roads still come to the hearth through the middle, as in every town.
