# The Quest Generator: how the whole system works

*Written 2026-10-26 by Claude, for Laz and for whoever builds it (v1.1, audit fixes the same day; changes at the end). No code here: this explains what the system does and how each part should work, mechanically.*

**Where the detailed rules live:** the claude.ai **Gahturiyu** Project, in the `claude/quest-*.md` docs. The rulebook (`quest-generator-rulebook.md`) wins any conflict.

| Doc | Owns |
|---|---|
| `quest-generator-rulebook.md` | shared rules |
| `quest-family-*.md` | the nine quest families |
| `quest-assembly.md`, `quest-main-quest.md` | the main story |
| `quest-fallout.md` | fallout |
| `quest-faction-stories.md`, `quest-guild-kits.md` | faction stories |
| `quest-locations.md` | pinning and the location pass |
| `quest-mechanics.md` | the mechanics layer |
| `quest-card-schema.md` | the data |
| `quest-casting.md` | casting: specs, classes, archetypes, court state, and how specs are filled layer by layer |
| `leaders.md` | the leader archetypes and their canon defaults (worldbuilding input) |
| `faction-places.md` | places |
| `worldgen-requirements.md` | what the land must provide |

This doc is the map of how they fit together. Where a big design decision was made in conversation and isn't in those docs yet (the casting pass itself), it is written out in full here.

**Status:**
- The **shape layer** (what quests *are*) is designed and has been tested on generated worlds.
- **Casting** is designed here; **mechanics and the grudge loop** are in `quest-mechanics.md`. None of them is tested yet.
- **Dialogue and the on-approach detail pass** are agreed in principle.
- A full-world test was running when this was written. Its findings will change details, so check the Project's test results.
- **Not yet in the topic docs:** which archetypes are guaranteed beyond the lore-critical ones (§16). Specs, archetype leaders, court state and the *trade* tie went into the topic docs with the 2026-10-26 audit fixes, so a gap anywhere else is stale text, not an intended one.

---

## 1. The big picture

The game builds its quests the way a dungeon master preps a campaign: **all the plot up front, the scenery when the players get there.**

