# Mechanics Layer — Draft (v0.4, 2026-10-27)

What happens **after** a quest is generated: how quests open, run and end, and how their endings change the world. This covers everything parked in rulebook §13.

**Laz's calls:**
- **When a key person changes:** situational (§6 is the table).
- **Checkpoints:** driven by both world time and main-story progress (§5).
- **Simulation events:** trigger quests too, through the same signals as the player's actions (§7).

---

## 1. The idea in one paragraph
Everything is **facts and signals**.
- A **fact** is a true statement about the world ("the captive is free", "the quay is seized"). The shape layer already writes them.
- When something happens, the world **adds or changes a fact**, and that change goes out as a **signal**.
- Quests **listen** for signals: "open when X is true", "close when Y is true".
- **Endings write facts.** Nothing else in the game needs to know about quests.

**Shapes never change.** The world reacts only by opening, closing or adding quests, never by rewriting one.

## 2. Quest life cycle

| State | Means | Moves on when |
|---|---|---|
| **hidden** | exists, but the player can't get it yet | its availability rule (§3) becomes true → *available* |
| **available** | can be picked up at its `offered_at` place | the player accepts → *active*; a close rule fires → *closed* |
| **active** | the player is on it | an ending is reached → *resolved*; a forced close (§6) → *resolved* with an unattended ending |
| **resolved** | finished with one ending | — (its effects have already fired) |
| **expired** | a clocked card's checkpoint passed first | its `world_ending` plays out (§5) |
| **closed** | can no longer happen (a key person is gone, or it was made moot). A card **withdrawn** by a clock's ending (locations, *Clocks*) is closed | — |

- **Discovery** (how the player hears of an *available* quest: notice boards, rumours, being approached) is a separate system. It reads the same states.

## 3. When quests open (progression)

