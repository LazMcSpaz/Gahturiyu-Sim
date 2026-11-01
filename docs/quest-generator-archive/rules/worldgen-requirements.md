# World Generation: What Places and Factions Need (stub v1.2, 2026-10-27)

This is a starting point for a later chat. World generation isn't designed yet. This doc only lists what the land generator must produce so that `faction-places.md` (the relationship map in §1) can place factions and places. **No design decisions here, just requirements.**

## 1. Land features the generator must make
Each feature needs a **score**, so the placer can pick "the most X" and fall back to "the next best".

| Feature | Needed by | Score needed |
|---|---|---|
| **Coast**, with how sheltered each stretch is | Reefhold, the Tide Houses, Openharbour | shelter |
| **Natural harbors** | Openharbour (the best one the First Peoples don't hold) | harbor quality |
| **Reefs or coves** | Reefhold (fallback: the calmest cove) | — |
| **Offshore islets, skerries, sea rocks** | the Far Sails, the Lantern Rock | distance from shore |
| **Elevation:** hills, highlands, mountains | the Holy House, Firststone, the Highfolk, the Last Wave | height |
| **Snowline** (set by height, not by latitude or direction) | the High Snows | yes or no |
| **Rough or broken ground:** bowls, mesas, rocky rims | Textrim's Height, the barrier between the Highfolk and the Great House | defensibility, roughness |
| **Fertility:** low valleys, farmland | the Ninth Ring, the First Peoples | fertility |
| **Rivers** and their **crossings** | Highcrossing (the most connected crossing) | connectedness |
| **Wild land:** forest, bog | the Binders, Breakwall, the Walking Road | how settled it is |
| **The Overgrowth:** always exists | the Wardens, the Rooted, Wellstone, the Heart | distance to its edge (so the "center" is the point farthest from every edge) |
| **Ruins:** a ruined imperial wall, an ancient temple site | Longwall, the Womb | — |

## 2. Networks and measures
- **Roads:**
  - A road network linking places, with **junctions** and a **traffic** score (the busiest road and the busiest junction).
  - Each road has a **real direction**, which fills in names like "[Dir]road".
- **Route-finding with forbidden zones.** The Walking Road must route around Open Hand land and First Peoples core towns.
- **Travel time** across terrain, which defines "near" (about a day).
- **Island size** sets "far": the opposite side, or as far as the land allows.
- **Line of sight** from shore, for "within sight of Reefhold's coast".
- **Distance from roads,** for the Womb (out of reach of any road) and the Highfolk fallback.

## 3. Territories
- **Contiguous territories,** one per faction, grown around each faction's terrain needs (faction-places §1a).
- **Borders** between territories, so "adjacent" can be tested and "where two or more territories meet" can be found (Lanternhall).
- **A territory core,** so seats can go at the center and frontier places on the edge.
- **"Most dangerous neighbor"** (Longwall) needs faction relations: read from the factions doc (rival or at war).
- **Settled vs unsettled land,** so the Binders get the least-settled land and the Wardens face the nearest settled territory.

## 4. Settlements
- **Size classes:** at least village, town, large town and great town. Guild HQs need a large or great town, and every large or great town holds a hall of each of the six joinable factions (`faction-places.md` §5b).
- **Inner and outer rings** inside a town, for the Low Shrine in a poor outer ring. This may already be covered by `town-layout.md`.
- **Generated holdings:** how many per faction, and how close to its fixed places. Not yet decided.
- **Presences** (faction-places §5b): how many of each, and how quickly they thin out with distance from home ground. Guild presences need a **town-size threshold** (which towns count as "large" and get a hall, and which get only a contact).

## 5. Placement order and conflicts
- **Order** (from faction-places §6):
  1. land
  2. territories
  3. fixed places
  4. generated holdings
- **Unresolved:** what to relax when the rules conflict. A suggested start for later:
  - the "never" lines (§1d) are **hard rules**;
  - terrain needs come next;
  - adjacency and "near" are **soft**, and give way first.
- **Unresolved:** what happens when no placement works at all. Options are to re-roll the land, or to allow a fallback with a warning.

## 6. The space between (Laz, 2026-10-26)
- The world between settlements must be dotted with small sites: bandit camps, caves, abandoned forts, and the like.
- **What dots the world is partly decided by the quests:** each generated quest that needs a place of some kind creates it into the registry, and the placer puts it down where it fits.
- **Needs later:**
  - a **catalogue of site kinds**, each with a placement rule in the same relational style;
  - extra land features some sites need: **mountain passes** (pass huts), **headlands** (beacons), **caves**, and **old fort sites**.
- Presences (faction-places §5b) are the first part of this. The neutral sites, such as bandit camps and caves, are not designed yet.

## 7. Open questions for the world-generation chat
- **Island or continent?** The faction-places doc assumes one island.
- **Scale:** how many days of travel across the land?
- **Reach:** is it generated from placement, or fixed? Reach is currently a world assumption (quest-factions §2); seats come from faction-places §4/§6.
- **Overgrowth size:** is it fixed or rolled? Does it grow over time? Growing over time is mechanical-layer work, so it's parked.
- **Climate:** is snow only from height, or is there any climate band?
- **Minor factions:** do they get generated holdings? Currently they don't.

## 8. Up front vs on approach (Laz, 2026-10-26)
- **Up front, at world creation:** every quest and its connections, plus the rosters they require. That means people, places, flags and facts: all text and numbers.
- **The planned world size comes first.** Before any quest is generated, the generator needs **N**, the planned number of cards in the world. The world ledger's caps scale with it (rulebook, *World ledger*), so the map summary must give enough (place sizes and quotas) to set N.
- **On approach** (when the player comes within range of a region or place): the physical details, worked out from the **descriptors** saved on the place (card-schema Place `descriptors`). For example, *large stronghold, well defended* → a garrison size from a table → enough housing for it → walls, a moat or both, picked from options.
- **Quests leave affordance tags on places.** These are things the quest's ways need: *a drain nobody guards*, *a cell in the lower yard*, *a back gate*, *a cellar under the street*. The detail generator **must** build them in (card-schema Place `affordances`). That's what makes a generated keep play the way its quest promised.
- **Details are seeded** by world seed plus place id, and saved once built, so a place never changes on a revisit.

## Changelog
- **2026-10-27 (v1.2), v6: World C fixes:** §8 notes that the world ledger's caps need the planned world size N before generation. v6b audit: §4's headquarters may sit in a large or great town, and every big town holds six halls.
- **2026-10-26 (v1.1), audit fixes:** §7 now says only reach is a world assumption (seats come from faction-places); §8 points at the schema fields for descriptors and affordances.
