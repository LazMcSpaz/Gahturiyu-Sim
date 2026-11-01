# Faction Profiles and Faction Stories — Draft (v5.3)

How joinable factions give quests that feel like *them*, and how each one gets its own overarching story. In Elder Scrolls style, many of a guild's jobs feel routine until a larger story is revealed.

- **Builds on:**
  - `quest-assembly.md`: cast, plants, ordeal events
  - `quest-main-quest.md`: chapters, moves
  - `quest-locations.md`: pinning
- **Guild content** (profiles and story kits) lives in `quest-guild-kits.md`.
- **Scope order (Laz):** the five guilds first, then the majors' sub-factions, then minors.
- **Six stories (Laz, 27 Oct):** the Holy House gets a full story you can join, like the five guilds. **Every rule here that says "guild" or "guild story" covers the Holy House too.** Its profile and kit are in `quest-guild-kits.md` §6; its places in `faction-places.md` §3 and §5b.

---

## 1. Two tools
1. **A profile** shapes every card the faction gives, whether it's a one-off job from a contact or a step in its story (§2–3).
2. **A faction story** is a smaller main quest that starts as routine jobs and turns out to have been one story all along (§4–9). Each joinable faction gets **one per world**, built from its **story kit** (§11). Guild stories are placed together so they cross the map and meet (§8b).

**Three kinds of faction-given card:**

| Kind | Relatedness | Scope cap | Rules |
|---|---|---|---|
| **One-off job** | a side quest (echo or independent) | community | every side-quest rule |
| **Hall job** | a side quest, given at a hall or contact of a joinable faction | community | every side-quest rule, plus locations §4 (*Hall jobs*). It counts toward rank (mechanics §9, *The guild ladder*), and never uses the story's secrets. |
| **Faction-story card** | **faction** (a new relatedness value, beside main) | **region** | this doc |

## 2. Profile spec

| Field | What it holds |
|---|---|
| **Weights** | per family: *high*, *normal*, *low*, or *never* |
| **Signature premises** | 2–3 premises on existing families, full spec in the kit doc, each with its own twist row and truth row. **Only this faction's cards may roll them.** |
| **Methods** | 1–2 preferred kinds of ways per family. When rolling, at least one rolled way is a preferred kind if the pool has one. |
| **Code** | forbidden ways, and forbidden **choice** endings. Applied to every faction-given card and logged in `ways_removed` with reason *code*. |
| **Vocabulary** | what the faction deals in (things), where its work happens (presence sites), and who comes to it (clients) |
| **Motives** | which rows of each family's reason and motive lists weight up, drawn from the faction's want, virtue and wound |
| **Faction twists** | 1–2 twists only this faction's cards may roll. Each has requirements, a family table and a twist group. |

## 3. How a profile shapes a faction-given card
1. **Family:** roll on the faction's weights instead of the location's (`quest-locations.md` §3). A *never* family is never given, and a *low* family appears **at most once** per faction story.
2. **Premise:** any premise of that family, plus the faction's signature premises. Weighted toward signatures: about 1 card in 3 uses one. For one-off jobs from a presence, or a fixed place that gives faction jobs, this is a **quota**, not a weight, and a small presence's first cards are free (locations, *Signature quota*).
   - If the rolled family has no signature premise for this faction, the preference is simply ignored for that card.
3. **Slots:** fill from the faction's vocabulary and presences first.
   - The requester is usually a client. Members fill slots only where the premise allows.
4. **Reasons and motives:** weighted by the profile.
5. **Ways:** the code removes forbidden ways first, then the card rolls with the methods preference.
   - **Filter:** if the code leaves fewer than 3 ways or fewer than 2 kinds, that premise is skipped for this faction.
   - **Added after the code:** the crisis card's **code-break way** (§6 phase 4). It is exempt from the code, and it doesn't count toward the 3-ways test. **The ending it leads to is exempt too** (going to the magistrate restores *justice done formally*; throwing the bout restores *thrown*).
6. **Twist:** the family's table plus the faction twists, minus any twist the kit filters for this faction. All twist-group limits apply.
7. **Endings:** code-forbidden endings are removed.
   - **A code only forbids endings the player would choose** (handing over, accusing, siding with, throwing a bout). Failure endings (*rescue fails*, *lost*, *destroyed*) always stay.

**Everything else is unchanged.** A faction-given card is an ordinary card of its family that has passed through a lens.

---

## 4. Faction story: the shape

| Phase | Cards | Job |
|---|---|---|
| **1. Routine** | 3–4 | ordinary faction-given jobs that look standard but each secretly carry a **quiet plant**; one is the **odd job** |
| **2. Reveal** | 1 | publishes the hidden story; the quiet plants connect |
| **3. The story** | 2 chapters of 2–3 cards each | the real conflict; the enemy acts before each chapter |
| **4. Crisis** | 1, plus an optional epilogue | wherever the story is (its centre, or where the plot comes to a head; at headquarters when the story is about it or its leader); the player steers the faction |

- **Length: 9–12 cards.** This replaces "8–12": 9 is the smallest total the phases allow.
- **One path:** a faction story is a chain, not a diamond. Each card may assume every earlier card was **reached**, never how it ended (rulebook §2.11).
- **The guild ladder (Laz, 27 Oct):** the story keeps its four phases, and the guild's **hall jobs** sit between its beats, about 20 jobs per guild in all (about 11 story cards plus about 9 hall jobs). Each beat opens at a rank, and rank rises with story cards and hall jobs alike. The rank rules are in mechanics §9 (*The guild ladder*) and §3 (*When quests open*); hall jobs are made by the location pass (locations §4, *Hall jobs*). Nothing in this doc generates hall jobs.

## 5. The cast

| Slot | Spec |
|---|---|
| **Enemy** | rolled from the kit, with an **enemy shape** (the story's `shape` field; not a rulebook truth *Shape*): |
| | *Within:* a member of the faction, High or Middle. Needs an ally, employer, family or kin link to the heart whenever the crisis is *Betrayal*. |
| | *Beside:* someone outside the faction, within one tier of the heart, with a rival, grudge or trade link to a faction member |
| | *Above:* a High or Top person outside the faction who is using it |
| **Want** | from the shared list (greed, revenge, cover-up, desperation, ambition, preservation), plus **faith** for guild-story enemies (v6c: someone who acts for what they believe the gods or the house demand). The enemy's **means** and **reach** come with it (§11a, *The throughline*). |
| **Stake** | something the faction can lose, from the kit. A registry entry: a place, a thing, a right, a reputation (a fact on a person **or on the faction entry**), or a bond. **Never foundational.** |
| **Heart** | rolled from the kit's heart list: a Low or Middle person with a tie to the stake. No way may harm or kill them, and no card describes them as dead. |
| **Heart's fallback** | a person linked to the heart. They fill active slots while the heart is captive (assembly §4's ordering clause), and count as cast **only** while standing in her place. If the heart is never captive, the fallback may go unused. |
| **Hand** | Middle, with an employer/employee link to the enemy. Under *Within*, the Hand may instead be linked to the heart. |
| **Leader** | the filler of the faction's leader archetype (`leaders.md`: the canon person by default, or a generated person who keeps the traits and secrets stories depend on) appears on **at least one** card: as its `giver` (the reveal or the crisis), in an authority-only slot, or as the enemy. Neither a giver nor an authority-only slot counts toward the cast slot limits (§8). When the rolled crisis variant names another giver, the leader gives the reveal or holds an authority-only slot instead. **Two leaders on two seats** (the Holy House's high priest at the Mount and the priestess at the Low Shrine): either one meets this rule, and both count as the faction's leaders for §9 *(Claude)*. |
| **Fault line** | required only when the crisis is *Schism*: a member whose virtue the enemy's want violates, planted before the crisis (main-quest §7.6) |

**Proxies in a faction story:**
- A proxy is anyone with a direct **working or personal** link to the enemy or the Hand: employer/employee, ally, family, kin, spouse, creditor/debtor. A *rival* or *grudge* link makes someone an opponent, not an agent.
- For an *Above* or *Beside* enemy, members of the enemy's own faction are proxies too.
- For a *Within* enemy, **sharing the guild is not enough**: otherwise every member would be a proxy.
- Main-quest §9's proxy definition doesn't apply to faction stories.

**Links the cast step creates:**
- A *Within* enemy whose crisis is *Betrayal* needs an ally, employer, family or kin link to the heart. If the rolled heart has none, the cast step creates one (usually employer, because the enemy outranks the heart in the guild).
- **Leaders** may fill authority-only slots (arbiter, ring-keeper, answers-to) in their own faction's story unless they are the enemy, even when the enemy is working around them.

**Heart types** (each kit lists which it allows):
- **the mentor:** the contact who recruited the player
- **the fellow recruit:** someone who joined around when the player did
- **the veteran:** an old member who remembers how things were
- **a client:** someone the faction serves, or failed
- **a member's kin:** family of someone in the faction

## 6. Phase rules
**Rank opens each beat.** The routine cards in turn, then the reveal, chapter 1, chapter 2 and the crisis each open at a rank (`rank_gate`), and hall jobs fill the gaps between them. The gates and their dials live in mechanics §9 (*The guild ladder*) and §3; hall jobs in locations §4 (*Hall jobs*). The phase rules below don't change.

### Phase 1: Routine (3–4 cards)
- **Faction-given cards** built by the profile (§3), at community scope or smaller.
  - At least one uses a **signature premise**. Decide which card at the start of the phase, and give that card a signature premise instead of rolling one.
  - No two back-to-back routine cards share a family.
  - No family appears more than twice in the phase.
- **They look standard.** No routine card's public text names the enemy, the stake or the story.
- **The heart's card:** decide at the start of the phase, like the signature card, which routine card carries the heart innocently (Story craft, *Meet them before they matter*). **It is never the odd job,** so no card needs the Hand and the heart together.
- **Quiet plant:** every routine card declares one plant of type *quiet*.
  - It's a public detail that's innocent on its face: a name, an object, a mark, a date, a route.
  - It is tied to the revelation, and it never states that card's own hidden truth.
- **The odd job:** exactly one routine card, never the first, has the **Hand or a proxy** in a hidden role, and leaves a loose thread.
  - **Hidden** means the card's truth names them, or they fill a slot whose link to the enemy is a **secret fact**. A public patron, holder or protector is not hidden.
  - The kit's odd-job entry forces whatever way or shield the role needs. Its family and premise **replace** the profile roll for that card.
- **Given at halls in the big towns** (§8b.2, *Hall work is in the big towns*): every routine card is offered at one of the guild's halls in a large or great town, and the routine cards and the reveal together use at least **3** of those towns. A routine card is set in its hall's town or beside it (§8b.2, *Beside a shared town*). Routine cards are hall work, so *Short steps* doesn't bind them (Story craft).

