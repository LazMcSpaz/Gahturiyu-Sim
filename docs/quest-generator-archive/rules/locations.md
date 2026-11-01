# Quests and Locations — Draft (v3.4)

How quests are pinned to the map, how many each location gets, and which few quests run on a clock. This builds on `faction-places.md` (fixed places, holdings, presences) and `worldgen-requirements.md` (what the land provides).

## 1. One world, budgeted by location
- **Quests are generated once per world, after the land and every place have been placed.** The generator reads a **map summary**:
  - territories and who holds them
  - regions
  - places, with their kind, size and presences
  - travel time between places
- **There are two passes:**

| Pass | Runs | Makes |
|---|---|---|
| **World pass** | once, top-down | the main story, its Return, minor quests and fallout; one **faction story** per joinable faction (`quest-faction-stories.md`) |
| **Location pass** | once per location, after the world pass | that location's side quests: echoes, independents, linked pairs, **hall jobs** and other one-off faction-given jobs (§4) |

- **Shapes are generated up front.** Shape cards are small data, so a few hundred are cheap. Story text (descriptions, dialogue) is filled in later, by the story layer, when the player gets close.
- **Both passes share one registry.**
  - **What location-pass cards may depend on:** the main story's **hook harm**, ordinary public world facts, and **public plants on story cards** (main or faction) **in the same region**, in time order (§2, *Settlement context*, Location dependencies). Such a plant is treated like the hook harm.
  - Anything that needs a later move goes through fallout (`quest-fallout.md`), so the delete test and the timing rules stay in one place. **One exception:** a guild's two move-made hall jobs (§4, *Hall jobs*, v7).
  - **Later moves still show:** they reach a town as its **context** (§2, *Settlement context*), which a card may read but which never gates it.
- **The location quotas (§3) replace assembly §10's echo, independent and linked-pair counts.**

## 2. New card fields
| Field | What it holds |
|---|---|
| `offered_at` | the place where the player picks the card up: the requester's home, or a faction presence for faction-given cards (§4) |
| `place` | where it happens (unchanged). It may differ from `offered_at`. |
| `giver` | the person who hands it over, if that isn't the requester (a contact, a ladder-keeper) |
| `giver_faction` | the faction whose profile shaped the card, or none |
| `clock` | none, or {`checkpoint`, `world_ending`} (§6) |

- **Not uses.** `offered_at` and `giver` don't count toward the reuse cap or once-per-family. They're where and how the card is found, not slots.
- **Settlements are containers.** A map settlement holds many sites (a quay, a mill, a house, a shrine), like a small region.
  - A card's `place` is normally a **site inside it**: a registry place with `at: <settlement>`.
  - The settlement itself fills `place` only when the card is about the whole settlement (a Threat to the town, a Siege). Those are uses, and the cap of 3 applies.
  - Sites follow the normal reuse rules.
  - **A generated holding's landmark** (its rolled landmark, trade and hook) is made a registry place when the map is placed, before any quest. Quests reuse it, so two quests never invent the same landmark twice. *(Without this, the reuse cap of 3 made the location quotas impossible.)*
- **Travel rule:** at most one side quest in five sends the player farther than **near** (about a day). This keeps most side quests local, and makes the far-sending ones feel like leads.

### Settlement context
So towns can see what the big stories did to them.
- **Every settlement carries a `context` list** (the field is in `quest-card-schema.md`): the **public** facts that touched it. **Each context fact carries `when`:** a move id, a faction-story position (`<story>:<phase or chapter>`), or a relative day.
- **Every pass writes it, as it makes its cards:**
  - **main story:** its moves and their fallout. A region-scale move reaches every settlement in its region; a realm-scale move reaches every settlement. Every town a fallout card is pinned in counts as touched;
  - **guild planner:** every guild move and phase-3 public event (a writ, a seizure, a levy, a public killing), in every settlement it reaches (faction-stories §12);
  - **public plants** on story cards pinned there;
  - **court state:** public facts about the town from the live tensions (casting §5);
  - **clocks:** a clock's ending, as a state fact, with `when` = its checkpoint (§6, *Clocks*).
