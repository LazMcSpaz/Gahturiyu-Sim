# Casting: who everyone is — Draft (v1.3)

How the quest generator fills the roles its quests need with actual people, places and things. Quests never name anyone when they are made. They write **job descriptions** (specs), and the casting pass fills them. It runs one layer of power at a time, so the same lord can turn up in a feud, a guild job and a village's trouble, and the world feels connected rather than full of strangers.

- **Builds on:**
  - rulebook §3 (resolving a slot), §4 (reuse and creation), §6 (links), *Seats and offices have one holder*, *Name check*;
  - card-schema *Spec*;
  - locations §7a (layers of power);
  - faction-stories §8 (crossings), §8b (guild stories meet) and §12 (generation order);
  - main-quest, *Return casting*;
  - mechanics §14 (the grudge loop).
- **Plain words used here:**
  - **spec:** a job description a card writes for one role;
  - **filler:** whoever ends up filling it;
  - **pool:** everyone or everything that could fill it.

---

## 1. Where casting runs

Casting runs once per layer, after that layer's cards are made and before the next layer starts (locations §7a). **One exception:** a guild story's cast is cast after the placement plan and before its cards (layer 3 below); the cards' other specs are cast after them, as usual.

| Layer | What casting does there |
|---|---|
| **0. World** | Fills each **leader archetype** (§4) and populates the **pre-made classes**: lords for every lordship seat, council heads, and so on (§3). |
| **1. Court state** | Picks the world's **live tensions** (§5) and writes them down as facts and links. |
| **2. Realm** | Casts the main story, its Return, its minor quests and its fallout. |
| **3. Factions** | Casts every guild story, under the crossing and meeting rules (faction-stories §8, §8b). **Cast after placement:** the stories' cast slots are cast once the placement plan exists, each scored against its card's planned region (faction-stories §12). |
| **4. Communities** | Casts the location pass: side quests, echoes, linked pairs, hall jobs and other one-off faction jobs. |
| **5. Households** | Nothing. The background pass adds ordinary people directly, and quests never use them. |

- **A lower layer may use what a higher one made, never the reverse.**
- **Add-only updates.** A lower layer may **add** to a higher layer's entry: facts, links (§6, step 5), a `dies_at` the entry doesn't have yet, and `state` facts its own endings write. Each addition is recorded in its story's `registry_updates`; the location pass, whose cards belong to no story, records its additions in the quest set's top-level `registry_updates`, each with its card as `source` (card-schema). It never changes standing, office, home, faction or an existing fact.
- **During play**, cards that the world makes at runtime are cast one at a time by the same rules (§9).

---

## 2. Specs: the job description

A card writes one spec per role it needs filled. Fields are in card-schema, *Spec*.

### 2.1 Needs (hard requirements)
A filler that fails any of these is out.

