# Quest Generator Rulebook (v7)

The single source of truth for the shape layer's **shared rules**. Topic docs own the detail for their topics. When a shared rule here conflicts with a topic doc, this rulebook wins.

## Doc map

| Doc | Owns |
|---|---|
| `quest-generator-rulebook.md` (this) | shared rules: principles, registry, reuse, standing, scope, links, entry specs, twist groups, the world ledger and its shape list |
| `quest-card-schema.md` | the exact fields of every card and registry entry, the handoff to later layers |
| `quest-family-*.md` (9 docs) | each family's premises, ways, twists, lists, endings |
| `quest-assembly.md` | the diamond, main cast, roles, plants, relatedness scale |
| `quest-main-quest.md` | main stories: chapters, moves, scale arcs, epic weight, set pieces, lieutenant. **Overrides assembly for main stories.** |
| `quest-fallout.md` | fallout side quests |
| `quest-factions.md` | the generator's faction spec (reads the worldbuilding `factions.md`) |
| `quest-locations.md` | pinning quests to the map, the location pass, clocks |
| `quest-faction-stories.md` | faction profiles and faction stories, and *The throughline* (§11a: the means and hit tags, and the test every guild story passes before its cards are made). **Overrides assembly and main-quest for faction-story cards.** |
| `quest-guild-kits.md` | the six joinable stories' profiles and story kits (the five guilds and the Holy House), each kit entry tagged for *The throughline* |
| `quest-mechanics.md` | what happens after generation: life cycle, effects, clocks, the grudge loop, and *Fresh stories* (§13: the last two games' kit entries, which a new world avoids) |
| `quest-casting.md` | the casting pass: fills the slot specs cards write, layer by layer |
| `faction-places.md` (worldbuilding input) | fixed faction places, generated holdings, and presences. Quests fill place slots from these. |
| `leaders.md` (worldbuilding input) | the leader archetypes of every faction, each with its canon person (the default filler) |
| `worldgen-requirements.md` (stub) | what the land generator must provide. Not shape-layer work. |
| `quest-detail-tables.md`, `quest-dialogue-kit.md`, `quest-card-text.md`, `quest-voices.md` (Lane B) | the story layer: built details on approach, pre-written lines, card and journal text, and the peoples' voices |

---

## 1. Scope
- **Shape layer:** what a quest *is* when generated.
- **Mechanical layer:** what *happens* after the player acts. Parked (§13).
- **Story layer:** what the player reads and hears: on-approach details, the dialogue kit, card text and voices. It is Lane B's (`quest-detail-tables.md`, `quest-dialogue-kit.md`, `quest-card-text.md`, `quest-voices.md`) and reads only the card schema.
- **In scope:** all nine families, assembly, main stories, fallout, factions in minimal form, locations, and faction stories.
- **Parked:** cultures as **shape** rules (a card never requires a people unless canon does; peoples show in voices and casting, not in what a quest can be), and the Rites/Trials family.
- **Shape layer is complete when** every doc in the map is locked, the full quest-set test passes, and the final audit passes.

## 2. Core principles
1. **Situation, not action.** A quest is who wants what, what's in the way, and what makes it hard.
2. **Shared structure.** Every family follows: card fields → premises → ways through → twists → reason/motive lists → possible endings.
3. **Ways-through rule** (all nine families):
   - Each premise lists **4–5 ways**, each tagged with one of the family's 2–4 **kinds**.
   - A card **rolls 2–3, covering at least 2 kinds.** A faction story's crisis rolls **exactly 3** (`quest-faction-stories.md`, *Phase 4: Crisis*). Only the climax and crises may roll a 4th, and only by *Forced ways overflow* (*Roll order*, below).
   - A way is offered only if the card supplies what it needs (a slot, a truth, or a cause).
   - **Pool growth is capped.** Beyond the premise list, a pool may gain:
     - one family-rule way (a shield's way, or *Old wound*)
     - in main stories and faction-story crises, one ally way
     - on the climax turn card, one sacrifice way
     - on a faction story's crisis card, one **code-break way** (in the sacrifice way's place; exempt from the faction's code)
     - on **one** chapter-2 card of a faction story, one **lesser code-break way** (v7, B5): a smaller break of the same kind as the crisis's code-break, such as keeping back one name. It is exempt from the code, rolled on that card like the crisis's code-break (*Roll order*, step 1), and costs less with the guild (`quest-mechanics.md` §9). **The crisis never assumes the player took it.**
   - **Cause-keyed pools** may list up to 6 ways, as long as every cause leaves 3–5.
   - **Signature premises:** a faction may add faction-only premises to a family, beyond its four (`quest-faction-stories.md` §2). Each follows that family's rules.
   - **Added ways** (shield, ally, sacrifice, Old wound) each take one of the card family's own kinds.
   - **Documents:** "go retrieve a document" appears at most **once per family**. A document *target* doesn't count.
   - **Contests:** contest ways are tagged *(contest)* and removed when parties are **2+ tiers apart**. The **Contest family** applies this test to its principals, not its champions (`quest-family-contest.md` §4).
   - **Ways name what's on the card.** Every rolled way names a person, place or thing from its own card's slots, plants or details: "turn Tavish Lark, who wants out", not "turn a lieutenant". Template ways are filled when rolled, never left generic. A way that can't be filled from its own card isn't rolled.
     - **Wording.** A template way is filled from its card: it names a place feature, a thing or a person on the card when the card has one, so the same template reads differently on different cards ("go over the charge with the tanner who swore it, at the tannery steps", not "go over the charge with the witness").
     - The world ledger counts each filled way's **opening phrase** (its verb and what it acts on, such as "go over the charge with"), as it reads once **filled**, not the template it came from. No opening phrase more than **3** times per world (*World ledger*).
     - **Not counted** (v6b audit decision): forced ways (*Roll order* step 1, the always-rolled kind) and the only way that reaches a card's truth. Neither can give its place, so the cap never squeezes them out.
     - Lane B's fill grammar does the filling, and gives every way template at least **3 openings** (`quest-card-text.md`). This rule only sets the limit.
   - **Roll order.** Forced ways go first, then random draws, all inside the 2–3 cap (a 4th only by *Forced ways overflow*, below) and the 2-kinds rule:
     1. ways a rule always rolls (*Old wound*, a required ally way, the climax's sacrifice way, the crisis's code-break way, the chapter-2 card's lesser code-break way). **These never give their place.** If two of them share a kind, the next way rolled must be another kind. When slots run short, the methods preference yields first, then the heart's-secret way (offer it as the twist instead);
     2. if the card has a truth and no way yet reaches it, one that does (rule 8);
     3. **Plant-backed ways roll first.** On climax, crisis and chapter turn cards, pool ways that pay a plant already in the world (an ally, a code-break, a shield, a lead) roll before any random way;
     4. random draws fill the rest.
     - A way that does two jobs (pays a plant and reaches the truth) is preferred.
     - **Forced ways overflow.** On a climax or crisis card, when the always-rolled ways (step 1) and the plant-backed ways (step 3) would fill all 3 slots and a plant-backed way would be left out, the card rolls a **4th** way for it. No other card may.
     - **Twists can come first.** When a twist decides the truth, the twist is rolled before the truth (*Roll order*, twists, in §9).
4. **Slots are specs, not people.** Every person a way or twist points at is a slot. A person a way names is an **optional slot**, filled only if that way is rolled.
5. **Every requirement is a compatibility check.** Anything that doesn't fit is filtered out before rolling.
   - **Premise must fit the place.** The requirements filter checks what the card's place actually has within reach (a noble stronghold for *Captured in battle*, a road fort, a reef village), not just what the registry has somewhere. The place's kind must fit the premise too: no country estate in a gutted reef village.
6. **Shape locks at generation.** Twists, truth, and endings are rolled then. A clocked card's `world_ending` (which ending plays if its checkpoint passes) is rolled then too; what it does, and when, is mechanical.
7. **Meaningful choice.** At least one premise or twist per family leaves the "right" outcome debatable.
   - **Every choice costs a named person** (v7, A4). On a turning card (rule 8's list), each choice ending names **`costs`**: the slot who loses by it (`quest-card-schema.md`, `possible_endings`).
     - **At most one** ending costs only the other side (the opposing slot and their people). One clean win turns a dilemma into decoration.
     - Each family doc gives the default `costs` beside its endings. A card's twist or truth may move a cost (with *Not alone*, *freed* also costs the second captive); the card then checks this rule again.
     - Failure and walk-away endings name `costs` too. What they write is mechanics §4 (*No empty failure*).
8. **Truth slot.** A card whose ways involve finding something out rolls a truth.
   - **No blank card.** Every card carries a hidden truth (a secret fact id in `truth`) or a real twist (not *none*), or both. Minor quests, fallout and Ordeal bouts are not exempt. Twist *none* is legal only on a card with a truth; a card that would have neither rolls its family's truth. A **public** truth (Threat's why-here on Raiders, Protected beast and Encroachment) doesn't count, so those cards never roll *none* (`quest-family-threat.md` §6). Contest's truth row (what the other side is really fighting for) lives in `quest-family-contest.md`.
     - **A confirming truth is a truth.** A row marked *(confirming)* in its family's truth table only confirms what the requester or premise says (*as it looks*, *as told*, *as declared*, *innocent* where the requester says so). A card with one is never blank. With twist *none* it is **plain** (card `plain: true`).
     - **Turning cards are never plain.** Main chapter turn cards, the Ordeal, the climax, the Return, faction-story reveals, phase-3 turn cards and crises each need a non-confirming truth or a twist, or both.
     - **Side cards may be plain** (location cards, echoes, linked pairs, one-off faction jobs, hall jobs, fallout): at most about **1 in 4** of a world's side cards. Plain cards give the twisted ones contrast. A plain card still needs a stake the player can see and a choice that matters.
     - **Minor quests:** at least **1 in 3** has a twist.
   - **The truth is reachable.** When a card has a truth, at least one rolled way reaches it: finds it out, proves it, or acts on it (roll order, rule 3).
   - **Hidden truths show a tell before the choice** (v7, A2). A card with a non-confirming secret truth (a public truth needs none), or a twist from *Not what it seems* or *Someone's hiding something*, names **one public fact that doesn't fit the told story**: the "kidnapped" daughter's chest is gone. It goes in `details.tell`.
     - **It points, and never states.** A reader should find it odd; they should not be able to read the truth off it.
     - **It is on the card's face:** the pressure, the place, a slot's public facts or a way's wording. Never only in a secret fact.
     - *The other side is right*, *They don't want it* and *Innocents pay* show their tell in the pitch, in the place, or through **every** rolled way, so a player on any way sees it. *They don't want it* shows as an act ("she hides from her rescuers").
     - Investigate's clues (its clue rule) are tells already; its card names one of them in `details.tell`. Rescue's hidden motives carry their tell in their table.
     - Truth tables may give a tell beside a row as a pattern; the story layer words it.
9. **Possible endings** are a static list. A faction story's crisis also carries its kit's faction endings (labels only), and a faction's code removes the choice endings it forbids. Endings that the card's own filters make impossible (for example, *accuse before the arbiter* with no arbiter) are removed at generation. What an ending *does* is mechanical-layer work: a failure ending still writes a state (mechanics §4, *No empty failure*), and every ending records **how** it was won, the kind of the way that decided it (mechanics §4, `won_by`; v7, C5).
10. **One entry, one role per card.**
11. **Order, not progress.** A card in a sequence (a chapter, a linked pair, the Return) may assume earlier cards were *reached*. It never assumes how they *ended*.
12. **Card text only says what's true then.** A card's text (pressure, ways, details) may only use facts made true at its time by its `depends_on`, its own plants, its settlement's `context`, or the **state facts of its site** (a licence revoked, a yard occupied; `quest-locations.md`, *Settlement context*). Nothing from a later move, even as a hint.
   - **Directions and travel times come from the map.** They are slots filled at generation (`{travel:A→B}`), never written freehand, so "south" and "six days" always match the map.
   - **No ids or notes in what the player reads.** Fact ids, entry ids and writer's notes never appear in a card's pressure, ways, details, place tie or endings.
   - **Context has a before.** A card that names a context fact lists it in `reads_context`, with its `before` reading: the plain face shown while that fact isn't true yet (`quest-locations.md`, *Settlement context*). `before` is null only when the fact is certainly true before the card can be offered.

## 3. The registry
- **Entry types:** person, place, thing, and **faction** (pre-made only), plus **links**.
  - **Regions** are places of kind *region*. They never count as a use.
  - **The registry carries the whole world faction roster**, whether or not a card uses each faction.
- **Resolving a slot:** the card writes a spec (`quest-casting.md`); the casting pass fills it by search → reuse if it fits and §4 allows → otherwise create. Until casting exists, a card resolves its own slots in that order.
  - **Casting runs layer by layer** (`quest-locations.md` §7a): each layer's specs are cast before the next layer is generated.
  - A reused place keeps its owner.
  - **Sites:** before creating a new site in a settlement (a hamlet, a shed, a yard), reuse one already there that fits.
- **Seats and offices have one holder.** A seat or office (lord of a keep, wharf-master, speaker) binds to its registry holder. A card may not invent a second holder of an occupied seat: a created person whose traits name an office must be its holder. Replacing a holder is an event, recorded as a fact (who, by whom, `when`).
  - **One office per person.** Nobody holds two seats or offices, unless canon gives a canon person two.
  - **Co-leaders sharing a seat:** the seat names one holder; the other holds a `title` at the same place. For leader caps (faction-stories, *Spread across one world*) both count as "the leader".
- **Tags** the main story may set:
  - **foundational:** a stake only main stories may touch
  - **landmark:** the place of a main-story turn card
- **Facts are public or secret.**
  - **Add, never contradict.**
  - **Secrets stay secret.** Reuse may build on a secret but never state it on another card.
  - **Another card's truth or outcome is never a known fact or a requirement.**
  - **Exceptions**, all detailed in their docs:
    - linked pairs (B may build on A's truth)
    - the main story's **revelation** (it may publish the hook's intent and the antagonist faction's hidden side). The revelation stays one secret fact with a `published_by` list of the cards that publish it.
    - a **faction story's revelation** (it may publish its own faction's hidden side, when its kit allows; its own places' secret hooks; and the odd job's hidden role, `quest-faction-stories.md` §6).
  - **No secret leaks.** Moves, plants and fallout never state an earlier card's hidden truth.
- **Past events** are facts on an entry.
  - Every card describing one must match the record, with no second cause.
  - Each carries **`when`**: a relative day (before the story, or days before a move) or the move id it follows. Cards that describe it match its time as well as its cause.
  - A past event is the premise of at most **one card per family**. The main story's and faction stories' **moves** are exempt; fallout uses its own family + premise rule.
- **States:**
  - **No conflicting roles.** Someone held, a fugitive, or dead on one card can't take a contradicting role elsewhere.
  - **No conflicting states.** No card gives an entry a state (broken, sick, seized, captive, dead) that conflicts with what another card needs.
  - **Main-story ordering:** a state set on a card binds later cards in its chapter, later layers, and the Return.
  - **State is written down.** A shield or ending that changes a person (Fled, Changed sides, Disgraced, took a new post) writes a **state fact** on that person. A shield's state holds from generation; an ending's state is recorded with the ending and applied by the mechanical layer. Later cards read it: someone who fled holds no office back home.
  - **Endings don't remove people a later card needs.** If an ending would kill or drive off a person a later card needs, either that ending is removed from the card, or the later card names a fallback for that slot (casting will choose fallbacks).
    - The check covers **every** card that can remove someone: blood bouts and deadly duels (the stake and both fighters), minors, and fallout.
    - **Plant carriers stay out of deadly bouts.** A person who carries a plant a later card needs may not stand in a bout that can turn deadly, unless the plant is also given to their fallback. (A fallback fills the slot, but can't make a plant true that only the dead person knew.)
- **Things are on the record.** A thing carries **`held_by`** (who has it now) and **`since`** (the card or move it changed hands at). A thing a card calls "the only X" (the only record, the seal) is a registry thing, created once and reused, never invented twice.
- **Stakes are registry entries:**
  - a **right**: a document, or attached to a place
  - a **reputation**: a fact on a person, or on a faction entry
  - a **bond**: a link
- **Name check (loosened for world scale, Laz 2026-10-26):**
  - **No two entries share a full name,** anywhere in the world.
  - **Stems are checked locally:** within one card, one chain (a main or faction story, a linked pair), or one location, no two generated people share a given name or its first four letters.
  - **Given names are unique in a region** among people on story cards (main, faction, minor, fallout).
  - **Kin share a family name:** people with a family link (parent, child, sibling) carry the same family name. **Exception:** where Contest's *Old blood* hides a kin tie, the hidden one goes by another name (a mother's line or a byname; `quest-family-contest.md` §6).
  - A **stem** is a shared proper-noun word, or the same first four letters.
  - Generic words (Mill, Hall, Lock), world-given faction names and **family names** are exempt.
  - **Canon names are exempt:** names from the worldbuilding docs (leaders, their bynames, fixed places) and the parts they share ("Lanternhall" and "Lamawina of the Lantern"). The stem rule applies to generated names only.
  - **Twist names** match the rulebook §9 table, which is canonical. Family docs may phrase them longer ("The law is on the wrong side").
- **Background pass:** ordinary Low and Middle people and places are added last.
- **Test rolls** in family docs are illustrative only. They show a shape, not wording to copy (§11, item 10).

## 4. Reuse and creation
- **Use = filling a slot.**
  - A card's `offered_at` place and its `giver` are **not** uses.
  - A card's `place` and every place-valued slot count as uses.
  - Regions don't count. **Map settlements used as containers** don't count either: a card set at a site inside a town uses the site, not the town (`quest-locations.md` §2).
  - An owner counts only when they hold a slot.
  - Being named in a fact or a plant doesn't count.
- **Once per family:** each entry appears in at most one card of each family.
  - **Exception, the crisis's family:** a cast member or a plant's knower placed on a card of the same family as their story's crisis doesn't break *once per family* (so an ally can be planted on a Reckoning card before a Reckoning crisis).
- **Cap:** at most **3 cards** total per entry.
- **Owners** are fixed at generation.
- **The dead** are newly created. A recorded dead person may be referenced again, but only as dead.
  - **Exception, `dies_at`:** a person whose death is a story move carries `dies_at: <move id>`. They may fill living slots on cards before that move, and only appear as dead after it.
- **Top-standing people:**
  - Pre-made only (roster default 4; archetype fillers count as pre-made). Top faction leaders count toward the roster.
  - **Authority-only slots** (arbiter, authority, petition target, answers-to, route authority, ring-keeper): exempt from the cap and once-per-family.
- **Officials of any standing** (a magistrate, a ring-keeper, a toll captain) may also fill authority-only slots on up to **3** cards regardless of family, so one killing doesn't need two magistrates.
  - **Premade High leaders** in authority-only slots (arbiter, judge) don't count toward that cap of 3, the same as Top.
- **No conflict of interest:** an arbiter or ring-keeper has no family, kin or ally link to a party, unless a twist (*Bought*, *law on the wrong side*) makes that the point.
  - **As a party:** normal rules apply.
  - **Bribe and buy-off ways** are removed whenever the target is Top.
- **Outranking slots:** every "someone who outranks X" is a resolved slot. A way that needs someone who outranks a party (a petition, a patron) is filtered out when no registry person outranks them. With the emperor the only Top person, nobody outranks a High party but him, and he doesn't hear such matters (Reckoning's *Rank* shield), so the way is filtered out.
- **Fallbacks:** a reused person whose role another card could change gets a fallback, chosen by the casting pass at generation (`quest-casting.md` §6, step 8).
- **Place facts** are reused when another card uses the same place.
- **Factions:** exempt from the cap and once-per-family. A faction is the antagonist's side in at most one main story per world generation.
- **Main-story cast:**
  - Members: antagonist, heart and her fallback (when standing in for her), Hand, lieutenant.
  - Exempt inside their own storyline. At most one slot per chapter.
- **Faction-story cast:** exempt inside its own story, like main-story cast (`quest-faction-stories.md` §8).
- **Proxies and bystanders:**
  - **Proxies** are people linked to the cast, or members of the antagonist's faction. They follow normal rules.
  - **Bystander members** in fallout are not proxies.

## 5. Standing and scope

### Standing

| Tier | Who |
|---|---|
| Low | peasant, laborer, servant |
| Middle | tradesperson, merchant, soldier, moneylender, steward, headman |
| High | gentry, noble |
| Top | ruler, high official (pre-made only; archetype fillers count as pre-made) |

- **"Outranks"** means strictly higher.
- **"2+ tiers apart"** is the test for contests.
- **Separation rule (one band):** in economic premises (debt, ransom, trade agreement, inheritance), the two main parties are at most one tier apart. It applies to main parties only, and each premise declares its status.
  - **The main parties:**
    - Captured in battle: captor and captive
    - Bound by debt: captor and captive
    - Contested inheritance: the two heirs
    - Broken agreement: the two parties
    - Succession: the two claimants
- **Exempt from separation:**
  - crime
  - authority
  - public offense
  - shared resources
  - employer/employee pairs
  - war
  - **acquisition** *(flagged for Laz)*
- **Requester standing:** a Low requester can't fund payments, so the cost shifts to the player or is paid in labor.

### Scope
Every card records how far its pressure reaches.

| Scope | Reaches | Landmark standing (the settlement's) |
|---|---|---|
| household | one family | the heart's home |
| community | one settlement or one group | Middle |
| region | several settlements or a lordship | High |
| realm | the whole land, a Top seat, a realm road or law | Top |

- **Landmark standing is the settlement's, not the site's:** a community landmark can be a yard or a shed in a Middle-standing town (`quest-main-quest.md`, *Scope*).

- **Side quests** are capped at **community** and never use foundational entries.
- **Faction-story cards** (relatedness *faction*) are capped at **region**, and never use foundational entries.
- **Faction seats:** a faction seat's standing follows the faction's reach (realm reach means a Top-standing seat), whatever the leader's standing.
- **Schism exception:** under *faction law* (the Schism set piece), the faction's own leaders count as the arbiter, whatever their standing.
- **Faction reach** caps the scope of any card a faction drives. **Exceptions:**
  - faction-story cards are capped at region whatever the faction's reach (`quest-faction-stories.md` §7);
  - in a main story's **Rising** arc, a sub-faction may borrow its parent faction's realm reach, for its moves and the cards they drive (`quest-main-quest.md`, *Scale arcs*, Rising).

## 6. Links
- **Person ↔ person:**
  - family (parent/child, siblings, heir)
  - kin (uncle, aunt, cousin)
  - spouse/lover
  - ally
  - employer/employee
  - rival
  - creditor/debtor
  - grudge
  - dependent
  - **trade** (partners in a deal)
- **Direction:** asymmetric links record which side each person is on, as {`type`, `to`, `as`}. For example, `as: employee`.
- **Person → faction:** *member*, *backed by*.
- **Faction ↔ faction:** *ally*, *rival*, *at war*, *trade* (dealings without loyalty), *serves*. A sub-faction inherits its parent's relations unless overridden.
- **Creditor roles:** the creditor's employee may fill them.
- **Groups** (a guild, a village) take part through one named representative.
- **Sharing a faction is not a thread.**

## 7. Entry specs
- **Person:** name, standing, people, age band, voice, role traits, links, facts, faction membership (optional).
- **Place:** kind, owner, access (*open, guarded, remote, fortified*), standing, tags.
  - *Fortified* rules out direct assault.
  - A guarded or fortified scene requires an insider.
  - **Route kinds:** road, pass, ford, river, wilds. Waypoints count toward the cap.
  - **Sea routes are parked** while the sea stays off-limits to the squad (sim canon). A coastal Passage goes by the shore road. They come back when Laz opens the sea.
  - **Until Laz opens the sea** (v6b audit decision):
    - **ways that need the sea are filtered out** (rule 5), and so are signature premises built on one: the Night Market's "put the worst of it on a Far Sails boat" (`quest-guild-kits.md`, *The market moves*), the Far Sails' *Out to the fleet* (`faction-places.md` §7);
    - **offshore places take no cards:** the Anchorage of the Wide Bay, the Lantern Rock, and any place only on an islet or skerry (`faction-places.md` §3);
    - **faction ending labels that mention the sea stay.** They say what NPCs do, not where the player goes.
- **Thing:** kind (*heirloom, document, weapon, relic, goods, tool*), owner, value, location.
- **Faction:**
  - name, size, parent, leaders, seat, reach, want
  - virtue, wound
  - hidden side (secret)
  - relations
- **Shared want list** (antagonists, factions, culprits): greed, revenge, cover-up, desperation, ambition, preservation.
- **Shared reason list** (requesters; v7, A8). One list for every family, so a profile's motive weights always land, and every pitch has a "why".

| Reason | What it is |
|---|---|
| survival | they can't live without it |
| dependents | others rely on them |
| legacy | it was the family's, or tradition promised it |
| pride | losing it would shame them (collecting counts here) |
| greed | they want more |
| obligation | they owe it to someone else |
| evidence | it proves something |
| love | someone they love is in it *(Claude)* |
| duty | their office, oath or post makes it theirs to ask |
| fear | of what comes if nobody acts |
| faith | their god or their house asks it |
| curiosity | they want to know |
| money | it is their trade or their wage: fair pay, not more (more is greed) |
| freedom | theirs, or someone else's |
| honor | a name to keep |
| standing | a ladder place, or a rank |
| peace | settle it without blood |
| a promise | they swore they'd see it through |
| proof | that a school, a style or a teacher is worth something |
| need | it clears a debt or feeds a household |

  - **Each family names its rows,** with any requirement beside them (Acquire's *legacy* needs a family link to a past owner). Other slots' reason lists (a holder's, a party's, an opponent's, why a cargo flees) stay in their family docs; where a word is on both lists it means the same thing.
  - **Every card fills `reasons.requester`** with one row its family names (card schema, `reasons`).
  - **A profile's motive weights name rows of this list** (`quest-faction-stories.md` §2). A weight on a row the card's family doesn't name does nothing on that card.
  - **No reason fills more than 30%** of a world's cards. The world ledger counts it (§9a).
- The full field list for cards lives in `quest-card-schema.md`.

---

## 8. Families: index

| Family | Situation | Kinds of ways | Premises | Twists (besides none) |
|---|---|---|---|---|
| **Rescue** | get someone out of where they're held | force, stealth, price, leverage | Kidnapped, Captured in battle, Imprisoned unjustly, Bound by debt | unwilling captive, hidden motive, justified captor, not alone |
| **Investigate** | find out who did something, and why | question, examine, follow, provoke | Theft, Disappearance, Suspicious death, Sabotage, *Watch them* | the truth costs the requester, sympathetic culprit, no crime, wrong target |
| **Dispute** | two partly-right parties, one stake | truth, deal, side | Contested inheritance, Broken agreement, Shared resource, Public offense | caught in the middle, hollow prize, old wound, law on the wrong side |
| **Threat** | something endangers a community | confront, undermine, protect, bargain | Raiders, Protected beast, Sickness or blight, Encroachment, *Go and look* | one of us caused it, divided community, passed along, protector's price |
| **Passage** | the cargo itself makes the journey hard | route, cover, force, arrangement | Smuggling, Fugitive, Unwilling passenger, Unwelcome cargo | not wanted at the end, pursuer has a point, need along the way, the one left behind |
| **Acquire** | obtain a specific thing someone else holds | take, deceive, trade, earn | Won't sell, Passed on, Under seal, Unknown worth, *Take the readings* | load-bearing, wants to be rid of it, meant for destruction, rightful keeper |
| **Reckoning** | a known wrong that justice failed | justice, ruin, amends, violence | Betrayal, Ruin from above, Blood debt, Swindle | wronged don't want it, changed, hand not head, disproportionate demand |
| **Restore** | something relied on is broken | craft, gather, persuade, substitute | Broken works, Lost good name, Estrangement, Lost livelihood | someone wants it broken, it broke for a reason, can't go back, one person pays |
| **Contest** | a matter sworn to a contest; winning is hard, or wrong | prepare, rules, edge, stakes | Champion's bout, The fix, First blood or last, The ladder | needs it more, the fix protects someone, old blood, the champion was promised something |

**Signature premises** (faction-only, `quest-guild-kits.md`) sit on top of these.

**Field-work premises** (v7, Laz's C1): spying, scouting and field work are premises in existing families, not a new family. Each is open to any card, like the other premises.
- Investigate *Watch them*: watch a person or place where no crime has happened yet, and report what you see.
- Threat *Go and look*: scout a dangerous or unknown place and come back with what's there. Threat, not Passage: Passage's cargo must be the problem (its anti-escort rule), and a scout carries nothing; Threat already has a community at risk from something nearby *(Claude)*.
- Acquire *Take the readings*: field work with an instrument, a survey or a copy made on site. Acquire, not Restore: nothing need be broken, and the obstacle is the keeper of the ground, which is Acquire's shape *(Claude)*.
- Their truth rows use shapes already on the list (*Shapes*).

**Main-story-only set pieces** (`quest-main-quest.md` §8):
- Siege (Threat)
- Succession (Dispute)
- Embassy (Passage)
- Schism (Reckoning), also allowed as a faction story's crisis

## 9. Twist groups
- **Main storyline** (main cards, the Return, and minors): at most **two twists from each group, or one per 10 storyline cards, whichever is more**. Two twists from the same group never sit in the same chapter or in linked chapters.
- **Fallout:** each main story's fallout counts as one storyline, with at most **one twist per group**.
- *(The test showed one-per-group starved the Ordeal and Climax of twists.)*
- **Roll order** (twists; the ways' part is rule 3):
  - **Twists before truths when the twist decides the truth.** Roll the twist first, then the truth from the rows the twist allows (Rescue's *justified captor*, Investigate's *no crime*, Contest's *old blood*).
  - **Twist budget goes to turning cards first.** In each storyline, roll the twists of the climax, the Return and the Ordeal first, then chapter turn cards and crises, then the rest. This is the order twists are *rolled*, not the order cards are written (assembly's fill order stands).

| Group | Twists |
|---|---|
| **The other side is right** | justified captor, sympathetic culprit, law on the wrong side, rightful keeper, pursuer has a point, needs it more, the fix protects someone, disproportionate demand; faction: the contract is on the wrong side, the old heads were right once |
| **You started it** | old wound, one of us caused it, old blood, it broke for a reason; faction: someone was told first, a lamp still lit, old debts |
| **Innocents pay** | caught in the middle, load-bearing, one person pays, protector's price, the one left behind, changed, not alone, the champion was promised something, need along the way; faction: someone's name is in it, someone's going to talk, the goods are evidence, better than they know, the sincere one pays |
| **They don't want it** | unwilling captive, the wronged don't want it, the truth costs the requester, not wanted at the end, divided community, can't go back; faction: the house won't thank you, faith, not fear |
| **Not what it seems** (the job is smaller, stranger or other than it was told) | no crime, wrong target, hollow prize, wants to be rid of it |
| **Someone's hiding something** (a person on the card hides their hand) | hidden motive, meant for destruction, hand not head, someone wants it broken, passed along |

- **Every twist is grouped.** Every twist a family doc or guild kit names sits in exactly one group above; a twist with no group can't be rolled. A new twist (or a parked one brought back, such as Dispute's *Arbiter wants cover*) is added here first.

- **Faction stories:** each counts as its own storyline, with at most **two twists per group**, never in the same chapter (`quest-faction-stories.md` §8).

## 9a. World ledger
One variety ledger is shared by every pass in a world: main, faction, location and fallout. Each pass reads it before rolling and adds to it as it goes. The faction-stories and locations ledgers are views of this one, and their own tighter limits still apply.
- **The main story counts toward every cap here.** The guild-only caps in faction-stories (*Spread across one world*) count guild stories only. Since v6c they include **writ fatigue** (at most 2 guild-story steps carried by paper aimed at the guild), and every guild story passes *The throughline* test before its cards are made (`quest-faction-stories.md` §9, §11a).
- **One world at a time.** The ledger starts empty in every world. The only thing a world reads from earlier games is the recent list (`quest-mechanics.md` §13, *Fresh stories*), which makes the last two games' guild entries and main arc and shape unavailable unless nothing else fits.
- **It counts:**
  - family + premise
  - truth rows, and their **shapes** (*Shapes*, below): the cap on what a world repeats is on the shape, not only the row's name
  - twists
  - move levers (the mechanism a move works through, such as a bought debt called early)
  - crisis family + premise
  - **ordeal events,** keyed by **target + event** (v6b): the main story's Ordeal and every guild story's chapter-2 event, together
  - **way wording:** each filled way's opening phrase, as filled; forced ways and a card's only way to its truth don't count (rule 3, *Ways name what's on the card*)
  - **requester reasons** (v7, A8): no row of the shared reason list on more than **30%** of the world's cards (§7)
- **Caps scale with the world.** N is the planned number of cards in the world, set before generation (`worldgen-requirements.md`).
  - a family + premise: about **4%** of N (at least 2);
  - a truth row: **max(3, ⌈N ÷ 40⌉)**. A signature premise's rows count apart from its base family's rows, but share their shapes;
  - a shape: **max(4, ⌈N ÷ 30⌉)**, counting every row and twist that has it. Confirming rows have no shape;
  - an ordeal **target + event** (Laz, v6b): each used **once** per world. The main Ordeal's (heart, event) counts. Across the guild stories, each **target** (the heart, the player, the guild) is used by at least **1** and at most **3** stories. Ten combinations for seven uses gives slack; the guild placement plan still deals them (`quest-faction-stories.md`, *Spread across one world*). This replaces "each event at most twice". *The cost* as a reveal at most twice, worded differently each time;
  - a way's opening phrase: at most **3** times (as filled; forced ways and a card's only way to its truth excepted, rule 3);
  - crises: at most **3** are Reckoning, and no crisis family + premise more than twice. These count guild and faction crises only; the main climax is not a crisis.
  - *Example:* N = 150 gives a truth row 4, a shape 5, a family + premise 6.
- **A family is never closed by the ledger.** When every non-confirming row of a premise is at its cap (or its shape is), those rows each gain one. A capped card rerolls its truth row, then its premise. It never drops its truth to dodge a cap.
- **Reserved levers.** The main story reserves its last move's lever **and every lever it uses on 2 or more moves**. Other stories may use each at most **once** in the world. The mechanism counts too: a side premise or ordeal event built on a reserved mechanism (called notes, say) counts as a use. Levers are checked when each story's moves are written; kits carry none.
- **Background people in truths.** A High or Top person who appears only inside a truth or twist (the better buyer, the fixer above) fills such a role on **one** card per world.
- **Passes run in sequence.** A real generator runs the main, guild and location passes one after another, so each reads the ledger as the last left it. Parallel writers (paper tests only) get **reserved shares**: the planner splits what's left of each cap between them beforehand.

### Shapes
A **shape** is the mechanism a truth or twist turns on, under whatever words or keys it wears.
- **One mechanism, one shape.** Rows that are variants of one another, carry a suffix ("bystander culprit (greed)", "(cover-up)"), or are a signature premise's copy of a base row ("The favor called in: still profiting") have one shape. Rows in different families that turn on the same mechanism share a shape too: water spoiled from upstream, a better buyer, a copy, both sides partly right.
- **Every truth row has one shape,** written beside the row in its family's truth table (and in the guild kits for signature premises). A card records it in `shapes` and the row in `truth_row` (`quest-card-schema.md`).
- **Confirming rows** (the truth only confirms what the requester or premise says: *as it looks*, *as told*, *as declared*, *honorable*, *the stated reason*, *project sound*, *innocent* where the requester says so) are marked *(confirming)*. They have no shape and don't count against shape caps. In every family they are a minority of the rows.
- **Twists** get a shape only when the twist itself names a mechanism: a family doc gives it one (*Passed along*, *Hand, not head*, *Someone wants it broken*), or its detail turns on one from the list (a better buyer for the captive, the fix protects someone, a step loosened in the night). Most twists' names are their shape; the ledger counts those as twists. When a twist fixes the truth row, the card counts that shape once.
- **The shape list.** A new row takes a shape from this list, or a new slug is added here first.

| Shape | What it turns on |
|---|---|
| `sold-from-inside` | someone on the requester's own side told or sold to the other side |
| `another-buyer` | already sold, pledged or promised to someone else |
| `held-rightly` | the one held or accused really did it, or really owes it |
| `wrong-name` | the debt or charge sits on the wrong person |
| `paid-already` | it was paid, at least in part, and the payment went astray or unrecorded |
| `made-bigger` | true, but blown up past what happened |
| `bystander-did-it` | someone at the edge of it, whom nobody looked at, did it |
| `close-to-the-victim` | someone the victim knew did it |
| `close-to-home` | someone the requester loves or relies on did it |
| `the-enemy-did-it` | the story's own enemy (the Hand, the lieutenant, a proxy) did it |
| `no-crime` | nobody did it: an accident, a choice, a thing mislaid |
| `record-survives` | a record, or someone's memory, settles it |
| `no-record` | nothing survives to settle it |
| `one-side-lies` | one party's account is false |
| `both-partly-right` | each claim holds in part |
| `ran-out` | what they lived on is gone (nothing left at home, a worked-out seam) |
| `pushed-here` | it was driven out of somewhere else, so it came here |
| `someone-above` | someone behind or above the one acting ordered it or pays for it |
| `harm-from-upstream` | spoiled from upstream: bad water, waste, a works above |
| `brought-in` | carried in from outside: a traded beast, travellers |
| `neglected` | left to rot by whoever should tend it |
| `built-to-fail` | made badly, or planned so it will fail |
| `more-than-said` | the cargo or the guest carries more than was said |
| `a-copy` | not the only one: a copy, or the original is elsewhere |
| `unheard-witness` | someone saw it and hasn't spoken |
| `shared-blame` | the requester's own side had a hand in it |
| `still-profiting` | someone still lives off the wrong or the harm |
| `ground-moved` | the land itself undid it |
| `stripped-for-parts` | someone took pieces of it for their own use |
| `broken-on-purpose` | deliberately broken or ruined (Return and epilogue only) |
| `true-but-past` | it really happened, long ago |
| `rumour-planted` | someone off the card started the story on purpose |
| `absent-when-needed` | one of them wasn't there when it mattered |
| `disapproved-match` | a marriage one of them couldn't accept |
| `took-a-side` | a side taken in someone else's quarrel |
| `secret-told` | a secret that got told |
| `lost-knowledge` | the one who knew how died with it |
| `trade-moved` | the road or the buyers went elsewhere |
| `hidden-aim` | the stated aim covers, or opens onto, another |
| `old-grudge` | an old grievance against the requester's side |
| `protects-someone` | what looks wrong is shielding someone |
| `wants-out` | they want to be stopped or let go, with honour |
| `owes-a-backer` | they do it to pay what a backer wants |
| `cant-pay` | whoever should pay can't, and hides it |
| `damning-record` | the record shows what its keeper needs hidden |
| `older-wrong-beneath` | it covers an earlier wrong |
| `rigged-to-fall` | something loosened or set so the next to use it falls |

## 10. Roadmap

| Item | Status |
|---|---|
| All nine families | eight locked; Contest drafted (v3.2) |
| Assembly, main quest, fallout, factions | locked |
| Card schema | locked |
| Full quest-set test | passed (`quest-test-results.md`) |
| Final audit | passed |
| Faction places and presences | drafted (`faction-places.md` v4.2) |
| Locations v3.3, faction stories v5.3, Contest v3.2, guild kits v5.1 | drafted |
| Mechanics | drafted (`quest-mechanics.md` v0.3) |
| Casting | drafted, v1.2 (`quest-casting.md`) |
| v7 book rules (A1–A11, B5, B7, C1, C5) | applied to the docs; tested first on the Writers |
| Cultures, Rites/Trials | parked |

## 11. Required contents of any future family draft
1. **Situation**, and how it differs from every family.
2. **Card fields**, including optional slots.
3. **2–4 kinds** of ways through.
4. **4 premises**, each with its requirements, 4–5 tagged ways, and separation status. (Since v7 a family may also carry a field-work premise, §8.)
5. **3–4 twists plus none**, with requirements, a compatibility table, and a twist-group check.
6. **Reason and motive lists.**
7. **Possible endings.**
8. **Standing effects.**
9. **Registry interactions.**
10. **At least 5 test rolls**, including one with a sympathetic High party.
    - **Examples show shape, not wording** (v7, A11). Our agents copy what they read. No two test rolls in a doc share all four of premise, twist group, standing and place kind, and none reuses another doc's names, objects or openings.
11. **Decisions made without Laz.**

## 12. Review checklists

**Logic reviewer:**
- contradictions with locked rules
- unfillable requirements
- missing filters
- reuse, state, and secret conflicts
- standing, separation, and scope errors
- overlap between families
- dead ends
- rolls with fewer than 2 ways or 1 kind
- mechanical drift

**Creative reviewer:**
- fetch, kill, or escort quests in disguise
- stock twists
- location-only variety
- repetitive ways
- real moral tension
- interesting Low and Middle characters
- cross-family repetition
- concrete replacements, culture-neutral, no new systems

## 13. Parked for the mechanical layer
- Ownership changing hands.
- Outcomes writing links and facts back to the registry.
- The player as a registry entry.
- Pressure resolving on its own.
- Deals and unresolved quests returning.
- Flags and triggers.
- **Clocks:** when checkpoints fire, and what a clocked card's `world_ending` does.
- **Progression:** which nodes are required, optional nodes, when side quests (including fallout) become available.
- What minor-quest advantages do, and which allies actually answer.
- A quest sliding along the relatedness scale.
- How endings change later cards.
- Faction reputation, faction relations shifting, joining and leaving factions.

## 14. Structure summary
- **Relatedness:** 4 Main (chapters and the Return), 3 Minor, 2 Fallout, 1 Echo, 0 Independent. Plus linked pairs, and **faction** (faction-story cards, outside the numbered scale).
- **Faction stories:** a 9–12 card chain per joinable faction (six: the five guilds and the Holy House): routine → reveal → two chapters → crisis. Plus about 9 **hall jobs** per guild, about 20 quests in all, climbed by rank (`quest-mechanics.md` §9, *The guild ladder*; `quest-locations.md` §4, *Hall jobs*).
- **Main stories:**
  - **Diamond:** L layers (5–8) and peak width W, with 2 ≤ W ≤ (L+1)/2.
  - **Chapters:** each node is a 2–4 card chapter.
  - **Moves:** the antagonist acts between layers.
  - **Scale arcs:** Rising, Ripple, Inward, Deep.
  - **Roles:** Hook > Climax > Ordeal > Threshold > Trials, plus the Return.
  - **Path:** 15–25 quests.

## 15. Decisions made without Laz
Each can be overturned.
1. **Way-wording cap** (v6b audit): the cap counts the filled wording, not the template; forced ways and the only way that reaches a card's truth don't count toward it; Lane B gives every way template at least 3 openings (rule 3, *Wording*; §9a).
2. **Sea leftovers** (v6b audit), until Laz opens the sea: ways and signature premises that need the sea are filtered out; offshore places (the Anchorage, the Lantern Rock, any islet-only place) take no cards; faction ending labels that mention the sea stay, since they describe what NPCs do (§7, *Sea routes are parked*).
3. **Field-work premises' homes** (v7, C1): *Go and look* in Threat, *Take the readings* in Acquire (§8). Their rows, ways, tells and endings are *(Claude)* in each family doc.
4. **`love`** joins the shared reason list (§7): Rescue, Investigate, Passage and Reckoning had no row for "someone they love is in it".

## Changelog
- **2026-10-27 (v7), v7: Laz-approved book rules (A1–A11, B5, B7, C1, C5; `research/proposals.md`):** every choice on a turning card names who pays, `costs`, at most one costing only the other side (rule 7, A4); hidden truths show a tell, `details.tell` (rule 8, A2); one chapter-2 card of a faction story may roll a lesser code-break way (rule 3, B5); endings record how they were won (rule 9, C5); one shared requester reason list, each family naming its rows, no reason on more than 30% of a world (§7, §9a, A8); field-work premises *Watch them*, *Go and look* and *Take the readings* in Investigate, Threat and Acquire, using existing shapes (§8, C1); examples show shape, not wording (§3, §11, A11); roadmap and §15. A1, A5, A6, A7, A9 and B7 live in locations, mechanics and the family docs; A10 in faction-stories.
- **2026-10-27 (v6), v6: World C fixes:** *Shapes* and the shape list, every truth row tagged in the family docs (§9a); *World ledger* counts shapes, ordeal events and way wording, caps scale with the planned world size N, a family is never closed, reserved levers include every lever the main story uses twice, background people in truths once per world, passes run in sequence, the main story counts toward every cap (§9a); *No blank card*: confirming truths count, plain cards, turning cards never plain, side cards up to 1 in 4 plain, minors 1 in 3 twisted (rule 8); *Roll order*: twists before truths when the twist decides it, twist budget to turning cards first (§9), forced ways overflow to a 4th way on the climax and crises (rule 3); *Ways name what's on the card*: wording, opening phrase at most 3 per world (rule 3); *Card text only says what's true then*: no ids or notes, `reads_context` (rule 12); *Seats and offices*: one office per person, co-leaders (§3); *Endings don't remove people*: every removing card, plant carriers out of deadly bouts (§3); *Old blood* name exception (name check); once-per-family exception for the crisis's family, *Outranking slots* filter (§4); sea routes parked until Laz opens the sea (§7). Follow-up: rule 12's sources include site state facts; Rising's borrowed parent reach is an exception to *Faction reach*; landmark standing is the settlement's (§5); ordeal events are dealt by the guild placement plan (§9a). v6b: Laz's guild changes (27 Oct): the ledger's ordeal key is target + event, each once per world (the main Ordeal counts), each target by 1–3 guild stories, replacing "each event at most twice" (§9a); hall jobs are side cards that may be plain (rule 8); six joinable stories, the Holy House included, and about 20 quests per guild with hall jobs (doc map, §14). v6b audit: roadmap versions brought up to date (§10). v6b audit decision: the way-wording cap counts filled wording and skips forced ways and a card's only way to its truth, and Lane B gives every template 3+ openings (rule 3, §9a); until the sea opens, sea ways are filtered out, offshore places take no cards, and sea ending labels stay (§7); new §15 *Decisions made without Laz*. v6c: pointers only: the doc map names *The throughline* (faction-stories §11a) and *Fresh stories* (mechanics §13); the *World ledger* points to writ fatigue, the throughline test and the recent list (§9a).
- **2026-10-26 (v5.2), lanes cross-check:** the story layer is Lane B's four docs, not parked; cultures are parked only as shape rules; the doc map lists Lane B's docs.
- **2026-10-26 (v5.1), audit fixes:** doc map adds mechanics, casting and `leaders.md`; crisis rolls exactly 3, forced ways never give their place (rule 3); a public truth doesn't satisfy *No blank card* (rule 8); slots are written as specs and cast layer by layer (§3); the odd job's hidden role joins the faction-revelation exception (§3); archetype fillers count as pre-made (§4, §5); *trade* relation (§6); `leaders` (§7); only guild and faction crises count toward the crisis caps (§9a); roadmap updated (§10).
- **2026-10-26 (v5), v5: world-test fixes:** *No blank card*, truth is reachable (rule 8); ways name what's on the card, roll order, *Plant-backed ways roll first* (rule 3); premise must fit the place (rule 5); card text only says what's true then (rule 12); *Seats and offices have one holder*, site reuse, `when` on past events, state is written down, things on the record, endings don't remove people a later card needs (§3); given names per region, kin share a family name (name check); premade High leaders exempt from the authority cap (§4); twist audit, two new groups (*Not what it seems*, *Someone's hiding something*), every twist grouped (§9); *World ledger* (§9a).
