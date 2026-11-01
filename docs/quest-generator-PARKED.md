# Quest generator: parked (2026-11-01)

**Status:** parked by Laz to narrow focus and ship sooner. It is not abandoned. Nothing on this branch is for merging.

## What it is
A design for generating a whole world's quests from its people and factions:
- a main story;
- six guild stories (the five guilds and the Holy House);
- side quests, including guild hall jobs.

Every quest is a *card*: one of nine families (Rescue, Investigate, Dispute, Threat, Passage, Acquire, Reckoning, Restore, Contest), with a premise, a hidden truth, twists, ways through and endings. Cards are cast from a shared registry of people, places and things, pinned to the map, and kept varied by a world ledger.

The design was tested **on paper**: agents stood in for the generator code, independent reviewers scored the worlds they produced, and Python checkers measured the results. No game code was written. A draft of the Rust types is parked separately on the `parked/questgen-types` branch.

## Where it stood
- **Rules:** v7 (rulebook, card schema v1.12, casting v1.3, family docs, faction stories with the guild ladder and the chapter-2 blows, locations, mechanics). Copies are in `docs/quest-generator-archive/rules/`; the claude.ai Project holds the same docs as `claude/quest-*.md`.
- **Guild stories had moved to *authored spines*:** hand-written plot outlines with slots the world fills. Assembling stories from random parts produced legal but senseless plots, and two review rounds confirmed that the method was the problem.
- **The Writers were done first:** 16 spines, proofread in every combination. The last scores (two blind reviewers, averaged) were coherence 3.6, creativity 4.3, emotional pull 3.95 and variety 3.5, against a target of 4+. See `docs/quest-generator-archive/writers-status.md` and `world-D/writers/`.
- **Test World D:** layers 0–2 built (leaders, offices, a 48-card main story). The guild and side-quest layers were never built.
- **Craft research:** *Procedural Storytelling in Game Design* plus two web sources, turned into rule proposals. Most of the in-scope ones were applied in v7; see `craft-proposals.md`.

## Why it was parked
- **It is large,** and the remaining coherence work is slow: each review round surfaced new seams.
- **Laz chose a smaller target he can finish:** the **Understory** dungeon prototype, where a place's generated history shapes its layout and challenges.
- **Much of this work carries over to that smaller project:** authored spines, combination proofreading, clues before twists, who pays, blind review.

## If it is resumed
1. Read `writers-status.md` (what's left for the Writers) and `rules/rulebook.md` (the doc map).
2. Finish the Writers to 4+, then carry the spine method to the other guilds.
3. Rebuild World D's guild and location layers, then run its blind review.

## What's in the archive
- **`rules/`:** every rule doc at v7.
- **`test-tools/`:** the checkers (`world_check.py`, `merge.py`, `ledger.py`), the guild kit roller, the World D contract, and the fix lists.
- **`world-D/`:** layers 0–2, the guild narrative samples and reviews, and the Writers spines, roller, proofs and reviews.
- **`craft-proposals.md`:** the craft research, turned into rule proposals.
