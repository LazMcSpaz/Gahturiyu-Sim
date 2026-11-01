# Rule fixes v6 (from World C)

Laz, 27 Oct: "Make the fixes then run world D before writing any code." Scope is frozen (`quest-parallel-plan.md` §0). Every item here clarifies, tightens or loosens an existing rule. **None adds a feature.** If an item seems to need a new system, write the smallest rule that does the job and log it under *Decisions made without Laz* in that doc.

Finding ids (KC1, CM4, SC-C2…) refer to `test/world/findings-ledger.md`, World C section. Each rule has **one home doc** (marked **Home**); other docs refer to it by doc + rule name (for example "rulebook, *World ledger*"), never by section number. The rule names in *italics* below are fixed: use them exactly so cross-references match.

Every doc you change gets a version bump and a changelog line: "2026-10-27 (vX), v6: World C fixes" plus a short list.

**Field names are fixed here** (the test tools read them):
- Card `truth_row`: string, "<Family or signature premise>: <row>", exactly as the family doc's truth table names the row.
- Card `shapes`: list of shape slugs (rulebook, *Shapes*): one for the truth's row, one for the twist if it has a shape.
- Card `plain`: true when the truth is a *confirming* row and the twist is *none*.
- Card `reads_context`: list of {`fact`, `before`}. `fact` is a context fact id the card's text uses; `before` is the card's plain reading for when that fact isn't true yet (null when the fact is certainly true before the card can be offered).
- Fact `when`: already required on past events. Context facts (moves, faction-story events, clock endings, state changes at a site) **must** carry it: a move id, a faction-story position (`<story>:<phase or chapter>`), or a relative day.
- Story `registry_updates`: list of {`entry`, `add_facts`, `add_links`, `set` (only `dies_at`, or a `state` fact id)}. Add-only.

---

## Group 1 — rulebook.md, card-schema.md, the nine family-*.md docs

1. ***Shapes*** (SC-C2, PD-C1, CM4, G2-5, G1-3, L3B-1, L2B-1, L1B-2). **Home: rulebook, World ledger.**
   - A **shape** is the mechanism a truth or twist turns on, under whatever words or keys it wears. Truth rows that are variants of one another, carry a suffix ("bystander culprit (greed)", "(cover-up)"), or are a signature premise's copy of a base row ("The favor called in: still profiting") have **one shape**. Rows in different families that turn on the same mechanism share a shape too: the bystander blamed; both sides partly right; the record is lost or survives; harm comes from upstream (bad water, waste upstream, a project that spoils the water); a better buyer or promised elsewhere; still profiting from it; it was a copy; someone inside sold the way in; and so on.
   - The rulebook gets a short **shape list** (aim for 30–50 slugs). **Every truth row in every family doc and guild kit gets one shape slug**, written beside the row in its truth table. Twists get a shape only when the twist itself names a mechanism (*a better buyer*, *the fix protects someone*, *loosened in the night*); most twists' names are their shape.
   - **Confirming rows** (the truth only confirms what the requester or premise says: *as it looks*, *as told*, *as declared*, *honorable*, *the stated reason*, *project sound*, *innocent* where the requester already says so) are marked *(confirming)* in each truth table. They have no shape and don't count against shape caps.
