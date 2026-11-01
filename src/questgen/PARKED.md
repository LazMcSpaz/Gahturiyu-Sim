# Parked: quest generator data types (draft)

**Status: parked on 2026-11-01. Do not merge.** This branch exists only to keep the draft safe.

## What this is
A first draft of the quest generator's Rust data types (`data.rs`: registry entries, cards, stories), plus a small test file. It was written on 26 Oct 2026, then set aside the same evening when Laz confirmed the generator was still in its design and research phase: no Rust until the rules were settled.

## Why it's parked
On 1 Nov 2026 Laz narrowed focus to ship something sooner: a **dungeon generator in which a small story generator shapes each map** (the "Understory" prototype). The full quest generator (main story, guild stories, side quests, casting, the world ledger) is parked as a whole, not abandoned.

- These types match card schema v1.10. The rules moved on afterwards (v1.12 and the authored guild spines), so they're **out of date**.
- Where the design stood when parked, and what to read first if it's ever resumed: `docs/quest-generator-PARKED.md` on the `docs/quest-generator` branch.

The branch hasn't been built or tested since it was parked, so check it compiles before reusing any of it.