| Quest kind | Opens when |
|---|---|
| **Main story** | The hook chapter is available from the start. Inside a chapter, each card opens when the one before it resolves. A layer's chapters open when the turn card of a linked chapter in the layer before resolves, with any ending. |
| **Unchosen siblings** | When the player advances through one node, the other nodes in that layer stay open as **optional**: they slide to minor relatedness and no longer advance the story. One taken late, after a later layer has opened, reads the moves since through its settlement context (its `reads_context` faces), so it never reads stale (locations, *Settlement context*; Lane B's text templates, `quest-card-text.md`). |
| **Main moves** | A layer's move becomes a world fact when that layer opens. This is when the world visibly changes. |
| **Minor quests** | open with their chapter; they close when their target node resolves |
| **Fallout** | opens when its move becomes a fact |
| **Late fallout** | opens when its checkpoint passes **and** the player hasn't yet reached the next main layer (§5) |
| **Faction story** | Each **beat** (every routine card, the reveal, each chapter's opening card, the crisis) opens when the card before it resolves **and** the player's rank in that guild has reached the beat's `rank_gate` (§9, *The guild ladder*). The joining card (routine card 1) has no gate. Middle and turn cards inside a chapter open in order, ungated; the epilogue opens when the crisis resolves. Moves become facts as their phase opens. |
| **Hall jobs** | open when the player is a member of that guild and their rank has reached the job's band (§9). A job **made by a move** (locations §4, v7) also waits until that move is a fact. |
| **Side quests** (echo, independent) | open from the start, or when their dependency fact exists (the hook harm, a public fact) |
| **Linked pairs** | card B opens when card A resolves, with any ending |
| **Exclusive pairs** (the syndicates; the Holy House and the Dueling Society) | joining one closes the other's story and its hall jobs (§9) |

## 4. Endings write facts (effect vocabulary)
Every ending maps to a short list of **effects**. These are the only effects that exist:

| Effect | Example |
|---|---|
| `set_fact` / `clear_fact` | "the sluice works again"; clear "the pans are broken" |
| `set_state` | a person becomes free, held, dead, disgraced or fugitive; a place becomes broken, seized or abandoned |
| `move` | the cargo is now at its destination; a fugitive is at their haven |
| `transfer` | ownership of a thing or place changes |
| `link` | a new link: the player gains an ally, or a grudge |
| `reputation` | a change with a faction, a sub-faction, or a named person |
| `rank` | the player's rank in one guild goes down or comes back (§9, *Blows that fall on the player*) *(Claude)* |
| `open` / `close` | another quest opens or closes |
| `generate` | ask the shape layer for **one new card** built from the new facts (§8) |

**Default effects by family** (an ending's own wording can add to these):

| Family | Example ending → effects |
|---|---|
| Rescue | *freed* → captive state cleared; captive moves home; link the player as the requester's ally · *moved elsewhere* → captive held at a new place (may `generate` a follow-up) |
| Investigate | *accuse before the arbiter* → culprit disgraced or held · *blackmail* → link the culprit with a grudge against the player; the player gains leverage (a fact) |
| Dispute | *A wins* → stake goes to A; B gains a grudge against A · *divided* → split ownership fact |
| Threat | *threat ended* → pressure fact cleared · *community lost* → place abandoned; its people move (may `generate`) |
| Passage | *delivered* → cargo at destination · *handed to the pursuer* → cargo held by the pursuer |
| Acquire | *obtained and handed over* → the thing goes to the requester · *destroyed* → the thing is destroyed |
| Reckoning | *wrongdoer ruined* → wrongdoer disgraced, standing −1 · *killed or driven off* → dead, or fugitive with a new place |
| Restore | *restored* → broken state cleared · *left broken* → state stays; dependents may leave |
| Contest | *won* → what rode on it goes to the winner · *turned deadly* → a death (§6 applies) |

### No empty failure (v7, A5)
Every failure, walk-away or unattended ending writes a **state on a named person or place**: the captive is sold on, the works are abandoned. An ending that only closes the card is not allowed. About **one default in five** also carries `generate` (marked ⟳ below). These are the defaults; an ending's own wording can add to them *(Claude: first guesses)*.

| Family | Ending → what it writes |
|---|---|
| Rescue | *moved elsewhere* → captive held at a new place, one access step harder ⟳ · *rescue fails* → captive still held; the holding place is *on guard* (a fact: a later card there loses its stealth ways) · *handed to another* (B7) → captive held by whoever took them, at their place; the requester gains a grudge against the player ⟳ |
| Investigate | *truth not found* → the already-blamed red herring takes the blame (disgraced, or turned off); with none blamed, the culprit *got away with it* (a fact a later card may read) · *let it go* → the culprit *got away with it* · *the wrong one accused* (B7) → the accused red herring is disgraced or held; the culprit *got away with it*; the accused gains a grudge against the player ⟳ · *seen and lost* (*Watch them*) → the subject is *wary* (a later card's watching ways against them drop out) and gains a grudge against the requester |
| Dispute | *the player walks away* → the stake stays with whoever holds it now (with nobody, the stronger party); the other gains a grudge against them; under *Caught in the middle* the occupant leaves (`move`) · *made worthless or abandoned* → the stake place is *abandoned* · *the player takes the stake* (B7) → `transfer` to the player; both parties gain a grudge against the player; reputation drops with both parties' factions |
| Threat | *protected, but the threat remains* → the pressure fact stays, and the community is *under threat* (a state later cards there read) · *community lost* → place abandoned; its people move ⟳ · *turned back* (*Go and look*) → the place's occupant is *alert* (it knows it was looked at), and the sign at the community spreads (a state fact) |
| Passage | *seized or lost* → the cargo is held by the obstacle (a person: held; goods: seized at the checkpoint place); the sender owes the recipient (a debt fact, §12) ⟳ · *handed to the pursuer* → the cargo is held by the pursuer · *kept or redirected by the player* → the cargo is `held_by` the player or whoever they chose; the sender gains a grudge against the player |
| Acquire | *left with the holder* → the holder is *wary* (a later card's take and deceive ways at that holding place drop out); the requester is *still without it* (a state) · *kept or redirected by the player* → the thing is `held_by` the player or whoever they chose; the requester gains a grudge against the player |
| Reckoning | *reckoning fails* → the requester is *still owed* (a state); the wrongdoer gains a grudge against the requester · *player sides with the wrongdoer* → the wrongdoer becomes the player's ally; the requester gains a grudge against the player |
| Restore | *left broken* → the state stays; the named member, or the requester's household when there is none, leaves (`move`) · *abandoned* → the place is *abandoned*; its people move ⟳ · *the player profits* (B7) → the thing is fixed on the player's terms (a toll, a share, the gathered funds kept: a fact); the requester gains a grudge against the player |
| Contest | *lost* → what rode on it goes to the other side; the champion is *beaten* (a fact) · *thrown* → as *lost*, plus a secret fact on the champion · *called off* → the oath fact records who withdrew, and that side's standing with the ring drops (a reputation change) |

- **Check:** by script, no failure ending only closes the card.
- **The selfish and wrong-guess endings** (B7: *handed to another*, *the wrong one accused*, *the player takes the stake*, *the player profits*) always link a grudge to the player. They feed the grudge loop (§14).

### How it was won (v7, C5)
Every ending records **how** it was won, so the world can remember it.
- **`won_by`:** a fact the ending writes, with the card as `source`: {`kind`, `how`}. `kind` is the family kind of the way that decided it (force, price, question, deal…). `how` is that kind's plain reading in §14's table, or the way's own `how` when its template carries one (card schema, `ways_pool`).
- **Unattended and walk-away endings** record `how: none`.
- **Visibility:** public, unless the deciding way went unseen (`how` *stealth* or *guile*) and the ending is private; then it is secret, but known to the opposing person *(Claude)*.
- The grudge loop reads it (§14). Nothing else needs to.

## 5. Time: checkpoints and clocks (both kinds of time)
There are two clocks:
- **World time:** checkpoints by season. A checkpoint passes every season of in-game days.
- **Story time:** how far the player has got in the main story (the layer reached).

| Thing | Uses |
|---|---|
| **Clocked side quests** | world time. When the checkpoint passes before the card resolves, the card *expires* and its pre-rolled `world_ending` plays out. That ending writes a **state fact** on its place or person, with `when` = the checkpoint; cards at that place read it (locations, *Clocks*). A card there that can't hold after it is withdrawn when the checkpoint passes: *hidden* or *available* → *closed*; *active* → a forced close (§6). |
| **Late fallout** | both: the checkpoint has passed **and** the player is still before the next main layer |
| **Main-story moves** | story time only. The main villain never moves without the player. When a layer opens, its move reads the facts the player has resolved: if its target is already resolved, the move's despite-you face plays (main-quest, *Wins stick*). A move never silently undoes the player's win. |
| **Faction-story moves** | story time, by the player's progress in that guild (its cards and rank gates, §9) |
| **Pressure resolving on its own** | only through clocked cards. Nothing else resolves while the player is away. |

## 6. When a key person changes (the "situational" table)
When an effect, or a simulation event (§7), kills someone, imprisons them, disgraces them or turns them, every other quest that uses that person checks this table. What happens depends on **the role they play** in that quest, and **how far along** the quest is.

| Their role in the other quest | Quest not yet active | Quest active |
|---|---|---|
| **The one who wants it** (requester, heart) | swap to their **fallback** if casting chose one and they share the want (kin, partner); otherwise **close** | the fallback takes over mid-quest if one exists; otherwise resolve with an unattended ending |
| **The opposition** (captor, culprit, holder, wrongdoer, raider leader) | **close as moot** and `generate` a follow-up if the change leaves a hole (a dead holder's heir now has the thing) | resolve with the ending closest to what happened (a dead captor means the captive is freed) |
| **The target** (captive, cargo, missing person) | dead: **close**, and `generate` a Reckoning or Restore follow-up if someone would want one · freed or moved: **close as done** | resolve with the matching ending |
| **A helper or optional slot** (a guide, a witness, a sponsor) | drop the ways that name them; if fewer than 2 ways remain, **close** | the same; the player is told |
| **An authority** (arbiter, magistrate, ring-keeper) | **recast** another person of the same class (the casting pass) | recast |
| **Story cast** (main or faction) | **protected:** no other quest's ending or simulation event may kill, imprison or turn them. Those endings are softened to *driven off* or *disgraced*. | protected |

- **Before anyone dies, the effect checks whether they are protected.** Story cast only change through their own story's beats.

## 7. Simulation events (both kinds of trigger)
- The simulation (armies, hunger, trade, old age, the green spreading) **writes facts through the same signal system** as the player's actions.
- **Quests can't tell the difference.** A lord dying of old age and a lord killed by the player both go through §6.
- **Limits:**
  - the simulation can never touch protected story cast (§6);
  - it can never contradict a generated **secret** fact. It can still produce events that reveal one later through a quest.
  - It can `generate` new cards, at the same budget as quest endings (§8).

## 8. The world reacting: generating new cards at runtime
- The **shape layer runs in the game too.** When an effect asks for `generate`, the generator builds one new card from the current facts and registry, following every shape rule. Its place is where the change happened.
- **Budget:** at most one generated follow-up per resolved quest, and a small number per checkpoint from the simulation. This stops cascades.
- New cards only add. They never rewrite anything.

## 9. Reputation and membership
- **Reputation** is kept per faction, per sub-faction and per named lord (`factions.md`: entry reads the parent; quests and hostility read the child).
- **Changes come from endings:**
  - helping a requester raises reputation with their faction;
  - acting against an opposing slot lowers it with theirs;
  - a **code-break way** costs heavily with the guild that has the code;
  - a **lesser code-break way** (rulebook rule 3, v7) costs less: `lesser_break_cost`, a dial, default **half** a code-break's *(Claude)*. It costs no rank.
- **Joining a guild** happens at a faction story's first routine card, when the player takes it. Six stories can be joined: the five guilds and the Holy House (Laz, v6b).
- **Exclusive pairs.** Joining one closes the other's story and its hall jobs. Both stories are still generated.
  - **The rival syndicates:** joining one also sets the other's members hostile (`factions.md`).
  - **The Holy House and the Dueling Society** (Laz, v6b; the house bans rings in canon): joining one closes the other. **Not hostile:** the other's members keep their ordinary standing with the player.
- **Rooted:** joining is one-way. It removes all other memberships.

### The guild ladder (Laz, v6b)
Laz: "one-off guild quests should count towards rank." A guild is about **20 quests**: its story (about 11 cards) plus about **9 hall jobs** (locations §4, *Hall jobs*).
- **Rank** is a number per guild: how many of that guild's jobs the player has resolved. **Story cards and hall jobs both count**, one each, with any ending *(Claude: the story itself moves on with any ending, so the ladder does too)*. A code-break costs reputation (above), not rank. A meeting card counts only for the story it belongs to.
- **Beats open at a rank.** Every story beat carries a `rank_gate` (card-schema): the routine cards in turn, then the reveal, chapter 1's opening card, chapter 2's opening card and the crisis (§3).
- **Gates are dials** (first guesses, in the rules data):
  - `hall_jobs_per_guild = 9`;
  - `hall_jobs_wanted`: how many hall jobs the player should have done by each beat. Default *(Claude)*: routine 2: **0**, routine 3: **1**, routine 4: **1**, reveal: **2**, chapter 1: **3**, chapter 2: **5**, crisis: **7**;
  - `rank_gates`: worked out per story, as **the story cards before the beat + `hall_jobs_wanted`**, and written on each gated card. A story with 4 routine cards and chapters of 3 and 2 gates at none (the joining card), 1, 3, 4, 6, 8, 13, 17.
- **What that gives:** 0–1 hall jobs between the routine cards, 1–2 between the later beats. A player needs **7 of the 9**, so they skip 2 of their choice. (Two between *every* beat would need about 14, more than the 9 Laz chose, so the early gaps are lighter.) Rank counts every job, so doing extra early carries forward.
- **Bands.** Each hall job has a `rank` band (about 3 each). A band opens at a rank *(Claude)*:
  - **early:** on joining;
  - **middle:** at the reveal's gate;
  - **late:** at chapter 2's gate.
- **Hall jobs are for members** *(Claude)*. A non-member is never offered one. A blow that throws the player out (below) closes them until it's undone.
- **No dead ends on the ladder** (v6b audit decision). Hall jobs are never clocked (locations §6). A hall job that closes without the player resolving it (its key person gone, §6, or the card dropped) is made again at another hall of the same guild, in the same rank band. It is a runtime card, cast on its own at once (casting §9). Hall jobs closed by an exclusive pair (§3) aren't made again.

### Blows that fall on the player (v6b)
Chapter 2's blow may fall on the player (assembly, *Guild chapter-2 doors*). The game acts on the player only through these pointers; the door card is the quest.
- **Framed** (accused of a code-break they didn't do): reputation with the guild drops as for a code-break, from the move. The door's clearing endings give it back; its other endings leave it. Rank doesn't change.
- **Stripped of rank:** the move writes `rank`: the player is **put down** by `stripped_cost` (dial, default **2**, never more than the spare hall jobs) or **thrown out** (no hall jobs from that guild until the door resolves). The door's restoring endings (*restored*, *won*) give the rank back in full. Any other ending readmits the player at `stripped_cost` below, which the spare hall jobs can make up *(Claude)*. The story's own chapter-2 cards never wait on the blow: they open in order as usual.
- **Arrested** (held and judged): the sim's guards and judges do it, not the quest system (`pursuit.rs`, `judge` in `law.rs`). The move names the charge and the town; the town's custom sets the sentence (a fine or a bond, a duel, shunning, the record). The door card is what comes of it. How a move starts an arrest is open (§16).

## 10. The player as a registry entry
- The player is the person `p-player`, with a standing, links (allies, grudges), reputation, memberships and a rank in each guild joined (§9).
- **The player is never cast.** When a guild story's chapter-2 blow falls on the player, the card's slot for the one it falls on (the accused, the subject, the cargo, the wronged) points straight at `p-player`. Every other slot is cast as usual; the requester is never the player.
- **Endings link the player** to the people involved: the requester becomes an ally; the wrongdoer gets a grudge. That's how a world remembers the player.

## 11. Minor advantages and allies
- **A minor quest's advantage ending** makes its fact true. Its target main node then gains that benefit: an extra way, or a hard part removed.
- **An ally answers** when its price has been paid (a fact). The ally way on the climax or crisis is then usable; otherwise it's shown but locked.

## 12. Deals that come back
- Endings like *a deal is struck* or *bargain struck* write a **debt fact** with a due checkpoint.
- When it comes due unpaid, the world `generate`s a follow-up (a Reckoning, a Threat, a Dispute) around it.

## 13. Saving
The whole game state is the registry (including each place's built `detail`, once the player has come near), the facts, and each card's state and ending. All of it is plain data, so a save file is just that list. A story's `registry_updates` are already merged into the entries they add to, so they need nothing saved of their own.

### Fresh stories: not repeating the last game (Laz, v6c)
A player who starts a new game shouldn't meet the same guild stories again. Four sample worlds showed one guild telling the same plot three times in four (`test/world/D/narratives/guild-narratives.md`).
- **A small file between playthroughs:** `recent_stories` *(Claude's name)*. It lives beside the game's settings, **not in any save**, so loading an old save never changes it *(Claude)*.
- **What it keeps, per world:** the kit entries each guild story used (its **enemy**, **crisis** and **odd job**, by entry id, without the variant letter), plus the main story's **arc** and enemy **shape**.
- **When a world is written in:** when it is **finished or abandoned**. Starting a new game while a world is unfinished counts as abandoning it *(Claude)*.
- **Only the last two worlds** are kept. A third pushes out the oldest.
- **What the generator does with it:** the roller treats those entries as **unavailable unless nothing else fits** (faction-stories §12 step 1; the main story reads its arc and shape the same way). When nothing else fits, the older world's entries come back first *(Claude)*.
- **Nothing else carries over:** no ledger counts, no people, no places, no endings. Each world's ledger still starts empty.
- **The split:** the shape layer just reads a list. Keeping the file is this layer's job.
- **Tests and sample runs** roll with no list, so a kit's own variety shows (fix list v6c, *Numbers to check*).

---

## 14. Recurring enemies: the grudge loop (Laz, 2026-10-26)
Enemies and friends who **remember the player and come back**, built from parts we already have. It is deliberately **not** a copy of WB's patented Nemesis System (US 10,926,179, filed 2015, reported valid until about 2035). See the design-around notes below. *Not legal advice: have a patent attorney look before release.*

**The loop**
1. **Remember.** When someone in an opposing slot **survives** an ending (driven off, fled, bought off, bargain struck, talked down), that **same person** gains a grudge link to the player and the ending's **`won_by`** fact (§4, *How it was won*), worded for them ("the player burned his camp").
2. **Change.** That **same person** rolls one change, drawn from `won_by`'s `how` (table below):
   - **a lesson:** *won't be bought twice* removes bribe ways against them;
   - **a scar:** *fears fire* adds a weakness way the player can learn about;
   - **a step up in the world:** a social standing step (a raider becomes a landholder), recorded as a fact, not a combat stat.

**What each `how` covers, and the change it leans to** (v7, C5). Every family kind has one plain reading. A way whose text plainly says otherwise carries its own `how` (an arrangement that calls in a favour is *word*; a bargain of honest work is *work*). The changes are first guesses *(Claude)*.

| `how` | Family kinds | Leans to |
|---|---|---|
| force | Rescue *force*, Passage *force*, Threat *confront*, Reckoning *violence* | a scar (fears fire, hires guards) |
| stealth | Rescue *stealth*, Passage *cover*, Acquire *take*, Investigate *follow* | a lesson: keeps watch (stealth ways against them need an insider) |
| coin | Rescue *price*, Acquire *trade*, Passage *arrangement*, Threat *bargain*, Dispute *deal* | a lesson: won't be bought twice |
| law | Reckoning *justice*, Dispute *side*, Contest *rules* | a lesson: clean papers (law ways against them need hard proof) |
| word | Rescue *leverage*, Investigate *question*, Restore *persuade*, Reckoning *amends*, Contest *stakes* | a lesson: won't hear it (one word way against them drops out) |
| guile | Acquire *deceive*, Investigate *provoke*, Threat *undermine*, Reckoning *ruin*, Contest *edge* | a lesson: trusts nobody (deceive ways against them drop out) |
| work | Restore *craft*, *gather*, *substitute*; Acquire *earn*; Threat *protect*; Investigate *examine*; Passage *route*; Dispute *truth*; Contest *prepare* | a step up in the world |
3. **Return through quests.** In the casting pass, someone with a grudge scores high for opposing slots in **new** cards (runtime follow-ups, late fallout). They come back as the captor of your ally, the raider on the town you saved, or the wronged party in a Reckoning against **you**.
4. **Death opens a quest, not a successor.** A killing is a fact like any other. Like any death, it can lead the generator to build a *Blood debt* card whose requester is the dead person's kin. That's an ordinary new quest from a fact; no other character is changed by the killing itself.
5. **Friends too.** Requesters the player helped hold an *ally* link, and turn up as "call in your allies" ways in later cards.

**Limits**
- **At most 5 active grudges.** The oldest or weakest fade into ordinary background.
- **Story cast are protected** (§6). Only people outside the main and faction casts can become recurring enemies.
- **Returns share the generation budget** (§8).

**Design-around notes** (how this differs from the patented system)
- **Claim 1 of the patent** covers an interaction between the player's avatar and one opposing NPC causing the game to change a **second** opposing NPC's parameters, and output an indication of that change.
  - **We never do that:** each change applies only to the person the player actually dealt with.
  - Other people enter the story only as **new quests** made from public facts, the same way any event makes quests.
- **No army hierarchy, promotion ladder, power struggles between enemies, or rank screen.** Grudges are quest-casting preferences and facts, not a ranked roster.
- **Changes are story facts, not combat stats.** They are shown through dialogue, rumours and quest text, never as a "this enemy was upgraded" display.
- **Recurring enemies return through the quest system,** not as roaming hunters who ambush the player in the open world.

## 15. Decisions made without Laz (flagged)
1. **Unchosen siblings stay open as optional side content,** rather than closing when the player picks a path.
2. **The main villain never acts without the player** (story time only). The world still moves through clocks and late fallout.
3. **Story cast are protected** from other quests and from the simulation.
4. **Runtime generation is limited** to one follow-up per resolved quest.
5. **A season is one checkpoint.** How many in-game days that is, is a tuning number.
6. **Reputation is three-level:** faction, sub-faction, named lord.
7. **Withdrawn is closed.** A card a clock's ending withdraws (locations, *Clocks*) moves to *closed*; there's no separate state. A card the player is already on gets a forced close instead (§6): *resolved*, with an unattended ending.
8. **The ladder's first guesses (v6b):** any ending counts toward rank; `hall_jobs_wanted` defaults to 7 of 9; bands open on joining, at the reveal's gate and at chapter 2's gate; hall jobs go to members only; `stripped_cost` 2, readmission after a lost door, and a new `rank` effect.
9. **Rank ladder dead end (v6b audit):** hall jobs are never clocked; one that closes unresolved (key person gone, card dropped) is made again at another hall of the same guild in the same rank band, through runtime casting (casting §9) (§9, *The guild ladder*).
10. **Fresh stories (v6c):** the file is named `recent_stories`, kept beside the settings and outside every save; a new game over an unfinished world counts as abandoning it; entries are kept without their variant letter; when nothing else fits, the older world's entries come back first (§13, *Fresh stories*).
11. **v7 book rules:** the failure defaults and which carry `generate` (§4, *No empty failure*); `won_by` is secret when the deciding way went unseen; the `how` readings and the change each leans to (§14); `lesser_break_cost` is half a code-break's (§9).

## 16. Not decided yet
- How discovery works (boards, rumours, being approached).
- Exact reputation numbers and thresholds.
- The length of a season in days.
- What the player's starting standing and links are.
- Rank titles (what each guild calls its ranks), and whether rank shows on screen.
- Arrests: how a quest move starts one in the sim, which squad member it takes, and whether the sim needs somewhere to hold them before judging (it judges on the spot today).

## Changelog
- **2026-10-27 (v0.4), v7: Laz-approved book rules (A5, A6, B5, B7, C5):**
  - §4 *No empty failure*: every failure, walk-away and unattended ending writes a state on a named person or place, about one default in five with `generate` (A5); defaults for the new endings *handed to another*, *the wrong one accused*, *the player takes the stake* and *the player profits* (B7), and for the field-work endings *seen and lost* and *turned back* (C1).
  - §4 *How it was won*: every ending writes `won_by` {`kind`, `how`} (C5); §14 reads it, with one table of `how` readings and the change each leans to.
  - §3: a hall job made by a move waits for that move (A6); §9: a lesser code-break costs half a code-break's reputation (B5); §15.
- **2026-10-27 (v0.3), v6: World C fixes:**
  - §5: a clock's world ending writes a state fact (with `when` = its checkpoint) that cards at that place read (PD-C5).
  - §3: an unchosen sibling taken late reads the moves since through its `reads_context` faces (PD-C5).
  - §13: `registry_updates` are saved as part of the entries they were merged into.
  - §2, §5, §15: a card a clock's ending withdraws is *closed* (no new state).
  - v6b: Laz's guild changes (27 Oct): *The guild ladder* (§9; rank from story cards and hall jobs, `rank_gates` from the dials `hall_jobs_per_guild` and `hall_jobs_wanted`, bands, about 20 quests per guild); §3 beats open by `rank_gate`, hall jobs by band; *Blows that fall on the player* (framed, stripped of rank, arrested to the sim's guards and judges) and a `rank` effect (§4, §9); the player is never cast (§10); Holy House ↔ Dueling Society exclusive, not hostile; six joinable stories (§3, §9); open questions on rank titles and arrests (§16). v6b audit: the gate example's first entry reads "none" (the joining card has no gate, as §3 and card-schema say). v6b audit decision: hall jobs are never clocked, and one that closes unresolved is made again at another hall of the guild in the same band (§9, *The guild ladder*; §15). v6c: new rule *Fresh stories*: a small `recent_stories` file outside the saves keeps the last two worlds' guild entries (enemy, crisis, odd job) and the main arc and shape; the roller treats them as unavailable unless nothing else fits; nothing else carries over (§13, §15).
- **2026-10-26 (v0.2):** saving names the places' built `detail` (lanes cross-check).
- **2026-10-26 (v0.1), audit fixes:** sections renumbered in order (grudge loop §14, decisions §15, open questions §16); fallbacks are chosen by casting.