2. ***World ledger*, rewritten caps** (CM4, SC-C2, L3B-1, L0B-2). **Home: rulebook.**
   - It counts: family + premise; **shapes** (in place of truth-row keys); twists; move levers; crisis family + premise; **ordeal events** (the main Ordeal's event and every guild chapter-2 event, together; SC-C7); **way wording** (see item 5).
   - Caps scale with the planned size of the world (N cards, set before generation):
     - family + premise: about 4% of N (at least 2), unchanged;
     - a truth row: max(3, ⌈N ÷ 40⌉);
     - a shape: max(4, ⌈N ÷ 30⌉), counting every row and twist that has it;
     - **a family is never closed by the ledger:** when every non-confirming row of a family's premise is at its cap, those rows each gain one. A capped card rerolls its truth row, then its premise; it never drops its truth to dodge a cap;
     - an ordeal event at most twice in the world (main and guilds together); *the cost* as a reveal at most twice, worded differently;
     - crises: unchanged.
   - **Reserved levers** (SC-C5): the main story reserves its last move's lever **and every lever it uses on 2 or more moves**; other stories may use each at most once. The mechanism counts too: a side premise or ordeal event built on the reserved mechanism (called notes, say) counts as a use.
   - **Background people in truths** (SC-C8): a High or Top person who appears only inside a truth or twist (the better buyer, the fixer above) fills such a role on **one** card per world.
   - **Passes run in sequence.** A real generator runs main, guild and location passes one after another, so each reads the ledger as the last left it. Parallel writers (paper tests only) get **reserved shares**: the planner splits what's left of each cap between them beforehand (G2-4, G1-1, L0B-2, L2B-2).
   - G1-2: the main story counts toward the world ledger's caps; the guild-only caps in faction-stories §9 count guild stories only. Say so in both places.
3. ***No blank card*, refined** (SC-C1, SC-C3, L2B story). **Home: rulebook, rule 8.**
   - A confirming truth **is** a truth, so a card with one is never blank. Such a card with twist *none* is **plain** (card `plain: true`).
   - **Turning cards may not be plain:** main chapter turn cards, the Ordeal, the climax, the Return, faction-story reveals, phase-3 turn cards and crises each need a non-confirming truth or a twist (or both).
   - **Side cards may be plain** (location cards, echoes, linked pairs, one-off faction jobs, fallout): at most about **1 in 4** of a world's side cards. Plain cards give the twisted ones contrast. A plain card still needs a stake the player can see and a choice that matters.
   - **Minor quests:** at least **1 in 3** has a twist.
4. ***Roll order* additions.** **Home: rulebook, rule 3 (ways) and §9 (twists).**
   - **Twists before truths when the twist decides the truth** (G1-7, L3B-2): roll the twist first, then the truth from the rows the twist allows.
   - **Twist budget goes to turning cards first** (SC-C3, CM2): in each storyline, roll the twists of the climax, the Return, the Ordeal, then chapter turn cards and crises, then the rest. This is the order twists are *rolled*, not the order the cards are written.
   - **Forced ways overflow** (PD-C3): when the always-rolled ways and the plant-backed ways would fill all 3 slots on a climax or crisis card and a plant-backed way would be left out, that card rolls a **4th** way. Only the climax and crises may.
5. ***Ways name what's on the card*, wording** (SC-C6). **Home: rulebook, rule 3.** A template way is filled from its card: it names a place feature, a thing or a person on the card when the card has one, so the same template reads differently on different cards. The ledger counts each filled way's opening phrase (its verb and what it acts on, such as "go over the charge with"); no opening phrase more than **3** times per world. Lane B's fill grammar does the filling (CR-B2); this rule only sets the limit.
6. ***Seats and offices have one holder*, additions** (L0-4). **Home: rulebook.**
   - **One office per person** (canon people excepted when canon gives them two).
   - **Co-leaders sharing a seat:** the seat names one holder; the other holds a `title` at the same place. For leader caps (faction-stories §9) both count as "the leader" (KC3).
7. ***Endings don't remove people a later card needs*, additions** (CO-C5, G2-1). **Home: rulebook.**
   - The check covers **every** card that can remove someone: blood bouts and deadly duels (the stake and both fighters), minors, fallout.
   - A person who carries a **plant** a later card needs may not stand in a bout that can turn deadly unless the plant is also given to their fallback.
8. ***Card text only says what's true then*, addition** (CO-C3). **Home: rulebook.**
   - **No ids or notes in what the player reads.** Fact ids, entry ids and writer's notes never appear in a card's pressure, ways, details, place tie or endings.
   - A card that names a context fact lists it in `reads_context` with its `before` reading (locations, *Settlement context*).
9. ***Outranking slots*, addition** (L1B-4). **Home: rulebook.** A way that needs someone who outranks a party (a petition, a patron) is filtered out when no registry person outranks them (nobody outranks a High party but the emperor).
10. ***Ally plant on a card of the crisis's family*** (G1-6). **Home: rulebook §4 (reuse: once per family).** A cast member or a plant's knower placed on a card of the same family as their crisis doesn't break *once per family*.
11. **card-schema.md:** add the fields above (`truth_row`, `shapes`, `plain`, `reads_context`, fact `when` on context facts, story `registry_updates`); a route kind `sea` (L1B-5); bump to v1.11.
12. **Family docs** (beyond tagging shapes and confirming rows in every truth table):
    - family-reckoning: *Ruin from above* needs a standing gap **or** a bought or protected shield, so Middle against Middle works (G2-2).
    - family-restore: a Restore card may serve as a faction-story reveal only when the harm on it was done off the card (G2-3); the *obligation* cause may reuse a local creditor instead of creating one (L0B-5).
    - family-contest: the *old blood* twist hides behind another name (a mother's line or a byname), since kin share a family name (L0B-6); *The fix*: one person may be both the fixer and the other side's champion (L0B-3).
    - Make sure every family's confirming rows are a minority of its rows; where a premise has only a confirming row and one other, that's fine.

## Group 2 — main-quest.md, assembly.md, fallout.md, mechanics.md

1. ***Plant what the climax asks for*** (SC-C4, PD-C3). **Home: main-quest** (port of faction-stories' *Plant what the crisis asks for*). A heart's secret (a hidden fact on the heart) is planted on a Trials or Ordeal card, and offered on the Ordeal or the climax as a way or a twist. The Return may pay it off but never shows it first. Otherwise the heart is cast without one.
2. **Return casting** (PD-C3). **Home: main-quest, Return.** The Return reuses kin and people the story has already shown; it creates nobody whose role an established person could fill.
3. **Twists on turning cards** (SC-C3). main-quest and assembly point to rulebook *No blank card* (turning cards are never plain) and *Roll order* (twist budget to turning cards first, climax and Return first; CM2). Assembly §4's fill order stays as it is for writing cards.
4. **Small clarifications:**
   - CM1: a *Rising* antagonist's side may borrow its parent faction's realm reach (main-quest §6).
   - CM3: with W=2, assembly §7's linked-node variety rule doesn't bar the Ordeal door's family (allow Reckoning).
   - CM5: the landmark standing rule (main-quest §4) is about the **settlement**, not the site.
   - CM6: the Return node takes no minors (assembly §8).
   - CM7: fallout's allowed Restore causes go into the rule text, not a sub-list (fallout §3 or §4).
   - PD-C4: **minors sit near their node** (within about 2 days' travel of it); a main layer's first card is offered within about 3 days' travel of where the layer before ended, unless the arc's journey is the point (Ripple). **Home: main-quest, Spread and presence.**
5. **mechanics.md:**
   - PD-C5: **a clock's world ending writes a state fact** on its place or person, with `when` = its checkpoint; cards at that place read it (locations, *Clocks*). One line pointing there.
   - PD-C5: **sibling chapters taken late:** an unchosen sibling chapter played after a later layer has opened reads the moves since through its settlement context (its `reads_context` faces), so it never reads stale. Point to locations and to Lane B's text templates.
   - Saving (§13) includes `registry_updates` already merged into entries (nothing new to save).

## Group 3 — faction-stories.md, guild-kits.md, factions.md, faction-places.md

1. ***One truth for a shared move*** (CO-C2). **Home: faction-stories §10.** The paired truce killing has one set of facts: who killed whom, where, when, who called the truce. **The planner fixes them**, and both stories' cards cite the same fact ids. The crisis that takes Reckoning *Blood debt* holds the truth row; the other crisis's truth is about its own aim (its people, the reprisal), never a second answer to who killed whom. A shared move counts **once** for levers, kills and fallout (KC2, GP4).
2. ***Guild moves reach the towns*** (CO-C1, PD-C2). **Home: locations, *Settlement context*;** faction-stories §12 adds the step: when the planner writes a story's moves and phase-3 public events, it adds each one's public fact to the `context` of every settlement it reaches (with `when`). A story that changes the state of a **site** another story uses (a ring's licence revoked, a yard occupied) writes that as a state fact on the site; the planner checks every other story's card at that site against it (locations has the rule).
3. ***Every region pulls*** (playthrough region note). **Home: faction-stories §8b.1.** Counting the main story's cards already pinned, **every region has at least one story card** (main or guild). The placement plan puts a routine card or a phase-3 card in any region that has none. Remote regions qualify like any other.
4. **Ordeal events across stories** (SC-C7): §9's "no chapter-2 event used by more than two stories" now counts the main Ordeal's event too (rulebook, *World ledger*); *the cost* as a reveal at most twice, worded differently.
5. **Trade pairs** (§8b.3): a trade pair meets when a shared town allows it; still optional.
6. **Cast after placement** (GP1): §12's order becomes: kits for all → placement plan → cast for all (casting scores each slot against its card's planned region) → cards per story. Casting says the same (Group 4).
7. **Small clarifications:**
   - KC1: "never twice in the same phase position" covers every kit pool: odd jobs, reveal, chapter openings and turns, crisis.
   - KC4: one enemy side per faction across all guild stories is intended (two stories can't both have Free Lords as enemy side). State it.
   - KC5: the lever caps are checked when moves are written, not at kit rolls; kits carry no levers.
   - KC6: in paired mode the pair's two reveals count as **one** *hidden hand* for the §9 cap.
   - GP3: word the paired meeting exception exactly: each enemy's appearance in the rival's story is that pair's meeting; its roles there follow §10, not §8b.3's rival roles.
   - GP5: a field site is "beside" a shared town when it's within half a day's travel and has the town as its `at`; it counts toward the town's caps and is a field site for pinning.
   - GP6: define the epilogue trigger *seized* (the stake passes to the enemy or a third party at the crisis).
   - GP7: guild-kits, Lantern C1: the named chapter happens inside the march its writ names (fix the kit text).
   - G1-4: in paired mode each story meets **its own** dead before the truce (not both).
   - G1-5: the fallback fills at most one slot per chapter, like any cast member (§8); the planner checks it.
   - G2-1: Dueling kit: point to rulebook *Endings don't remove people a later card needs* (bout stakes and plant carriers).
   - L2B-3: a fixed place that gives faction jobs (the Soft Hollow, the Green Wall) follows the per-presence signature quota (faction-places §7 points to locations §4).
   - Lantern–Old Hands and any other "none" ties are unchanged.

## Group 4 — locations.md, casting.md, worldgen-requirements.md

1. ***Settlement context*, completed** (CO-C1, CO-C3, PD-C2, L0B-4, L1B-5). **Home: locations.**
   - **Who writes it:** every pass, as it makes its cards: main moves and their fallout (every town a fallout card is pinned in counts as touched), **guild moves and phase-3 public events** (the guild planner), clock endings (when they fire, as a state fact with `when`), and court-state facts about a town. Each context fact carries `when`.
   - **Sites have state too:** a public change to a site (a licence revoked, a yard occupied, a shrine closed) is a state fact on the site with `when`. Every card at that site whose time can fall after `when` reads it (`reads_context`, with a `before` face); a card that can't hold under either order is pinned at another site.
   - **Time order:** a card that names a context fact keeps a plain face (`before`) unless the fact is true before the card can be offered. Context still never gates a card.
   - **Location dependencies** (L3B-3): a location card may `depends_on` a same-region public plant only when that plant's card is offered before it (time order), or the plant was made at or before the main story's Hook.
2. ***Per-place caps*, additions** (L0B story, L1B story, PD-C1). **Home: locations.**
   - The theme cap counts **the harm's shape** (water spoiled from upstream, a bystander blamed) and **the place's trade** (the Mount's protection tithe), not only the theme word.
   - **No two cards at one settlement roll the same set of way kinds** for the same family.
   - **Leanings move on** (L0B-1): when a place's leaning points only at premises at their world cap, the leaning moves to the place's next feature.
   - **Signature quota** (L1B-3, L2B-3): a presence's first card is free; the "one in three" signature rule applies from its third card (so a presence with 1–2 cards needn't make either a signature job). Fixed places that give faction jobs follow it too.
3. ***Clocks*, addition** (PD-C5). **Home: locations §6.** The rolled `world_ending` writes a state fact on its place or person when its checkpoint passes; every card at that place reads it (a `before` face) or is withdrawn. A clocked card's ending never contradicts its own text.
4. **casting.md:**
   - **A generated filler replaces the canon person** (CO-C4). **Home: casting §4.** When an archetype rolls a generated filler, the canon person doesn't exist in that world: faction `leaders`, seats, links and foils all point at the filler. (The test merge tool had a bug that kept both; it is being fixed.)
   - **Add-only updates** (GP2). **Home: casting §1.** A lower layer may add to a higher layer's entry: facts, links, `dies_at`, and `state` facts its own endings write, recorded as `registry_updates`. It never changes standing, office, home, faction or an existing fact.
   - **Cast after placement** (GP1): guild cast slots are scored against the card's planned region (faction-stories §12).
   - **Archetype data** (L0-2): every archetype row lists its `keeps` explicitly; nothing is read from leaders.md at run time.
   - **Kin of generated fillers** (L0-3): a filler of canon kin keeps the canon relation and gets a shared family name; canon names stay exempt.
   - **Guild contacts** (L0-5): in-town hall and contact keepers are on-demand class members, not pre-made offices.
   - **Tensions** (L0-6): "one tension per major faction" counts a tension against every major faction whose land or people it involves.
   - **The reuse aim is world-wide** (L3B-4, L1B-6): 25–45% across the world; a batch in a remote region may sit below it.
   - **Return casting:** point to main-quest's Return rule (reuse established people).
5. **worldgen-requirements.md:** note that the world ledger's caps need the planned world size N before generation (rulebook, *World ledger*).

---

## Not changed (watch in World D)
- GP8 (layer 2 made no High people, so Above enemies were all created): may be working as designed.
- L1B-6 (the 3-card reuse cap for ordinary townsfolk) and the `create_score` of 1.5: watch World D's reuse.
- 33 shared given names across regions: legal under the regional rule.

## For Lane B (CR-A2, written in the plan doc)
- `reads_context` faces: card-text templates need a `before` and an after reading where a card names a context fact.
- Sibling chapters taken late read the moves since (mechanics).
- The id-in-text lint (rulebook, *Card text only says what's true then*) belongs in `text_check.py` too.
- Filled ways name a place feature, thing or person (rulebook rule 3, wording); the fill grammar does it.