- **Secret facts never enter context.**
- **Sites have state too.** A public change to a site (a licence revoked, a yard occupied, a shrine closed) is a **state fact on the site**, with `when`.
  - Every card at that site whose time can fall after `when` reads it, with a `before` face (below). Story cards too.
  - A card that can't hold under either order (before the change and after it) is pinned at another site when it is placed. The planner checks every story's cards at a site against the state facts other stories write there.
- **Never a gate.** Context never enters `depends_on` and never decides when a card is offered. The delete test and fallout's timing rules are untouched. (One exception: a clock's ending withdraws the cards that can't hold after it, §6.)
- **Time order: how a card reads it.** A card may name a context fact in its pressure or place tie.
  - It lists the fact in `reads_context` with a **`before`** face: how the card reads while the fact isn't true yet. The card shows the fact only once it's true (rulebook, *Card text only says what's true then*).
  - `before` is left empty only when the fact is certainly true before the card can be offered.
  - Fact ids and notes never appear in what the player reads.
- **Location dependencies.** A location card may `depends_on` a same-region public plant only when that plant's card is offered before it (time order), or the plant was made at or before the main story's Hook. Anything later reaches the town as context, or through fallout.
- **Touched towns notice.** In every settlement a story touched (fallout pins and guild moves included), at least one location card reads its context: a town on the salt road grumbles about salt prices; a Marches village hears of the levy.

## 3. The location pass

### How many side quests
| Location | Side quests |
|---|---|
| village or generated holding | 1–2 |
| town | 3–5 |
| large town | 6–10 |
| great town (Top-standing) | 10–15 |
| presence or wild site **out in the land** | 0–1 |

- **Presences inside a town** (a hall, a contact, a parish shrine) share the town's count; they don't add their own.
- **The count includes** cards the world pass already pinned there (main chapters, fallout, faction-story cards). The location pass fills only the remainder, with a floor of 1 for towns and larger.
- **Hall jobs don't count toward it** (v6b audit decision). The quota is the town's side quests besides its hall jobs (§4, *Hall jobs*). Hall jobs still count under every per-place cap below: themes, families, premises, hooks, landmarks.
- **The scope cap still applies:** side quests stay at community scope or smaller.

### What the place leans toward
A location's family weights come from what's there. These are weights, not limits.

| Feature at the location | Leans toward |
|---|---|
| a market, a port, a trade yard | Acquire, Dispute (*Broken agreement*), Reckoning (*Swindle*), Investigate (*Watch them*: a rival's yard) |
| a road junction, a toll, a ferry | Passage, Threat (*Raiders*) |
| farms, mills, works | Restore, Threat (*Sickness or blight*, *Encroachment*), Dispute (*Shared resource*) |
| an old house, a hall of records | Dispute (*Contested inheritance*), Investigate, Restore (*Lost good name*) |
| a jail, a keep, a lord's seat | Rescue (*Imprisoned unjustly*), Reckoning (*Ruin from above*), Investigate (*Watch them*) |
| a ring, a fair, a challenge stone | Contest |
| the Overgrowth's edge | Threat (*Go and look* among them), Rescue, Restore (*Lost livelihood*) |
| a ruin, a lair, a wild site or camp within near reach | Threat (*Go and look*) *(Claude)* |
| a dig, a quarry, a boundary, a survey line, an old inscription | Acquire (*Take the readings*) *(Claude)* |
| a faction presence | that faction's profile (§4) |

- **The strongest two only.** Leanings come from a location's two strongest features. Weaker features don't stack on top (a great town's market and its keep lean; its old house, records hall and fair don't add more).
- **Leanings move on.** When a feature's leaning points only at premises already at their world cap (rulebook, *World ledger*), that leaning moves to the place's next feature. A lord's seat whose *Imprisoned unjustly* and *Ruin from above* are both capped gives its lean to the place's market or mill.
- **Threat in towns.** Every town and larger also leans toward Threat, set at its edge: *Raiders* on the road out, *Sickness or blight* in the wells or stores, a beast at the outskirts (*Protected beast*), a lord's works next door (*Encroachment*). This lean always applies, beside the strongest two.
  - The place-fit check (rulebook, *Premise must fit the place*) counts what lies **within near reach** of a town (a raid camp, a preserve, a lord's quarry), not only its own sites.

### The town's own features first (v7, A9)
Towns of one kind lean alike, so the ports read the same. So:
- **Double weight.** A premise that this town's **own** landmark, trade, hook or site can fill rolls at double weight.
- **Each feature once before any twice.** Every one of the town's features is a card's `place_tie` once before any is used a second time (the per-place caps below still apply).
- **Check:** each card's `place_tie` names something of its own town, never a thing every town of its kind has.

### The enemy seen in passing (v7, A7)
Suspicion needs a face seen before it matters.
- In each region a story crosses (main or guild), **at least 2 side cards** cast someone from the **enemy's faction** in an **innocent public role**: a client, a rival bidder, a buyer at the fair.
- **Never someone who acts for the enemy, and never a secret.** They have no link to the story's cast, and the card states nothing hidden about them (casting §6, *story limits*).
- These cards open **before the story's reveal** (the main story's revelation, or a guild story's reveal), so the player meets the faction before learning what it hides.
- For a guild story with a *Within* enemy, the enemy's faction is the guild's own sub-faction or circle the kit names *(Claude)*.

### Spacing along the road (v7, A1)
Caps count how often, not how close together. Eight cards with one shape can still fall on one road.
- **Two cards within about 2 days' travel** of each other (by the map) share **no shape, no twist, and no family + premise**, unless they are one chain (a chapter, a linked pair, a story's own cards).
- The location pass checks each new card against every card already pinned within 2 days, story cards included. A card that clashes rerolls the clashing part (twist, then truth row, then premise). Twist *none* and confirming rows (no shape) never clash.
- **Stricter than the per-place caps** below inside one town: those caps still bound a chain's own cards.
- *(Claude)* If the rerolls run out in a crowded stretch, the card keeps the clash with the farthest card it can, and the check logs it; it is never dropped for this alone.
- **Check:** a script tests every card pair within 2 days on the map. It should find none.

### Per-place caps
These count **every** card pinned at the settlement or its sites, story cards and hall jobs included. If story cards already reach a cap, the location pass adds no more of that kind; it never moves a story card.
- **Hooks:** a place's hook (fixed or generated) is used by at most **2** cards.
- **Landmarks:** a landmark is the focus of at most **2** cards. It may appear in passing on more.
- **Themes:** one theme (what a card turns on: lineage, bondage, a toll, salt) on at most **3** cards per settlement, and on at most **a third** of a region's cards.
  - **A theme is more than its word.** The cap also counts **the harm's shape** (water spoiled from upstream, a bystander blamed: rulebook, *Shapes*) and **the place's trade** (the Mount's protection tithe). Two cards that share any one of these share a theme.
- **Ways:** no two cards of the same family at one settlement roll the same set of way kinds.
- **Families:** no family above **30%** of a settlement's cards, rounded up. A village with two cards has two families.
- **Premises:** no premise more than **twice** per settlement.
- **World-wide counts** belong to the one world ledger (rulebook, *World ledger*). The location pass reads and writes it like every other pass. Passes run in sequence, so each reads the ledger as the last one left it.

### Neighbours
- **Linked pairs** may span two locations that are **near** each other, or two **sites in one settlement**. That is how one town's quest points you to the next, or to the next street.
- **Echoes** prefer people from a main card pinned within two days' travel.

## 4. One-off faction-given jobs and hall jobs
- A one-off job offered at a faction presence (a hall, a contact, a shrine, a field site) is **faction-given**. It uses that faction's **profile** (`quest-faction-stories.md` §2): its weights, signature premises, methods, code, vocabulary, motives and twists.
  - Guild profiles are in `quest-guild-kits.md`; minor-faction profiles are in `faction-places.md`, *Minor-faction profiles*.
- **Signature quota.** The profile's "about 1 card in 3" signature rule is a **quota per presence**, not a weight. Of the cards a presence offers, one in every three, rounded down, uses a signature premise. So its first two cards are free (1–2 cards: none needed; 3–5: one; 6–8: two). If no signature premise passes the place's requirements, the card rolls normally and the miss is logged.
  - **Fixed places that give faction jobs** (the Soft Hollow, the Green Wall) follow the same quota.
- **Public front.** A presence with a secret backer, or a hidden presence, gives its jobs under its public front (`faction-places.md` §5b, *Rolls read secrets*).
- **Share:** a presence out in the land gives its one card. A joinable faction's halls give their **hall jobs** (below). A minor faction's presence in a town gives about a third of the town's quota left after story cards, not counting hall jobs (§3).
- **One-off jobs are side quests.** All side-quest rules apply, including the community scope cap. Faction-story cards are a separate relatedness value with their own rules.

### Hall jobs (Laz, v6b: the guild ladder)
The one-off jobs of the six joinable stories (the five guilds and the Holy House). They count toward the player's rank in that guild (mechanics §9, *The guild ladder*).
- **How many:** about **9** per guild story (`hall_jobs_per_guild`). The location pass makes them with the guild's profile (§4 above), at its halls and contacts. The signature quota applies per presence, as for any faction-given job.
- **Where:**
  - at least **half** of each guild's hall jobs are at its halls in large or great towns; the rest at its contacts and field presences;
  - **every large or great town offers hall jobs from at least 3 guilds;**
  - they don't count toward the town's side-quest quota, but they do count under its per-place caps (§3; v6b audit decision). A town whose caps leave no room sends the guild's next hall job to another of its halls *(Claude)*.
- **Rank bands.** Each hall job carries a `rank` band: **early**, **middle** or **late**, about 3 each. When each band opens is mechanics §9.
  - Later bands carry higher stakes *(Claude)*: early jobs are household or community scope, for Low and Middle clients; middle and late ones are community scope, with more at risk (a higher pressure, a Middle or High client, the guild's own good name), and lean to the signature premises.
  - Every hall job stays a side quest: **community scope or smaller**.
- **Never clocked, never lost.** A hall job that closes unresolved is made again at another of the guild's halls, in the same band (mechanics §9, *The guild ladder*).
- **Simple tests.** They may be plain (rulebook, *No blank card*), inside the world's plain share for side cards.
- **No story secrets.** A hall job never uses its guild story's cast, plants or revelation: no story cast member fills a slot or gives it, and it never `depends_on` a plant of that story.
- **They read the story's public moves** through context, like any card (§2, *Settlement context*). After a hall is seized, jobs at the guild's other halls mention it. A hall job can be taken before or after a move, so each context fact it names has a `before` face.
- **Some hall jobs grow from the story's moves** (v7, A6). Without this, the nine jobs feel like a separate game.
  - **From the middle band on, each band has one hall job that reads the latest move** through context (with its `before` face).
  - **Two hall jobs are made by a move:** one middle-band job made by chapter 1's move, and one late-band job made by chapter 2's blow. Their premise exists because of the move: poached clients want their old contracts honoured; members thrown out need their tools back.
  - A job made by a move `depends_on` that move's **public** fact, and opens only once the move is a fact and its band is open (mechanics §3). This is the one hall-job exception to *Location dependencies* (§2). It is still never clocked.
  - **No story cast, plants or secrets** in them, as for every hall job (above).
  - **Check:** by script, 2 such jobs per guild, with no story cast in them.
  - The ladder still has no dead end: enough unmade jobs stay open at every gate (mechanics §9) *(Claude: checked against the default gates)*.
- **They count in the world ledger** like any card (rulebook, *World ledger*), and in the plain share.
- **Planned before the location pass** *(Claude)*: one hall-job plan deals every guild's jobs to its presences, with their bands, so the town and band rules hold before any location is filled (§7, step 3).

## 5. Pinning stories to the map

### Main story
**Rules for every arc:**
1. **The hook** is near the player's start.
2. **Siblings spread out:** chapters in the same layer are in different places.
3. **The Return** is at the heart's home community.
4. **Never at a secret place.** No main card is pinned at a hidden-lore place (the Womb, the Heart) or any place whose existence is a secret fact.
5. **The climax's place is an earlier landmark** (main-quest §5). The table's *Climax* column says **which** landmark. For Rising and Ripple it's a seat, which must first appear on an earlier card: as a non-turn card's place, or as a slot (main-quest §6). One rule, three docs.
6. **Spread:** no single settlement hosts more than **5** main-story cards. This counts **every** main-story card pinned at the settlement or any of its sites: hook cards, chapter cards on every path, the climax, the Return, and the main story's minor quests. (Fallout isn't main-story; fallout rule 4 keeps it off main places anyway.) Story cards count toward a place's quota, but a quota is a floor for the location pass, never a reason to pile story cards in one town. The checker enforces this cap. If a forced minor quest (the revelation's destination, a heart-thread card) would push a full settlement over the cap, it goes to a site or settlement **near** it instead.

**Rules that depend on the arc:**

| Arc | Journey rule | Territory rule | Climax |
|---|---|---|---|
| **Rising** | each layer's chapters sit farther than near from the previous layer's turn-card place | the path touches at least 3 territories | at a faction seat |
| **Ripple** | the same | at least 3 territories; on any one path, each trials chapter is in a different **region** (per path, as main-quest §6) | at the antagonist faction's seat (main-quest §6) |
| **Inward** | community layers: a journey apart. Household layers are exempt. | stays inside one faction's territory | the heart's home (household scope) |
| **Deep** | community layers: a journey apart. Household layers are exempt. | at least 2 territories | a Middle-standing community landmark on remote ground: high ground, the green's edge, or far from roads |

- **Journeys have an upper bound.** "Farther than near" and "a journey apart" mean more than about a day, but no more than about **3 days' travel**: each chapter's opening card sits within that of the turn card of every chapter linked into it (main-quest, *Layers start near where the last one ended*). **Exempt:** Ripple, whose journey is the point, and the Return, which goes home wherever the climax was.

**How each arc looks on the map:**
- **Rising** climbs from a village, to a regional town, to a seat.
- **Ripple** spreads outward from the home region.
- **Inward** goes deeper into one faction's places.
- **Deep** heads toward remote ground, without ever reaching a secret place.

### Faction stories
*The rules below are a summary; their home is faction-stories §6 and §8b (v6b: guilds share the big towns).*
- **Routine cards and the reveal** are hall work: given at the guild's **halls in large or great towns**, across at least **3** of those towns. Short steps don't bind them.
- **Story chapters** happen at **field sites**. At least one chapter's opening card is a journey away from the card before it (the chapter boundary).
- **Short steps inside a chapter:** each card is within about 1 day of the card before it (faction-stories, Story craft).
- One phase-3 card may sit in a shared town instead.
- **The crisis** is at the faction's **headquarters** (`faction-places.md` §4).
- **Hall jobs** are not story cards; they are placed by the hall-job plan (§4, *Hall jobs*).
- **All guild stories are placed together** by one placement plan (`quest-faction-stories.md` §8b), made before any guild card:
  - each story touches **3+ regions**, moving at phase and chapter boundaries, never inside a chapter;
  - **every pair of guild stories shares at least 2 towns** (both have a card there), and **every large or great town holds cards from at least 2 guild stories**;
  - rival and allied guilds **meet** on one card;
  - **every region has at least one story card,** main or guild, counting the main story's cards already pinned (faction-stories §8b.1, *Every region pulls*); no region has more than about a third of the guild-story cards.
- Guild-story cards count toward each town's per-place caps alongside every other card.

### Fallout
- A fallout card is pinned **within the move's reach,** and never at a place a main card uses (`quest-fallout.md` rule 4).
- A region-scope move's fallout cards go to **two different locations**.

## 6. Clocks (only a few cards)
**The world waits for the player.** Only a few cards carry a clock. This follows Laz's approved rule: a clocked card's outcome is picked at generation, and the story waits otherwise.

- **Checkpoints** are big steps of world time, such as seasons. The shape layer only names which checkpoint a card belongs to. **When checkpoints fire, and what an ending does beyond the state fact it writes (below), is mechanical-layer work** (rulebook §13).
- **Which cards can carry one:** only premises with a natural deadline:
  - Rescue *Kidnapped* and *Bound by debt*
  - Threat *Raiders* and *Sickness or blight*
  - Restore *Estrangement* (it already has a deadline)
  - Passage cards whose pressure is a deadline
  - Acquire *Passed on* (the thing is about to be sold on)
- **Never clocked:**
  - main-story cards
  - faction-story cards
  - minor quests
  - linked-pair cards
  - hall jobs (mechanics §9, *The guild ladder*)
- **How many:** at most **1 in 10** side quests.
- **The world's ending** (`world_ending`) is picked at generation: one of the card's own endings that needs no player at all.

| Family | Unattended endings |
|---|---|
| Rescue | moved elsewhere; rescue fails |
| Threat | community protected but the threat remains; community lost |
| Restore | left broken; abandoned |
| Passage | seized or lost; handed to the pursuer |
| Acquire | left with the holder |

  - The pick is weighted to suit the card. A cruel captor leans toward "moved elsewhere"; a weak community leans toward "community lost".
  - **The ending fits the card.** The pick is checked against the card's own pressure and truth: never an ending the card's text rules out (a card whose pressure says the buyer is coming to the sale can't end *left with the holder* without saying why).
  - **The ending writes state.** When its checkpoint passes, the `world_ending` writes a **state fact** on its place or person, with `when` = the checkpoint. It is context like any other: on a settlement's `context` list, or the site's state when the place is a site (*Settlement context*).
  - **Other cards at that place** (or using that person) whose time can fall after the checkpoint read it, with a `before` face, or are **withdrawn** when it passes. A village a clock ends as *lost* hosts no more quests there.
  - The shape layer records **which** ending and the state fact it writes; the mechanical layer decides **when** checkpoints fire and anything else the ending does. This is an amendment to rulebook §2.6 and §13.

### The main story keeps spreading
- The main story waits, but its **fallout** doesn't stop.
- **Late fallout:** each region- or realm-scope move gets **one extra fallout card**, tagged `checkpoint: k`.
  - It follows every fallout rule (this amends fallout rule 3's count).
  - It is generated with everything else.
  - When it becomes available is mechanical-layer work (fallout rule 17).
  - It's called *late fallout* to avoid clashing with fallout's existing *aftershock* rule (rule 15).

## 7. Generation order (with locations)
Casting runs **layer by layer** (§7a): each layer's specs are cast before the next layer is generated. Until casting exists, each card resolves its own slots in the same order (rulebook §3). The passes run **in sequence**, and each writes settlement context as it makes its cards (*Settlement context*).
1. Read the map summary.
   1b. **Archetypes and court state:** each leader archetype gets its filler (the canon person, or a generated one that replaces them: casting, *The filler replaces the canon person*); then casting makes the court state (§7a, layer 1).
2. **World pass:**
   1. Main story spine.
   2. Pin chapters by §5.
   3. Main cards, then the Return.
   4. Minor quests.
   5. Fallout, including late fallout.
   6. **Casting for the realm layer:** every spec from steps 1–5.
   7. Faction stories: kits for all, then one placement plan for all, then the cast for all (each slot scored against its card's planned region), then each story's cards (`quest-faction-stories.md` §12). The guild planner writes each move and phase-3 public event into settlement context.
   8. **Casting for the faction layer.**
3. **Hall-job plan** *(Claude)*: each guild's hall jobs dealt to its presences, with their bands (§4, *Hall jobs*). Then the **location pass,** largest locations first, filling each location's remainder (its dealt hall jobs first); then **casting for the community layer.**
4. Clocks, last, on eligible side cards. Each writes its ending's state fact and checks the other cards at its place (§6).
5. Background pass.

## 7a. Layers of power (Laz, 2026-10-26)
Generation runs **top-down by power**. Each layer may read and reuse what the layers above it made (their public facts and their people, under the normal reuse rules), never the reverse. Nothing lower may contradict anything higher. A lower layer may **add** to a higher layer's entries (facts, links, `dies_at`, its own endings' state), never change them (casting §1, *Add-only updates*). Each layer is cast before the next one is generated (§7).

| Layer | Makes | Reads |
|---|---|---|
| **0. World** | map, canon factions, and leader archetypes with their fillers | — |
| **1. Court state** (made by casting before the main story; the overview, `docs/quest-generator.md` §8.2) | which canon tensions are live this world (a Banner Lords' succession race, the Holy House split, the Tide Houses speaker's term…), as public facts naming the High people involved | 0 |
| **2. Realm** | the main story, its Return, minor quests and fallout | 0–1 |
| **3. Factions** | the faction stories. They reuse layer 2's High people freely in authority-only and background slots. A faction story may use **one** layer-2 person in a meaningful role (client, opposing, knower); that is its one crossing (`quest-faction-stories.md` §8). That person may be the story's *Above* enemy. They never use main-story cast or proxies, and never a main-story secret. | 0–2 |
| **4. Communities** | the location pass | 0–3 |
| **5. Households** | the background pass | 0–4 |

- **Why:** the powerful exist before the people who work for them, plot against them or petition them. A guild job can involve a lord whose feud the realm layer already set up, instead of inventing a second, parallel lord.
- **Fallout is made with its story.** Only echoes, linked pairs and location cards wait for cast people.

## 8. Parked
- When checkpoints fire, and what a clocked card's ending does to the world beyond the state fact it writes (§6).
- How the player finds quests: notice boards, overheard talk, being approached.
- Neutral wild sites (bandit camps, caves, abandoned forts) and their catalogue (`worldgen-requirements.md` §6).

## 9. Decisions made without Laz
1. **Up front vs on demand:** shapes are generated up front for the whole world; only story text waits for the player.
2. **Travel limits:**
   - at most 1 in 5 side quests sends the player farther than near;
   - main layers are a journey apart, per arc, and no more than about 3 days apart (main-quest, *Layers start near where the last one ended*; Ripple and the Return exempt).
3. **Clock budget:** at most 1 in 10 side quests carries a clock, and story cards never do.
4. **A minor faction's town presence gives about a third** of its town's side quests. (v6b: a guild's halls give its hall jobs instead.)
5. **Location-pass cards** depend only on the hook harm, public world facts and same-region public story plants offered before them (or made by the Hook). Later moves reach towns as context (never a dependency) or through fallout.
6. **Clock endings withdraw cards** that can't hold after them. It's the one way context touches when a card is offered, and only at the checkpoint, never at generation.
7. **Signature quota rounds down:** a presence's first two cards needn't be signature jobs, so a small presence doesn't repeat its one premise.
8. **Hall jobs (v6b):** a full town passes a hall job to another of the guild's halls; later bands carry higher stakes (scope, client standing, pressure); hall jobs are dealt by a plan before the location pass.
9. **Big-town capacity (v6b audit):** hall jobs don't count toward a town's side-quest quota, but they count under its per-place caps (themes, families, premises, hooks, landmarks). A minor faction's "about a third" is a third of the quota left after story cards, not counting hall jobs (§3, §4).
10. **v7 book rules:** a *Within* enemy's faction, for *The enemy seen in passing*, is the circle the kit names; a clashing card under *Spacing* rerolls its twist, then its truth row, then its premise; the two move-made hall jobs come from chapter 1's move and chapter 2's blow, and the ladder's default gates still leave enough jobs open (§3, §4).

## 10. Changelog
- **2026-10-27 (v3.4), v7: Laz-approved book rules (A1, A6, A7, A9, C1).**
  - §3: *Spacing along the road*: no shared shape, twist or family + premise within about 2 days, except inside one chain (A1).
  - §3: *The town's own features first*: double weight for premises the town's own features can fill, each feature used once before any twice (A9).
  - §3: *The enemy seen in passing*: 2 side cards per region a story crosses cast an innocent member of the enemy's faction, before the reveal (A7).
  - §4: hall jobs that read the latest move from the middle band on, and two made by moves, which wait for their move (A6).
  - §3: leanings for the field-work premises *Watch them*, *Go and look* and *Take the readings* (C1).
- **2026-10-27 (v3.3), v6: World C fixes.**
  - *Settlement context* completed: every pass writes it (fallout pins, guild moves and phase-3 events, court state, clock endings), each fact with `when`; sites carry state facts; `reads_context` with a `before` face; location dependencies in time order (CO-C1, CO-C3, PD-C2, L0B-4, L1B-5, L3B-3).
  - *Per-place caps*: themes count the harm's shape and the place's trade; no repeated way-kind set per family per settlement; leanings move on past capped premises (L0B-1, L1B, PD-C1).
  - Signature quota rounds down from a presence's third card; fixed places that give faction jobs follow it (L1B-3, L2B-3).
  - *Clocks*: the ending fits its card, writes a state fact, and other cards at the place read it or are withdrawn (PD-C5).
  - Faction stories cast after the placement plan; every region has a story card (GP1, region note).
  - Follow-up: arc journeys are capped at about 3 days, Ripple and the Return exempt (main-quest, *Layers start near where the last one ended*); the filler rule cited by name.
  - v6b: Laz's guild changes (27 Oct): §4 *Hall jobs* (about 9 per guild, half or more in large or great towns, every large or great town with hall jobs from at least 3 guilds, rank bands, no story secrets, reading context, counted in the ledger and the plain share); a hall-job plan before the location pass (§7); a guild's halls no longer give "a third" of a town's side quests (§4, §9); §5's faction-story summary follows faction-stories §8b's shared big towns.
  - v6b audit decision: hall jobs don't count toward a town's quota but do count under its per-place caps, and a minor faction's third is of the quota left after story cards (§3, §4, §9); hall jobs are never clocked and are made again elsewhere if they close unresolved, pointing to mechanics (§4, §6).
- **2026-10-26 (v3.2): audit fixes.**
  - Layer 3 reuses layer-2 High people freely only in authority-only and background slots; one may take a meaningful role as the story's single crossing, and may be its *Above* enemy (core H3).
  - Court state is made by casting before the main story, no longer "proposed"; casting runs layer by layer, with step 1b and a casting step after each layer in §7 (core M5, H6).
  - The faction journey is a chapter's opening card; steps inside a phase or chapter are short (core M2).
  - "Crossroads towns" are now "shared towns" (core improvement 1).
  - §7 now comes before §7a (core L5).
- **2026-10-26 (v3.1): guild stories meet** (Laz: medium overlap).
  - Faction-story pinning follows the guild placement plan: 3+ regions, shared crossroads, meetings, region coverage.
  - Generation order updated.
  - Removed the parked name-stem item; the rulebook now checks stems locally.
- **2026-10-26 (v3), v5: world-test fixes.** Same-region public story plants may be dependencies (§1); settlement `context` (§2); leanings stop at the strongest two, Threat leans in every town, per-place caps counting story cards, linked pairs inside one settlement (§3); signature quota per presence and public fronts (§4); the 5-main-card spread counts every main-story card (§5).