### Phase 2: Reveal (1 card)
- **Publishes the revelation.**
  - The revelation is a secret fact stored on the Hand or a proxy, naming what the enemy is doing and why, in line with their want.
  - It's recorded `published_by` this card. The no-leak rule still holds everywhere else.
  - It may also publish the secret hooks of **its own faction's** places (`faction-places.md`), and the faction's hidden side when the kit allows. It never publishes:
    - another faction's secrets;
    - anything in `hidden-lore.md`;
    - a routine card's own hidden truth (only the quiet plants connect). **Exception: the odd job.** The revelation naturally names the Hand, and exposing the odd job's hidden role is the point of the reveal.
- **Pays off at least 2 quiet plants** (`depends_on`).
- **Family:** any family whose card can carry the revelation as a shared plant with a knower. Investigate is no longer preferred: the world ledger (§9) allows at most two Investigate reveals.
- **Given by** the leader or the heart; when the leader is the enemy, by the heart or an elder member the kit names.
- **Given at a hall** in a large or great town, like the routine cards (§8b.2).

### Phase 3: The story (2 chapters)
- **Chapters follow main-quest §3:**
  - opening, middle and turn cards;
  - the chapter problem grows from the move;
  - the chapter payoff rule;
  - no two cards in a chapter share a family.