| Need | Examples |
|---|---|
| **kind** | person, place, thing |
| **standing** | exact (*High*) or a range (*Low–Middle*) |
| **class** | *Banner Lord*, *house steward*, *Lantern captain*, *ring-keeper* (§3) |
| **archetype** | *the Lantern leader archetype* (§4): only the filler fits |
| **faction** | a faction or sub-faction; or *not* a faction |
| **people** | Roduro, Horaro, Qotiro or Ṭaḍoro, only where canon or the premise needs it |
| **sex** | only where canon fixes it (a Qotiro priest by trade is a woman) |
| **traits** | *lawless*, *holds legal authority*, *noble*, *has a secret* |
| **links** | to another role or spec: *creditor of the requester*, *kin of spec s-12*, *employee of the enemy* (secret) |
| **where** | lives or sits *at*, *near* (about a day), *in the region of* a place or another spec; or *a journey from* one |
| **office** | *holds the seat of X*: casting looks the holder up directly, with no search (rulebook, *Seats and offices have one holder*) |
| **timing** | *alive until move m3* (`dies_at`), or *alive at* this card's time |
| **state** | *free* (not held, fled, dead or captive at this card's time); *at home* |
| **must reuse** | the filler must already exist (a canon landmark, the heart's home, the only record) |

### 2.2 Prefers (soft wishes)
Each one met adds to the score (§6, step 3): *wealthy*, *old*, *a woman*, *already in a live tension*, *same people as the requester*, *owns a boat*.

### 2.3 Specs that point at each other
- A need may name another spec on the same card or the same story: *kin of s-12*.
- Casting treats specs linked this way as a **group**. It fills them so the link holds: it either finds two people already linked, or it creates one of them with the link (§7).

### 2.4 Place and thing specs
- **Places:**
  - kind, owner (a class or a spec), access, standing, *where*, required **affordances** (a cell, a back door, a strongroom: what the quest physically needs there; Lane B's detail tables build them on approach).
  - Settlements, landmarks and regions come from the map. They are always **must reuse**, because casting never creates a map place.
  - **Sites** inside a settlement (a yard, a back room, a hamlet's barn) may be created, but only after every fitting site already there has been tried (rulebook, *Sites*).
- **Things:** kind, holder (`held_by`), place, value.
  - "The only X" is **must reuse** once it exists (rulebook, *Things are on the record*).

---

## 3. Classes

A **class** is a kind of person the world has many of: *Banner Lord*, *Free Lord*, *house steward*, *Lantern captain*, *inquisitor*, *ladder-keeper*, *toll captain*, *wharf-master*. Each world generates its own members, so two feuding Banner Lords are different people every playthrough.

**Each class is one data row** (`data/quests/rules/classes.ron`):

| Field | What it holds |
|---|---|
| **standing** | e.g. High |
| **faction** | faction and sub-faction, or *any* |
| **traits** | always (*noble*, *holds legal authority*) and sometimes, with a share |
| **links** | the class's usual ties, made at creation (a steward is the employee of a house's lord) |
| **lives at** | the place kinds and owners they live at (a Free Lord lives at a Free Lord holding) |
| **office** | the seat or post they hold, if any. Holders follow rulebook *Seats and offices have one holder*. |
| **people and sex** | weights, or fixed where canon fixes them |
| **population** | *pre-made* (made at layer 0: one per seat or post on the map, such as every lordship's lord) or *on demand* (made only when a spec needs one, such as stewards and captains) |

- **Seats on the map get their holders at layer 0.** Every lordship, council seat, temple and guild headquarters is filled before any quest exists. A spec that needs "the lord of Breakwall" looks the holder up, never searches.
- **Guild halls and contacts in towns are not pre-made.** Their keepers (a hall head, a ladder-keeper, a contract-board veteran) are **on-demand** class members, made when a spec needs one.
- **One office per person, one holder per seat** (rulebook, *Seats and offices have one holder*). Co-leaders sharing a seat: the seat names one holder; the other holds a `title` at the same place.
- **A class gives creation its defaults:** a created Lantern captain has the class's standing, faction, traits and usual links.

---

## 4. Archetypes (canon leaders, made optional)

Each canon leader in `leaders.md` is an **archetype**: a role the world needs, with the canon person as its **default filler**. For example, *the Dueling Society co-founder who once killed in a duel*, default Kitim.

**Each archetype is one data row** (`data/quests/rules/archetypes.ron`):
- the role;
- the default (canon) person;
- **keeps:** the traits, secret facts and links that stories depend on (the killing, the secret Binder ties, the foil link to another archetype), **listed explicitly** in the row;
- the canon person's age band and voice;
- the peoples and sex the role allows;
- *guaranteed* yes/no.

- **Nothing is read from `leaders.md` at run time.** The data row carries everything; `leaders.md` is where the rows were written from.

**At layer 0, for each archetype:**
1. **Guaranteed archetypes always exist.** These are the roles the late-game secrets run through (`hidden-lore.md`): someone who has scouted the green (Hoqa's role), and someone guarding the Womb (Shihi's role). Any others are listed in the data after a pass over `hidden-lore.md`; that list is Laz's call (overview §16).
2. **Canon or generated:** one keyed roll per archetype, against the world setting **`canon_leaders`**:

| Setting | Share of archetypes that use the canon person |
|---|---|
| *always* | all |
| *mostly* (default) | about 70% |
| *sometimes* | about 35% |
| *never* | none |

3. **A generated filler** gets a new name (from the naming module, by people) and **keeps** everything the archetype lists. Its standing is the archetype's. Foil links between archetypes are kept whoever fills either side.
   - **The filler replaces the canon person.** In that world the canon person doesn't exist: the faction's `leaders`, seats, links and foils all point at the filler.
   - **Kin of generated fillers:** a filler of a canon person's kin keeps the canon relation (twin, adopted sister) and gets a family name shared with that kin. Canon names stay exempt from *kin share a family name* (rulebook, *Name check*).
4. **Either way the filler is pre-made** (rulebook §4): a generated emperor is legal.
5. **An archetype that isn't guaranteed may be left empty** in a world only when the setting allows it (a future option, *sparse*). Kit entries that need a missing archetype are filtered out before rolling (faction-stories §11).

---

## 5. Court state (layer 1)

Before the main story is made, casting picks the world's **live tensions**: the politics that quests will orbit.

- **Each tension is one data row** (`data/quests/rules/tensions.ron`):
  - the archetypes or classes involved (from the leaders' canon foils: the Banner Lords' succession race, the high priest against the sincere priestess, the Tide speaker's term running out);
  - the public facts it writes;
  - the links it adds (rival, ally, grudge);
  - which factions it touches.
- **Pick 3–4,** by keyed roll weighted by the data. Rules:
  - at most one tension per major faction. A tension counts against **every** major faction whose land or people it involves (a schism on First Peoples land counts for the First Peoples too);
  - a tension needs all its archetypes filled.
- **Each tension tags its people** (`tension: <id>`). Specs score them higher (§6), so a guild job's client is more likely to be a lord already caught up in the succession race than a stranger.
- **The main story reads court state** like any other layer-0 and layer-1 fact. It may build on a tension, but a tension is never the main story's revelation (it is public).

---

## 6. Casting one layer: the loop

Inputs: every spec written by this layer's cards, and the registry as the layers above left it.

**Step 1. Group.** Join specs that point at each other (§2.3) into groups. A spec on its own is a group of one.

**Step 2. Order: hardest first.** For each group, count its candidates after the filter (step 3a). Cast the group with the fewest first. Ties go to cards made earlier (story cards before side cards), then by spec id. This is the usual "most constrained first" order for puzzles like this; it keeps an easy spec from taking the only person a hard spec could use.

**Step 3. For each spec in the group:**

a. **Filter.** Everyone or everything fails who:
- misses a **need** (§2.1);
- would break the **reuse rules**: the 3-card cap, once per family, conflicting roles or states at this card's time, `dies_at` timing (rulebook §3–4);
- would break **separation**: money matters a standing apart (rulebook §5);
- would break **conflict of interest**: an arbiter linked to a party (rulebook §4);
- would break **story limits**:
  - main-story cast and proxies never fill a faction-story slot (faction-stories §8);
  - a faction story takes at most **one** layer-2 person in a meaningful role (its one crossing; they may be its *Above* enemy);
  - a guild-meeting visitor is never cast in either story (faction-stories §8b);
  - a **hall job** never takes its guild story's cast, in a slot or as its giver (locations §4, *Hall jobs*);
  - **the enemy seen in passing** (v7; locations §3, A7): a side card's spec for an innocent member of a story's enemy faction needs that faction (public membership) and **no link** to the story's cast; it never takes the enemy, the Hand, a proxy who acts for them, or anyone with a secret tie to the scheme;
  - **the player is never cast.** A guild door's slot for the one the blow falls on points straight at `p-player`, written by the card, not a spec; the card's other slots are cast as usual (mechanics §10; assembly, *Guild chapter-2 doors*);
- would break **names:** the local stem rule on this card and chain (rulebook, *Name check*);
- would break the **world ledger:** a High or Top person who appears only inside a truth or twist (the better buyer, the fixer above) fills such a role on one card per world (rulebook, *World ledger*).

b. **Score** each survivor (the numbers are dials in `data/quests/rules/casting.ron`):

| Factor | Points |
|---|---|
| already linked to someone on this card | +3 |
| on another card of the same story, outside the cast | +2 |
| same region as the card | +2 |
| lives near the card's place | +1 |
| in a live court tension (§5) | +2 |
| a grudge-holder against the player, for an opposing slot (runtime only, §9) | +4 |
| each preference met | +1 |
| each card already used on | −1 |
| each link that would have to be added to fit (§6, step 5) | −1 |

c. **Reuse or create.** Creating a new entry scores a flat **`create_score`** (default **1.5**). The best reuse wins only if it beats that.
- So a stranger is made unless an existing person fits *and* has some real reason to be here: a link, the region, the story.
- **Must-reuse specs** never create. Other specs can't create when the need rules it out: Top people, archetypes, seat holders, map places.

d. **Ties** are broken by a keyed roll (§8), never by list order.

**Step 4. Commit.**
- Set the spec's `filled_by`.
- Point the card's slot at the filler.
- Count the use.
- Write `cast_how` (*reused*, *created*, *looked up*) and the score.

**Step 5. Add the links the spec needs.** Allowed when the new link contradicts nothing already recorded:
- **Working and dealing links** (creditor and debtor, employer and employee, ally, rival, trade, grudge) may be added between two existing people.
- **Family, kin and spouse links** may be added only when one of the two is **created now**. They take the family name (rulebook, kin share a family name).
- **Secret links** are added only where the spec asks for one: a proxy's hidden tie to the Hand.
- An added link is recorded with `when` (before the story, unless the spec says otherwise), like any fact.

**Step 6. Stuck? Backtrack.** If a spec has no survivor and may not create:
1. Undo the most recent commit in this layer that removed one of its candidates (the person it would have used, now capped or in a conflicting role). Give that earlier spec its next-best choice, and try again.
2. **Budget:** at most **50** undo steps per layer (a dial). This keeps generation fast and its time predictable.

**Step 7. Still stuck? Repair the card.** The card that wrote the spec re-rolls the smallest piece that removes the need, in this order:
1. the slot's alternative, where the premise offers one ("an official, **or** their clerk");
2. the twist;
3. the premise, within the same family.

- Then casting runs again for that card only.
- Each card may be repaired at most **3** times.
- **A side card** that still can't be cast is dropped, and the location pass refills its quota.
- **A story card** never drops. If it can't be repaired, the generator logs it as a **generation error**. Tests must show this never happens on the standard island.

**Step 8. Fallbacks.** After the layer is cast, find every **reused person** who:
- fills a role on one card whose ending could change them (captive, fugitive, culprit, captor, a party who may die, a defector), and
- fills a role on a later card in the same chain, or on a lower layer's card.

This covers every card that can remove someone: blood bouts and deadly duels (the stake and both fighters), minors and fallout included. A person who carries a plant a later card needs stands in a bout that can turn deadly only if the plant is given to their fallback too.

For each one, choose a **fallback**:
- someone linked to them (kin, employee, ally) who passes the later card's spec;
- or, if nobody fits, create one with that link.

Record it on the person (`fallback`) and on the later card (`fallbacks`). This meets rulebook *Endings don't remove people a later card needs*.

**Step 9. Write the layer down.** Nothing in this layer changes again, except by lower layers' add-only updates (§1) and runtime play (§9).

---

## 7. Creating an entry

When a spec creates someone:
- **From the class** (§3): standing, faction, traits, usual links.
- **People:** from the spec if it names one. Otherwise a keyed roll weighted by the home community's mix of peoples (the sim's culture blend, once wired; until then, the map place's people).
- **Sex:** fixed if canon fixes it; otherwise a keyed roll. It shows only through the name (pronouns come from it), so it isn't stored as a field.
- **Age:** a band (young, grown, old) from a keyed roll, weighted by the class (lords lean grown and old; recruits lean young); an archetype keeps its canon age. A spec may need or prefer one.
- **Want and past** (v7, B1): a guild story's heart gets the `want` of her kit heart type and one `past` event from her spec, with its `when` (card schema, Person). Others leave them empty.
- **Voice:** a `manner` and a `habit`, each a keyed roll from the tables in `quest-voices.md` (the habit table is per people). Canon archetypes keep the voice in their data row (§4).
- **Name:** from the naming module, by people. It must pass the *Name check* (full name unique world-wide, stems checked locally, given names unique in the region among people on story cards, kin share a family name). On a clash, roll again with the next key.
- **Home:**
  - a settlement allowed by the spec's *where*, weighted by size;
  - a class that lives at a place kind (a Free Lord at a holding) gets one there, reusing a fitting holding before creating one.
- **Links:** those the spec needs (step 5), plus the class's usual ones.
- **Facts:** those the spec needs, such as a past event with its `when`.
- **The Return** reuses kin and people its story has already shown, and creates nobody an established person could fill (main-quest, *Return casting*).

A created place or thing works the same way, from its spec and its kind's defaults. Created sites sit inside a map settlement or beside a road.

---

## 8. Keyed rolls and data

- **Every roll is keyed by what is being decided** (sim rule 2):
  - archetype choice: `[seed, archetype id, "canon"]`;
  - tension picks: `[seed, "court", slot]`;
  - casting ties: `[seed, spec id, "tie"]`;
  - creation: `[seed, spec id, "make", field]`.
- **Lists are walked in id order,** never in hash-map order, so a world rebuilt from the same seed comes out the same. Adding an unrelated card doesn't change another card's casting, except where they compete for the same person (and that is decided by the fixed order in step 2).
- **Data files** (Lane A, `data/quests/rules/`):
  - `classes.ron`
  - `archetypes.ron`
  - `tensions.ron`
  - `casting.ron`: the scores in step 3b, `create_score`, the undo budget, the repair limit, and the `canon_leaders` default.

---

## 9. Casting during play

The mechanics layer can make new cards at runtime: fallout from a sim event, or a grudge-holder's return (mechanics §8, §14). Each one is cast **on its own, at once**, by the same loop with three differences:
- **The pool is the world as it is now:** the dead are dead, the fled are gone, and holders are whoever holds the seat today.
- **Grudge-holders** score +4 for opposing slots. This is how a person the player wronged comes back in a later quest. They return as the same person, with their own history; there is no ranks, promotions or hierarchy screen (mechanics §14).
- **Keys** include the event's id and time, so a runtime card's casting is the same however often the world is looked at (sim rule 1).

---

## 10. Checks after casting

The checker (Lane B ports it to Rust) confirms, per layer:
- **Specs:** every spec filled, or its card repaired or dropped; every slot points at an entry.
- **Reuse:** cap, once per family, conflicting roles and states, `dies_at` timing, separation, conflict of interest.
- **Story limits:** one crossing per faction story; guild-meeting visitors outside both casts; main cast never in a faction slot.
- **Seats:** one holder per seat; one office per person (canon excepted); no created person claims an occupied office.
- **Canon fillers:** no canon person is listed where a generated filler replaced them.
- **Names:** the *Name check*.
- **Fallbacks:** every person whose role another card could change, and who is needed later, has one.
- **Reuse rate,** reported, not failed: the share of non-cast story-card slots filled by reuse. A world with almost none feels unconnected; one with most feels small. **Aim for 25–45% across the world.** A batch in a remote region, with few people yet, may sit below it. It's tuned with `create_score`.

---

## 11. Later: wiring into the sim

The sim already has about 5,000 people with homes, households, jobs and peoples, and towns with real buildings. When the generator is wired in:
- **The pool should include the sim's townsfolk** for Low and Middle specs. A quest's smith is then a real smith who works at a real forge, and a created person joins a real household. This needs the sim agents' agreement, through the questions page.
- **Place specs map onto the sim's buildings and workplaces** (`Settlement::styles`, `layout`).
- **Created people must fit the sim's budget** of about 200 bytes per person (sim rule 19). Quest facts live in the quest registry, not on the sim's `Person`.
- **Until then,** the generator keeps its own registry, and the background pass stands in for townsfolk.

---

## 12. Decisions made without Laz
1. **`create_score` 1.5:** reuse needs a real reason (a link, the region, the story), or a stranger is made. The target reuse rate is 25–45% world-wide, tuned by this one dial.
2. **`canon_leaders` defaults to *mostly* (about 70%).** Laz said canon leaders may be optional ("why not?"); a share keeps the canon cast familiar while still varying worlds.
3. **Family links are added only to someone newly created,** so casting never rewrites an existing person's family.
4. **Hardest-first order, a bounded undo budget, then card repair.** These keep generation fast and its time predictable. A story card never drops.
5. **Seat holders are made at layer 0 and looked up, never searched.** This fixes World B's two lords of one keep.
6. **Court state is 3–4 tensions, at most one per major faction** (counting every faction whose land or people a tension involves). Enough politics to orbit, not so much that every quest is politics.
7. **Add-only updates:** lower layers add facts, links, `dies_at` and their own endings' state to higher entries, never change them. This lets a guild story kill a layer-0 lord without rewriting him.
8. **Hall and contact keepers are made on demand,** not pre-made with the seats: a hall or contact is a presence, not a seat.
9. **Where updates outside a story go:** the location pass writes its add-only updates to one quest-set-level `registry_updates` list, each with a `source` card. Stories keep their own list (the field the test tools read), so nothing existing moves.

## Changelog
- **2026-10-27, v1.3, v7: Laz-approved book rules:** the filter for *the enemy seen in passing*: the enemy's faction, no link to the cast, never someone who acts for the enemy (§6; locations §3, A7); creating a guild story's heart sets her `want` and `past` (§7; B1).
- **2026-10-27, v1.2, v6: World C fixes:**
  - a generated filler replaces the canon person (§4; CO-C4);
  - add-only updates, recorded as `registry_updates`: a story's own, or the quest set's top-level list for the location pass (§1, §12; GP2);
  - guild cast slots are cast after the placement plan and before the cards, scored against each card's planned region; the cards' other specs after them (§1; GP1);
  - archetype rows list `keeps`, age and voice explicitly; nothing read from `leaders.md` at run time (§4, §7; L0-2);
  - kin of generated fillers keep the canon relation and a shared family name (§4; L0-3);
  - hall and contact keepers are on demand; one office per person, co-leader titles (§3, §10; L0-4, L0-5);
  - a tension counts against every faction whose land or people it involves (§5; L0-6);
  - the reuse aim is world-wide (§10; L3B-4, L1B-6);
  - pointers: *Return casting* (main-quest), background people in truths (*World ledger*), fallbacks for bouts and plant carriers (*Endings don't remove people a later card needs*).
  - v6b: Laz's guild changes (27 Oct): hall jobs are cast at layer 4 and never take their story's cast; the player is never cast, and a player-target door points its slot straight at `p-player` (§1, §6); the grudge loop pointer is mechanics §14. v6b audit: §9's runtime pointer is mechanics §8 and §14 (was §16, the open questions); §6's meeting-visitor pointer names faction-stories §8b.
- **2026-10-26, v1.1:** creating a person now sets age band and voice (Lane B CR-B3, CR-B4).
- **2026-10-26, v1:** first draft, from the overview §8, Laz's decisions on deferred casting, classes, optional canon leaders and court state, and the world-test findings (two lords of one keep, people in two offices, fallbacks).
