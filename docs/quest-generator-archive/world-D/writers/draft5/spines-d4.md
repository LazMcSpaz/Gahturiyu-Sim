# The Writers: authored spines (v8, draft 4)

Sixteen spines, two per kit enemy W1–W8, in the format of `fixlist-v7-writers.md` §1. Each spine is one guild plot written whole. The generator picks a spine, then fills its slots and its colour; it never picks a spine's parts separately. The spine's own text rules where it differs from a kit entry: a kit move, odd job or crisis named here is used **with the spine's reading**, never with the kit's other readings or variants.

Canon followed: `world-factions.md` (the Writers, the Wardens, the Holy House), `leaders.md` (the leader is always on the road and never at the Wayhouse), `faction-places.md` (the Wayhouse, the Lending Wall, reading rooms, stalls, digs, survey camps, towers, relay lofts; the Wardens' cairns). Invented canon is marked *(Claude)* and kept to what a plot needs. No spine touches `hidden-lore.md`, the Wayhouse's empty shelf, or Highcrossing's sealed tower.

**What changed in draft 2** (from `review-v7.md`): every spine now has a field **17 · Placing** (its centre, each slot's kind, owner and bound, and each card's place); the *Placing a spine* rules, *Fresh stories*, *Flavour* and *Rendering* below are new; W2-B is rewritten (*Sold by the Wayhouse*); W2-A's heart is the new **H6**; no hidden-hand enemy wounds anyone openly; no heart is shamed twice; every flavour option that changed a fact is gone; every *a copy in every hall* and *trade what we know* ending has an earlier plant. `roll_samples.py` applies all of it against `map-D.json` and `D/d-l0.json`, and fails loudly on any break.

**What changed in draft 3** (from `review-v7b.md`, and Laz's call of 27 Oct): **a crisis happens wherever the story is** (*Placing a spine* 4), so nobody rides back to the Wayhouse unshown; only W5-A, W6-A and W7-B, whose plots are about the Wayhouse or the leader's hold on it, keep their crisis there. Every deadline is `{days}`, worked out from the map (rule 9). Every person or party a door or an ending names is set up on an earlier card that is always dealt (rule 10). The logic holes the review named are closed in a sentence each (W1-A's burned page against a clerk's memory, and the truce's draft, reply, sealing and lodging in order; W5-B's charge now rests on her copied page; W6-A's court keeps a short roll without the margin grants; W8-B's journal is in the mother's own cipher; W7-A says why a fresh copy helps; W3-B says why a false name voids a seal). The near-twins are pulled apart: W7-A loses its record house, its laid-off copyists and its *stripped* overlap with W3-B; W4-B's lost surveyor is the father, and his surety is a chain and staff, not a journal; W5-A's lost surveyor is the father too. *Unwelcome cargo* opens one chapter in sixteen. Ledger cards on the routine have swaps (field 5, *Swaps*), and *Fresh stories* now keeps stake, blow, chapter openings, move, routine premises and endings. No region name is written into a spine (`{region}`), and every day, house and region slot is filled by the roller.

**What changed in draft 4** (from `review-v7c.md`; each fix is at the root, in the spine text, a slot table or a roller rule, not in one sample):
1. **Slots carry ownership, not only distance.** A lord's land is his *lordship*: his seat, and every place nearer his seat than any other lord's seat in the same territory (*Placing a spine* 11). W2-B's dig, stall and toll are in `{lord}`'s lordship (the toll on the dig's own track, since no map road reaches a Free Lord's bog); W2-A's dig, valleys and hamlet likewise; W6-A's `{tower}` and W7-B's are in the hall's own territory or Great House land, never an enclave; W1's two lords share a major faction, and their lenders never lodge at another faction's chief seat; W7-B's `{mother_town}` has a market; W5-B's `{dig_lord}` and W7-A's tenants are the lordship the place stands in. The roller prints and checks each.
2. **The reveal shows, then adds** (*Rendering*). Field 7 now says what the found thing or the knower's own eyes can show (**Shows**) and what someone adds, with how they know it (**Adds**). W2-B's suit, W6-A's letters and reason, W3-B's copyist and W7-B's old wrong now come with a source, or later.
3. **What the player holds.** Every crisis has a **Held** line: one sentence on why the payload still lands though the player holds a warning or a proof, or the real chance the player had and how the crisis reads it (W4-A's tube holds last fair's list; W7-B's hearing won't hear the watermark from an unsworn hand; W5-A's Wall promise).
4. **Clocks sum every leg** (*Placing a spine* 9). W2-B's clock now counts the summons' ride from Highcrossing, her run to the haven and the lofts' day; W8-B's counts both villages; the roller rounds the sum it prints. W3-B's reveal box comes from her back room, not the copy house's first week; W8-A's levy is voted on the draft maps, to be paid once they are certified.
5. **Status words.** W7-B's apprentice is *barred*, never "struck off" a roll he isn't on, and the teacher's list is unsigned; a stripped player's **Standing** is said at the crisis (W1-A, W4-B, W6-B, W7-A); W6-A's frame rests on the courier's road the player followed.
6. **Flavour holds wherever it is placed.** No place name, no season word, and every outside faction glossed (`roll_samples.py` checks all three); W5-A's lodging is said placeless by the cook and named for the crisis town by the roller; W8-A's cairn night and staff have no year words.
7. **Structural ceilings** (variety): chapter 1 has **three shapes** (field 8), the reveal **five givers** and **four closing forms** (field 7), each capped in a set; the stories that deal a card at the Wayhouse are capped at three (W2-A's and W7-A's routine leave it); only W6-A's heart works from the shapes alone (W1-B's and W7-B's can read); W4-B's heart is a copyist's son, not a third lost surveyor's child, so the lost-surveyor *motif* (W5-A, W8-B), not the H5 type, is the once-a-set heart; that is what pinned W4-A in 17 sets of 21.

## How to read a spine

- **Fields** are numbered as in fixlist §1: **1** id, title, enemy · **2** scheme and payload · **3** stake · **4** heart · **5** routine · **6** odd job · **7** reveal · **8** chapter 1 · **9** chapter 2 · **10** crisis · **11** closing account (in the crisis's endings table) · **12** truths and tells · **13** hall-job hooks · **14** what varies · **15** presuppositions · **16** survey, scout and spy cards · **17** placing *(new)*.
- **Payload** is the concrete thing the scheme achieves if nobody stops it. Every spine marks the act that **is** the payload attempted (⚑).
- **Plants** are written as the noun phrase the reveal repeats, in quotes. ∅ marks the plant that is an absence.
- **Blows**: each option is a target + event + how it fell (the assembly §4 *Doors by how the blow fell* key) + its door. The world's deal picks one (each target + event once per world). Every option says why *this* scheme strikes there.
- **Endings**: each names who pays (`costs`, A4), how it is won (C5: bribe, force, stealth, law, persuasion; or *none* for an ending nobody wins), and the closing account a month on (C2): **heart / place / other side**.
- **Tells** (A2) point and never state.
- **Pressure** climbs 1 → 3 inside each chapter; chapter 2 opens at 2 or more.
- **Pronouns** in a spine are placeholders. The heart is *cast* (see *Hearts*); the summary and the cards use the cast person's.
- **Field 7** *(draft 4)* opens with its **giver** (the leader, the heart, an elder copyist, the ally, or a found letter) and its **form** (how its closing sentence ties the routine's plants together; *Rendering*), then **Shows**, **Adds**, **Repeats** and **Closes**.
- **Field 8** *(draft 4)* names its **shape** (*Chapter 1's shapes*, below).
- **Field 10** *(draft 4)* carries a **Held** line (what the player holds before the payload lands, and why it still lands, or the chance it gave), and, where a blow can strip the player, a **Standing** line.

## Slot vocabulary

Every slot below is filled by the generator under its constraint, and every one is placed by *Placing a spine*. A spine may tighten a constraint; it never loosens one. Each spine's field 17 lists the kind, owner and bound of every slot it uses. World D defaults are given where the map has the place.

| Slot | Constraint | World D default |
|---|---|---|
| `{enemy}` | the kit enemy named in field 1, with its variant; keeps the kit's want, means, reach, shape, reveal kind and side | — |
| `{hand}` | Middle; employer/employee link to `{enemy}` (under a *Within* enemy, may instead link to the heart); fills one hidden role (the odd job) and one phase-3 slot | — |
| `{heart}` | Low or Middle; the kit heart type and variant the spine names, plus the spine's own requirements; carries a **want** (shown on the heart's routine card and once later, never deciding a way) and a **past** | — |
| `{fallback}` | linked to the heart (kin or workmate); stands in only while the heart is held | — |
| `{leader}` | the leader archetype (default Fiyeweya of No Road); always on the road, never at the Wayhouse; gives the reveal off the road, wherever its knower is, only in the spines whose field 7 names her (W1-A, W2-B, W4-A, W8-A: draft 4; she gave seven reveals of eight), and at most three times in a set | Fiyeweya |
| `{eldest}` | the Wayhouse's eldest copyist (Middle, member); keeps its strongroom key-book and reads the leader's letters aloud. The only "eldest" in any spine; a hall's old reader is `{tutor}` | — |
| `{wayhouse}` · `{wall}` · `{strongroom}` | the Writers' headquarters in Highcrossing; the Lending Wall in it; its strongroom. Fixed | Highcrossing |
| `{hall_*}` and other town slots | a place of the kind field 17 names (reading room, stall, town, seat), drawn by *Placing a spine*. **The old rule that 2 of 3 routine towns lie outside the Crown Crossing is withdrawn**: it fought the spines that fix their towns, and drew cards days from their people. Spread across the map now comes from *Fresh stories* (main town) | reading rooms: Ninth Ring, Reefhold, Longwall, Southroad, Chainmarket, Openharbour, Carter's Rest |
| `{reading_room}` | the reading room a spine centres on, in a large town; field 17 says what it must stand near | as above |
| `{stall}` | a letter-writer's stall (map presence) in a smaller town; where field 17 needs one the map lacks, the placer adds it and says why. *(Draft 4)* Where the plot has a lord's men serve paper at the stall, the stall is in that lord's lordship (*Placing a spine* 11) | the Mount, Elderhollow, Oathford, Saltmarket |
| `{dig}` | a Writers' dig at a ruin, a Lantern contract camp guarding it; field 17 may fix its territory (the placer adds one where the map has none, and says why) | the Old Road dig (Great House land) |
| `{camp}` | a Writers' survey camp at the green's edge, near Warden towers | the Edge survey camp |
| `{cairns}` | the Wardens' marker cairns, one a year, showing how far the green had come | the Year Cairns |
| `{watchtower}` | a Warden watchtower on the spine's stretch of edge | Watchtower Ash, Watchtower Thorn |
| `{green_wall}` | the Wardens' fort. Fixed | the Green Wall (held by Haʻo) |
| `{edge_village}` | a village with a Low or Middle holder within a day of the camp | Ashring (held by Ḍeshiha) |
| `{tower}` | a lone building on remote ground, with a keeper (added by the placer when absent); *(draft 4)* in the centre's territory or Great House land, never in an enclave (the Last Wave's Rim, the High Snows, the green, the Far Sails' sea) | — |
| `{loft}` | a pigeon relay loft on high ground by a hall (added when absent) | — |
| `{lord}`, `{lord_a}`, `{lord_b}`, `{court}` | **a seat holder from the world's registry** (World D: `d-l0.json`, `registry.places[].holder`), never invented. Field 17 names the kinds allowed: Banner Lord, Free Lord, Seated House head, Tide House speaker, the Binders' head (who holds the Chainmarket's court), Open Council head. His court sits at his seat. A lord's writ binds only his own lordship (*Placing a spine* 11). *(Draft 4)* Two lords whose feud or border a spine turns on share a **major faction** (a Great House lord and a First Peoples house are a treaty, not a ford feud) | Banner: Longwall (Haʻahoqo), Southroad (Shahayuh), Oathford (Laʻetalu) · Free: Breakwall (Kotqot), Bogford (Hausethayai) · Seated: the Ninth Ring (Tuhuḍo), Elderhollow (Roduhuli) · Tide: Reefhold (Momilei) · Binders: the Chainmarket (Paqax) · Open Council: Openharbour (Waarua) |
| staff (steward, archivist, under-clerk, heirs, captain) | made on demand, and belong to the registry holder of the place they work at (a lord's steward is that lord's; a Warden captain serves under the Green Wall's holder) | — |
| `{magistrate}` | Highcrossing's magistrate, the Throne's officer (made on demand); his writ binds Highcrossing | — |
| `{mount}` | the Holy House's seat. Fixed | the Mount (held by Hotorìṭo) |
| `{days}` *(draft 3)* | a deadline in days, **never a number written in the spine**: the road from where the crisis's parties are to where it falls (field 17's *Clock*), rounded up, plus one day | worked out per story |
| `{region}` *(draft 3)* | the region word of the slot field 17 names (the map's region name), never written into the spine | e.g. the Elder Vale, the Crown Crossing |
| day slots *(draft 3)* | `{truce_day}`, `{hearing_day}`, `{feast}`, `{fair}`, `{vote_day}`: a day's name, rolled from the roller's list; where the spine has a clock, the day is `{days}` days off at the last move | — |
| `{house}` *(draft 3)* | W3-B's house the rolls shame most: a d-l0 house within three days of the court (no faction leader, not the court's holder), else made on demand | — |

A spine's own one-off slots (a mill, a truce day, a border village) are defined in its fields 15 and 17.

### Hearts

The kit's five heart types stand, plus one *(Claude, draft 2)*, so that no type is the heart of more than three of the sixteen spines:

- **H6, holder** *(Claude)*: a freeholder whose land is written in the halls' rolls and who can't read them. *a* holds a farm in a valley whose grant the halls keep · *b* holds a mill or ford in two lords' debt · *c* holds a lease the halls copied for a lord.
- **The lost-surveyor motif** *(draft 4)*: the child of a surveyor who went into the green (W5-A's son, W8-B's son). It is the motif, not the H5 type, that a set allows once: W4-B's H5 heart is now a dead copyist's son, so H5 can be dealt twice in eight without a third lost surveyor.

| Type | Spines |
|---|---|
| H1 mentor | W2-B, W3-B, W5-B |
| H2 fellow recruit | W1-B, W6-A, W7-B |
| H3 client | W3-A, W4-A, W8-A |
| H4 veteran | W1-A, W6-B, W7-A |
| H5 member's kin | W4-B (a dead copyist's son), W5-A, W8-B (both the lost-surveyor motif) |
| H6 holder | W2-A |

**The heart is cast, not drawn.** Its type and variant are part of the spine, so there is nothing to draw between; the casting step gives the heart's sex (unless the spine's kin words fix it: "the son", "his wife"), name, people and age. The roller prints the cast sex.

## The three new premises used here (C1)

Defined in the family docs (v7): `family-investigate.md`, `family-threat.md`, `family-acquire.md`. They suit the Writers, and draft 3 uses them on the routine as well as in the chapters.

- **Investigate *Watch them*:** watch a named person or place where no crime is known yet, and report what they do; the truth is what they are really doing.
- **Threat *Go and look*:** scout a dangerous or unknown place and come back with what is there; the truth is what is really there (Threat, not Passage: a scout carries no cargo).
- **Acquire *Take the readings*:** field work: take measurements with a chain, a staff or a survey book at a site whose keeper won't let anyone measure; the truth is what the readings show (Acquire, not Restore: nothing need be broken).

## Placing a spine *(draft 2)*

Map distances are in days (1 unit ≈ a day's travel). Every number below is a named constant in `roll_samples.py`.

1. **A centre.** Each spine names its centre in field 17: the enemy's seat, dig or reading room, or the court or camp the plot turns on. The placer fills the centre first, and every other slot around it.
2. **Cards sit near their people.** Every card bound to a slot sits **within 1.25** of the place of each person and place on it (field 17's *Within 1.25 of* column). Where a slot needs a feature that nothing within 1.25 has (the nearest reading room to a dig, the nearest stall to a hall), it may stretch to **1.6**, and the placer says so ("stretched: the nearest …"); nothing goes further.
3. **Journeys are marked.** A card the spine marks *(a journey)* may lie up to **3.5** from the centre, and its text says why the player travels. A card bound to no slot (a generic routine job) lies within **3.0** of the centre.
4. **The crisis happens wherever the story is** *(Laz, 27 Oct; draft 3)*: at the story's centre, or at the place the plot comes to a head, and **the giver and the parties are there**. Nobody rides days back to the Wayhouse unshown. Field 17's *Crisis* line names the place and who gives the card there, and the crisis card keeps its bounds like any other. **Exception:** a spine about the Wayhouse or the leader's hold on it keeps its crisis at the Wayhouse (W5-A: its Wall's oldest pigeonholes burn in its yard; W6-A: her order keeps the copy behind its door; W7-B: its own hearing strikes the apprentice off), and its card says why each party is in Highcrossing. Other cards the spine sets at the Wayhouse (routine hall work, mostly) are exempt from the bounds, and each says why any slot person on it is there.
5. **Nearness weights the draw.** Every hall and town pick is drawn among the places its bounds allow, weighted 1 / (0.2 + d)², d the distance to the card's anchor (else the centre). A town already holding another of the story's routine cards weighs a quarter, so routine cards spread where the bounds allow; where they don't, two cards may share a town.
6. **Features come from the map first.** A slot that needs a feature (a fence's shop, a pawn house, a Binder debt broker, a Lantern chapter house, a tithe-house, a Warden recruiting post, a court, a dig in a given owner's land) takes a place whose `presences_in_town` or registry entry has it. Only if none lies within its bound does the placer add the feature to a town that can hold it (size and owner allowed), with the reason printed: "the map's only fence's shop is out of reach; a Night Market town can hold one". A site the map lacks (a ford mill, a dig in Free land, a tower) is added at coordinates the spine gives, inside the right territory, with its reason.
7. **Holders, never inventions.** Lords, courts and owners come from the registry's holders (`d-l0.json`); the map's `owner_faction` and territory decide whose land a place is. Staff are made on demand and belong to the holder of their place. A dying lord (W6-B) is never a faction leader.
8. **Slot checks.** Every slot in field 17 names the place kind and owner it needs; the roller checks each fill against the map and the registry (a seat has a registry holder of an allowed kind; a hall has a reading room; a stall has a stall; a dig in Free land is in `t-free`), and fails loudly on a break. Every slot a spine's text uses is one the roller fills or casts (day, house and region slots included), and an unfilled one fails.
9. **The clock comes from the map** *(draft 3)*. A spine never writes "three days off" or "the night before". Its deadline is `{days}`: the road (map roads at their length, off-road stretches at 1.35 times the straight line) from where its parties are to where the crisis falls, rounded up, plus one day; field 17's *Clock* names both ends. The roller prints the sum, and fails a spine whose text carries a hard-numbered deadline.
10. **Parties are set up first** *(draft 3)*. Every person or party a door option or an ending names sits on an earlier card that is always dealt (a routine card, the reveal, a chapter-1 card or chapter 2's opening), never only on a door option that may not be dealt. Field 17's *Parties* line lists each with its card; the roller checks the dealt door and the rolled endings against it.
11. **Whose land** *(draft 4)*. Distance never decides ownership. A place's **lordship** is the seat it stands at, or else the nearest lord's seat (a d-l0 holder of a court kind) in its own territory; a site the placer adds is in the lordship of the point it stands on. A slot whose plot has a lord's men act there (serve a summons, close a dig, keep a toll), or a lord's grant cover it (valleys, tenants, a dig under a lord's old grant), names that lordship in field 17 and the roller checks it. A hoard, a tower or a refuge is in the centre's territory or Great House land, never in an enclave. A slot named for a trade or a custom (a market woman, a fence, a pawnbroker, a court) is a town of that kind, by its map kind and presences, not merely a settlement in range.

## Chapter 1's shapes *(draft 4)*

The review found one chapter-1 shape under all eight samples (a third card where an opposing proxy holds something and an ally helps for a price). Each spine now has one of three, named in field 8 and checked by the roller (the links, the turn, where the ally is met), and a set deals each shape at most three times:

1. **The price at the turn.** C1-1 · C1-2 *therefore* · C1-3 *but*: the turn is the third card, where an opposing proxy holds or blocks and the ally is met, with a price. (W1-A, W2-A, W3-B, W6-A, W7-B, W8-B.)
2. **The ally first.** The ally is met on C1-1, at the first job, with a want and a price; C1-2 *therefore*; C1-3 *but*: the turn strikes the ally (her cart held, his protest found, her name cried, his master's order), so the player sees what helping costs before the door. (W1-B, W2-B, W4-B, W5-A, W8-A.)
3. **The early turn.** C1-2 is the *but*: the enemy's proxy shows its hand in the middle card; C1-3 *therefore* is the player's answer, where the ally **asks** the player for something (the price is the request, not a sale). (W3-A, W4-A, W5-B, W6-B, W7-A.)

## Fresh stories *(extends mechanics §13)*

`recent_stories` keeps, for each of the last two worlds, not only the kit entries (enemy, crisis, odd job) but, for the Writers, **the spine id, the heart type (H1–H6, without the variant), the lords and courts the story used, and its main town** (the centre's town, or field 17's *main* slot). Canon places and canon people recur across worlds, so places are kept by id, and canon lords by their archetype; a generated lord is kept by his seat.

- All five are **unavailable unless nothing else fits** (the mechanics rule; the older world's come back first).
- **A sample set reads as playthroughs in a row.** The sample roller keeps every earlier sample of the set in memory, not just two, for the spine, the enemy, the lords and the main town, so eight samples are eight different enemies, lords and main towns; a heart type (the parent type, H5 not H5a) is barred for the next two samples and used at most twice in eight, and the lost-surveyor motif (W5-A, W8-B; draft 4: the motif, not the H5 type) and the holder (H6) at most once.
- **Draft 3 widens the memory** (`review-v7b.md`, pattern 5): the stake (S1–S7) at most three times in eight and never twice running; the blow (target + event) at most twice and never twice running; each chapter opening's premise at most twice in its position, and *Unwelcome cargo* once in all; the chapter-1 move at most three times and never twice running; *The ledger doesn't add up* on the routine of at most three stories, *Copy it before it burns* of at most five; endings rolled least-used first, and no ending shown twice. The roller looks ahead (a sample is taken only if the rest of the set can still be filled under the caps), backs up a sample when a later one can't be placed, and relaxes memory only if no whole set fits, saying so (a relaxed set fails the check).
- **Draft 4 widens it again** (`review-v7c.md`, structural ceilings): a chapter-1 shape at most three times in eight; a reveal giver at most three times (the leader gave seven reveals of eight); a reveal's closing form at most three times; the stories that deal any card at the Wayhouse (Highcrossing) at most three (five of eight passed through it); the lost-surveyor motif once. A spine whose cards are fixed at the Wayhouse is taken only while the cap allows; a story that draws the Wayhouse by chance counts too.
- One person is never two roles across spines: W7-A's heart (the struck-off copyist's silent friend) is no other spine's ally (W7-B's ally is now the old hearing-scribe).

## Flavour *(draft 2)*

- **Every flavour list is rolled on its own.** A spine with three lists rolls three options; "the flavour variant" no longer means one list.
- **No flavour option may change a fact another part relies on.** An option may change only what nothing else in the spine reads: a reason voiced once, a person's trade, a name, a colour. Where a flavour list sets words a plant repeats (W2-B's seal flaw, W5-B's word for the ground), the option *defines* that phrase everywhere it appears. Draft 1's options that moved a fact are gone: W4-A's carters' yard and granary (the heart's chest is hidden in his house), W4-A's lists by Wall post (the loft is the odd job), W1-A's toll bridge and stud (the mill is everywhere), and each list that swapped an ally for someone who couldn't do the ally's way (W1-A, W1-B, W2-A, W3-B, W5-A, W6-A, W6-B, W7-A, W8-A, W8-B).
- Every list has at least two options.
- **Every list names the card that voices it** *(draft 3)*: the reveal, a chapter card, the crisis. A flavour nobody says is not rolled.

## Rendering *(summaries now; card text later)*

- **One full sentence per card, naming who does what.** The subject is a role or a name, never a pronoun that could point at two people ("the court clerk accuses the heart, whose copy it is", not "the clerk accuses her friend"). No "X: Y" telegraph.
- **Print the heart's past once,** on the heart's routine card or where the spine first leans on it, and **the crisis's tell once,** on the crisis card. Both are authored; they are the emotional load, and they must reach the page.
- Plants appear in quotes, in the words the reveal repeats.
- A crisis need not close on "X asks: A, or B?"; say what the giver wants and what it costs.
- **The reveal shows, then adds** *(draft 4)*. The knower says only what the found thing, or what the knower saw with their own eyes, can show: that is field 7's **Shows**. Anything more is **Adds**, given in the same sentence with who adds it and how they know it ("the leader adds …, from the Throne's clerks on the road"). A knower never names a person, a plan or a reason the found thing can't show.
- **The reveal's closing sentence takes the spine's form** *(draft 4)*, never "so X, Y and Z all point at …": *laid side by side* (the giver lays the three plants beside the found thing, one by one); *a chain* (each plant explains the next); *the knower's question* (the knower asks the player about the three things the player saw, and the answers are the plot); *the thing names them* (the found document itself says the plants in its own words). Field 7's **Closes** is that sentence's matter.
- **What the player holds** *(draft 4)*. When the player holds a warning or a proof before the payload lands, the crisis says why it still lands (too late, not believed, the holder can't act, no rule lets a Writer act on it) or reads what the player did with it; field 10's **Held**.
- **Status words** *(draft 4)*. Struck off only a roll one is on; one who is not is *barred*. A thing signed is a thing published. A player stripped of rank acts at a crisis on a standing the card names (a witness, the heart's advocate), never as a Writer with rank; field 10's **Standing**.
- **Flavour holds wherever it is placed** *(draft 4)*. A flavour option names no place (the roller fills places), carries no season word (the story's days do that), and glosses an outside faction in the option itself ("a Sharp Hands buyer (the counting-house traders who buy up claims)"). A line said far from where it is true is said placeless ("they sleep at the chapel of the town they burn in"), and the crisis names the place.
- **Words for the write-up** *(draft 3)*: a bookbinder is "the bookbinder", never "the binder" (in the Binders' towns that reads as a slaver); a court holder who is no lord is named by his title ("the Binders' head", "the Binders' head's keep"); the region is the map's word for it (`{region}`); a deadline is the roller's `{days}`.

## Rules every spine keeps

- The enemy fills the crisis's opposing slot; the crisis decides the stake; the heart (or the fallback, while the heart is held) fills its requester, wronged or party slot. A crisis whose heart must act is never dealt after a blow that holds the heart.
- No crisis's family + premise is its own door's. No two spines share a crisis family + premise (all 16 differ; see the variety table).
- Each spine spends at most one writ (paper aimed at the guild).
- Routine cards never name the stake or the enemy. The odd job is never the first routine card and never the heart's.
- Hall jobs named in field 13 use no story cast and no secret; each reads a move through its town's context, with a *before* face.
- Under *the cost* (W1, W6) the blow options come only from the kit's per-enemy list.
- **A hidden-hand enemy (W2, W3, W4, W8) never wounds anyone openly:** no `wounded-men` blow. Rigged harm (`wounded-rigged`) and quiet force stay.
- **No heart is shamed twice.** A spine whose crisis blames the heart in public (W6-B, W7-A) has no `cried-down` blow; where `cried-down` is dealt elsewhere, no ending costs the heart a second disgrace.
- **Endings rest on plants.** *A copy in every hall* needs a copy to send and a hall that would take it; *trade what we know* needs the secret. Each is planted on an earlier card and listed in field 15.
- **No two cards in a chapter share a family**, chapter 2's opening included against every door option; no two routine cards in a row share one, and at least one routine card is a signature premise whatever the swaps give. The roller checks all three.

---
## W1-A · The Truce Held for a Debt

**1 · Enemy:** W1a, the master of `{reading_room}`, the hall where both `{lord_a}`'s and `{lord_b}`'s lenders lodge their bonds. High · *Within* · preservation · `paper` `name` `inside` · region · *the cost* · side: Writers.

**2 · Scheme.** Both houses borrow from the same lenders, and the lenders lodge every bond at `{reading_room}`, so its loan rolls hold both lords' debts. For twenty years its master has sent each lord, unsigned through the Lending Wall, a true debt of the other's whenever either grows strong enough to win. Neither can afford war or peace, and both keep paying her hall to copy the other's rolls. She calls it keeping the balance. *(Draft 2: the hall need not stand between the lordships; it must be neither lord's seat and within reach of both, which is where lenders sit. Field 17 picks the lords to fit the hall.)*
**Payload ⚑ (crisis):** this season the lords mean to end it with a truce. It binds only once it is lodged at `{reading_room}`, the hall both houses' lenders use, on `{truce_day}` *(draft 3: lodged at the lenders' hall, not the Wayhouse, so the crisis is where the story is)*. She sends the last letter, the ford-mill debt, to `{lord_b}`'s steward `{days}` days before the lodging, just time for him to ride in and object. When he objects, she holds the sealed truce in her hall's strongroom "until the debt is ruled", the lodging day passes, and the truce lapses.

**3 · Stake:** S2a, the season's undelivered letters. *Reading:* the Wall is her instrument; whoever sorts it decides what each lord learns, and when.

**4 · Heart:** H4a, `member`, cast: the old clerk of `{reading_room}` who knows its loan rolls by heart (Middle). *Needs:* has copied in that hall's loan rolls for forty years.
- **Want:** to train one apprentice to keep the loan rolls after her.
- **Past:** as a girl she copied the ford-mill debt into the roll herself. *(Printed on R1, and leaned on at the crisis: it is why she can swear to the roll's line, and why swearing costs her.)*

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{reading_room}` | Investigate *The ledger doesn't add up* (signature; *swap* below) | a loan roll page was altered for a grain buyer (culprit *for themselves*); the heart is the old clerk who knows the original. **Heart's card**: she asks whether the player has a steady hand (her want), and says she copied half these lines herself as a girl (her past) | ∅ "**the blank sender column**": letters for the lords' stewards, chalked by the hall's own sorters, with no sender written |
| R2 | from `{reading_room}`'s Wall branch to `{lord_b}`'s seat (a journey) | O3 Passage *Smuggling* (a sealed letter, carried as a will). **Odd job** | carry a sealed "will" from the hall's Wall branch into `{lord_b}`'s lordship, where sealed letters from the other side are searched at his toll | "**a hall seal no hall cut**" |
| R3 | `{mill_market}`, the market nearest the ford | Acquire *Unknown worth* | the miller brings the hall his father's mill tally to be read, not knowing what it is worth to either lord; he says his ford mill "has stood in two lords' debt since before I was born" | "**the ford mill's old debt**" |

*Swap* (the roller's choice, by the profile's weights and the set's memory; the plant is kept): **R1** may be Acquire *Copy it before it burns* (signature): a grain buyer who bought a loan-roll page off a clerk means to burn it, and the hall wants its line copied back first; the heart, who knows the original by heart, is the one who can say the line was ever there (her want and past as above).

**6 · Odd job:** O3b. *How it serves the scheme:* the Hand, her Wall sorter, is the "will"'s true recipient at the steward's gate (he rides the branch's post out to `{lord_b}`'s gatehouse each market eve, which is why he is there again on C1-3); he passes it on unsigned, so the player has carried one of her debt letters into `{lord_b}`'s house through the guild's own post.

**7 · Reveal** (giver: *the leader*; form: *laid side by side*). The leader, off the road at `{lord_b}`'s seat. Acquire *Won't sell*: `{lord_b}`'s retired steward (the present steward's predecessor, not the man of C1-3) keeps a bundle of unsigned letters that ruined his master's every campaign, and won't part with them; the leader asks only to read them.
- **Shows:** the letters themselves: twenty years of true debts of `{lord_a}`'s house, each sent the season `{lord_b}` grew strong, in roll wording and in a hall sorter's chalk; the retired steward tells the old debt's origin (flavour a) from his own years of paying it.
- **Adds:** the leader adds whose hall chalks its post that way (she knows every hall's hand: `{reading_room}`'s, and so its master's) and the master's reason (flavour c), from what the master said to her face at the last hall masters' table; the retired steward adds that a truce is coming, which his old house talks of. The leader is not sorry ("I'd have done it too"), but the master means to stop the truce.
- **Repeats:** "the blank sender column", "a hall seal no hall cut", "the ford mill's old debt".
- **Closes:** the leader lays a letter's chalk beside the hall's "blank sender column", its seal beside "a hall seal no hall cut", and, last, the one debt the bundle never used beside the miller's tally: "the ford mill's old debt", the letter the master has kept back.

**8 · Chapter 1.** *Shape:* 1 · the price at the turn. **Move:** M3 on S2: the master takes the sorting of her hall's Wall branch into her own hands, "since letters have gone astray". *Uses* `inside`, lever theft; *cost* yes (she protects what pays).

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{ford_mill}`, on the border (a journey) | Restore *Broken works* (cause `stripped-for-parts`) | — | after the latest letter the two lords' tenants broke the mill race, each carting stones to its own bank; the miller and his mother, the old miller's widow, who keeps the mill's tally, stand between them. Plant: "**the truce day**": the lords mean to lodge a truce at `{reading_room}` on `{truce_day}`; it binds once lodged there, and a lodging someone objects to is "**held until ruled**" (the miller explains both, since his mill's peace rides on it) |
| C1-2 | `{reading_room}` | Investigate *Watch them* (C1) | therefore | watch who carries letters out of the branch now she sorts it. Plant: "**the back-stair satchel**". Second plant: the sealed truce, agreed at last, has come in through the branch to wait for its lodging day, and the heart made "**the hall's door copy**" of it, as the hall copies every lodging |
| C1-3 turn | `{lord_b}`'s steward's gatehouse | Dispute *Broken agreement* (the season's carriage of the steward's letters, witnessed) | but | a month before the truce was sealed, the steward's clerk paid for `{lord_b}`'s reply to `{lord_a}`'s draft to go by the Wall, and it never came, which nearly sank the truce before it was sealed; the sorter (the Hand, opposing) says it was never handed in, and the steward himself takes the sorter's side. The carriage was paid in grain, from a store the clerk lets slip is kept in the loan roll under the steward's wife's name: "**the steward's grain store, in the roll under his wife's name**". **Ally plant:** `{steward_clerk}`, who wants the truce (flavour b, voiced here, says why); her price: her own name kept off every copy of the branch's carriage book, where the grain payment stands |

**9 · Chapter 2.** Opening (pressure 2): the ford again, Passage *Unwilling passenger*: with the race broken and both banks' tenants coming to blows over its stones, the miller must be got off the ford to `{mill_market}` before he is hurt, and won't leave the wheel his father kept turning. Then the door. *(Draft 3: Unwelcome cargo opened three chapters in the last sample set.)*

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | heart · *disgraced* · cried down for taking a side in public (`cried-down`) | `name` | Dispute *Public offense* (offended: `{lord_b}`'s steward, whose debt she named aloud in the market) | she is the one person who can tie the letters to the rolls; under the cost, discrediting her protects the hall |
| b | player · *framed* · for taking a side (`framed-shame`) | `inside` | Reckoning *Ruin from above* (framer: the master; shield *Protected*: the leader, who'd have done the same and won't strike off a master for keeping the balance) | the sorter swears the player carried `{lord_a}`'s letters free; the player's word on the satchel is spoiled |
| c | player · *stripped of rank* (`stripped`) | `inside` | Restore *Lost good name* (the stain: "reads others' letters") | the same end: whoever watched the back stair must not be believed |

- **The door carries the crisis's plant:** the old miller's widow (the miller's mother, met on C1-1), in its witness or knower slot, says her late husband ground `{lord_b}`'s house's grain free for seven years "to clear the mill": "**seven years' free grinding**".
- **Lesser code-break (B5; kind: destroy a text):** an extra way on the door: scrape out the line in the branch's carriage book that the charge rests on (a: the market clerk's note of the debt she named; b: the satchel's carriage the sorter swears to; c: the one witness entry behind the stain), which clears whoever the blow fell on, and breaks the code doing it.

**10 · Crisis** (at `{reading_room}`, where the truce is to be lodged). **Last move:** `{days}` days before `{truce_day}` the ford-mill debt reaches `{lord_b}`'s steward, unsigned; he rides to `{reading_room}` and lodges a sealed objection with the master, as the truce's lodging keeper. She holds the sealed truce in her hall's strongroom "until the debt is ruled" (the rule planted on C1-1), and the heart, as the hall's oldest clerk, has to write the hold into the strongroom's key-book. The lodging day ends at the hall's evening bell. **A ruling reads the roll, not a clerk's memory** *(Claude, draft 3)*: the hall rules from the page, and the heart's word that she copied the line stands only beside it. The enemy's want survives: the balance holds as long as the hold does.
- **Held:** the player has seen the back-stair satchel (C1-2) and knows the hold rule (C1-1) before the last letter goes, but the letter is a true debt carried unsigned by the Wall's own post, and no rule lets a Writer open a sealed letter or warn one lord of another's debt; so it reaches the steward, and the crisis is the player's chance against the hold itself.
- **Standing:** if blow c stripped the player, the heart asks the player as a witness to the lodging (any witness may speak at a ruling), not as a Writer of the hall; the master can bar a clerk from her strongroom, not a witness from her ruling.
- **Family:** Acquire *Under seal*. Holder: the master (by her hold). Seal type: *held until a ruling*. Holding place: the hall's strongroom (guarded). Target: the sealed truce. Requester: the heart. Present: the master, the steward, `{steward_clerk}` (who rode in with him) and the old miller's widow, come to see her mill's debt ruled.
- **Giver:** the heart, who keeps the strongroom's key-book. *Reason:* the hold sits in her own book, against the line she copied as a girl.
- **Choice:** let the hold stand and keep the guild's balance, or force the truce through, and say by what.
- **Ways:** *code-break:* burn the ford-mill page of the loan roll, so there is no debt to rule (her memory of it rules nothing) · *ally:* `{steward_clerk}` swears to the seven years' grinding before the hall (her price: her name on the copy after all) · *meet the condition:* hold the ruling before the evening bell, the page read aloud, the heart swearing to the line she copied as a girl, and the widow's grinding set against it.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Lodged on the ruling** (obtained and handed over) | `{steward_clerk}` (named; her master turns her off) | law | keeps the rolls and has taken an apprentice / the mill grinds for both banks / the master writes these lords no more letters, and waits |
| **A copy in every hall** (the heart swears before the hall that her door copy matches the sealed truce; the hall lodges her copy in the original's place, and the Wall sends it to every hall) | the heart (she swears against her own master's hold, and is put out of her hall) | persuasion | copies at the Wayhouse, her rolls in other hands / the truce is posted in every hall / the master keeps her hall and her counsel |
| **Burn your copy** (code-break) | the heart (the line she copied as a girl is ash, and what she remembers of it rules nothing; she leaves the rolls) · `{lord_b}`'s house (a true claim gone) | stealth | won't touch a roll again / truce lodged, ford quiet / the master knows who burned it and can prove nothing |
| **Keep the balance** (the hold stands) | the ford's tenants (the race is broken again within the season) · `{steward_clerk}` | none | goes on copying the lines that keep the feud / the mill stands idle / her next letter is already written |
| **Trade what we know** (the steward withdraws his objection, bought with his grain store under his wife's name) | the steward's wife (the secret names her) | bribe | knows what was traded and won't say it / truce lodged / the steward owes the guild and hates it |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *not what it seems*: the "will" is a debt letter for the Hand | a will addressed to a steward, not an heir |
| C1-2 | the sorter takes the satchel out by the back stair on each market eve | the branch's oil book shows twice the lamp oil it used to buy |
| C1-3 | *one side lies*: the sorter took the reply | the reply's seal lies in the branch's spoiled-wax basket |
| door a | *both partly right*: she named the debt, but it is the hall's own line | she quoted the roll's page number, as only a roll clerk would |
| door b | *a witness exists*: the ford miller saw the sorter take `{lord_a}`'s letters | the miller won't meet the player's eye |
| door c | cause *false, spread off the card* (`rumour-planted`) | the story reached a hall in another town before the player did |
| crisis | *a copy* (`a-copy`): the strongroom holds the hall's own door copy; the steward took the sealed original away with his objection | the key-book, in the heart's own hand, reads "one truce, received sealed, *in our hand*" |

**13 · Hall-job hooks**
- *Middle:* at `{reading_room}`'s town, a carter whose letters home now wait a week in the branch wants them found and sent. Reads M3; *before* face: "letters are slow this season".
- *Late:* at a hall near the border, a family wants its ford-side lease copied "whatever comes of the truce". Reads the truce; *before* face: "talk of a truce".
- *Seen in passing:* before the reveal, the master is the fair arbiter of a weights quarrel on a side card in her town.

**14 · What varies.** `{lord_a}`/`{lord_b}`: two adjoining lordships from the registry (field 17), with a feud the spine establishes on C1-1 and R3 (a rivalry link in the registry is preferred where one exists; they may be of different peoples, and the feud is over the ford, not their whole houses). `{reading_room}`: the lenders' hall, neither's seat, where the truce is lodged. `{ford_mill}`: the debt object, on the border. `{truce_day}` (rolled; `{days}` from `{lord_b}`'s seat to the hall). *Flavour* (each list rolled on its own; none moves a fact): (a) the old debt's origin, told on the reveal: a famine year's seed-grain loan, a wedding's cost, or a ransom; (b) why `{steward_clerk}` wants the truce, told on C1-3: her family farms both banks, her son is promised to a girl across the ford, or she buried a brother after the last fight; (c) the master's reason, voiced on the reveal: she watched the last peace here become one lord swallowing the other, or she believes balance is the Writers' oldest duty. *(Draft 1's toll bridge and stud of horses moved the mill, its miller and its flour; its maid and cook couldn't have paid the carriage on C1-3. Both are gone.)*

**15 · Presuppositions:** `{lord_a}`, `{lord_b}` (registry holders) and their feud (C1-1, R3) · `{reading_room}` keeps both lords' loan rolls because their lenders lodge there (scheme; reveal) · its Wall branch (faction-places) · the ford-mill debt (R3) · the heart copied it (her past, R1) · `{truce_day}` and the lodging rule: a truce between two lords binds once lodged at a Writers' hall both use *(Claude)* (C1-1) · the halls' hold on an objected lodging *(Claude)* (C1-1, "held until ruled") · a ruling reads the page, not memory *(Claude)* (crisis) · the order of the truce: draft, reply (C1-3), sealing and coming in to the hall (C1-2), lodging (crisis) · the hall's door copy of the truce (C1-2; *a copy in every hall*) · the steward's grain store under his wife's name (C1-3; *trade what we know*) · `{steward_clerk}` (C1-3) · the old miller's widow, the miller's mother (C1-1), and her grinding (door) · the retired steward, not the present one (reveal) · `{hand}` = her Wall sorter (R2 hidden, C1-3 open).

**16 · C1 cards:** C1-2 Investigate *Watch them*.

**17 · Placing.** Centre: `{reading_room}`. Main town (for *Fresh stories*): the centre's town. Lords and courts kept in memory: `{lord_a}`, `{lord_b}`. Kept in memory too: stake S2, move M3, openings Restore · Broken works, Passage · Unwilling passenger. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{reading_room}`, where the story is; the giver is there (the heart, who keeps the strongroom's key-book); its action falls at `{reading_room}`. *Clock:* `{days}` = the road from `{lord_b}` to `{reading_room}`, rounded up, + 1 (the steward rides in from his master's seat to object); `{truce_day}` is that many days off at the last move.
*Swaps* (field 5): R1 may be Acquire · Copy it before it burns.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{lord_a}` | seat | a High holder with tenants: Banner Lord, Free Lord or Seated House head (d-l0) | seats ≤2.8 apart |
| `{lord_b}` | seat | as lord_a, of the same major faction (draft 4); adjoining (seats ≤2.8 apart) | — |
| `{reading_room}` | hall | a reading room in a large town, neither lord's seat, ≤2.8 from both seats (their lenders' hall, where the truce is lodged), in the lords' own faction's town or a neutral market's, never a rival faction's | centre |
| `{ford_mill}` | site | on the border between the seats (added: the map marks no mill) | midpoint of the seats |
| `{mill_market}` | town | any settlement | ≤1.25 of ford_mill |
| `{truce_day}` | day | the lodging day both lords set (roller: a day name; `{days}` days off at the last move) | — |
| `{hand}` | person | her Wall sorter at reading_room (made on demand; the hall's) | — |
| `{steward_clerk}` | person | lord_b's household (made on demand) | at lord_b's seat |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{reading_room}` | `{reading_room}` | bound |
| R2 | `{lord_b}` | `{lord_b}` | journey (≤3.5 of the centre): carried from reading_room's Wall branch to lord_b's steward's gate |
| R3 | `{mill_market}` | `{ford_mill}` | bound |
| RV | `{lord_b}` | `{lord_b}` | bound |
| C1-1 | `{ford_mill}` | `{ford_mill}` | journey (≤3.5 of the centre): the border, a ride out |
| C1-2 | `{reading_room}` | `{reading_room}` | bound |
| C1-3 | `{lord_b}` | `{lord_b}` | bound |
| C2-0 | `{ford_mill}` | `{ford_mill}` | journey (≤3.5 of the centre): back to the ford |
| DOOR | a: `{reading_room}` · b: `{reading_room}` · c: `{reading_room}` | `{reading_room}` | bound |
| crisis | `{reading_room}` | `{reading_room}` | bound |
| crisis, acted | `{reading_room}` | — | where its action falls |

*Parties set up first* (checked): the master (RV); the leader (RV); the sorter (the Hand) (C1-3); lord_b's steward (C1-3); the steward's clerk (C1-3); the steward's wife (C1-3); the old miller's widow (C1-1); the ford's tenants (C1-1); lord_b's house (R2).

---

## W1-B · The Stone That Walked

**1 · Enemy:** W1c, the master of the survey hall at `{reading_room}`, releasing boundary readings, not debts. High · *Within* · preservation · `paper` `name` `inside` · region · *the cost* · side: Writers.

**2 · Scheme.** Whenever either lord sets a stone on the border between `{lord_a}` and `{lord_b}`, she sends the other, unsigned through the Wall, an old boundary reading out of the court rolls the halls keep, one that puts the line somewhere else. No stone stands; both lords keep paying her hall to survey; neither swallows the border villages.
**Payload ⚑ (crisis):** one reading too many. Sent to `{lord_a}`'s steward the week his masons are at the border, it moves his stone onto `{border_village}`'s fields; `{lord_b}`'s men pull it down; `{lord_a}`'s men answer; `{border_village}` burns (kit C3 on S7, W1c's reading).

**3 · Stake:** S7 (a/b: a market or toll lord's court). *Reading:* the boundary readings are entries in `{lord_a}`'s court rolls; the halls' keeping of them is her weapon, and what the halls lose if it comes out.

**4 · Heart:** H2c, `member`, cast: a letter-writer's apprentice who copies for the relay lofts at night, sheet by numbered sheet. *Needs:* born in `{border_village}`; her mother still farms a strip there; she copies the survey hall's outgoing sheets onto loft paper at speed, by number, never reading them for sense *(draft 4: she can read; only W6-A's heart works from the shapes alone)*.
- **Want:** to bring her mother to live in town before winter.
- **Past:** the headman sent her to the halls at twelve "to learn the letters for us".

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{free_hall}`, any hall within three days | Investigate *The ledger doesn't add up* (signature; *swap* below) | a survey roll was altered to give a miller his neighbour's strip; comparing it with another hall's copy shows the culprit (*for themselves*) | "**two readings for one stone, ten years apart, both in the halls' hand**" (a margin note on the other hall's copy) |
| R2 | `{reading_room}` (the loft beside the survey hall) | Dispute *Broken agreement* (the night-copy terms) | the loft keeper docks the heart's pay for blotted sheets. **Heart's card**: she is saving her night coin to bring her mother to town; the headman sent her to the halls at twelve "to learn the letters for us" (her past) | ∅ "**sheets logged by number, with no name and no sender**" |
| R3 | `{border_village}`'s parish chapel | O1c Acquire *Copy it before it burns*. **Odd job** | a priest means to burn his chapel's tithe book | "**the tithe book's last page: which farms paid to which chapel**" |

*Swap:* **R1** may be Acquire *Take the readings*: a miller wants his strip measured again with `{free_hall}`'s chain before the sowing, against a neighbour who won't let the chain on his side; the hall's copy of the old survey, laid beside the new readings, carries the same margin note (the plant is kept). R3 is the routine's signature either way.

**6 · Odd job:** O1c (thread `answers-to`). *How it serves the scheme:* the Hand, the survey hall's copyist, told the priest that the halls already hold a copy and that the book would draw the lords' quarrel into his chapel. The book is the one record outside the halls that shows where the border ran (which farms tithed to whose chapel), so burning it leaves the halls' readings the only word on it.

**7 · Reveal** (giver: *the heart*; form: *a chain*). At `{lord_a}`'s seat, where his retired surveyor lives; the heart has carried the loft's night sheets there for the seat's steward, and brings the player. Restore *Lost good name*: `{lord_a}`'s retired surveyor is shunned as "the man who lost the border"; the harm is off the card. He keeps every contrary reading that undid his stones.
- **Shows:** each reading came through the Wall within the month of a stone he set, on loft paper, with a loft number and no name; the heart knows the numbers as her own: she copied every one of them at night.
- **Adds:** the heart adds whose post they were, from the loft's night book (every sheet she copies is the survey hall's outgoing post, signed out by its master): the master has been releasing old readings out of `{lord_a}`'s court rolls, and the halls have been steering the border (the hidden side).
- **Repeats:** "two readings for one stone", "sheets logged by number, with no name and no sender", "the tithe book's last page".
- **Closes:** her own number on his reading leads to "sheets logged by number, with no name and no sender"; those lead to the other hall's margin, "two readings for one stone, ten years apart"; and that leads to the one record of the border the halls never held, "the tithe book's last page", which is why it had to burn.

**8 · Chapter 1.** *Shape:* 2 · the ally first. **Move:** M9 on S7: before the whole survey hall, the master dismisses the copyist who asked who has been sending border readings out of `{lord_a}`'s rolls. *Uses* `inside`, lever blackmail (she names an old fault of his); *cost* yes.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | the border by `{border_village}` (a journey) | Investigate *Watch them* (C1) | — | watch `{lord_a}`'s masons at the border, who set out a line no survey hall sent them; `{fault}`, the survey hall's eldest surveyor, who swore every stone he ever set, comes out with the player to see, and across the line `{lord_b}`'s surveyor watches the same masons. Plant: "**a fresh reading in the steward's hand**" (the masons' foreman works from it). **Ally plant:** `{lord_b}`'s surveyor, who wants the line fixed (flavour c, voiced here, says why); his price: the halls survey his master's line too, free |
| C1-2 | `{border_village}` | Dispute *Shared resource* | therefore | the village and the masons quarrel over the strip the fresh reading claims; the heart's mother's strip is in it. Plant: "**a new stone cut and waiting on the cart**" |
| C1-3 turn | `{reading_room}` (within a day) | Acquire *Won't sell* | but | the Hand (opposing) holds his own chain-book, every stone and every reading sent, and won't lend it; and he has shown `{lord_b}`'s steward the ally's name in it beside a Writer's, so the surveyor is ordered off the border: he can still swear the true line, but only in person before a court, and never walk it with the player again (the crisis's ally way) |

**9 · Chapter 2.** Opening (pressure 2): the border at night, Threat *Go and look* (C1): count `{lord_b}`'s men camped across the line. Then the door.

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | heart · *disgraced* · cried down for taking a side in public (`cried-down`) | `name` | Restore *Lost good name* (cause *exaggerated*: she warned her own village; she backed no lord) | she copied the readings; disgraced, she can't be believed about what she copied |
| b | player · *framed* · for destroying a text (`framed-damage`) | `inside` | Investigate *Sabotage* (blamed for burning the chapel's copy-sheet; done by the Hand) | the player was at the tithe book's burning; it is easy to say who struck the match |
| c | guild · *hall seized* · shut by her own order (`seized-order`) | `inside` | Acquire *Under seal* (the copy room's keys, under the hall's rule) | "until the border is settled", no chain-book or loft sheet leaves the room |

- **The door carries the fault line (Schism):** `{fault}` (met on C1-1) says he will stand against the master if a village pays for her readings.
- **Lesser code-break (kind: take a side in public):** an extra way on the door: tell `{border_village}`'s headman, privately, which reading is the true line.

**10 · Crisis** (at the survey hall's door in `{reading_room}`'s town). **Last move ⚑:** the reading moves the stone; `{lord_b}`'s men pull it down; `{lord_a}`'s men answer; `{border_village}` burns (who sets the fire is flavour b). The heart's mother is driven, with the village, to the survey hall's door, since the halls' readings burned it. Found out before the leader, the master says it plainly (flavour a). The enemy's want survives: the border is still open.
- **Held:** the player knows the masons work from a fresh reading (C1-1), and that it is the halls' (the reveal), before the stone moves; but no Writer can stop a lord's masons or his neighbour's men, and the halls' rule forbids telling either lord which reading is true. The door's code-bending way breaks that rule, and the crisis reads it: if the player told the headman, the village got its old people out before the fire, and the heart's mother comes to the door with her neighbours, not alone.
- **Family:** Reckoning *Schism*. Fault line `{fault}`. The leader, off the road at the survey hall, judges in person *(draft 3: a W1 enemy is not the leader, so she can come; no letter rides from the Wayhouse)*. Wronged: the heart (her village). Wrongdoer: the master.
- **Giver:** the heart's mother. *Reason:* the last move burned her house.
- **Choice:** the guild judges its own steering: keep it, or own it.
- **Ways:** *code-break:* take a side in public: the guild stands before `{lord_a}`'s court for the village's line · *ally:* `{lord_b}`'s surveyor swears the true line (his price: the free survey) · *justice:* bring it before the leader's letter, with `{fault}` speaking.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Open the ledgers** (every reading she sent, and her name, published) | the guild's name (both lords learn the halls steered them) · the master (struck off) | law | her mother lives in town at last / the village rebuilds on the old stone / the master keeps a tower and her readings |
| **Keep the balance** (the leader's letter keeps the master; the village is paid for) | the heart (the village rebuilds on the moved line; her mother's strip is lost) · `{fault}` (he leaves) | persuasion | copies again, and no longer for the lofts / a village on borrowed ground / the master writes her next reading |
| **Take a side at last** (code-break: the halls stand for the village before `{lord_a}`'s court) | the guild's neutrality: `{lord_a}` takes his rolls from the halls (S7 lost) | law | hailed in the village, watched in the halls / the old stone is reset / the master keeps her hall and loses the rolls |
| **Sides with the wrongdoer** (the player argues her case: without her, one lord would have had the border long ago) | the heart · the village's claim | persuasion | stops speaking to the player / the village scatters to the two lords' farms / the master is thanked by letter |
| **The guild splits** (the Schism goes on) | the survey hall (half its hands leave with `{fault}`) · the heart (she must choose a half) | none | copies for `{fault}`'s new table / two surveys of one border / the master keeps the half that stayed |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R3 (odd job) | *already copied* (`a-copy`): the survey hall's master holds the other copy | the priest asks whether "the halls will want it back" |
| C1-1 | the masons' line is set by a reading, not by the ground: the oldest stone never moved | `{fault}` points out that its lichen is unbroken round the base |
| C1-3 | *promised elsewhere* (`another-buyer`): the Hand has promised the book to the master's strongroom by week's end | its strap is already sealed with the hall's wax |
| door a | cause *exaggerated* | her warning was "the stone will walk"; she named no lord |
| door b | the Hand burned the copy-sheet | the ash is loft paper, which the chapel never buys |
| door c | twist *wants to be rid of it*: the copy room's door-keeper is tired of the hold | he leaves the key-ring on its hook in plain sight |
| crisis | *shared blame* (`shared-blame`): the reading that moved the stone was copied in her hand | the loft's night book shows her number against that week's sheet |

**13 · Hall-job hooks**
- *Middle:* at a hall near the border, a farmer wants his corner stone re-read before he sows; the survey hall is a copyist short. Reads M9; *before* face: "the survey hall is behind".
- *Late:* at a stall near the border, burned-out families want letters written to kin. Reads the last move; *before* face: "trouble on the border".
- *Seen in passing:* before the reveal, a survey hall chain-man helps a side card's requester measure a field.

**14 · What varies.** `{lord_a}`/`{lord_b}`: adjoining lordships from the registry with no standing border (field 17, as W1-A). `{border_village}`: Low, unseated, on the border, with a parish chapel (added where the border has none). `{reading_room}`: the survey hall, a reading room in a large town that is neither's seat, within reach of both. *Flavour* (each list rolled on its own): (a) what the master says when found out, voiced at the crisis: the border is safer open, neither lord deserves the villages, or the master before her did the same; (b) who sets the fire, told in the last move: `{lord_b}`'s men, or a fire spreading from the night's fighting; either way the village burns in the quarrel the reading made; (c) why `{lord_b}`'s surveyor wants the line fixed, told on C1-1: he was born on the border, his brother farms a strip there, or he set the oldest stone as an apprentice. *(Draft 1's river mid-line and wood's edge contradicted the stones, the masons and the lichen; its miller and steward's son couldn't swear the true line. Gone.)*

**15 · Presuppositions:** the lords (registry holders) and their open border (slots; reveal) · `{border_village}` is the heart's birthplace (R2; C1-2) · the court rolls the halls keep for `{lord_a}` (stake; reveal) · the loft and its night book (R2) · the chapel's tithe book and its priest (R3) · the dismissed copyist (move) · the masons and their stone (C1-1, C1-2) · `{hand}` = the survey hall's copyist (R3 hidden, C1-3 open) · `{lord_b}`'s surveyor (C1-1) · `{fault}` (C1-1; door) · the heart's mother (R2, C1-2) · the leader judging at the survey hall (crisis).

**16 · C1 cards:** C1-1 Investigate *Watch them*; ch2 opening Threat *Go and look*; R1's swap Acquire *Take the readings*. *(Draft 3: C1-1 was Take the readings, which put two Acquire cards in chapter 1.)*

**17 · Placing.** Centre: `{reading_room}`. Main town (for *Fresh stories*): the centre's town. Lords and courts kept in memory: `{lord_a}`, `{lord_b}`. Kept in memory too: stake S7, move M9, openings Investigate · Watch them, Threat · Go and look. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{reading_room}`, where the story is; the giver is there (the heart's mother, driven with the village to the survey hall's door); its action falls at `{reading_room}`.
*Swaps* (field 5): R1 may be Acquire · Take the readings.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{lord_a}` | seat | a High holder with a border: Banner Lord, Free Lord or Seated House head (d-l0) | seats ≤2.8 apart |
| `{lord_b}` | seat | as lord_a, of the same major faction (draft 4) | — |
| `{reading_room}` | hall | the survey hall: a reading room in a large town, neither seat, ≤2.8 from both, in the lords' own faction's town or a neutral market's | centre |
| `{border_village}` | village | Low or Middle, unseated, on the border, with a parish chapel (added if none) | ≤1.25 of the seats' midpoint |
| `{hand}` | person | the survey hall's copyist (made on demand) | — |
| `{ally}` | person | lord_b's surveyor (made on demand; his) | — |
| `{fault}` | person | the survey hall's eldest surveyor (made on demand; the hall's) | — |
| `{free_hall}` | hall | any reading room | ≤3.0 of the centre |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{free_hall}` | — | free (≤3.0 of the centre) |
| R2 | `{reading_room}` | `{reading_room}` | bound |
| R3 | `{border_village}` | `{border_village}` | bound |
| RV | `{lord_a}` | `{lord_a}` | bound |
| C1-1 | `{border_village}` | `{border_village}` | journey (≤3.5 of the centre): the border, a ride out |
| C1-2 | `{border_village}` | `{border_village}` | bound |
| C1-3 | `{reading_room}` | `{reading_room}` | bound |
| C2-0 | `{border_village}` | `{border_village}` | journey (≤3.5 of the centre): the border at night |
| DOOR | a: `{reading_room}` · b: `{border_village}` · c: `{reading_room}` | — | bound |
| crisis | `{reading_room}` | `{reading_room}` | bound |
| crisis, acted | `{reading_room}` | — | where its action falls |

*Parties set up first* (checked): the master (RV); the heart's mother (R2); the village (R3); fault (the eldest surveyor) (C1-1); the Hand (C1-3); lord_b's surveyor (C1-1); lord_a (RV).

---

## W2-A · The Last of the Four

**1 · Enemy:** W2a, a Free Lord's archivist (High), buying through a dig foreman and paying a hired blade. *Above* · ambition · `coin` `men` `paper` · region · *the hidden hand* · side: Free Lords.

**2 · Scheme.** He buys the early imperial survey rolls out of `{dig}` through its foreman, so `{lord}` can file a "restored" claim to three valleys in his own march. The later rolls record those valleys granted away to their present holders. Four diggers who wouldn't sell to the foreman sold them to the Wayhouse instead, and those diggers are dying in "accidents" his hired blade arranges.
**Payload ⚑ (crisis):** at `{court}` on `{hearing_day}`, with nobody left to swear where the later rolls came out of the ground, he calls the Wayhouse's rolls stolen from `{lord}`'s land. The claim stands, and the three valleys' holders are turned off.

**3 · Stake:** S5c, the last three rolls, newly lodged at the Wayhouse. *Reading:* the later rolls are worth only what someone will swear about where they were dug.

**4 · Heart:** H6a *(Claude, draft 2)*, `outside`, cast: a holder of a farm in one of `{valley}`'s three valleys, under a grant the halls keep a copy of, who can't read it. *Needs:* the heart's son, `{son}`, digs at `{dig}`, and was one of the four who sold the later rolls to the halls, on the heart's word ("sell to the halls, not the foreman; the halls keep our grant"). If the claim stands, the heart's own farm goes with the valleys.
- **Want:** to see the boy take the copyists' oath and leave the dig: a son who can read what the family holds.
- **Past:** the heart never learned letters, and sent the boy to dig for the halls so that he would.

*(Draft 1's heart was a grandmother clerk at a hall, which put a fourth old clerk among the sixteen hearts and, on map D, sat her days from the dig. A holder in the valleys puts the heart's own land and the heart's own child on opposite sides of the crisis.)*

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{r1_hall}`, a hall that keeps the year's copy of the Wayhouse's lodging book (every hall keeps the year's copy *(Claude)*) | Investigate *The ledger doesn't add up* (signature) | a lodging fee was altered by the hall's clerk (*for themselves*); the year copy the player checks against shows the dig's last entry *(draft 4: was at the Wayhouse, one of five stories through Highcrossing)* | "**three rolls from {dig}, sworn by four diggers before the eldest copyist**" |
| R2 | `{lantern_post}`, the dig's Lantern chapter | Dispute *Broken agreement* | a Lantern guard of the dig's contract camp quarrels with his chapter's contract board over a dead digger's last pay (the camp pays the diggers through the Lantern contract, so the chapter owes it; an ally-guild meeting); the dead man went by flavour b. The heart stands witness for the dead man's widow, a neighbour. **Heart's card**: the heart wants the boy off the dig and sworn a copyist, and says why (the past), and what the heart told the boy when the foreman came buying: "sell to the halls, not the foreman; the halls keep our grant" | "**two diggers dead in one season, both of the crew that sold to the halls**" |
| R3 | `{lodge_hall}`, the reading room where the dig's finds are catalogued | O4 Investigate *Disappearance* (the dig's reader, gone from her room above the reading room, the ink still wet). **Odd job** | find the dig's reader, who came to town to lodge her finds catalogue | ∅ "**the catalogue's empty shelf**" |

**6 · Odd job:** O4 (thread `wrong-place`; forced truth *the story's own*). *How it serves the scheme:* the Hand, the dig's foreman, took the reader and her catalogue the night she came to lodge it. The catalogue says which trench each roll came out of; with it gone, only the diggers can swear where the later rolls were found, and the diggers are the ones dying.

**7 · Reveal** (giver: *a found letter*; form: *the thing names them*). At `{pawn_town}`. Acquire *Passed on*: a pawnbroker bought a dead digger's tool roll in good faith, from his widow; tucked in its handle is the dead man's last letter, never sent, which the player reads. *(Draft 4: the leader gave seven reveals of eight; here nobody explains, the letter does.)*
- **Shows:** the letter names the foreman, buying the early rolls for "the lord's reader" (the archivist) so that `{lord}` can claim three valleys back by them; it says he and three others sold the later rolls to the halls instead, that two of them are dead this season, and that he is afraid of the man the foreman brought up from the lord's seat (the hired blade).
- **Adds:** nothing past the letter; the pawnbroker adds only that the widow pawned the roll the week he was buried.
- **Repeats:** "sworn by four diggers", "two diggers dead in one season", "the catalogue's empty shelf".
- **Closes:** the letter says the plants in its own words: the four of them "swore to the halls' rolls before the eldest copyist"; "two of us are dead this season, both of the crew that sold to the halls"; and the foreman has asked him which trench the reader's catalogue sets his rolls against, "the only book that says where they came out of the ground", the book the player found gone from its shelf.

**8 · Chapter 1.** *Shape:* 1 · the price at the turn. **Move:** M6 on S5: `{lord}`'s men close `{dig}` and turn its diggers off, the ruin "being his lord's by the old rolls". *Uses* `men`, lever force. *Needs:* `{dig}` stands in `{lord}`'s lordship.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{dig}`, closed (a journey) | Threat *Go and look* (C1) | — | slip into the closed dig and see what the lord's men are doing. Plant: "**the later rolls' trench, filled in and turfed over**" |
| C1-2 | `{hamlet}`, the diggers' hamlet below the dig | Dispute *Public offense* (the season's wage-roll) | therefore | the turned-off diggers cry the foreman's clerk a cheat before the hamlet over unpaid wages on the season's wage-roll; `{son}` talks of going over the hills. Plant: "**a boy who means to run**" |
| C1-3 turn | the Lantern contract camp at the dig | Acquire *Under seal* (seal type *ownership unproven*) | but | the foreman (the Hand, opposing), now keeping the closed dig's books under `{lord}`'s seal, holds the wage-roll, every digger's mark against every trench by day. **Ally plant:** `{sergeant}`, the camp's Lantern sergeant (flavour c says why he helps); his price: his chapter's dig contract paid in full |

**9 · Chapter 2.** Opening (pressure 2): `{hamlet}`, Dispute *Shared resource*: with the dig's wages gone, the turned-off diggers and the valleys' holders quarrel over the hamlet's grazing, and `{lord}`'s crier, come to publish `{hearing_day}`, takes the holders' side. Then the door.

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | guild · *hall seized* · by a lord's men (`seized-lord`) | `men` | Acquire *Under seal* (`{lodge_hall}`'s lodging copies, under `{lord}`'s seizure "until the old rolls' ownership is ruled") | the reading room's copies of the lodgings name the four who swore; a lord's men at a hall's door are the lord's business, and the archivist's hand stays hidden |
| b | player · *arrested* · in a lord's cells by his men (`arrested-men`) | `men` | Passage *Fugitive* (cargo: the player; pursuer: his men) | the player went into the closed dig |
| c | guild · *members taken* · bonded for debts he bought (`taken-bonded`) | `coin` | Rescue *Bound by debt* (creditor: the archivist) | bonded men swear what their creditor says; `{son}` hid and is not among them |

*(Draft 1's option a, the lord's men wounding the leader on the road, is barred: a hidden-hand enemy never wounds anyone openly.)*

- **The door carries the crisis's plant:** the heart, who has counted them, says that of the four who swore, two are dead, one is bonded or gone, and only `{son}`'s mark is left: "**the last of the four**". `{sergeant}` says why it matters *(Claude)*: a lord's court hearing a claim on ground takes no written oath about where a thing was dug; a living digger must swear it there in person.
- **Lesser code-break (kind: destroy a text):** an extra way on the door: rub `{son}`'s mark out of the wage-roll, so no one can trace him.

**10 · Crisis** (at the heart's farm in `{valley}`). **Last move:** the archivist's men ride the road to `{court}`; `{hearing_day}` is `{days}` days off. The Wayhouse has sent the later rolls down to the court's town with one of its clerks, to be shown at the hearing. `{son}` is hidden on the heart's farm, and won't go: he expects to be punished at his lord's court as a thief of his lord's ground (flavour a, voiced here). The enemy's want survives: the claim isn't heard yet.
- **Held:** the player knows of the buying and the murders from the reveal and holds the wage-roll's marks (C1-3), but a lord's court hears a claim on its own day and takes no written oath about where a thing was dug, so nothing the player holds answers the claim but a living digger at the hearing: the crisis is that chance.
- **Family:** Passage *Unwilling passenger*. Cargo: `{son}` (reason: expects punishment on arrival). Requester: the heart (family link; not outranked). Opposing, the pursuer: the archivist's men. Destination: `{court}`.
- **Giver:** the heart, at the farm where the boy hides. *Reason:* the boy, and the heart's own word that sent him to the halls ("sell to the halls, not the foreman", R2).
- **Choice:** take the boy to swear and risk him, and so keep the family's land, or keep him safe and let the claim take the farm.
- **Ways:** *code-break:* burn the later rolls the Wayhouse sent down for the hearing, so nothing is left to swear to and he needn't go · *ally:* `{sergeant}` rides with them (his price: the contract paid) · *bring along someone he loves:* the heart rides too.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Delivered** (he swears; the later rolls stand; the claim fails) | `{son}` (marked by the lord's men, he can never go home) · `{sergeant}` (his chapter loses the dig) | law | keeps the farm, and the boy copies at the Wayhouse: the want, at a price / the valleys keep their holders / the archivist turns to the next dig |
| **Burn your copy** (code-break) | the three valleys' holders, the heart among them (turned off) | stealth | keeps the boy, and loses the farm / the valleys go to `{lord}` / the archivist is promoted |
| **Released** (he goes over the hills) | the valleys' holders, the heart among them · the heart (the boy lost to the road) | none | a letter a season, through the Wall, read out by a clerk / the claim stands for now, unsworn / the archivist hunts the last digger |
| **Handed to the pursuer** (given up for the archivist's promise to spare him) | `{son}` · the valleys | bribe | is told the boy lives, and is not told where / the claim stands / the archivist keeps his promise as far as suits him |
| **A copy in every hall** (the rolls copied into every hall and to each valley holder; the boy rides up to swear before `{eldest}`, not the court) | the valleys at `{court}` (lost there) · the guild (`{lord}`'s enmity) | persuasion | the boy is safe in Highcrossing, and the farm is lost in the lord's court and kept in every other / the claim wins at `{court}` and no other court honours it / `{lord}` bars the halls from his march |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R3 (odd job) | *the story's own* (`the-enemy-did-it`): the foreman took the reader | her landlady says a man with dig clay on his boots paid the rent a month ahead |
| C1-1 | the trench is filled so no one can show where the later rolls lay | fresh turf on a trench the dig's map marks "worked out" |
| C1-3 | *promised elsewhere* (`another-buyer`): the foreman has promised the wage-roll to the archivist | he keeps it in his own satchel, not the dig chest |
| door a | twist *wants to be rid of it*: the lord's captain at the hall is tired of holding it | he lets readers in to "fetch their own books" |
| door b | *the pursuer was told* (`sold-from-inside`): the dig's cook told his men the road | the cook asks which road the player means to take |
| door c | *inflated* (`made-bigger`) | the debt slips are written in a hand that isn't the moneylender's |
| crisis | *not what it seems* (`more-than-said`): the boy kept one early roll from the foreman's cart, which proves the buying | he won't let anyone carry his pack |

**13 · Hall-job hooks**
- *Middle:* at a hall near the dig, a Lantern widow needs her husband's last contract pay claimed in writing. Reads M6; *before* face: "the dig's quiet".
- *Late:* at a hall, a holder from one of the valleys wants his old grant charter copied at a second hall "in case". Reads the claim; *before* face: "talk of a lord's claim on the valleys".
- *Seen in passing:* before the reveal, a Free Lord's steward buys grain fairly at a market on a side card.

**14 · What varies.** `{lord}`: a Free Lord from the registry whose march holds three valleys of freeholders. `{dig}`, `{valley}` and `{hamlet}`: inside his lordship (*Placing a spine* 11; the placer adds the dig where the map has none). `{r1_hall}`: any other hall within three days. `{hearing_day}` (rolled; `{days}` from the farm to his court). *Flavour* (each list rolled on its own): (a) the boy's fear, voiced on the crisis: hanging as a thief of his lord's ground, or being sold to the Binders (the slavers the Free Lords buy from); either way he expects punishment on arrival; (b) the accidents, told on R2: a trench collapse, a drowning in the flooded trench, or a fall from the ruin's stair; (c) why the sergeant helps, told on C1-3: he stood guard the nights they died, one of the dead was his nephew, or he was paid to be elsewhere one of those nights and hates it. *(Draft 1's "the dig's cook" as ally contradicted door b's tell, where the cook is the informer; its drowning "at the ford" assumed a ford.)*

**15 · Presuppositions:** `{lord}` (registry), his `{court}` and march, the three valleys (slots; reveal) · `{dig}` in his lordship (slot) · the four diggers who swore before `{eldest}` (R1) · the court's need of a living witness *(Claude)* (door; why the murders matter) · two dead (R2) · the heart's farm and grant (heart's spec; R2) · the reader and her catalogue (R3) · `{hand}` = the foreman (R3 hidden, C1-3 open) · the hired blade (reveal) · `{son}` (heart's spec; C1-2) · `{sergeant}` (C1-3) · the filled trench (C1-1) · the later rolls lodged at the Wayhouse (S5c; R1), and sent down to the court's town for the hearing (crisis) · the heart's word to the boy (R2) · `{hearing_day}` (ch2 opening) · an oath before `{eldest}` stands in every court that honours the halls (R1; *a copy in every hall*).

**16 · C1 cards:** C1-1 Threat *Go and look*.

**17 · Placing.** Centre: `{dig}`. Main town (for *Fresh stories*): `{lord}`. Lords and courts kept in memory: `{lord}`. Kept in memory too: stake S5, move M6, openings Threat · Go and look, Dispute · Shared resource. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{valley}`, where the story is; the giver is there (the heart, at the farm where the boy hides); its action falls at `{lord}`. *Clock:* `{days}` = the road from `{valley}` to `{lord}`, rounded up, + 1 (the boy must be got from the farm to the court); `{hearing_day}` is that many days off at the last move.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{lord}` | seat | a Free Lord (d-l0 holder, sub f-free-lords); his court is at his seat | — |
| `{dig}` | site | a Writers' dig at an imperial ruin in the lord's territory (added: the map has none there) | centre; 0.8 from the seat |
| `{valley}` | site | the three valleys, in the lord's territory; the heart's farm is in one, and the boy hides there | 1.0 from the seat |
| `{hamlet}` | site | the diggers' hamlet | ≤1.25 of the dig |
| `{lodge_hall}` | hall | the reading room nearest the dig, where its finds are catalogued | ≤1.25 of the dig, else ≤1.6 (nearest) |
| `{lantern_post}` | town | a Lantern chapter house or contract board (the dig's guards' chapter) | ≤1.25 of the dig, else ≤1.6 (nearest) |
| `{pawn_town}` | town | a pawn house (presence); else the placer adds a pawnbroker at lodge_hall | ≤1.25 of the dig, else ≤1.6 |
| `{r1_hall}` | hall | a reading room (not lodge_hall) keeping the year's copy of the Wayhouse's lodging book, as every hall does | ≤3.0 of the dig |
| `{court}` | seat | the lord's court, at his seat (= lord) | — |
| `{hearing_day}` | day | the court's sitting (roller: a day name; `{days}` days off at the last move) | — |
| `{hand}` | person | the dig's foreman (made on demand) | — |
| `{sergeant}` | person | the Lantern sergeant of the dig's camp (made on demand; the Lantern's) | — |
| `{son}` | person | the heart's son, a digger at the dig (cast with the heart's household) | at the dig |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{r1_hall}` | — | free (≤3.0 of the centre) |
| R2 | `{lantern_post}` | `{dig}` | bound |
| R3 | `{lodge_hall}` | `{dig}` | bound |
| RV | `{pawn_town}` | `{dig}` | bound |
| C1-1 | `{dig}` | `{dig}` | journey (≤3.5 of the centre): out to the dig |
| C1-2 | `{hamlet}` | `{dig}` | bound |
| C1-3 | `{dig}` | `{dig}` | bound |
| C2-0 | `{hamlet}` | `{dig}` | bound |
| DOOR | a: `{lodge_hall}` · b: `{lord}` · c: `{hamlet}` | `{dig}` | bound |
| crisis | `{valley}` | `{valley}` | bound |
| crisis, acted | `{lord}` | — | where its action falls |

*Parties set up first* (checked): the son (R2); the sergeant (C1-3); the lord (C1-3); the archivist (RV); the valleys' holders (R2); the eldest copyist (R1).

---

## W2-B · Sold by the Wayhouse

*(Draft 2: rewritten. Draft 1, "The Seal With a Plain Rim", was W2-A's twin: the same lord, the same three valleys, the same claim, with the witness running instead of hiding. This spine keeps the false seal, which was its good part, and drops the valleys: the payload is the Wayhouse made a seller of its own lodgings, and the core is the heart's guilt.)*

**1 · Enemy:** W2b, a Free Lord's archivist (High), buying under a false Writers' seal. *Above* · ambition · `coin` `men` `paper` · region · *the hidden hand* · side: Free Lords.

**2 · Scheme.** His lord wants an archive of the old empire's rolls of his own, so his court need never ask the halls for one. The archivist's buyer buys `{dig}`'s whole season under a Wayhouse seal no hall cut, through bills of sale written at `{stall}` for diggers who sign with a thumb. The bills' wording is his: "sold by the Wayhouse: the rolls of `{dig}`, this season's and all lodged before". The diggers think the halls have their finds; the halls hear nothing.
**Payload ⚑ (crisis):** `{lord}` sues before `{magistrate}` for what the bills say he bought. Unless the seal is proved false, the magistrate rules that the Wayhouse sold `{dig}`'s rolls and must deliver every one it holds to `{lord}`'s archive. The halls are named, in open court, as sellers of their own lodgings, and every dig on the island learns that a Writers' seal is a buyer's.

**3 · Stake:** S5b, read wide: the rolls `{dig}` has sent the Wayhouse, this season's and every season's before. *Reading:* the bills sell them all; whoever is believed about the seal decides whose they are.

**4 · Heart:** H1c, `member` (lapsed), cast: the reading master who recruited the player, who left the halls after a quarrel and keeps `{stall}`. *Needs:* wrote every one of this season's bills, and drew each buyer's seal into her day-book's margin, the old way.
- **Want:** to be taken back onto the Wayhouse's roll.
- **Past:** she left the halls the year they stopped copying for people who couldn't pay, and has written for them for nothing since, at a loss. *(Printed on R1.)*
- **Her guilt** (the crisis's hidden truth, printed once on the crisis card through its tell): she saw the seal was wrong, and wrote on for the fee, because the fee paid her stall's debt and kept it open for the poor.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{stall}`'s town | Dispute *Broken agreement* | a carter and a grain buyer over a bill she wrote; she is the witness. **Heart's card**: "the halls are buying at last, all season"; she would go back if they asked (her want); she writes for the poor for nothing (her past) | "**bills of sale for men who sign with a thumb**" |
| R2 | from `{stall}` past `{toll}` (a journey) | O3c Passage *Smuggling* (a sealed letter, carried as a bill of sale). **Odd job** | carry a sealed packet from her stall past `{lord}`'s toll on the dig's track, where it leaves his land for the high road *(draft 4: no map road reaches a Free Lord's bog; the toll is on his own track)* | "**a Wayhouse seal with a plain rim**" (the halls' own has a knotted rim; flavour a, seen here first, may set another flaw, and its words) |
| R3 | the Wayhouse (its lodging book) | Investigate *The ledger doesn't add up* (signature; *swap* below) | a late fee was hidden in the lodging book (*for themselves*): the clerk hid it for the magistrate's own household, whose lease was lodged a year late and back-dated: "**a late fee for the magistrate's own house, hidden**" | ∅ "**every season of {dig} lodged, but this one**" |

*Swap:* **R3** may be Acquire *Copy it before it burns* (signature): the magistrate's steward means to burn his household's receipt for its lease, the one paper left that shows the lease lodged a year late and back-dated (the Wayhouse's own page of it was cut from the lodging book), and the Wayhouse wants the receipt copied before it goes; both plants are kept (the receipt is "a late fee for the magistrate's own house, hidden"; the lodging book its page was cut from shows every season of `{dig}` lodged but this one).

**6 · Odd job:** O3c (thread `shouldnt-exist`; forced *not what it seems*). *How it serves the scheme:* the Hand, the archivist's buyer, is the packet's true recipient beyond the toll. It holds the diggers' copies of the season's bills under the false seal, and because a real Writer carries it, the toll's book records "Wayhouse post, one packet, sealed": the halls' own post has carried the sale.

**7 · Reveal** (giver: *the leader*; form: *the knower's question*). The leader, off the road at `{reveal_town}`. Dispute *Contested inheritance*: two sons of a digger who died lately, in his bed, quarrel over the price in his last bill, "paid by the Wayhouse"; the leader, amused, says the Wayhouse bought nothing. The dead man's confidant is the knower.
- **Shows:** the confidant saw the buyer's seal ring and its flaw (flavour a), and keeps the dead man's copy of the bill, whose wording is the buyer's: "sold by the Wayhouse: the rolls of `{dig}`, this season's and all lodged before".
- **Adds:** the leader adds whose buyer it is and the suit to come, from the Throne's clerks she shared a road-house with, who had been asked by `{lord}`'s archivist how a lord sues the Wayhouse in Highcrossing for rolls it sold him. *(Draft 4: the confidant can't know of a suit.)*
- **Repeats:** "bills of sale for men who sign with a thumb", "a Wayhouse seal with a plain rim", "every season lodged, but this one".
- **Closes:** the confidant asks the player three questions: does a Writer sell for men who sign with a thumb ("bills of sale for men who sign with a thumb"), does the halls' seal have a knotted rim ("a Wayhouse seal with a plain rim"), and why is this season missing from the Wayhouse's book ("every season lodged, but this one")?

**8 · Chapter 1.** *Shape:* 2 · the ally first. **Move:** M4 (spine reading on S5): the archivist's buyer buys the season at `{dig}`'s gate in public, at twice the halls' price, "with the Wayhouse's thanks". *Uses* `coin`, lever bribery.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{dig}`'s gate (a journey) | Investigate *Watch them* (C1) | — | watch where the bought rolls go, with `{ally}`, the headman's daughter, who keeps the crew's cart tally; the buyer at the gate wears flavour b's cover. Plant: "**carts turning off the track at the toll, away from the high road**". **Ally plant:** `{ally}`, who wants her father's bill honoured; her price: the halls buy next season at the honest price |
| C1-2 | the diggers' camp at `{dig}` | Dispute *Public offense* | therefore | told the halls bought nothing, the diggers' headman cries the halls liars before his crew. Plant: "**the headman's own bill, kept in his hat**" |
| C1-3 turn | `{toll}` | Acquire *Won't sell* (the last cart) | but | the toll keeper (a proxy in the archivist's pay, opposing) holds the diggers' last cart, the one `{ally}` drives, for dues, and won't let it go at any price the diggers have, so the ally's own season stands held at the toll; the toll's clerk writes down every cart that passes |

**9 · Chapter 2.** Opening (pressure 2): `{stall}`, Restore *Lost livelihood*: the heart's trade has gone, now that the diggers distrust any letter-writer. Then the door.

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | guild · *members taken* · bonded for debts he bought (`taken-bonded`) | `coin` | Rescue *Bound by debt* (creditor: the archivist) | the diggers who began to doubt the seal are bought into his lord's quarry by `{lord}`'s seat, where no one asks them who sealed what |
| b | player · *arrested* · at the toll by his men, and let go; the player's satchel held (`arrested-men`) | `men` | Acquire *Under seal* (the satchel, with the packet's wrapper and its seal's print, under `{lord}`'s seal "for dues") | the wrapper is the one thing that shows the seal's flaw outside the bills |

- **The door carries the crisis's plant:** its knower (a bonded digger, or the toll's clerk) has seen her day-book: "she drew every seal, the odd one too": "**the drawn seal in her margin**".
- **Lesser code-break (kind: destroy a text):** an extra way on the door: burn the headman's bill, so he can't be made to swear to it.

**10 · Crisis** (at her stall in `{stall}`'s town). **Last move:** the suit is cried in Highcrossing, to be heard on `{hearing_day}`, `{days}` days off: time for the summons to ride from Highcrossing to her stall, for her to run on to a haven and swear, and for its lofts to carry her statement up in a day (the clock counts all three, *Placing a spine* 9). On the archivist's complaint (flavour c), `{magistrate}` writes a summons for the bills' writer (the story's one writ), and `{lord}`'s men bring it to her stall, which is in his own land, where his men may serve it; held until the hearing, an accused forger's word counts for nothing. His men watch the ways out of his land. The night it reaches her, she comes to the player, because the summons names her and the halls she left are all she has. The enemy's want survives: the hearing is still to come.
- **Held:** the player has held the seal's flaw since the reveal, and the toll book's "Wayhouse post" line since R2, but a Writer swearing to the halls' own seal is the halls defending themselves, and the magistrate hears only the bills' writer; so the suit lands unless she swears, and the crisis is that chance (if blow b was dealt and the door won back the wrapper, the haven's holder sends it up with her statement).
- **Family:** Passage *Fugitive*. Cargo: the heart (reason: witnessed the seal; accused). Pursuer, opposing: the archivist's men, carrying the summons. Haven: `{haven}`, a seat outside `{lord}`'s land and `{magistrate}`'s writ, whose holder will take her sworn statement and send it up to the hearing by the relay lofts in a day.
- **Giver:** the heart. *Reason:* she must swear the seal false before she is held, and swearing it means saying she knew.
- **Choice:** run with the truth, and confess she took the fee, or stand and be held, and let the halls be named sellers.
- **Ways:** *code-break:* burn her day-book and let them search her · *ally:* `{ally}` hides her in the diggers' carts (her price: the honest price next season) · *route:* reach `{haven}`, if its holder will take her.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Delivered** (she swears at the haven, fee and all; her drawing goes up by the lofts and reaches the hearing; the bills fail) | the heart (she swears she took the fee; she is not taken back onto the roll) | stealth | copies for `{haven}`'s market, her want refused / the Wayhouse keeps `{dig}`'s rolls / the archivist disowns his buyer |
| **Burn your copy** (code-break: day-book burned; held, then freed for want of proof) | the halls' name (the bills stand) · `{dig}`'s diggers (their finds sold twice) | none | free, and the diggers won't come to her / the dig's rolls go to `{lord}`'s archive / the archivist begins on the next dig |
| **Take a side at last** (the halls swear the seal false on their own word, against a Free Lord, without her) | every hall in `{lord}`'s lordship (closed by his writ next season) | law | taken back onto the roll, and never asked about the fee / the suit is thrown out / `{lord}` shuts the halls in his march |
| **Handed to the pursuer** | the heart · the Wayhouse's rolls | bribe | held until the hearing, then let go / the rolls go to `{lord}` / the archivist owes the player a favour he'll call in |
| **Trade what we know** (the summons withdrawn, bought with the magistrate's back-dated lease) | the magistrate's household (the secret names it) | bribe | swears freely, and is ashamed of how / the suit fails / `{magistrate}` will rule against the halls the next time he can |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *not what it seems*: the packet holds the season's bills, for the Hand | a bill of sale sealed like a letter of state |
| C1-1 | the carts go to `{lord}`'s seat, not Highcrossing | the carters buy fodder for a longer road |
| C1-2 | *one side lies*: the headman knew the buyer wasn't from the halls | he won't show the bill, only say he has it |
| C1-3 | *as told*, twist *rightful keeper*: the dues really are unpaid, and the cart holds only rolls | the toll's dues board names the dig's cart, and the date |
| door a | *misassigned* (`wrong-name`): one bonded digger's debt is another man's | his slip carries a different father's name |
| door b | twist *wants to be rid of it*: the toll's clerk is tired of holding the satchel | he leaves it on the counter while he eats |
| crisis | *not what it seems* (`more-than-said`): she guessed the seal was false and wrote on for the fee | she paid off her stall's debt this season, in full |

**13 · Hall-job hooks**
- *Middle:* at a hall near `{dig}`, a digger's wife wants the bill her husband signed read aloud to her. Reads M4; *before* face: "the dig's sold its season".
- *Late:* at another hall, a digger from elsewhere asks whether a seal on a buyer's bill means the halls are buying. Reads the suit; *before* face: "a lord says the halls sold him rolls".
- *Seen in passing:* before the reveal, a Free Lord's toll men wave a caravan through courteously on a side card.

**14 · What varies.** `{lord}`: a Free Lord from the registry. `{dig}`: a dig in his land whose diggers can't write (added where the map has none). `{stall}`: in `{lord}`'s own land within a day of the dig (his seat, where no other place of his is near; the stall added where none). `{toll}`: on the dig's own track, inside his land. `{haven}`: a registry seat outside the lord's land and Highcrossing's writ, within reach of her stall (field 17). `{hearing_day}` (rolled; `{days}` = the summons' ride from Highcrossing to her stall, her road on to the haven, and the lofts' day). *Flavour* (each list rolled on its own): (a) the false seal's flaw, first seen on R2, which sets the plant's words wherever it is repeated: a plain rim (the halls' has a knotted rim), a reversed letter in the legend, or red wax (the halls seal in black); (b) the buyer's cover at the dig's gate, seen on C1-1: a Writers' grey gown, a hall's satchel, or the halls' own price board, copied; (c) the archivist's complaint in the summons, read on the crisis: that she cut the seal herself, or that she sold the halls' name for a fee.

**15 · Presuppositions:** `{lord}` (registry) and his wish for an archive (scheme; reveal) · `{dig}` and its diggers who sign with a thumb (R1) · `{stall}` and the heart's bills (R1) · the false seal and its flaw (R2; flavour a) · the halls' own seal *(Claude)* (R2) · the lodging book and the dig's earlier seasons in it (R3) · the magistrate's back-dated lease (R3; *trade what we know*) · `{hand}` = the archivist's buyer (R2 hidden; open at the dig's gate in the move) · `{ally}` (C1-1) and the headman (C1-2) · the toll keeper, the toll's clerk and the toll's book (C1-3; odd job) · the relay lofts *(faction-places)* (crisis) · the day-book's drawn seals (heart's spec; door) · `{magistrate}` · `{haven}` (registry) · the suit (crisis).

**16 · C1 cards:** C1-1 Investigate *Watch them*.

**17 · Placing.** Centre: `{dig}`. Main town (for *Fresh stories*): `{lord}`. Lords and courts kept in memory: `{lord}`. Kept in memory too: stake S5, move M4, openings Investigate · Watch them, Restore · Lost livelihood. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{stall}`, where the story is; the giver is there (the heart, at her stall, the summons in her hand); its action falls at `{haven}`. *Clock:* `{days}` = the road from `{wayhouse}` to `{stall}`, plus the road from `{stall}` to `{haven}`, plus 1 day (the haven's lofts carry her sworn statement up to the hearing), rounded up, + 1 (the summons rides from Highcrossing to her stall; she must then reach the haven and swear, and its lofts carry her statement up in a day); `{hearing_day}` is that many days off at the last move.
*Swaps* (field 5): R3 may be Acquire · Copy it before it burns.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{lord}` | seat | a Free Lord (d-l0 holder, sub f-free-lords) | — |
| `{dig}` | site | a Writers' dig whose diggers can't write, in the lord's territory (added: none on the map there) | centre; 0.8 from the seat |
| `{stall}` | town | the heart's letter-writer's stall, in the lord's own land (his seat or a place nearer it than any other lord's seat), where his men can serve a summons (presence; else added, with reason) | ≤1.25 of the dig |
| `{toll}` | site | the lord's toll where the dig's track leaves his land, toward the high road (added; in his lordship) | ≤0.7 from the dig |
| `{reveal_town}` | town | any settlement (the dead digger's sons) | ≤1.25 of the dig |
| `{haven}` | seat | a d-l0 seat outside the lord's land and the magistrate's writ, held by an Open Council head, a Banner Lord or a Seated House head | ≤3.0 of the stall |
| `{hearing_day}` | day | the suit's hearing in Highcrossing (roller: a day name; `{days}` days off at the last move) | — |
| `{magistrate}` | person | Highcrossing's magistrate (the Throne's officer, made on demand) | Highcrossing |
| `{hand}` | person | the archivist's buyer (made on demand) | — |
| `{ally}` | person | the diggers' headman's daughter (made on demand) | at the dig |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{stall}` | `{stall}` | bound |
| R2 | `{toll}` | `{toll}` | journey (≤3.5 of the centre): carried from her stall past the lord's toll on the dig's track |
| R3 | `{wayhouse}` | — | the Wayhouse: the dig's lodgings are kept at the Wayhouse |
| RV | `{reveal_town}` | `{dig}` | bound |
| C1-1 | `{dig}` | `{dig}` | journey (≤3.5 of the centre): out to the dig |
| C1-2 | `{dig}` | `{dig}` | bound |
| C1-3 | `{toll}` | `{toll}` | bound |
| C2-0 | `{stall}` | `{stall}` | bound |
| DOOR | a: `{lord}` · b: `{toll}` | `{dig}` | bound |
| crisis | `{stall}` | `{stall}` | bound |
| crisis, acted | `{haven}` | — | where its action falls |

*Parties set up first* (checked): the headman (C1-2); the headman's daughter (C1-1); the toll keeper (C1-3); the toll's clerk (C1-3); the magistrate's household (R3); the lord (R2); the diggers (C1-2); the haven's holder (CR); the archivist (RV).

---
## W3-A · Every Family Its Own File

**1 · Enemy:** W3a, a Night Market forger (Middle) with a trade link to a copyist, selling each family its own file to buy her way out of the Market. *Beside* · greed · `craft` `name` `coin` · local · *the hidden hand* · side: Night Market (barred when the syndicates are paired).

**2 · Scheme.** The copyist (her proxy) makes her fair copies of the private files `{reading_room}` keeps on the town's families: every letter each family sent through the Wall, every debt, every note. She sells each family its own file as "a forgery going about the market", promising hers is the only copy and burning it for a price. She is buying her bond out of the Market.
**Payload ⚑ (crisis):** in the market's last week she sells the town's carters and stallholders their files all at once, at a hired booth ("buy yours before it's read out"), pays her bond, and is gone by night.

**3 · Stake:** S3c, a market town stops trusting the copyists' oath. *Reading:* once the town learns the "forgeries" are the halls' own words, no family will let a Writers' copyist near its letters.

**4 · Heart:** H3c, `outside`, a man (he writes to his wife): a carter who can't write, writing to a wife who can't read either, so the Wall's clerk reads his letters out to her. *Needs:* lives in `{reading_room}`'s town; every letter he sent went through the Wall and into his family's file.
- **Want:** to buy his own cart and stop hauling for a master.
- **Past:** once, years ago, he asked the Wall's clerk not to read his wife one letter; what it says is flavour c, and it touches the cart money every time. It is still in his file. *(Printed on R1, beside the plant.)*

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{reading_room}` | Dispute *Broken agreement* (his cart-hire contract) | the carter against his master over the price of buying the cart he drives; the hall reads the contract out to both, at the Wall's reading-out bench, where the clerk's book lies open at his name. **Heart's card**: his want; and the book shows he once asked the clerk to keep one letter back from his wife (his past) | ∅ "**one letter in the reading-out book marked 'kept back'**" |
| R2 | from `{reading_room}`'s Wall to `{barn}`, a day out | O3a Passage *Smuggling* (a sealed letter, carried as a love letter). **Odd job** | carry a "love letter" waiting at `{reading_room}`'s Wall to a sweetheart a day out of town, past the gate, where the watch has begun opening sealed post | "**a love letter sealed with the copy room's own wax**" |
| R3 | `{widow_town}`, within a day (its letters go through `{reading_room}`'s Wall) | Acquire *Copy it before it burns* (signature) | a widow means to burn her husband's letters | "**four silver paid to a market woman for 'the only copy'**" |

**6 · Odd job:** O3a (thread `shouldnt-exist`; forced *not what it seems*). *How it serves the scheme:* the Hand, her runner, is the "love letter"'s true recipient. It is a bundle of file copies the copyist sends her through the Wall's own post, sealed with the copy room's wax so the sorters pass it unopened, and the player carries her stock to her.

**7 · Reveal** (giver: *the ally*; form: *a chain*). At `{reading_room}`, where its night porter works. Investigate *Theft*: `{wall_clerk}`, the Wall clerk who reads letters out to those who can't (the spine's ally, met here first), asks the player who has been taking from the copy room at night; the night porter knows.
- **Shows:** the porter has let a copyist of the hall into the copy room after the bell, and seen him carry file bundles out to a woman who waits at the lane end and pays in Market notes (the forger).
- **Adds:** `{wall_clerk}` adds what the bundles are, from her own reading-out book: the halls keep a file on every family whose letters go through the Wall (the hidden side, as W3's revelation allows), and the families she reads to have begun asking her whether a market woman's "only copy" is theirs; the porter adds what the woman told him she was buying (flavour d).
- **Repeats:** "one letter marked 'kept back'", "a love letter sealed with the copy room's own wax", "four silver for 'the only copy'".
- **Closes:** each thing explains the next: the files are why a family would pay "four silver for 'the only copy'"; a family's file is how a market woman could know of "one letter marked 'kept back'"; and the copy room's own seal is how the bundles left by the Wall unopened, "a love letter sealed with the copy room's own wax".

**8 · Chapter 1.** *Shape:* 3 · the early turn. **Move:** M5 on S3: she has it cried in the town's market square that forged Writers' files on the town's families are going about, and that she can get a family its own out of circulation. *Uses* `name`, lever rumour.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | the night market, in a barn a day out (a journey) | Threat *Go and look* (C1) | — | go into the night market and see what she sells and who brings it. Plant: "**files bound with the copy room's cord**" |
| C1-2 turn | the town's carters' yard | Dispute *Public offense* | but | the move bites: carters who bought their files, cried up by the forger's crier (a proxy, opposing), cry the reading room thieves; the halls' copyist who came to deny it is shouted down. Plant: "**a collection for 'clean copies'**" |
| C1-3 | the road out of town | Passage *Fugitive* | therefore | shouted down, the copyist who lets her in (flavour a: she has no tie to him she'd spare) fears the Market and runs; the Hand (opposing) hunts him on the forger's orders, since a copyist caught by the halls would lead them to her. **Ally asks:** `{wall_clerk}`, who knows every family who bought, asks the player to bring the copyist back to the hall alive, so the families hear from his own mouth who sold them; her price, later: the reading-out book kept from the hall master |

**9 · Chapter 2.** Opening (pressure 2): `{reading_room}`, Acquire *Unknown worth*: a carter's widow holds her file, bought at the booth, not knowing it is the halls' original. Then the door.

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | heart · *estranged* · his kin turned (`kin-turned`) | `name` | Restore *Estrangement* (cause `secret-told`; deadline: his wife leaves for her sister's at the season's end) | he was gathering the carters against her; she has his kept-back letter read to his wife in the market, and every family sees what she holds |
| b | player · *framed* · for destroying a text (`framed-damage`) | `name` | Investigate *Sabotage* (blamed for burning a family's file; done by the Hand) | the player was chasing her copies; she cries that a Writer burns what families paid for |
| c | guild · *members taken* · bonded for debts she bought (`taken-bonded`) | `coin` | Rescue *Bound by debt* (creditor: the forger) | with the families' coin she buys two copyists' debts, and they copy for her until paid |

- **The door carries the crisis's plant:** `{wall_clerk}` (met on C1-3), who hears every family's business at the reading-out bench, says she has hired the big booth for the market's last day, and a cart for the night after: "**a booth for the last day, a cart for the night after**".
- **Lesser code-break (kind: destroy a text):** an extra way on the door: burn one family's bought file, at their begging.

**10 · Crisis** (at `{reading_room}`, the morning after the booth). **Last move ⚑:** on the market's last day she sells the town its files at the booth. The heart pays his cart savings for his and his wife's. She pays her bond and is gone by night with the rest of her stock, to flavour b's refuge. The enemy's want survives: she has coin, and stock to sell elsewhere.
- **Held:** the player knows her trade from the reveal and her booth's day from the door, but the families buy willingly, no law forbids selling a family its own letters, and the one warning that would stop them (that the "forgeries" are the halls' own files) is the halls' shame said aloud; so the sale lands, and the crisis asks who pays for it.
- **Family:** Reckoning *Swindle*. Wronged: the heart (his savings), with every family who bought. Wrongdoer: the forger. Shield: *Fled* (a new name in another town).
- **Giver:** the heart, at `{reading_room}`'s Wall, where every letter he sent went through. *Reason:* he paid for what the hall let out.
- **Choice:** go after her for the families' coin, for her stock, or let her keep the freedom she bought.
- **Ways:** *code-break:* destroy her stock of copies · *ally:* `{wall_clerk}` names every family who bought, and they act together (her price: the book kept from the hall master) · *shield's way:* follow her to her new life and decide there.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Amends made** (she repays before witnesses in her new town) | the forger (poor again, and known) · the copyist (named, struck off) · the heart (a season's hauling lost to the chase; his letter is not read out again) | persuasion | has his savings back, and still no cart / the families hold their coin and distrust the halls / she is a copyist for hire under her own name |
| **Open the ledgers** (the halls admit the files and open each family's to it) | the guild's name in the town (S3c lost) · the copyist | law | his wife has heard every letter, and stays / no family will write through the Wall / the forger sells in a town that hasn't heard |
| **Burn your copy** (code-break: her stock and the halls' family files both burned) | the halls' records (generations of letters) · the families' coin (she keeps it) | stealth | his kept-back letter is ash; his wife never hears it / no file left on anyone / she is free and rich |
| **Wrongdoer ruined** (the Market told how she paid her bond; it takes her back) | the forger | stealth | savings gone, wife won back / the town's trust half-mended / she is bonded again |
| **Sides with the wrongdoer** (she was buying her way out of bondage; the player lets her go) | the heart (savings gone) · the town's families | none | hauls for his master another year / the town hates the halls / she writes to the player, once, from far off |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *not what it seems*: the "love letter" is file copies, for the Hand | a love letter as thick as a ledger |
| R3 | *it names the holder* (`damning-record`): his letters record years of short weight | she asks that only the last year's letters burn |
| C1-1 | the runner brings her stock in copy-room bundles | the barn's lamp is copy-room oil, sweet-smelling |
| C1-2 | *both partly right*: the halls copied them; someone stole the copies | the loudest crier quotes his own letter word for word |
| C1-3 | *the pursuer was told* (`sold-from-inside`): the Wall's night porter told the runner he'd run | the porter asks the player when they'll be back |
| door a | cause `secret-told` | his wife heard it from a woman she had never met |
| door b | the Hand burned it | the ash carries copy-room wax |
| door c | *paid* (`paid-already`): one copyist's debt was paid last spring | he still keeps the Market's chit, torn in half |
| crisis | *still profiting* (`still-profiting`): she took the unsold files to sell to the houses they shame | the cart she hired was for a heavier load than a woman and a chest |

**13 · Hall-job hooks**
- *Middle:* at `{reading_room}`'s town, a stallholder wants a contract copied by a copyist who will swear to keep no copy. Reads M5; *before* face: "talk of forgeries in the market".
- *Late:* at another hall, a family asks whether the halls "keep anything of ours", and wants a letter written with no copy made. Reads the booth; *before* face: "a booth for the market's end".
- *Seen in passing:* before the reveal, a Night Market fence pays a widow a fair price for her pawn on a side card.

**14 · What varies.** The town: any large town whose reading room keeps families' files. *Flavour* (each list rolled on its own): (a) the copyist who lets her in, told on C1-3: a copyist in her debt, a copyist she has an old false line on, or a copyist she pays in Market notes (the Night Market's own paper money) (none she would spare from her runner); (b) where she flees, told on the crisis: a port of the Open Hand, the free coast towns; a far hall town; or the moving market, the Night Market's travelling fair; (c) what the kept-back letter says, always touching the cart money (R1; read aloud if door a is dealt): he will pay his dead brother's debt from it, he sends money from it to a child elsewhere, or he hides a sickness it would pay for; (d) what her bond is for, told on the reveal by the porter: a debt of her own to the Night Market, the syndicate that bonds its debtors, or her daughter's release from a Night Market bond (the syndicate's own servitude). *(Draft 1 let the copyist be her brother and her bond be her brother's freedom, which read as two brothers or one, and set the heart's past to a brother's debt whatever (c) said. Each now stands alone.)*

**15 · Presuppositions:** the town and `{reading_room}` (slots) · the halls' family files (kit; reveal) · the Wall's reading-out for those who can't read (heart's spec; R1) · the kept-back letter (his past, flavour c; R1) · the copyist (proxy; reveal, C1-3) · `{hand}` = her runner (R2 hidden; C1-3 open) · her bond (reveal) · the night market (C1-1) · the forger's crier (C1-2) · `{wall_clerk}` (reveal; C1-3 asks) · the booth and cart (door) · the widow (R3) · the barn a day out (R2, C1-1).

**16 · C1 cards:** C1-1 Threat *Go and look*.

**17 · Placing.** Centre: `{reading_room}`. Main town (for *Fresh stories*): the centre's town. Kept in memory too: stake S3, move M5, openings Threat · Go and look, Acquire · Unknown worth. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{reading_room}`, where the story is; the giver is there (the heart, at the Wall his letters went through); its action falls at `{reading_room}`.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{reading_room}` | hall | a reading room in a large town that keeps families' files | centre |
| `{barn}` | site | the night market's barn (added) | 0.9 from the centre |
| `{widow_town}` | town | any settlement whose letters go through reading_room's Wall | ≤1.25 of the centre (the centre itself if none) |
| `{hand}` | person | her runner (made on demand) | — |
| `{wall_clerk}` | person | reading_room's Wall clerk (made on demand; the hall's) | — |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{reading_room}` | `{reading_room}` | bound |
| R2 | `{barn}` | `{reading_room}` | bound |
| R3 | `{widow_town}` | `{reading_room}` | bound |
| RV | `{reading_room}` | `{reading_room}` | bound |
| C1-1 | `{barn}` | `{reading_room}` | journey (≤3.5 of the centre): the barn a day out |
| C1-2 | `{reading_room}` | `{reading_room}` | bound |
| C1-3 | `{reading_room}` | `{reading_room}` | bound |
| C2-0 | `{reading_room}` | `{reading_room}` | bound |
| DOOR | a: `{reading_room}` · b: `{reading_room}` · c: `{reading_room}` | `{reading_room}` | bound |
| crisis | `{reading_room}` | `{reading_room}` | bound |
| crisis, acted | `{reading_room}` | — | where its action falls |

*Parties set up first* (checked): the heart's wife (R1); the Wall clerk (RV); the copyist (RV); the families (C1-2); the forger (RV).

---

## W3-B · The Copy House Across the Street

**1 · Enemy:** W3c, a Night Market forger (Middle) selling files to the houses they shame, through the copyist's sister. *Beside* · greed · `craft` `name` `coin` · local · *the hidden hand* · side: Night Market (barred when the syndicates are paired).

**2 · Scheme.** Through `{copyist}` and his kinswoman (flavour d), a clerk in `{lord}`'s counting room and the Hand, she has the halls' copies of `{lord}`'s court rolls, with the hall's own margin notes (who owes, who is weak), and sells each entry to the house it shames. To keep that trade without a thief inside, she opens a copy house beside `{reading_room}` and bids for the court's own commission.
**Payload ⚑ (crisis):** `{lord}`'s court seals its copying commission to her house for the year. The Night Market holds the lordship's rolls, every debt, holding and lawsuit, and the halls' keeping of them ends.

**3 · Stake:** S7 (a/b: a market or toll lord whose court sits in `{reading_room}`'s town). *Reading:* the commission is the right; the margins are what makes it worth stealing.

**4 · Heart:** H1b, `member`, cast: the half-blind reading master who has you read aloud to her. *Needs:* she has kept `{lord}`'s court rolls at `{reading_room}` for thirty years, and knows them by ear.
- **Want:** to finish her concordance of the court rolls, every name to its page, before her eyes go.
- **Past:** she lost most of her sight copying the rolls by lamplight in the winter of the great lawsuit.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{reading_room}` | Dispute *Contested inheritance* | two heirs, one farm; the roll entry decides it. **Heart's card**: she is the confidant who knows the entry by ear, and asks the player to read to her for the concordance; she lost most of her sight copying these rolls by lamplight in the winter of the great lawsuit (her past) | "**a lawyer's clerk quoting rolls the halls never copied for him**" |
| R2 | `{lord}`'s counting room, where the court pays the halls' copying bill | O2 Investigate *The ledger doesn't add up* (signature; spine variant: the court's copying account; the Hand is the court's counting-room clerk). **Odd job** | the halls' bill to the court, as it stands in the counting room's book, has been altered upward *(draft 4: was settled at the Wayhouse, one of five stories through Highcrossing)* | "**a line for copies the halls never made**" |
| R3 | `{bindery_hall}`, another hall within three days | Restore *Lost livelihood* (cause `trade-moved`) | an old bindery's trade has failed; its bookbinder bound for `{reading_room}` for thirty years | ∅ "**the bindery's empty press-room**" (presses and cords sold to "a buyer from {reading_room}'s town") |

**6 · Odd job:** O2 (thread `lie-in-record`; the culprit is the Hand). *How it serves the scheme:* the Hand, the copyist's kinswoman (flavour d, named here), now a clerk in the court's counting room, altered the halls' copying account upward. The court thinks the halls overcharge, and so opens the commission to bids.

**7 · Reveal** (giver: *the heart*; form: *the knower's question*). At `{pawn_town}`, where the heart has come with the player to read. Acquire *Unknown worth*: a pawnbroker (a pawn house, or one the placer adds) holds a box of "spoiled sheets" that a woman of the Market pawned from her back room for a first month's rent, not knowing they are court-roll entries with the halls' margin notes. The player reads them aloud to the heart. *(Draft 4: the box was "from the copy house's first week", before the copy house opens on chapter 1.)*
- **Shows:** the heart knows the entries by ear and the notes as her own hall's (who owes, who is weak): the halls' copies of `{lord}`'s rolls, margins and all, are being sold out of the Market (the forger's trade; the halls' margins, the hidden side).
- **Adds:** the heart adds whose copies they are, from her copying book, which says who read which entries back to her this season: `{copyist}`'s. Who carries them out of the hall is not hers to know; his kinswoman is seen on C1-2.
- **Repeats:** "a lawyer's clerk quoting rolls", "a line for copies the halls never made", "the bindery's empty press-room".
- **Closes:** the heart asks the player three things the routine answered: who quoted rolls the halls never copied for him ("a lawyer's clerk quoting rolls"), who billed the court for "a line for copies the halls never made", and who bought an old bindery's presses ("the bindery's empty press-room"); one house, she says, and it hasn't opened yet.

**8 · Chapter 1.** *Shape:* 1 · the price at the turn. **Move:** M1 on S7: she opens a copy house across the street from `{reading_room}`, bidding half the halls' price for the court rolls. *Uses* `coin`, lever bribery (the court's under-clerks are sweetened).

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{record_house}`, the court's hall of record outside the town walls (a journey) | Dispute *Broken agreement* (the old commission: the halls keep the rolls "while the halls are true") | — | the court's steward against the halls, over the altered account. Plant: "**while the halls are true**" |
| C1-2 | the copy house's lane | Investigate *Watch them* (C1) | therefore | watch the copy house at night. Plant: "**a second door onto the lane**" |
| C1-3 turn | the copy house | Rescue *Bound by debt* | but | she staffs the house with young copyists bound for Market debts; her door-keeper (a proxy, opposing) keeps them. **Ally plant:** `{binder}`, the old bookbinder whose presses she bought (flavour b says why he sold), who bound the heart's rolls for thirty years; his price: the halls buy his presses back |

**9 · Chapter 2.** Opening (pressure 2): the court's archive room at `{record_house}`, Acquire *Copy it before it burns*: the steward means to burn the halls' old copies once the new house delivers, because one of them carries the hall's margin note on his own debt, owed to `{house}`: "**a margin note on the steward's own debt**". Then the door.

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | heart · *disgraced* · cried down for taking a side in public (`cried-down`) | `name` | Dispute *Public offense* (offended: the court, before which she spoke) | she told the steward, in open court, that the new house's copies are stolen; the forger cries that the halls' keeper took sides against a lawful bidder |
| b | player · *framed* · for destroying a text (`framed-damage`) | `name` | Investigate *Sabotage* (blamed for the halls' old copies burned early in the archive room; done by the steward's man) | the player copied in the archive room on chapter 2's opening; the forger cries that a Writer burned the court's own copies to hide the halls' margins *(draft 3: was "player stripped", the blow W7-A also deals)* |

- **The door carries the crisis's plant:** `{binder}` (met on C1-3), who knows whose money bought his presses, says the copy house's rent is owed to the Market, and she has never been out of debt in her life: "**the copy house's rent, owed to the Market**".
- **Lesser code-break (kind: destroy a text):** an extra way on the door: burn the stolen bundle of margins found at the lane door.

**10 · Crisis** (at `{reading_room}`, across the street from her house). **Last move ⚑:** the court seals the commission to her house for the year and sends the heart a letter ending the halls' keeping. She won't sell it back; if she means to go straight (flavour a), it is after this one last sale. The enemy's want survives: the rolls are the richest stock she has ever held.
- **Held:** the player knows the copy house's stock is stolen (the reveal, C1-2), and the heart may say so in open court (door a), but the court seals to the lower bid under the old commission's own words ("while the halls are true"), and the altered account (R2) says the halls are not; so the seal lands, and the crisis is the chance to undo it.
- **Family:** Acquire *Won't sell*. Holder: the forger (reason: security; the commission keeps her out of the Market's bond). Requester: the heart (reason: pride). Target: the commission's seal.
- **Giver:** the heart, at the reading room, with the court's letter in hand. *Reason:* her keeping has ended.
- **Choice:** win the rolls back, and at what price to the forger and to the halls' secrets.
- **Ways:** *code-break:* burn the commission deed in her strongroom · *ally:* `{binder}` swears his presses were bought under a false name, and a commission sealed to a house bought in a false name is void (his price: the presses back) · *give her what it does for her another way:* the halls take her on as a sworn copyist, her bond paid, and the young copyists bound to her with her.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Obtained and handed over** (she sells it back for a place on the halls' roll and her bond paid, and the young copyists she bound are sworn with her) | the halls' oath (a forger is sworn in) · `{house}` (loses the rolls it was promised, and will come for her) | bribe | finishes her concordance / the rolls come back to the reading room, copied by the young copyists the Market had bound / she copies beside the heart, and is watched |
| **Left with the holder** | the heart (her keeping ends; she goes blind at another hall's table) · every family in the rolls · the young copyists (still bound, copying for her) | none | reads aloud no more / the court's rolls live across the street / the Market's richest book |
| **Burn your copy** (code-break: the deed burned; the court must seal again, to whoever brings the oldest copies) | the forger (her house fails; back into the Market's bond) · the halls (it's known they burned a deed) | stealth | keeps the rolls, ashamed of how / the reading room has its commission and a reputation / she owes the Market twice over |
| **Trade what we know** (the steward bought with the margin note on his own debt; he voids her seal) | `{house}`, the steward's creditor (the note names it) · the halls (now the steward knows what the margins hold) | bribe | keeps the rolls and stops writing margins / the court distrusts both houses / she sells her stock to the houses, and goes |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *for the one who profits* (`someone-above`): altered for the forger, by the Hand | the altered line is in fresher ink than the page |
| R3 | cause `trade-moved` | the owner was paid in Market notes |
| C1-1 | *record survives* (`record-survives`): the old agreement, and the halls' true account | the steward has never asked to see the halls' own account |
| C1-2 | the sister carries bundles from the reading room | ink-stained cuffs on a woman who doesn't copy |
| C1-3 | *real and fair* (`held-rightly`): the young copyist really owes the Market | he talks about the dice game unasked |
| door a | *both partly right* | she spoke only after the forger's bid was read |
| door b | the steward's man burned them, to be rid of the margin on his master's debt | the ash smells of the steward's lamp oil, not the archive's |
| crisis | *promised elsewhere* (`another-buyer`): the first year's copies are pledged to `{house}` | `{house}`'s carriage waits at the lane door on court days |

**13 · Hall-job hooks**
- *Middle:* at `{reading_room}`'s town, a widow asks the halls, not the new house, to copy her husband's will, "because the halls swear". Reads M1; *before* face: "a new copy house in town".
- *Late:* at another hall, a clerk of `{lord}`'s court quietly wants an old entry copied from the halls' own books, because the new copies "read oddly". Reads the commission; *before* face: "the court is changing copyists".
- *Seen in passing:* before the reveal, a Night Market card-game host returns a lost purse on a side card.

**14 · What varies.** `{lord}`: any lord whose court sits in a large town and whose rolls the halls keep. `{house}`: the house the rolls shame most, the steward's creditor (C2-0); a d-l0 house within three days, else made on demand (the roller fills it). `{copyist}` and his kin: the Hand is his sister, daughter or wife. *Flavour* (each list rolled on its own): (a) the forger's own reason for the house, voiced on the crisis: to keep her trade without a thief, or to go straight at last, after one last sale (the first year's copies pledged to `{house}`, the crisis's tell, are that sale, so the hypocrisy is on the card; the ways read the same); (b) why the binder sold his presses, told on C1-3: a debt, grief after his wife died, or a threat from the Night Market, the forger's own syndicate; (c) who reads to the half-blind heart, on R1: the player, or a child of the hall; (d) the Hand's link to `{copyist}`, first named on R2's hidden side: his sister, his daughter or his wife (the word is used wherever the Hand is named). *(Draft 1's court under-clerk and copyist's father as ally couldn't swear to the presses, which is the ally's way.)*

**15 · Presuppositions:** `{lord}` (registry holder) and his court in the town (slot) · the halls keep his rolls, with margins (stake; reveal) · `{copyist}` (proxy; reveal) · `{hand}` = his kinswoman, a court counting-room clerk (R2 hidden; seen on C1-2, carrying bundles) · her door-keeper (C1-3) · the bindery and its presses (R3) · `{binder}` (C1-3) · the old commission's wording (C1-1) · the bound copyists (C1-3) · the copy house's rent (door) · the margin note on the steward's own debt, owed to `{house}` (ch2 opening; *trade what we know*) · `{house}` (C2-0; crisis truth) · a commission sealed to a house bought in a false name is void *(Claude)* (crisis ally way).

**16 · C1 cards:** C1-2 Investigate *Watch them*.

**17 · Placing.** Centre: `{reading_room}`. Main town (for *Fresh stories*): the centre's town. Lords and courts kept in memory: `{lord}`. Kept in memory too: stake S7, move M1, openings Dispute · Broken agreement, Acquire · Copy it before it burns. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{reading_room}`, where the story is; the giver is there (the heart, at the reading room, the court's letter in hand); its action falls at `{reading_room}`.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{lord}` | seat | a court holder whose seat is a large town with a reading room: Banner/Free Lord, Seated or Tide House head, the Binders' head, Open Council head (d-l0) | = reading_room's town |
| `{reading_room}` | hall | that town's reading room | centre |
| `{record_house}` | site | the court's record house outside the walls (added) | 0.5 from the centre |
| `{pawn_town}` | town | a pawn house (presence); else a pawnbroker added at the centre | ≤1.25 of the centre |
| `{bindery_hall}` | hall | another reading room (the old bindery's town) | ≤3.0 of the centre |
| `{house}` | house | the house the rolls shame most, the steward's creditor: a d-l0 house within three days (no faction leader, not the court's holder), else made on demand | ≤3.0 of the centre |
| `{hand}` | person | the copyist's kinswoman (flavour d), a clerk in the court's counting room (made on demand; the court's) | — |
| `{copyist}` | person | a copyist of reading_room, her proxy (made on demand; the hall's) | — |
| `{binder}` | person | the old bookbinder of bindery_hall's town (made on demand) | — |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{reading_room}` | `{reading_room}` | bound |
| R2 | `{lord}` | `{lord}` | bound |
| R3 | `{bindery_hall}` | — | free (≤3.0 of the centre) |
| RV | `{pawn_town}` | `{reading_room}` | bound |
| C1-1 | `{record_house}` | `{lord}` | journey (≤3.5 of the centre): out to the record house |
| C1-2 | `{reading_room}` | `{reading_room}` | bound |
| C1-3 | `{reading_room}` | `{reading_room}` | bound |
| C2-0 | `{record_house}` | `{lord}` | bound |
| DOOR | a: `{record_house}` · b: `{record_house}` | `{reading_room}` | bound |
| crisis | `{reading_room}` | `{reading_room}` | bound |
| crisis, acted | `{reading_room}` | — | where its action falls |

*Parties set up first* (checked): the house (C2-0); the steward (C1-1); the binder (C1-3); the bound copyists (C1-3); the court (C1-1); the forger (RV).

---

## W4-A · The Stalls' Oath

**1 · Enemy:** W4c, the clerk of `{clerk_hall}` who runs its district's letter-writers (Middle), selling to fences which houses stand empty. *Within* · greed · `inside` `coin` `name` · region · *the hidden hand* · side: Writers.

**2 · Scheme.** The letter-writers read every letter they write. He has them tell him what they read, and sells fences his list: whose house will stand empty, and when ("I'm on the salt road till the fair; the key's with your sister").
**Payload ⚑ (crisis):** fair night. Every house whose letters said it would stand empty for `{fair}` is robbed in one night, the heart's among them.

**3 · Stake:** S2b, the carters' letters home. *Reading:* the letters are what he sells; the oath that they are never told is what he breaks.

**4 · Heart:** H3a, `outside`, cast: a carter who can't write, writing through `{stall}` to a daughter in service far off. *Needs:* his house in `{town}` stands empty while he hauls for the fair.
- **Want:** to bring his daughter home with a dowry.
- **Past:** in a letter he told her where the dowry chest is hidden, so she'd have it if he died on the road.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{stall}`, in the heart's own town | Dispute *Broken agreement* (his wage) | the carter against his cart master over a season's pay, which he is saving. **Heart's card**: his want, and his monthly letter at the stall; in one letter he told his daughter where the dowry chest is hidden in the house, "in case the road takes me" (his past) | ∅ "**the oath board over the stall, its last line painted out**" |
| R2 | `{loft}`, on the hill by `{clerk_hall}` | O4 Investigate *Disappearance* (O4b: a relay loft beside the clerk's hall, the pigeons unfed for a week). **Odd job** | find the loft keeper | "**a loft whose pigeons went unfed**" |
| R3 | `{robbed_town}`, another of the district's towns | Acquire *Copy it before it burns* (signature) | a robbed merchant means to burn his letter-book: "my letters brought the thieves" | "**robbed the week his letter said he'd be away**" |

**6 · Odd job:** O4b (thread `wrong-place`; forced *the story's own*). *How it serves the scheme:* the clerk's lists go to the fences by pigeon from this loft. The keeper read one tube and asked about it, so the Hand, the letter-writer who writes the lists, paid him off the hill and fed his birds himself only when a list was due.

**7 · Reveal** (giver: *the leader*; form: *the thing names them*). The leader, off the road at `{hideout}`. Passage *Unwilling passenger*: the loft keeper, found in hiding there, won't come back; the leader persuades him.
- **Shows:** he kept the one tube he opened, the one he asked about: a list of houses and the weeks they would stand empty, in a letter-writer's hand, flown to the fences from his loft; he knows who paid him off the hill (a letter-writer of the district, the Hand), not for whom.
- **Adds:** the leader adds for whom, from the stalls' carriage book she was riding to read: only `{clerk_hall}`'s clerk sends from that loft, so the lists are his sales.
- **Repeats:** "the oath board, its last line painted out", "a loft whose pigeons went unfed", "robbed the week his letter said he'd be away".
- **Closes:** the tube names them itself: the robbed merchant's house is on it, for the week "his letter said he'd be away"; it is tied with the loft's own ring ("a loft whose pigeons went unfed"); and it is written on the back of a stall's spoiled oath sheet, its last line struck through, as on "the oath board, its last line painted out".

**8 · Chapter 1.** *Shape:* 3 · the early turn. **Move:** M7 on S2, as his answer to being looked at: a letter-writer he has turned off is paid to read a carter's letter aloud, drunk, in a market square, so the town blames one stall and not a list. *Uses* `name`, lever rumour.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{town}`'s market square | Restore *Lost good name* (cause *exaggerated*) | — | the turned-off letter-writer, shamed as the drunk who reads letters out, says what the clerk told the stalls their reports were for (flavour b). Plant: "**a letter-writer paid in Market notes**" |
| C1-2 turn | a fence's shop in `{fence_town}` | Acquire *Unknown worth* | but | the Market notes lead to a fence, the clerk's buyer (a proxy, opposing), whose boy holds a pigeon tube and doesn't know what's in it. Plant: "**a list of houses and dates in a stall's hand**": last fair's list, every house on it robbed then *(draft 4: the tube is last fair's; this fair's list flies from the loft on fair eve)* |
| C1-3 | the road out of `{fence_town}` | Passage *Fugitive* | therefore | a boy the fence sent into a house that wasn't empty was caught, let go, and is now hunted by the fence, who thinks he talked. **Ally asks:** `{ally}` (flavour c, voiced here), who has seen one of the lists, asks the player to get the boy out of the fence's reach; the price, later: a copy of last fair's list |

**9 · Chapter 2.** Opening (pressure 2): `{town}`, Threat *Go and look* (C1): see which of the town's streets the fences' crews have walked; they mark nothing, and on every street they walked half the houses have a carter on the road for `{fair}` (at `{fair_town}`); `{painter}`, the town's old sign-painter, who knows every door, walks them with the player. Then the door.

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | player · *framed* · for taking a side in public (`framed-shame`) | `name` | Dispute *Public offense* (offended: the clients whose business the player "told the watch") | if the player warns `{town}`, it must look like a Writer selling clients' secrets, not the clerk |
| b | guild · *hall seized* · shut by his own order (`seized-order`) | `inside` | Acquire *Under seal* (the stalls' room and its books, under his rule) | he runs the letter-writers; he shuts their room "while the drunk is dealt with", and the books with it |
| c | guild · *leader struck down* · wounded through something rigged (`wounded-rigged`) | `inside` | Investigate *Sabotage* (her mule's girth, cut in the reading room's stable) | the leader was coming to read the stalls' books herself |

- **The door carries the crisis's plant:** its knower is `{painter}` (met on chapter 2's opening), who paints every sign and board for the district's halls and stalls, so whichever door is dealt he has been there (under door c he was repainting the reading room's stable board, and saw who went to the mule): he painted out the oath's last line ten years ago at the clerk's order: "**painted out at the clerk's order**".
- **Lesser code-break (kind: take a side in public):** an extra way on the door: tell one family, privately, who sold their letter.

**10 · Crisis** (in `{town}`, under the stall's oath board). **Last move ⚑:** the night of `{fair}`. Every house on this fair's list, flown from the loft on fair eve, is robbed; the heart's dowry chest is gone. Called to account before the town, the clerk says he sold no letter, and that the oath binds the writer at the stall, not the hall. "The halls have always known what the stalls read." The enemy's want survives: there will be another fair.
- **Family:** Dispute *Broken agreement*. Party A: the heart, for the stalls' clients. Party B: the clerk of `{clerk_hall}`, for the stalls (link: trade; the clients pay the stalls). The agreement: the stalls' oath, painted on the board and first written in the Wayhouse's founding book, whose copy every hall keeps in its charter chest *(Claude)*. Arbiter: the leader, off the road at `{town}`, where she was coming to read the stalls' books (by letter, read out by `{clerk_hall}`'s hall master, if blow c laid her up). The player is the heart's advocate.
- **Held:** the player holds last fair's list (C1-2) and the crews' streets (chapter 2's opening), not this fair's houses, which fly from the loft on fair eve; a warning to whole streets of absent carters puts the watch in the wrong lane, and a Writer telling the watch whose letters said what is the stalls selling secrets (door a's charge). So fair night lands, and the crisis reads a warning given: if the player warned the watch, the crews took the streets it wasn't in, and the town blames the halls' warning along with their stalls.
- **Giver:** the heart, in his robbed house. *Reason:* his daughter's dowry.
- **Choice:** does the stalls' oath bind the halls?
- **Ways:** *code-break:* take a side in public: the halls name their own clerk and stand with the clients · *ally:* `{ally}` swears the list's houses and dates against fair night (the price: the list) · *truth (the family's one document way):* the oath's first wording, in `{clerk_hall}`'s copy of the founding book: "nor shall the halls be told".

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **The heart wins** (the oath binds the halls; the clerk struck off; the robbed told who sold them) | the clerk · the guild's quiet (the halls admit the stalls were read) | law | the dowry half-repaid from the clerk's takings / the stalls' board repainted, whole / the clerk sells his list one last time, to anyone |
| **The clerk wins** (the halls keep the stalls' reading; the clerk stays, and says he'll sell no more) | the heart (no redress) · every stall client | persuasion | writes his daughter that there's no dowry / the board stays painted out / the clerk waits for the next fair |
| **A deal is struck** (the clerk repays the robbed and goes quietly; nothing said of the halls' reading) | the clerk's purse · the heart's daughter (part of the dowry only) | bribe | sends for his daughter, short / the town thinks it was the drunk / the clerk runs a stall in another region |
| **The player takes the stake** (B7: the player keeps the clerk's list) | every client on it · the heart | none | trusts no letter-writer again / the robberies stop, for now / the clerk knows who holds his list |
| **Take a side at last** (code-break: the halls stand with the clients before `{town}`) | the guild's neutrality: every stall must now swear before its town | law | dictates his letters to `{ally}` now / the stalls sworn in public / the clerk struck off and blamed for everything, some of it not his |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *the story's own* (`the-enemy-did-it`): the Hand paid the keeper off | the feed sack was bought a month ahead |
| C1-1 | cause *exaggerated* | she read one letter, and the crowd swears she read ten |
| C1-2 | *a copy* (`a-copy`): the list in the tube is a copy; the original is in the clerk's book | the list is in a letter-writer's hand, not a fence's |
| C1-3 | *not what it seems* (`more-than-said`): the boy has next season's list too | he keeps one hand in his coat |
| door a | *one side lies* | the clients the clerk names have never met the player |
| door b | *a copy*: the shut room's books are copies; he has the originals at home | dust marks on the shelves where larger books stood |
| door c | the Hand cut it | a clean cut, made with a pen-knife |
| crisis | *record survives* (`record-survives`): the founding book's wording | the paint over the board's last line is newer than the rest |

**13 · Hall-job hooks**
- *Middle:* at a stall town, a carter wants a letter written by someone who will read it back only to him. Reads M7; *before* face: "talk of a drunk letter-writer".
- *Late:* at `{clerk_hall}`'s town, a family going to the fair wants its house's key left with the reading room, not a neighbour. Reads fair night; *before* face: "the fair's coming".
- *Seen in passing:* before the reveal, a stall letter-writer kindly reads a soldier's letter to his mother on a side card.

**14 · What varies.** `{clerk_hall}` and its district; `{fair}` (rolled: the fair's name) and `{fair_town}`; `{town}` (the heart's, the stall's); `{stall}` (in a smaller town, run from `{clerk_hall}`); `{fence_town}` (the map's fence's shop where in reach, else one added in a Night Market town). *Flavour* (each list rolled on its own): (a) how a letter gave a house away, told on R3 by the robbed merchant: the dates of a journey, where the key is left, or that the dog goes with him; (b) what the clerk tells the letter-writers the reports are for, told on C1-1: the halls' own safety, a tally for the Wayhouse, or nothing (they're paid); (c) the ally, met on C1-3: the watch sergeant of `{town}`, whose own house was on a list; a robbed innkeeper of `{town}`; or a fence's crewman who wants out. *(Draft 1's carters' yard and granary moved the robbery out of the house where his chest is hidden; its lists "in the Wall's own post" left the loft and the pigeon tube nothing to do. Both are gone.)*

**15 · Presuppositions:** `{clerk_hall}`, `{town}`, `{stall}`, `{fair}`, `{fair_town}` (slots) · the heart's letter about the chest (his past, R1) · the oath board and its painted-out line (R1) · the founding book's oath *(Claude)* (crisis; the board on R1 points to it) · the loft and its keeper (R2) · `{hand}` = his list-writer (R2 hidden; C1-3 behind the fence) · the turned-off letter-writer (C1-1) · the fence and his boy, and last fair's list in the tube (C1-2) · `{ally}` (C1-3, asks) · this fair's list flown on fair eve (crisis) · `{painter}` (chapter 2's opening; door) · every hall keeps a copy of the founding book *(Claude)* (crisis).

**16 · C1 cards:** ch2 opening Threat *Go and look*.

**17 · Placing.** Centre: `{clerk_hall}`. Main town (for *Fresh stories*): the centre's town. Kept in memory too: stake S2, move M7, openings Restore · Lost good name, Threat · Go and look. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{town}`, where the story is; the giver is there (the heart, in his robbed house in his own town); its action falls at `{town}`.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{clerk_hall}` | hall | the reading room whose clerk runs its district's stalls (the enemy's seat) | centre |
| `{stall}` | stall | a letter-writer's stall (presence) run from clerk_hall | ≤1.25 of the centre, else ≤1.6 (nearest) |
| `{town}` | town | the heart's own town = the stall's | — |
| `{loft}` | site | the relay loft on high ground (added) | 0.5 from the centre |
| `{fence_town}` | town | a Night Market fence's shop (presence); else a large Night Market town, the shop added | ≤3.0 of the centre |
| `{fair_town}` | town | a town the heart hauls to (road-linked preferred) | ≤2.5 of the heart's town |
| `{fair}` | day | the fair at fair_town (roller: a fair's name) | — |
| `{robbed_town}` | hall/stall | any town with a Writers' presence | ≤3.0 of the centre |
| `{hideout}` | town | where the loft keeper hides | ≤1.25 of the loft |
| `{hand}` | person | his list-writer (made on demand) | — |
| `{ally}` | person | someone who has seen a list (made on demand; see flavour c) | in the heart's town |
| `{painter}` | person | the old sign-painter of the heart's town (made on demand) | — |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{stall}` | `{stall}` | bound |
| R2 | `{loft}` | `{clerk_hall}` | bound |
| R3 | `{robbed_town}` | — | free (≤3.0 of the centre) |
| RV | `{hideout}` | `{loft}` | bound |
| C1-1 | `{town}` | `{stall}` | bound |
| C1-2 | `{fence_town}` | `{fence_town}` | journey (≤3.5 of the centre): to the fence's shop |
| C1-3 | `{fence_town}` | `{fence_town}` | journey (≤3.5 of the centre): the road out of the fence's town |
| C2-0 | `{town}` | `{stall}` | bound |
| DOOR | a: `{town}` · b: `{clerk_hall}` · c: `{clerk_hall}` | — | bound |
| crisis | `{town}` | `{stall}` | bound |
| crisis, acted | `{town}` | — | where its action falls |

*Parties set up first* (checked): the painter (C2-0); the ally (C1-3); the leader (RV); the clerk (RV); the heart's daughter (R1).

---

## W4-B · Who Is Poor

**1 · Enemy:** W4a, the clerk of `{clerk_hall}` (a reading room, or the Wayhouse itself) who runs its district's letter-writers (Middle), selling to moneylenders who want to know who is poor. *Within* · greed · `inside` `coin` `name` · region · *the hidden hand* · side: Writers.

**2 · Scheme.** The stalls write every begging letter ("give me till harvest"). He sells moneylenders the names; they buy those debts cheap from creditors tired of waiting, then call them in. He takes his share in the bought debts themselves, and keeps a few for his own.
**Payload ⚑ (crisis):** quarter day in `{town}`. Every debt on his list is called in; families who can't pay go to `{broker}`, a Binders' debt broker, and the heart's sister is held at the clerk's rooms as surety for a debt the clerk bought for himself.

**3 · Stake:** S2a, the season's undelivered letters. *Reading:* the begging letters are worth more to him than any debt in them.

**4 · Heart:** H5b, `outside`, a man (the son): the son of a copyist who died in debt at his copy table; he copies by night at a bindery in his own town, `{town}`, to pay the debts his father left. *Needs:* his father's debt is owed to `{old_creditor}` in `{town}`, and he has begged her for time through `{stall}`. *(Draft 4: no longer a lost surveyor's son. Three of sixteen hearts were, and the once-a-set cap on that heart kept this spine out of all but four sets in twenty-one, so W4-A came up in seventeen.)*
- **Want:** to pay the last of his father's debt by midsummer, so his sister can marry clear of it.
- **Past:** he wrote his first begging letter at fourteen, the month his father died.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{town}`, the heart's own town (the stall's) | Restore *Lost livelihood* (cause `lost-knowledge`) | a bookbinder's widow can't run his bindery; the heart copies there by night. **Heart's card**: his debt, his want, and his sister, who is to marry; he wrote his first begging letter at fourteen, the month his father died at his copy table (his past; flavour c says what the debt was for); his uncle, `{uncle}`, a carter who hauls for the bindery, tells him begging letters shame the family | "**a moneylender's clerk buying ink at the stall bench**" (the stall's bench, across the lane from the bindery) |
| R2 | `{steward_seat}`, a lord's seat within three days | O1b Acquire *Copy it before it burns*. **Odd job** | the steward of `{steward_seat}`'s holder is told to burn the accounts of his master's late predecessor | "**a steward's accounts of debts his old lord forgave**" |
| R3 | `{r3_hall}` | Investigate *The ledger doesn't add up* (signature; *swap* below) | a moneylender's loan roll copy was altered (*for themselves*) | ∅ "**no debt bought from anyone who can write**" |

*Swap:* **R3** may be Investigate *Watch them*: `{r3_hall}`'s master wants eyes on a moneylender's clerk who comes every week to read the hall's debt sales; watching, the player sees that among the debts he buys there is "no debt bought from anyone who can write" (the plant is kept). R2 is the routine's signature either way.

**6 · Odd job:** O1b (thread `answers-to`). *How it serves the scheme:* the Hand, the clerk's letter-writer, told the steward the house would be better without the old accounts, and copied them first. Now the clerk knows which forgiven debts can be sold again as live ones, and the only record that they were forgiven is ash.

**7 · Reveal** (giver: *the heart*; form: *laid side by side*). At `{town}`, at the stall bench where the heart's begging letters are written. Dispute *Shared resource*: two of its stall-keepers quarrel over the bench nearest the moneylenders' row; the heart brings the player, since the older one writes his letters. She is the knower: she refused to sell what she read, and lost her bench.
- **Shows:** the clerk's letter-writer asked her for every begging letter's name and sum, paid by the name, and keeps a book of them, which she has seen and which he calls by a name (flavour a); she wrote to the leader of it and had no answer.
- **Adds:** the heart adds what the book is for, from the notice in his own pocket: his father's debt was bought last month by the moneylender whose clerk buys ink at this bench; the old stall-keeper adds, from what the letter-writer let slip, that the clerk of `{clerk_hall}` takes his share in the bought debts themselves.
- **Repeats:** "a moneylender's clerk at the stall bench", "debts his old lord forgave", "no debt bought from anyone who can write".
- **Closes:** the heart lays his debt notice beside the bench where "a moneylender's clerk" buys his ink, the copied accounts of "debts his old lord forgave" beside the debts now sold as live, and the hall's debt sales beside his own begging letters: "no debt bought from anyone who can write", and none from anyone who hadn't begged.

**8 · Chapter 1.** *Shape:* 2 · the ally first. **Move:** M9 on S2: before the hall, the clerk dismisses the Wall clerk who asked who reads the begging letters. *Uses* `inside`, lever blackmail.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{hamlet}`, a weavers' hamlet out from `{town}` whose debts were bought (a journey) | Acquire *Unknown worth* | — | a weaver family holds their old creditor's letter, "our creditor gave us till harvest!", not knowing it voids the debt the moneylender bought; the old creditor herself, `{old_creditor}`, has come out to the hamlet, ashamed. Plant: "**a debt bought for a tenth of its face**". **Ally plant:** `{old_creditor}`, who gave the heart time too and was tricked into selling his father's debt with theirs; she wants to buy them back; her price: her money returned |
| C1-2 | `{broker}`'s yard | Threat *Go and look* (C1) | therefore | go and see who waits in the Binders' broker's yard (a broker in the map's presences, or one the placer adds in `{town}`, hidden where the town is Open Hand). Plant: "**families from one stall's letters, waiting in the yard**" |
| C1-3 turn | `{stall}` | Dispute *Public offense* | but | the Hand cries down the old stall-keeper who struck him in the street, and demands redress (opposing); his cry names `{old_creditor}` before the stall as the usurer who sold her debtors, so the ally is shamed before the door, and her oath (the crisis's ally way) will now cost her her trade |

**9 · Chapter 2.** Opening (pressure 2): `{clerk_hall}`, Acquire *Under seal*: his father's writing case and seal, the family's one thing of worth, lodged in the hall's strongroom as surety by the debt's new holder, whom the strongroom book doesn't name: "**surety lodged for a holder unnamed**". Then the door.

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | heart · *estranged* · his kin turned (`kin-turned`) | `name` | Reckoning *Betrayal* (one kin member: `{uncle}`, met on R1, who tells the town what the boy's letters begged) | the clerk's man tells the uncle the begging letters shame the family; the uncle adds no secret the buyers lack, only the shame, and with it public, the families on the list won't stand with a beggar |
| b | player · *stripped of rank* (`stripped`) | `inside` | Restore *Lost good name* (the stain: "reads the Wall's letters") | the player has been asking at the stalls |
| c | guild · *leader struck down* · disgraced (`disgraced`) | `inside` | Restore *Lost good name* (the stain on the leader: "the head of the Writers sells what the stalls read") | caught at it, he spreads that the trade is the leader's; it is half true of the guild, which makes it stick |

- **The door carries the crisis's plant:** its knower, the moneylender's clerk (met on R1), says the boy's father's debt "wasn't ours to buy; he kept that one for himself": "**a debt the clerk kept for himself**".
- **Lesser code-break (kind: destroy a text):** an extra way on the door: burn one bought-debt slip to free one family.

**10 · Crisis** (at the clerk's rooms behind `{clerk_hall}`). **Last move ⚑:** quarter day. Every bought debt is called in; families who can't pay go to the broker's yard. The heart's sister is held at the clerk's rooms behind `{clerk_hall}` (flavour b says at what work), as surety for their father's debt. The heart comes to the clerk's door, a day's walk at most from his town; the ways go through his rooms. The enemy's want survives: the yard pays him too.
- **Family:** Rescue *Bound by debt*. Captor: the clerk, as creditor. Captive: the heart's sister. Place: his rooms behind `{clerk_hall}`. Requester: the heart.
- **Held:** the player knows the clerk's sales from the reveal before quarter day, but the debts are lawfully bought and fall due on quarter day whatever anyone knows, and the stalls' clients won't stand together as beggars; the one thing that answers a bought debt is proof that it was paid, which the crisis's tell and the old creditor's oath hold.
- **Standing:** if blow b stripped the player, the player comes to the clerk's door as the heart's friend and the creditor's witness, not as a Writer of his hall; the clerk can shut his rooms to his hall's Writers, not to a witness to a payment.
- **Giver:** the heart, at the clerk's door. *Reason:* his sister.
- **Choice:** free her by paying, by proof, or by the halls' own fire.
- **Ways:** *code-break:* burn the clerk's debt ledger, freeing every family in it and destroying every true debt with them · *ally:* `{old_creditor}` swears the last payment was made before she sold (her price: her money back from the clerk) · *work it off:* the heart takes service in his sister's place.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Freed** (the creditor's oath: the debt was paid) | `{old_creditor}` (she swears she sold a paid debt; she lends no more) | law | his sister marries in summer, as he wanted / the yard's other families stay held / the clerk loses one debt and keeps the rest |
| **Burn your copy** (code-break: the ledger burned; every debt in it unprovable) | the honest lenders whose true debts were in it · the halls (a halls' clerk's own ledger burned) | stealth | his sister is free and he won't say how / the yard empties, and the town's credit dries up / the clerk is ruined, and free |
| **Exchanged** (the heart works off the debt in her place) | the heart (bound to the clerk for a year) | none | copies for the clerk, and sees every letter / his sister goes home / the clerk has a copyist who can't refuse |
| **Open the ledgers** (the clerk's ledger and lists published at the Wall) | the clerk · the stalls (no one will write through them now) | law | his sister free, his begging letters read by the whole town / the families learn who sold them / the clerk is struck off and the moneylenders find another way |
| **Handed to another** (B7: the player sells the sister's debt on to `{old_creditor}`, who will "be kinder") | the heart's sister (bound to a kinder creditor, still bound) · the heart | bribe | owes a woman he trusts, and can't forgive the player / the yard stays full / the clerk takes the player's coin and smiles |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *already copied* (`a-copy`): the Hand copied the accounts first | the pages are pricked at the edge, as copyists prick them |
| R3 | culprit *for themselves* | the moneylender's clerk has a new house |
| C1-1 | the old creditor's letter granting till harvest is dated before the sale, so the debt was sold with time on it | the family quotes a letter they can't read |
| C1-2 | the yard's families all wrote through one stall | the yard keeper calls them by their town, not their names |
| C1-3 | *one side lies*: the Hand struck first | the bruise is on her cheek, not his |
| door a | *still profiting* (`still-profiting`): the uncle has the boy's share of the family cart | the uncle's cart has new wheels |
| door b | cause *false, spread off the card* | the story came from the stalls, not the hall |
| door c | *shared blame* (`shared-blame`): the guild does trade in secrets | the leader doesn't deny it |
| crisis | *paid* (`paid-already`): the boy's copying paid it | his own tally of payments adds up to the debt's face |

**13 · Hall-job hooks**
- *Middle:* at `{clerk_hall}`'s town, a widow wants a letter sent "not through the stalls". Reads M9; *before* face: "the Wall clerk's been turned off".
- *Late:* at another hall, a family wants a copy of the paid-off note their creditor gave them, "before it's sold". Reads quarter day; *before* face: "quarter day's coming".
- *Seen in passing:* before the reveal, a reading-room copyist teaches a child her letters for nothing on a side card.

**14 · What varies.** `{clerk_hall}`, its `{stall}` and `{town}` (where the heart lives and copies); `{broker}` (a Binders' debt broker, hidden where the town is Open Hand); `{steward_seat}` (registry); `{old_creditor}`. *Flavour* (each list rolled on its own): (a) what the clerk calls his list, published on the reveal: the poor book, the patience list, or nothing at all; (b) the sister's work while held at his rooms, seen on the crisis: in service, at the clerk's copy table, or minding the clerk's children; (c) what his father's debt was for, told on R1: the presses of a bindery that failed, his mother's burial, or a lawsuit his father lost; the debt is the same either way. *(Draft 1's Sharp Hands or steward as buyers contradicted the moneylender's clerk on R1, R3 and the door; holding the sister at the broker's moved the crisis's place.)*

**15 · Presuppositions:** `{clerk_hall}`, `{town}`, `{stall}`, `{broker}`, `{steward_seat}` (slots) · the bindery in the heart's town (R1) · the father's debt and `{old_creditor}` (heart's spec; C1-1) · `{uncle}` (R1; door a) · the writing case held as surety (ch2 opening) · the heart's sister (his want; crisis) · the moneylenders' buying (reveal; C1-1) · the steward's forgiven-debt accounts (R2) · `{hand}` = the clerk's letter-writer (R2 hidden; C1-3 open) · the old stall-keeper (reveal; C1-3) · the clerk's rooms behind `{clerk_hall}` *(Claude)* (crisis) · the moneylender's clerk (R1, door).

**16 · C1 cards:** C1-2 Threat *Go and look*; R3's swap Investigate *Watch them*. *(Draft 3: C1-1 is Acquire Unknown worth, so chapter 1 no longer holds two Dispute cards.)*

**17 · Placing.** Centre: `{clerk_hall}`. Main town (for *Fresh stories*): `{town}`. Lords and courts kept in memory: `{steward_seat}`. Kept in memory too: stake S2, move M9, openings Acquire · Unknown worth, Acquire · Under seal. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{clerk_hall}`, where the story is; the giver is there (the heart, at the clerk's door); its action falls at `{clerk_hall}`.
*Swaps* (field 5): R3 may be Investigate · Watch them.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{clerk_hall}` | hall | the reading room (or the Wayhouse) whose clerk runs its stalls | centre |
| `{stall}` | stall | a letter-writer's stall (presence) run from it | ≤1.25, else ≤1.6 (nearest) |
| `{town}` | town | the heart's town = the stall's; he copies by night at its bindery; his father's creditor lives there | — |
| `{broker}` | town | a Binders' debt broker (presence); else one added in the heart's town (hidden if Open Hand) | ≤1.25 of the heart's town, else ≤1.6 |
| `{hamlet}` | site | the weavers' hamlet whose debts were bought (added) | 0.7 from the heart's town |
| `{steward_seat}` | seat | a High holder (Banner/Free Lord, Seated House head) whose steward keeps his late predecessor's accounts | ≤3.0 of the centre |
| `{r3_hall}` | hall | any other hall | ≤3.0 of the centre |
| `{hand}` | person | the clerk's letter-writer (made on demand) | — |
| `{uncle}` | person | the heart's uncle, a carter of the heart's town (cast with the heart's household) | in the heart's town |
| `{old_creditor}` | person | his father's old creditor, in the heart's town (made on demand) | — |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{town}` | `{stall}` | bound |
| R2 | `{steward_seat}` | — | free (≤3.0 of the centre) |
| R3 | `{r3_hall}` | — | free (≤3.0 of the centre) |
| RV | `{town}` | `{stall}` | bound |
| C1-1 | `{hamlet}` | `{town}` | journey (≤3.5 of the centre): out to the hamlet |
| C1-2 | `{broker}` | `{town}` | bound |
| C1-3 | `{stall}` | `{stall}` | bound |
| C2-0 | `{clerk_hall}` | `{clerk_hall}` | bound |
| DOOR | a: `{town}` · b: `{clerk_hall}` · c: `{clerk_hall}` | — | bound |
| crisis | `{clerk_hall}` | `{clerk_hall}` | bound |
| crisis, acted | `{clerk_hall}` | — | where its action falls |

*Parties set up first* (checked): the uncle (R1); the heart's sister (R1); the old creditor (C1-1); the moneylender's clerk (R1); the leader (RV); the clerk (RV); the old stall-keeper (RV).

---
## W5-A · The Letters Never Collected

**1 · Enemy:** W5a, a Holy House inquisitor (High), seizing under a heresy charge. *Above* · cover-up · `faith` `paper` `men` · region · *the old debt* · side: Holy House.

**2 · Scheme.** Generations ago the Writers drew up the Mount's land grants, for a fee: the grants the house calls god-given are Ṭaḍoro work for hire, and the Writers took a side to write them. He burns as heresy every text that records it, before either side can use it. The last are the drafters' own letters, sealed and never collected, in the Lending Wall's oldest pigeonholes at the Wayhouse.
**Payload ⚑ (crisis):** by a Highcrossing writ on his heresy charge, the oldest pigeonholes are opened and their letters burned in the Wayhouse's yard at dawn.

**3 · Stake:** S2c, the sealed letters held for collection. *Reading:* the Wall's promise is that a letter waits for its reader; he means to make the oldest wait for nobody.

**4 · Heart:** H5c, `outside`, a man (the son): the son of a surveyor who went into the green and never came back; he still writes to him through the Wall. *Needs:* his letters wait, sealed, in his family's pigeonhole, inherited from his grandfather's grandfather, in the Wall's oldest section, where his father's own letters waited before his *(draft 4: a nine-year-old hole needs a reason to be among two-hundred-year-old ones)*.
- **Want:** to find someone who walked with his father on his last survey.
- **Past:** he has written to him every month for nine years, and the Wall's clerk files each letter unopened. *(Draft 3: the lost surveyor is his father; W8-B's is a mother.)*

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | the Wayhouse (his letters wait in its Wall) | Dispute *Broken agreement* (the pigeonhole fee) | the son against the Wall's clerk, who says his father's hole is overfull and nine years unpaid. **Heart's card**: he asks everyone if they knew his father; he has written to him every month for nine years (his past) | "**the Wall's oldest pigeonholes, sealed letters two hundred years uncollected**" |
| R2 | `{widow_hall}`, a hall within three days | O1a Acquire *Copy it before it burns*. **Odd job** | a dead copyist's widow means to burn his letters | "**a dead copyist's letters, burned 'for the house's mercy'**" |
| R3 | `{tithe_hall}`, a hall whose town has a tithe-house | Investigate *The ledger doesn't add up* (signature; *swap* below) | a tithe-house book copied at the hall was altered (*for themselves*); comparing it with the old grant charter the hall also keeps | ∅ "**the grant charter with no copyist's name at its foot**" |

*Swap:* **R3** may be Investigate *Watch them*: `{tithe_hall}`'s master wants eyes on the tithe-keeper, who has begun locking the old grant charter away at night; watching, the player sees him take it out to a buyer by lamplight, and sees "the grant charter with no copyist's name at its foot" (the plant is kept). R2 is the routine's signature either way.

**6 · Odd job:** O1a (thread `answers-to`). *How it serves the scheme:* the Hand, a pious copyist in the inquisitor's pay, told the widow her husband's letters were heresy and the house would be merciful if they burned. He copied them first. They name the pigeonholes where the drafters' letters wait, so the inquisitor now knows exactly where to look.

**7 · Reveal** (giver: *a found letter*; form: *the thing names them*). At `{tithe_hall}`. Acquire *Unknown worth*: an old woman's copying desk, her grandfather's, holds in its drawer the drafters' own copy of their covering letter to the Mount, unread for a century; the player reads it. *(Draft 4: the letter sent to the Mount would be at the Mount; this is the drafters' copy.)*
- **Shows:** the letter: the Mount's grants were drawn by Ṭaḍoro copyists of a Writers' drafting house, for a fee, and signed by none of them (the old debt); and the drafters' own letters, it says, are left at the Wall for their readers, in the decade it names.
- **Adds:** the old woman adds that a pious copyist came last month offering to buy the desk for the house, "to be kept shut, for the house's mercy"; so the inquisitor's burnings, which the parish talks of, are for silence, and a shut drawer would serve him as well as fire.
- **Repeats:** "the Wall's oldest pigeonholes", "letters burned 'for the house's mercy'", "the grant charter with no copyist's name".
- **Closes:** the letter names two of the plants in its own words: the grants are "signed by no copyist of ours", as on "the grant charter with no copyist's name at its foot", and the drafters' letters wait "in the Wall's oldest pigeonholes" for their readers; the old woman's buyer said the third: "for the house's mercy".

**8 · Chapter 1.** *Shape:* 2 · the ally first. **Move:** M8 on S2: the inquisitor has the Wall branch at `{branch_hall}`, a reading room in First Peoples land where the house's word holds, searched for heresy and its pigeonholes sealed for a week. *Uses* `faith` (not a writ), lever faith. Shuts a place.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{branch_hall}`'s sealed branch (a journey) | Passage *Smuggling* | — | carry the carters' ordinary letters out of the sealed branch past the inquisitor's guard; the branch's old Wall clerk, who keeps its index, hands out what letters he dares, and `{ally}` (flavour c, voiced here), of the dead copyist's house, has come for the dead man's last letters. Plant: "**the inquisitor's clerk asking which letters came from the Wayhouse's old wall**". **Ally plant:** `{ally}`, who says the dead copyist walked with the heart's father on his last survey (his want, again); the price: the dead copyist's name read at the Wayhouse |
| C1-2 | `{inq_camp}`, the inquisitors' camp on the road beyond (a journey) | Threat *Go and look* (C1) | therefore | go and see what the camp has burned. Plant: "**a burned page with a Ṭaḍoro copyist's mark**". Second plant, from the camp's cook, in flavour b's placeless words: before any burning "**they sleep at the chapel of the town they burn in, and burn at dawn**" (or at an inn by its gate, or the wayside shrine nearest it); the crisis names the place for Highcrossing, `{lodging}` *(draft 4: the cook, days from Highcrossing, can't say "Highcrossing's chapel")* |
| C1-3 turn | the letter-writer's stall at `{mount}` | Acquire *Won't sell* | but | the Hand (opposing) holds the copies he made of the dead copyist's letters and won't sell them to the halls, and he has told `{ally}` that the halls will burn the dead copyist's name with the rest; the ally wavers, and will give what the house's papers say (the crisis's ally way) only once the name has been read |

**9 · Chapter 2.** Opening (pressure 2): `{branch_hall}`'s town, Restore *Lost livelihood*: the branch's letter-carriers out of work while it is sealed. Then the door.

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | guild · *members taken* · arrested on a charge (`taken-arrested`) | `faith` | Rescue *Imprisoned unjustly* (captor: the house, on the inquisitor's charge) | the branch's Wall clerks know the old index of pigeonholes; he wants it, and them quiet |
| b | player · *arrested* · on a charge (`arrested-charge`) | `faith` | Passage *Fugitive* (cargo: the player; pursuer: the Hand, on the charge) | the player was seen with the burned page |

- **The door carries the crisis's plant:** the branch's old Wall clerk (met on C1-1) recites from the branch's old index which pigeonholes belong to the drafters' decade: "**the old index of the drafters' pigeonholes**".
- **Lesser code-break (kind: destroy a text):** an extra way on the door: burn the branch's old index, so he can't find the holes.

**10 · Crisis.** **Last move ⚑:** the inquisitor obtains a writ from `{magistrate}` on his heresy charge (the story's one writ; the magistrate sits by the Throne's leave, and the Throne keeps the Mount's goodwill, so a house inquisitor's charge gets its writ). The Wall's oldest pigeonholes are to be opened and burned in the Wayhouse's yard at dawn; the heart's father's hole lies in that section, with his nine years of letters; the heart is in the yard, as he comes every month. The enemy's want survives: the burning hasn't happened yet.
- **Family:** Acquire *Copy it before it burns*. Holder: the inquisitor, holding the writ's authority. Requester: the heart. Pressure: dawn.
- **Held:** the player has known the drafters' letters are his target since the reveal, but no Writer may open or move a sealed letter before its reader comes (the Wall's promise, its whole worth), so the halls can only guard the Wall until the writ comes; the copy way breaks that promise, and that is its cost.
- **Giver:** the heart, at the Wall where his letters wait. *Reason:* his letters, and his father's hole.
- **Choice:** copy them, keep them, or let them burn, and so decide whether the Writers' oldest compromise is read.
- **Ways:** *code-break:* burn them yourself tonight, so no one reads them · *ally:* `{ally}` has the dead copyist's old copies, which say which holes hold what (the price: his name read) · *copy in secret* while the inquisitors sleep at `{lodging}` (planted on C1-2) · *trade:* give the holder something he'd rather have than the fire, the Wayhouse's silence on the grants for ever.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **A copy in every hall** (every drafter's letter copied before dawn and lodged at each hall) | the guild's name and the Mount's (the old debt is out) · the heart (marked by the inquisitor as the one who held the lamp) | stealth | goes on writing to him; the hole is empty but his / the grants are argued in every hall / the inquisitor is recalled to the Mount, and blamed |
| **Burn your copy** (code-break: the halls burn them first) | the heart (his father's hole burns too) · the record | stealth | writes to him again, starting a new hole / the old wall stands empty / the inquisitor finds ash, and is satisfied |
| **Trade what we know** (the letters stay sealed and unburned, for the Wayhouse's silence) | the guild's freedom (an inquisitor holds its promise) · the Mount's tenants (the grants stand unexamined) | bribe | his letters stay in the hole / nothing changes, which is the cost / the inquisitor keeps the Writers' word in his sleeve |
| **Left with the holder** (they burn at dawn) | the heart (his letters too) · the record | none | stops writing / the yard is scorched / the inquisitor moves on to the next hall |
| **Open the ledgers** (the letters read aloud in the yard before the writ is served) | the guild's name · the inquisitor (shamed in his own house) · the Mount (its grants questioned) | persuasion | hears a stranger read his father's hole's neighbours aloud / the town talks of nothing else / the inquisitor is ruined, and his writ stands |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *already copied* (`a-copy`): the Hand copied the letters before the burning | their ribbons were retied in a different knot |
| R3 | culprit *for themselves* | the tithe-keeper's new boots |
| C1-1 | *the pursuer was told* (`sold-from-inside`): the inquisitor's clerk knows which letters the carters want out | the house's seal is on the door, but the guard is the inquisitor's own man, and he asks for the carters by name |
| C1-2 | the camp's fire holds Wayhouse pages | the camp buys Writers' ink |
| C1-3 | *a copy*: his copies are copies; he burned the originals | his copies stop mid-sentence where a page was torn |
| door a | *innocent*, twist *not alone*: a lamp-boy is held with them | the jailer asks for a second bowl |
| door b | *the pursuer was told*: the branch's porter told the Hand the player's road | the porter offers directions nobody asked for |
| crisis | *it names the holder* (`damning-record`): the letters name his own family as the grants' payers | the writ names one decade's pigeonholes only, which no heresy charge would care about |

**13 · Hall-job hooks**
- *Middle:* at a hall in First Peoples land, carters want letters from the sealed branch fetched or sent elsewhere. Reads M8; *before* face: "inquisitors on the road to `{branch_hall}`".
- *Late:* at Highcrossing, a family whose ancestor's letter waits in the old wall wants it collected at last. Reads the writ; *before* face: "talk of inquisitors in Highcrossing".
- *Seen in passing:* before the reveal, a parish priest blesses a carter's cart kindly on a side card.

**14 · What varies.** `{branch_hall}`: a Wall branch (a reading room or a letter-writer's stall) in First Peoples land, not the Mount, within a journey of Highcrossing. The decade of the drafters (any, a century or more back). *Flavour* (each list rolled on its own): (a) what his letters to his father tell, said on R1: the farm, his own child born last spring, or the camp's news; (b) where the inquisitors lodge before a burning, told on C1-2 by the camp's cook in placeless words (the chapel of the town they burn in, an inn by that town's gate, or the wayside shrine nearest it), and named for Highcrossing on the crisis as `{lodging}` by the roller (Highcrossing's chapel, an inn by Highcrossing's gate, or the nearest wayside shrine); (c) the ally, `{ally}`, met on C1-1: the dead copyist's widow, his son, or his old apprentice. *(Draft 1's "treasurer's receipts" and "a scholar's account" contradicted the drafters' pigeonholes and the Ṭaḍoro mark; its "old woman with the desk" as ally was the reveal's own knower.)*

**15 · Presuppositions:** the old debt (kit) · the Wall's oldest section (faction-places: the Lending Wall; R1) · the heart's father's hole there (his spec; R1) · the branch's old Wall clerk (C1-1; door) · the widow and her husband (R2; C1-1) · `{hand}` = the pious copyist (R2 hidden; C1-3 open) · the grant charter copy (R3) · `{branch_hall}` (slot) · the camp, and where they sleep before a burning (C1-2; *a copy in every hall*) · the old index (door) · the inquisitor's want, silence (reveal; *trade what we know*) · `{magistrate}` (fixed slot).

**16 · C1 cards:** C1-2 Threat *Go and look*.

**17 · Placing.** Centre: `{wayhouse}`. Main town (for *Fresh stories*): the centre's town. Kept in memory too: stake S2, move M8, openings Passage · Smuggling, Restore · Lost livelihood. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{wayhouse}`, about the Wayhouse (its Wall's oldest pigeonholes, burned in its own yard); the giver is there (the heart, at the Wall where his letters wait); its action falls at `{wayhouse}`.
*Swaps* (field 5): R3 may be Investigate · Watch them.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{wayhouse}` | fixed | the Wayhouse's Lending Wall, oldest section | centre |
| `{branch_hall}` | hall/stall | a Wall branch in First Peoples land where the house's word holds; not the Mount | ≤3.5 of Highcrossing (a journey) |
| `{inq_camp}` | site | the inquisitors' camp on the road beyond (added) | 0.5 beyond branch_hall |
| `{tithe_hall}` | hall | a reading room in a town with a tithe-house (presence) | ≤3.0 of Highcrossing |
| `{widow_hall}` | hall | any other hall | ≤3.0 of Highcrossing |
| `{mount}` | fixed | the Mount (the Holy House's seat; its stall) | — |
| `{inquisitor}` | person | a Holy House inquisitor, the Mount's (made on demand under its holder) | — |
| `{hand}` | person | a pious copyist in the inquisitor's pay (made on demand) | — |
| `{magistrate}` | person | Highcrossing's magistrate (made on demand) | — |
| `{ally}` | person | of the dead copyist's house (made on demand; flavour c) | — |
| `{lodging}` | place | where the inquisitor sleeps before a burning (flavour b) | Highcrossing |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{wayhouse}` | — | the Wayhouse: his letters wait in the Wayhouse's Wall |
| R2 | `{widow_hall}` | — | free (≤3.0 of the centre) |
| R3 | `{tithe_hall}` | — | free (≤3.0 of the centre) |
| RV | `{tithe_hall}` | — | free (≤3.0 of the centre) |
| C1-1 | `{branch_hall}` | `{branch_hall}` | journey (≤3.5 of the centre): the road to the branch, where the inquisitor searches |
| C1-2 | `{inq_camp}` | `{branch_hall}` | journey (≤3.5 of the centre): the road beyond the branch |
| C1-3 | `{mount}` | — | free (≤3.0 of the centre) |
| C2-0 | `{branch_hall}` | `{branch_hall}` | journey (≤3.5 of the centre): the branch's town |
| DOOR | a: `{branch_hall}` · b: `{branch_hall}` | `{branch_hall}` | journey (≤3.5 of the centre): the branch's town |
| crisis | `{wayhouse}` | — | the Wayhouse: the burning is in the Wayhouse's own yard, where his letters wait; he comes to it as he comes every month |
| crisis, acted | `{wayhouse}` | — | where its action falls |

*Parties set up first* (checked): the branch's old Wall clerk (C1-1); the ally (C1-1); the Hand (C1-3); the inquisitor (RV).

---

## W5-B · The Last Reader

**1 · Enemy:** W5c, a Holy House inquisitor (High), who has a dig closed as unclean before its roll is read. *Above* · cover-up · `faith` `paper` `men` · region · *the old debt* · side: Holy House.

**2 · Scheme.** `{dig}`, at the ruin of the house where the Mount's grants were drafted, has turned up the drafting roll, in a Writers' hand. The scribes' house stood far from the Mount, among the grants' first holders, so that no priest's hand would be seen in them *(Claude, draft 3)*. He has the dig closed as unclean ("grove-goers' ground") before the roll is read, seizes it to burn, and silences whoever has read it.
**Payload ⚑ (crisis):** the roll burned at the Mount, and its one reader held at the Mount until she recants at `{feast}`, so the house can say it never said what she read.

**3 · Stake:** S1c, the Wayhouse's strongroom. *Reading:* where what the guild's digs find is lodged and kept; whether the Wayhouse ever holds this roll's words.

**4 · Heart:** H1a, `member`, cast: the reading master who recruited you, who still teaches at `{home_hall}`. *Needs:* she was the dig's reader this season; the afternoon before the closing she copied the roll's first page into her day-book, and that night she read the first pages aloud to the diggers.
- **Want:** to read one roll whole, once in her life; this one.
- **Past:** she was raised in the Mount's orphan school, and left it for the halls.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{home_hall}` | Acquire *Copy it before it burns* (signature; *swap* below) | a pupil's family means to burn a grandmother's prayer-book with Ṭaḍoro notes in its margins; her elder pupil `{fallback}` carries it to the hall. **Heart's card**: she wants it copied whole, and says what she wants most: to read one roll whole, once (her want); she was raised in the Mount's orphan school and left it for the halls (her past). The family's word for the book is flavour a's | "**'unclean', said of a book with a Writers' hand in it**" |
| R2 | from `{reader_lord}`'s seat | Passage *Smuggling* | carry home to `{home_hall}`, through the house's land, a crate of the dig's earlier finds that were lent to `{reader_lord}`'s archive to be read aloud to him; `{reader_lord}`, whose archive is read to him, asks to hear the dig's next roll the day it comes out of the ground | ∅ "**a lodging list with one line left blank, 'to be read at the Wayhouse'**" |
| R3 | `{lantern_hall}`, the dig's Lantern chapter house | O1 Acquire *Copy it before it burns* (spine variant: a Lantern captain told to burn a dig's day-book). **Odd job** | the captain of the dig's contract camp, at his chapter house, means to burn the dig's day-book | "**a dig day-book burned as unclean**" |

*Swap:* **R1** may be Dispute *Public offense*: the pupil's family cries the hall before the parish for teaching from the grandmother's prayer-book, whose margins the priest calls "unclean"; the heart stands for the book (her want and past as above; the plant is kept). R3 is the routine's signature either way.

**6 · Odd job:** O1 (thread `answers-to`). *How it serves the scheme:* the Hand, the inquisitor's clerk, told the captain the day-book was unclean and copied it first. It is the one record that the roll came out of the dig, in which trench, under a Writers' reader; burning it lets the house say the dig found nothing, and the copy tells the inquisitor where the roll lies.

**7 · Reveal** (giver: *the heart*; form: *a chain*). At `{home_hall}` (a journey: the sister comes down from the Mount). Restore *Estrangement*: the heart asks the player to help her meet her orphan-school sister, now in the house (flavour c, voiced here), to whom she hasn't spoken since she left it.
- **Shows:** reconciled, the sister tells what she has seen herself: the inquisitor's clerk asking the Mount's archive for the old name of a drafting house, and riders sent to the heart's dig.
- **Adds:** the heart adds what the dig is, from her own season as its reader: the ruin of the house where the Mount's grants were drafted, whose roll she has begun to read (the old debt); the sister adds that the leader is riding to claim the roll for the Wayhouse.
- **Repeats:** "'unclean', said of a book with a Writers' hand", "a dig day-book burned as unclean", "a lodging list with one line left blank".
- **Closes:** each explains the next: the house's word for a book with a Writers' hand ("'unclean', said of a book with a Writers' hand") is the word it will use for the dig; the burned day-book ("a dig day-book burned as unclean") is why nobody will be able to say a roll came out of it; and the blank line ("a lodging list with one line left blank, 'to be read at the Wayhouse'") is the line the roll was to fill.

**8 · Chapter 1.** *Shape:* 3 · the early turn. **Move:** M6 (spine reading, on S1 through the dig's people): the inquisitor's men close `{dig}` as unclean, seize its finds and turn the diggers off. *Uses* `faith`, lever faith. Shuts a place. *Needs:* `{dig}` stands in First Peoples land.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{dig}`, closed (a journey) | Dispute *Shared resource* (the dig's ground, claimed by the halls and by the parish chapel beside it, both under `{dig_lord}`'s old grant) | — | walk the old line with the oldest local, before `{dig_lord}`'s reeve. Plant: "**the oldest local, who calls the ruin 'the scribes' house'**" |
| C1-2 turn | the ruin's trench | Acquire *Take the readings* (C1) | but | the inquisitor's clerk (the Hand, opposing) comes with the house's men to fill the trench, and the player must measure the footings against the old plan before they do, to show where the roll lay; the diggers say she copied the first page by lamplight before she read it to them. Plants: "**the hearth-niche of the scribes' house**"; "**her copy of the first page**" |
| C1-3 | the dig's spoil heaps | Passage *Unwilling passenger* | therefore | the clerk takes the diggers who heard her read for questioning; `{digger}`, who can say the first lines with her (flavour b says why he stayed), must be got away before he is taken, and won't leave the place the roll was read. **Ally asks:** `{digger}` asks the player to keep his name from the house, and for that he will stand at the Mount's gate when it counts; his price: his name kept |

**9 · Chapter 2.** Opening (pressure 2): the parish chapel beside the dig, Investigate *Watch them* (C1): watch the chapel, where the inquisitor's clerk questions the turned-off diggers one by one. Plant: "**a list of the diggers who heard her read**". Then the door. *(Draft 3: was Passage Unwelcome cargo, one of three such openings in the last sample set.)*

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | guild · *leader struck down* · wounded by men on the road (`wounded-men`) | `men` | Passage *Fugitive* (cargo: the wounded leader; pursuer: his men) | she was riding to claim the roll for the Wayhouse |
| b | guild · *outlawed* · a lord's writ in one lordship (`outlawed`) | `paper` | Dispute *Public offense* (the writ's charge, harbouring heresy, before that lordship's court) | `{dig_lord}` (met on C1-1), whose lordship the dig stands in, at the inquisitor's asking closes the halls there, so no Writer may read or copy there (the story's one writ; the crisis then uses `faith` only) |

- **The door carries the crisis's plant:** her sister, at the Mount (reconciled on the reveal), sends word that the house means her to recant at `{feast}`, `{days}` days off: "**the feast-day recanting**".
- **Lesser code-break (kind: destroy a text):** an extra way on the door: burn her copy of the first page (C1-2), so the charge has no "unclean" writing to point to.

**10 · Crisis** (at the Mount's gate). **Last move ⚑:** the roll is burned in the Mount's yard. The heart is taken from `{home_hall}` on the inquisitor's charge of reading and copying unclean writing, and held at `{mount}` until she recants at `{feast}`. The roll is ash, so the charge rests on her copy of the first page: the house holds a reader only on the page itself *(Claude)*. Her pupil follows the riders to the Mount's gate. The enemy's want survives: until she recants, someone living has read it.
- **Held:** the roll is seized with the dig on the move, before the player reaches it, and the house holds a reader on its own authority in its own land; the sister's warning gives the player the feast day, which is the crisis's clock, not a way to stop the taking.
- **Family:** Rescue *Imprisoned unjustly*. Captor: the inquisitor, holding the house's authority. Place: the Mount's cells (fortified). Requester: `{fallback}`, her pupil. The card is at the Mount's gate, where the giver has come; the guild's word reaches it by the leader's letter.
- **Giver:** `{fallback}`, at the Mount's gate. *Reason:* she was taken from his lesson.
- **Choice:** get her out, or let her speak.
- **Ways:** *code-break:* burn what the charge names, her copy of the first page, so there is nothing to hold her on · *ally:* `{digger}` and the diggers on the chapel's list crowd the Mount's gate (making the hold cost the house; his price: his name kept, which the crowd can't keep) · *the witness:* go over the charge with the Hand, who swore to it.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Freed** (the crowd at the gate; the house lets her go quietly) | `{digger}` (named to the house; his parish shuns him) | persuasion | teaches again, and writes down what she remembers / the dig stays closed / the inquisitor says the roll was a heretic's forgery |
| **Captive stays by choice** (she recites the roll before the Mount at the feast) | the heart (banished from First Peoples land; `{home_hall}` loses its master) · the Mount's name | none | reads to pupils in Highcrossing, and has read it whole, aloud, once / `{region}` argues about the grants / the inquisitor is recalled |
| **Burn your copy** (code-break: her page burned; freed for want of proof) | the record (the roll's words now live only in her) · the heart (she promises never to write them) | stealth | free, and silent / the dig is grassed over / the inquisitor has won, and knows it |
| **Freed but harmed** (sprung on the road to the feast) | the halls in `{region}` (searched by the house's riders) · `{fallback}` (seen) | stealth | hides in Highcrossing, a fugitive from the house / the reading rooms of `{region}` are watched / the inquisitor hunts the pupil |
| **Handed to another** (B7: given to `{reader_lord}`'s court, which asked on R2 to hear the dig's next roll and wants its words for itself) | the heart (kept as a lord's reader) · the guild (a house now holds its secret) | bribe | reads aloud for `{reader_lord}` in a closed room / `{region}` hears nothing / the house and the lord bargain over her |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R3 (odd job) | *already copied*: the Hand copied the day-book first | its binding thread is new |
| R2 | *the pursuer was told*: the checkpoint knew the crate's contents | the officer asks for "the dig's crate" by name |
| C1-1 | *record survives*: the oldest local remembers | she calls it "the scribes' house" before anyone asks |
| C1-2 | the roll lay in the drafting room's hearth-niche | soot in a niche of a room with no chimney |
| C1-3 | *not what it seems* (`more-than-said`): the digger has the first lines by heart, and means to keep them | he keeps going back to the hearth-niche, mouthing |
| door a | *not what it seems* (`more-than-said`): the leader carries `{reader_lord}`'s letter asking for the roll | she won't let her saddlebag out of sight |
| door b | *one side lies*: the lord was asked to write it | the writ is dated before any heresy was named |
| crisis | *guilty* (`held-rightly`), twist *unwilling captive*: she did read it aloud, and means to again | she sends back the rope `{fallback}` passed in through her sister, with a request for paper |

**13 · Hall-job hooks**
- *Middle:* in `{region}`, turned-off diggers want letters written to other digs, asking for work. Reads M6; *before* face: "the dig's quiet".
- *Late:* at a hall outside First Peoples land, a family of `{region}` wants its prayer-book copied "where it isn't unclean". Reads the hold, or the outlawing; *before* face: "inquisitors in `{region}`".
- *Seen in passing:* before the reveal, an inquisitor patrol helps a carter right an overturned cart on a side card.

**14 · What varies.** `{dig}`: a dig at a ruin in First Peoples land (the placer creates one where the map has none). `{home_hall}` (within a journey of the Mount); `{feast}` (rolled from the Mount's calendar; `{days}` from her hall to the Mount); `{region}` (the map's word for the hall's region). `{dig_lord}`: the registry's Seated or Tide House head whose lordship holds the dig. `{reader_lord}`: another Seated House head, whose archive is read to him (on map D the Ninth Ring's Tuhuḍo, the blind archivist-lord, fits it best). *Flavour* (each list rolled on its own): (a) the house's word for the ground, first said on R1, which sets the plant's words wherever "unclean" appears: unclean, grove-goers' ground, or green-touched; (b) why `{digger}` stayed to listen, told on C1-3: he can't read and wanted to hear a roll read once, his father dug the ruin before him, or he had nowhere else to sleep; (c) her sister's post in the house, told on the reveal: a priestess, the novice-mistress, or the house's cook. *(Draft 1's Lantern guard and grandson as ally couldn't say the first lines with her or bring the diggers to the gate. Draft 3: no region name in the text, and the Seated House court of the last ending is `{reader_lord}`, set up on R2.)*

**15 · Presuppositions:** the old debt (kit) · `{dig}` at the drafting house, far from the Mount *(Claude)* (slot; reveal; C1-1) · the roll (the move) · the heart read it aloud and copied its first page (her spec; C1-2) · the house holds a reader only on the page *(Claude)* (crisis) · the Lantern captain and the day-book (R3) · `{hand}` = the inquisitor's clerk (R3 hidden; C1-2 open) · the lodging list (R2) · `{reader_lord}` and his archive (R2; door a; last ending) · her sister (reveal; door; crisis) · the oldest local (C1-1) · `{digger}` (C1-3) · the chapel's list (ch2 opening) · `{feast}` (door) · `{dig_lord}` (registry; C1-1; door b) · `{fallback}` (her elder pupil; R1, crisis).

**16 · C1 cards:** C1-2 Acquire *Take the readings*; ch2 opening Investigate *Watch them*.

**17 · Placing.** Centre: `{home_hall}`. Main town (for *Fresh stories*): the centre's town. Lords and courts kept in memory: `{dig_lord}`, `{reader_lord}`. Kept in memory too: stake S1, move M6, openings Dispute · Shared resource, Investigate · Watch them. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{mount}`, where the story is; the giver is there (her pupil, who has followed her to the Mount's gate); its action falls at `{mount}`. *Clock:* `{days}` = the road from `{home_hall}` to `{mount}`, rounded up, + 1 (word of the recanting comes to her hall; the Mount is that far); `{feast}` is that many days off at the last move.
*Swaps* (field 5): R1 may be Dispute · Public offense.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{home_hall}` | hall | a reading room in First Peoples land, where the heart teaches | centre; ≤3.5 of the Mount |
| `{dig}` | site | a Writers' dig at the drafting house's ruin, in First Peoples land (added: none there) | 0.8 from the centre |
| `{lantern_hall}` | town | a Lantern chapter house (the dig's camp's chapter) | ≤1.25 of the dig, else ≤1.6 |
| `{dig_lord}` | seat | the Seated or Tide House head whose lordship holds the dig: the nearest lord's seat in its territory (d-l0) | ≤1.25 of the dig, else ≤1.6 |
| `{reader_lord}` | seat | a Seated House head with an archive, not dig_lord (d-l0); his court wants the roll's words | ≤3.0 of the centre |
| `{mount}` | fixed | the Mount and its cells | — |
| `{feast}` | day | a feast of the Mount's calendar (roller: a feast's name; `{days}` days off when word comes) | — |
| `{hand}` | person | the inquisitor's clerk (made on demand; the Mount's) | — |
| `{fallback}` | person | her elder pupil (cast) | — |
| `{digger}` | person | a digger who heard her read (made on demand) | at the dig |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{home_hall}` | `{home_hall}` | bound |
| R2 | `{reader_lord}` | — | free (≤3.0 of the centre) |
| R3 | `{lantern_hall}` | `{dig}` | bound |
| RV | `{home_hall}` | `{home_hall}` | journey (≤3.5 of the centre): her sister comes down from the Mount |
| C1-1 | `{dig}` | `{dig}` | journey (≤3.5 of the centre): out to the dig |
| C1-2 | `{dig}` | `{dig}` | bound |
| C1-3 | `{dig}` | `{dig}` | bound |
| C2-0 | `{dig}` | `{dig}` | bound |
| DOOR | a: `{dig}` · b: `{dig_lord}` | `{dig}` | bound |
| crisis | `{mount}` | `{mount}` | journey (≤3.5 of the centre): her pupil follows the inquisitor's riders to the Mount's gate |
| crisis, acted | `{mount}` | — | where its action falls |

*Parties set up first* (checked): the reader lord (R2); the dig's lord (C1-1); her sister (RV); the digger (C1-3); her pupil (R1); the leader (RV); the inquisitor (RV); the Hand (C1-2).

---

## W6-A · The Copies Sent for Mending

**1 · Enemy:** W6a, **the leader herself**: every other copy sent "for mending" and lost on the road. *Within* · preservation · `inside` `craft` `name` · realm · *the cost* · side: Writers. `{tutor}`, the old reader at `{reading_room}` who taught the leader, gives the reveal (draft 1 called him the hall's "eldest reader", which read as `{eldest}`); the seat ending *take the road* is offered as a fourth.

**2 · Scheme.** Letter by letter from the road, she has every other copy of `{region}`'s land rolls (the region of `{reading_room}`) sent to the Wayhouse "for mending". Her courier loses each one on the way: not burned, for she keeps the code, but carried to `{tower}`, where no one reads them. When the Wayhouse's roll is the last, whoever holds the Writers decides who owns the land, and she means to keep the balance by it.
**Payload ⚑ (crisis):** with the last outside copy "lost", `{court}` sits on `{valley}`'s claims. Her letter orders the court's copy made without the margin grants, so `{valley}`'s holders lose and no other copy may leave the Wayhouse.

**3 · Stake:** S4, `{reading_room}` (S4a/c), and its master copies. *Reading:* the last copy outside the Wayhouse is this reading room's; when it goes, the halls hold the only roll.

**4 · Heart:** H2a, `member`, cast: a letter-writer's apprentice who copies the old imperial hand by shape. *Needs:* she works at the Wayhouse, copying its land rolls by night, for mending, unread, and keeps every sheet she has made.
- **Want:** to be taught to read the hand she copies.
- **Past:** she came to the halls as a foundling from `{valley}`.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{court}`'s seat | Acquire *Won't sell* | the archivist of `{court}`'s holder won't lend the halls his master's court roll: it is a short roll, kept without the margin grants, as a court's always is *(Claude, draft 3)*, and the last full copy his house sent for mending never came back | ∅ "**a copy sent for mending that never came back**" |
| R2 | the Wayhouse, where she copies by night | Dispute *Broken agreement* (her night-copy pay) | the apprentice against her copy master. **Heart's card**: she wants to read what she copies; she came to the halls a foundling from `{valley}` (her past); she keeps "**the shape-sheets under her bed**", every page she has copied, in case they're asked for again, `{valley}`'s page on top, copied more often than any other; the copies are made by the leader's letter, which `{eldest}` reads out to the night table | "**copies made for mending, of rolls that weren't torn**" |
| R3 | `{reading_room}` | O2 Investigate *The ledger doesn't add up* (signature; spine variant: the mending book; the Hand is the courier who signs it). **Odd job** | the halls' mending book doesn't match a hall's shelves | "**the courier's mark against 'returned', in one ink for a year**" |

**6 · Odd job:** O2 (thread `lie-in-record`; the culprit is the Hand). *How it serves the scheme:* the Hand, the leader's courier, signs every copy back into the mending book as returned to its hall, so no hall asks after a copy sitting unread at `{tower}`. The altered book lets the copies go astray one at a time without anyone counting.

**7 · Reveal** (giver: *an elder copyist*; form: *the thing names them*). `{tutor}`, at `{reading_room}`. Acquire *Passed on*: a pedlar in the town sells a roll-case bearing the hall's mark, bought from a carter who found it fallen from the courier's mule on the road out; inside are a roll and its mending label, addressed to no hall. `{tutor}`, who taught the leader her letters, buys it and reads it with the player.
- **Shows:** the label is in the courier's hand, with the "returned" mark he signs in the mending book, though the roll was never returned; the roll is whole, never torn; the label names no hall to mend it.
- **Adds:** `{tutor}` adds who orders the copies sent, from the hall's file of mending orders, every one in the leader's hand (he taught it to her, and knows it), and her reason (flavour b), from the letter she wrote him, her old teacher, last winter, when he asked why the halls' copies take a season to mend; that is the guild's hidden side. *(Draft 4: a label alone can't show her letters or her reason.)*
- **Repeats:** "a copy sent for mending that never came back", "copies made for mending, of rolls that weren't torn", "the courier's mark against 'returned'".
- **Closes:** the label says the plants in its own marks: "returned", in the courier's one ink ("the courier's mark against 'returned', in one ink for a year"); a roll sent for mending that was never torn ("copies made for mending, of rolls that weren't torn"); and no hall to send it back to ("a copy sent for mending that never came back").

**8 · Chapter 1.** *Shape:* 1 · the price at the turn. **Move:** M3 on S4: by the leader's letter, `{reading_room}`'s master takes the reading room's master copy of the land rolls into his own keeping, "to be sent for mending". *Uses* `inside`, lever theft; *cost* yes.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{tower}` (a journey) | Acquire *Unknown worth* | — | follow the courier's road by the tolls he paid (the mending book signs his road, R3) to a lone tower, whose keeper (flavour a) keeps what the courier leaves without knowing what it is worth. Plant: "**shelves of copies, dry and unread**" |
| C1-2 | `{reading_room}` | Investigate *Watch them* (C1) | therefore | watch the hall's strongroom, where the master copy waits under the leader's letter for the courier. Plant: "**the courier due at the new moon**". Second plant: the hall's master, ashamed of the letter, says he is "**a master who'd sooner copy than send**", and would take a copy into his hall if one came |
| C1-3 turn | the road out of town | Dispute *Public offense* | but | the new moon has come, and the courier (the Hand, opposing) has taken the master copy out under the leader's letter; when the ashamed hall master cries him a thief before the hall, the courier demands redress, the letter in his hand. **Ally plant:** `{claimant}`, `{valley}`'s headwoman, who needs a copy at `{court}` for her claim (flavour c); she says the margin grants name the families who held `{valley}` before the claim, and that a foundling of the valley might be of one of them; her price: the halls copy her claim free |

**9 · Chapter 2.** Opening (pressure 2): the Wayhouse, Acquire *Under seal*: by the leader's letter the Wayhouse's own land rolls are put under her seal "until the mending is done", and no night copyist may touch them. Plant: "**the land rolls under her seal**". Then the door. *(Draft 3: was Restore Lost livelihood, the night copyists laid off, which shared door b's family and echoed W7-A's laid-off copyists.)*

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | player · *framed* · for destroying a text (`framed-damage`) | `inside` | Investigate *Sabotage* (blamed for the "lost" master copy; done by the Hand) | the player was seen on the courier's road, following his tolls to the tower (C1-1); a copy lost on that road needs someone to have lost it |
| b | heart · *disgraced* · cried down for taking a side (`cried-down`) | `inside` | Restore *Lost good name* (cause *exaggerated*: she copied by night, by shape, for no side) | she is found copying the Wayhouse's roll without leave, for the claimants; the leader's letter cries her down |

- **The door carries the crisis's plant:** the heart, who has watched them come and go every night, writes to the player by the Wall's night post (she is at the Wayhouse, the door wherever its blow fell) that the carters who bring the Wayhouse its paper are never searched at its door: "**the paper carters, never searched**".
- **Lesser code-break (kind: take a side in public):** an extra way on the door: sign a private letter vouching for the claimants' copy.

**10 · Crisis** (at the Wayhouse's door: the spine is about the Wayhouse and the leader's hold on it, so the crisis stays there). **Last move:** the master copy is "lost" on the road. `{court}` sits on `{valley}`'s claims in `{days}` days and asks the Writers for the full roll, margins and all. The leader's letter, read out by `{eldest}`, orders the court's copy made without the margin grants and forbids any copy of the land rolls to leave the Wayhouse without her seal; the courier keeps the door. The heart holds a full copy: her shape-sheets of the master copy (R2). ⚑ The payload is attempted here: the court hears only the leader's reading unless the copy gets out.
- **Family:** Passage *Smuggling*. Cargo: the heart's full copy. Route authority, opposing: the leader, by her order. Checkpoint officer: the courier at the Wayhouse door. Requester: the heart.
- **Held:** the player knows the leader is the enemy from the reveal and has seen the tower (C1-1), but no Writer outranks the head of the guild, her order is lawful in the halls, and the tower shows her losses, not that the court's copy is short; so the payload lands unless the copy gets out, and the crisis is that chance.
- **Giver:** `{eldest}`. *Reason:* he must read the order aloud, and hates it.
- **Choice:** obey the head of the guild, or carry the copy out.
- **Ways:** *code-break:* take a side in public: the copyists carry it out openly, before the town · *ally:* `{claimant}` carries it out in her own satchel (her price: her claim copied free) · *hide it inside ordinary goods:* in the paper carters' empty bales.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Delivered** (the full copy reaches `{court}`; the claims are heard on both) | the heart (struck off the roll by the leader's letter) · `{eldest}` (put out for letting her pass) | stealth | taught to read at last, by `{claimant}`'s clerk / `{valley}`'s holders keep their land / the leader writes, and begins again elsewhere |
| **A copy in every hall** (copied into every hall; no one place can hold the only roll again) | the guild's hold on land (with a copy in every hall, no one reading of the rolls can decide a claim again) · the courier (named) | persuasion | copies the rolls for every hall, reading as she goes / the claims are heard fairly / the leader's balance has nothing to stand on |
| **Seized** (the courier takes it at the door) | `{valley}`'s holders (they lose) · the heart (sent to a far hall; not disgraced again) | none | sent to a far hall / `{valley}` goes as the leader's reading says / the leader thanks the courier |
| **Kept by the player** (the player keeps the only other copy) | `{valley}`'s holders · the heart | none | asks the player for it, and is refused / `{valley}` loses / two people now hold the land rolls, and one of them is the player |
| *Seat ending (fourth):* **take the road** (the player takes the leader's road and her seat) | the leader (her seat) · the player (the road, always) | persuasion | the heart is the new head's first reader / the copies come home from `{tower}` / the old head walks on, with no road to claim |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R3 (odd job) | *for the one who profits* (`someone-above`): altered for the leader, by the Hand | the "returned" entries are all in one ink, across a year |
| R1 | twist *rightful keeper*: the archivist is right not to lend | he names the hall that "lost" his father's copy |
| C1-1 | *not what it seems*: the tower holds every lost copy, safe | the tower's keeper buys lamp oil for a library, not for one reader |
| C1-2 | the reading room's master is ashamed of the letter, and wants it watched | he leaves the strongroom's ledger open at the page |
| C1-3 | *one side lies*: the copy goes to `{tower}`, not the Wayhouse | the courier's road runs away from Highcrossing |
| door a | the Hand "lost" it | the roll-case is found dry in a ditch it rained on all week |
| door b | cause *exaggerated* | her copy has no margin of her own, only the shapes |
| crisis | *not what it seems* (`more-than-said`): the copy carries her own family's line in `{valley}`'s margin grants | she asks to carry it herself, though she can't read it |

**13 · Hall-job hooks**
- *Middle:* at `{reading_room}`'s town, a holder wants his title checked against the hall's own copy, and is told it has gone for mending. Reads M3; *before* face: "the hall's copies are away being mended".
- *Late:* at another hall, a lord's clerk asks why the halls' copies take a season to mend. Reads the loss; *before* face: "copies are slow coming back".
- *Seen in passing:* before the reveal, the leader turns up on a side card and pays a ferryman's toll for a stranger.

**14 · What varies.** `{reading_room}` and its region. `{court}`: a registry holder within reach whose writ binds `{valley}`; his archivist is his (R1). `{valley}`: in his lordship. `{tower}`: a lone, dry building on remote ground, with `{keeper}`, in the hall's own territory or Great House land, never an enclave (*draft 4*: map D's first roll put it in the Last Wave's land). `{region}`: the map's word for the hall's region. *Flavour* (each list rolled on its own): (a) who keeps the hoard, met on C1-1: a lone scholar, a pensioned copyist, or a deaf caretaker (the tell is his lamp oil either way); (b) the leader's reason, from her letter to `{tutor}`, on the reveal: one lord rising too fast; a Sharp Hands buyer waiting (the counting-house traders who buy up claims), who would buy the valley from whoever the court favoured; or a house about to swallow its neighbours; (c) what the headwoman's claim is for, told on C1-3: the spring meadow, the upper fields, or the mill. *(Draft 1's "Ṭaḍoro friend's attic" was a friend nobody defined, and an abandoned reading room has no keeper to buy oil; its claimants' clerk and under-clerk as ally broke "taught to read by `{claimant}`'s clerk". Gone. Draft 1's archivist sat in a lordless port; he is now the court's.)*

**15 · Presuppositions:** the leader's letters from the road (canon: she is never at the Wayhouse) · `{court}` (registry) and his archivist (R1) · `{tutor}` (reveal) · `{keeper}` (C1-1) · `{hand}` = her courier (R3 hidden; C1-3 open) · the mending book (R3) · `{tower}` (C1-1) · `{reading_room}`'s master copy (stake; move) · `{valley}`, its claim and `{court}` (C1-3; crisis) · the margin grants, which courts' short rolls leave out *(Claude)* (R1; crisis; on C1-3's claim) · the land rolls under her seal (ch2 opening) · the paper carters (door) · the heart's foundling past (her spec; R2) · her shape-sheets (R2) and a master who would take a copy (C1-2) (*a copy in every hall*) · `{eldest}`.

**16 · C1 cards:** C1-2 Investigate *Watch them*. *(Draft 3: chapter 1 had two Acquire cards; C1-1 is now Acquire Unknown worth and C1-3 Dispute Public offense.)*

**17 · Placing.** Centre: `{reading_room}`. Main town (for *Fresh stories*): the centre's town. Lords and courts kept in memory: `{court}`. Kept in memory too: stake S4, move M3, openings Acquire · Unknown worth, Acquire · Under seal. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{wayhouse}`, about the Wayhouse and the leader (her order keeps the copy in, at the Wayhouse's door); the giver is there (the eldest copyist, who must read the order aloud); its action falls at `{court}`. *Clock:* `{days}` = the road from `{wayhouse}` to `{court}`, rounded up, + 1 (the copy must ride from the Wayhouse to the court before it sits).

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{reading_room}` | hall | a reading room in a large town keeping its region's land rolls (S4) | centre |
| `{region}` | region | the region of reading_room (roller: the map's region name) | — |
| `{court}` | seat | a court holder whose writ binds the valley: Banner/Free Lord, Seated or Tide House head (d-l0); his archivist is his | ≤1.25 of the centre, else ≤1.6 (nearest) |
| `{valley}` | site | the valley of the claims, in the court's territory (added) | 0.6 from the court's seat |
| `{tower}` | site | the hoard: a lone dry building on remote ground, with a keeper (added), in the centre's territory or Great House land, never an enclave | 1.8 from the centre |
| `{hand}` | person | the leader's courier (made on demand) | — |
| `{keeper}` | person | the hoard's keeper, in the leader's pay (made on demand; see flavour a) | at the tower |
| `{tutor}` | person | the old reader at reading_room who taught the leader (made on demand) | — |
| `{claimant}` | person | the valley's headwoman (made on demand) | — |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{court}` | `{court}` | bound |
| R2 | `{wayhouse}` | — | the Wayhouse: she copies the Wayhouse's land rolls by night, for mending |
| R3 | `{reading_room}` | `{reading_room}` | bound |
| RV | `{reading_room}` | `{reading_room}` | bound |
| C1-1 | `{tower}` | — | journey (≤3.5 of the centre): out to the tower, by the tolls the courier paid |
| C1-2 | `{reading_room}` | `{reading_room}` | bound |
| C1-3 | `{reading_room}` | `{reading_room}` | bound |
| C2-0 | `{wayhouse}` | — | the Wayhouse: the land rolls and their night copyists are the Wayhouse's |
| DOOR | a: `{reading_room}` · b: `{wayhouse}` | — | bound |
| crisis | `{wayhouse}` | — | the Wayhouse: the copy must leave the Wayhouse past her courier; the heart works there, and the eldest reads the order there |
| crisis, acted | `{court}` | — | where its action falls |

*Parties set up first* (checked): the headwoman (C1-3); the tutor (RV); the courier (the Hand) (C1-3); the eldest copyist (CR); the hall master (C1-2); the leader (RV); the archivist (R1).

---

## W6-B · The Archive Nobody Copied

**1 · Enemy:** W6b, **the leader herself**: a lord's archive left to burn uncopied. *Within* · preservation · `inside` `craft` `name` · realm · *the cost* · side: Writers. `{tutor}`, `{reading_room}`'s old reader, gives the reveal; the seat ending *take the road* is offered as a fourth.

**2 · Scheme.** `{lord}`'s steward asked the halls three times to copy his master's archive before the old lord died. It stands in a lamp-lit tower, and two heirs are waiting to fight. By her letter the commission was "held" each time. When the archive goes, the halls' roll of the lordship will be its only record, and the halls will decide its succession and its tenancies.
**Payload ⚑ (last move):** the archive burns uncopied the night the old lord dies, and the heirs must come to the halls.

**3 · Stake:** S7 (c: a Banner Lord near Highcrossing; or a/b). *Reading:* the halls' keeping of `{lord}`'s rolls, and the commission to copy his archive, which was asked for and held.

**4 · Heart:** H4c, `member`, cast: the old clerk who knows `{reading_room}`'s copying book: who copied what, and when. *Needs:* she logged the steward's three requests, and the leader's letters holding them.
- **Want:** to copy `{lord}`'s archive herself.
- **Past:** she wrote to the leader three times asking leave to copy it, and was told to wait.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{reading_room}`, in a town near `{lord}`'s seat | Investigate *The ledger doesn't add up* (signature; *swap* below) | a will was altered (*for the one who profits*); the heart is the old clerk who knows who copied what. **Heart's card**: she would give anything to copy an archive whole; she wrote to the leader three times asking leave to copy this one, and was told to wait (her past) | "**a commission entered 'held, by letter', three seasons running**" |
| R2 | `{lord}`'s seat | O1 Acquire *Copy it before it burns* (spine variant: a steward's clerk told to burn his master's letter-book). **Odd job** | the steward's clerk at the seat means to burn the steward's letter-book | "**a steward's letter-book, three letters to the halls in a year**" |
| R3 | `{r3_hall}`, another hall within three days | Dispute *Contested inheritance* | two heirs, and their father's will burned in a house fire | ∅ "**a will the halls never got round to copying**" |

*Swap:* **R1** may be Dispute *Broken agreement*: a family against the hall over a copying contract paid three seasons ago and never done; the heart, who keeps the copying book, opens it to settle them, and on the facing page is "a commission entered 'held, by letter', three seasons running" (her want and past as above; the plant is kept). R2 is the routine's signature either way.

**6 · Odd job:** O1 (thread `answers-to`). *How it serves the scheme:* the Hand, the leader's courier, who carried each "held" answer, told the steward's clerk that begging letters shame his master and the halls keep copies anyway. It is the one record outside the halls that the copying was asked for three times; burned, it lets the halls say it was never pressed.

**7 · Reveal** (giver: *an elder copyist*; form: *laid side by side*). `{tutor}`, at `{reading_room}`. Restore *Lost good name*: `{lord}`'s steward, who rides in from the seat, is laughed at as the man who "begs the halls" (the harm is off the card); he is the knower.
- **Shows:** the steward's own copies of his three letters asking the halls to copy his master's archive, and `{reading_room}`'s copying book, where each is entered "held, by letter" a week after it came.
- **Adds:** `{tutor}` adds whose letter held them, from the hall's letter file (the leader's hand, which he taught her), and why (flavour b), from what she wrote him when he asked: she would choose `{lord}`'s heir by the halls' roll (the balance).
- **Repeats:** "held, by letter, three seasons running", "three letters to the halls in a year", "a will the halls never got round to copying".
- **Closes:** `{tutor}` lays each of the steward's letters beside its line in the copying book ("held, by letter, three seasons running"), the steward's letter-book count beside them ("three letters to the halls in a year"), and the burned will of R3 beside both: "a will the halls never got round to copying", which is what an archive left uncopied comes to.

**8 · Chapter 1.** *Shape:* 3 · the early turn. **Move:** M9 on S7: by the leader's letter, the hall master dismisses before the hall the copyist who asked who has been holding `{lord}`'s commission. *Uses* `inside`, lever blackmail; *cost* yes.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{lord}`'s seat, the archive tower (a journey) | Restore *Broken works* (cause *neglected*) | — | the steward wants the tower made safe before winter; the heirs won't pay. Plant: "**the archive's lamp-lit stair, oil in every niche**" |
| C1-2 turn | the seat's hall | Acquire *Won't sell* | but | the move reaches the seat: the courier (the Hand, opposing) holds the commission's seal and won't release it without the leader's word, so nobody may copy the archive |
| C1-3 | the seat, by night | Investigate *Watch them* (C1) | therefore | with no seal, the steward watches the archive door himself. Plant: "**two heirs' men at one door**". **Ally asks:** the steward, who begged for the copying, asks the player to watch with him, and to say in public, when it comes to it, that he asked; that is his price |

**9 · Chapter 2.** Opening (pressure 2): the seat, Passage *Unwilling passenger*: the old lord's ward is to be moved out before the heirs come to blows, and won't go. Then the door.

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | guild · *hall seized* · shut by her own order (`seized-order`) | `inside` | Acquire *Under seal* (`{reading_room}`'s copying book, sealed by the leader's letter "while the succession is argued") | the copying book is the one record in the halls that the commission was asked for; sealed, it can't be laid beside the steward's letters |
| b | player · *framed* · for taking a side (`framed-shame`) | `inside` | Reckoning *Ruin from above* (framer: the leader; shield *Rank*: no one in the guild outranks her) | the player carried the steward's letters |
| c | player · *stripped of rank* (`stripped`) | `inside` | Restore *Lost good name* (the stain: "meddles in a lord's succession") | the same end, by her letter alone |

- **The door carries the crisis's plant:** the copying book's page for that season (in the hall's book, or, if blow a sealed it, in her own working book) is in the heart's own hand, with the leader's "held" letters pinned to it: "**the page in her own hand**".
- **Lesser code-break (kind: take a side in public):** an extra way on the door: tell one heir, privately, what the halls' roll says.

**10 · Crisis.** **Last move ⚑:** the old lord dies; both heirs' men fight at the archive door; a lamp goes over on the oiled stair; the archive burns uncopied. The heirs come to the halls for the only roll. A letter of the leader's that had waited, sealed, in `{reading_room}`'s strongroom "to be opened if the archive is lost", and is "**dated the week before the fire**", names the heart as the clerk who "lost" the steward's requests, so the halls' delay becomes a clerk's fault; it is read before the whole hall, and she is to be sent from `{reading_room}` to a far hall. The enemy's want survives: the halls hold the only roll. *(Draft 3: the letter is read at the hall where the heart works, not at the Wayhouse, so nobody rides up unshown, and the face-to-face way can bring the leader, who never sets foot in the Wayhouse.)* *(Draft 2: this crisis is the heart's one public shaming; the spine has no `cried-down` blow, so it never falls twice.)*
- **Family:** Reckoning *Betrayal*. Wronged: the heart. Wrongdoer: the leader (link: employer). Demand: confession.
- **Held:** without the commission's seal (C1-2) no Writer may copy a lord's archive, and the old lord's death and his heirs' men at one door (C1-3) are nobody's to stop; so the fire lands, and the crisis is about who is blamed for it.
- **Standing:** if blow c stripped the player, the player stands before the hall as the steward's witness (the player carried his letters, and he asks for the player), not as a Writer of the hall; `{tutor}` reads the letter to the hall either way.
- **Giver:** `{tutor}`, the hall's old reader who taught the leader her letters. *Reason:* he must read the letter aloud, before the hall.
- **Choice:** the heart's name or the leader's, and with it who reads the only roll.
- **Ways:** *code-break:* take a side in public: the halls back the heir the archive would have favoured · *ally:* the steward swears he wrote three times and was answered "held, by letter" (his price: said in public) · *face to face:* bring the betrayer to the betrayed, which means the leader coming off the road to `{reading_room}` to meet her.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Amends made** (face to face, the leader takes back her letter and lets the heart copy what's left) | the leader (her word, before the hall masters) · the steward (named as having begged) | persuasion | copies the archive's charred remains, which is her want, smaller / the succession is argued from scraps / the leader walks on, watched |
| **Open the ledgers** (the copying book read at every hall) | the guild (the hall masters split over her) · the leader | law | is famous, and not forgiven by half the halls / the heirs go to law / the leader keeps her road and loses her letters' weight |
| **Keep the balance** (the heart goes to the far hall; the roll is read as the leader reads it) | the heart · the heir who loses | persuasion | copies at a far table / the lordship goes to the leader's choice / the leader writes the heart kindly |
| **Take a side at last** (code-break: the halls back the steward's heir in public) | the guild's neutrality · the other heir's tenants | law | stays, and copies for the new lord / the halls choose a lord / the leader's balance broken by her own hall |
| **Sides with the wrongdoer** (the player argues the leader's balance) | the heart | persuasion | leaves without a word to the player / the roll is read her way / the leader remembers who argued for her |
| *Seat ending (fourth):* **take the road** | the leader (her seat) · the player (the road, always) | persuasion | the new head's first copyist / the halls copy every archive that asks / the old head is a guest at the halls she ran |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *as told* (confirming); the hidden role is the Hand's link to the leader, a secret fact | the clerk calls the courier "the halls' gentleman" |
| R3 | *no record* (`no-record`) | both heirs quote the will, differently |
| C1-1 | cause *neglected* | the tower's lamp-oil bill has been paid every month, the repairs never |
| C1-2 | *promised elsewhere*: the seal is promised back to the leader | the seal is wrapped for a journey |
| C1-3 | the heirs' men mean to seize the archive keys the night the old lord dies | each side has a locksmith with it |
| door a | *a copy* (`a-copy`): the sealed book is the hall's fair copy; her working book, with the "held" letters pinned in it, is in her own desk | the sealed book's spine is new |
| door b | *a witness exists*: the courier saw who carried what | the courier won't ride to the seat again |
| door c | cause *false, spread off the card* | it reached the far halls before it reached `{reading_room}` |
| crisis | *a witness exists* (`unheard-witness`): the courier carried every "held" letter, and kept the wrappers | the courier's saddlebag is stuffed with old wrappers he never throws away |

**13 · Hall-job hooks**
- *Middle:* at `{reading_room}`'s town, a family wants its deeds copied this season, "before anything happens to them". Reads M9; *before* face: "a copyist turned off".
- *Late:* at another hall, tenants of `{lord}`'s lordship want their leases copied from the halls' roll, now that the archive is gone. Reads the fire; *before* face: "the old lord is dying".
- *Seen in passing:* before the reveal, the leader turns up on a side card and settles a ferry quarrel with a joke.

**14 · What varies.** `{lord}`: an old registry holder, never a faction leader, with an archive at his seat; his two heirs are made as his kin. `{reading_room}`: the hall that keeps his rolls, in another town within reach (draft 1 put seat, archive and hall in one town, which left C1-1's journey nowhere to go). *Flavour* (each list rolled on its own): (a) the heirs, met on C1-3: two sons, a son and a niece, or twin daughters; (b) the leader's reason, from what she wrote `{tutor}`, on the reveal: the stronger heir would swallow his neighbours, or the weaker heir would sell to the Sharp Hands, the counting-house traders who buy up land; (c) why the steward begged, told on C1-3: he loves the old lord, he fears the stronger heir, or he was a halls' copyist once. *(Draft 1's roof fall and damp contradicted the oiled stair on C1-1 and the lamp in the crisis; its ward and archive-keeper as ally hadn't written the three letters the ally swears to.)*

**15 · Presuppositions:** `{lord}` (registry), his seat, his archive and two heirs (slots; C1-1, C1-2) · `{tutor}` (reveal) · the steward's three requests (R1, R2, reveal) · the leader's "held" letters (R1) · `{hand}` = her courier (R2 hidden; C1-2 open) · the oiled stair (C1-1) · the heirs (C1-3) · the copying book page (door) · the far hall, and the letter dated before the fire, sealed in the hall's strongroom (the last move) · `{tutor}` reads it (crisis).

**16 · C1 cards:** C1-2 Investigate *Watch them*.

**17 · Placing.** Centre: `{lord}`. Main town (for *Fresh stories*): the centre's town. Lords and courts kept in memory: `{lord}`. Kept in memory too: stake S7, move M9, openings Restore · Broken works, Passage · Unwilling passenger. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{reading_room}`, where the story is; the giver is there (the old reader who taught the leader, who reads her letter before the hall); its action falls at `{reading_room}`.
*Swaps* (field 5): R1 may be Dispute · Broken agreement.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{lord}` | seat | an old High holder (d-l0 age 'old'; Banner/Free Lord or Seated House head) with an archive at his seat; never a faction leader | centre |
| `{reading_room}` | hall | the hall that keeps his rolls: a reading room in another town | ≤1.25 of the seat, else ≤1.6 (nearest) |
| `{r3_hall}` | hall | any other hall | ≤3.0 of the centre |
| `{heirs}` | people | his two heirs (made on demand as his kin) | at the seat |
| `{hand}` | person | the leader's courier (made on demand) | — |
| `{steward}` | person | his steward (made on demand; his) | at the seat |
| `{tutor}` | person | reading_room's old reader (made on demand) | — |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{reading_room}` | `{reading_room}` | bound |
| R2 | `{lord}` | `{lord}` | bound |
| R3 | `{r3_hall}` | — | free (≤3.0 of the centre) |
| RV | `{reading_room}` | `{reading_room}`, `{lord}` | bound |
| C1-1 | `{lord}` | `{lord}` | journey (≤3.5 of the centre): out to the seat |
| C1-2 | `{lord}` | `{lord}` | bound |
| C1-3 | `{lord}` | `{lord}` | bound |
| C2-0 | `{lord}` | `{lord}` | bound |
| DOOR | a: `{reading_room}` · b: `{reading_room}` · c: `{reading_room}` | `{reading_room}` | bound |
| crisis | `{reading_room}` | `{reading_room}` | bound |
| crisis, acted | `{reading_room}` | — | where its action falls |

*Parties set up first* (checked): the steward (RV); the heirs (C1-3); the tutor (RV); the courier (the Hand) (C1-2); the leader (RV).

---
## W7-A · The Court's New Clerk

**1 · Enemy:** W7b, a copyist struck off the Wayhouse's roll (Middle), now copying for a lord's court under another name; grudge link to `{master}`, the hall master who struck her off. *Beside* · revenge · `craft` `name` `inside` · local · *the old debt* · side: none.

**2 · Scheme.** As `{lord}`'s court clerk, under another name, she checks the halls' copies of the court rolls when they come in, and enters false lines into them in a hand no hall can tell from its own, so that no Writers' copy will stand again. She begins with `{master}`'s hall, `{reading_room}`.
**Payload ⚑ (crisis):** `{lord}`'s commissioner lays the halls' copy beside the court's original at `{lord}`'s court and finds it false. The court refuses every Writers' copy and ends the commission.

**3 · Stake:** S7 (a/b: the court sits in `{reading_room}`'s town). *Reading:* the commission rides on whether a halls' copy matches the court's original line for line.

**4 · Heart:** H4 (this spine's spec), `member`, cast: the enemy's oldest friend from the Wayhouse's copying tables, who still copies `{lord}`'s rolls at `{reading_room}`.
- **Want:** to see her friend's name back on the Wayhouse's roll.
- **Past:** twenty years ago she sat silent at the hearing that struck her friend off.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{reading_room}` | Dispute *Contested inheritance* | two heirs and one copied will; the heart is the confidant. **Heart's card**: "she never made a false line in her life"; the want; twenty years ago the heart sat silent at the hearing that struck her friend off (the past); her daughter `{kin}` copies at the same table | "**a copyist struck off twenty years ago for one false line**" |
| R2 | `{lord}`'s court, its counting room, where the court's day-book is kept | O2 Investigate *The ledger doesn't add up* (signature; spine variant: a court's copying day-book; the Hand is its under-clerk). **Odd job** | a court's day-book disagrees with the halls' delivery book | "**copies entered 'late, unsealed'**" |
| R3 | `{r3_hall}`'s town, where a bookbinder works | Acquire *Passed on* | a bookbinder there bought in good faith a leaf of the Wayhouse's old copyists' roll, sold off as the Wayhouse's scrap *(draft 4: was in Highcrossing, one of five stories through it)* | ∅ "**a leaf with one name cut out**" |

**6 · Odd job:** O2 (thread `lie-in-record`; the culprit is the Hand). *How it serves the scheme:* the Hand, the court's under-clerk, enters the halls' copies in the day-book as "late, unsealed". That makes the court check every copy line by line against the original, alone with her, which is when she sets her false lines in.

**7 · Reveal** (giver: *the ally*; form: *the knower's question*). At `{son_town}`, where the master's son lives (his trade is flavour c). Restore *Estrangement*: `{master}`'s son, the spine's ally, met here first, asks the player to carry a letter to his father, to whom he hasn't spoken since he was a boy.
- **Shows:** the son tells why: on the night of the false line twenty years ago he saw his father sign out the copying book and carry it home, and the line she was struck off for was in flavour b's kind of entry; so the line was the master's (the old wrong, as he saw it).
- **Adds:** the son adds, from his trade (flavour c takes him past the court every day), that the court's new clerk, who copies left-handed, is the copyist his father struck off, under a name not hers (flavour a); her false lines follow from the court's "late, unsealed" copies, which the player has seen.
- **Repeats:** "struck off for one false line", "copies entered 'late, unsealed'", "a leaf with one name cut out".
- **Closes:** the son asks the player three things: what a copyist "struck off twenty years ago for one false line" does with her hand now, why a court would enter the halls' copies "late, unsealed", and whose name was cut from the Wayhouse's leaf ("a leaf with one name cut out").

**8 · Chapter 1.** *Shape:* 3 · the early turn. **Move:** M5 on S7: in her court name, she has it cried at `{lord}`'s court that the Writers' copies of its rolls are false, and the commissioner orders every copy checked. *Uses* `name`, lever rumour.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{tenant_village}`, a village of the court's tenants (a journey) | Dispute *Shared resource* | — | two tenants quarrel over a field the court's checked copy of the rolls now gives to one of them, by a line the halls' copy never had; the other is `{client}`. Plant: "**a field moved by one line**" |
| C1-2 turn | the court's gate | Passage *Smuggling* | but | the move bites: the commissioner orders every copy checked and lets no Writer into the check room, so the player must bring into the court, past the under-clerk (the Hand, as checkpoint officer, opposing), a fresh copy made at `{reading_room}` from the hall's own master roll, which never went through the check room, to lay beside the sealed copies and show which lines changed in the court's keeping |
| C1-3 | the court's check room | Investigate *Watch them* (C1) | therefore | the fresh copy shows lines changed in the court's keeping, so the player watches the check room after the bell, where the checked copies are kept. Plants: "**a lamp in the check room after the bell**"; "**a court clerk who copies left-handed**" (as the heart's friend did). **Ally asks:** `{master}`'s son (met on the reveal), who wants his father's wrong owned, asks the player to keep his father from a public striking-off; that is his price |

**9 · Chapter 2.** Opening (pressure 2): `{tenant_village}`, Acquire *Unknown worth*: `{client}` holds the halls' first copy of his lease, made before any check, not knowing it is the one copy of his entry the court clerk never touched: "**a lease copy made before the check**". Then the door. *(Draft 3: was Restore Lost livelihood, the hall's court copyists laid off, the twin of W6-A's laid-off copyists; and the record house, W3-B's twin, is gone: the check room is in the court itself.)*

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | player · *stripped of rank* (`stripped`) | `name` | Restore *Lost good name* (the stain: "carries copies into a court by night") | the player smuggled the fresh copy in on C1-2; her court name tells it at the halls, and a Writer who smuggles copies can't be believed about false ones |
| b | heart · *estranged* · her kin turned (`kin-turned`) | `name` | Restore *Estrangement* (cause `secret-told`: the clerk tells `{kin}` her mother sat silent while her friend was struck off) | the heart must stand alone at the hearing, as her friend once did *(draft 3: was "player framed", a blow W3-B also deals)* |

- **The door carries the crisis's plant:** the heart finds on `{reading_room}`'s shelves the hall's copy of that year's Wayhouse copying book (every hall keeps the year's copy *(Claude)*), which shows the master's book signed out that night in his own hand: "**the night the master signed out the book**".
- **Lesser code-break (kind: destroy a text):** an extra way on the door: scrape one of her false lines off a halls' copy.

**10 · Crisis** (at `{lord}`'s court, in `{reading_room}`'s own town). **Last move ⚑:** at `{lord}`'s court the commissioner lays the halls' copy beside the original, and three false lines are found. The court clerk, in her own name at last, accuses the heart, whose copy it is. The commission rides on the hearing. The enemy's want survives: a public hearing is what she wanted. *(Draft 2: this accusation is the heart's one public shaming; the spine has no `cried-down` blow, so it never falls twice. Draft 1 set the comparing at the Wayhouse, days from the court; it is at the court now.)*
- **Family:** Dispute *Public offense*. Parties: the heart and the enemy; offended: the court. Arbiter: the commissioner. The player is the heart's advocate.
- **Held:** the player knows the court's clerk is the struck-off copyist (the reveal) and holds the fresh copy (C1-2), but the commissioner's comparing is lawful and public, and the halls can't accuse the court's own clerk without the old wrong, which the master won't tell; the crisis reads the fresh copy: with it, the hearing must hear which lines changed in the court's keeping.
- **Standing:** if blow a stripped the player, `{client}` names the player his advocate, as any party before a court may, so the player speaks for him and the heart, not for the hall.
- **Giver:** `{client}` (met on C1-1), whose lease and field rest on that copy, who comes to `{reading_room}` to ask the hall for an advocate; the court sits in the same town. *Reason:* his lease.
- **Choice:** save the halls' name by sinking her friend, or tell the old wrong and risk the commission.
- **Ways:** *code-break:* destroy the disputed copy, so nothing can be proved · *ally:* `{master}`'s son tells the night he saw (his price: his father spared in public, which this may not allow) · *deal:* make mercy cost less: her name restored for her false lines named.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **The heart wins** (the lines shown to be the clerk's; the court takes her) | the enemy (held for false copying) · the heart (her friend held on her word) | law | visits her friend in the court's cells / the commission kept, and `{client}`'s field set back / she copies nothing, ever again |
| **A deal is struck** (the Wayhouse restores her name and owns the master's line; she names every false line) | `{master}` (struck off at last) · the guild's name (it owns a wrong) | persuasion | copies beside her friend again, her want met / the court keeps the halls, warily / her name on the roll, and no hall will seat her |
| **Burn your copy** (code-break: the copy destroyed; nothing proved) | the halls (the court refuses Writers' copies) · `{client}` (his lease rests on nothing) | stealth | keeps her post and loses her friend's trust / the court copies its own rolls / she has won, and the old wrong is still untold |
| **The enemy wins** (the court refuses all Writers' copies) | the guild (the commission is lost) · the heart (dismissed by `{master}`) | none | dismissed, as her friend was / the court's rolls leave the halls / she tells the old wrong anyway, to nobody who listens |
| **Take a side at last** (the Writers stand with the struck-off copyist against their own master, and let her three false lines go unpunished) | `{master}` · the commission (the court won't keep a guild that owns a false line) · `{client}` (his field stays moved, by a line nobody will answer for) | persuasion | stands beside her friend before the whole town / the halls lose the rolls and keep their honesty / she is a Writer again, with nothing left to revenge, and three false lines nobody made her answer for |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *for the one who profits* (`someone-above`): altered for the clerk, by the Hand | the copies were delivered sealed; the halls' wax bills say so |
| R3 | *a copy*: the bookbinder's leaf is a copy; the original was burned at the striking-off | the leaf's knife-cut is drawn in ink |
| C1-1 | *both partly right*: the checked copy's line is false, and the tenant it favours believes it | the moved line is in fresher ink than the rest of the page |
| C1-2 | twist *pursuer has a point*: a fresh copy brought in by night looks like forgery | the under-clerk asks, fairly, why it couldn't come by day |
| C1-3 | the under-clerk lets the court clerk in alone | two cups on the check room's sill in the morning |
| C2-0 | *as told*: the lease copy really is untouched | its seal is the halls' old black wax, unbroken |
| door a | cause *false, spread off the card* (`rumour-planted`) | the story is told in the court clerk's own phrases, in a town the player hasn't been to |
| door b | cause `secret-told` | `{kin}` quotes the old hearing's record, which only a court clerk could have shown her |
| crisis | *both partly right* (`both-partly-right`): the lines are hers, and the halls struck her off for one that was the master's | she names the night and the book, and the master's face goes still |

**13 · Hall-job hooks**
- *Middle:* at `{reading_room}`'s town, a merchant wants a contract copied "and checked twice, the way the court does now". Reads M5; *before* face: "talk at the court about the halls' copies".
- *Late:* at another hall, a tenant of `{lord}`'s wants his lease copied there, not at `{reading_room}`. Reads the hearing; *before* face: "the commissioner is coming".
- *Seen in passing:* before the reveal, a lone scholar copies a letter cheaply for a carter on a side card.

**14 · What varies.** `{lord}`: a registry court holder whose seat is `{reading_room}`'s town (on map D the Chainmarket's court is the Binders' head's, held by Paqax, and is named by his title, never "the lord's"; Southroad's is a toll lord's). `{tenant_village}`: a village of the court's own tenants within a day (added where none). `{master}`, `{client}`, `{kin}`. *Flavour* (each list rolled on its own; all published on the reveal): (a) her court name: a Ṭaḍoro road-name, a dead aunt's, or none, just "the clerk"; (b) the master's old false line: a debt, a boundary or a will; it was his either way; (c) what the master's son does now: a ferryman, a carter, or a lamp-lighter. *(Draft 1's old servant and door-warden as ally hadn't seen the night the son saw; the ally is always the son.)*

**15 · Presuppositions:** the old striking-off (kit; R1) · the heart's silence at it (her past, R1) · `{kin}`, her daughter (R1; door b) · `{lord}` (registry) · `{master}` and his hall (slot; reveal) · the enemy's court post (W7b; reveal) · `{hand}` = the under-clerk (R2 hidden; C1-2 open) · the copyists' roll leaf (R3, in `{r3_hall}`'s town) · `{master}`'s son (reveal; C1-3) · every hall keeps the year's copy of the Wayhouse copying book *(Claude)* (door) · the hall's master roll, never checked (C1-3) · the commissioner (C1-2; crisis) · `{client}` and his field (C1-1; ch2 opening; crisis).

**16 · C1 cards:** C1-2 Investigate *Watch them*.

**17 · Placing.** Centre: `{reading_room}`. Main town (for *Fresh stories*): the centre's town. Lords and courts kept in memory: `{lord}`. Kept in memory too: stake S7, move M5, openings Dispute · Shared resource, Acquire · Unknown worth. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{lord}`, where the story is; the giver is there (the tenant whose lease rests on the copy, come to the hall for an advocate); its action falls at `{lord}`.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{lord}` | seat | a court holder whose seat is a large town with a reading room (S7 a/b/c): Banner/Free Lord, Seated or Tide House head, the Binders' head, Open Council head (d-l0) | = reading_room's town |
| `{reading_room}` | hall | that town's reading room ({master}'s hall) | centre |
| `{tenant_village}` | village | a village of the court's own tenants, in its lordship (added if none) | ≤1.25 of the centre |
| `{son_town}` | town | where the master's estranged son lives | ≤1.25 of the centre |
| `{r3_hall}` | hall | another hall, whose bookbinder bought the Wayhouse's scrap | ≤3.0 of the centre |
| `{hand}` | person | the court's under-clerk (made on demand; the court's) | — |
| `{master}` | person | reading_room's hall master (made on demand; the hall's) | — |
| `{kin}` | person | the heart's daughter, a copyist at the same hall (cast with the heart's household) | — |
| `{client}` | person | a tenant of the court whose lease rests on the copy (made on demand) | in tenant_village |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{reading_room}` | `{reading_room}` | bound |
| R2 | `{lord}` | `{lord}` | bound |
| R3 | `{r3_hall}` | — | free (≤3.0 of the centre) |
| RV | `{son_town}` | `{reading_room}` | bound |
| C1-1 | `{tenant_village}` | `{lord}` | journey (≤3.5 of the centre): out to the tenants' village |
| C1-2 | `{lord}` | `{lord}` | bound |
| C1-3 | `{lord}` | `{lord}` | bound |
| C2-0 | `{tenant_village}` | `{lord}` | bound |
| DOOR | a: `{reading_room}` · b: `{reading_room}` | `{reading_room}` | bound |
| crisis | `{lord}` | `{lord}` | bound |
| crisis, acted | `{lord}` | — | where its action falls |

*Parties set up first* (checked): the tenant (client) (C1-1); the heart's daughter (R1); the master (RV); the master's son (RV); the commissioner (C1-2); the clerk (RV).

---

## W7-B · The Apprentice's Hand

**1 · Enemy:** W7c, a copyist struck off the Wayhouse's roll (Middle), who has taught a letter-writer's apprentice the halls' hand. *Beside* · revenge · `craft` `name` `inside` · local · *the old debt* · side: none.

**2 · Scheme.** From `{tower}` she sends the apprentice exemplar sheets to copy from, with false lines set in. The apprentice copies each sheet letter for letter, as her teacher taught her ("never mend an exemplar"), and so the false lines pass into halls' copies in a hand the halls can't tell from their own.
**Payload ⚑ (last move):** she has the false lines found, so the Wayhouse bars the apprentice's hand from every hall, as it struck her off its roll twenty years before. Then she publishes that the lines were hers, as the master's line was his: the Wayhouse has condemned two innocent hands for two false lines, and no Writers' judgment, or copy, can stand. *(Draft 4: the apprentice was never on the roll, so she is barred, not struck off.)*

**3 · Stake:** S3 (b/c). *Reading:* the oath that a Writers' copy stands rests on the Wayhouse's judging its copyists rightly.

**4 · Heart:** H2 (W7c), `member`, cast: a letter-writer's apprentice, taught her letters and the halls' hand in secret by the enemy, who copies an exemplar letter for letter and never mends one. *Needs:* in its busy season she copies for `{reading_room}` by the sheet, as halls hire letter-writers' hands, so her sheets go out under its master's check and its oath *(draft 4: she can read; only W6-A's heart works from the shapes alone)*.
- **Want:** to be sworn a copyist at the Wayhouse.
- **Past:** no hall would take her; the struck-off copyist found her copying shop signs in a market, and taught her letters and the halls' hand for nothing.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{reading_room}`'s market | Restore *Lost livelihood* (cause `lost-knowledge`) | her old sign-painter master has lost his eyes, and his trade with them. **Heart's card**: she wants to be sworn; no hall would take her, and a struck-off copyist found her copying (flavour a says what) and taught her for nothing (her past); her mother, a market woman of `{mother_town}`, has never asked who taught her | "**a hand the halls can't tell from their own**" (the reading room's master on her copies) |
| R2 | `{reading_room}`, which buys its exemplars from the tower | O1 Acquire *Copy it before it burns* (spine variant: a hall's checker told to burn spoiled exemplar sheets). **Odd job** | a checker means to burn a batch's exemplars | "**exemplar sheets burned as spoiled**" |
| R3 | `{r3_hall}`, another hall | Investigate *The ledger doesn't add up* (signature; *swap* below) | a contract copy was altered (*for themselves*) | ∅ "**no name on the exemplars: every sheet unsigned**" |

*Swap:* **R3** may be Investigate *Watch them*: `{r3_hall}`'s master wants eyes on the carrier who now brings his copyists bought exemplars; watching, the player sees the carrier's bundles are "no name on the exemplars: every sheet unsigned" (the plant is kept). R2 is the routine's signature either way.

**6 · Odd job:** O1 (thread `answers-to`). *How it serves the scheme:* the Hand, the tower's agent who sells the exemplars to the reading room, tells the checker that a batch's exemplars are spoiled and should burn. The exemplars are the only proof that the false lines came from the sheet, not the hand that copied it.

**7 · Reveal** (giver: *an elder copyist*; form: *a chain*). `{reading_room}`'s master, a man who checks everything twice, at his hall. Passage *Unwilling passenger*: the shepherd boy who carries the hall's exemplar sheets between `{tower}` and the market won't go back up the hill; the master wants him brought round, and he is the knower.
- **Shows:** the boy says a woman in the tower writes the sheets, dries them on a line, and pays him to bring them down to the agent at the paper stall.
- **Adds:** the master adds two things, each with its source: laying one of the boy's dried sheets beside the hall's old roll, he finds a line set in that the roll never had (the false lines start in the exemplars); and from the hall's copy of the Wayhouse's striking-off roll he knows who would write so, the copyist struck off twenty years ago, and which master brought the charge. The old wrong behind it, that the line was her master's, is not his to know; her letter tells it at the crisis. *(Draft 4: a shepherd boy can't know a twenty-year-old wrong.)*
- **Repeats:** "a hand the halls can't tell from their own", "exemplar sheets burned as spoiled", "no name on the exemplars".
- **Closes:** each explains the next: "no name on the exemplars" leads up the hill to the tower; the tower's sheets explain "exemplar sheets burned as spoiled", since they were the proof; and those explain "a hand the halls can't tell from their own", the apprentice's, which copied every line exactly as the sheet set it.

**8 · Chapter 1.** *Shape:* 1 · the price at the turn. **Move:** M5 (spine reading on S3: posted, not cried): she nails to `{reading_room}`'s door a list of twelve Writers' copies, each false line marked, unsigned, in a hand the old hearing-scribe will know *(draft 4: a list signed in her roll name would publish, before the hearing, what her payload saves for after it)*. *Uses* `name`, lever rumour.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{tower}` (a journey) | Threat *Go and look* (C1) | — | go up to the tower and see what she does there. Plant: "**exemplars drying on the line, the false lines already in**" |
| C1-2 | `{reading_room}`'s market | Dispute *Public offense* | therefore | a merchant whose copy is on the list (its false line changes flavour b's kind of entry) cries the reading room cheats. Plant: "**twelve copies in one hand**", the apprentice's |
| C1-3 turn | the market's paper stall | Acquire *Won't sell* | but | the Hand holds the next batch of exemplars as his own stock and won't sell it to the halls (opposing). **Ally plant:** `{scribe}`, the Wayhouse's old hearing-scribe, who wrote down the striking-off twenty years ago and kept his draft, against the rule; he has come to buy paper and knows the hand on the batch; his price: his post kept, though the draft breaks the rule |

**9 · Chapter 2.** Opening (pressure 2): `{reading_room}`, Dispute *Broken agreement*: the twelve copies' owners hold the reading room to its oath that a copy stands, and its master, a man who checked everything twice, must answer them. Then the door. *(Draft 3: was Restore Lost good name, which shared door a's family.)*

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | heart · *estranged* · her kin turned (`kin-turned`) | `name` | Restore *Estrangement* (cause *a side taken*: her mother, told her girl copies for a struck-off forger, sides with the halls) | the apprentice must stand alone when the halls judge her, as her teacher did |
| b | player · *framed* · for destroying a text (`framed-damage`) | `name` | Investigate *Sabotage* (blamed for scraping the false lines off; done by the Hand) | anyone trying to clean the copies must look like they are hiding them |

- **The door carries the crisis's plant:** `{scribe}` (met on C1-3), who has bought the halls' paper for forty years, holds a sheet to the light: the exemplars' paper carries the tower press's watermark, which the halls' paper doesn't: "**the tower press's watermark**".
- **Lesser code-break (kind: destroy a text):** an extra way on the door: burn one of the twelve copies, so its owner can't take it to court.

**10 · Crisis** (at the Wayhouse: the spine is about the Wayhouse's own judging, so the crisis stays there). **Last move ⚑:** the Wayhouse summons the apprentice to a hearing; she rides up with `{scribe}`, who came down to buy paper (C1-3) and goes back with her. The hearing bars her hand from every hall for the twelve false lines. That night the struck-off copyist's letter is read at the Wall: the lines were hers, as the master's was his (flavour c says what she calls the girl). The enemy's want survives: she has made the Wayhouse do it again, and it hasn't yet answered.
- **Held:** the player holds the watermark finding (the door) before the hearing, but the hearing sits on the halls' rule that a copyist answers for every line in her hand, a watermark shows where a sheet came from and not who set the line, and the scribe who found it may not bring a hearing the draft its rule forbids; so the bar lands, and the crisis reads the finding: with it in hand, the ally's way stands on the scribe's sworn word on the paper as well as his draft.
- **Family:** Reckoning *Ruin from above*. Wronged: the heart. Wrongdoer: the enemy (she outranks the apprentice). Shield: *Fled* (her tower, beyond the halls). Requester: `{scribe}`, acting for her (he will not watch the Wayhouse do it twice). Twist: *the wronged don't want it*: the apprentice won't see her teacher ruined.
- **Giver:** `{eldest}`, who read the letter at the Wall. *Reason:* he read it.
- **Choice:** ruin the teacher, or answer her, and whether the Wayhouse owns two wrongs.
- **Ways:** *code-break:* destroy what she built: burn every copy she touched · *ally:* `{scribe}` reads his old draft, which names the master's book that night, so the Wayhouse's own record answers the teacher's letter (his price: his post kept) · *shield's way:* follow her to her tower and decide there.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Amends made** (the teacher confesses at the Wall; the apprentice's bar lifted and the teacher's name restored to the roll, the master's line owned) | the old hall master (struck off at last) · the guild's name (two wrong judgments owned) | persuasion | sworn at the Wayhouse, her want, in her teacher's hand / the twelve copies corrected / the teacher copies again, under her own name |
| **Justice done formally** (the apprentice's bar lifted; the teacher named a forger) | the teacher (hunted by the halls) · the heart (she must swear against her) | law | sworn, and won't use the hand she was taught / the oath stands / the tower stands empty |
| **Burn your copy** (code-break: every copy she touched burned) | the twelve copies' owners (their contracts burned) · the halls' records | stealth | the bar lifted, and her teacher's work is ash / the market has no copies to doubt / the teacher's revenge is half undone, and half done |
| **The requester forgives** (the apprentice goes to the tower to copy with her teacher) | the heart (barred from every hall for good) · the halls (they lose the best hand they had) | persuasion | copies at the tower, for anyone who pays / the Wayhouse says nothing / the teacher has a pupil, which is more than revenge gave her |
| **Reckoning fails** (the letter stands; the halls never answer it) | the heart · the guild's name | none | barred, and writes shop signs again / the market laughs at the halls' oath / the teacher waits for the next hand to teach |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *already copied*: the checker made a copy before burning | he asks whether the halls will want "the spoiled lot" back |
| R3 | culprit *for themselves* | the clerk's purse is heavier on market days |
| C1-1 | the false lines start in the exemplars | the exemplars dry on a line, not in a press |
| C1-2 | *one side lies*: the merchant knew his copy was flawed and kept it to bargain with | he has it marked already |
| C1-3 | *promised elsewhere*: the batch is promised back to the tower | the stock is wrapped and labelled for a carrier |
| door a | cause *a side taken* (`took-a-side`) | her mother repeats the halls' words exactly |
| door b | the Hand scraped them | the scraped places are on sheets the player never held |
| crisis | *as told*, twist *the wronged don't want it* | she still copies in the hand her teacher gave her, and won't change it |

**13 · Hall-job hooks**
- *Middle:* at `{reading_room}`'s town, a holder of one of the twelve copies wants it recopied "by somebody else". Reads M5; *before* face: "a list on the hall's door".
- *Late:* at another hall, a letter-writer's apprentice asks how she might be sworn. Reads the bar; *before* face: "the Wayhouse holds a hearing".
- *Seen in passing:* before the reveal, a shepherd boy gives a lost traveller directions on a side card.

**14 · What varies.** `{tower}` (remote, within a day of `{reading_room}`'s market). `{scribe}` (the Wayhouse's). *Flavour* (each list rolled on its own): (a) where the teacher found her, told on R1: copying shop signs, inn boards, or headstones; (b) what the false lines change, seen on C1-2: a sum, a name, or a date; always small, always findable; (c) what the teacher calls her in the letter read at the Wall, on the crisis: "my hand", "the halls' next mistake", or her own name. *(Draft 1's ally was the struck-off copyist's silent friend, who is W7-A's heart: one person in two roles. The ally is now the hearing-scribe, who is in no other spine. Its sheets carried "by the Hand himself" left the reveal's shepherd boy nothing to know.)*

**15 · Presuppositions:** the old striking-off (kit; reveal) · the heart taught by the enemy (W7c; her past) · `{tower}` (C1-1; reveal) · `{hand}` = the tower's agent (R2 hidden; C1-3 open) · the exemplars (R2, R3) · the twelve copies (move) · `{scribe}` and his draft (C1-3) · the watermark (door) · the hearing (the last move) · `{eldest}`.

**16 · C1 cards:** C1-1 Threat *Go and look*.

**17 · Placing.** Centre: `{reading_room}`. Main town (for *Fresh stories*): the centre's town. Kept in memory too: stake S3, move M5, openings Threat · Go and look, Dispute · Broken agreement. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{wayhouse}`, about the Wayhouse's own judging (its hearing bars her hand from every hall, as it struck her teacher off); the giver is there (the eldest copyist, who read the letter at the Wall); its action falls at `{wayhouse}`.
*Swaps* (field 5): R3 may be Investigate · Watch them.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{reading_room}` | hall | a reading room in a large town (its market, its door) | centre |
| `{tower}` | site | her lone tower on remote ground (added), in the hall's territory or Great House land, never an enclave | 1.0 from the centre |
| `{mother_town}` | town | where the heart's mother keeps her market stall: a town with a market (no keep, fort, counting-house or enclave); else the centre | ≤1.25 of the centre |
| `{r3_hall}` | hall | another hall | ≤3.0 of the centre |
| `{hand}` | person | the tower's agent who sells exemplars (made on demand) | — |
| `{scribe}` | person | the Wayhouse's old hearing-scribe (made on demand; the Wayhouse's) | — |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{reading_room}` | `{reading_room}` | bound |
| R2 | `{reading_room}` | `{reading_room}` | bound |
| R3 | `{r3_hall}` | — | free (≤3.0 of the centre) |
| RV | `{reading_room}` | `{tower}` | bound |
| C1-1 | `{tower}` | `{reading_room}` | journey (≤3.5 of the centre): up to the tower |
| C1-2 | `{reading_room}` | `{reading_room}` | bound |
| C1-3 | `{reading_room}` | `{reading_room}` | bound |
| C2-0 | `{reading_room}` | `{reading_room}` | bound |
| DOOR | a: `{mother_town}` · b: `{reading_room}` | `{reading_room}` | bound |
| crisis | `{wayhouse}` | — | the Wayhouse: the Wayhouse's hearing bars her hand: summoned to it, she rides up with the hearing-scribe, and the eldest reads the teacher's letter at the Wall that night |
| crisis, acted | `{wayhouse}` | — | where its action falls |

*Parties set up first* (checked): the heart's mother (R1); the hearing-scribe (C1-3); the teacher (RV); the old hall master (RV); the twelve copies' owners (C1-2).

---

## W8-A · The Wall Through the Spring

**1 · Enemy:** W8a, a Warden captain whose stretch of towers is failing (High); his sister heads `{camp}` and signs its readings. He acts behind the Green Wall's back: its holder, whose leaders never lie to recruits, knows nothing of it. *Above* · desperation · `men` `kin` `name` · region · *the old debt* · side: Wardens.

**2 · Scheme.** His sister signs edge readings a mile nearer the towns, and his men move the newest of `{cairns}` in by night to match, so frightened lords will pay for a wall along the new line.
**Payload ⚑ (crisis):** the lords' levy raises the wall along the false line. Its ditch cuts `{edge_village}` off from its spring and pasture, a side effect of a wall that will stand a mile inside the real edge.

**3 · Stake:** S6a, the season's maps. *Reading:* the lords pay on the Wayhouse's maps; the maps are what he has to bend.

**4 · Heart:** H3b, `outside`, cast: a carter of `{edge_village}`, writing to a creditor begging for time. *Needs:* he hauls the wall's timber to pay his debt.
- **Want:** to clear his debt and keep his family's pasture.
- **Past:** his father's cart carried the stones for the old tower line, a generation ago.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{gate_town}`, the gated town nearest the village | Passage *Unwelcome cargo* | his cart of salvage from the edge is barred at the town gate as green-touched; the hall can vouch. **Heart's card**: his debt, his want; his father's cart carried the stones for the old tower line (his past); his son drives the second cart | "**timber for a wall, hauled from the edge**" (his next load) |
| R2 | `{lodge_hall}`, where the camp lodges its surveys | O2 Investigate *The ledger doesn't add up* (signature; O2b: a survey; the Hand is the hall's checker). **Odd job** | a survey lodged at the hall doesn't match its field book | "**a checker's mark on readings nobody walked**" |
| R3 | `{post_town}`, a Warden recruiting post or the Wardens' own fort, where the towers' widows draw their pensions (a post added at a hall the routine doesn't use where none is in reach) | Acquire *Unknown worth* | a Warden's widow at the post keeps her husband's notched cairn staff, a pace-count for every cairn since the line was drawn, not knowing its worth | ∅ "**no notch on the staff for where the newest cairn stands now**" *(draft 4: was "last year's cairn", against a cairn moved this year)* |

**6 · Odd job:** O2b (thread `lie-in-record`; the culprit is the Hand). *How it serves the scheme:* the Hand, the hall's checker, in the captain's pay, marks the redrawn readings as walked and checked, so they pass into the season's maps with the halls' own mark on them.

**7 · Reveal** (giver: *the leader*; form: *laid side by side*). The leader, off the road at `{camp}` *(draft 4: was at the hall, which put all four pre-chapter cards in one town)*. Acquire *Won't sell*: the captain's sister's daughter, a copyist at `{lodge_hall}` come out to the camp to her mother, keeps her mother's private field book and won't part with it; she lends it to the leader for one reading, and is the knower.
- **Shows:** the field book, in the sister's own hand, puts the edge a mile further out than the camp's signed sheets, and enters the newest cairn at its old place, struck through: the redrawn readings, the moved cairn, and the sister's signature on both.
- **Adds:** the leader adds the old debt, from the Wayhouse's certification book, which she has read: a generation ago the Wayhouse certified the survey that drew his towers' line where a lord wanted it, not where the green was; the daughter adds what her uncle tells himself (flavour a).
- **Repeats:** "timber for a wall", "a checker's mark on readings nobody walked", "no notch on the staff for where the newest cairn stands now".
- **Closes:** the leader lays the field book's true readings beside "a checker's mark on readings nobody walked", its struck-through cairn beside the widow's staff ("no notch on the staff for where the newest cairn stands now"), and the true edge, a mile out, beside the carter's next load: "timber for a wall", for the wrong line.

**8 · Chapter 1.** *Shape:* 2 · the ally first. **Move:** M5 on S6: he has it cried in the edge towns that the Writers' maps put the green too far off. *Uses* `name`, lever rumour.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{cairns}` (a journey) | Acquire *Take the readings* (C1) | — | take the cairns' bearings against the oldest stones, where the Wardens' cairn-keeper won't let a Writer's chain near, with `{surveyor}`, the camp surveyor, carrying the chain (the herders say the newest was moved on flavour b's night). Plant: "**the newest cairn, its stones scraped clean of lichen**". **Ally plant:** `{surveyor}`, who signed the season's sheets under protest (flavour c, voiced here, says why he signed); his price: his name off the maps |
| C1-2 | `{edge_village}` | Dispute *Shared resource* | therefore | the village against the Wardens' diggers over the spring meadow the new line crosses; the oldest herder walks the old line for the village, and its headman says little. Plant: "**pegs for a ditch across the spring meadow**" |
| C1-3 turn | `{camp}` | Rescue *Imprisoned unjustly* | but | the camp's chain-boy is held in the store-hut under the captain's escort for carrying the ally's written protest out of the camp, the captain's sister keeping the key (kin, a proxy, opposing); the protest, with the ally's name on it, is in her hands now |

**9 · Chapter 2.** Opening (pressure 2): the wall's timber yard, Dispute *Broken agreement*: the village's other carters against the Wardens' yard master, who hires only from the towns that cried the alarm, against the contract that promised the village's carts the hauling. Then the door. *(Draft 3: was Restore Lost livelihood, which shared door a's family.)*

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | heart · *estranged* · his kin turned (`kin-turned`) | `kin` | Restore *Estrangement* (cause *a side taken*: his son takes the Wardens' shilling and calls his father the fool who'd leave the village open) | the carter hauls the timber; his doubt would stop the loads |
| b | player · *arrested* · in a lord's cells by his men (`arrested-men`) | `men` | Passage *Fugitive* (cargo: the player; pursuer: his men) | held at `{green_wall}` as green-touched after walking the true edge |

- **The door carries the crisis's plant:** the oldest herder (met on C1-2) says the spring under the meadow also feeds the lowland town below: "**the spring the lowland town drinks from too**".
- **Lesser code-break (kind: take a side in public):** an extra way on the door: tell the headman privately that the readings are false.

**10 · Crisis.** **Last move:** the lords' levy is voted on the season's draft maps, to be paid once `{lodge_hall}`, where the camp lodges its surveys, certifies them *(draft 4: was voted before the certification it waits on)*; on the vote the Wardens' diggers start the ditch along the new line, and next week it crosses the spring meadow. The lords' stewards want the certification in `{days}` days, and the hall's master has come out to the meadow to see the line before he signs. ⚑ The payload is attempted here: the wall goes up unless it is stopped or moved. The enemy's want survives: his towers still need money.
- **Held:** the player knows the readings are false from the reveal, but the levy is the lords' and the certification is the hall master's, and he certifies from the halls' own checked sheets; the crisis is where the player meets the one man who can refuse, at the meadow, with what the player holds.
- **Family:** Threat *Encroachment*. Encroacher: the captain (the strip is granted to the Wardens). Community: `{edge_village}`. Requester: the heart. Truth (public): *project flawed* (`built-to-fail`): a wall a mile inside the real edge guards nothing. Twist: *one of us caused it*: the headman sold the meadow strip to the Wardens.
- **Giver:** `{lodge_hall}`'s hall master, at the spring meadow. *Reason:* the stewards want his certification, and the heart has met him there with the village's petition and the last load of timber.
- **Choice:** stop the wall, move it, or let it stand and save what you can.
- **Ways:** *code-break:* take a side in public: `{lodge_hall}` refuses to certify, and says why (the Wayhouse's old certification too) · *ally:* `{surveyor}` swears the true readings (his price: his name off the maps) · *prove the project will fail:* lay the cairns' true bearings before the paying lords.

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Threat ended** (the true maps certified; the levy withdrawn; the wall stopped) | the captain's stretch (no money; the towers keep failing) · `{surveyor}` (his name on the true maps; the Wardens' enemy) | law | his pasture kept, his debt not / the spring flows / the captain begs at the next recruiting post |
| **Bargain struck** (the lords pay for a wall on the true edge, as the old certification should have drawn it) | the Wayhouse (it owns the old certification) · the captain's sister (her false signature out) | persuasion | hauls timber for the right wall, and clears his debt / the village inside the wall, with its spring / the captain has his wall, and his sister's name is gone |
| **Protected, the threat remains** (the spring piped round the ditch; the wall on the false line) | the lowland towns (paying for a wall in the wrong place) · the heart (debt still owed) | bribe | keeps the meadow and owes the Wardens the pipe / the wall stands a mile from the green / the captain's towers fail a mile further out |
| **Relocated** (the herds moved below the line) | the heart (pasture lost) · his neighbours | none | sells his cart to pay the creditor / the meadow is the Wardens' / the captain wins, for one season |
| **Take a side at last** (code-break: the Writers stand for the village against the Wardens in public) | the guild's tie with the Wardens (the survey camps lose their towers' shelter) | persuasion | his son comes home / the wall is stopped / the Wardens turn the Writers' camps out of the edge |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *to hide an older change* (`older-wrong-beneath`): the checker's mark covers an older false line, the Wayhouse's old certification | the readings' oldest line is in a hand a generation older than the rest |
| R3 | *as told* | the staff's last notch is cut deeper than the rest, at a pace-count from the old line |
| C1-1 | the newest cairn stands where no green has come | fresh earth round its base, and no seedlings |
| C1-2 | *record survives*: the oldest herder remembers where the green stood | he walks to the old line without looking down |
| C1-3 | *innocent*, twist *not what it seems*: the boy only carried a letter; the sister burned nothing, and keeps the true journals | her own tent has a locked chest she sits on |
| door a | cause *a side taken* | the boy repeats the captain's cry word for word |
| door b | *the pursuer was told*: the village's headman told them the player's road | the headman's new fence is Warden timber |
| crisis | public truth *project flawed*; twist *one of us caused it* | the headman won't stand on the meadow |

**13 · Hall-job hooks**
- *Middle:* at a hall near the edge, a farmer wants the halls to say whether his farm is "inside the line". Reads M5; *before* face: "talk of the green coming nearer".
- *Late:* at a hall below the edge, a mill wants its water rights from the spring copied. Reads the ditch; *before* face: "a wall to be built".
- *Seen in passing:* before the reveal, a Warden at a recruiting post shares his bread with a beggar on a side card.

**14 · What varies.** `{camp}`, `{cairns}`, `{edge_village}` (a registry village with a Low or Middle holder; its headman is that holder): all on the captain's stretch. *Flavour* (each list rolled on its own): (a) what the captain tells himself, voiced on the reveal: the green will come anyway, his men are starving, or the lords never paid for the old line; (b) the night the cairn was moved, told on C1-1: a moonless night, a storm night, or the night of the herders' fair (no season word: the cairn is this year's, whatever the story's season); (c) why `{surveyor}` signed, told on C1-1: his pay, his sister-in-law's post, or an order he couldn't refuse. *(Draft 1's ford and drove road contradicted the spring the title, C1-2, the door and three endings rest on; its "hired carter" moving the cairn blurred into the heart, a carter; its ally options couldn't all swear the readings.)*

**15 · Presuppositions:** the captain's failing stretch (kit) · his sister heads `{camp}` (W8a) · the cairns, one a year (faction-places) · the old certification (kit; reveal) · `{hand}` = the hall's checker (R2 hidden; open on the reveal) · the heart's debt and timber hauling (R1) · the widow's staff (R3) · `{surveyor}` (C1-1) · the spring (C1-2; door) · the headman (C1-2; door b; crisis) · the chain-boy and the surveyor's protest (C1-3) · the lords' levy, and `{lodge_hall}`'s certification of the maps, its master at the meadow (the last move; crisis).

**16 · C1 cards:** C1-1 Acquire *Take the readings*. *(Draft 3: C1-3 is Rescue Imprisoned unjustly, so chapter 1 no longer holds two Acquire cards.)*

**17 · Placing.** Centre: `{camp}`. Main town (for *Fresh stories*): `{edge_village}`. Kept in memory too: stake S6, move M5, openings Acquire · Take the readings, Dispute · Broken agreement. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{edge_village}`, where the story is; the giver is there (the hall master who must certify the maps, come out to see the line); its action falls at `{edge_village}`. *Clock:* `{days}` = the road from `{edge_village}` to `{lodge_hall}`, rounded up, + 1 (the maps must be certified at the hall before the ditch reaches the spring).

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{camp}` | site | a Writers' survey camp at the green's edge (presence); his sister heads it | centre |
| `{cairns}` | site | the Wardens' marker cairns | nearest to the camp |
| `{edge_village}` | village | a Low or Middle village (d-l0 holder) within a day of the camp, not a Warden fort | ≤1.25 of the camp |
| `{green_wall}` | fixed | the Green Wall (f-wardens; held by d-l0's Haʻo) | — |
| `{gate_town}` | town | a large town with a gate | ≤1.25 of the village, else ≤1.6 |
| `{lodge_hall}` | hall | the hall where the camp lodges its surveys, which must certify the season's maps | ≤1.25 of the camp, else ≤1.6 (nearest) |
| `{post_town}` | town | a Warden recruiting post (presence) or the Wardens' own fort, not the Wayhouse's town; else one added at the nearest hall (draft 4: not a town the routine already uses) | ≤3.0 of the camp |
| `{timber_yard}` | site | the wall's timber yard (added) | 0.4 from the village |
| `{captain}` | person | a Warden captain of the towers here, under the Green Wall's holder (made on demand) | — |
| `{hand}` | person | lodge_hall's checker (made on demand; the hall's) | — |
| `{surveyor}` | person | a camp surveyor who signed under protest (made on demand; the camp's) | — |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{gate_town}` | `{edge_village}` | bound |
| R2 | `{lodge_hall}` | `{camp}` | bound |
| R3 | `{post_town}` | — | free (≤3.0 of the centre) |
| RV | `{camp}` | `{camp}` | bound |
| C1-1 | `{cairns}` | `{camp}` | journey (≤3.5 of the centre): out to the cairns |
| C1-2 | `{edge_village}` | `{camp}` | bound |
| C1-3 | `{camp}` | `{camp}` | bound |
| C2-0 | `{timber_yard}` | `{edge_village}` | bound |
| DOOR | a: `{edge_village}` · b: `{green_wall}` | `{camp}` | bound |
| crisis | `{edge_village}` | `{camp}` | bound |
| crisis, acted | `{edge_village}` | — | where its action falls |

*Parties set up first* (checked): the heart's son (R1); the oldest herder (C1-2); the headman (C1-2); the surveyor (C1-1); the captain's sister (RV); the captain (RV).

---

## W8-B · The Mother's Journal

**1 · Enemy:** W8c, a Warden captain whose stretch of towers is failing (High); his men "escort" the camps and carry their journals to the hall where they are lodged themselves. *Above* · desperation · `men` `kin` `name` · region · *the old debt* · side: Wardens.

**2 · Scheme.** His men escort the survey camps and carry their journals to `{lodge_hall}`, where the camps lodge them and from whose lodgings the Wayhouse's register is copied. On the road the true journals are "lost" and redrawn ones lodged in their place, so the lords' stewards, who vote the wall levy at `{lodge_hall}` on the season's lodged maps, read maps that put the green nearer and vote money for his wall.
**Payload ⚑ (crisis):** the redrawn journals stand as the season's lodged maps, and the stewards vote the levy on them at `{lodge_hall}` on `{vote_day}`, unless the true journals get there first.

**3 · Stake:** S6c, the old maps the new ones are checked against. *Reading:* the halls check each season against the old maps; whoever carries the journals decides what gets checked.

**4 · Heart:** H5a, `outside`, a man (the son): the son of a surveyor who went into the green and never came back; he keeps her last field journal and can't read it. *Needs:* she was lost on the captain's stretch, and her journal is of it.
- **Want:** to have someone read him his mother's journal.
- **Past:** he found it in her hut at the camp, the day she didn't come back. It is written in her own cipher *(draft 3: the camps are the Writers' own, so a common surveyors' shorthand would be read at any hall)*.

**5 · Routine**

| # | Place | Family · premise | The job | Quiet plant |
|---|---|---|---|---|
| R1 | `{lodge_hall}`, the hall nearest the camp | Restore *Lost good name* (the harm is off the card) | his mother is mocked as "the surveyor who walked into the green for nothing". **Heart's card**: he wants her journal read; he found it in her hut at the camp the day she didn't come back, and has kept it since (flavour b says where) (his past) | "**a journal in her own cipher, which nobody at the hall can read**" |
| R2 | `{widow_town}`, near the camp | O1 Acquire *Copy it before it burns* (spine variant: a surveyor's widow, told his journals carry the green). **Odd job** | a widow means to burn her husband's field journals | "**field journals burned as green-touched**" |
| R3 | `{r3_hall}`, which keeps the year's copy of the Wayhouse's edge-map register (every hall keeps the year's copy *(Claude)*) | Investigate *The ledger doesn't add up* (signature; *swap* below) | the register's year copy, made each season from the halls' lodgings, has an altered line (*for themselves*) *(draft 4: was at the Wayhouse, one of five stories through Highcrossing)* | ∅ "**the register's 'carried by' column, blank since the escorts began**" |

*Swap:* **R3** may be Dispute *Broken agreement*: the hall's register clerk and the Warden escorts who carry the lodging copies quarrel over the carriage fee, since the escorts sign for nothing; the register, opened to settle it, shows "the register's 'carried by' column, blank since the escorts began" (the plant is kept). R2 is the routine's signature either way.

**6 · Odd job:** O1 (thread `answers-to`). *How it serves the scheme:* the Hand, the captain's sergeant who leads the escorts, told the widow her husband's journals carry the green (she repeats what he tells the camps, flavour a). They are twenty years of the true edge on his stretch, the last record his redrawn lines could be checked against outside the Wayhouse.

**7 · Reveal** (giver: *the heart*; form: *the knower's question*). At `{lantern_hall}`, where the Lantern chapter is; the heart brings the player, to ask the Lantern captain who escorted his mother's camp. Dispute *Broken agreement*: the Lantern contract camp that used to escort the surveyors (the Lantern Company, the road guards' company), against the Wardens who took the contract (an ally-guild meeting). The Lantern captain is the knower.
- **Shows:** he saw the Wardens' escort redraw journals by a tower fire, on the road from the camp, the season they took the contract.
- **Adds:** the heart adds whose stretch it is, from his mother's last post (the captain's towers, where she was sent in); the Lantern captain adds the old debt, from his chapter's own contract book: the Lantern escorted the Wayhouse's certifiers a generation ago, and the book records where a lord's steward had them set the towers' line.
- **Repeats:** "her own cipher, which nobody can read", "field journals burned as green-touched", "the 'carried by' column, blank".
- **Closes:** the Lantern captain asks the player three things: who at the hall can read a journal "in her own cipher, which nobody at the hall can read", why a widow would burn "field journals burned as green-touched", and who has signed "the register's 'carried by' column" since the escorts began.

**8 · Chapter 1.** *Shape:* 1 · the price at the turn. **Move:** M6 on S6: he closes `{camp}` "for its safety" and keeps its journals. *Uses* `men`, lever force. Shuts a place.

| Card | Place | Family · premise | Link | What happens · plant |
|---|---|---|---|---|
| C1-1 | `{watchtower}` above the closed camp (a journey) | Investigate *Watch them* (C1) | — | watch the tower where the journals went. Plant: "**a straight-edge and a pot of camp ink in a Warden tower**" |
| C1-2 | the true edge on his stretch | Acquire *Take the readings* (C1) | therefore | take the edge's readings again with the camp's staff and chain. Plant: "**the true edge, a mile out from the redrawn line**" |
| C1-3 turn | `{green_wall}` | Dispute *Public offense* | but | the sergeant (the Hand, opposing) cries the heart's mother a deserter who walked into the green; the son struck him, and he demands redress. His mother's brother, `{uncle}`, a sergeant of the Wall, stands with the Hand. **Ally plant:** `{old_surveyor}`, who learned the mother's cipher on the surveys they walked together (flavour c says why he left the camps); the heart leaves her journal with him, to be read at last; his price: his own old journals lodged at the Wayhouse under his name |

**9 · Chapter 2.** Opening (pressure 2): `{edge_village_b}`, Passage *Unwelcome cargo*: the closed camp's surveyors, now "green-touched", are barred from its inn by its headman. Then the door. *(Draft 3: the one Unwelcome cargo opening left in the sixteen; its village is placed, and is the crisis's first waypoint.)*

| Option | Target · event · how it fell | Uses | Door | Why this scheme strikes here |
|---|---|---|---|---|
| a | player · *framed* · for destroying a text (`framed-damage`) | `name` | Investigate *Sabotage* (blamed for a camp journal "lost" from the escort's packs; the Hand took it) | the player walked the true edge with the camp's staff; a Writer seen near the journals is the easiest one to blame |
| b | heart · *estranged* · his kin turned (`kin-turned`) | `kin` | Dispute *Contested inheritance* (a third kin member: `{uncle}`, met on C1-3, claims the mother's journal as her brother) | the journal is the one record of the old line on his stretch; in a Warden's hands, it is the captain's |

- **The door carries the crisis's plant:** `{old_surveyor}` (met on C1-3), who knows the camps' calendar, says the stewards vote the levy at `{lodge_hall}` on `{vote_day}`, on the season's lodged maps: "**the vote on {vote_day}**".
- **Lesser code-break (kind: destroy a text):** an extra way on the door: cut the sergeant's redrawn page out of one lodged journal.

**10 · Crisis** (at `{old_surveyor}`'s hut beyond the towers). **Last move:** the redrawn journals are lodged at `{lodge_hall}`. The stewards vote there on `{vote_day}`, `{days}` days off. The captain's towers bar green-touched cargo from his stretch. The true readings and the mother's journal, read at last by `{old_surveyor}`, wait at his hut, where the heart has come to hear it. ⚑ The payload is attempted here: the vote on the false maps, unless the true journals arrive. The enemy's want survives: the vote hasn't passed.
- **Held:** knowing the journals are redrawn (the reveal) is not the true journals; only the true readings lodged at the hall answer them, the captain's towers stand between the hut and the hall, and no Writer can make the stewards wait; the crisis is that carriage.
- **Family:** Passage *Samples from the green* (signature). Cargo: the true journals and the mother's. Waypoints: `{edge_village_b}` and `{edge_village}`, whose headmen bar green-touched goods. Route authority, opposing: the captain, whose towers search what comes out of the green *(Claude: the Wardens search at their towers)*. Requester: the heart. The card is at the hut; the ways run from it to `{lodge_hall}`'s door.
- **Giver:** the heart, at the hut, his mother's last page just read to him. *Reason:* her journal, and the vote.
- **Choice:** carry the truth through the line, and what to let be read on the way.
- **Ways:** *code-break:* burn the redrawn journals lodged at `{lodge_hall}`, so the stewards have nothing to vote on · *ally:* `{old_surveyor}` walks it along the tower line, a Warden-wise route (his price: his journals lodged) · *seal it under a copyist's wax, as the hall's letters.*

| Ending | Costs | Won by | A month on: heart / place / other side |
|---|---|---|---|
| **Delivered** (the true journals reach `{lodge_hall}`; the vote is on the true edge) | the captain's stretch (no wall money) · `{old_surveyor}` (his name set against a Warden captain) | stealth | has heard his mother's journal read, every page / the edge maps true / the captain's towers fail, and he says so |
| **Delivered late, or in part** (only the mother's journal arrives; the vote is put off a season) | the heart (her last page is read out at the vote) · the lords' towns (a season arming against nothing) | persuasion | knows what her last page says, and so does everyone / the maps argued for a season / the captain has a season more |
| **Open the ledgers** (`{lodge_hall}` publishes the true maps, and the Wayhouse its old certification) | the Wayhouse's old certifiers · the Wayhouse's name | law | his mother named as the surveyor who was right / the towers' line redrawn where the green is / the captain's grievance answered, and his redrawing too |
| **Seized** (the captain's men take the journals at the towers) | the heart (his mother's journal goes into the Wardens' keeping) · the edge towns | none | has nothing of her left / the vote passes on the false maps / the wall is built |
| **Burn your copy** (code-break: the redrawn journals burned at `{lodge_hall}`) | the edge towns (no maps this season) · the guild (it burned its own lodgings) | stealth | still waits for his mother's journal / the lords vote nothing / the captain redraws next season's |

**12 · Truths and tells**

| Card | Hidden truth | Tell |
|---|---|---|
| R2 (odd job) | *as told* (confirming); the hidden role is the sergeant's link to the captain, a secret fact | the widow calls him "the kind Warden who carries the journals" |
| R3 | culprit *for themselves* | the register clerk's late fees go unpaid by one camp only |
| C1-1 | the sergeant redraws the journals by the fire | camp ink in a tower that never surveys |
| C1-2 | the true edge lies a mile out | old stumps, not seedlings, on the redrawn line |
| C1-3 | *one side lies*: the sergeant knew she was sent in | he says "deserter" before anyone mentions her |
| door a | the Hand took the journal | its strap was cut with a Warden's belt-knife, notched the way theirs are |
| door b | *both partly right*: the uncle is her heir too | he can't read the journal either, and holds it the right way up |
| crisis | *not what it seems* (`more-than-said`): her last page names the captain's order sending her in | the last page is in a different, hurried hand |

**13 · Hall-job hooks**
- *Middle:* at a hall near the edge, a survey camp's cook wants a letter written home, since the camp is closed. Reads M6; *before* face: "the camps are escorted now".
- *Late:* at `{lodge_hall}`'s town, a lord's steward wants last season's edge map copied before the vote. Reads the vote; *before* face: "a levy for a wall".
- *Seen in passing:* before the reveal, a Warden escort helps a stuck surveyor's cart out of a ford on a side card.

**14 · What varies.** `{camp}`, `{watchtower}`, `{green_wall}`: on the captain's stretch. `{edge_village_b}`: a second settlement within a day of the camp (a hamlet on the road to the hall is added where none). `{vote_day}` (rolled; `{days}` = the road from the hut through `{edge_village_b}` and `{edge_village}` to the hall). `{r3_hall}`: a hall within three days of the camp, not `{lodge_hall}` where another is in reach. *Flavour* (each list rolled on its own): (a) what the sergeant tells the camps, repeated on R2 by the widow: the green's too near to walk alone, the captain's orders, or bandits on the road; (b) where the son kept the journal, told on R1: inside his shirt, in a tin box, or wrapped in his mother's scarf; (c) why `{old_surveyor}` left the camps, told on C1-3: his knees, a quarrel with the captain, or grief. *(Draft 1's escorts that "swap" or "lose and replace" contradicted the redrawn journals everywhere else; a tower or shieling for the true readings left `{old_surveyor}` out of his own crisis way; the Lantern captain as ally was the reveal's knower.)*

**15 · Presuppositions:** the captain's escorts (W8c; reveal) · his failing stretch and the old certification (kit; reveal) · the mother lost on his stretch, and her journal (the heart's spec; R1) · the widow and her husband's journals (R2) · `{hand}` = the escorting sergeant (R2 hidden; C1-3 open) · the register (R3) · the Lantern captain (reveal) · the mother's own cipher (R1) · `{old_surveyor}`, who reads it, and the journal left with him (C1-3) · `{uncle}` (C1-3; door b) · `{edge_village_b}` (ch2 opening; crisis) · the vote at `{lodge_hall}` (scheme; door) · the towers' search *(Claude)* (crisis).

**16 · C1 cards:** C1-1 Investigate *Watch them*; C1-2 Acquire *Take the readings*.

**17 · Placing.** Centre: `{camp}`. Main town (for *Fresh stories*): `{edge_village}`. Kept in memory too: stake S6, move M6, openings Investigate · Watch them, Passage · Unwelcome cargo. Checked by `roll_samples.py` (*Placing a spine*).

*Crisis:* at `{hut}`, where the story is; the giver is there (the heart, at the hut, his mother's last page just read to him); its action falls at `{hut}`. *Clock:* `{days}` = the road from `{hut}` to `{edge_village_b}`, plus the road from `{edge_village_b}` to `{edge_village}`, plus the road from `{edge_village}` to `{lodge_hall}`, rounded up, + 1 (the journals must go from the hut, through both villages, to the hall's door); `{vote_day}` is that many days off at the last move.
*Swaps* (field 5): R3 may be Dispute · Broken agreement.

| Slot | Kind | Needs (owner, feature; from d-l0 and the map) | Bound |
|---|---|---|---|
| `{camp}` | site | a Writers' survey camp at the green's edge (presence) | centre |
| `{watchtower}` | site | a Warden watchtower above the camp (f-wardens) | nearest |
| `{green_wall}` | fixed | the Green Wall (f-wardens; held by d-l0's Haʻo) | — |
| `{edge_village}` | village | a Low or Middle village within a day of the camp | ≤1.25 of the camp |
| `{edge_village_b}` | village | a second settlement within a day of the camp, not a Warden fort (added as a hamlet on the road to the hall if none) | ≤1.25 of the camp |
| `{hut}` | site | the old surveyor's hut beyond the towers (added) | 0.6 past the watchtower |
| `{true_edge}` | site | the true edge (added) | 0.6 from the camp |
| `{lodge_hall}` | hall | the hall nearest the camp, where the camps lodge their journals and the lords' stewards vote the levy | ≤1.25, else ≤1.6 |
| `{lantern_hall}` | hall | a reading room with a Lantern chapter house (the old escorts) | ≤1.25, else ≤1.6 |
| `{widow_town}` | town | a settlement near the camp, not a Warden fort | ≤1.25 of the camp |
| `{r3_hall}` | hall | a hall keeping the year's copy of the Wayhouse's edge-map register (every hall keeps the year's copy), not lodge_hall where one is in reach | ≤3.0 of the camp |
| `{vote_day}` | day | the day the stewards vote the levy (roller: a day name; `{days}` days off at the last move) | — |
| `{captain}` | person | a Warden captain of the towers here (made on demand) | — |
| `{hand}` | person | the captain's escort sergeant (made on demand; the Wardens') | — |
| `{uncle}` | person | the heart's mother's brother, a sergeant of the Green Wall (cast with the heart's household) | at the Green Wall |
| `{old_surveyor}` | person | an old surveyor who learned the mother's cipher on the surveys they walked (made on demand) | at the hut |

| Card | Place | Within 1.25 of | Mode |
|---|---|---|---|
| R1 | `{lodge_hall}` | `{camp}` | bound |
| R2 | `{widow_town}` | `{camp}` | bound |
| R3 | `{r3_hall}` | — | free (≤3.0 of the centre) |
| RV | `{lantern_hall}` | `{camp}` | bound |
| C1-1 | `{watchtower}` | `{camp}` | journey (≤3.5 of the centre): up to the tower above the camp |
| C1-2 | `{true_edge}` | `{camp}` | bound |
| C1-3 | `{green_wall}` | `{camp}` | bound |
| C2-0 | `{edge_village_b}` | `{camp}` | bound |
| DOOR | a: `{green_wall}` · b: `{edge_village}` | `{camp}` | bound |
| crisis | `{hut}` | `{camp}` | bound |
| crisis, acted | `{hut}` | — | where its action falls |

*Parties set up first* (checked): the uncle (C1-3); the old surveyor (C1-3); the second village (C2-0); the widow (R2); the sergeant (the Hand) (C1-3); the captain (RV).

---
## Self-check

### Draft 4 checks (`review-v7c.md`'s six patterns and its ceilings)

`roll_samples.py` checks each of these on every run; a break prints FAIL and exits 1.

| Pattern | Root fix | Checked by the roller |
|---|---|---|
| 1 · slots carry distance, not ownership | *Placing a spine* 11 (lordship, enclaves, kind of town); `owns` needs on W2-A, W2-B, W5-B, W6-A, W7-A, W7-B; W1's lords share a major faction and lodge at their own or a neutral town | every `owns` slot per story (lordship, territory, market); W1's faction and hall |
| 2 · the reveal's knower publishes what the found thing can't show | *Rendering*: Shows, then Adds with who and how; all 16 fields 7 rewritten | field 7 has Shows, Adds, Repeats, Closes; no "all point at" |
| 3 · the player holds the warning and the payload lands anyway | a **Held** line in every field 10 (W4-A's tube is last fair's list; W7-B's hearing; W5-A's Wall promise) | every field 10 has Held |
| 4 · clocks count one leg | `{days}` sums every leg plus fixed extras, rounded from the printed sum (W2-B: summons, run, lofts; W8-B: both villages); W3-B's box from her back room; W8-A's levy on the draft maps | the clock equals its legs, rounded up, + 1 |
| 5 · status words | W7-B *barred*, list unsigned; **Standing** where a blow strips the player; W6-A's frame on the courier's road | every spine with a `stripped` blow has Standing |
| 6 · flavour that doesn't travel | no place names, no season words, outside factions glossed; W5-A's lodging named per place by the roller | every flavour option, against the map's place and region names |
| ceiling · one chapter-1 shape | three shapes (field 8), each ≤4 in a set | links, turn card, where the ally is met; shape cap |
| ceiling · the leader at every reveal | five givers (leader ×4 spines, heart ×5, elder copyist ×3, ally ×2, found letter ×2); the leader ≤3 in a set | field 7's giver and form match the roller; giver and form caps |
| ceiling · five of eight through Highcrossing | W2-A, W3-B, W7-A, W8-B's routine leaves the Wayhouse; ≤3 stories in a set deal a card there | per story, any dealt card at Highcrossing |
| ceiling · two copyists who copy by shape | only W6-A's heart (W1-B and W7-B can read) | "copies … by shape" in at most one spine |
| ceiling · W4-A in 17 sets of 21 | W4-B's heart is no longer a lost surveyor's child; the motif, not H5, is once a set | sets-run spine counts (`--sets`) |

### Draft 2 checks (the review's work list)

`roll_samples.py` runs the static checks over all 16 spines before it rolls, and the placement checks on every story it rolls; any break prints FAIL and exits 1.

| Check | Result |
|---|---|
| Heart type in at most 3 of 16 spines | ✓ H1–H5 three each, H6 one (draft 1: H4 in four) |
| No `wounded-men` blow for a hidden-hand enemy (W2, W3, W4, W8) | ✓ W2-A's option a is now a hall seized; W8-B's is the player framed |
| No `cried-down` blow where the crisis blames the heart (W6-B, W7-A) | ✓ W6-B's option a is now a hall seized by her order; W7-A's is the player stripped |
| Every *a copy in every hall* / *trade what we know* ending has an earlier plant, listed in field 15 | ✓ W1-A (C1-2, C1-3), W2-A (R1), W2-B (R3), W3-B (ch2 opening), W5-A (C1-2, reveal), W6-A (R2, C1-2) |
| Every flavour list has 2+ options, and no option moves a fact | ✓ by reading; each field 14 notes what draft 1 option was cut and why |
| One heart's card and one odd job in the routine; the odd job is never R1 or the heart's | ✓ all 16 |
| Lords and courts from the registry | ✓ every lord, court and haven slot is a `d-l0` seat holder of a named kind (field 17) |
| W7-A's heart is no other spine's ally | ✓ W7-B's ally is the hearing-scribe |
| Two "eldests" | ✓ `{eldest}` is only the Wayhouse's eldest copyist; a hall's old reader is `{tutor}` (W6-A, W6-B) |
| Placement on seeds 7301–7308 | ✓ all 8 pass; see `roll_samples.py`'s placement report |

### Draft 1 self-check (kept)

Run over all 16 against the five tests in the brief. ✓ means it held as written; *fixed* means it failed on the first pass and the text above is the corrected version. Rows for spines draft 2 rewrote say so.

| Spine | Presuppositions cover the text | Odd job ↔ scheme sentence | Every truth has a tell | Every ending names who pays (≤1 other-side-only) | Payload attempted on the page |
|---|---|---|---|---|---|
| W1-A | ✓ (truce lodging rule and hold rule marked *Claude*, both set before they're used) | ✓ the sorter receives the "will" | ✓ 7/7 | ✓ none other-side-only | ✓ crisis: the hold |
| W1-B | ✓ | ✓ the tithe book's burning leaves the halls' readings alone | ✓ 7/7 | ✓ | ✓ last move: the village burns |
| W2-A | ✓ (*fixed:* C1-3 was *Won't sell*, but the dig's wage-roll isn't the foreman's to own; now *Under seal* under the lord's closure. Draft 2: the court's need of a living witness marked *Claude* and planted on the door) | ✓ the catalogue was the provenance | ✓ 7/7 | ✓ | ✓ crisis: get the last witness to the hearing |
| W2-B | ✓ (draft 2, rewritten: the halls' own seal marked *Claude*; the magistrate's back-dated lease planted on R3) | ✓ the player's crossing puts the sale in the halls' own post | ✓ 7/7 | ✓ | ✓ crisis: the suit that makes the Wayhouse a seller |
| W3-A | ✓ (*fixed:* R1 now puts the reading-out book on the card where the plant is seen) | ✓ the copy room's wax gets her stock through the Wall | ✓ 9/9 | *fixed:* "Amends" cost only the other side, alongside "Wrongdoer ruined"; it now also costs the heart | ✓ last move: the booth, and her flight |
| W3-B | ✓ (*fixed:* the odd job's account is now the halls' bill to a lord's court, settled at the Wayhouse, so the stake isn't named and no second court is needed) | ✓ the altered account opens the commission to bids | ✓ 8/8 | ✓ | ✓ last move: the commission sealed to her |
| W4-A | ✓ (the founding book's oath *Claude*; the painted board on R1 points to it) | ✓ the loft carries the lists | ✓ 8/8 | ✓ | ✓ last move: fair night |
| W4-B | ✓ (the clerk's rooms behind his hall *Claude*) | ✓ the copied accounts show which forgiven debts can be resold | ✓ 9/9 | ✓ | ✓ last move: quarter day, the sister held |
| W5-A | ✓ | ✓ the copied letters name the pigeonholes | ✓ 8/8 | ✓ | ✓ crisis: burn at dawn |
| W5-B | ✓ (*fixed:* R2 and R3 swapped so no two routine cards in a row share a family; the fallback is first seen on R1) | ✓ the day-book's copy tells him where the roll lay | ✓ 8/8 | ✓ | ✓ last move: roll burned; crisis: the recanting |
| W6-A | ✓ (the leader acts only by letter; never at the Wayhouse) | ✓ the mending book hides the losses | ✓ 8/8 | ✓ | ✓ crisis: the court hears only her reading |
| W6-B | ✓ | ✓ the letter-book was the only outside proof of the asking | ✓ 9/9 | ✓ | ✓ last move: the archive burns |
| W7-A | ✓ | ✓ "late, unsealed" makes the court check alone with her | ✓ 8/8 | ✓ | ✓ crisis: the comparing at the court |
| W7-B | ✓ | ✓ burning the exemplars removes the proof the lines came from them | ✓ 8/8 | ✓ | ✓ last move: the apprentice struck off |
| W8-A | ✓ (*fixed:* the reveal was a third "estranged kin, one is the knower" card; now Acquire *Won't sell*) | ✓ the checker's mark launders the readings | ✓ 8/8 | ✓ | ✓ crisis: the wall's ditch |
| W8-B | ✓ (the towers' search *Claude*) | ✓ the widow's burned journals were the outside check | ✓ 8/8 | ✓ | ✓ crisis: the vote on false maps |

Rules checked across all 16: no crisis shares its door's family + premise; no blow holds the heart, so no crisis that needs her acting follows a captive; each spine spends at most one writ (W2-B the summons, W5-A the burning writ, W5-B the outlawing; the rest none); cost enemies (W1, W6) take only per-enemy blows; a struck-down leader is only ever wounded or disgraced, never killed, and never under W6; every move is the kit's, read for this spine, with a lever its means allow.

**Known soft spots** (left for the reviewer to judge):
- **Crisis cards whose action is away from the Wayhouse** (W5-B: the Mount's cells; W8-B: the road through the towers; W7-A: the court; W2-A, W2-B: the road to a court or a haven) are pinned at the Wayhouse through their giver. Draft 2 makes this a rule (*Placing a spine* 4: decided at the Wayhouse, acted where field 17 says), and every such card says why its giver is in Highcrossing. Faction-stories §6 says "at headquarters" without saying whether the action may be elsewhere.
- **Routine signature cards lean on *The ledger doesn't add up*** (13 of 16 spines use it once). It is the Writers' only signature premise that fits a hall desk; *Copy it before it burns* carries most of the rest. The family + premise of the routine cards is the generator's colour, so the roller can swap it where a spine allows.

---

## Variety table

| Spine | Enemy | Want (enemy / heart) | Payload | Crisis family · premise | Blow targets (options) | Emotional core |
|---|---|---|---|---|---|---|
| W1-A The Truce Held for a Debt | W1a | preservation / train an apprentice | a truce never lodged | Acquire · *Under seal* | heart, player, player | the keepers of balance against a peace; an old clerk's own line used to stop it |
| W1-B The Stone That Walked | W1c | preservation / bring her mother to town | a border stone moved, a village burned | Reckoning · *Schism* | heart, player, guild (hall) | "the line I copied burned my home"; the guild judges its own steering |
| W2-A The Last of the Four | W2a | ambition / get the son off the dig (H6) | a "restored" claim stands for want of a witness | Passage · *Unwilling passenger* | guild (hall), player, guild (members) | a grandmother's word put her grandson in a killer's path; courage against fear |
| W2-B Sold by the Wayhouse | W2b | ambition / be taken back onto the roll | the Wayhouse ruled the seller of its own lodgings | Passage · *Fugitive* | guild (members), player | the woman who writes for those who can't sold the halls' name to keep doing it; saving the halls means confessing |
| W3-A Every Family Its Own File | W3a | greed / buy his own cart | a town sold its own files; the forger free | Reckoning · *Swindle* | heart, player, guild (members) | people who can't write paying to buy back their own words |
| W3-B The Copy House Across the Street | W3c | greed / finish her concordance | a lord's court rolls go to the Night Market | Acquire · *Won't sell* | heart, player | a half-blind keeper's ear against a forger's cheaper hand; hire the thief? |
| W4-A The Stalls' Oath | W4c | greed / bring his daughter home with a dowry | fair night: every empty house robbed | Dispute · *Broken agreement* | player, guild (hall), guild (leader) | does the promise the stalls make bind the halls behind them? |
| W4-B Who Is Poor | W4a | greed / clear his father's debt for his sister | quarter day: the poor called in and bonded | Rescue · *Bound by debt* | heart, player, guild (leader) | a dead copyist's son paying his father's debt while his poverty is sold |
| W5-A The Letters Never Collected | W5a | cover-up / find someone who walked with his mother | the oldest letters burned at dawn | Acquire · *Copy it before it burns* | guild (members), player | a son's unanswered letters swept up in a cover-up of the guild's oldest compromise |
| W5-B The Last Reader | W5c | cover-up / read the whole roll, once | the roll burned and its reader made to recant | Rescue · *Imprisoned unjustly* | guild (leader), guild (outlawed) | a reader who won't be saved from saying what she read |
| W6-A The Copies Sent for Mending | W6a | preservation / learn to read what she copies | the last outside copy lost; the court hears only the leader's reading | Passage · *Smuggling* | player, heart | smuggling the truth out of your own guild, against your own head |
| W6-B The Archive Nobody Copied | W6b | preservation / copy an archive herself | an archive burns uncopied; the halls decide a succession | Reckoning · *Betrayal* | guild (hall), player, player | the leader keeps the letter of the code and betrays its keeper |
| W7-A The Court's New Clerk | W7b | revenge / see her friend restored | the court refuses all Writers' copies | Dispute · *Public offense* | player, player | an old friend's silence, twenty years on, on the other side of a hearing |
| W7-B The Apprentice's Hand | W7c | revenge / be sworn a copyist | the Wayhouse condemns an innocent hand again | Reckoning · *Ruin from above* | heart, player | a teacher who loved and used her pupil; the guild repeats its injustice |
| W8-A The Wall Through the Spring | W8a | desperation / clear his debt, keep his pasture | a wall on a false line cuts a village from its spring | Threat · *Encroachment* | heart, player | building the wall that ruins you because it pays; fear sold as safety |
| W8-B The Mother's Journal | W8c | desperation / have his mother's journal read | the levy voted on redrawn maps | Passage · *Samples from the green* | player, heart | a son carrying the mother he lost through the line of the man who sent her |

**Pairs checked for closeness, and what was changed:**
- **W1-A / W1-B** (same enemy, both letters through the Wall): different objects (debts against a peace; readings against a stone), payloads (a truce lapses; a village burns), crises and hearts. Kept.
- **W2-A / W2-B** (same enemy; draft 1 shared the claim scheme, the lord and the valleys, and the review called W2-B a pale twin): draft 2 rewrites W2-B. W2-A is a father whose son is the last witness to a murderous claim; W2-B has no claim, no murders and no valleys: a Free Lord's suit makes the Wayhouse a seller of its own lodgings, and the heart's guilt is the core. Fresh stories keeps one lord per sample set, so they never share a lord either.
- **W5-A / W5-B** (same old debt): first draft both used a Highcrossing writ in the crisis. W5-B now holds its reader at the Mount on the house's own authority, and spends its writ on the outlawing, if that blow is dealt. Blow sets made disjoint.
- **W6-A / W6-B** (same narrow per-enemy blow list): different targets lead each list (player first in W6-A, the guild's hall in W6-B); different crisis families (Passage, Reckoning).
- **W7-A / W7-B** (false lines in the heart's copies, both): W7-A is a public hearing about the guild's commission, with the enemy's old friend as heart; W7-B is the Wayhouse repeating its own injustice on her pupil. Draft 2: the old friend is W7-A's alone (W7-B's ally is the hearing-scribe), and the blow sets differ (W7-A: player stripped, player framed; W7-B: heart estranged, player framed).
- **W3-A / W4-A / W8-A** (all three hearts are carters, H3, the kit's only client type): different variants and wants (a cart, a dowry, a pasture), and different relations to the scheme (victim of a sale; victim of a theft; the scheme's own hired hand).
- **Reveals:** three spines first had an "estranged pair, one is the knower" reveal; W8-A changed (Acquire *Won't sell*), and two remain (W5-B, W7-A).
- **Hearts across the set** (draft 2): H4 (old clerks) is down to three spines, and W2-A's H6 holder is a new kind of heart; hearts are cast, so the sex varies.