| When | What happens | Speed |
|---|---|---|
| **World creation** | Every quest in the world is generated (the main story, five guild stories, and every town's side quests), along with the people, places, things and facts they need. | a few seconds; it's all text and numbers |
| **On approach** (the player nears a region or place) | The physical detail of places is worked out from what's saved about them: a keep's layout, its guard count, the drain a quest said was there. | invisible: a background job |
| **During play** | Quests open, run and end. Endings change facts. The world reacts by opening, closing or (rarely) generating new quests. Enemies who survive remember the player. | instant |

**The system has five parts:**
1. **The shape layer:** the rules for what a quest *is* (its situation, who's in it, its ways through, its twist, its endings) and how quests join into stories.
2. **Casting:** deciding *who* the people in quests actually are, and *which* places, by filling job descriptions from a world roster.
3. **The mechanics layer:** what happens after generation. Facts, signals, quest states, effects, clocks, reputation, the grudge loop.
4. **The detail pass:** turning a place's saved description into its physical form when the player approaches.
5. **Dialogue:** a large, pre-written, tagged kit of lines, filled in from quest facts.

**One promise holds through all of it: a quest's shape never changes after generation.** The world reacts by opening, closing or adding quests, never by rewriting one. That is what keeps a generated world consistent, saveable and debuggable.

---

## 2. Vocabulary

| Word | Meaning |
|---|---|
| **Card** | one quest. Everything about a quest is on its card. |
| **Family** | the kind of situation. There are nine: Rescue, Investigate, Dispute, Threat, Passage, Acquire, Reckoning, Restore, Contest. |
| **Premise** | the specific situation within a family, such as Rescue *Kidnapped* or Reckoning *Swindle*. Each family has four, plus faction-only **signature premises**. |
| **Slot** | a role on a card: requester, captive, captor, holder, arbiter, guide… A slot is a *spec*, not a person, until casting fills it. |
| **Way through** | one approach the player can take. Each premise lists 4–5, each tagged with a **kind**; Rescue's kinds are force, stealth, price and leverage. A card rolls 2–3 of them (a faction crisis exactly 3), covering at least 2 kinds, so there is always more than one way. |
| **Twist** | an optional complication rolled at generation (*the captive doesn't want rescuing*). Twists belong to six **groups** (*the other side is right*, *you started it*, *innocents pay*, *they don't want it*, *not what it seems*, *someone's hiding something*) that limit repetition. Every twist is in exactly one group. |
| **Truth** | the hidden reality behind a card (who really did it). Usually secret. |
| **Ending** | how a card can finish. Each family has a fixed list. Which one happens is up to the player, and its effects belong to the mechanics layer. |
| **Fact** | a true statement about the world: "the quay is seized", "Hesper is held at the pens". Facts are **public** or **secret**. Everything the system knows is facts. |
| **Plant** | a fact a card deliberately declares so a *later* card can depend on it, like a clue. A **payoff** is a later card that uses it. |
| **Move** | something a story's villain *has done* between chapters, recorded as a public past-event fact. Each move works through a **lever** (debt, law, rumour, force…). |
| **Registry** | the world's roster of people, places, things and factions, plus the links between them. |
| **Standing** | Low (peasant), Middle (tradesperson), High (gentry), Top (ruler; pre-made only: a canon person or an archetype's generated filler, §8.2). |
| **Scope** | how far a card's pressure reaches: household, community, region or realm. |
| **Relatedness** | how tied a card is to a big story: main, minor, fallout, echo, independent, or faction. |

---

## 3. The registry: people, places, things, factions, facts

Everything quests talk about lives in one registry.

**People** have:
- a name and a standing;
- traits;
- links to other people (family, kin, spouse, ally, employer/employee, rival, creditor/debtor, grudge, dependent, trade), each with a direction and a visibility (a secret link is how a hidden agent is hidden);
- a faction membership;
- facts about them;
- whether they're pre-made (canon people and archetype fillers, §8.2), and which **archetype** they fill, if any;
- whether they're dead, or **die at a known story move** (`dies_at`: alive on every card before that move, dead after);
- a home, and a **fallback** (a backup person, chosen by casting, who can take their place if a quest outcome removes them).

**Places** have:
- a kind, an owner, an access level (open, guarded, remote, fortified) and a standing;
- the map place they sit in (`at`);
- tags (*landmark*, *foundational*);
- facts;
- **affordance tags**: things the place must physically contain because a quest needs them (§11).

**Things** have a kind (heirloom, document, weapon, relic, goods, tool), an owner, a value and a location.

**Factions** are pre-made from the worldbuilding. Each has:
- a size and a parent;
- leaders (archetypes, §8);
- a seat and a reach;
- a want, a virtue and a wound;
- a hidden side (a secret fact);
- relations with other factions: *ally*, *rival*, *at war*, *trade* (dealings without loyalty), *serves*;
- facts.

**Rules that keep the registry sane:**
- **Add, never contradict.** A new fact may build on old ones but never deny them.
- **Secrets stay secret.** A card may build on a secret, but only a story's **revelation** card may publish one.
- **No conflicting states.** Someone held on one card can't walk free on another. The dead stay dead (except through `dies_at` timing).
- **Reuse limits:**
  - an ordinary person or place is used on at most **3 cards**, and at most **once per family**;
  - **authorities** (magistrates, ring-keepers, arbiters) can appear more often;
  - **story cast** are exempt inside their own story.
  - These limits stop the same blacksmith being kidnapped, robbed and accused in one world.
- ***Seats and offices have one holder.*** A seat or office (lord of a keep, wharf-master) belongs to one registry person; no card invents a second. Replacing a holder is an event with a fact.
- **State is written down.** When an ending changes a person (fled, changed sides, disgraced, took a new post), it writes a **state fact** on them that later cards read. Past events carry **when** they happened; things carry **who holds them, since when**. "The only record" is one registry thing, never invented twice.
- **Settlement context.** Each settlement keeps a list of public facts that the big stories did to it (a salt crisis, a war writ). It never gates a quest, but at least one local quest in each touched town refers to it, so towns notice the world around them.
- **Open threads.** A loose end, such as the hook's stolen mules, is recorded as an open thread that some later card must close.
- **Settlements are containers.** A card set in Openharbour happens at a *site inside* Openharbour (a quay, a shop, a house), so a big town can hold many quests without hitting the reuse limit.
- **Names:**
  - full names are unique world-wide;
  - first names don't share a stem within one card, story or location, and are unique within a region among people on story cards;
  - kin share a family name;
  - canon names are exempt.

---

## 4. One card, start to finish

How a single card is rolled. Every random choice uses a **keyed roll** (§9), so a world rebuilt from the same seed comes out the same.

1. **Family.** Rolled from weights. Those come from the location's features (a port leans toward Acquire and Dispute, a farm toward Restore and Threat) or, for a faction-given job, from the faction's profile.
2. **Premise.** Rolled within the family, plus the giving faction's signature premises.
3. **Requirements filter.** Each premise lists what it needs: *Bound by debt* needs a creditor/debtor link; *Under seal* needs an authority holding the thing in a guarded place. Anything the card can't satisfy is filtered out before rolling, never patched afterwards.
4. **Slots become specs.** Every role the premise needs becomes a **job description** for casting: "a captor, Middle standing, lawless, with a hideout within a day of here". Specs can point at each other: "the captive's kin". The card stores it in its `specs` list; casting fills it later (§8). Until casting exists, a card resolves its own slots in the same order: search, reuse if it fits and the limits allow, otherwise create.
5. **Separation.** In money matters (debt, ransom, trade deals, inheritance) the two main parties are at most one standing step apart. A peasant doesn't ransom a lord.
6. **Reasons and motives.** Each party's reason is rolled from the family's list (Rescue captor reasons: ransom, leverage, revenge, holding them for someone else…).
7. **Ways through.** The premise's ways are pooled. Ways the card can't support are removed and logged: no bribing a Top-standing target, no assault on a fortified place. Then 2–3 are rolled (a faction crisis rolls exactly 3), covering at least 2 kinds, in a fixed order: forced ways first, then a way that reaches the truth, then **plant-backed ways** (ones that pay off something planted earlier, like an ally's offer; rulebook, *Plant-backed ways roll first*), then random ones. Every way names a person, place or thing from its own card; no generic template ways.
   - **Forced ways never give their place:** *Old wound*, a required ally way, the climax's sacrifice way, the crisis's code-break way. If two share a kind, the next way rolled is another kind. When slots run short, the methods preference yields first, then the heart's-secret way (offered as the twist instead).
8. **Twist.** Rolled from the family's twist table, filtered by requirements and by the story's twist-group budget. *None* is a valid result.
9. **Truth.** Rolled where the card's ways involve finding something out. ***No blank card:*** every card carries a hidden truth or a real twist; twist *none* is legal only on a card with a truth. ***The truth is reachable:*** at least one way reaches it. ***Premise must fit the place:*** no country estate in a reef village.
10. **Endings.** The family's static list, minus any ending the card's own filters make impossible.
11. **Plants and dependencies.** The card declares what it plants for later cards, and lists the facts it depends on. The **plant rule** checks that every dependency was planted somewhere the player must already have been.

**Card text only says what's true then:** a card's words may use only facts its dependencies, plants or settlement context make true at its time. Directions and travel times are filled in from the map, never written freehand.

The result is a small block of data: a few hundred bytes.

---

## 5. The nine families

| Family | The situation | Kinds of ways |
|---|---|---|
| **Rescue** | get someone out of where they're held | force, stealth, price, leverage |
| **Investigate** | find out who did something, and why | question, examine, follow, provoke |
| **Dispute** | two partly-right parties, one stake | truth, deal, side |
| **Threat** | something endangers a community | confront, undermine, protect, bargain |
| **Passage** | the cargo itself makes the journey hard (anti-escort: it's forbidden, hunted, unwilling or unwelcome) | route, cover, force, arrangement |
| **Acquire** | obtain a specific thing someone else holds | take, deceive, trade, earn |
| **Reckoning** | a known wrong that justice failed | justice, ruin, amends, violence |
| **Restore** | something relied on is broken | craft, gather, persuade, substitute |
| **Contest** | a matter sworn to a bout, race or match, where winning is hard or wrong | prepare, rules, edge, stakes |

**Design rules every family follows:**
- **The situation, not the action.** A quest is who wants what, what's in the way, and what makes it hard.
- **At least one debatable premise or twist** per family, so the right choice isn't obvious.
- **Every person a way names** (a guide, a witness) is an optional slot, filled only if that way is rolled.

---

## 6. How cards join into stories

### 6.1 The main story (15–25 quests on one path)
- **Shape: a diamond of layers.** The story has L layers (5–8), each a row of **nodes**, widest in the middle. With L=5 and W=3 the widths go 1-2-3-2-1. The player passes through **one node per layer**. Unchosen nodes stay open as optional content.
- **Each node is a chapter** of 2–4 cards: an opening, middles, and a turn card that does the chapter's job.
- **Roles by layer:**

| Role | Layer | Job |
|---|---|---|
| **Hook** | 1 | the villain's side harms the heart |
| **Threshold** | 2 | the player first openly opposes them |
| **Trials** | middle layers | learn the methods; gain allies and proof |
| **Ordeal** | second-to-last | the villain wins something, and it costs the heart personally: captured, disgraced, or estranged from kin |
| **Climax** | last | face the villain |
| **Return** | after the climax | homecoming, changed: repair what the hook broke |

- **The cast:**
  - an **antagonist**, with a *shape*: *Above* (a lord), *Beside* (a rival trader) or *Within* (someone close to the heart);
  - their **want**;
  - the **stake**;
  - the **heart**: the ordinary person the player is doing this for;
  - the heart's **fallback**;
  - the **Hand**: the villain's recurring face;
  - a **lieutenant**.
- **Moves.** Between layers the villain *does something* (a move), recorded as a finished past event. Each chapter opens in the world that move made. Moves are what make the world visibly change as the story goes on.
  - Each move names a **lever**. The story's lever is the last move's lever, and it is reserved: all the guild stories together may use it at most once.
- **Scale arcs.** Rolled per story; they set how scope changes across the layers:

| Arc | Direction | Scope changes like this |
|---|---|---|
| **Rising** | up | village → region → realm |
| **Ripple** | out | one scheme seen in many communities |
| **Inward** | in | a war that narrows to one household |
| **Deep** | back | an older wrong under the community's foundations |

- **Plants and payoffs.** Each layer's turn cards plant a **shared fact** with a named **knower** (a person who can attest it). The **revelation**, the villain's secret intent, is planted in the layer before the Ordeal. The climax must pay off at least two plants from different layers, the revelation among them.
- **Epic weight:**
  - **Set pieces:** sieges, successions, embassies, schisms;
  - **allies with a price;**
  - a **fault line** inside the villain's faction;
  - a **sacrifice way** at the climax: winning by giving something up.
- **Minor quests** hang off chapters and share a person with them. Each declares an **advantage**: a fact that helps a later node.
- **Fallout quests** show a move landing on people *outside* the story. There are 1–3 per move by scope, covering the three faces *profit*, *resist* and *endure*, plus one card from the enemy's own people and one **mirror** card (someone else in the heart's position). **Late fallout** adds one more per big move, which opens only if the player dawdles (§10.5).
- **Spread rules:**
  - the heart appears in every layer;
  - no town hosts more than 5 main cards;
  - the climax pays off its ally;
  - no premise appears more than three times.

**Rules added after the world test (v5):**
- **Meet them before they matter.** On every path, the villain, the lieutenant and anyone the climax needs are met in person on an earlier card. Being the absent owner of something doesn't count.
- **The heart on every layer, on every path.**
- ***Chain and climb.*** Each card follows from the last one's result, and the pressure rises.
- **Wins stick.** A move aimed at something the player already settled changes how it plays out instead of undoing it. The Return has versions for each climax ending.
- **Every taking gets a rescue; threads close.**
- **Plants pay off.** The climax ally way is always rolled.
- **The villain's want survives the last move,** and a revelation's named cost is paid later.

### 6.2 Faction stories (one per joinable faction, 9–12 quests)
Elder Scrolls guild questlines, generated. They start as **routine jobs** that turn out to have been one story.

| Phase | Cards | What happens |
|---|---|---|
| **Routine** | 3–4 | ordinary guild jobs. Each secretly carries a **quiet plant** (an innocent detail: a black cord, a lamp number, a date). One is the **odd job**, where the villain's agent is hidden. |
| **Reveal** | 1 | the quiet plants connect, and the hidden story is published |
| **Story** | 2 chapters | the enemy moves before each one. Each card chains to the last, and pressure climbs. Chapter 2's move costs the heart personally. |
| **Crisis** | 1 | at the guild's headquarters. It rolls exactly 3 ways, and one is always a **code-break**: breaking the guild's own code. The player chooses where the guild goes next (the faction endings). |
| **Epilogue** | optional | repair what the last move broke |

- **Profiles make guild jobs feel like that guild:**
  - family weights;
  - **signature premises** only that guild offers;
  - **methods** (preferred kinds of ways);
  - a **code** that forbids certain ways and endings ("never inform");
  - vocabulary (what they deal in, where they work, who comes to them);
  - **faction-only twists**.
- **Story kits** give variety. Each guild has 5–6 enemies (each with a shape, a want and a revelation *kind*: *the hidden hand*, *the cost*, *the old debt*), 4–5 stakes, 4 hearts, 3 odd jobs and 4–5 crises. Rolling one of each, with compatibility rules, gives hundreds of distinct spines per guild.
  - **Kits name leader archetypes, not people** ("the Lantern leader archetype"); a canon name appears only as the default filler (§8.2).
  - **Every signature premise carries a truth,** as its family requires (*No blank card*).
- **Story craft rules** (learned in testing):
  - the enemy and their agent appear before the crisis;
  - every move advances the revealed scheme;
  - anyone a move kills was met earlier;
  - each move uses a **lever** (debt, law, rumour, force…), at least two different ones per story;
  - people moved or turned on one card can't turn up somewhere contradictory on the next.
- **World variety ledger** across all guild stories:
  - at most two reveals of the same family;
  - each premise and each twist at most twice;
  - at most two last moves that kill someone;
  - levers spread out; the main story's lever at most once in all guild stories together.
- **The rival syndicates** (Old Hands and Night Market, canonically *at war*) may be generated as a pair: two views of one feud, ending at the same deadly truce. Their crises differ.
- **Kept smaller than the main story:** region scope at most, no foundational stakes, 9–12 cards.
- **One crossing with the main story.** A guild story may use the main story's High people freely as authorities and in the background. It may give **one** of them a meaningful role (a client, an opponent, a knower); that is its single crossing, and that person may be the story's *Above* enemy (a lord the main story already set up). Never main-story cast or proxies, and never a main-story secret.

**Rules added after the world test (v5):**
- **Event ceiling.** A guild story never outweighs the main story; the enemy acts through local tools.
- ***Faction fallout is required.*** A move that reaches beyond the guild spawns at least one fallout quest in a town it touched; at most one per move, so at most 3 per story.
- **Short steps inside a phase or chapter; journeys between them.** Inside a phase or chapter, each card is within about a day of the one before. In the routine phase only, one step may be long, and the next card comes back near. Phase 3's journey is a chapter's opening card.
- **Kit variants.** Every kit entry has 2–3 versions, so the same roll doesn't repeat a story.

**Guild stories meet (Laz: medium overlap).** World B's five guild stories barely shared a town, so joining one guild could leave a whole part of the island unvisited. Now all guild stories are placed together, by one plan, before any of their quests are made:
- **Each story crosses the map:** it touches at least 3 regions. It moves at phase and chapter boundaries, never inside a chapter.
- **The plan places every routine card** (in at least 2 towns), each chapter, the reveal and the crisis.
- **Shared towns:** each story puts at least 2 quests in shared towns, which hold two or more guilds' halls (every large town qualifies) and are also used by another guild story. They use different sites inside the town.
- **Rivals and allies meet:** guilds have ties (rival, at war, ally, trade). Every rival or allied pair gets one quest where a member of the other guild plays a real part: an opponent, a competitor, an ally, a guide. It is set in a shared town or at a field site both guilds use. Neither story needs the other to have been played. If the player belongs to both guilds, it becomes a choice of loyalty. This isn't a main-story crossing.
- **Across all guild stories,** every region with a guild presence gets a guild quest, and no region gets more than about a third of them.
- ***World ledger.*** One variety count shared by every pass.

### 6.3 Side quests (the location pass)
- **Every place on the map has a quota:**

| Place | Quests |
|---|---|
| village | 1–2 |
| town | 3–5 |
| large town | 6–10 |
| great town | 10–15 |
| presence (a shrine, a camp) | 0–1 |

- Story cards pinned there count toward it; the location pass fills the rest. A presence inside a town shares the town's count.
- ***Per-place caps*** count every card pinned at a settlement, story cards included: a hook or landmark is the focus of at most 2 cards, a theme of at most 3, no family above 30%, no premise more than twice.
- ***Settlement context:*** in every town a big story touched, at least one side quest refers to what it did there.
- **Families lean on what's there:** ports breed trade quests, junctions breed Passage and raiders, the green's edge breeds Threat and Rescue.
- **Each card should tie to its place:** its landmark, trade, hook or faction presence. A quest that could happen anywhere is a weak quest.
- **Kinds:**
  - **echoes** reuse a person from a story card;
  - **linked pairs** join two neighbouring places;
  - **one-off faction jobs** come from guild halls and presences, through the profile.
- **Side quests stay at community scope,** and at most 1 in 5 sends the player more than a day away.

### 6.4 Clocks
- **The world waits for the player.** Only about 1 in 10 side quests carries a clock, and only premises with a natural deadline.
- **What a clock is:** a checkpoint, like a season, plus a **world ending** rolled at generation (*the captive is moved elsewhere*), which plays out if the player doesn't get there first.
- **Story quests never carry clocks.**

---

## 7. The map and places

- **Placement is relational,** not by compass. Faction territories and places are placed from a **relationship map**: what each faction needs from the land, what clusters, what sits next to what, and what must never touch (the Walking Road, the slave road, never crosses Open Hand land), with fallbacks when the terrain doesn't exist.
- **Fixed and generated places:**
  - **fixed canon places:** the Ninth Ring, Highcrossing, Lanternhall…
  - **generated holdings** per major faction;
  - **presences** that dot the land: wayside shrines, Warden towers, Binder camps, road forts, challenge stones, Writers' digs.
- **Pinning stories:**
  - the main story's layers sit a real journey apart, so the player crosses the map past side content;
  - guild routine jobs are offered at halls and contacts in two or more towns;
  - guild chapters happen at field sites;
  - crises happen at headquarters.
- **No quest is ever pinned at a secret place** (the Womb, the Heart).
- **What the land generator must provide is listed in `worldgen-requirements.md`:**
  - a score for each feature (harbors, heights, fertility, river crossings, the Overgrowth's edge);
  - roads with traffic;
  - travel times;
  - territories.

---

## 8. Casting: who everyone actually is

Quests don't name people when they're generated. They post **job descriptions** (specs), and a separate **casting pass** fills them, one layer of power at a time (§8.4). This is what makes the world feel connected (the same lord in three quests) instead of populated by strangers. The full rules are in `quest-casting.md`.

### 8.1 Job descriptions (specs)
A spec says what a quest needs:
- **Hard requirements:**
  - standing;
  - a class (*Banner Lord*, *house steward*, *Lantern captain*, *ring-keeper*);
  - faction or sub-faction;
  - traits (*lawless*, *holds legal authority*);
  - required links (*creditor of the requester*, *kin of spec #12*);
  - location (*lives within a day of the Chainmarket*);
  - timing (*alive until move 2*);
  - not in a conflicting role elsewhere.
- **Preferences:**
  - same region;
  - already connected to this story;
  - wealthy;
  - a woman (for canon-gendered roles like Qotiro priestesses this is a hard requirement).

Place specs work the same way: *a toll bridge near Southroad, owned by a Banner Lord, guarded*.

### 8.2 Classes and archetypes
- **Classes, not fixed names.** Banner Lords, Free Lords, stewards, chapter captains, inquisitors and ladder-keepers are classes. Each world generates its own members, so two feuding Banner Lords are different people every playthrough.
- **Canon leaders are optional archetypes** (world setting `canon_leaders`: always / mostly (default, about 70%) / sometimes / never). Each canon leader is an **archetype with a default filler**. For example, *Dueling Society co-founder who once killed in a duel*: default Kitim.
  - Per world, each archetype either uses the canon person or **generates a new person** who keeps the traits and secrets stories depend on.
  - **Either way the filler is pre-made:** it exists before any quest, and counts as pre-made wherever the rules say "pre-made only" (a generated emperor is legal).
  - **Story kits refer to archetypes, not names.** An entry that needs an archetype this world lacks is filtered out.
  - A few archetypes are **guaranteed** because the late-game secrets run through them: someone who has scouted the green (Hoqa's role), someone guarding the Womb (Shihi's role).
- **Court state (made by casting).** Before the main story, casting picks 3–4 **live tensions** from the leaders' canon foils (the Banner Lords' succession race, the high priest against the sincere priestess, the Tide speaker's term running out). Each is seeded as public facts naming the archetypes involved. Specs then *prefer* people already caught up in a live tension, so the world's quests orbit the same politics.

### 8.3 The matching
Each layer's specs are cast together (`quest-casting.md` §6):
1. **Group** specs that point at each other ("kin of the captive"), so the link holds.
2. **Hardest first:** the group with the fewest possible fillers goes first, so an easy spec never takes the only person a hard one could use.
3. **Filter:** needs, reuse limits, conflicting roles and states, separation, conflict of interest, the story limits (one crossing per guild story; main cast never in a guild slot), names.
4. **Score:**

| Factor | Points |
|---|---|
| already linked to someone on this card | +3 |
| on another card of the same story | +2 |
| same region | +2 |
| lives near | +1 |
| in a live court tension | +2 |
| a grudge-holder, for an opposing slot (during play, §10.11) | +4 |
| each preference met | +1 |
| each card already used on | −1 |
| each link that must be added to fit | −1 |

5. **Reuse or create:** creating scores a flat 1.5, so an existing person wins only with a real reason to be there. The aim is 25–45% reuse, tuned by that one number.
   - **Seat holders, archetypes, Top people and map places are never created here.** Seat holders and map places are looked up; archetype fillers and Top people come from layer 0.
6. **Add needed links:**
   - working links (creditor, employer, ally, rival, trade, grudge) between anyone;
   - family links only to someone newly created, who takes the family name.
7. **Stuck:** undo recent choices, up to a budget of 50 per layer. Then the card **repairs** itself by re-rolling the slot's alternative, then its twist, then its premise, at most 3 times. A side card that still fails is dropped and refilled; a story card never drops (that would be a logged generation error).
8. **Fallbacks:** anyone reused in a role another card could end (captive, fugitive, culprit) and needed later gets a linked backup.

Every tie is broken by a keyed roll; nothing depends on list order.

### 8.4 Layer by layer
Generation runs **top-down by power** (Project `quest-locations.md` §7a, *layers of power*). Each layer may read and reuse what the layers above it made, never the reverse, and nothing lower contradicts anything higher. **Casting runs layer by layer:** each layer's specs are cast before the next layer is generated, so a guild job can involve the lord the main story already set up instead of inventing a second one.

| Layer | Made | Then |
|---|---|---|
| **0. World** | map, canon factions, leader archetypes and their fillers | — |
| **1. Court state** | the live tensions (§8.2) | made by casting |
| **2. Realm** | the main story, its Return, minors and fallout, all as specs | cast |
| **3. Factions** | every guild story, as specs | cast |
| **4. Communities** | the location pass: side quests, echoes, linked pairs, one-off faction jobs | cast |
| **5. Households** | the background pass | — |

- **Fallout is made with its story.** Only echoes, linked pairs and location cards wait for cast people.
- **Layer-3 reuse of layer-2 people** follows the one-crossing rule (§6.2).
- **Repair** that re-rolls a card re-casts only that card.

**It always finishes:** each layer only *adds* cards and entries, never changes those above it, and quotas cap the total.

---

## 9. World creation, end to end

1. **Land:** terrain, rivers, roads, the Overgrowth (land generator).
2. **Places:** territories, fixed places, generated holdings and their landmarks, presences, all by the relationship map. Each place is saved with its **descriptors** (*large*, *well defended*, *river-fed*) and a landmark, trade and hook.
3. **Canon:** factions, archetypes (canon person or generated), then court state, made by casting.
4. **Quests and casting, layer by layer** (§8.4):
   1. main story, Return, minors, fallout; then cast them;
   2. faction stories: every guild's kits and cast, then one placement plan for all guild stories, then their quests; then cast them;
   3. the location pass by quota; then cast it;
   4. clocks.
5. **Background pass:** ordinary people and places that fill out each town (shared with the sim's crowd filler).
6. **Validate:** the mechanical checks below, every one cheap.
   - every reference resolves;
   - reuse limits, names and states hold;
   - every plant is paid off or deliberately seeds something;
   - every revelation is published; every move reaches a card;
   - every settlement has at least one quest (presences and wild sites may have none), and every region holds at least one story card (main or faction);
   - the variety ledgers hold.
   - **On a failure,** repair the offending card (re-roll that piece) and re-check that card only.
7. **Save.** Everything above is plain data in the save file.

**Determinism.** Every roll is keyed by what is being decided: world seed, story or card id, field name. It is never drawn from a running random stream. The same seed always builds the same world, and a card's rolls don't change if an unrelated card is added. This is the sim's existing rule 2, applied to quests.

**Size.** A world of about 300 cards, about 500 people and about 150 places is a few hundred kilobytes, built in seconds at most.

---

## 10. The mechanics layer: what happens during play

### 10.1 Facts and signals
- **The whole game state is facts,** plus each card's state.
- **When anything changes a fact,** whether the player's action, a quest ending or a simulation event, a **signal** goes out.
- **Quests subscribe to the facts they care about.** No system needs to know about quests except through facts.

### 10.2 Quest life cycle
| State | Moves on when |
|---|---|
| **hidden** | its availability rule becomes true → *available* |
| **available** | the player accepts → *active*; a close rule fires → *closed* |
| **active** | an ending is reached → *resolved*; a forced close → resolved with an unattended ending |
| **expired** | a clocked card's checkpoint passed first; its world ending plays out |
| **closed** | a key person is gone, or the quest became moot |

### 10.3 When quests open
| Quest kind | Opens when |
|---|---|
| **Main story** | Hook from the start. Inside a chapter, each card when the one before resolves (any ending). A layer when a linked chapter's turn card resolves. A layer's **move becomes a world fact when the layer opens.** |
| **Minor quests** | open with their chapter; close when their target node resolves |
| **Fallout** | opens when its move becomes a fact |
| **Faction stories** | routine card *n* when card *n−1* is done and guild rank is at least *n*; then reveal, chapters and crisis in order |
| **Side quests** | from the start, or when the fact they depend on exists |
| **Linked pairs** | card B when card A resolves |

Discovery (how the player hears of an available quest: being approached, rumours, gossip, notice boards) is its own system reading these states.

### 10.4 Endings write effects
Every ending maps to a short list of **effects**, and these are the only effects there are:

| Effect | What it does |
|---|---|
| `set_fact` / `clear_fact` | adds or removes a fact |
| `set_state` | free, held, dead, disgraced or fugitive for a person; broken, seized or abandoned for a place |
| `move` | a person or thing is now somewhere else |
| `transfer` | ownership of a thing or place changes |
| `link` | a new link, such as the player gaining an ally or a grudge |
| `reputation` | a change with a faction, sub-faction or named person |
| `open` / `close` | another quest opens or closes |
| `generate` | the generator builds **one** new card from the new facts |

**Defaults per family** (an ending's own text can add more), for example:
- **Rescue *freed*:** the captive goes home, and the requester becomes the player's ally.
- **Dispute *A wins*:** the stake goes to A, and B gains a grudge against A.
- **Threat *community lost*:** the place is abandoned, and its people move (may generate a follow-up).

### 10.5 Time: two clocks
- **World time** passes in **seasons** (checkpoints). Clocked side quests expire when their season passes.
- **Story time** is how far the player is in the main story. **The main villain never moves without the player:** moves happen as layers open.
- **Late fallout** needs both: its season has passed **and** the player hasn't reached the next main layer. So the world visibly gets worse if you dawdle, without the story running away from you.
- **Faction-story moves** follow the player's progress in that guild.

### 10.6 When a key person changes (the situational table)
When an effect, or a simulation event, kills, imprisons, disgraces or turns someone, every other quest that uses them checks this table. What happens depends on **their role there** and **how far along that quest is**.

| Their role in the other quest | Quest not yet active | Quest active |
|---|---|---|
| **The one who wants it** (requester, heart) | swap to their **fallback** if one shares the want; otherwise close | the fallback takes over; otherwise resolve with an unattended ending |
| **The opposition** (captor, holder, culprit, raider leader) | close as moot; **generate** a follow-up if it leaves a hole (a dead holder's heir now has the thing) | resolve with the matching ending (a dead captor means the captive walks free) |
| **The target** (captive, cargo, the missing) | dead: close, maybe generate a Reckoning · freed or moved: close as done | resolve with the matching ending |
| **A helper** (guide, witness, sponsor) | drop the ways naming them; close if fewer than 2 remain | the same |
| **An authority** | **recast** someone of the same class | recast |
| **Story cast** | **protected:** no other quest or simulation event may kill, imprison or turn them; such endings soften to *driven off* or *disgraced* | protected |

### 10.7 Simulation events
- **The simulation writes facts through the same signals** as the player: armies, hunger, trade, old age, the green spreading. A lord dying of old age and a lord killed by the player go through the same table.
- **Limits:**
  - the simulation never touches protected story cast;
  - it never contradicts a secret;
  - it shares the generation budget.

### 10.8 The world reacting: runtime generation
- **The generator runs in the game too.** `generate` builds one new card from current facts and the registry, under every shape rule, placed where the change happened.
- **Budget:** one follow-up per resolved quest, plus a few per season from the simulation, so nothing can cascade.

### 10.9 Reputation and membership
- **Reputation is kept at three levels:** per faction, per sub-faction, and per named lord. Entry to a faction's land reads the faction; quests and hostility read the sub-faction or person.
- **Changes:**
  - helping a requester raises it with their side;
  - acting against an opposing slot lowers it with theirs;
  - a code-break costs heavily with its guild.
- **Joining:**
  - joining a guild happens at its first routine card, and rank rises as routine cards are done;
  - joining one rival syndicate closes the other's story;
  - joining the Rooted removes every other membership.

### 10.10 The player in the world
- **The player is a registry person** with standing, links, reputation and memberships.
- **Endings link the player** to the people involved: the requester becomes an ally, the wrongdoer gets a grudge. That's how the world remembers them.
- **Minor-quest advantages** become facts that give a later main node an extra way or remove a hard part.
- **Allies whose price was paid** unlock their "call in your allies" way.
- **Deals that come back:** *deal struck* endings leave a debt fact with a due season. Unpaid, it generates a follow-up.

### 10.11 The grudge loop (recurring enemies and friends)
Enemies who survive the player remember them, learn something, rise a step, and come back. This uses parts that already exist, and is deliberately **not** a copy of WB's patented Nemesis System (US 10,926,179; reported valid to about 2035).

1. **Remember.** When someone in an opposing slot survives an ending (*driven off*, *fled*, *bought off*, *bargain struck*, *talked down*), that **same person** gains a grudge link to the player, plus a fact about how it went ("the player burned his camp").
2. **Change.** That **same person** rolls one change, drawn from that fact:
   - **a lesson:** *won't be bought twice* removes bribe ways against them;
   - **a scar:** *fears fire* adds a weakness way the player can learn about;
   - **a step up in the world:** a social standing step, recorded as a fact, not a combat stat.
3. **Return through quests.** In casting, grudge-holders score high for opposing slots in **new** cards (runtime follow-ups, late fallout). They come back as the captor of the player's ally, the raider on a town the player saved, or the wronged party in a Reckoning *against the player*.
4. **Death opens a quest, not a successor.** A killing is a fact. Like any death, it may lead the generator to build a *Blood debt* card whose requester is the dead person's kin. No other character is changed by the killing itself.
5. **Friends too.** People the player helped hold an ally link and turn up as ally ways later.

**Limits:**
- **At most 5 active grudges;** the weakest fade into the background.
- **Story cast are protected;** only people outside the main and faction casts can become recurring enemies.
- **Returns share the generation budget.**

**Design-around notes:**
- The patent's main claim covers an interaction with one enemy changing a **second** enemy's parameters, with the change shown to the player. Every change here applies only to the person the player dealt with.
- **No army hierarchy, promotion ladder, enemy power struggles or rank screen.**
- **Changes surface through dialogue, rumours and quest text,** never an "enemy upgraded" display.
- **Returns come through quests,** not roaming hunters.
- **Have a patent attorney review this before release.**

---

## 11. On approach: the detail pass

- **What's saved up front** is a place's **descriptors** and its **affordance tags**:
  - **descriptors:** *large stronghold, well defended, river-fed, decaying*;
  - **affordance tags:** things quests promised are there, like *a drain nobody guards*, *a cell in the lower yard*, *a back gate*, *a cellar under the street*.
- **When the player comes within range** of a region or place, its physical detail is worked out from them:
  - **Descriptor tables turn words into numbers.** *Large* means a garrison of so many, which needs so much housing and so many stores. *Well defended* means one of walls, a moat, or both, picked by a keyed roll.
  - **Affordances are built first, and must exist.** That is what makes a generated keep play the way its quest promised. If the quest's stealth way is "the drain nobody guards", there is a drain, and nobody guards it.
  - **The rest is filled** around them from the place's culture and kind (layout rules, building variants, the town forge).
- **Built once, then permanent.** The detail is keyed by world seed plus place id and saved once built, so a place never changes on a revisit. This is the sim's existing rule 4, applied to places.

---

## 12. Dialogue: a pre-written kit

- **No language model runs on the player's machine.** Dialogue is a large **kit of tagged pieces, written up front** during development by a strong model, reviewed by Laz, and shipped as data files (the sim's `data/lines/` format).
- **Pieces are tagged** by family, premise, slot role, twist, ending, the speaker's people and voice (Roduro slow and formal, Qotiro clipped, Horaro warm and indirect, Ṭaḍoro quick and curious), faction, standing, mood and trust level.
- **At runtime the game assembles lines** by matching tags, in the same way the sim's emergent dialogue already works (greeting + topic + feeling + hook + farewell). The most specific match wins, recently used pieces are skipped, and slots are filled from the card's facts and names.
  - **Lines can't contradict the quest,** because every name and fact in them comes from the card.
  - **A secret can only be spoken** by a speaker who knows it, after it's published, or under trust rules.
- **What the kit covers for quests:**
  - **the pitch:** a requester asking for help, by family, premise and reason;
  - **refusals and deals:** an opposing party offered a bribe;
  - **rumours and gossip** that point toward available quests (the discovery system);
  - **reveal lines** for knowers;
  - **ending reactions;**
  - **grudge-holders' lines** on meeting the player again.
- **Scale is fine,** because writing is automated offline. Thousands of pieces is a few days' generation and review, not a lifetime of writing.

---

## 13. How this fits the existing sim (rules in `CLAUDE.md`)

The quest system should follow the sim's load-bearing rules rather than bend them:
- **Keyed randomness** (rule 2): every quest roll is keyed by what's decided, never a running stream.
- **Outcomes don't depend on bands** (rule 1): a quest's ending and its effects are the same whether the place is near or far. Bands only decide detail.
- **Details are permanent once built** (rule 4): the on-approach pass builds once and saves.
- **Events run on the world's timeline** (rules 7, 18, 19):
  - quest availability, clocks and late fallout are checked at **dawn** (and at seasons);
  - generated follow-ups are made at dawn too;
  - nothing ticks per step.
- **Content is data** (rules 13, 19): families, premises, ways, twists, kits, profiles, descriptor tables and dialogue pieces live in data files. Code gathers facts and matches tags; it doesn't hard-code quests.
- **Everything is saved** (rule 12): the registry, facts and card states, with the save format bumped when the shape changes.
- **The emergent layer stays.** The sim's existing storyteller (`stories.rs`), opportunities (`chances.rs`), memories and grudges (`memory.rs`) and journal (`quests.rs`) keep producing small everyday jobs from people's lives.
  - **Generated quests are the authored-structure layer above them:** the big stories and every town's crafted quests.
  - **They should meet in three places:**
    - an available generated card appears as an opportunity in the journal;
    - the player's quest deeds become memories (`memory.rs`), feeding gossip and the grudge loop;
    - the storyteller's townsfolk can be cast into quest slots when they fit.
  - **The exact wiring is for the build** to work out against the current code.
- **Budgets:** per-person data stays small (the sim's about-200-byte budget). Quest links and grudges are short capped lists.

---

## 14. How it's tested

The test process exists already (Project: `quest-test-results.md`, `quest-faction-test-results.md`, the world tests):
- **A test island** with every faction placed by the relationship map.
- **Generators following the rules exactly,** logging every rule that was ambiguous, contradictory or dull. Until the Rust prototype exists, these are Claude agents standing in for the code.
- **Mechanical checkers:**
  - references, reuse, names and states;
  - plants paid off;
  - spread and variety ledgers;
  - coverage per settlement and region;
  - story cards per region.
- **A story rubric** (`story-rubric.md`) built from storytelling and quest-design principles: coherence (*but / therefore*), span (every place has something), depth, each unit's beats (incident, escalating complications, a real dilemma, payoff), and the whole's enjoyment.
- **Independent readers** who never saw the generation: a coherence reader, a story critic, and a simulated playthrough diary.

---

### 14.1 First results (2026-10-26)
Two test worlds: a guild suite (54 quests) and a full world (188 quests across all ten regions).
- **Story critic:** 4/5 on depth, units and the whole; the theme scored 5.
- **Playthrough:** 6.5/10. The guild stories are about 8, the main story about 5.
- **Coherence:** 3/5. Each story holds together, but the seams between stories contradict.

The fixes went into the rules as v5 (marked "added after the world test" above). Full write-up: Project `quest-world-test-results.md`. Several fixes (state facts, seat holders, travel from the map) are bookkeeping that code does better than agents, so the Rust prototype is the real test of them.

---

## 15. Building it (the Rust prototype, in words)

**Two agents build it in parallel:** Lane A does shape and mechanics, and Lane B does the story layer. Who owns which files, the branches and Lane B's brief are in the Project doc `claude/quest-parallel-plan.md`. The Python checkers and sample worlds used during design are in `tools/quests/`.

The first build is a **standalone generator**: a program that builds one world's quests from a seed and writes them out, with the checkers. Later it becomes part of the sim. Its parts, in the order they'd be built:

1. **Data:** families, premises, ways, twists, endings, reasons, profiles, kits, quotas, descriptor tables, all loaded from files.
2. **Registry and facts:** entries, links, facts, the add-never-contradict rule, reuse counting.
3. **Keyed rolls:** one roll function keyed by seed and path, shared with the sim's.
4. **The card roller:** §4, one card from a spec of where and why.
5. **Story assemblers:** main story (diamond, chapters, moves, plants), faction stories (phases, kits, ledger), fallout, the location pass.
6. **Casting:** specs, filter, score, create, backtrack, repair, run layer by layer.
7. **Checkers and repair:** the validators from §9, run after each pass.
8. **Output:** the world as data, plus a readable digest and the map view the tests already use.
9. **Then the mechanics:** facts and signals, life cycle, effects, clocks, the situational table, the grudge loop. Exercised by a **mechanics test** that "plays" a generated world with random choices and checks nothing contradicts itself, quests open and close sensibly, and grudge-holders come back.

---

## 16. Open questions

- How the player **discovers** quests: approached, rumours, boards, and how much is shown.
- **Reputation numbers** and thresholds; season length in days; the player's starting standing and links.
- **Which canon archetypes are guaranteed** besides the lore-critical ones.
- **The detail pass's descriptor tables:** garrison sizes, defence options, per place kind.
- **The dialogue kit's first scope:** which families, voices and moments to write first.
- **The Rites and Trials family,** parked until cultures exist.

---

## Changes
- **v1.1 (2026-10-26, audit fixes):**
  - Canon leaders as optional archetypes reach the registry: fillers count as pre-made, kits name archetypes (core H2).
  - One crossing with the main story; it may be the guild story's *Above* enemy (core H3).
  - Faction ties include *trade*; the syndicates are *at war* (core H4, content M11).
  - Slots are specs, filled by casting layer by layer; court state is made by casting before the main story; fallout is made with its story (core H6, M5).
  - Short steps inside a phase or chapter, journeys between them (core M2).
  - Moves name levers; the main story's lever is reserved (core M3).
  - Forced ways never give their place; the faction crisis rolls exactly 3 (core M4).
  - The validator checks settlements and story cards per region; "deep thread" is gone (core M6).
  - Six twist groups (core L2); "crossroads towns" are "shared towns" (core improvement 1); the Walking Road named (content L5); shared rule names match the rulebook; a "Not yet in the topic docs" line under Status (core improvement 8).
