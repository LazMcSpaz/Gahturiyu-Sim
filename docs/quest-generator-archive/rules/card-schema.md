# Card Schema — Locked (v1.12)

This is the exact shape of what the generator outputs. It's the **contract** for the mechanical layer (flags, state, progression) and the story layer (dialogue): both read these fields and nothing else. Field names are written in `code`, so they translate directly to data (JSON, Godot resources, Rust structs).

A generated **quest set** has these parts:
1. `registry`: the people, places, things, and factions the quests use.
2. `facts`: every fact, **stored once**. Entries and cards refer to facts by id; an entry's `facts` field is a list of fact ids.
3. `story`: the one main story's structure.
4. `faction_stories`: one per joinable faction.
5. `cards`: every quest card (main, minor, fallout, echo, independent, faction).
6. `specs`: the job descriptions cards write for slots the casting pass hasn't filled yet (*Spec* below; `quest-casting.md`).
7. `registry_updates`: add-only updates made by cards outside any story (the location pass, layer 4), each as in *Story* with a `source` card id. Stories keep their own (`story.registry_updates`, `faction_stories[].registry_updates`).
8. `map`: the land generator's summary, kept beside the output as input for the checkers: `regions` {`id`, `name`}, `territories` {`id`, `holder`, `sub`}, `places` {`id`, `name`, `kind`, `size`, `territory`, `region`, `x`, `y` (in days of travel), `standing`, `canon`, `owner_faction`, `quota` [least, most], `near`, `presences_in_town`, `presence_of`, `landmark`, `trade`, `hook`, `secret`}, and `roads` (each a list of place ids).

**In code** this is `src/questgen/data.rs` in the repo, field for field. A quest set is plain data written as JSON; every collection is ordered (lists, or maps sorted by key), so nothing depends on iteration order.

---

## 1. Registry entries