- **Moves:** one before each chapter. Each is a public past event and a finished act, at most region scope.
  - **Chapter 1's move:** the enemy openly acts against the faction. Its turn card is never Investigate, because a hidden culprit can't be the open opposition.
    - **It comes from the kit's move table** (v6c). Each kit has about 6 open moves, each a **public act** with `needs` (the means it takes), `hits` (what it damages), `aims` (the stakes it can be aimed at), `shapes` and `cost` (§11a): a rival hall opened next door, clients poached, a route closed, a member bought in public, a record called in by a court, a public challenge (§11).
    - **It is rolled with the kit** (§12 step 1), not invented at step 5. The roll takes only a move whose `needs` the enemy's means meet, and that passes the throughline test (§11a).
    - **Step 5 writes it** as the enemy's next step in the scheme, in the kit's words, with a lever its means allow (§11a, *Levers come from means*).
    - It still obeys the *Event ceiling* (§7), the lever caps and writ fatigue (§9).
  - **Chapter 2's move is an ordeal event with a target** (Laz, 27 Oct: "if it's always the heart … it kind of gets predictable"). The placement plan deals each story one **target + event** (§9, §12 step 2):

    | Target | Events |
    |---|---|
    | **the heart** (as before) | *captive*, *disgraced* (under the code), *estranged* (assembly §4) |
    | **the player** | *framed* (accused of a code-break they didn't do); *stripped of rank* (thrown out, or put down a rank until they win it back); *arrested* (held and judged) |
    | **the guild** | *hall seized* (a hall or the headquarters taken or shut); *members taken* (bonded, arrested or marched off); *outlawed* (a writ against the guild in one lordship or march); *leader struck down* (wounded, killed or disgraced) |

    - **Guild stories only.** The main story's Ordeal stays on the heart (Laz).
    - **The blow needs a means** (v6c): the plan deals a story only a target + event its enemy can carry out (§11a, *What each blow needs*).
    - **Its turn card is a door:** the family the door table gives for that target + event. **Home: assembly §4, *Guild chapter-2 doors*** (the heart's rows are assembly's ordeal table, as amended by main-quest §9). This doc doesn't repeat them.
    - **The door fits how the blow fell** (v6d): the plan deals a row of assembly §4's *Doors by how the blow fell* (held by men, for a debt or on a charge; wounded by men or through something rigged; a hall taken, occupied, sealed or shut by order; framed for a damage, a theft, a shaming or a broken contract). The row sets the blow's `uses`, its doors and what fills them (what is sealed, who claims the place, who is the cargo).
    - **The blow fits the heart** (v6d): only a `member` heart is *disgraced* under the code (`quest-guild-kits.md`, *Reading the kits*).
    - **A door's family must be one the profile allows.** A *never* family is never a door (§3): so *stripped of rank*'s Contest door is closed to the Writers, the Old Hands and the Holy House, and those stories take its Restore door.
    - **No two guild stories' doors share a family + premise** (they share one phase position, §9).
    - **Disgraced and framed are under the code:** the public accusation is a code-break (abandoned a contract, destroyed a text, informed, threw a bout, shamed a house, shamed the Holy House in public). For the heart, whether it's true is the door's cause; for the player, it is false by definition.
    - **The player as target:** the event acts on the player through rank and reputation (*framed*, *stripped of rank*: mechanics §9) or through the sim's guards and judging (*arrested*: mechanics §9 points there). The player is never cast: the door's slot for the one the blow falls on (the accused, the subject, the cargo, the wronged) points straight at `p-player`, and its other slots hold the people round the event (the accuser, the jailer, the witness) (assembly, *Guild chapter-2 doors*; mechanics §10) *(Claude)*.
    - **The guild as target:**
      - *hall seized* and *members taken* name one hall or one group of members, never every hall at once, except a realm-reach guild's own guild-wide act (§7, *Event ceiling*). The seized hall is a state fact on the site (§8b.2, *One state per site*). **A seized hall is still where a crisis set there is:** when the crisis falls at the seized hall or headquarters, the crisis card reads the site's state with a `before` face, so it holds whether or not the door took it back (locations, *Settlement context*);
      - *members taken* **never takes the heart**: if the heart is taken, the event is the heart's *captive*;
      - *outlawed* is a local instrument and binds only inside its own lordship or march (§7, *Event ceiling*);
      - *leader struck down* **never hits the story's enemy.** Each kit's *Leader struck down* line says which leaders may be hit, and how (`quest-guild-kits.md`). A leader the rest of the story needs (an authority slot, a Schism's judge, an ending that rests on them) is wounded or disgraced, never killed;
      - **a killed leader** carries `dies_at` (this move), is met on an earlier card first (the reveal, say; Story craft), is never the crisis giver (the rolled crisis variant must name someone else, or the variant rerolls), and leaves a **seat successor**: the cast step names who holds the seat after the move (rulebook, *Seats and offices have one holder*). The successor fills the leader's later roles.
    - **The heart is still in every story:** on a routine card, behind chapter 1's ally plant, and in the crisis. Only chapter 2's blow moves.
    - **The Restore door** (*Lost good name*) is barred whenever the epilogue repairs the disgrace, on whoever it falls.
- **Opposing slots:**
  - chapter 1's turn card: the Hand or a proxy;
  - chapter 2's turn card: as its door requires.
- **Plants:**
  - **chapter 1's turn card** plants an **ally plant**: a person with a reason to stand with the heart, and their price;
  - **chapter 2's turn card** plants one fact the crisis will pay off. If the crisis is *Schism*, this is the **fault line**.
- **The private side shows:** at least one phase-3 card takes its twist, reason or truth from the faction's **wound or hidden side**.
  - A faction with neither uses its **code**: a card where keeping the code costs someone.
  - A card builds on the hidden side but never states it. Only the revelation may publish it, and only if the kit says so.
- **Pinned** at **field sites**: any presence out in the land, any village or holding, or any place outside a town. At least one is a journey away: a chapter's opening card, a journey from the card before it (the chapter boundary).
  - **Door cards are exempt.** Their place is set by their family (where the captor holds the heart, for example).
  - **One shared-town card:** one phase-3 card may sit in a shared town instead (§8b, *Shared towns*), at a site inside it.

### Phase 4: Crisis (1 card, plus an optional epilogue)
- **The last move** comes before it: a finished act, as in main-quest §5.
- **The crisis card**, wherever the story is (v7c, Laz, 27 Oct): at the story's centre, or at the place where the plot comes to a head, and **the giver and the parties are there**. Nobody rides days back to headquarters unshown. Only a story about the headquarters or the faction's leader keeps its crisis at headquarters, and its card says why each party is there:
  - **its clock comes from the map:** a deadline on the crisis or its last move is at least the road from where the parties are to where the crisis falls, plus a day, never a number written into the story *(Claude)*;
  - **given by** whoever the rolled crisis variant names (the leader, the heart, an elder member, a survivor, a client), so crises don't all open the same way. The leader rule (§5) still holds;
  - **the giver has a reason** (v6d): the heart, her fallback or kin, a leader, an elder member the kit names, or someone the last move touched, tied to the story by the variant's bracket (on a stake; with some hearts). A captive heart never gives it;
  - **the heart fits it** (v6d): the crisis's `heart` tag names the hearts it can take (a carrier for a cargo, a copyist for a false copy, a fighter for a bought rung). A crisis tagged *herself* (the heart accused, sold, carrying or taken) never follows a *captive* heart, and its family + premise is never the chapter-2 door's;
  - the enemy fills the opposing slot (for Contest, the other principal);
  - **it is personal:** the heart or her fallback fills the requester, wronged or party slot. If the crisis is a Reckoning, **the wrong is personal harm to the heart**, never only the taking of the stake (as assembly §4 Climax);
  - it pays off the revelation, plus at least one phase-3 plant;
  - **family:** set by the kit's crisis. Each kit has at least two crises outside Reckoning (§11).
  - **No fixed enemy list** (v6c): a crisis may roll with any enemy whose means meet its `needs`, whose shape its `shapes` allow, on a stake it `decides`, and that passes the throughline test (§11a). Kit `limits` name an enemy only where the crisis is about that one (Old Hands C4, the old don's letters). Two crisis kinds always need a certain enemy: the paired truce (§10), and those that need a link the heart trusted: *Schism*, and *Betrayal* (a *Within* enemy, or one with `kin` the cast step can link by marriage) *(Claude, from the drafts)*.
  - **Schism** (main-quest §8) is allowed as a crisis, and nowhere else in a faction story. It **needs a *Within* enemy** (v6c: a split inside the guild needs someone inside to split it), and it is **barred when the enemy is the faction's only leader**, since then nobody is left to judge.
  - its rolled ways always include one **code-break way**: winning by breaking the faction's own code. It takes the sacrifice-way slot in the pool cap.
  - **Ally way (required):** the crisis pool always gains the "call in your allies" way, naming chapter 1's ally plant and its price (as main-quest §7.5), and **it is always among the rolled ways**, so the ally plant is never left unpaid.
  - **Plant-backed ways roll first** on the crisis and on every turn card (see rulebook, *Plant-backed ways roll first*): ways that pay a plant already in the world are rolled before random ones.
  - **Exactly 3 rolled ways** on the crisis (the code-break way, the ally way, and one more). Forced ways never give their place (rulebook, *Roll order*). **Exception:** when a plant-backed way would be left out, the crisis rolls a 4th (rulebook, *Roll order*: forced ways overflow).
  - **Faction endings:** besides its family's endings, the crisis carries **3 faction endings rolled from the kit's pool of 5–6** (v6c), as labels only. What they change is mechanical-layer work.
    - **Only endings that fit are rolled:** an ending's `fits` must name the stake's `hit`, the crisis's family, or the crisis itself; *any* fits every crisis (§11a). If fewer than 3 fit, the crisis rerolls *(Claude)*.
    - *Take the seat* is offered only when the leader is the enemy, as a **fourth** ending beside the 3 rolled. It is never in the rolled 3. The crisis carries 3–4. *(Before v6c the kit had 3–4 fixed endings and the seat ending replaced one.)*
- **Epilogue** (optional; required when the last move left a place or thing **broken, burned or seized**). *Seized* means the stake passes to the enemy or a third party, and is theirs when the crisis opens (a writ handing them the Lending Wall, say).
  - **It counts toward the 12-card cap.** When the last move requires one, generate at most 11 cards before it (shorten the routine phase first).
  - one Restore card, set in the world the last move made, at the crisis's place, headquarters or a phase-1 place;
  - it may use Restore's *deliberately broken/ruined* causes (an amendment to Restore);
  - like the main Return, it never assumes how the crisis ended.

### Story craft (v4, from the world tests)
These apply to every faction story, on top of the phase rules.

**Chain and climb** (phase 3)
- **Each card after a chapter's opening depends on a plant from the card before it** in that chapter. The chapter reads as *therefore* / *but*, not as side-by-side doors.
- **Pressure climbs.** Every phase-3 card records `pressure_level` 1–3, and it never falls within a chapter. Chapter 2's opening starts at least at chapter 1's turn level.
- *Main chapters now chain and climb too (main-quest, *Chain and climb*); this is the same rule.*

**Moves advance the scheme**
- **Each move is the enemy's next step in the revealed scheme, or their answer to being exposed.** It is never an unrelated new act.
- **Every date or deadline in the revelation** is met by a move or by the crisis.
- **Each move names a `lever`** (debt, law, rumour, theft, force, bribery, blackmail, kin, faith, fire): how the enemy acts. A story uses at least two different levers.
- **The enemy's want survives the last move.** The last move may not satisfy the enemy's own want: a last move that pays off the enemy's debt can't be followed by the enemy still needing the score. A revelation's named cost (someone taken, someone struck off) is paid by a later card or move. Same rule as main-quest, *The villain's want survives the last move*.
- **Event ceiling:** see §7.

**Meet them before they matter**
- **The enemy** fills a slot on at least one phase-3 card, so the crisis isn't the first face-to-face.
- **The Hand** fills a slot on at least one phase-3 card, and no more than **two** hidden roles in the whole story.
- **The heart** appears innocently on one routine card (as a requester, a witness, a neighbour), before the story is about them.
- **Anyone a move kills** appears on an earlier card first. They carry `dies_at` (the move's id): alive on every card before it, dead after (rulebook §4). For the paired truce, each story meets only **its own** dead (§10).

**Plant what the crisis asks for**
- **A heart's secret** (a kit heart's hidden fact, such as the paper-house heir) is planted before the crisis and offered there as a way or twist, or the heart is rolled without it.
- **At least two of the faction endings** each rest on a planted fact that makes them a live option (an ally who would back reform, a ledger that could buy the houses back).
- **Each kit revelation names who is harmed** if it comes out, so publishing it is a choice with a cost.

**Short steps inside a phase, journeys between them**
- **Inside a phase or chapter, each card is within about 1 day of the card before it** (or in the same region). Distances come from the map. **The routine phase is exempt** (v6b): routine cards are hall work, given at halls across the big towns (§8b.2), so a routine step may be any length. *(Before v6b, one long routine step was allowed.)*
- **Between phases and chapters, the story may move:** the step from routine to reveal, reveal to chapter 1, chapter 1 to chapter 2, and chapter 2 to the crisis may each be **one journey**. This is how a guild story crosses the map (§8b) without the player commuting back and forth inside a chapter. It moves at phase and chapter boundaries, never inside a chapter.
- **The minimum-spread rules stay** (routine and reveal across at least 3 big towns, §8b.2; one phase-3 card a journey away). Phase 3's journey-away card is a chapter's opening card, a journey from the card before it (the chapter boundary).
- **Offered where the player is:** a card is offered near where the previous card ended (at a contact, a member who comes to find you, a messenger), not only at the requester's home.

**Track people along the chain**
- A role on an earlier card that can **move a person or change their side** (cargo, captive, fugitive, defector, a turned kin) bars them from later slots that assume where they are or whose side they're on. Use them later only in roles that fit any outcome (as assembly's ordering clause, extended to faction chains).

**Names and clues**
- Cast full names are unique world-wide, and given names follow the rulebook's *Name check*. **Kin share a family name.** Names come from the people's naming patterns when the naming module is wired in.
- **Clue objects** (a black cord, a green seal) are unique across the world's stories.

### Which main-quest rules apply
- **Apply to phase 3 and the crisis:**
  - §3: chapter shape and chapter rules
  - §5: moves are public past events and finished acts
  - §7.5: allies are planted, and have a price
  - §7.6: the fault line (Schism crises only)
  - §7.8: the way-pool cap, with the code-break way in the sacrifice way's place
- **Do not apply:**
  - the epic weight rules §7.1–7.4 and §7.7
  - scale arcs
  - set pieces other than Schism
  - landmarks

## 7. Keeping the main story bigger
- **Scope cap:** **region**, never realm, **whatever the faction's reach**. A guild with halls in every territory may have realm reach (`faction-places.md` §6), but its story still stays at region. A guild-wide move ("every hall is shut") is region scope, because only one region's halls are named on cards.
- **Never foundational.**
- **Event ceiling.** The scope cap covers what a move, crisis or ending **does**, not only where its cards sit.
  - No realm-scale act: no Throne edict, realm law, declaration of war, or war writ against a whole faction.
  - **An *Above* enemy acts through a local instrument:** a magistrate's writ, a lord's seizure, one lordship's court, one march's muster. The instrument's holder is that seat's registry holder (see rulebook, *Seats and offices have one holder*).
  - **A realm-reach guild** may have a guild-wide act of its own (every hall shut, every contract recalled), counted at region scope as above. That is the only realm-reach exception.
  - **Never larger than the main story's climax.** If a move's event would outweigh what the main climax decides, scale it down to one town or lordship.
- **Set pieces:**
  - **Schism** only, and only as the crisis;
  - Siege, Succession and Embassy stay main-story only.
- **Length:** 9–12 cards, against the main story's 15–25.

## 8. Reuse, crossings, twists, fallout
- **Cast:** exempt from the reuse cap and once-per-family inside its own story.
  - **Each cast member** fills at most one slot per card in phases 1–2, and one per chapter in phase 3. The limit is per cast member, not one cast member per card. **The heart's fallback is held to it too** (at most one slot per chapter while standing in); the planner checks it (§12).
  - **Authority-only slots** (arbiter, judge, ring-keeper, answers-to) don't count toward it, and neither does being a card's giver.
- **Separate from the main story:**
  - a faction story's cast is never main-story cast, and vice versa;
  - neither story's stake is used by the other;
  - **a faction with a faction story is never the main story's antagonist side.** Otherwise every member would be a main proxy.
- **One optional crossing:** a faction story may share **one** person (never main-story cast or a main-story proxy, and not cast in this story except as below) or **one public plant** with the main story. Neither story may need it in order to finish.
  - **A crossing is** a person from a main-story card in a meaningful role on a faction card (requester, opposing, knower, captive, ally), or a shared public plant.
  - **Layer-2 High people** (the lords and officials the realm layer made, `quest-locations.md` §7a) are free in authority-only and background slots. **One** may take a meaningful role (client, opposing, knower) as the story's single crossing. **That crossing person may be the story's *Above* enemy** (a guild story can involve a lord the realm layer already set up), as long as they are not main-story cast or a main-story proxy.
  - **Not a crossing:** reusing realm authorities or clients (a magistrate, a lord as arbiter, a recurring merchant) in background or authority-only slots. These follow the ordinary reuse rules.
- **Enemy factions:** a faction is the enemy's side in at most one faction story per world, across all guild stories. This is meant: two stories can't both have the Free Lords as the enemy's side (Writers W2 and Lantern L1), even with different enemies.
  - **A *Within* enemy makes the story's own faction its enemy's side** (as the kits already read it: Night Market N2 bars Writers W3).
  - **So the Holy House** may be the enemy's side of Writers W5, Lantern L7 or Dueling D9 only if no other story uses it, its own included: each is barred when the Holy House story's enemy is *Within* (HH1, HH2, HH6, HH9), and the other way round. Every side and its enemies are in one table (`quest-guild-kits.md`, *Rules for every kit*) (v6c).
- **Moves:** like main-story moves, they are exempt from the one-card-per-family past-event limit.
- **Twists:**
  - each faction story is its own storyline: at most **two twists per group**, never in the same chapter (one per group starved most cards of a twist);
  - faction twists count in the group they declare;
  - the reveal, the phase-3 turn cards and the crisis are never plain, and their twists are rolled before the other cards', the crisis's first (rulebook, *No blank card* and *Roll order*).
- ***Faction fallout is required.*** A move that reaches beyond the guild (a writ, a seizure, a public killing, a raid, a burning) **spawns at least one fallout card**, set in a settlement it touched. A move that stays inside the guild may spawn one only if it is region scope. **At most one per move, so at most 3 per story** (raised from 2). Required fallout counts inside that cap, so the cap can always be met.
  - Every settlement a move reaches gets it in its context, fallout or not (§12, *Guild moves reach the towns*).
  - **Reading line:** in faction-story fallout, read "main card" and "main cast" throughout as this faction story's cards and cast.
  - **These fallout rules apply:** 1, 2, 4, 5, 6, 7, 8, 9, 11, 15, 16, 17 and 18.
  - **These are waived:** 3 (the count), 10, 12, 13 and 14.

## 8b. Where guild stories meet (Laz, 2026-10-26: medium overlap)
**Why:** whichever guild the player joins, its story should carry them across the island, through towns other guild stories also use, and past the other guilds. No single playthrough should leave a whole part of the map unvisited, and the guilds should feel like they share one island. (World B's five stories shared almost no towns; the Old Hands never left two regions.)
- **v6b (Laz, 27 Oct): the guilds share the big towns, Elder Scrolls style.** "I have pictured it like Elder Scrolls, how the Fighters Guild and the Mages Guild are always in the same town." In World C the Lantern Company and the Writers shared one town, with one card each, so a player with two builds missed most of the other guild's content. Hall work now happens in the big towns every guild shares; each guild's own content (the Writers' digs, the Lantern Company's frontier forts) stays out in the field, in its chapters.

**1. Each story crosses the map**
- A guild story's cards touch at least **3 regions**, at least **2** of them outside the region of its headquarters.
- It moves at phase and chapter boundaries, never inside a chapter. **The routine phase is exempt:** its cards are hall work, given at halls in the big towns (§8b.2), so *Short steps* doesn't bind them (Story craft, *Short steps inside a phase, journeys between them*). It still binds chapters.
- ***Every region pulls.*** Counting the main story's cards already pinned, **every region has at least one story card** (main or guild). The placement plan puts a routine card (where the region has a large or great town, since routine cards are given at halls there, §8b.2) or a phase-3 card (at a field site) in any region that has none. Remote regions (the Rim, the Snows) qualify like any other: having content isn't enough, a story has to send the player there.
- **Across all the world's guild stories,** no region holds more than about **a third** of all guild-story cards.
- Routine and reveal span at least 3 big towns (§8b.2); the crisis falls where the story is (§6, phase 4).

**2. Shared towns** (v6b, Laz: Elder Scrolls style)
- **Shared towns** are settlements holding the headquarters or a hall of **two or more** guilds. Every guild, and the Holy House, has a hall in every **large or great** town (`faction-places.md` §5b; the Holy House's is a chapel or parish shrine), so **every large or great town qualifies**; a town that is home to two headquarters (Openharbour) always does. The placement plan (§12) marks them before any guild story is pinned. A card set in one has `shared_town: yes`. (*Crossroads* keeps its plain meaning, a road junction.)
- ***Hall work is in the big towns.*** A guild story's **routine cards and reveal** are given at its halls in large or great towns, across at least **3** of those towns. Routine cards are hall work, so *Short steps* doesn't bind them (it still binds chapters). The guild's hall jobs are given there too (locations §4, *Hall jobs*).
- ***Every pair of guild stories shares at least 2 towns.*** That's every pair, not only rival and ally pairs. Two stories **share** a town when both have a card in it (at a site inside it, or beside it).
- ***Every large or great town holds cards from at least 2 guild stories.***
- **Chapters still go to the field** (phase 3, *Pinned at field sites*): that's where each guild's own content lives, the Writers' digs, the Lantern Company's frontier forts. The one phase-3 shared-town card and the crisis (when it falls in a big town) count toward the towns a story uses.
- *(This replaces the v5 rules "2 cards in shared towns per story" and "no shared town used by just one story".)*
- **Different sites, same town.** The stories use different sites inside the town (settlements are containers, rulebook). The town's own caps still hold and count every story's cards (locations, *Per-place caps*), so the stories spread across the shared towns instead of piling into one.
- **Beside a shared town.** A field site is *beside* a town when it is within half a day's travel and has the town as its `at`. It counts toward the town's caps, and it is still a field site for pinning (phase 3).
- **One state per site.** When one story changes a site another story uses (a ring's licence revoked, a yard occupied), the change is a state fact on the site, and the other story's cards there read it (§12, *Guild moves reach the towns*).

**3. Rivals and allies meet**
- **Each pair of guilds has a tie:** *at war*, *rival*, *ally*, *trade* or *none*. These are faction relations (`quest-factions.md`). The defaults are below; world generation may shift a non-canon tie one step (none ↔ trade ↔ ally, or trade ↔ rival).
  - **A shifted tie is a fact:** it is written as a public fact on both faction entries (`source`: world), rolled with `Rng::from_keys([seed, pair, "tie"])`, so ties are reproducible and the mechanical layer can read them.

| Pair | Tie | Why |
|---|---|---|
| Old Hands – Night Market | **at war** (canon; counts as *rival* for meetings) | the native and the open syndicate; contested sites |
| Lantern Company – Dueling Society | **rival** | the Company's old fighters sneer at the Society; both sell skill with a blade |
| Lantern Company – Night Market | **rival** | Lantern bounty writs hang on Market crews' heads |
| Writers – Lantern Company | **ally** | a Lantern contract camp guards the Writers' digs (`faction-places.md` §5b) |
| Writers – Old Hands | **trade** | the Writers keep the records the Old Hands forge; uneasy |
| Night Market – Dueling Society | **trade** | wagers on bouts are settled through the Market |
| Holy House – Dueling Society | **rival**, and **exclusive** (Laz) | the house bans rings (canon); a ring may be *banned by the Holy House* (`faction-places.md` §5b) |
| Holy House – Writers | **rival** | inquisitors seize "heretical" texts (Writers kit W5) |
| Writers – Night Market, Writers – Dueling Society, Old Hands – Lantern Company, Old Hands – Dueling Society; Holy House – Lantern Company, Holy House – Old Hands, Holy House – Night Market *(Claude)* | none | |

*The non-canon ties are Claude's proposal (§14), for Laz to confirm.*

- **Exclusive pairs:** the Old Hands and the Night Market (§10), and **the Holy House and the Dueling Society** (Laz, 27 Oct). Joining one closes the other's story and its hall jobs (mechanics §9). Both stories are still generated. The Holy House pair is **not hostile**: unlike the syndicates, joining one doesn't set the other's members against the player. A meeting card between them still works for a player who has joined either.

- **Every rival or ally pair gets one meeting:** a card in one of the two stories where a **member of the other guild fills a meaningful role**, set in a shared town or at a field site both guilds use. The meeting records its `site`, so the checker can confirm this.
  - **Rival:** the opposing slot, the other principal of a Contest, a competitor for the same thing, or the culprit.
  - **Ally:** a candidate for the chapter-1 ally plant, a knower, a guide, or the requester.
  - **Trade** pairs meet (as a client, seller or go-between) when a shared town allows it: one story has a card in a shared town where the other guild has a hall, and a slot one of those roles fits. It stays optional: a plan with no such card isn't broken.
- **Spread the meetings:**
  - each story **hosts** at most **2** meeting cards;
  - every guild **takes part** in at least **1** meeting, as host or guest. A guild whose ties give it none gets one with a *trade* partner.
- **Who the visitor is:**
  - an ordinary registry person who is a member of the other guild, under the ordinary reuse rules;
  - **never** cast in either story, never that guild's leader, and never the other story's enemy, Hand or heart (paired syndicates excepted, §10).
- **Neither story needs the other.** The meeting card works whether or not the player ever joins the visitor's guild. If the player belongs to both, the card stays and becomes a choice of loyalty; what that does to standing is mechanical-layer work.
- **Not a main-story crossing.** §8's one-crossing limit is about the main story only.
- **The syndicates, paired:** each enemy's appearance in the rival's story (§10) is that pair's meeting, one in each story. This is the exception to *Who the visitor is*: the Old Hands' enemy is a Night Market member appearing in the Night Market's own story (and the other way round), and its roles there follow §10, not the rival roles above.
- **The syndicates, unpaired:** they still get one meeting, at a contested site, under the ordinary rules.
- **A meeting card is an ordinary card of its phase.** The meeting only fills a slot. It obeys the slot limits, the twist budget and the family rules.

**4. One placement plan for all guild stories** (see §12)
- All guild stories are placed together, after every kit is rolled and before anyone is cast or any card is made.
- **The plan** sets, for each story:
  - a region and settlement for each chapter and for the crisis, and a **large or great town** (with the hall there) for **each routine card and the reveal**, at least 3 towns across them;
  - which cards are shared-town cards, so every pair of stories shares 2+ towns and every large or great town holds cards from 2+ stories (§8b.2);
  - which cards host which meetings, and at which site;
  - each story's chapter-2 **target + event** (§9).
- **The arithmetic (six stories, checked v6b):** a story with 3 hall towns plus its crisis's town or its phase-3 shared-town card uses 4–5 of the island's big towns. Six such sets can share 2+ towns pairwise and cover every big town twice (World C's map has 8: e.g. Highcrossing and Openharbour in every story, the other six spread). The easy plan, every story in the same two great towns, is legal, but it piles cards there: the plan spreads the third and later towns first, and keeps the per-place caps and the ⅓ region cap in view (the Crown Crossing holds 3 of the 8 big towns).
- Each story's cards are then generated against the plan. The checks in §12 confirm the plan held.

## 9. Spread across one world
Across all the guild stories generated for one world (**six** since v6b: the five guilds and the Holy House; every number below was rechecked for six):
- **Leaders:** at most **one** story has its leader as the enemy. Co-leaders both count as "the leader" (the Dueling showman and the sister; rulebook, *Seats and offices have one holder*), and so do the Holy House's two leaders (§5). The Holy House's high priest as enemy (HH1) counts here.
- **Enemy shapes:** at most **two** stories have a *Within* enemy, and at least one is *Above* and one is *Beside*. (These are the enemy shapes of §5, not the rulebook's truth *Shapes*.) With six stories, at least four enemies are *Above* or *Beside*; every kit has both, so this holds.
- **Revelation kinds:** at most **two** stories use *the hidden hand* (§11). In paired mode the two syndicates' reveals count as **one**. *The cost* is a reveal at most **twice** in the world, each worded differently (rulebook, *World ledger*). With six stories, at least two (paired: at least one) reveal *the old debt*, which has no cap; every kit has one.
- **Hearts:** no heart type is used by more than **two** stories. Five types × 2 = 10 places for 6 stories. (Five types across five guilds forced the last guild's heart; found by the kit roller.)
- **Ordeal targets and events** (v6b; replaces "each event at most twice"):
  - **The ledger key is target + event** (§6, *Phase 3*), and each target + event is used **once** per world. The main story's Ordeal counts, as (heart, its event) (rulebook, *World ledger*).
  - **Each target** (heart, player, guild) is used by **at least 1 and at most 3** guild stories.
  - **The slack:** 10 combinations (3 heart, 3 player, 4 guild) for 7 uses (the main Ordeal and six guild stories). The main Ordeal takes one heart event, so at most **2** guild stories hit the heart; the player's 3 events allow up to 3; the guild's 4 events are held to 3 by the target cap.
  - **The placement plan deals them** (§12 step 2): the main Ordeal's (heart, event) is counted first, then each story is dealt a target + event its kit allows (each kit's *Chapter-2 blows* line) and its enemy's means meet (§11a, *What each blow needs*), whose door premise no other story's door uses, and step 5 writes what it was dealt.
  - *(The old one-*captive* limit and the "each event at most twice" cap are gone. The second had no slack: three events, six stories.)*
- **Crises:** at most **three** stories end on Reckoning, and each crisis family + premise at most **twice** (guild and faction crises only; the main climax doesn't count as a crisis, rulebook *World ledger*). With six stories, at least three crises are outside Reckoning; every kit has at least two (§11), so twelve or more are on offer.
- **Writ fatigue** (v6c): across a world's guild stories, at most **2** chapter-1 moves, chapter-2 blows or crises whose **main means** (`uses`, §11a) is `paper` **aimed at the guild**: a writ, a licence revoked, an outlawing, a court ruling or seizure order against the guild, its halls, its rights or its trade. *Outlawed* always counts. Paper used against one person (a debt bond on the heart, a warrant for the player) doesn't *(Claude)*. A guild-target blow carried by `paper` always counts, and so does the house's `faith` used as a writ or ban against the guild. Each kit marks the moves and crises that can be writs (`writ` *yes*). **One story spends at most one** *(Claude, from the drafts)*. This is on top of the lever caps below: a writ is also a `law` lever.
- **The heart's captivity** stays spread by dealing the targets (above); nothing more is needed.
- **Crisis givers:** at most **two** stories' crises are given by their own leader.
- **One world ledger.** These counts are kept in the world's one ledger (see rulebook, *World ledger*), shared with the main, location and fallout passes. **The kit roller reads it before rolling**, so every earlier pass's premises, twists and shapes count against a guild story.
- **The main story counts toward the world ledger's own caps** (rulebook, *World ledger*), but **not** toward the guild-only caps below, which count guild stories only.
- **Guild-story caps in that ledger** (counted across all the world's guild stories):
  - at most **two** reveals of the same family;
  - each family + premise at most **twice**, and never twice in the same phase position. This covers **every kit pool and phase**: odd jobs, the reveal, chapter openings and turns, and the crisis;
  - each twist at most **twice**;
  - at most **two** last moves that kill someone;
  - each move `lever` at most **three** times across all stories' moves. The main story's **reserved levers** (its last move's lever, and every lever it uses on 2 or more moves) at most **once** each in all other stories together, and a premise or ordeal event built on the same mechanism counts as a use (rulebook, *World ledger*);
  - at most **two** stories whose stake is a reputation fact.
  - *Six-story check (v6b):* six reveals need at least 3 families (fine); 18 moves against 10 levers at 3 each, less the main story's reserved levers held to one use each, still leave 20 or more uses; the ⅓ region cap is about **22** of about 66 guild-story cards (§8b.1; hall jobs aren't guild-story cards and don't count toward it *(Claude)*).
- **A shared move counts once.** The paired truce killing is one move for levers, kills and fallout (§10).
- **When it's checked:**
  - the "at most" limits are checked as each story is generated;
  - **lever caps are checked when moves are written** (§12 step 5), not at kit rolls: kits carry no levers;
  - **writ fatigue** is checked at the kit rolls (chapter-1 moves and crises) and again when the plan deals the blows (§12 step 2);
  - the "at least one Above and one Beside" rule is checked **once every guild story exists**.
- **On a violation,** reroll **only the pool that broke the limit** (enemy, heart, chapter-1 move, or crisis) in the latest story, then recheck compatibility and the throughline (§11a). If a variant broke it, reroll the variant first.

## 10. The rival syndicates
The Old Hands and the Night Market are **generated as a pair**. Roll once for the pair:
- **Paired (about 1 in 3 worlds;** v6c, down from about half: three of four sample worlds paired, so the Old Hands and the Night Market told the truce three times**):**
  - **both stories take a paired crisis from their kit** (the crises each kit marks *paired*; before v6c, Old Hands C2 or C5 and Night Market C1 or C2). The paired truce is one of the two crisis kinds still tied to one kind of enemy (§6, phase 4). They share one last move: a truce meeting at a **contested site** (`faction-places.md` §5b) ends with one member of each syndicate dead, and both killers walk home.
  - **Both enemies are Beside,** drawn from the other syndicate (each kit has two paired-only enemies), and each enemy is the killer of its story's dead.
  - **The feud varies** (v6c): the planner rolls it once, with the truce facts, from the two kits' *paired feud* (word for word the same in both): the enemy pair, who struck first, what it is over (it must fit the first striker's scheme), who called the truce, and how the truce ended (both killers' means must meet it).
  - **The feud grows from both schemes** (v6d): what it is over must fit the other enemy's scheme too (the kits' *Answered by*), and each enemy's chapter-1 move is read as his scheme at work (the kits' *Readings by enemy and variant*). The feud is how the schemes collide; it never replaces them.
  - ***One truth for a shared move.*** The truce killing has **one** set of facts: who killed whom, where, when, and who called the truce. **The planner fixes them** before either story's cards are written, and both stories' cards cite the same fact ids. Neither story may tell it differently.
  - **Two views of one feud:**
    - each story's chapter-1 move is a public fact that the **other** story's chapter 2 grows from, so the feud escalates both ways;
    - each enemy appears once in the **rival's** story, on a phase-3 card (not as cast there). That appearance is the pair's meeting (§8b.3);
    - each story's chapter-2 turn card plants the **truce meeting** (who called it, where, when), citing the planner's facts, so the shared last move is set up in both.
  - **The crises differ:** roll which story takes Reckoning *Blood debt* (shield *Protected*: its leader shields the killer; code-break way: go to the magistrate). **That crisis holds the truth row** about the killing. The other takes its kit's **non-Reckoning** paired crisis (Old Hands C5, Night Market C2; both decide only headquarters, so a story with another stake holds *Blood debt*), aimed at the truce killing (getting its own people out before the reprisal, say), with its own code-break way. **Its truth is about its own aim** (its people, the reprisal), never a second answer to who killed whom.
  - **Each story meets its own dead.** The dead on each side is met on an earlier card of **that** story (`dies_at`), not the rival's, and is linked (ally or kin) to that story's heart.
  - **Counted once.** The shared move is one move for levers, for "last moves that kill" and for fallout (at most one fallout card between the two stories, §8). The two syndicates' reveals count as one *hidden hand* (§9).
- **Unpaired:** each story rolls a non-paired crisis, and the two share no move.
- **The pairing is rolled first,** before the kit-roll order is counted (§12 step 1), with `Rng::from_keys([seed, "syndicates", "paired"])` *(Claude)*.
- **Either way,** they share no cast, and joining one makes the other's story unavailable. That part is mechanical-layer work; both are generated.

## 11. Story kits
Each joinable faction has a kit. The story rolls one entry from each pool. **An entry can be rolled only when the enemy's means meet its `needs`**, and the story must pass the throughline test (§11a). Each entry's compatibility notes and §9 still apply.

**v6c (Laz, 27 Oct): kits are about twice as wide.** Four sample worlds showed stories varying between guilds but not inside one: Dueling told one plot three times in four, the Writers repeated one crisis three times, and chapter 1's move, the endings and the odd job's hint never varied (`test/world/D/narratives/guild-narratives.md`).

| Pool | Size (v6c) | Each entry gives |
|---|---|---|
| **Enemies** | about **8** (at least 7) | enemy shape, who (a slot spec, or a leader archetype), `want`, `means` (2–3 tags), `reach`, **`reaches`** (the stakes its scheme can touch), a one-line **scheme** (what they are doing and why, in the kit's words), the revelation in outline, its **revelation kind**, and whether it may publish the faction's hidden side |
| **Stakes** | at least 4 | entry type and what it is, its `hit`, and `needs` (the means that can threaten it) |
| **Hearts** | 4–5 types | heart type and a specific person spec |
| **Chapter-1 moves** (new) | about **6** (at least 5) | a public act, its `needs`, `hits`, `aims`, `shapes`, `cost` and `writ`, and a reading per stake (§6, phase 3) |
| **Odd jobs** | about **5** (at least 4) | family, premise, the hidden role, any forced way or shield, and its **thread kind** (below). No two share a thread kind. |
| **Crises** | about **7** (at least 6), at least 2 outside Reckoning | the last move, the crisis family and premise, the code-break way, its `needs`, `hits`, `decides`, `shapes`, `writ` and `kills`, and a reading per stake. **No enemy list** (§6, phase 4) |
| **Faction endings** | a pool of **5–6**, plus the seat ending | the steering choice (a label only) and its `fits` (the stake hits, crisis families or crises it can close). The crisis rolls 3 that fit (§6, phase 4) |

**Spreading the enemies:**
- over the three enemy shapes (*Within*, *Above*, *Beside*), and over wants and means;
- **no two enemies share a want *and* a means set** (the same want with the same tags);
- at least one of each revelation kind (below).

**Odd-job thread kinds** (what kind of loose thread the odd job leaves; before v6c every odd job hinted alike, "someone secretly works for the enemy"). Slugs *(Claude)*:
- `answers-to`: a person who answers to someone else;
- `shouldnt-exist`: a thing that shouldn't exist (a forged seal, a second copy);
- `wrong-place`: a place that's wrong (an empty house, a fresh grave);
- `pattern`: a pattern (the third time this month);
- `lie-in-record`: a lie in a record.
- The odd job's hidden role is still the Hand or a proxy (§6, phase 1). The thread kind is what the player *notices*.
- An odd job may also carry `needs` when its thread only works with a means (a forged seal needs `paper` or `craft`) *(Claude)*.

**Nothing new from outside the docs:**
- New entries follow the canon in `world-factions.md` and `leaders.md`, and the existing kits' tone. Anything invented is marked *(Claude)*.
- Every twist and premise an entry names already exists in the rulebook's *Twist groups* and the family docs: **no new families or premises**. Signature premises stay as they are.

**Revelation kinds:**
- **the hidden hand:** someone is working against the faction;
- **the cost:** the faction's own routine work, added up, is doing harm nobody chose;
- **the old debt:** something the faction did long ago is coming due.

**Each kit has at least one of each kind.**

**Leaders in kits are archetypes.** A kit names a leader archetype ("the Lantern leader archetype", "the quartermaster"); a canon name is given only as its default filler.

**Every signature premise carries a truth row,** as its family requires (rulebook, *No blank card*), and names the way that reaches each value (rulebook, *The truth is reachable*).

**Variants:** every kit entry (enemy, stake, heart, odd job, crisis) carries **2–3 internal variants** (who / why / how), so a rolled id doesn't reproduce the same story. Chapter-1 moves may carry them *(Claude)*.
- The roller rolls the entry, then one of its variants (recorded as e.g. `L5b`).
- A variant changes the person spec, the reason or the method. It never changes the entry's enemy shape, want, revelation kind, entry type or compatibility, **nor its throughline tags** (`means`, `reach`, `reaches`, `hit`, `hits`, `needs`, `aims`, `decides`, `shapes`, `cost`, `writ`, `fits`, `thread`).
- **Crisis variants name the crisis giver** (phase 4).

- **Variety:** each kit gives 25–59 legal combinations after the needs and the throughline test, multiplied by the variants. Phases 1 and 3 are then filled from the profile and the families. *(v7: guild stories are moving to authored spines, starting with the Writers; see `test/world/fixlist-v7-writers.md`.)*
- **Share band** (v7, A10; report only). Over 400 seeds (`--stats`), every enemy and crisis rolls between ⅓ and 2.5 times its even share. A part outside the band gets wider tags first, then a `weight`. `--stats` covers every pool and lists parts that never roll.
- **Across worlds:** the ledger is per world. Since v6c the roller also reads the last two games' entries and avoids them unless nothing else fits (mechanics, *Fresh stories*). (Dueling D5 rolled in worlds A and B, and in three of four sample worlds.)
- **Kits grow:** add entries freely, as long as each one meets its pool's spec and carries its throughline tags.
- **Kits as data:** the kit's words live in `quest-guild-kits.md`; the generator reads them, tags included, as rules data in `data/quests/rules/` (Lane A). A quest set records only what was rolled (§13).

## 11a. The throughline: how kit parts fit (v6c)
Wider kits alone would trade repetition for nonsense. Laz: "Assembling a random kit of parts will not always make a story that makes sense unless we're very careful to make sure that it does." So **two goals weigh the same**: more distinct stories per guild, and every assembled story holding together as cause and effect.

**The idea:** every enemy says what they **want** and what they can **use**. Every other part says what it **needs** from the enemy, and what it **hits**. A part can be rolled only when its needs are met and the stake is one it can reach, aim at or decide (*Means aren't enough*, below). Then the whole story must pass the **throughline test** before any card is made.

### Means tags
What an enemy can use. An enemy has **2–3**.
- `paper`: writs, contracts, grants, records, the law's own instruments;
- `coin`: buying people and debts;
- `men`: armed force of their own or a lord's;
- `inside`: people inside the guild who answer to them;
- `name`: rumour, standing, the power to disgrace;
- `faith`: the house's authority, penance, heresy charges;
- `kin`: family ties, marriages, heirs;
- `craft`: a skill the guild also sells, such as forgery, fighting, copying or escort.

*Three things share words and are kept apart:* the want *faith* (what the enemy is after, §5), the means `faith` (what they can use), and the lever `faith` (how one move works, Story craft). The same goes for the means `kin` and the lever `kin`.

### Hit tags
What a part damages.
- `place`: a hall, a yard, a crossing;
- `thing`: a record, a relic, a contract;
- `name`: the guild's or someone's standing;
- `people`: members, the heart, the player;
- `trade`: the guild's living, its routes and clients.

### Reach
How far the enemy's power runs *(Claude's glosses)*:
- `local`: one town, one hall, one household;
- `region`: a lordship, a march, a region's courts or markets;
- `realm`: more than one region. **It still acts through local instruments** (§7, *Event ceiling*): realm reach lets an enemy get a lord's writ anywhere, never a realm-scale act.

### Means aren't enough (v6c, from the kit drafts)
Three pairs of writers widened the six kits apart, and each found the same thing: **means say what an enemy *can* do, not what their scheme *would* do.** Matched on means alone, a forger could threaten the survey rolls, the showman could pay for a riot to close his own tournament, a sincere priest could lay a heresy charge, and a notes crisis could "decide" the toll trails. So every kit carries six more things, under the same names in every kit:
- **`reaches`** on each enemy: the stakes its scheme can touch. A stake rolls only when the enemy reaches it **and** has one of its `needs`.
- **`decides`** on each crisis: the stakes it can bring to a head. A crisis rolls only on a stake it decides.
- **`aims`** on each chapter-1 move: the stakes it can be aimed at (*any* only for a move that hits `people`).
- **`shapes`** on moves and crises: the enemy shapes they roll with. Means can't tell an enemy attacking the guild from outside from one working inside it.
- **Per-enemy blows,** where means alone mislead: such an enemy takes only the blows its list names (the master-at-arms never bans the rings he recruits from).
- **Readings:** each move and crisis says, per stake it aims at or decides, what the act is for that stake. Writing them out is what caught the nonsense; a part with no sensible reading for a stake doesn't list it.

Two smaller rules came with them:
- **The cost moves protect.** A move marked `cost` *yes* is one an enemy can make protecting what pays them; under *the cost* (point 5) only those roll. A crisis can carry *not under the cost*.
- **One writ per story** *(Claude)*: a story spends at most one of the world's two writs (§9), so one story can't take both.

### What each part carries

| Part | Carries |
|---|---|
| **Enemy** | `want` (greed, revenge, preservation, desperation, cover-up, ambition, faith), `means` (2–3 tags), `reach`, **`reaches`** (stakes), and a one-line **scheme**: what they are doing and why, in the kit's words |
| **Stake** | its `hit` tag, and `needs` (the means that can threaten it) |
| **Chapter-1 move** | `needs`, `hits`, **`aims`**, `shapes`, `cost`, `writ`, and a reading per stake |
| **Chapter-2 blow** | `needs` (the table below, which each kit restates in full and may tighten); some enemies a **per-enemy** list |
| **Crisis** | `needs`, `hits`, **`decides`**, `shapes`, `writ`, `kills`, and a reading per stake. No enemy list |
| **Ending** | `fits`: the hits, crisis families or crises it can close (*any* closes all) |
| **Odd job** | its thread kind (§11), and sometimes `needs` |

*`reaches` (stakes) is not `reach` (how far the enemy's power runs).* Moves and crises may carry `limits`: wants, named enemies, *not under the cost*, or the paired syndicates' modes (`quest-guild-kits.md`, *Reading the kits*).

**Reading `needs`:**
- A part's needs are a list of choices: **any one** of them, among the enemy's means, is enough.
- "A **+** B" needs both. A choice can be limited to a shape (`men` (Above)), to region reach (realm meets region), or away from named enemies.

**`uses`: the main means** *(Claude)*. When a move, blow or crisis is rolled, the roll records **which** of the enemy's means carries it out: a tag that is both in its `needs` and in the enemy's `means`. When there are two or more, a keyed roll picks one (`Rng::from_keys([seed, story, part, "uses"])`). **On a part marked `writ`, a choice that takes `paper` is used only when no other fits** *(Claude, from the Writers and Lantern drafts)*, so writ fatigue is spent only when nothing else will do. Writ fatigue (§9) and the move's lever (below) read it.

### What each blow needs
The chapter-2 blow's target + event (§6, phase 3). Needs are from the fix list (Laz's brief); the `hits` column, *stripped of rank*'s needs and the three limits marked v6c are *(Claude)*, from the kit drafts. Each kit's *Chapter-2 blows* table restates these in full and may tighten them (`faith` as the house's authority; *stripped of rank* by `inside` only).

| Target | Event | Needs | Hits |
|---|---|---|---|
| the heart | *captive* | `men`, `coin` or `paper` | `people` |
| the heart | *disgraced* | `name` or `inside` | `people`, `name` |
| the heart | *estranged* | `kin` or `name` | `people` |
| the player | *framed* | `inside` or `name` | `people`, `name` |
| the player | *stripped of rank* | `inside` or `name` | `people` |
| the player | *arrested* | `paper`, or `men` for an *Above* enemy (v6c: anyone else's men holding the player is a kidnapping) | `people` |
| the guild | *hall seized* | `men` or `paper`; or `inside` for a *Within* enemy, who shuts it by their own order (v6c) | `place` |
| the guild | *members taken* | `men`, `paper` or `coin` (a debt) | `people` |
| the guild | *outlawed* | `paper` **and** region reach; never a *Within* enemy, who doesn't outlaw their own guild (v6c) | `trade`, `place` |
| the guild | *leader struck down* | `men` or `inside` | `people` |

### Levers come from means *(Claude)*
Each move still names a `lever` (Story craft, *Moves advance the scheme*). Since v6c the lever must be one its `uses` means can pull:

| Means | Levers |
|---|---|
| `paper` | law |
| `coin` | debt, bribery |
| `men` | force, fire, theft |
| `inside` | theft, blackmail, fire |
| `name` | rumour, blackmail |
| `faith` | faith |
| `kin` | kin |
| `craft` | what the craft works through: forgery, law or theft; fighting or escort, force; copying, theft or rumour |

The lever caps don't change (§9). A story still uses at least two levers (Story craft), so its steps can't all lean on one means with one lever: where they would, the `uses` roll picks another of the enemy's means for one step *(Claude)*.

### The throughline test
Checked by the planner. The roller only ever rolls parts that pass it: it lists every legal enemy, crisis, stake and move under the tags and rolls among them, so nothing is rolled and then rejected *(Claude)*.
1. **The want explains the scheme.** The kit's scheme line is written for its want, and no variant changes either; the reviewer checks the line reads that way.
2. **Each step uses the enemy.** The chapter-1 move, the blow and the crisis each use **at least one of the enemy's means** (each has a `uses`), within their `shapes`, `limits` and the per-enemy blows. **The blow's `uses` is read from how it fell** (v6d; assembly §4, *Doors by how the blow fell*): a wounding by men needs `men`, a rigged one `inside`. No step's words give the enemy means he lacks (his men, his coin).
   - **The odd job serves the scheme** (v6d): the Hand's job is one the odd job's `fits` lists for this enemy.
3. **One story, one thing.** The stake is one the enemy `reaches`. The move `aims` at it, and the move and the blow each **hit the stake or people tied to it**: their hits include the stake's `hit`, or `people` (the heart, the guild's members, the player).
4. **They climb:**
   - the move is **public** (every move-table entry is a public act);
   - the blow is **personal**: it lands on someone the player knows by name (the heart, the player, the leader, members met on cards) or on the hall the player works from *(Claude's reading)*;
   - the crisis **decides the stake**: the stake is in its `decides`, and its hits include the stake's `hit`.
5. **The reveal's kind matches the enemy entry** (*the hidden hand*, *the cost*, *the old debt*), variant included, and the steps don't contradict it. Under *the cost*, nobody planned the harm, so only moves marked `cost` *yes* roll: the enemy protecting what pays them, not the opening of a plot. **The blow protects too** (v6d): every *cost* enemy has a per-enemy blow list, and never takes a blow that shuts what pays him. The last move fits the want (a crisis's `limits` bar the enemies whose want it would undo).
6. **One sentence can be written:** "Because [enemy] wants [want], they [scheme]; they [move], then [blow], and it comes to a head when [crisis]."
   - It is filled from the entries' own words (the scheme line and the readings), and recorded as the story's `throughline` (§13). **Where the rolled variant has its own reading** (v6d; the kits' *Readings by enemy and variant*: boundary readings, not debts; a purse, not a guard post), the sentence is written from it.
   - **If the sentence needs an "and also"** (a step that has to be explained by a reason of its own, not by the scheme), the roll fails. That is what the readings are for: a stake a part can't be read against isn't on its list.
   - The sentence names the secret scheme, so **no card text reads it**. It is for checkers and reviewers *(Claude)*.

**When it's checked** *(Claude: the blow is dealt later than the kit roll, so the test runs in two passes)*:
- **At the kit roll** (§12 step 1): points 1, 2, 3 and 5, for the stake, the chapter-1 move and the crisis, and point 4's crisis. A pair is legal only when at least one blow could still be dealt to it.
- **After the plan deals the blows** (§12 step 2b): the whole test, the blow included, and the sentence is written.
- **On a fail:** re-deal that story's blow first; then reroll the story with the fewest legal blows, enemy and every part after it (§12 step 2) *(Claude)*.

## 12. Generation order
**Slots are specs.** Each step below writes specs for the people and places it needs; the casting pass fills them (`quest-casting.md`). Guild stories are layer 3 (`quest-locations.md` §7a), cast before the next layer is generated. Until casting exists, each card resolves its own slots in the same order (search, reuse, create).

**The order is: kits for all → placement plan → throughline test → cast for all → cards per story.**

1. **Kit rolls,** for **every** guild story before any is placed.
   - **Read first:** the world ledger, and the recent list from the last two games (mechanics, *Fresh stories*). Their entries are unavailable unless nothing else fits.
   - **Roll the syndicate pairing** (§10).
   - **Fewest legal options first** (v6c): before each guild's roll, count each unrolled guild's **legal enemy + crisis pairs** (under the caps, the compatibility notes, the world spread, writ fatigue, the recent list and throughline points 2 and 3), and roll the guild with the fewest next. Ties go by a keyed roll *(Claude)*. No guild gets the leftovers. (Before v6c the order was fixed, and Dueling, rolled last, was left one enemy in two of four sample worlds.)
   - **For that guild:** a legal enemy + crisis pair (enemy shape, want, means, reach), rolled enemy first and then a crisis among that enemy's legal ones *(Claude: a uniform pair favours the enemies with most crises)*. **The enemy is rolled by revelation kind first** (v6d), then among the legal enemies of that kind, so an enemy legal in nearly every world (the Writers' W7 and W8, whose kinds have no world cap) doesn't take the leftovers of the capped kinds; then the stake, the heart, the chapter-1 move, the odd job, each with its variant; then 3 faction endings that fit (plus the seat ending when it applies).
   - **Check** compatibility, the world spread (§9) and the kit-roll half of the throughline test (§11a). Each part records its `uses`. If the last story leaves the world with no *Above* or no *Beside* enemy, that story rerolls its enemy (and every part after it) among the missing shape, rather than the world restarting *(Claude)*.
2. **Placement plan,** for all guild stories at once (§8b): a region and settlement for each chapter and for the crisis, a big town and its hall for each routine card and the reveal (at least 3 towns across them), shared-town cards (every pair sharing 2+ towns, every big town holding 2+ stories), meetings and their sites, a card in any region that has no story card yet (*Every region pulls*, counting the main story's pinned cards), and each story's chapter-2 **target + event**, dealt so each is used once with the main Ordeal's (heart, event) counted first, each target by 1–3 stories, and no two doors on one family + premise (§9).
   - **v6c:** a story is dealt only a target + event its enemy's means meet (its kit's *Chapter-2 blows* table, and the enemy's per-enemy list where it has one; §11a, *What each blow needs*), whose hits pass throughline point 3, and that keeps writ fatigue (§9).
   - **v6d:** it is dealt a row of assembly §4's *Doors by how the blow fell*, its `uses` among that row's means; the heart must fit (a `member` heart for *disgraced*; a *captive* heart only when she doesn't give the crisis and the crisis isn't *herself*); a hall is not seized after a move that shut one; the door's family + premise isn't the crisis's. **`--joins`** (`roll_kits_d.py`) rolls 400 worlds and counts every break of these joins.
   - If no deal fits every story, the story with the fewest legal blows rerolls its enemy (and every part after it), and the plan deals again *(Claude)*.
2b. **The throughline test,** for every story, before anyone is cast or any card is made (§11a): the whole test, the blow included. Write each story's `throughline` sentence. On a fail, re-deal or reroll as §11a says, then recheck the plan.
3. **Cast,** for every guild story: the enemy and heart from their kit specs, the Hand, the heart's fallback, the leader's role, meeting visitors, and the fault line if the crisis is Schism. Casting scores each slot against **its card's planned region** (`quest-casting.md`). Then, in paired mode, the planner fixes the truce killing's facts for both stories (§10).

Steps 4–12 then run story by story, against the plan.
4. **Revelation,** stored on the Hand or a proxy.
5. **Moves:** write chapter 1's move (the move the kit rolled), chapter 2's ordeal event (the target + event the plan dealt) and the last move (the crisis's), each as the next step of the scheme and with a lever its `uses` means can pull (§11a, *Levers come from means*). Check the lever caps here (§9). A *leader struck down* that kills names the seat successor here (§6).
6. **Phase 1:** routine cards, quiet plants, the odd job.
7. **Phase 2:** the reveal card.
8. **Phase 3:** the chapters.
9. **Phase 4:** the crisis, with the code-break way and faction endings; then the epilogue if needed.
10. ***Guild moves reach the towns*** (the rule is locations, *Settlement context*). For each move and each phase-3 public event, the planner adds its public fact to the `context` of every settlement it reaches, with `when` (the move id, or `<story>:<phase or chapter>`). A move or event that changes the state of a **site** another story uses (a ring's licence revoked, a yard occupied) is also written as a state fact on that site.
11. **Checks:**
   - plant rule (quiet plants paid by the reveal; the crisis payoffs)
   - no leaks
   - scope cap and event ceiling
   - short steps
   - twist groups
   - the code filter
   - pinning
   - crossing limit
   - world spread, writ fatigue included
   - the throughline held as written: each move and the crisis act through their `uses` means, and the cards tell the `throughline` sentence's story, with no step left needing its own reason (§11a)
   - the placement plan held: 3+ regions per story, routine and reveal at halls in 3+ big towns, every pair of stories sharing 2+ towns, every big town holding 2+ stories, meetings (§8b), and across all stories, *Every region pulls* and the one-third cap
   - the crisis is where the story is (v7c): at a place its chapters, door or centre use, with its giver there, or at headquarters only when the story is about it or its leader; its deadlines are no shorter than the map's road plus a day
   - chapter-2 blows: each target + event once (the main Ordeal counted), each target in 1–3 stories, the door's family allowed by the profile, a seized hall read by a crisis set there, taken members never the heart, a killed leader with `dies_at`, an earlier card, another crisis giver and a seat successor (§6, §9)
   - every planned meeting's role (opposing, Contest principal, ally candidate, knower, guide) exists on the card's rolled family; otherwise reroll that card's premise within its phase
   - each cast member, the heart's fallback included, fills at most one slot per chapter (§8)
   - every other story's card at a site with a state fact reads it (a `before` face in `reads_context`) or is moved to another site (locations, *Settlement context*)
   - paired mode: both stories cite the planner's truce facts, and only the *Blood debt* crisis answers who killed whom (§10)
12. **Fallout:** one card for every move that reached beyond the guild (required), and any optional ones from region-scope moves (§8). A shared move spawns at most one, between both stories.

## 13. Schema additions (for `quest-card-schema.md`)
- **Relatedness value:** `faction`.
- **Story level: `faction_stories`,** a list, one per joinable faction, each with:
  - `faction`
  - `kit_rolls` (entry id plus variant, e.g. `L5b`; since v6c the chapter-1 move and the 3 rolled faction endings are among them)
  - `cast` {`enemy`, `shape`, `want`, `stake`, `heart`, `heart_type`, `fallback`, `hand`, `leader`, `fault_line`}
  - `throughline` (v6c): the one sentence of the throughline test (§11a). Never read by card text.
  - `means_used` (v6c) {`move`, `blow`, `crisis`}: each one's `uses` tag, read by writ fatigue and the checkers *(Claude)*
  - `revelation` {`fact`, `kind`, `stored_on`}
  - `moves`, each with its `lever`; the chapter-2 move also has `target` (heart / player / guild) and its `event` (§6), and its turn card a `door` (`<target>: <event> → <family>`), as card-schema has them. *(Not to be confused with main moves' `faces.target`.)*
  - `phases` {`routine`, `reveal`, `chapters`, `crisis`, `epilogue`}
  - `faction_endings`
  - `code_endings` (each {`ending`, `when`}, where `when` is a condition or *always*)
  - `crossing`
  - `paired_with` (syndicates)
  - `placement` (card position → region and settlement, from the plan; each routine card has its own)
  - `meetings` (each {`card`, `guild`, `tie`, `person`, `role`, `site`})
- **Card fields:**
  - `faction_story` (id)
  - `phase` (routine / reveal / story / crisis / epilogue)
  - `odd_job` (yes/no)
  - `signature` (yes/no)
  - `private_side` (yes/no)
  - `guild_meeting`: {`guild`, `person`, `tie`} or none
  - `shared_town` (yes/no)
  - `pressure_level` (1–3, phase-3 cards)
- **Plant type:** `quiet`.
- **Way source:** `code_break`. **Removal reason:** `code`.
- **Faction entry:** gains `facts`, for reputation stakes.
- **Rank fields** (`hall_job`, `rank_gate`) are card-schema's, from mechanics §9 (*The guild ladder*); this doc doesn't define them.
- **Kit tags are rules data, not output** (v6c): `want`, `means`, `reach`, `reaches`, `hit`, `hits`, `needs`, `aims`, `decides`, `shapes`, `cost`, `writ`, `limits`, `fits`, `thread` and the per-enemy blow lists sit on kit entries in `data/quests/rules/`. A quest set carries only `kit_rolls`, `throughline` and `means_used`.

## 14. Decisions made without Laz
*v3 changes, from the test run (`quest-test-results.md`): epilogue inside the 12-card cap; the spread check timing; single-pool rerolls; created enemy–heart links; field sites defined; the odd job exposed by the reveal; faction-story proxies; the ally way on the crisis; which main-quest rules apply.*

1. **A chain, not a diamond:** keeps faction stories readable and clearly smaller than the main story.
2. **Chapter 2's move is always an ordeal event,** so the story costs the heart personally, like the main Ordeal.
3. **Region scope cap, no realm moves:** the main story keeps the biggest canvas.
4. **Schism is allowed as a faction crisis.** Succession was dropped: guilds have no sub-factions to put up claimants.
5. **The code-break way** is the faction story's version of the sacrifice way.
6. **About 1 card in 3** of a faction's cards uses a signature premise.
7. **Length is 9–12, not 8–12.** The approved phases can't add up to fewer than 9.
8. **The world-spread rules (§9)** keep the guild stories (six since v6b) from all being "the leader is the traitor".
9. **A guild that has a faction story can't be the main antagonist's side.**
10. **Guild ties (§8b):** only Old Hands – Night Market is canon. The rest are proposed from each guild's trade and canon notes: two fighter guilds as rivals; Lantern bounty writs on Market crews; a Lantern contract camp at the Writers' digs; the Writers' records versus the Old Hands' forgeries; bout wagers through the Market.
11. **v5.3 (World C):**
    - *Ordeal events:* counting the main Ordeal, three events across six stories (the main and five guilds) fill the cap of two exactly, so the old one-*captive* limit was dropped (it made the cap impossible whenever the main Ordeal wasn't *captive*). *(Superseded in v6b: the key is target + event, once each, §9.)*
    - *Every region pulls:* a region with no hall or contact gets a phase-3 card at a field site, since routine cards need a hall or contact. *(Since v6b, routine cards need a big town's hall, §8b.1.)*
    - *Trade pairs* "meet when a shared town allows it" is read as: one story already has a card in a shared town where the other guild has a hall, with a client, seller or go-between slot.
    - *Seized* is defined by the stake changing hands.
    - *Ordeal events are dealt, not rolled:* with no slack in the cap, the placement plan hands out the five guild uses left after the main Ordeal, so no late story is left with nothing legal. *(v6b keeps the dealing, now of target + event for six stories, §9.)*
    - *Enemy shape* keeps the field name `shape` (the test tools read it); only the words change, so it isn't confused with the rulebook's truth *Shapes*.
12. **v6b (Laz's guild changes, 27 Oct); Claude's calls inside them:**
    - *Hall jobs* are not guild-story cards, so they don't count toward the ⅓ region cap or the shared-town rules; their own spread is in locations §4.
    - *A routine card* is set in its hall's town or beside it (the existing *beside* definition), not only at the hall.
    - *The player is never cast* on a player-target door: the slot the blow falls on points at `p-player`; the door's other slots hold the people round the event.
    - *A Within enemy makes its own faction the enemy's side*, as the kits already read it; so W5 and a *Within* Holy House enemy never roll together.
    - *The Holy House's two leaders* (high priest, priestess) hold two seats, not one; either meets the leader rule, and both count for §9.
    - *A door's family must be one the profile allows* (a *never* family is never a door): the Writers, the Old Hands and the Holy House take *stripped of rank*'s Restore door, never its Contest door.
    - *Ties:* Holy House – Lantern Company, Old Hands and Night Market are *none*.
    - **Open for Laz:**
      - Should a guild leader ever be **killed** by a chapter-2 blow? The kits allow it only where they say so (`quest-guild-kits.md`), and require a seat successor.
      - The ⅓ region cap counts story cards only. Should hall jobs have a region cap of their own, or is locations' "every big town offers hall jobs from 3 guilds" enough?
13. **v6c (Laz's wider kits, 27 Oct); Claude's calls inside them:**
    - *Faith as a want:* the fix list counts *faith* among the canon wants, but the shared list (assembly, `factions.md`, card-schema's faction `want`) has six. It is added here for guild-story enemies only (§5). **Open for Laz:** add it to the shared list too?
    - *`uses`, the main means:* each move, blow and crisis records the one means it is carried out with, so writ fatigue can count and step 5 can pick a lever (§11a).
    - *Levers come from means:* a table tying the ten levers to the eight means (§11a).
    - *Writ fatigue's "aimed at the guild":* writs, licences, outlawings, court rulings and seizure orders against the guild, its halls, rights or trade count; paper used against one person doesn't (§9).
    - *Blows:* the hits of each target + event, and *stripped of rank*'s needs (`inside` or `name`), which the fix list didn't give (§11a).
    - *"Personal" in the climb:* lands on someone the player knows by name, or on the hall they work from (§11a, point 4).
    - *The test in two passes:* the blow is dealt after the kit roll, so points 1, 2, 3 and 5 run at the kit roll and the whole test after the deal; on a fail, re-deal the blow, then the move, the crisis, the enemy (§11a, §12).
    - *Endings:* `fits` names stake hits or crisis families; fewer than 3 fitting endings rerolls the crisis; the seat ending is always a fourth, never in the rolled three (§6, phase 4).
    - *Odd jobs:* thread-kind slugs, no two jobs in a kit sharing one, and an optional `needs` (§11).
    - *Roll order ties* go by a keyed roll; the syndicate pairing is rolled before the order is counted (§10, §12).
    - *Schism needs a Within enemy*, as every existing Schism crisis already had (§6, phase 4).
    - *Pool sizes:* "about 8" etc. are aims; the minimums are one or two lower (§11), so a kit is never broken by one entry short.
    - *The merged kits (v6c):* the three pairs of draft writers each added fields to stop nonsense, under different names (an enemy's stake list, *Reaches*, a crisis's stakes in brackets, *Shapes*). They are one set now: `reaches`, `aims`, `decides`, `shapes`, `cost`, `writ`, `limits`, per-enemy blows and per-stake readings (§11a, *Means aren't enough*).
    - *One writ per story,* and *`paper` only when nothing else fits* on a writ part, from the Writers and Lantern drafts; a guild-target blow by `paper`, and the house's `faith` as a ban, count as writs (§9, §11a).
    - *Blow needs tightened for every kit:* *arrested* by `men` only for an *Above* enemy; a *Within* enemy shuts a hall with `inside`; *outlawed* never under a *Within* enemy (§11a).
    - *The roll is enemy first, then a crisis* among that enemy's legal ones (§12 step 1); the roller lists every legal part and rolls among them, so nothing is rolled and rejected.
    - *The Above-and-Beside rule rerolls the last story* among the missing shape, and a failed blow deal rerolls the story with the fewest legal blows, before the world restarts (§12).

## 15. Changelog
- **2026-10-27 (v5.3), v6d: coherence joins** (from the v6c sample review, `test/world/D/narratives/v6c/review-coherence-creativity.md`): the door fits how the blow fell, and the blow's means come from it (§6, §11a point 2, §12; assembly §4); the heart fits the crisis and the blow, and a captive heart never gives the crisis or fills a *herself* one (§6); the crisis giver has a reason (§6); the odd job serves the scheme (§11a point 2); the feud grows from both schemes (§10); under *the cost* the blow protects what pays, and the last move fits the want (§11a point 5); the sentence is written from the variant's own reading (§11a point 6); the enemy rolled by kind first (§12 step 1); `--joins` (§12); v7c: crisis wherever the story is (Laz, 27 Oct): at the story's centre or where the plot comes to a head, giver and parties there, headquarters only for a story about it or its leader, its clock from the map (§4, §6 phase 4, §8b, §12).
- **2026-10-27 (v5.3), v6: World C fixes.**
  - *One truth for a shared move* (§10): the planner fixes the truce killing's facts; only the *Blood debt* crisis answers who killed whom; a shared move counts once for levers, kills and fallout; each story meets its own dead; the pair's reveals are one *hidden hand* (CO-C2, KC2, KC6, GP4, G1-4).
  - Generation order is kits → placement plan → cast → cards; new step *Guild moves reach the towns* writes context and site state facts; new checks for the fallback, site state and the truce truth (§12; GP1, CO-C1, PD-C2, G1-5).
  - *Every region pulls* replaces the guild-presence coverage rule (§8b.1).
  - Ordeal events count the main Ordeal, one-*captive* limit dropped; *the cost* at most twice; main counts in the world ledger caps, not the guild-only ones; reserved levers; lever caps checked at move-writing; phase-position rule covers every pool (§9; SC-C7, G1-2, SC-C5, KC1, KC5).
  - Co-leaders both count as the leader; one enemy side per faction is intended (§8, §9; KC3, KC4).
  - Paired meeting exception worded exactly; trade pairs meet when a shared town allows; "beside" a shared town defined; one state per site (§8b; GP3, GP5).
  - *Seized* defined; crisis may roll a 4th way when a plant-backed way would be left out; turning cards are never plain and get twists first (§6, §8; GP6, PD-C3, SC-C3).
  - Signature quota points to locations (§3; L2B-3).
  - Follow-up: the placement plan deals the chapter-2 ordeal events (§9, §12, §14); "enemy shape" named apart from truth *Shapes* in §5, §11, §12 (§14).
  - v6b: Laz's guild changes (27 Oct). **A:** hall work is in the big towns: routine cards and the reveal at halls in 3+ large or great towns, every pair of stories shares 2+ towns, every big town holds 2+ stories, routine exempt from *Short steps*, chapters still in the field (§6, §8b, §12). **B:** pointers to the guild ladder and hall jobs; a *Hall job* row in §1 (§1, §4, §6, §13). **C:** chapter 2's blow has a target (heart, player or guild) and its event; doors live in assembly §4 *Guild chapter-2 doors*; target + event once per world, each target in 1–3 stories, dealt by the planner; killed leaders, seized headquarters and taken members (§6, §9, §12, §13). **D:** six stories, the Holy House included; its ties (rival and exclusive with the Dueling Society, rival with the Writers); W5 vs a *Within* Holy House enemy; two leaders on two seats; every §8b and §9 number rechecked for six (§1, §5, §8, §8b, §9, §14). v6b audit: §1 lists three kinds of faction-given card; a player-target door's slot points at `p-player`, as assembly, casting and mechanics say (§6, §14); *Every region pulls* puts a routine card only where the region has a big town (§8b.1); v5.3 notes in §14 marked superseded. v6c: Laz's wider kits (27 Oct; `test/world/fixlist-v6c.md`): new §11a *The throughline* (means, hit and reach tags; what each part carries; what each blow needs; levers come from means; the six-point test, run at the kit roll and after the deal, recorded as `throughline`); chapter 1's move rolled from the kit's move table (§6, phase 3); crises have no enemy list, Schism needs a *Within* enemy, 3 endings rolled from a pool of 5–6 to fit (§6, phase 4); faith joins the wants (§5); writ fatigue, at most 2 per world (§9); pairing about 1 in 3, rolled first (§10); wider pools, odd-job thread kinds, variants keep the tags, *Fresh stories* across worlds, kits as data (§11); fewest legal options first, the plan deals blows by means, a throughline step 2b before casting (§12); `throughline`, `means_used`, kit tags as rules data (§13); Claude's calls (§14 item 13). v6c merge: the six kits' extra fields made one set, and §11a says what the drafts learned (*Means aren't enough*: `reaches`, `decides`, `aims`, `shapes`, per-enemy blows, per-stake readings; the cost moves protect; one writ per story); the test reads them; blow needs tightened (*arrested* by `men` only *Above*, *hall seized* by `inside` under *Within*, *outlawed* never *Within*); `uses` takes `paper` last on writ parts; guild-target blows by `paper` count as writs (§9); the paired feud varies (§10); fits may name a crisis or *any*; *Betrayal* needs a trusted link (§6); enemy first, then crisis; rerolls before restarts (§12); tags in §13; §14 item 13.
- **2026-10-26, v5.2: audit fixes.** Leaders are archetypes (§5, §11); layer-2 High people free in authority and background slots, one may be the crossing and the *Above* enemy (§8); reveal givers (§6); short steps and the journey-away card agree with §8b.1 (Story craft, phase 3); crisis rolls exactly 3, forced ways never give their place; seat ending replaces one or is the fourth; crisis caps count guild and faction crises only (§9); names follow the rulebook's *Name check*; one fallout reading line (§8); "crossroads" renamed shared towns, every large town qualifies, `shared_town` (§8b, §13); Old Hands – Night Market *at war*; two tie reasons regrounded; shifted ties are facts; meetings carry a site; placement per routine card; meeting roles checked (§12); slots are specs (§12); paired syndicate crises named (§10); signature premises carry truth rows (§11); §13 matches the schema.
- **2026-10-26, v5.1: guild stories meet (Laz: medium overlap).**
  - New §8b: each story covers 3+ regions; shared crossroads towns; rival and ally guilds meet; one placement plan for all guild stories.
  - Short steps now apply inside a phase. Phases and chapters may move a journey.
  - One phase-3 card may sit in a crossroads town.
  - Generation order: every guild story's kits and cast are rolled before placement.
  - Fixed stale section references: kits are §11.
- **2026-10-26, v5: world-test fixes.** Event ceiling (§7, SC8); *Faction fallout is required*, cap raised to 3 (§8, PD5, CO1); short steps and where cards are offered (story craft, PD4); kit variants, crisis givers varied, kit roller reads the world ledger (§9, §11, §12; W9, G8, SC9, H1); seat ending replaces a kit ending (H2); per-cast-member slot limit, authority slots and givers exempt, heart's routine card chosen at phase start and never the odd job (H3, G1–G3); crossing defined (G5); the enemy's want survives the last move (CO8); plant-backed ways roll first on crisis and turn cards (SC5); main chapters now chain too.
