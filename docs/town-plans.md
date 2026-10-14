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
2. **Claude reads it** into `assets/world/plans/<town key>.ron`: a scale
   (metres a pixel, from how long a "room for 12" house is drawn), a pixel
   taken as the centre (the hearth), and every building with its offset in
   metres, its facing in degrees, its size, who it houses, the nearest
   variant the game has, and a `wants` line saying what the drawing asks for
   that the variant doesn't give. Workplaces and pens the same. Anything
   nothing stands for yet goes under `later`. The drawing is kept beside it in
   `docs/plans/`.
3. **The town needs a mark** in `assets/world/towns.ron` with the same key
   (its spot on the map; the Map Marks page can move it).
4. **Laz checks it** in the game: `GAHT_GOTO=<town key>` puts the squad
   beside it for a screenshot, or he walks there. Corrections are edits to
   the plan file, in words he can read ("facing: 90" is a door looking south).
5. **The shapes come after.** A `wants` line is a brief for a new building
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
- The plan is kept in the world (`World::plans`, saved) because a town is
  laid out again when its customs shift.

## Conventions

- Offsets are metres east and south of the centre (the simulation's y grows
  southward).
- Facing is degrees: 0 east, 90 south, 180 west, 270 north. It is the way the
  door looks; the building's long side runs across it.
- Size is metres across; the variant's `half` ratios give the depth and width.

## Not yet

- No new shapes: timber two-tier houses with a workshop below, an incense hall
  with an antechamber, a forge courtyard, and a closed temple are `wants` and
  `later` lines in the first plan.
- Gardens and a timber stack have nothing to stand for them.
- The woods: trees keep their usual distance from a town (`TOWN_CLEAR` in
  `view/foliage.rs`); a drawn tree line isn't read.
