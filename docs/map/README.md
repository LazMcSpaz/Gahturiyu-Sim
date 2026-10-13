# Laz's map (draft, 13 Oct 2026)

`laz-map-draft-2026-10-13.png` (2576 × 1932): the first draft of the world map, from Laz. Not done; the start of the "rebuild the map" step he held back until the systems were built.

## What it shows (the primary's reading)

- **One continent in a sea**, not the square with a western coast the generator makes: a long **northern arm** (west to east), a **southern body**, snowy mountains and rocky ground across the middle and the east, cliffs marked on the coasts (contours 250 m), rivers everywhere, a lake in the north-west, bog in the Overgrowth.
- **Purple, the north and west: the Great House** (the Throne and the Banner Lords). The **Imperial capital** on the west coast ("large castle on cliff, ancient city, mouth of mountain"), a **large Imperial city** on the northern coast ("biggest presence"), Imperial villages and fishing villages, mines, an Imperial resupply port and a remote village "claims no Imperial ties", the **High Folk** town ("large halls, snow plates") and villages in the mountains, a **secret temple**, an **ancient lookout**, the **Last Wave outpost**, the Imperial Lord's keep.
- **Orange, the south-west coast: the Open Hand.** Capital "large, old Roduro roots, partially built into mountain"; a city ("spread out, docks, keep"), a town "on the main vein of travel, strategic", villages (marshy/fishing; small fishing/forestry), a marsh hamlet, a farming hamlet, mines, an ancient lookout; the **Last Wave outpost** "second largest, rely on Open Hand business, concealed in forest".
- **Green, the south and east: the First Peoples.** Capital "large sea castle, twin rivers"; a city "fort on road, big keep"; towns ("mostly farming, built on ancient city, lots of ruins"), villages, hamlets, an outpost on cliffs; villages **half-abandoned** on the Overgrowth's edge; one village "edge of territory".
- **Grey, the Overgrowth** in the north-east and the centre-east, with a ruined village, an abandoned hamlet "not yet taken", a ruined city; the **Wardens** hold the line round it: the **primary Warden fort** and village in the north, a Warden fort at the river-crossing **crossroads village** (where the two road networks meet), a Warden fort "small hamlet, primary fort in the south" and a Warden outpost.
- **Two road networks**: red in the north (Imperial), orange in the south, joined at the crossroads village.
- **Islands**: **Salt Island** just off the Imperial capital (the tutorial island: it moves there); the **Sea Fort** island (Sea Fort Town: Imperial, small, a loyal Banner Lord, hunts pirates; a Horaro wizard's tower; the exiled son of the Sea Fort lord; an ancient Roduro tomb).

## What it means for the engine (to agree with Laz before building)

1. **The land becomes authored.** `Terrain::generate` (a seed's coast, hills, the Qotiro plateau) gives way to land read from a painted map: the legend's classes (plains, forest, coast, bog, rocky ground, snowy mountains, the Overgrowth) plus the cliff contours give height, ground and plants; sea all round. Rivers and bog are new ground (fords, bridges, slow going). The seed still rolls people, weather and everything else.
2. **Towns are placed by hand** from the map's marks, each with a kind (capital, city, town, village, hamlet, outpost, fort, ruined/abandoned), a faction holder (the territories: the Throne, Open Hand, First Peoples, the Wardens' forts, Last Wave outposts, High Folk) and the notes; the factions' leaders get their seats. The English names on the map are placeholders for the naming module.
3. **Roads from the drawn lines**, not found by the pathfinder.
4. **The Overgrowth** becomes real regions with the ruins and half-abandoned places on its edge; the Wardens' forts are where the line is held.
5. **The Qotiro plateau** of the current canon isn't on the map (the south-east is First Peoples plains and forest): dropped, or Laz says where.

## Open (for Laz)

- **Scale**: how far across is the continent? The current world is 21 km square (walked in a day or two). This looks like several hundred; the sim can take a bigger world, but travel, caravans, the bands and the day-plans were tuned at 21 km. A suggestion: ~60–80 km across, so a crossing is a few days.
- **Source of truth**: is the finished map one image I trace (classes by colour, towns and roads by hand into a `.ron`), or layers (land, water, roads, labels) exported separately? Layers are easier and exact.
- **The plateau**, and whether any of the current seed-made towns should survive (none need to).
- **Salt Isle**: off the Imperial capital as drawn; the ferry port becomes the capital's dock.
