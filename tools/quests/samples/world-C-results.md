# Quest generator: World C test results (2026-10-27)

World C is the first full world built under rules v5.2, card schema v1.10 and casting v1.1, after Laz froze the scope.
- **Layers built in order:**
  - layers 0–1: leaders, seat holders, court tensions;
  - layer 2: the main story;
  - layer 3: one placement plan for all five guild stories, then their cards;
  - layer 4: four location batches.
- **Size:** 152 cards across all ten regions, on the updated test island (Lanternhall is inside Carter's Rest, guild halls are in every large town, and the Old Hands' reach is wider).

## Scores, against World B

| Reader | World B | World C |
|---|---|---|
| Story critic | depth 4 · units 4 · whole 4 | **depth 5** · units 4 · whole 4 (memorable people 5) |
| Main story (critic) | 3 | **4** |
| Playthrough | 6.5 / 10 (main ~5) | **7.5 / 10** (main 7.5, guilds 8, side 6.5) |
| Coherence | 3 / 5 | **3.5 / 5** |
| Script check | 26 issues | **0 issues** |
| Secrets | clean | clean |

- **Families:** evenly spread, 13–19 cards each, and no repetition notes.
- **Casting reuse:** 27%, inside the 25–45% aim.
- **Voices:** every person on a story card has a people, an age and a voice.
- **Guild stories meet (§8b):** every number passes.
  - Each story touches 3–4 regions.
  - The busiest region holds 28% of guild cards.
  - Every guild takes part in a meeting.
  - The paired syndicates' truce reads "superbly"; the other meetings only fill a slot.

## Fixed since World B
- **Heart and villains on stage:** Feyesh is on the very first card, and Orrin Vane is met three times before turning at the climax.
- **Wins stick;** plants pay off.
- **Main chapters chain and climb.**
- **The captor can be right,** and the event ceiling holds.
- **No second lord** of one keep, and nobody in two offices.
- **Timelines add up;** geography text matches the map; no "only X" duplicates.

## Still wrong, and the rule causes
1. **The ledger counts words, not shapes.**
   - Seven "water spoiled uphill" quests and seven "bystander blamed" truths slip through under different keys.
   - Small families' truth rows hit the cap of 3 and close whole families: by layer 4, Acquire and Passage were blocked.
   - The cap must scale with world size and count a shape once.
2. **Faction-story events don't reach towns or other stories.** Settlement `context` holds main-story facts only, so shared sites contradict (the Ring's licence is revoked, then the climax bout is fought there).
3. **One shared truce killing, two different answers.** The two syndicate crises disagree on who killed whom.
4. **Turn cards and crises without twists:** the main storyline had 26 of 40 with twist *none*. "As it looks" truths read as blank. Yet side quests also need some plain ones for contrast.
5. **The main story's heart secret first appears in the Return,** and the same ordeal (the heart disowned) repeats across five stories.
6. **Leaders appear twice.** When an archetype gets a generated filler, the canon person stayed listed. That is a rule gap (casting §4 must say the filler *replaces* them) plus a test-tool bug.
7. **Smaller:**
   - context read before the move happens;
   - fact ids shown in card text;
   - a clock ending that contradicts its own card;
   - a remote region (the Rim, the Snows) with content but nothing pulling the player there;
   - a 52-day gap before the climax.

## Process findings
- **Paper tests with parallel writers can't keep one ledger.** Every parallel batch overran a cap it couldn't see. A real generator runs these steps in sequence, so this one is the test's problem, not the design's.
- **Lower layers need "add-only" updates to higher entries** (a `dies_at`, a new link). Casting §1 should allow these.

Every finding is in `findings-ledger.md` (World C section), with doc and section.
