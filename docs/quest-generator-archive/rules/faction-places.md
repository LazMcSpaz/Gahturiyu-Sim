# Faction places (v4.2, 2026-10-27)

The named places each faction holds, with a few things that make each one unique. This is for location-specific quests and the quest generator.

**How this works (Laz's calls):**
- **Hybrid.** The places below are **fixed**: the same names every playthrough. Each faction's ordinary holdings (§5) are **generated** from its template.
- **Scaled by size.**
  - Majors: a seat plus one place per sub-faction.
  - Minors: 1–2 places each.
  - Guilds: 1 headquarters each.
- **Naming:** evocative English names. Native names can be built with the naming module later. Many names reuse the house and road bynames already in `leaders.md`.
- **Each place gets three unique things:**
  - **Landmark:** something you can see and go to.
  - **Trade:** what the place makes, sells or controls.
  - **Secret or hook:** a tension or hidden truth for quests. A hook marked *(secret)* is a hidden fact: quest cards may build on it, but only a main story's revelation, or a faction story's revelation about its **own** faction's places, can expose it.
- **Placement:** a relationship map (§1), not compass directions. The world generator places everything wherever those relationships hold best on the land it generated.
- **Canon secrets** (the Womb, the Overgrowth's heart) are included, flagged secret.

---

## 1. Relationship map
Placement is **relational, never by compass direction**. The world generator builds the land first, then places every faction and place where these relationships hold best. Directions in a name ("Northroad") are filled in from the generated land, never assumed.

### 1a. Terrain each faction needs
If the world has no such terrain, the fallback applies.

| Faction | Needs | Fallback if missing |
|---|---|---|
| **The First Peoples** | the most fertile low valleys, plus a sheltered stretch of coast for the Tide Houses | the best farmland, plus the calmest coast |
| **The Great House** | the most-connected land: the biggest river crossing and the roads that meet there | wherever the most routes converge |
| **The Open Hand** | the best natural harbor *not* held by the First Peoples, plus busy road junctions | the busiest unclaimed coast and junctions |
| **The Holy House** | high ground | the highest hill in the territory it sits in |
| **The Last Wave** | the roughest high ground: bowls and mesas on a rocky rim | the most defensible broken ground |
| **The Far Sails** | islets or skerries offshore | an anchorage in open water off the coast |
| **The Binders** | wild frontier land: forest or bog | the least-settled land |
| **The Wardens** | the edge of the Overgrowth | — (the Overgrowth always exists) |
| **The Highfolk** | the highest, snowiest mountains | the highest, roughest ground, the farthest from roads |
| **The Rooted** | inside the Overgrowth | — |

### 1b. Shape rules for every faction
- **Clustered.** A faction's places form one connected territory. Its generated holdings sit near its fixed places, never scattered across the map.
- **Sub-factions sit inside their parent's territory,** near their own seat.
- **Seat at the core.** A faction's seat is the most central of its places. Its frontier places sit on the borders where its problems are.
- **Guild headquarters sit inside a large or great town** that belongs to some other faction (§4). The Holy House is the exception: its headquarters is its own seat, the Mount (§3).
- **Distances are relative to the island's size:**
  - **adjacent:** territories share a border
  - **near:** about a day's travel
  - **far:** the opposite side of the island, or as far as the land allows

### 1c. Who sits next to whom

| Relationship | Reason (canon) |
|---|---|
| **First Peoples adjacent to the Great House** | the two old powers, side by side |
| **Open Hand threaded between and around both** (ports and junctions on their edges) | the upstart trading in the gaps and squeezing the old powers |
| **Holy House inside First Peoples territory**, near the Ninth Ring | the church leans on the Alliance as its patron |
| **Banner Lords' places near Highcrossing; Free Lords' places on the Great House's frontier** | the loyal stay close to the Throne, and the defiant hold the edges |
| **Last Wave adjacent to the Free Lords' frontier, far from the First Peoples' core** | Kotqot has Qotiro blood and ties there; the Alliance is hostile to them |
| **Binders inside or against the Free Lords' frontier** | the Free Lords are their main buyers; First Peoples houses buy Qotiro |
| **Far Sails offshore within sight of the Tide Houses' coast** | the Horaro who left, and the families they left behind |
| **Highfolk adjacent to Great House territory, but separated from it by rough ground** | the Empire claims the mountains, and nobody bothers to climb up and enforce it |
| **Wardens between the Overgrowth and the nearest settled territory** | the front line |
| **Lanternhall where at least two factions' territories meet** | neutral ground, for hire by anyone |

### 1d. Lines that must never cross

| Never | Why |
|---|---|
| **The Walking Road through Open Hand territory, or through its roads and ports** | the Open Hand frees people; slavers route around them |
| **Last Wave places adjacent to First Peoples territory** | open hostility |
| **The Womb within reach of any road** | it is hidden |
| **The Heart near the Overgrowth's edge** | it sits at the center, the point farthest from every edge |

**Slavery (Laz, 2026-10-26):** the First Peoples allow slavery, but only of Qotiro (Ṭaḍoro too, but rarely). So:
- The Walking Road **may** run into First Peoples land. It is their second market after the Free Lords.
- Qotiro Binders selling Qotiro to the old houses is a built-in tension for quests.
- The Open Hand is still the one territory slavers can't cross.

## 2. Major factions

### The First Peoples

**The Ninth Ring** — *the Alliance's seat; home of the Seated Houses (Tuhuḍo of the Ninth-Ring House)*
- **Kind:** a great town at the heart of First Peoples territory, in its most fertile valley. **Standing:** Top.
- **Landmark:** nine rings of grown-stone houses around a sunken court, each ring a generation older than the one outside it. The innermost houses have grown so large their doors are now too high to use, so they're entered by bridges from the eighth ring.
- **Trade:** records and pedigree. Every house on the island has a line in its archive, and proof of descent is bought and sold here.
- **Hook:** Tuhuḍo rules from memory. The archive has gaps that only he knows about, so a forged lineage could pass if he dies or forgets. *(secret)* He's begun to forget.

**Firststone** — *seat of the True Line (Garoṭaqa and Ruʻo of the First-Stone House)*
- **Kind:** an old hill estate, half-ruined, within a day of the Ninth Ring. **Standing:** High.
- **Landmark:** the Founding Stone, a grown-stone stump older than any house in the Ninth Ring, which the True Line say proves their claim. Pilgrims of the "pure blood" leave cuttings of their hair in its cracks.
- **Trade:** almost nothing. The estate runs on gifts from the bitter and the young.
- **Hook:** Garoṭaqa is digging under the Stone for older records to prove the claim. *(secret)* What's down there contradicts the "first two peoples" story she's trying to prove.

**Reefhold** — *seat of the Tide Houses (Niralea of the Reef Line)*
- **Kind:** a Horaro stilt town behind a reef, on the First Peoples' most sheltered stretch of coast. **Standing:** High.
- **Landmark:** the Speaker's Deck, a round platform out over the lagoon. Each season's speaker sits there, and it's built so that it slowly sinks a hand's width a year, so it has to be rebuilt by every generation.
- **Trade:** deep kelp and divers. The finest divers on the island come from here.
- **Hook:** half the houses have a cousin in the Far Sails, and boats slip out to the fleet at night. *(secret)* Niralea knows, and allows it.

### The Great House

**Highcrossing** — *the capital; seat of the Throne (the emperor Gahìru, and Litiṭaʻa)*
- **Kind:** the old imperial city at the island's biggest river crossing, where the most roads meet. **Standing:** Top.
- **Landmark:** the Empty Gallery, a long hall of plinths that once held one statue for each province of the empire. Most stand bare now, and the court has learned to walk past them without looking.
- **Trade:** offices, titles and appointments. Every lordship's papers are sealed here.
- **Hooks:**
  - The emperor's public audiences are staged. *(secret)* A double sits in for him when his hands shake.
  - The old court mage's tower is sealed and no one goes in. *(secret)* This is the mage who made the Overgrowth (`hidden-lore.md`).

**Longwall** — *fortress of the Banner Lords' old general (Haʻahoqo of the Long-Wall House)*
- **Kind:** a fortress at the end of a ruined imperial wall, on the Great House border that faces its most dangerous neighbor. **Standing:** High.
- **Landmark:** the Wall itself, a day's march of broken rampart. Veterans walk it every year, naming the dead at each tower.
- **Trade:** soldiers. Longwall trains the Throne's best troops and rents out the rest.
- **Hook:** the general has quietly begun counting how many soldiers would follow him, rather than the Throne, if the emperor died.

**[Dir]road** ("Northroad" in a world where that road runs north) — *seat of the Banner Lords' Ṭaḍoro-born lord (Shahayuh of the [Dir] Road)*
- **Kind:** a toll town on the busiest road leaving Highcrossing. The name takes that road's real direction. **Standing:** High.
- **Landmark:** the Great Bridge, the span his title is named for. He paid for it out of his own pocket, and his name is cut into every stone, which the old blood finds vulgar.
- **Trade:** tolls. All the road's traffic crosses here.
- **Hook:** old-blood lords keep "accidentally" damaging the bridge, and Shahayuh suspects his fellow Banner Lord.

**Breakwall** — *hold of the Free Lords' Qotiro-blooded lord (Kotqot Wall-breaker)*
- **Kind:** a frontier keep on the Great House's wild edge, adjacent to the Chainmarket and to Last Wave land. **Standing:** High.
- **Landmark:** a hole smashed through its own outer wall and left open on purpose, from the day Kotqot took the keep. His rule: whoever can hold the breach can have the keep.
- **Trade:** timber, bog iron, and the labor to dig both, much of it bought from the Binders.
- **Hook:** *(secret)* Kotqot keeps a written record of every lie the Throne has told, to publish the day he declares independence.

### The Open Hand

**Openharbour** — *seat of the Open Council (Waarua of the Harbour)*
- **Kind:** the island's largest natural harbor that the First Peoples don't hold. **Standing:** Top.
- **Landmark:** the Unchained Quay, built from broken shackles melted into its railings. Every freed bondsman who joins the Hand adds a link.
- **Trade:** everything. The island's busiest market and its biggest shipyard.
- **Hook:** the council hall has more locked ledger rooms every year, and Waarua no longer has keys to all of them.

**The Silver Road Counting-House** — *seat of the Sharp Hands (Feyesh, Dorog and Kodor)*
- **Kind:** a fortified counting-house at the first major crossroads on the road from Openharbour into Great House lands. **Standing:** High.
- **Landmark:** the Scales Hall, a hall of mirrored brass scales where debts are weighed in public. The scales are famously accurate.
- **Trade:** notes, loans, and the debts of half the Open Hand's territory.
- **Hook:** *(secret)* a cellar below the counting-house keeps a second set of books, for the Binder trade the Sharp Hands profit from.

**Carter's Rest** — *yard and meeting place of the Road Crews (Doruḍoqu the Carter)*
- **Kind:** a waystation town at the busiest road junction in Open Hand territory. **Standing:** Middle.
- **Landmark:** the Long Shed, a stable a quarter mile long. The crews vote here by standing on one side of it or the other.
- **Trade:** hauling. Every caravan on the island changes teams here.
- **Hook:** the crews are close to voting to strike against Sharp Hands rates. One bought vote would decide it.

---

## 3. Minor factions

### The Holy House
**The Mount** — *seat of the high priest (Hotorìṭo)*
- **Kind:** a fortified shrine-town on the highest hill within a day of the Ninth Ring. **Standing:** High.
- **Landmark:** the Armed Nave, a cathedral turned barracks. Weapon racks stand where the pews were.
- **Trade:** protection, for a tithe.
- **Hook:** *(secret)* Hotorìṭo no longer believes, and has stopped performing the inner rites; no one has checked in years.

**The Low Shrine** — *where the sincere priestess Aloqi keeps faith*
- **Kind:** a small shrine in the poor outer ring of a First Peoples town (not the Ninth Ring). **Standing:** Middle.
- **Landmark:** a soup kitchen in the nave, the old way. The line goes out the door every evening.
- **Trade:** none. It runs on donations, increasingly from people who have stopped going to the Mount.
- **Hook:** the Mount wants it closed, and Aloqi has more followers than she realizes.

### The Last Wave
**Textrim's Height** — *the main enclave (Textrim, High Priestess)*
- **Kind:** a stepped enclave in the most defensible bowl on the roughest high ground. **Standing:** High.
- **Landmark:** a small ziggurat in the local dark stone. At the summit the priestesses' fire is never allowed to go out.
- **Trade:** forge salt and weapons, the best smithing on the island.
- **Hook:** the enclave is one hard winter from hunger, and Textrim would trade peace for grain if anyone offered it with respect.

**The Proving Ground** — *the war-chief's camp (Totkit Sea-crosser)*
- **Kind:** a fortified camp between Textrim's Height and the nearest lowland settlement. **Standing:** Middle.
- **Landmark:** the Circle, a ring of standing stones where outsiders must stand against a fighter before they're let in to speak.
- **Trade:** sellswords and guards, hired out to anyone who passes the Circle.
- **Hook:** young warriors here want war with the lowlands, and Totkit is holding them back by a thread.

### The Far Sails
- **Offshore:** both places. Until Laz opens the sea they take no cards, and neither does any Far Sails holding only on an islet or skerry (rulebook, *Entry specs*).

**The Anchorage of the Wide Bay** — *the fleet's home water (Nemu of the Wide Bay)*
- **Kind:** a ring of anchored ships and skerry huts, offshore and within sight of Reefhold's coast. **Standing:** High.
- **Landmark:** the Raft of Captains, a huge raft lashed from old hulls where the captains vote.
- **Trade:** shipping and passage, and smuggled goods for the Night Market.
- **Hook:** runaway bondsmen are sheltered here, and the Binders have offered a price for the list of names.

**The Lantern Rock** — *the fleet's lookout*
- **Kind:** a tower on a lone sea rock between the Anchorage and the mainland. **Standing:** Middle.
- **Landmark:** a beacon that signals the fleet in a code no landsman knows.
- **Trade:** pilots for hire through the reefs.
- **Hook:** *(secret)* the code has been sold to someone on the mainland.

### The Binders
**The Chainmarket** — *seat of the Binders (Paqax Chain-breaker)*
- **Kind:** a market town on the Free Lords' frontier, adjacent to Breakwall. **Standing:** High.
- **Landmark:** the Block, a raised stone platform where people are sold. Paqax had the shackles he wore as a slave set into it.
- **Trade:** people, sold to the lords.
- **Hook:** *(secret)* many Binders here hate the trade. A quiet group buys some of the captives free at night, with coin skimmed from the Block's own takings.

**The Walking Road** — *the slave road*
- **Kind:** a road with chained waystations, running from the wild lands into Free Lords territory. It never crosses Open Hand land (§1d). **Standing:** Middle.
- **Landmark:** the Last Well, where captives get their final drink before the market. Families leave tokens there for the ones who were taken.
- **Trade:** passage for slave caravans.
- **Hook:** a waystation keeper is passing escape routes to captives, one at a time.

### The Wardens
**The Green Wall** — *the Wardens' fort (Haʻo and Hoqa of the Green Wall)*
- **Kind:** a long timber-and-stone wall with a fort at its center, between the Overgrowth and the nearest settled territory. **Standing:** Middle.
- **Landmark:** the Names Post, a pillar carved with the names of every Warden lost to the green. It's nearly full.
- **Trade:** none. They live on begged coin and on what they salvage from lost villages.
- **Hook:** *(secret)* Hoqa has mapped a path deeper into the green than anyone has gone, and found something that walks there with purpose.

**Wellstone** — *a lost village*
- **Kind:** a village swallowed by the Overgrowth, just past the Green Wall. **Standing:** Low.
- **Landmark:** the village well, still standing among the trees. The Wardens say you can hear the old village's bell from it on still nights.
- **Trade:** salvage, for the brave.
- **Hook:** many Wardens came from here, and they never talk about why the village fell so fast.

### The Highfolk
**The High Snows** — *seat of the clans (Shihi of the High Snows)*
- **Kind:** a mountain hold carved into a cliff above the snowline, on the side of the mountains facing Great House land. **Standing:** High.
- **Landmark:** the Guest Fire, a hearth that's kept lit for travellers. Anyone who reaches it is sacred while they sit there.
- **Trade:** furs, mountain ore, and guides over the passes.
- **Hook:** the Great House sends a tax collector every few years. He never comes back down, and no one asks why.

**The Womb of Creation** — *(secret; `hidden-lore.md`)*
- **Kind:** an ancient temple complex hidden in the mountains, farther in than the High Snows and out of reach of any road. **Standing:** Top.
- **Landmark:** carvings of a single people, before the gods split them into four.
- **Trade:** none. It's guarded, and its location is never spoken.
- **Hook:** *(secret)* the truth of the one race. It's reachable only through the Highfolk's deepest trust, or by force.

### The Rooted
**The Soft Hollow** — *a Rooted village*
- **Kind:** a village inside the Overgrowth, between the Green Wall and the Heart. **Standing:** Low.
- **Landmark:** the Listening Trees, where the sworn press their ears to the bark to hear the dead.
- **Trade:** none. People come here to stay.
- **Hook:** someone the player knows may be speaking from these trees.

**The Heart** — *(secret; `hidden-lore.md`)*
- **Kind:** the center of the Overgrowth, the point farthest from every edge. **Standing:** Top.
- **Landmark:** the old court mage's animated corpse, at the root of everything.
- **Trade:** none.
- **Hook:** *(secret)* destroy the corpse and the Overgrowth ends. This is the late-game goal.

---

## 4. Guild headquarters

| Guild | Headquarters | Landmark | Trade | Hook |
|---|---|---|---|---|
| **The Writers** | **The Wayhouse**, a library in Highcrossing that's mostly empty shelves, because the books travel with the scholars. Fiyeweya is never there. | the Lending Wall, a wall of pigeonholes holding letters to and from every scholar on the road | records, copies, secrets | *(secret)* one shelf is always kept empty, waiting for the court mage's papers |
| **The Lantern Company** | **Lanternhall**, a fortified hall in a large town at a crossroads where two or more factions' territories meet | the Lamp Room: one lamp for every contract, lit until it's done | swords for hire | an old contract the Company can't break is about to put it on the wrong side of a lord's feud |
| **The Old Hands** | **The Old Gate House**, an old-family quarter of the Ninth Ring | a gatehouse whose gate hasn't been closed in two hundred years | blackmail, forged lineages, smuggling | *(secret)* Yitolaʻo has kept a copy of every favor done for the old houses, and they've stopped answering his letters |
| **The Night Market** | **Under the Market Road**, a shop in Openharbour with a cellar that runs under the main street | a plain door that only opens after dark | fencing, smuggling, forged notes | Yaweth is planning the biggest score in the island's history |
| **The Dueling Society** | **The Ring**, an arena in Openharbour | a sand floor where the dueling lines are marked in chalk and redrawn every morning | bouts, training, wagers | the old heads want it shut, and someone keeps sabotaging the bouts |

---

## 5. Generated holdings (templates)
These cover each faction's **ordinary** places, which are rolled per world. Each one gets a name from the faction's name pattern, plus one landmark, one trade and one hook from its pools.

**Name patterns:**
- **First Peoples:** *[old thing]* + *ring / hollow / stone* (Ashring, Elderhollow)
- **Great House:** *[rank or deed]* + *hold / ford / gate* (Oathford, Crownsgate)
- **Open Hand:** *[trade]* + *market / yard / quay* (Saltmarket, Wainyard)
- **Last Wave:** *[deed]* + *height / step* (Ironstep)
- **Far Sails:** *[sea word]* + *mooring / rock* (Gullrock)

| Faction | Landmark pool | Trade pool | Hook pool |
|---|---|---|---|
| **First Peoples** | a house grown so big it swallowed its neighbor; a sealed archive room; a stump-ring of a vanished house | records, stone milk, old-family goods | a forged lineage; a house going quiet; a Horaro family planning to leave |
| **Great House** | an empty plinth; a lord's tower; a toll bridge | tolls, soldiers, grain | a lord backing the wrong heir; a slave-bought field; a forged imperial seal |
| **Open Hand** | a new quay; a caravan yard; a freed family's shop | trade goods, loans, hauling | a debt bought up by the Sharp Hands; a strike brewing; a Binder contract hidden in a deal |
| **Last Wave** | a dark-stone step-hall; a duel circle; a smoke hall | forge salt, weapons, guards | a hungry winter; a raid planned; a mixed marriage |
| **Far Sails** | a hulk turned house; a signal beacon | pilots, passage, smuggled goods | a runaway hidden aboard; a debt to the Night Market |

Minors and guilds don't get settlements. They get **presences** instead (§5b).

## 5b. Presences (minor factions and guilds)
A **presence** is lighter than a settlement: one building, camp, marker or back room, not a town. Presences keep minor factions and guilds visible across the whole map, and they help fill the space between towns.

**Each presence rolls:**
- **Site:** one entry from the faction's site list.
- **Setting:**
  - *in town:* inside an existing settlement;
  - *in the land:* between settlements.
- **Condition:** one of the site's states. The usual set is *kept*, *neglected*, *ruined*, and *taken over* (by bandits, a rival, or the green).
- **Nearby (0–1):** one attached detail that turns the site into a small scene, such as traders for pilgrims or bandits who prey on them.
- **Hook:** one from the faction's pool.
- **Placement:** the site's own rule, which is relational (§1).
- **Rolls read secrets.** A presence's rolls read the secret facts on it and on its faction, and always show its **public front**.
  - A presence with a secret backer shows its front: a road fort "under contract to the Lantern Company" whose coin is really the Throne's rolls and reads as the Lantern Company's. The backer stays a secret fact.
  - A hidden presence (a Rooted cell, a Binder broker in an Open Hand town) shows its cover: a cellar, a money-lender.
  - Only a revelation can expose what's behind the front (the rule for *(secret)* hooks above).

**How many** is a world-generation setting (`worldgen-requirements.md`). As weights:
- Minors are densest near their home ground and thin out with distance.
- Guilds follow the towns.

### Guilds: a four-step ladder
So joining a guild feels like joining something island-wide:
1. **Headquarters:** one, fixed (§4).
2. **Hall:** in every large or great town (v6b: every guild, and the Holy House, has one there, so the guilds share the big towns, Elder Scrolls style; `quest-faction-stories.md` §8b.2). Join, rank up, get the main jobs: the story's routine cards and reveal, and the hall jobs (locations §4, *Hall jobs*).
3. **Contact:** in some smaller towns. One person, a notice board or a back room; small jobs and basic supplies.
4. **Field sites:** out in the land, where the guild's work actually happens.

### The Holy House
Joinable since v6b (Laz), so it climbs the guilds' four-step ladder: **headquarters** the Mount (§3); **hall** a chapel or parish shrine in every large or great town; **contacts** the parish shrines and tithe-houses of smaller towns; **field sites** wayside shrines and inquisitor camps. The Low Shrine (§3) is the priestess's seat and gives jobs as a fixed place (locations, *Signature quota*).

| Site | Setting | Placement | Conditions |
|---|---|---|---|
| **Wayside shrine** | land | on roads, crossings and hilltops; densest in First Peoples land, thinning outward; never in Last Wave land | kept (a keeper lives there), neglected (offerings left to rot), ruined (roof gone), fortified (an inquisitor post) |
| **Chapel or parish shrine (hall)** | town | **every large or great town** *(Claude, for v6b)*: the town's parish shrine where it has one, otherwise a chapel (one room, one priest, a tithe box). It is the house's hall: its routine cards, reveal and hall jobs are given there | kept, neglected |
| **Parish shrine** | town | every First Peoples town, and many Great House towns | kept, neglected |
| **Tithe-house** | town | towns where the Holy House sells "protection" | kept |
| **Inquisitor camp** | land | moves along roads near towns the Holy House suspects | kept |

- **Nearby:**
  - pilgrim stalls (bread, candles, fake relics)
  - bandits waiting for pilgrims
  - a checkpoint questioning pilgrims about heresy
  - a beggar who knows everyone who passes
  - a hermit who stayed when the priests left
- **Hooks:**
  - someone other than the keeper empties the offering box
  - the relic is fake, and the keeper knows it
  - the pilgrims stopped coming, and nobody says why
  - a priest here preaches Aloqi's way and is about to be reported to the Mount

### The Last Wave
| Site | Setting | Placement | Conditions |
|---|---|---|---|
| **Watch cairn** | land | a ring on high ground within about two days of Textrim's Height | kept, abandoned |
| **Hunting camp** | land | rough ground near the enclaves | peaceful, armed (turned war camp), abandoned |
| **Lone smoke hall** | land | on the trail from the lowlands toward an enclave; travellers are tested here | kept, burned (by lowlanders) |

- **In town:** none. Isolation is the point. Earlier-wave Qotiro in mixed towns are **not** Last Wave.
- **Nearby:**
  - a lowland trader who has learned their customs
  - a duel circle
  - an earlier-wave Qotiro who came to plead or to warn
  - lowland sheep that "wandered off"
- **Hooks:**
  - a raid is being planned from here
  - a hunting party is missing
  - a mixed couple is hiding here
  - lowland farmers blame them for losses that are really the Binders'

### The Far Sails
| Site | Setting | Placement | Conditions |
|---|---|---|---|
| **Landing** | land (coast) | beaches and jetties, densest along the Tide Houses' coast | active, abandoned, watched (by the Alliance or the Binders) |
| **Smugglers' cove** | land (coast) | hidden coves, shared with the Night Market | active, informed on |
| **Hulk house** | land (coast) | a beached ship turned into a home, anywhere on the coast | kept, wrecked |
| **Pilots' hall** | town | port towns | kept |

- **Nearby:**
  - Tide House relatives meeting in secret
  - a runaway waiting for a boat
  - a Night Market buyer
  - a customs man paid not to look
- **Hooks:**
  - a boat is overdue
  - someone informed on the cove
  - a Tide House family is sending a child to the Sails

### The Binders
| Site | Setting | Placement | Conditions |
|---|---|---|---|
| **Waystation** | land | on the Walking Road only | active, burned (by liberators), quietly turned (a keeper is helping captives escape) |
| **Raid camp** | land | wild land within reach of a road; never in Open Hand land | active, abandoned |
| **Holding pen** | land | hidden in forest or bog | active, emptied |
| **Debt broker** | town | Free Lords towns, and some First Peoples towns. Illegal and hidden in Open Hand towns, which makes it a hook in itself | kept |

- **Nearby:**
  - a lord's buyer, waiting
  - a family searching for someone who was taken
  - caravan guards paid to look away
  - a freed bondsman, watching
- **Hooks:**
  - a captive here is somebody's kin
  - a Binder wants out
  - the keeper sells routes to whoever pays

### The Wardens
| Site | Setting | Placement | Conditions |
|---|---|---|---|
| **Watchtower** | land | spaced along the Overgrowth's edge: close together near the Green Wall, farther apart beyond it | manned, undermanned (one old Warden), abandoned, taken by the green |
| **Marker cairn** | land | at the Overgrowth's edge, marking how far the green had reached in a given year | standing, swallowed |
| **Salvage camp** | land | in a lost village just inside the green | active, abandoned |
| **Recruiting post** | town | a table in the market or a room at an inn, with a begging bowl and a list of lost villages; in many large towns, refused in some | kept |

- **Nearby:**
  - a family that won't leave its farm on the edge
  - Writers studying the green
  - scavengers
  - a Rooted listener in the trees
- **Hooks:**
  - the cairns show the green moving faster
  - a tower's signals have stopped
  - someone is moving the cairns back to make the green look slower

### The Highfolk
| Site | Setting | Placement | Conditions |
|---|---|---|---|
| **Pass hut** | land | on mountain passes. The Guest Fire custom applies: whoever sits at its fire is safe | kept, snowed in, abandoned |
| **Boundary cairn** | land | where the clans' land begins | standing, knocked down |
| **Summer camp** | land | high pasture | occupied in summer only |
| **Foothill trading post** | land | at the foot of the mountains, on the side facing Great House land; the one place lowlanders meet the clans easily | kept, claimed (by the Great House or by bandits) |

- **In town:** none.
- **Nearby:**
  - a guide for hire
  - a lowland prospector
  - a hurt traveller under guest-right
  - a Great House surveyor
- **Hooks:**
  - a lowlander has broken guest-right
  - the trading post's elder is taking bribes
  - *(secret)* a pass hut's carvings show one people, not four

### The Rooted
| Site | Setting | Placement | Conditions |
|---|---|---|---|
| **Listening grove** | land | a ring of trees; likeliest near the Overgrowth, but it can appear anywhere | hidden, growing, discovered, burned out |
| **Hollow farm** | land | at the green's edge: a family that stopped resisting | hidden, discovered |
| **Cell** | town | *(secret)* a cellar where the sworn meet | hidden, discovered |

- **Rare and alarming:** a "seed" grove far from the green.
- **Nearby:**
  - a grieving visitor
  - a Warden watching from cover
  - an inquisitor hunting them
- **Hooks:**
  - a respected person visits a grove at night
  - the voices in the grove ask for one person by name
  - a new grove has sprung up where no green should be

### The Writers
| Step | Site | Placement and conditions |
|---|---|---|
| **Hall** | **Reading room and copy house**, with a branch of the Lending Wall | large or great towns |
| **Contact** | **Letter-writer's stall**: writes letters for those who can't, and reads every one | smaller towns |
| **Field** | **Survey camp** | at the Overgrowth's edge, near Warden towers. Active, abandoned, or gone wrong |
| **Field** | **Dig** | at a ruin, often with a Lantern contract camp guarding it. Active, or gone wrong |
| **Field** | **Lone scholar's tower** | remote ground. Kept, or abandoned |
| **Field** | **Message relay** | a pigeon loft on a high point between halls. Kept |

- **Hooks:**
  - a letter-writer is selling what she reads
  - a dig found something the guild wants buried again
  - a scholar never came back from the green

### The Lantern Company
| Step | Site | Placement and conditions |
|---|---|---|
| **Hall** | **Chapter house**, with its own lamp room of local contracts | large or great towns |
| **Contact** | **Contract board** at an inn, kept by one veteran who vouches for newcomers | smaller towns |
| **Field** | **Road fort** | a dangerous stretch of road, paid for by whoever's caravans use it. Manned, abandoned, or under contract this season to a named faction |
| **Field** | **Contract camp** | temporary: a beast hunt, an escort halt, a guard on a Writers' dig |
| **Field** | **Lamp grave** | where Lanterns died holding a contract; someone keeps the lamp lit |

- **Hooks:**
  - two chapter houses hold contracts on opposite sides of the same fight
  - a road fort's paymaster has stopped paying

### The Old Hands
| Step | Site | Placement and conditions |
|---|---|---|
| **Hall** | **Back rooms behind a respectable front** (a lineage registry, a pawn house) | First Peoples towns, Great House large towns, where they sell to lords, and **every other large or great town**, behind an old family's factor *(Claude, for v6b)* |
| **Contact** | **An old family's steward** who "knows people" | First Peoples towns; Great House towns, where they sell to lords; Open Hand port towns, where an old family keeps a factor |
| **Field** | **Smuggling cache** | along the Alliance's closed borders. Active, burned (exposed), or abandoned |
| **Field** | **Quiet crossing** | a way past any closed border: the Alliance's, a lord's toll line, a march |
| **Field** | **Safehouse estate** | a half-empty country house in First Peoples land |

- **Hooks:**
  - a steward is being squeezed by both his house and the syndicate
  - a cache was found, and someone has to take the blame

### The Night Market
| Step | Site | Placement and conditions |
|---|---|---|
| **Hall** | **A fence's shop** | Open Hand port towns, and **every other large or great town**, behind a card game or a pawn counter *(Claude, for v6b)* |
| **Contact** | **A dockhand or a card game** that knows whom to ask | any port, and most toll towns |
| **Field** | **The night market itself** | sets up somewhere different each time: a barn, a beached hulk, an empty warehouse |
| **Field** | **Smugglers' cove** | shared with the Far Sails |
| **Field** | **Toll-dodge trail** | a path around a toll town such as [Dir]road |

- **Hooks:**
  - the next market's location has leaked
  - a toll-dodge trail has been found by the toll town's lord

**Turf:** where the Old Hands' ground meets the Night Market's (Great House towns, and Alliance border towns), roll a **contested site**: a tavern both use, or a warehouse both claim. Joining one syndicate makes these sites hostile.

### The Dueling Society
The canon has them with no presence on the world stage, so their sites are sporting, never political: they travel the circuit, not the halls of power.

| Step | Site | Placement and conditions |
|---|---|---|
| **Hall** | **A ring** in a tavern yard or an old courtyard | large or great towns. Popular, banned (by a lord or by the Holy House), or tolerated by a lord who likes to bet |
| **Contact** | **The ladder-keeper**, a fighter who keeps the town's ranking list | smaller towns |
| **Field** | **Circuit stop** | a ring that travels with fairs and market days |
| **Field** | **Challenge stone** | at a crossroads: anyone may call out a member here |
| **Field** | **Famous ground** | where a legendary bout was fought; members come to see it |

- **Hooks:**
  - someone keeps sabotaging the bouts
  - an old-school fighter has turned a friendly bout into a killing

### Guild reach
Which territories each guild's presences can sit in. A guild story must touch at least 3 regions, 2 outside its headquarters' region (`quest-faction-stories.md` §8b), so the placement plan reads this table first and rejects a plan the guild can't reach.

| Guild | Hall | Contact | Field sites | Reach |
|---|---|---|---|---|
| **Writers** | large or great towns, any territory | smaller towns, any territory | the Overgrowth's edge; ruins anywhere; remote ground; high points between halls | every territory |
| **Lantern Company** | large or great towns, any territory | smaller towns, any territory | dangerous roads; hunts and escort halts; lamp graves; Writers' digs | every territory |
| **Old Hands** | First Peoples towns; every large or great town (v6b) | First Peoples towns; Great House towns; Open Hand port towns | the Alliance's closed borders; any closed border (a lord's toll line, a march); First Peoples country estates | First Peoples, Great House, Open Hand ports, and every big town's hall. Thinnest guild in the field: a story leaves its region through big-town halls, Open Hand factors and quiet crossings |
| **Night Market** | Open Hand port towns; every large or great town (v6b) | any port; most toll towns | wherever the moving market sets up; coves on any coast; trails round toll towns | Open Hand, Great House toll country, every coast, and every big town's hall |
| **Holy House** (v6b) | a chapel or parish shrine in every large or great town | parish shrines and tithe-houses in smaller towns | wayside shrines on roads, crossings and hilltops (never in Last Wave land); inquisitor camps on roads near suspect towns | **realm, for story purposes** *(Claude, for v6b)*: it already reaches the Vale and the Crossing. Field sites thin out with distance from First Peoples land |
| **Dueling Society** | large or great towns, any territory (banned in some) | smaller towns, any territory | circuit stops; challenge stones at road junctions; famous grounds | every territory |

- **Land with no towns** (the Last Wave's high ground, the Highfolk's mountains, the Overgrowth) holds guild field sites only, never halls or contacts.
- **Every large or great town has a hall of every joinable faction** (v6b): six halls per big town. A hall behind a front (the syndicates' outside their home ground) shows its front, by *Rolls read secrets*.
- **Contested sites** (the Turf, above) sit where the two syndicates' reach overlaps: Great House towns and Alliance border towns.

---

## 6. How the quest generator uses this
- **Presences** (§5b) are registry places too. A quest that needs "a shrine" or "a Lantern contact" fills its slot from one, or creates one that the placer then puts down by its rule.
- **A site's condition can change in play.** When a story changes a site publicly (a ring's licence revoked, a chapter house's yard occupied), that is a state fact on the site, and other cards there read it (locations, *Settlement context*).
- **One-off jobs at presences** use the giving faction's profile: guilds in `quest-guild-kits.md`, minor factions in §7.
- **Fixed places** are registry entries tagged *landmark*. Their hooks are **facts**: public ones are free to use, while *(secret)* ones are hidden facts that only a main story's revelation (or a faction story's, for its own places) can expose.
- **Placement order:** generate the land, then place territories by §1a–1d, then fixed places, then generated holdings clustered around them.
- **What the land generator must provide** for this to work is listed in `worldgen-requirements.md` (a stub to build on later).
- **Seats** fill each faction's `seat` field. **Reach** follows from where its places are:
  - several territories → realm
  - one territory → region
  - one place → community
- **Set pieces:**
  - **Siege:** Breakwall, Textrim's Height, the Green Wall
  - **Succession:** Longwall vs [Dir]road, the Ninth Ring vs Firststone
  - **Schism:** the Mount vs the Low Shrine, now only as the Holy House story's crisis C1 (`quest-guild-kits.md` §6): a faction with a faction story is never the main antagonist's side, so no main-story Schism falls here
  - **Embassy:** Reefhold to the Anchorage of the Wide Bay, **parked** while sea routes are (rulebook, *Entry specs*): the Anchorage is offshore
- **Foils become places that answer each other:** Openharbour's Unchained Quay vs the Chainmarket's Block; Reefhold vs the Anchorage; Longwall vs [Dir]road.

---

## 7. Minor-faction profiles
Light profiles for the minor factions, so their presences (§5b) can give one-off jobs. (The Holy House's is now a full profile, in `quest-guild-kits.md` §6.) They follow `quest-faction-stories.md` §2–3, like the guild profiles in `quest-guild-kits.md`, but shorter. They are profiles only: no story kits.

**Rules for every light profile:**
- **Signature premises are dressings.** Each one is built on one base premise of its family. It keeps that premise's requirements, ways, twist row and separation, adds what's listed (a requirement, one extra way), and may swap one requirement where it says so. Only this faction's cards roll it.
  - **No truth tables of their own.** A minor-faction signature rolls its base premise's truth rows, with those rows' shapes and *(confirming)* marks (rulebook, *Shapes*). Its `truth_row` names the base premise's row (`Rescue: guilty`), and it counts against that row's cap.
  - How many must be signature jobs is the per-presence quota (locations, *Signature quota*), where a small presence's first cards are free.
- **No faction twists yet.** The family twist tables apply.
- **Canon secrets stay late-game.** No card exposes a *(secret)* hook from §3 or anything in `hidden-lore.md` (guild-kits, *Rules for every kit*).
- **Public front.** Hidden presences give jobs under their cover (§5b, *Rolls read secrets*).
- **Where they're offered:** at the faction's presences (§5b) and fixed places (§3). The Last Wave and the Highfolk have no presences in towns, so their jobs are offered out in the land. **A fixed place that gives faction jobs** (the Soft Hollow, the Green Wall) follows the same signature quota as a presence.

### The Holy House
*Moved (v6b).* The Holy House is joinable since v6b (Laz), so its light profile here became a full guild-style profile with a story kit: `quest-guild-kits.md` §6. Its weights, methods, code (and the priestess's *never turn the hungry away*), vocabulary and motives carry over unchanged; its two dressings (*The tithe unpaid*, *The offering box*) are now full signature premises with truth rows there. One-off jobs and hall jobs at Holy House presences use that profile.

### The Last Wave
*The newest Qotiro: warrior enclaves seen as a threat. Depth: fierce family loyalty, which runs one way. Leaders: Textrim, High Priestess; Totkit Sea-crosser, war-chief.*
- **Weights:** high: Contest, Threat, Rescue · normal: Dispute, Reckoning, Passage, Investigate · low: Acquire, Restore.
- **Methods:** Contest: prepare, edge · Threat: confront · Rescue: force.
- **Signature premises:**
  - Contest ***Stand in the Circle***, on *Champion's bout*: an outsider must stand against a fighter before the enclave will hear them (a plea for grain, a lost flock, a truce). *Swaps:* the proving custom stands in for the oath, as the open rules do on *The ladder*. *Adds:* a proving site (the Circle, a duel circle, a lone smoke hall). *Way:* lose well, so the loss shows you stood your ground and earns the hearing anyway (stakes).
  - Dispute ***Grain for respect***, on *Broken agreement*: a lowland grain deal with an enclave has gone sour, and each side says the other broke it. *Adds:* a lowland trader with a trade link to the enclave. *Way:* carry the grain up yourself and eat at their fire (deal).
- **Code:** **Face them; family first.** Removes bribe ways, and any choice ending that leaves one of their children, elders or wounded behind.
- **Vocabulary:** things: forge salt, blades, grain, sheep that "wandered off", the priestesses' fire · sites: watch cairns, hunting camps, lone smoke halls, duel circles · clients: lowland traders who learned their customs, earlier-wave Qotiro who came to plead or warn, mixed couples, enclave families.
- **Motives:** reasons about family, honor and need; culprit motives *desperation* and *revenge*.
- **Depth used:** family loyalty, and its flaw. The hungry-winter hook counts toward Textrim's Height's hook cap (`quest-locations.md`, *Per-place caps*).

### The Far Sails
*Horaro who left the Alliance: ships, refuge, and a sign it's cracking. Shadow: smuggling for the Night Market. Leader: Nemu of the Wide Bay.*
- **Weights:** high: Passage, Rescue, Acquire · normal: Investigate, Restore · low: Dispute, Reckoning, Threat · never: Contest.
- **Methods:** Passage: route, cover · Rescue: stealth, price · Acquire: trade, deceive.
- **Signature premises:**
  - Passage ***Out to the fleet***, on *Fugitive*: a runaway, or a Tide House child sent to the Sails, must reach a landing and a boat. *Adds:* a Far Sails landing or cove within near. *Way:* signal from the landing with a pilot's mark and wait out the night for the boat (route; fills the pilot, who names a price). The beacon code itself is never stated. *Filtered out until Laz opens the sea* (rulebook, *Entry specs*): the runaway's way out is a boat.
  - Restore ***A cousin in the fleet***, on *Estrangement*: a Tide House family and the kin who sailed away; the deadline is the fleet putting out. *Adds:* a kin link between a Tide House person and a Far Sails one. *Way:* bring them together aboard a hulk house, neither side's ground (persuade).
- **Code:** **Nobody is handed back.** Removes ways and choice endings that return a runaway, bondsman or deserter to whoever they fled (Passage's *handed to the pursuer*).
- **Vocabulary:** things: boats, manifests and what isn't on them, pilots' marks, the list of sheltered names · sites: landings, smugglers' coves, hulk houses, pilots' halls · clients: Tide House relatives, runaways, Night Market buyers, port merchants who need a pilot.
- **Motives:** reasons about freedom, kin and need; culprit motives *greed* and *desperation*.
- **Depth used:** the smuggling shadow.

### The Binders
*Mainly Qotiro slavers, selling to the lords. Depth: reluctant; many see it as the only door to acceptance. Leader: Paqax Chain-breaker.*
- **Weights:** high: Investigate, Passage, Dispute · normal: Acquire, Reckoning, Rescue · low: Threat, Restore · never: Contest.
- **Methods:** Investigate: follow, provoke · Passage: force, arrangement · Dispute: deal.
- **Signature premises:**
  - Investigate ***Who opened the chains***, on *Disappearance*: captives slipped away from a waystation or pen, and the Binders want to know who helped. *Adds:* a waystation or holding pen; the missing are its captives. *Way:* read the tokens at the Last Well to see whose families came (examine). Finding the helper sends the captives back; that's the dilemma.
  - Dispute ***Whose bond?***, on *Broken agreement*: a lord's buyer and a debt broker both claim one bonded person's papers. *Adds:* the bond paper is the agreement thing. *Way:* ask the bonded person whose hand made the mark (truth; fills them).
- **Code:** **Stock is never damaged.** Removes ways that put captives in harm's way (storming a pen, fire, a fight among the captives).
- **Vocabulary:** things: bond papers, shackles, debt ledgers, routes · sites: waystations, raid camps, holding pens, debt brokers · clients: lords' buyers, debt brokers, caravan guards, families buying someone back.
- **Motives:** reasons about need and acceptance; culprit motives *greed* and *desperation*.
- **Depth used:** the reluctant Binder (the hook pool's *a Binder wants out*). The Chainmarket's night buyers are secret and never used.

### The Wardens
*A small order holding the line against the Overgrowth, begging the powers for help. Depth: almost every Warden lost someone to the green. Leaders: Haʻo and Hoqa of the Green Wall.*
- **Weights:** high: Threat, Rescue, Restore · normal: Passage, Investigate · low: Acquire, Dispute, Reckoning · never: Contest.
- **Methods:** Threat: protect, confront · Restore: gather, craft · Rescue: force.
- **Signature premises:**
  - Investigate ***The cairns lie***, on *Sabotage*: someone is moving the marker cairns back to make the green look slower. *Adds:* a marker cairn presence. *Way:* walk the old cairn line with the Warden who set the stones (examine; fills that Warden).
  - Restore ***The tower's gone quiet***, on *Broken works*: a watchtower's signals have stopped, and the line has a gap. *Adds:* a watchtower presence; *the ground moved* reads as roots under the footings. *Way:* hold the tower yourself for a night so the line stands while it's mended (substitute).
- **Code:** **Nobody is left to the green.** Removes ways and choice endings that leave a living person inside the Overgrowth.
- **Vocabulary:** things: the Names Post's names, cairn stones, salvage, signal fires, begged coin · sites: watchtowers, marker cairns, salvage camps, recruiting posts · clients: edge farms that won't leave, Writers studying the green, towns asked for coin, families of the lost.
- **Motives:** reasons about grief, duty and need; culprit motives *desperation* and *cover-up*.
- **Depth used:** the grief under the jokes. Hoqa's path into the green is secret and never used.

### The Highfolk
*Roduro mountain clans the Empire claims; they see themselves as free. Depth: old-fashioned pride. Leader: Shihi of the High Snows.*
- **Weights:** high: Passage, Reckoning, Rescue · normal: Threat, Dispute, Investigate, Restore · low: Acquire, Contest.
- **Methods:** Passage: route, arrangement · Reckoning: amends, justice · Rescue: force.
- **Signature premises:**
  - Reckoning ***Guest-right broken***, on *Betrayal*: a lowlander sat at a clan fire and broke guest-right (stole, struck, guided a surveyor in). *Swaps:* guest-right counts as the ally link Betrayal needs. *Adds:* a pass hut or the Guest Fire. *Way:* bring him back to the fire and let the elders hear him (amends).
  - Passage ***Over the pass***, on *Fugitive*: someone must cross the mountains before the snow, with a pursuer behind. *Adds:* a pass hut on the route. *Way:* sit them at the pass hut's fire, where guest-right holds the pursuer off for a night (arrangement).
- **Code:** **Guest-right is sacred.** Removes ways that strike, seize or deceive anyone while they sit at a guest fire.
- **Vocabulary:** things: furs, mountain ore, guides' marks, boundary cairns · sites: pass huts, boundary cairns, summer camps, foothill trading posts · clients: lowland traders and prospectors, hurt travellers under guest-right, Great House surveyors.
- **Motives:** reasons about honor, custom and kin; culprit motives *preservation* and *cover-up*.
- **Depth used:** pride and old courtesy. The Womb and the pass-hut carvings are secret and never used.

### The Rooted
*Sworn to Qotihìqì; they would cover the island with the green. Terrible appeal: the dead speak through it.*
- **Weights:** high: Restore, Acquire · normal: Rescue, Investigate, Passage · low: Dispute, Reckoning, Threat · never: Contest.
- **Methods:** Restore: persuade, substitute · Acquire: earn, deceive.
- **Signature premises:**
  - Restore ***A voice in the grove***, on *Estrangement*: two living people stopped speaking after a death; a listener offers them the dead one's voice. The deadline is an inquisitor coming to burn the grove. *Adds:* a listening grove. *Way:* take them both to the grove to hear the dead together (persuade).
  - Acquire ***What the grove asks for***, on *Won't sell*: the voices ask for a dead person's keepsake, and a relative won't part with it. *Way:* let the holder hear the voice that asks for it (earn).
- **Code:** **Never cut the green.** Removes ways and choice endings that burn, cut or cull growth.
- **Givers:** always the public front (a grieving visitor, a hollow-farm family, a respected neighbour). The giver's oath is a secret fact, and no card makes anyone join (joining is one-way).
- **Vocabulary:** things: keepsakes of the dead, cuttings, seeds, names spoken in the trees · sites: listening groves, hollow farms, cells · clients: the grieving, edge families who stopped resisting.
- **Motives:** reasons about grief and love; culprit motives *preservation* and *desperation*.
- **Depth used:** the dead return. The Heart and what lies at it are secret and never used.

---

## 8. Decisions made without Laz
- **v6b (for Laz's guild changes, 27 Oct):**
  - *Holy House hall:* every large or great town has a Holy House chapel or parish shrine, and it is the house's hall. Its routine cards, reveal and hall jobs are given there.
  - *Holy House reach:* realm, for story purposes. It already reaches the Vale and the Crossing (the Mount, the Low Shrine, parish shrines in Great House towns).
  - *Syndicate halls in every big town:* Laz's v6b change has every guild keep a hall in every large or great town, and canon says the guilds are "present in every large town". So the Old Hands and the Night Market now keep a hall behind a front in every big town outside their home ground. Their field sites and contacts stay where they were.
  - *The Holy House's light profile* moved to the guild kits; this doc points there.
- **Open for Laz:**
  - Is a Holy House chapel in Openharbour and Carter's Rest (Open Hand towns) right? The church is Roduro, and those towns are mixed.
  - Should the syndicates really keep halls in every big town, or only contacts there? If only contacts, their stories can't meet the "3 big towns" rule without more halls, and the rule needs an exception for them.

## 9. Changelog
- **2026-10-27 (v4.2), v6: World C fixes.** The signature quota points to locations, and fixed places that give faction jobs follow it (§7; L2B-3); a story's public change to a site is a state fact other cards read (§6; PD-C2). Follow-up: minor-faction signatures use their base premise's truth rows and shapes (§7).
  - v6b: Laz's guild changes (27 Oct): a hall of every joinable faction in every large or great town (§5b ladder, guild reach); the Holy House climbs the guild ladder, with a chapel or parish shrine as its hall in every big town and realm reach for story purposes (§5b); Old Hands and Night Market halls widened to every big town (§5b); the Holy House's light profile moved to `quest-guild-kits.md` §6 (§7); Schism set piece points to its crisis (§6); new §8 *Decisions made without Laz*.
  - v6b audit: headquarters sit in a large or great town (§1b); the Mount's Schism is the Holy House crisis only, and the offshore Embassy is parked with sea routes (§6). v6b audit decision: until the sea opens, the Anchorage, the Lantern Rock and islet-only Far Sails holdings take no cards (§3), and *Out to the fleet* is filtered out (§7); both point to rulebook, *Entry specs*.
- **2026-10-26 (v4.1), audit fixes.** Lanternhall sits in a large town, and its hook is a lord's feud, not a war (§4); Old Hands halls, contacts and quiet crossings widened past First Peoples land (§5b); a Lantern contract camp at Writers' digs; Dueling sites read "sporting, never political"; new guild reach table (§5b).
- **2026-10-26 (v4), v5: world-test fixes.** Presence rolls read secrets and show the public front (§5b); minor-faction profiles (§7); §6 points one-off jobs to them.
