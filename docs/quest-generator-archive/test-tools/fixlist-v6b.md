# Rule changes v6b (Laz, 27 Oct, after World C)

Laz asked for these four changes before World D. They're his decisions, so they're inside the scope; everything else stays frozen. As in v6, each rule has **one home doc**, other docs point to it by doc and rule name, and every changed doc gets a line added to its v6 changelog entry: "v6b: Laz's guild changes (27 Oct)". Mark anything Claude invents *(Claude)*. Laz's own words are quoted where they set the intent.

---

## A. Guilds share the big towns (Elder Scrolls style)
Laz: "the Lantern Company is our analog for the Fighters Guild and the Writers is our analog for the Mages Guild, and their quests occur on completely separate parts of the map … a player with two different builds is going to miss a big portion of the content… I have pictured it like Elder Scrolls, how the Fighters Guild and the Mages Guild are always in the same town."
In World C, Lantern and Writers shared one town, with one card each. §8b only asked each story to share 2 towns with *some* other guild.

**Home: faction-stories §8b, *Shared towns*.** Replace the per-story count with:
- **Hall work is in the big towns.** Every guild (and now the Holy House, see D) has a hall in every large or great town. A guild story's **routine cards and reveal** are given at its halls there, across at least **3** of those towns. Routine cards are hall work, so *Short steps* doesn't bind them (it still binds chapters).
- **Every pair of guild stories shares at least 2 towns.** That's every pair, not only rival and ally pairs. "Shares" means both stories have a card in that town.
- **Every large or great town holds cards from at least 2 guild stories.**
- **Chapters still go to the field** (phase 3, *Pinned at field sites*). That's where each guild's own content lives: the Writers' digs, the Lantern Company's frontier forts.
- The region rules stand: at least 3 regions per story, *Every region pulls*, and no region over ⅓ of the guild cards.

## B. The guild ladder: rank, and hall jobs that count
Laz: "one-off guild quests should count towards rank … only having 10 or 12 quests for a guild doesn't seem like a lot … a major crisis … a change of leadership or the destruction of the guild or the exaltation of the guild … not having much build up to get to that doesn't seem right." He chose **about 20 quests per guild**: about 11 story cards plus about 9 hall jobs.

- **Home of the rank rules: mechanics §9** (*Reputation and membership*) and §3 (*When quests open*).
  - Rank rises as the player resolves that guild's jobs. **Story cards and hall jobs both count.**
  - Each story beat opens at a rank: the routine cards in turn, then the reveal, chapter 1, chapter 2 and the crisis.
  - Set the gates as first-guess dials so that about 2 hall jobs fall between beats, and a player needs most of the hall jobs but not all of them: they choose which.
  - Name the dials, for example `hall_jobs_per_guild = 9` and `rank_gates`.
- **Home of hall-job generation: locations §4** (*One-off faction-given jobs*), renamed or extended as ***Hall jobs***.
  - Each guild story gets about **9** hall jobs (`hall_jobs_per_guild`), made by the location pass with the guild's profile, at its halls and contacts.
  - At least half are in large or great towns, and **every large or great town offers hall jobs from at least 3 guilds.**
  - Each hall job carries a **`rank`** band (early / middle / late; about 3 each), so later ones carry higher stakes. They are the simple tests: side quests at community scope or smaller. They may be plain, within the world's plain share (rulebook, *No blank card*).
  - **Hall jobs never use the story's secrets:** no cast, no plants, no revelation. They may read the story's public moves through context (`reads_context`): after the hall is seized, jobs at the other halls mention it.
  - They count in the world ledger like any card.
- **card-schema:** add a card `hall_job` {`guild`, `rank`: early / middle / late}, and story-card `rank_gate` (number, filled from the dials).
- **faction-stories §4 and §6:** point to the ladder. The story keeps its four phases; the hall jobs sit between them.

## C. Who the enemy hits in chapter 2
Laz: "maybe sometimes it's not the heart that the enemy hits. Maybe it's the player directly, or maybe it's kind of the guild as a whole … if it's always the heart … after two or three playthroughs, it kind of gets predictable." **Guild stories only;** the main story's Ordeal stays on the heart (Laz).

