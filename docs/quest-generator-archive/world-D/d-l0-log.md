# World D, layers 0–1: log

Built by `D/d-l0-build.py` (adapted from World C's `c-l0-build.py`, brought up to casting v1.2). Seed **4401**, map `map-D.json`, canon `canon-D.json`, `canon_leaders` *mostly* (0.70). Output `D/d-l0.json` with World C's keys: `world`, `layer`, `seed`, `registry`, `facts`, `tensions`, `archetypes`, `faction_leaders`.

## Counts

| | |
|---|---|
| Archetypes | **29**: 23 canon, **6 generated** (21% generated; the setting aims at ~30%) |
| Seats (place `holder`) | **27** |
| Titles (offices at a place, co-leaders) | **12** |
| People | **39**: 23 canon, 16 generated (6 archetype fillers, 10 class members) |
| Facts | 34 (27 archetype keeps, 7 court state); 13 secret |
| Tensions | **3** (count roll 0.1627 → 3) |
| Top people | 1 (Gahìru, canon) |

Every settlement has a seat holder except **the Soft Hollow** (the Rooted have no fixed leader, by canon). Wellstone (lost), the Womb and the Heart (secret) have none. One office per person holds everywhere; the self-check reports no seat, name or reference problems.

## What changed from World C's script (casting v1.2)

- **The filler replaces the canon person** (§4.3). The six replaced canon people (Liʻo, Shihi, Yitolaʻo, Aloqi, Niralea, Hoqa) appear nowhere: seats, titles, links, foils, facts and `faction_leaders` all point at the fillers. `merge.py` drops them.
- **Keyed rolls** (§8). The canon roll is `K[4401, archetype id, "canon"]` (World C walked one running RNG in id order); creation is `K[4401, id, "make", field]`; tensions `K[4401, "court", slot]`.
- **Archetype rows list `keeps` explicitly** (§4): `traits`, `facts` (fact ids), `links` (to other archetypes), `sex` (where the canon role is gendered), plus canon `age`, `voice`, allowed `peoples` and `sex`, `guaranteed`. A generated filler gets exactly its keeps; World C's random "flavour" trait is gone (no rule asks for it).
- **One office per person; co-leaders** (§3): the True Line's face, the Wardens' scout, the Sharp Hands twins and the Dueling Society's sister each hold a title at their co-leader's seat.
- **Hall and contact keepers on demand** (§3): none pre-made. World C pre-made about 60 (chapter captains, reading-room masters, ring-masters, ladder-keepers, back-room keepers, market contacts, letter-writers, board veterans, parish priests, tithe-masters, pilots' hall masters, recruiters, stewards, factors, a fence, debt brokers, toll captains, a harbour-master, the eldest copyist). Later layers create them when a spec needs one, and the first one made holds the post.
- **Tensions count against every major** whose land or people they involve (§5): the Schism counts for the First Peoples, the Quay against the Block for the Open Hand and the Great House, the syndicates' war for the First Peoples and the Open Hand, the Rim war for the Great House.
- **Kin of generated fillers keep the canon relation** and share a family name where one exists (§4.3). The canon relations are links now: the emperor and his sister (World C only had *ally*), mother and son (World C: *kin*), the twins, the adopted siblings, the married Wardens.
- **Names from the naming module** (§7): given names from `assets/lang/names/<people>.ron` by sex (Male/Female/Either), bynames by the people's pattern in `people.ron` (Roduro house, Qotiro deed, Horaro sea-place, Ṭaḍoro road). World C used English placeholders.
- **People, age and voice on everyone**, canon people included.

## Rolls

### Archetypes (`K[4401, id, "canon"] < 0.70` → canon)

| Archetype | Roll | Result | Filler |
|---|---|---|---|
| a-banner-climber | 0.2729 | canon | Shahayuh of the South Road |
| a-banner-general | 0.1958 | canon | Haʻahoqo the General |
| a-binder-head | 0.1933 | canon | Paqax Chain-breaker |
| a-council-head | 0.4010 | canon | Waarua of the Harbour |
| a-dueling-showman | 0.9843 | **generated** | **Waahuho of the Salt Sand** (Horaro, man, young; careful / out_of_the_wind), replaces Liʻo |
| a-dueling-sister | 0.5688 | canon | Kitim Duel-ender |
| a-emperor | 0.5120 | canon | Gahìru of the Great House |
| a-fleet-speaker | 0.0499 | canon | Nemu of the Wide Bay |
| a-free-lord-breaker | 0.5603 | canon | Kotqot Wall-breaker |
| a-high-priest | 0.2543 | canon | Hotorìṭo the High Priest |
| a-highfolk-head | 0.7220 | **generated** (guaranteed) | **Gure of the Herb-Stream House** (Roduro, woman, grown; weary / traveller), replaces Shihi |
| a-lantern-quartermaster | 0.4044 | canon | Lamawina of the Lantern |
| a-market-fence | 0.6312 | canon | Yaweth of the Market Road |
| a-old-don | 0.7094 | **generated** | **Ḍolo of the Four-Ring House** (Roduro, man, old; weary / stone_keep_you), replaces Yitolaʻo |
| a-road-foreman | 0.0413 | canon | Doruḍoqu the Carter |
| a-seated-head | 0.3085 | canon | Tuhuḍo of the Ninth-Ring House |
| a-sharp-broker | 0.5574 | canon | Feyesh of the Silver Road |
| a-sharp-twin-counts | 0.3923 | canon | Kodor Debt-taker |
| a-sharp-twin-deals | 0.1358 | canon | Dorog Coin-counter |
| a-sincere-priestess | 0.8265 | **generated** | **Qeḍuʻata of the Rain-Shore House** (Roduro, woman, old; warm / traveller), replaces Aloqi |
| a-throne-voice | 0.0161 | canon | Litiṭaʻa of the Great House |
| a-tide-speaker | 0.8794 | **generated** | **Momilei of the Deep Reef** (Horaro, man, grown; wry / another_day), replaces Niralea |
| a-true-line-face | 0.2782 | canon | Ruʻo of the First-Stone House |
| a-true-line-schemer | 0.6199 | canon | Garoṭaqa of the First-Stone House |
| a-warden-gate | 0.5559 | canon | Haʻo of the Green Wall |
| a-warden-scout | 0.8763 | **generated** (guaranteed) | **Linuha of the Star Harbour** (Horaro, woman, old; proud / have_you_eaten), replaces Hoqa |
| a-wave-priestess | 0.1465 | canon | Textrim, High Priestess |
| a-wave-warchief | 0.6170 | canon | Totkit Sea-crosser |
| a-writers-leader | 0.1054 | canon | Fiyeweya of No Road |

Both guaranteed archetypes are generated this world: the green's scout and the Womb's guardian are new people, with their secret facts.

### Class members (pre-made: one per seat on the map)

| Seat | Class | Holder |
|---|---|---|
| Elderhollow | house head | Roduhuli of the Three-Ring House (Roduro, woman, grown), Seated Houses |
| Ashring | headman | Ḍeshiha of the Wind-Ledge House (Roduro, woman, grown), Seated Houses, of an old Vale house |
| Shellring | headman | Haewano of the Eel Headland (Horaro, man, old), Tide Houses |
| Oathford | Banner Lord | Laʻetalu of the Bark-Field House (Roduro, man, old) |
| Crownsgate | crown gate-warden | Ḍuhuḍì of the Spring-Cliff House (Roduro, woman, old), employee of the emperor |
| Bogford | Free Lord | Hausethayai of the Quiet Road (Ṭaḍoro, woman, old) |
| Saltmarket | town council head | Liriwu of the Deep Island (Horaro, man, grown), Open Council |
| Wainyard | headman (yard-boss) | Wayafayus of the Dusk Road (Ṭaḍoro, man, grown), Road Crews, employee of the foreman |
| Ironstep | enclave priestess | Pentir Beast-binder (Qotiro, woman, old), employee of the High Priestess |
| the Lantern Rock | rock-keeper | Walalimi of the Quiet Cove (Horaro, young), employee of the fleet speaker |

People were rolled from the place's region mix, except where the seat's faction fixes them (Seated Houses: Roduro; Tide Houses: Horaro; Ironstep: Qotiro woman). The first build gave Ashring a Qotiro headman "of an old Vale house" and Shellring a Roduro Tide-House headman; see rule problem 6.

### Seats and titles

- **Ninth Ring:** Tuhuḍo; title *head of the Old Hands (the Old Gate House)*: Ḍolo.
- **Firststone:** Garoṭaqa; title *face of the True Line*: Ruʻo.
- **the Mount:** Hotorìṭo. **Reefhold:** Momilei. **Elderhollow:** Roduhuli; title *keeper of the Low Shrine*: Qeḍuʻata.
- **Green Wall:** Haʻo; title *first scout of the Wall*: Linuha.
- **Highcrossing:** Gahìru; titles *keeper of the Throne's court*: Litiṭaʻa; *head of the Writers (the Wayhouse)*: Fiyeweya.
- **Longwall:** Haʻahoqo. **Southroad:** Shahayuh. **Breakwall:** Kotqot. **the Chainmarket:** Paqax.
- **Openharbour:** Waarua; titles *head of the Night Market*: Yaweth; *head of the Dueling Society*: Waahuho; *co-head*: Kitim.
- **Counting-House:** Feyesh; titles *partner who deals*: Dorog; *partner who keeps the count*: Kodor.
- **Carter's Rest:** Doruḍoqu; title *head of the Lantern Company (Lanternhall)*: Lamawina.
- **Textrim's Height:** Textrim. **the Proving Ground:** Totkit. **the High Snows:** Gure. **the Anchorage:** Nemu.
- Class seats as in the table above.

### Court state (layer 1)

Count `K[4401, court, count]` = 0.1627 → 3.

| Slot | Candidates (weight) | Roll → pick | Majors used |
|---|---|---|---|
| 1 | all 8 | **t-schism** | First Peoples |
| 2 | banner-race 3, long-shed 2, quay-block 3, rim-war 2 | **t-long-shed** | + Open Hand |
| 3 | banner-race 3, rim-war 2 | **t-rim-war** | + Great House |

**The three tensions:**
1. **t-schism, the Mount against the Low Shrine:** Hotorìṭo has ordered the Low Shrine's soup kitchen at Elderhollow closed, and Qeḍuʻata keeps it open; the Mount calls her way a heresy (grudge, Hotorìṭo → Qeḍuʻata). Counts for the First Peoples.
2. **t-long-shed, the Long Shed vote:** Doruḍoqu has called the Road Crews to vote on striking against the Counting-House's rates; Kodor Debt-taker says any crew that strikes will have its notes called in (grudge, Doruḍoqu → Kodor). Counts for the Open Hand.
3. **t-rim-war, war talk on the Rim:** the Proving Ground's young warriors want a raid below the Rim, and Totkit forbids it until the stores are counted; Textrim will hear any offer of grain made with respect. Counts for the Great House (the lowlands below the Rim are the Free Lords' frontier).

The tension rows (all 8, with their weights and majors) are in the build script. Tagged people carry `tension: <id>` and the tension's facts.

## Rule problems

1. **Kin with a personal byname** (casting §4.3 vs rulebook *Name check*). The generated showman Waahuho is the adopted sibling of canon Kitim *Duel-ender*. Kitim's byname is a deed, not a family name, so "a family name shared with that kin" has nothing to share. The same would happen to a generated Sharp Hands twin (Dorog Coin-counter and Kodor Debt-taker carry deeds), and to any Qotiro pair, since the Qotiro pattern (latest deed) has no family name at all. Kept the filler's own byname and logged it. The rule needs a line for cultures without family names.
2. **Does a generated filler keep the canon voice?** Casting §4 puts the canon voice in the archetype row; §7 says "canon archetypes keep the voice in their data row", and "an archetype keeps its canon age". Read: age is kept by any filler, voice only by the canon person; a generated filler rolls a voice from its own people's habit table (a Horaro filler can't use a Roduro habit). Ambiguous as written.
3. **Canon voices and ages aren't written anywhere I may read.** `quest-voices.md` is a Project doc, not in `/home/claude/quests/`. I wrote each row's canon voice and age from `leaders.md` (the foreman *blunt*, the speaker *warm*, Nemu *old*). World C wrote different ones for several (the foreman was *weary*). They need one source.
4. **Is the sex of a role fixed?** Casting §2.1 fixes sex "only where canon fixes it", and §4 lists "the peoples and sex the role allows" without saying when a role allows only one. The first build fixed only the High Priestess, and so produced a male "sincere priestess" and a female "old don". **Fixed (coordinator's call):** a role that canon genders keeps that sex as a `keeps` item. That covers mother and son (True Line), the emperor and his sister, husband and wife (Wardens), brother and sister (Dueling Society), the priestesses, the freed bondswoman, the don and the war-chief (canon "a male war-chief"). The showman, the sincere priestess and the old don were re-rolled to match (same people, new given names from the right list; links, seats and facts follow). Roles where only the canon person's *description* is gendered, such as the Tide speaker ("a Horaro woman") and the Highfolk head ("a Roduro woman"), stay open, so this world has a male Tide speaker. Where that line falls needs writing into casting §4.
5. **Co-leaders on two seats** (casting §3, rulebook *Seats*). The Banner Lords' two leaders each hold their own seat (Longwall, Southroad), and the Holy House's hold the Mount and the Low Shrine. "The other holds a title at the same place" can't apply without breaking one office per person; each kept its own seat. Faction-stories §5 says the same for the Holy House only.
6. **Who may hold a seat?** Casting §7 rolls people from the home community's mix, and the class row gives weights. Nothing stops a Qotiro headman "of an old Vale house" in a Seated Houses village, or a Roduro headman of a Tide House village, both of which the first build produced. Fixed by giving those two seats the faction's people. A class row (headman) can't know which faction's village it serves, so the people weight should come from the seat's faction where canon ties a faction to a people (Seated Houses Roduro, Tide Houses Horaro, the Last Wave Qotiro).
7. **Which posts are pre-made?** Casting §3 names lordships, council seats, temples and guild HQs as pre-made, and hall and contact keepers as on demand. It doesn't say where **toll captains**, a **harbour-master** (rulebook §3's *wharf-master* example) or the **Lantern Rock's keeper** fall. I made toll captains and the harbour-master on demand (the map lists no such post), and pre-made the Lantern Rock's keeper (a fixed faction place with its own hook). Parish shrines are halls in map-D (*Holy House chapel = parish shrine*), so parish priests are on demand. Is a parish shrine a "temple"?
8. **The Soft Hollow has no holder.** The Rooted have no fixed leader in canon, so no seat was filled. "Every lordship, council seat, temple and guild headquarters is filled" didn't force one, but the Rooted's faction seat is empty.
9. **Map text names replaced canon people.** `map-D.json` lists "the Low Shrine (Aloqi)" at Elderhollow, and `faction-places.md` hooks name Yitolaʻo, Shihi and Hoqa. In this world those people don't exist (casting §4.3). The map summary should name the role, not the default.
10. **At most one tension per major caps the court** (casting §5). With three majors and every tension now counting one or two of them, a 4-tension court is reachable only if every pick is a one-major tension. This world rolled 3, so it didn't bite. A world that rolls 4 and picks the Quay or the syndicates first can never reach 4.
11. **Archetype canon share** (casting §4): 23/29 canon (79%) against "about 70%". It's a per-archetype roll, so it drifts. Fine as written; noted.
12. **Stems** (rulebook *Name check*): the build keeps given-name stems unique **world-wide** among generated people, and off canon stems, which is stricter than "checked locally". That leaves later layers room, since their local checks never clash with layer 0.

## Story notes

- Both guaranteed late-game roles are new people: the scout of the green (Linuha, Horaro, Haʻo's wife) and the Womb's guardian (Gure). The secrets are kept on them.
- The Holy House split is live (t-schism) with a generated sincere priestess, Qeḍuʻata. Its Schism crisis (guild kits C1) and the main story's Schism set piece both build on a public feud here.
- No Great House court politics this world: the Banner race lost to the Rim war on the Great House's slot. The emperor's double and the Throne's lies stay quiet background secrets.
- The Dueling Society's founders are now a Horaro brother, Waahuho, and his adopted Qotiro sister, canon Kitim. The canon "Roduro brother" is gone, so the adoption no longer bridges Roduro and Qotiro, which "convince the Qotiro" leaned on. Nothing in the keeps carries that.
