# Laz's map (13 Oct 2026)

- `laz-map-2026-10-13.png` (2576 × 1932): the clean export, legend colours only (rolling plains, forest, coast, bog, rocky ground, snowy mountains, the Overgrowth, rivers, cliff ticks at 250 m contours), no labels. **This is what `sim/atlas.rs` reads** (by colour, until flat layers exist in `assets/world/`).
- `laz-map-draft-2026-10-13.png`: the annotated draft (territories in colour, towns and notes in Laz's hand, the two road networks in red and orange).
- `laz-roads-2026-10-13.png`: the draft's red and orange strokes lifted out as a transparent overlay (`tools`-free: a colour threshold in Python), 150 m a pixel like the rest. The road layer the routes will read.
- Towns are placed on a web page (Map Marks: generic icons, Laz's roads, a road tool; its RON export is the shape of `assets/world/towns.ron`).

**Scale (Laz, 14 Oct): "Make it 4 Kenshis."** Kenshi's map is about 870 km², so the continent is about 3,500 km²: **26.5 m a pixel**, 68 × 51 km (`WORLD_SIZE` = 2576 × 26.5 m). The capital to the crossroads village (about 1 100 px of road) is now about 6 hours' walk. Before (13 Oct) it was 36 hours and 150 m a pixel, 390 × 290 km, which Laz found too big. Heights stay in metres; the atlas's blur and coast band are set in metres (`BLUR_M`, `COAST_M`), so the slopes and shores are the same lengths they were.

## The draft's reading (the primary, before the clean map came)

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

## Answered (Laz, 13 Oct)

- **Scale**: four Kenshis (above; was 36 hours capital to crossroads on 13 Oct). The road grid is 200 m, animal regions 3 km.
- **Source of truth**: Laz can export layers; until then the clean map is read by colour. Wanted layers, same 2576 × 1932 canvas: `terrain.png` (legend fills only), `water.png`, `roads.png`, `cliffs.png`, `territory.png`.
- **All the generated towns are dropped**; `assets/world/towns.ron` holds placeholders read off the draft until Laz's marks come from the page. **Don't lay the towns out carefully yet.**
- **The plateau** isn't on the map: `plateau_mask` is 0 everywhere; the weather's Plateau region never shows.
- **Salt Isle** lies off the Imperial capital (`geo::isle_centre`: `ISLE_OFF` seaward of the first capital mark); the ferry port is the capital's dock.