**Home: faction-stories §6, *Phase 3*, chapter 2's move.** The ordeal event now has a **target**, rolled per story:
- **the heart** (as now): *captive*, *disgraced* (under the code), *estranged*;
- **the player:** *framed* (accused of a code-break they didn't do), *stripped of rank* (thrown out, or put down to a lower rank until they win it back), *arrested* (held and judged; the sim already has guards and judging, `pursuit.rs` / `judge`, so the mechanics doc only points there);
- **the guild:** *hall seized* (a hall or HQ taken or shut), *members taken* (bonded, arrested or marched off), *outlawed* (a writ against the guild in one lordship or march; the *Event ceiling* still holds), *leader struck down* (wounded, killed or disgraced; the kit says which leaders may be).

Each target+event has a **door**: the chapter-2 turn card's family. **Home: assembly's ordeal-door table** (assembly §4, as amended by main-quest §9). Add rows for the new events. Suggested *(Claude)*:
- framed → Investigate or Reckoning (clear your name);
- stripped of rank → Restore *Lost good name*, or Contest (win it back before the guild);
- arrested → Passage (get out) or Dispute (before the judge);
- hall seized → Acquire (take it back) or Dispute;
- members taken → Rescue;
- outlawed → Dispute or Reckoning (the writ's backer);
- leader struck down → Investigate (who) or Reckoning.

The heart still exists in every story (routine, ally plant, crisis); only chapter 2's blow moves.

- **Spread** (faction-stories §9, rulebook *World ledger*): across the world's guild stories, each target is used by **at least 1 and at most 3** stories. The ledger's ordeal-event key becomes **target + event**, and each target+event is used **once** per world; the main Ordeal's (heart, event) counts. This gives slack: 10 combinations for 7 uses. It replaces the "each event at most twice" cap that had no slack. The planner deals them (faction-stories §12), as now.
- **Mechanics:** a player-target event needs the game to act on the player. Framed and stripped of rank touch rank and reputation (§9). Arrested points to the sim's guard-and-judge system. Keep it to pointers.

## D. The Holy House gets a story (a sixth faction story)
Laz: "I'm not seeing anything for the Holy House … I thought it would be pretty important for priestly characters." He chose: **a full story you can join, like the five guilds**, and **joining the Holy House closes the Dueling Society, and the reverse** (the house bans rings in canon).

- **Canon to build from:**
  - `world-factions.md`: the Roduro church with heavy Horaro overlap; corrupt, militarised, selling protection; its hidden side is the sincere priests.
  - `leaders.md`: Hotorìṭo, the high priest, has lost his faith and turned the church into an army; Aloqi, the sincere old priestess, wants to win believers back by deserving them.
  - `faction-places.md` §7: the profile, code, signature premises and vocabulary; §2–§4 for its places (the Mount, the Low Shrine, tithe-houses, inquisitor camps).
- **guild-kits.md: a sixth kit** in the same format as the five: enemies (with shapes and sides), stakes, hearts, odd jobs, crises with givers, endings, the seat ending, and twists, with variants, truth columns tagged with shapes, and the confirming rows marked.
  - Natural crisis endings: reform it under Aloqi, keep the sword, or break the house; the seat ending is becoming its voice.
  - Hotorìṭo's lost faith is a canon trait that the profile says is "secret and never used". For the kit, **keep it unused** unless Laz says otherwise; list it in the doc's *Decisions* section as a question for him.
  - The leader as enemy (Hotorìṭo) is allowed and counts against §9's one-leader-enemy cap.
- **faction-stories:**
  - six stories now; check every §8b and §9 number still works with six (enemy shapes, hearts, crises, reveals, the ⅓ region cap);
  - a **profile** entry, by moving or extending its faction-places §7 profile;
  - **ties:** Holy House – Dueling Society *rival* (the house bans rings; canon) and *exclusive*: joining one closes the other's story, like the syndicates, but not hostile; both are still generated. Holy House – Writers *rival* (inquisitors seize "heretical" texts; Writers kit W5). The rest are *none* *(Claude)*.
  - **One enemy side per faction across stories:** the Holy House may still be the Writers' W5 enemy side only if no other story uses it.
- **factions.md:** the Holy House is a joinable guild-like faction; joining it closes the Dueling Society (Laz).
- **faction-places.md:**
  - every large or great town has a Holy House chapel or parish shrine as its hall *(Claude, for A)*, which settles where the hall jobs and routine work go;
  - its reach becomes **realm** for story purposes (it already reaches the Vale and the Crossing). Log both as *Decisions made without Laz*.
- **mechanics §9:** the exclusivity rule, Holy House ↔ Dueling Society.

## Numbers for World D
Planned size **N ≈ 220**:
- main about 40;
- 6 guild stories × about 11 = 66;
- hall jobs 6 × 9 = 54;
- other side quests about 60.