### Person
| Field | Type | Notes |
|---|---|---|
| `id` | id | |
| `name` | text | full name unique world-wide; given names and stems follow the rulebook's *Name check* (kin share a family name) |
| `standing` | Low / Middle / High / Top | Top only from the pre-made roster (archetype fillers count as pre-made) |
| `archetype` | archetype id or none | the leader archetype (`leaders.md`) this person fills, whether the canon person or a generated one |
| `class` | class name or none | the class they were made from (`quest-casting.md` §3) |
| `traits` | list | e.g. *lawless, noble, creditor, holds legal authority, has a secret* |
| `links` | list of {`type`, `to`, `as`, `visibility`, `when`} | link types and direction from rulebook §6; `visibility` public (default) or secret, e.g. a proxy's hidden tie to the Hand; `when`, for links casting added (`quest-casting.md` §6, step 5) |
| `faction` | {`id`, `sub`} or none | **public** membership only. A secret membership (a Rooted giver's oath, a hidden broker) is a secret fact on the person, not this field, until a revelation publishes it; text and dialogue read only this field, so they show the public front |
| `facts` | list of fact ids | see *Fact* below |
| `premade` | yes / no | Top people and archetype fillers (canon or generated) are pre-made |
| `dead` | yes / no | the dead are always newly created |
| `home` | place id or none | the person's home community (the Return and epilogues use it) |
| `dies_at` | move id or none | alive on cards before that move, dead after (rulebook §4) |
| `fallback` | person id or none | the backup chosen by casting for a reused person whose role another card could change |
| `people` | roduro / qotiro / horaro / tadoro | the sim calls it `race`; set by casting (`quest-casting.md` §7) |
| `age` | young / grown / old | a band relative to the person's people's lifespan, not years; rolled by casting, or the archetype's canon age |
| `want` | text or none | v7 (B1), guild-story hearts: something she seeks, never "not…". It comes from her kit heart type, shows on her routine card and once more later, and never decides a way (`quest-faction-stories.md`) |
| `past` | fact id or none | v7 (B1), guild-story hearts: one past event of her own, with `when` |
| `voice` | {`manner`, `habit`} | two slugs rolled by casting from the person's seed: `manner` from the short list in `quest-voices.md`, `habit` from that doc's per-people table. Dialogue picks lines by people, manner and age (`quest-dialogue-kit.md`). Pronouns come from the name (the sim's `names::gender`), so there is no sex field |

### Place
| Field | Type | Notes |
|---|---|---|
| `id`, `name` | | |
| `kind` | text | village, keep, ford, mill, toll-house… (routes: road, pass, ford, river, wilds; sea routes are parked while the sea is off-limits: rulebook, *Entry specs*) |
| `owner` | person id | fixed at generation |
| `access` | open / guarded / remote / fortified | |
| `standing` | Low / Middle / High / Top | |
| `region` | region id | needed for Ripple and for "region" scope |
| `tags` | list | *landmark*, *foundational* |
| `facts` | list of facts | |
| `holder` | person id or none | seats only (a keep, a hall): who holds it now. One holder; a change is a fact (rulebook, *Seats and offices have one holder*) |
| `titles` | list of {`title`, `holder`, `since`} | offices tied to this place with no building of their own (wharf-master, speaker); same one-holder rule |
| `context` | list of fact ids | settlements only: the public facts that touched it, each with `when` (`quest-locations.md`, *Settlement context*): main moves and their fallout (a town a fallout card is pinned in counts), guild moves and phase-3 public events, public story plants, court-state facts, and clock endings (as state facts). Cards may mention it (`reads_context`); it never gates availability and never enters `depends_on` |
| (site state) | in `facts` | sites only: a public change to a site (a licence revoked, a yard occupied, a clock's ending there) is a `state` fact in the site's `facts`, with `when`. Cards at the site read it like context (`reads_context`) |
| `at` | settlement id or none | sites only: the settlement a registry site sits in (`quest-locations.md` §2) |
| `descriptors` | list of slugs | what the detail generator builds from on approach: the class first as `class:<name>`, then slugs from `quest-detail-tables.md` §4 (`large`, `well_defended`, `river_fed`, `neglected`…) (`worldgen-requirements.md` §8) |
| `detail` | the built plan, or none | written by the detail pass the first time the player comes near, then never rebuilt (`quest-detail-tables.md`; sim rule 4) |
| `affordances` | list of {`tag`, `text`, `from_card`, `way`} | things a card's ways need the place to have. `tag` is a catalogue slug from `quest-detail-tables.md` §6 (`way_in:drain`, `hold:cell`, `record:ledger`), found by looking the rolled way up in that doc's way-to-tag mapping, then applying that row's rule where it offers a choice (read from the place's class, or a keyed roll on `[seed, card id, way, "tag"]`); `text` keeps the readable words (*a drain nobody guards*); `way` is the way's position in the card's pool. The detail generator builds from the tag |

### Thing
| Field | Type | Notes |
|---|---|---|
| `id`, `name` | | |
| `kind` | heirloom / document / weapon / relic / goods / tool | |
| `owner` | person id | |
| `value` | Low / Middle / High / Top | |
| `location` | place id | |
| `held_by` | person id | who has it now (may differ from `owner`) |
| `since` | card id or move id | when it came to `held_by` |
| `tags`, `facts` | | a thing a card calls "the only X" is created once and reused (rulebook, *Things are on the record*) |

### Faction (pre-made)
| Field | Type | Notes |
|---|---|---|
| `id`, `name`, `size` | | size: major / minor / guild |
| `parent` | faction id or none | |
| `leaders` | list of person ids | several factions are led by pairs |
| `seat` | place id | |
| `reach` | community / region / realm | |
| `want` | greed / revenge / cover-up / desperation / ambition / preservation | |
| `virtue`, `wound` | text | public |
| `hidden_side` | fact | secret |
| `relations` | list of {`type`: ally / rival / at war / trade / serves, `to`} | *trade*: dealings without loyalty (rulebook §6) |
| `facts` | list of facts | e.g. a reputation a faction story puts at stake |

### Fact (used inside every entry and card)
| Field | Type | Notes |
|---|---|---|
| `id` | id | |
| `text` | text | |
| `visibility` | public / secret | Truths are secret by default, but public where the family says so (Threat's Raiders, Protected beast and Encroachment why-here). |
| `published_by` | card ids, optional | revelation only: the cards that make it public |
| `kind` | past event / state / trait / plant / move | `state` = broken, sick, seized, captive, dead, fugitive, disgraced, unaware, fled, changed sides, new post. A state fact on a person is written by a shield (at generation) or an ending (applied by the mechanical layer); later cards read it (rulebook, *State is written down*) |
| `when` | relative day, move id, or faction-story position | required on past events (e.g. "40 days before the story", "after m-2") **and on context facts**: moves, faction-story events, clock endings and state changes at a site. A faction-story position is written `<story>:<phase or chapter>` (e.g. `fs-lantern:c2`) |
| `source` | card id, story, or world | where it was established (*world*: set by world generation, such as a shifted guild tie) |

### Spec (a slot not yet cast)
| Field | Type | Notes |
|---|---|---|
| `id` | id | |
| `card` | card id | the card that wrote it |
| `role` | role name | the slot it fills |
| `kind` | person / place / thing | |
| `layer` | 0–4 | the layer of power whose casting fills it (`quest-locations.md` §7a) |
| `needs` | list | hard requirements: kind, standing, class, archetype, faction, people, sex, traits, links (to a role or another spec), where, office, timing, state, must reuse (`quest-casting.md` §2.1) |
| `prefers` | list | soft wishes, each worth a point when met (`quest-casting.md` §2.2) |
| `group` | group id or none | specs that point at each other are cast together |
| `filled_by` | entry id or none | set by the casting pass; the card's slot then points at this entry |
| `cast_how` | reused / created / looked up | how it was filled |
| `score` | number | the winning score (for the checker and tuning) |

- Cards write specs; the casting pass fills them, layer by layer (rulebook §3; full rules in `quest-casting.md`). Until casting exists, a card resolves its own slots and writes no specs.

**Derived, not stored:** each entry's list of cards that use it. The checker computes it from card slots and uses it to enforce the cap and once-per-family.

---

## 2. Card (every quest)

### Identity and placement
| Field | Type | Notes |
|---|---|---|
| `id` | id | |
| `family` | one of the nine | |
| `premise` | text | including set pieces |
| `variant` | text or none | e.g. Siege *target* / *in the way* |
| `relatedness` | main / minor / fallout / echo / independent / faction | levels 4–0; *faction* = a faction-story card |
| `scope` | household / community / region / realm | |
| `place` | place id | where it happens |
| `offered_at` | place id | where the player picks it up (not a use) |
| `giver` | person id or none | who hands it over, if not the requester (not a use) |
| `giver_faction` | faction id or none | whose profile shaped the card |
| `signature` | yes / no | uses a faction signature premise |
| `clock` | none, or {`checkpoint`, `world_ending`} | side quests only (`quest-locations.md` §6) |

### Who
| Field | Type | Notes |
|---|---|---|
| `slots` | map of {role → entry id, fact id, or spec id} | every filled slot, required and optional; e.g. `requester`, `captive`, `captor`, `holder`, `wrongdoer`, `guide` |
| `opposing_slot` | role name | which slot is the opposition (assembly §5) |
| `fallbacks` | map of {slot role → person id} | filled by casting: who stands in if an earlier ending removes that slot's person (rulebook, *Endings don't remove people a later card needs*) |
| `reasons` | map of {slot role → reason} | every card fills `requester` with one row of the shared reason list that its family names (rulebook §7, v7); other slots (holder, party, opponent) from their family's own lists |
| `motive` | text or none | Rescue *hidden motive*; Investigate culprit motive |
| `factions` | list of {`faction`, `role`: member / backer / attacker / representative} | |

### What
| Field | Type | Notes |
|---|---|---|
| `truth` | a fact id (secret, except Threat's public why-here, which never stands in for a twist) or none | `truth` and `twist` are never both none (rulebook, *No blank card*) |
| `truth_row` | text or none | the truth's row, written `<Family or signature premise>: <row>`, exactly as the family doc's (or guild kit's) truth table names it: `Rescue: guilty`, `Threat: bad water`, `Copy it before it burns: already copied`. A row name used under two premises of one family is one row (`Contest: a way out`) |
| `shapes` | list of shape slugs | from rulebook *Shapes*: one for the truth's row (none for a confirming row), one for the twist if it has a shape; a shape the twist and row share is listed once |
| `plain` | yes / no | yes when the truth is a *confirming* row and the twist is *none* (rulebook, *No blank card*) |
| `pressure` | text | what's at risk |
| `pressure_level` | 1–3 or none | chapter cards and faction-story phase 3 cards (`quest-faction-stories.md`, *Chain and climb*) |
| `separation` | applies / exempt (reason) / n/a | |
| `twist` | {`name`, `group`} or none | group from the rulebook's twist groups table |
| `open_threads` | list of {`id`, `opens`: yes/no, `closes`: yes/no} | loose ends this card starts or ties off (e.g. the hook's lost thing); every opened one is closed by some card. Not `story.edges.thread`, which only names what links two nodes |

### Ways through
| Field | Type | Notes |
|---|---|---|
| `ways_pool` | list of {`text`, `kind`, `source`, `requires`, `contest`, `names_slot`, `price`, `pays`, `how`} | `source`: premise / family rule / ally / sacrifice / code_break / lesser_code_break (one chapter-2 card of a faction story, rulebook rule 3). `how`: optional, set only when the way's text reads otherwise than its kind (mechanics §14, v7 C5). `names_slot` and `price` are filled when a way names a person (route ways, helpers). `text` is filled from the card's own slots, plants or details (rulebook, *Ways name what's on the card*). `pays`: the plant fact id the way pays, or none (rulebook, *Plant-backed ways roll first*) |
| `ways_rolled` | list of positions in the pool | 2–3, covering at least 2 kinds; a 4th only on the climax or a crisis (rulebook, *Roll order*, forced ways overflow) |
| `ways_removed` | list of {`way`, `reason`} | for example, *fortified*, *target is Top*, *truth doesn't support it*, *code* |

### Endings and knowledge
| Field | Type | Notes |
|---|---|---|
| `possible_endings` | list of text, or {`text`, `state`: {`person`, `state`}, `costs`} | static; an ending that changes a person (fled, changed sides, disgraced, new post) carries the state it writes. `costs` (v7, A4): the slot role (or roles) who loses by this ending, from the family's defaults; required on turning cards, where at most one ending costs only the opposing side (rulebook rule 7) |
| `plants` | list of {`fact`, `type`, `document`, `knower`} | `type`: shared / unique / ally / fault line / lead / harm / quiet. `knower`: the person slot who can attest it (required for shared plants) |
| `details` | map | family-specific fields: `shield`, `demand`, `wrong`, `seal_type`, `cause`, `hard_part`, `bond`, `why_taken`, `red_herrings` (each with a reason, a clearing fact, and `already_blamed` yes/no), `holder_state` (e.g. unaware), `twist_detail`, `named_in_truth`, `arbiter_absent`, `player_position`, `requester_is` (when the requester is also a party or the wronged); `tell` (v7, A2: the one public fact that doesn't fit the told story, rulebook rule 8); `noted` (v7, A3: a harmless odd detail on a hall job, `quest-faction-stories.md` §6); field-work premises (v7, C1): `suspicion` (*Watch them*: what the requester fears), `sign` (*Go and look*: what the community has seen), `read_from` and `instrument` (*Take the readings*: what is read, and with what); Contest: `contest_kind`, `terms`, `ring_keeper`, `rides_on`, `oath`, `fix_type`, `champion` |
| `depends_on` | list of fact ids | every fact this card relies on; checked by the plant rule |
| `reads_context` | list of {`fact`, `before`} | every context fact the card's text uses (`quest-locations.md`, *Settlement context*). `before` is the card's plain reading for when that fact isn't true yet; null when the fact is certainly true before the card can be offered (rulebook, *Card text only says what's true then*). Unlike `depends_on`, it never gates the card |

### Main-story fields (main cards only)
| Field | Notes |
|---|---|
| `chapter` | chapter id |
| `position` | opening / middle / turn |
| `cast_present` | cast slots filled on this card |
| `proxies_present` | proxies on this card |
| `move` | the move id this card grows from |
| `door` | for Ordeal turn cards: which door |
| `landmark` | yes, on turn cards |

### Faction-story fields (faction cards only)
| Field | Notes |
|---|---|
| `faction_story` | the faction story id |
| `phase` | routine / reveal / story / crisis / epilogue |
| `chapter`, `position` | phase 3 only, as for main cards |
| `door` | chapter 2's turn card only: which door, written `<target>: <event> → <family>` (e.g. `player: framed → Investigate`; assembly, *Guild chapter-2 doors*) |
| `rank_gate` | number or none: the rank in this guild the player needs before the card opens (mechanics §9, *The guild ladder*). Set on every **beat** (each routine card after the first, the reveal, each chapter's opening card, the crisis), filled from the dials as the story cards before it plus `hall_jobs_wanted`; none on the joining card, chapter middle and turn cards and the epilogue |
| `odd_job` | yes / no |
| `private_side` | yes / no: draws on the faction's wound, hidden side or code |
| `faction_endings` | crisis only: the kit's steering labels |
| `guild_meeting` | {`guild`, `person`, `tie`} or none: another guild met on this card (`quest-faction-stories.md` §8b) |
| `shared_town` | yes / no: the card is set in a shared town, a settlement holding the headquarters or a hall of two or more guilds (`quest-faction-stories.md` §8b, *Shared towns*) |

### Side-quest fields
| Field | Notes |
|---|---|
| `attached_to` | minor: the chapter id |
| `advantage` | minor: {`fact`, `for_node`} |
| `source_move` | fallout: the move id |
| `move_fills` | fallout: which field the move fills (the delete test) |
| `face` | fallout: profit / resist / endure / enemy's own / mirror |
| `aftershock_of` | fallout: an earlier fallout card id, or none |
| `checkpoint` | late fallout: the checkpoint after which it becomes available (`quest-locations.md` §6), or none |
| `echo_of` | echo: {`person`, `from_card`} |
| `linked_pair` | {`with`, `order`: 1 or 2} or none |
| `theme` | any card: one short tag for what the card turns on (bondage, lineage, tolls…), read by the per-place theme cap (`quest-locations.md` §3) |
| `place_tie` | any card: how it is tied to its place's trade, hook or landmark (`quest-locations.md` §3, *Per-place caps*: hooks, landmarks, themes) |
| `hall_job` | {`guild`, `rank`: early / middle / late} or none: a hall job of that guild, which counts toward the player's rank there (`quest-locations.md` §4, *Hall jobs*; mechanics §9). `giver_faction` is the same guild |

---

## 3. Story (one per quest set)

| Field | Notes |
|---|---|
| `arc` | Rising / Ripple / Inward / Deep |
| `shape` | Above / Beside / Within: the **enemy shape** (where the enemy stands to the heart). Not a rulebook *Shape* (a card's `shapes`) |
| `L`, `W` | diamond size |
| `cast` | {`antagonist`, `want`, `stake`, `heart`, `fallback`, `hand`, `lieutenant`} |
| `factions` | {`antagonist_side`, `ally`, `fault_line`: {`person` or `sub`, `virtue violated`}} |
| `revelation` | {`fact`, `stored_on`: person id} |
| `moves` | list of {`id`, `layer`, `fact`, `scope`, `lever` (from faction-stories' lever list; the last move's lever, and every lever the main story uses on 2 or more moves, are reserved in the rulebook's *World ledger*), `faces` {`plain`, `despite_you`, `target`} (main-quest, *Wins stick*), `kind`: harm / move / ordeal event / last move, `harm_type` (hook only: works / livelihood / kin held / kin missing / kin dead / kin hunted / kin sick / claim challenged / bond broken), `event` (ordeal only: captive / disgraced / estranged; the main Ordeal's target is always the heart, Laz v6b)} |
| `layers` | list of {`layer`, `role`, `scope`, `node ids`} |
| `nodes` | list of {`id`, `layer`, `role`, `chapter`}. The Return is layer L+1 with role *return* |
| `edges` | list of {`from`, `to`, `thread`: cast id / proxy id / fact id} (what links two nodes: a person or fact carried from one to the next; it never needs closing, unlike `open_threads`) |
| `open_threads` | list of {`id`, `what`: thing / person / fact id, `opened_by`, `closed_by`}: loose ends (the hook's lost thing) and the card that closes each |
| `chapters` | list of {`id`, `node`, `role`, `problem` fact, `cards` in order, `region`, `settlement`, `face` (Ripple trials only: profit / resist / already won)} |
| `return` | chapter id |
| `return_variants` | map of {climax ending group → problem text}: the Return's text for each way the climax can end |
| `registry_updates` | list of {`entry`, `add_facts`, `add_links`, `set`}: what this story adds to entries a higher layer made (`quest-casting.md` §1). Add-only: `set` may hold only `dies_at` or a `state` fact id its own endings write; standing, office, home, faction and existing facts never change |

### Faction stories (one per joinable faction)
A list `faction_stories`, each with: `faction`, `kit_rolls` (entry id plus variant; since v6c the chapter-1 move and the 3 rolled faction endings too), `throughline` (v6c: the one sentence of the throughline test, "Because [enemy] wants [want], they [scheme]; they [move], then [blow], and it comes to a head when [crisis]"; for checkers and reviewers, never read by card text, since it names the secret scheme), `means_used` (v6c: {`move`, `blow`, `crisis`}, each the enemy's means tag that step is carried out with, its `uses`) *(Claude)*, `cast` {`enemy`, `shape` (the enemy shape, as in Story), `want`, `stake`, `heart`, `heart_type`, `fallback`, `hand`, `leader`, `fault_line`}, `revelation` {`fact`, `kind`, `stored_on`}, `moves` (kinds: move / ordeal event / last move; the ordeal event carries `target`: heart / player / guild, and `event`: for the heart captive / disgraced / estranged, for the player framed / stripped of rank / arrested, for the guild hall seized / members taken / outlawed / leader struck down; assembly, *Guild chapter-2 doors*), `phases` {`routine`, `reveal`, `chapters`, `crisis`, `epilogue`}, `faction_endings`, `moves[].lever`, `code_endings` (each {`ending`, `when`}, where `when` is a condition or *always*), `crossing`, `paired_with`, `placement` (card position → region and settlement; each routine card has its own), `meetings` (each {`card`, `guild`, `tie`, `person`, `role`, `site`}), `registry_updates` (as for the main story; cards outside any story use the quest set's top-level list). Faction cards also carry `guild_meeting` and `shared_town` (§2, faction-story fields). Full rules: `quest-faction-stories.md` §8b and §13.

**Kits aren't in the quest set** (v6c). Story kits and their entries are **rules data** in `data/quests/rules/` (Lane A), read by the roller, not generator output. This schema doesn't describe them. Their throughline tags are spec'd in `quest-faction-stories.md` §11 and §11a: `want`, `means` and `reach` on enemies; `hit` and `needs` on stakes; `needs` and `hits` on chapter-1 moves and crises; `fits` on faction endings; `thread` (and sometimes `needs`) on odd jobs. A quest set records only what was rolled (`kit_rolls`, `throughline`, `means_used`).

---

## 4. What the later layers read
- **Mechanical layer:**
  - flags come from `ways_pool`, `possible_endings` (with their `costs`), and `plants`
  - a resolved ending writes its `won_by` fact {`kind`, `how`} (mechanics §4, *How it was won*); the grudge loop reads it
  - state comes from facts with `kind = state`, and from the `state` an ending carries
  - open threads come from `story.open_threads`
  - progression comes from `story.edges`, `chapters` order, `linked_pair`, and for guilds `rank_gate` and `hall_job`
  - what advantages do comes from `advantage`
- **Story layer** (`quest-detail-tables.md`, `quest-dialogue-kit.md`, `quest-card-text.md`, `quest-voices.md`):
  - text and dialogue key off `family`, `premise`, `variant`, `twist`, `reasons`, `motive`, `truth`, `plain`, `reads_context` (a `before` and an after reading for each context fact the card names), `details`, `slots` (names and standing), `opposing_slot`, `ways_pool`, `possible_endings`, `plants`, `clock`, `phase` and `pressure`;
  - from each person: `people`, `age`, `voice`, `standing`, the **public** `faction`, and a heart's `want`;
  - the detail pass reads each place's `descriptors` and `affordances` and writes `detail`;
  - **the story layer's templates also fill** the Return's `return_variants` text (from the climax's ending rows) and each chapter's `problem` wording, so the generator never writes free prose.

## 5. Example (one card, abbreviated)
```json
{
  "id": "c-ordeal-3",
  "family": "Restore", "premise": "Lost good name", "relatedness": "main",
  "scope": "realm", "place": "pl-court-town",
  "slots": { "requester": "p-maro", "subject": "p-ilse" },
  "truth": "f-stain-planted",
  "truth_row": "Restore: false, spread by someone off the card", "shapes": ["rumour-planted"], "plain": false,
  "twist": null,
  "ways_pool": [
    { "text": "find the people who remember what really happened", "kind": "gather", "source": "premise" },
    { "text": "a public act that shows who she is now", "kind": "craft", "source": "premise" },
    { "text": "get someone respected to vouch", "kind": "persuade", "source": "premise" },
    { "text": "help her start fresh where nobody knows her", "kind": "substitute", "source": "premise" }
  ],
  "ways_rolled": [0, 2],
  "ways_removed": [{ "way": "answer it at the source", "reason": "cause is 'spread off the card'" }],
  "possible_endings": ["restored", "restored changed", "substitute", "left broken", "abandoned"],
  "chapter": "ch-ordeal", "position": "turn", "move": "m-ordeal",
  "door": "Disgraced → Restore", "cast_present": ["p-ilse"], "landmark": true
}
```

## Changelog
- **2026-10-27 (v1.12), v7: Laz-approved book rules:** endings carry `costs` (A4); `details.tell` (A2) and `details.noted` (A3, homed in faction-stories); `reasons.requester` on every card, from the shared list (A8); way `source` gains `lesser_code_break` (B5) and ways an optional `how` (C5); Person gets `want` and `past` for guild-story hearts (B1); details for the field-work premises: `suspicion`, `sign`, `read_from`, `instrument` (C1); the mechanical layer reads `costs` and writes `won_by`.
- **2026-10-27 (v1.11), v6: World C fixes:** cards get `truth_row`, `shapes` and `plain` (rulebook, *Shapes*, *No blank card*) and `reads_context` {`fact`, `before`}; Fact `when` is required on context facts too, and may be a faction-story position; Story and faction stories get add-only `registry_updates`, and the location pass uses a top-level list with a `source` card; `context` lists every writer and its `when`, and sites keep state facts in `facts`; the enemy `shape` is named apart from card `shapes`; sea routes parked until Laz opens the sea; `ways_rolled` may hold a 4th way on the climax or a crisis; the story layer reads `plain` and `reads_context`; the example's truth is now a non-confirming row, since a turn card is never plain; reserved levers follow the new *World ledger*. v6b: Laz's guild changes (27 Oct): side-quest field `hall_job` {`guild`, `rank`}; faction cards get `rank_gate` (from the dials) and `door`; a faction story's ordeal event carries `target` and `event`, and the main Ordeal's target is always the heart; the mechanical layer reads `rank_gate` and `hall_job`. v6b audit: `place_tie` points at locations' *Per-place caps* (there is no "place ties" section). v6c: faction stories get `throughline` (the one sentence) and `means_used`; `kit_rolls` includes the chapter-1 move and the rolled faction endings; kits and their tags (`want`, `means`, `reach`, `hit`, `hits`, `needs`, `fits`, `thread`) are rules data in `data/quests/rules/`, not part of the quest set.
- **2026-10-26 (v1.10), the lanes cross-check:** Person `faction` is public membership only (a secret one is a secret fact); Place gets `detail` (the built plan, never rebuilt); affordance tags are a lookup, then the row's rule; §4 lists everything the story layer reads, and names its templates as the fillers of `return_variants` and chapter problem text.
- **2026-10-26 (v1.9), Lane B's requests:** Person gets `people`, `age` (young / grown / old) and `voice` {`manner`, `habit`} (CR-B3, CR-B4); Place `descriptors` are slugs with the class first, and `affordances` are {`tag`, `text`, `from_card`, `way`} (CR-B1).
- **2026-10-26 (v1.8), matching the code (`src/questgen/data.rs`):** facts are stored once in a top-level `facts` list and referred to by id; the quest set's parts are listed in full, `faction_stories` and the `map` summary included; Person gets `class`; links get `when`; cards get `place_tie`; `open_threads` marks are {`id`, `opens`, `closes`}; chapters get `settlement`.
- **2026-10-26 (v1.7):** Spec gains `kind`, `layer`, `prefers`, `group`, `cast_how` and `score`; `needs` lists its vocabulary (`quest-casting.md` v1).
- **2026-10-26 (v1.6), audit fixes:** a fourth part, `specs`, and slots may point at a spec (casting fills them); Person gets `archetype`, archetype fillers are pre-made, `fallback` is chosen by casting; Place gets `at`, `descriptors` and `affordances`; Faction `relations` gains *trade*; Fact `source` may be *world*; `truth` may be Threat's public why-here, which never stands in for a twist; cards get `pressure_level`; story moves get `lever` and `faces`; story gets `return_variants`; faction cards' `crossroads` is renamed `shared_town` and listed with `guild_meeting` in the faction-story fields; `placement` is per card position; `meetings` get `site`; the example's `truth` is a fact id.
- **2026-10-26 (v1.5):** faction stories gain `placement` and `meetings`, and faction cards gain `guild_meeting` and `crossroads`, for guild stories that meet (`quest-faction-stories.md` §8b). Faction relations gain *trade*.
- **2026-10-26 (v1.4), v5: world-test fixes:** Place gets `holder`, `titles` and (settlements) `context`; Thing gets `held_by` and `since`; Fact gets `when` and the states fled, changed sides, new post; Card gets `fallbacks` (filled by casting), `open_threads` (distinct from `edges.thread`), and `pays` on ways; endings may carry the `state` they write; `truth`/`twist` never both none; Story gets `open_threads`.
