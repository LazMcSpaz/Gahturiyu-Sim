# Merge log

What has been merged into `main`, when, and how it was checked. Newest consolidation first.
Add a row whenever a branch is merged.

## How branches are merged (Laz's ruling, 10 Oct 2026)

- Every branch is merged into `main` in the way that is least destructive and easiest to track.
- No rebase, no squash, no cherry-pick, no force-push, no branch deletion.
- Each feature branch comes in as one real merge commit (`git merge --no-ff`), so
  `git log --first-parent main` reads as one line per branch, and any merge can be undone with
  `git revert -m 1 <merge commit>`.
- Conflicts are settled on the feature branch, by its own agent, by merging `main` into it.
- One merge, one full `cargo test --release`, one push.

## After the consolidation

| Date | Branch | How | Commit on `main` | Checked | Left open |
|---|---|---|---|---|---|
| 10 Oct 2026 | `claude/language-dialogue` (tip `756d071`) | Merge commit. Branched from `main` at `b0d2da9`; merged cleanly onto `be5862c`. No `save::FORMAT` or `Cargo.toml` change. | `b153782` | Full suite on the merge result: 52 groups, 424 passed, 0 failed, 1 ignored (cloud container, Linux). | Not yet seen in the window: the teal words, the hover meanings and the Names setting were checked by its own agent only. |
| 10 Oct 2026 | `agent2/trade-squad` (tip `2f1c25e`) | Merge commit. Branched from `main` at `6dd0da1`, which had not moved. Playtest items 6, 10 and 11. `save::FORMAT` goes from 43 to 44. | `0a094ce` | Full suite on the branch tip, whose files are identical to the merge result: 53 groups, 432 passed, 0 failed, 1 ignored (cloud container, Linux). The sell list, the walk-over give and the stray flags were also tried by hand in the text play tool. | Not yet seen in the window. Purchases still land in the pack of whoever is talking. |
| 10 Oct 2026 | `agent2/trade-squad` (follow-up, tip `53576e1`) | Merge commit. Shorter trade labels, a wider topic column while trading, and the rest of the sell list a page at a time. No further save change. | `291f628` | Full suite on the branch tip (same code as the merge result): 53 groups, 432 passed, 0 failed, 1 ignored. The first page of the trade panel was checked in a headless screenshot of the window, before and after. | The later pages of the sell list have not been seen in the window. |
| 10 Oct 2026 | `claude/ruins-gangs` (tip `a0fa75b`, the commit marked ready) | Merge commit. Its agent had merged `main` (`de39a3c`) in first, so the merge result is identical to that commit. Playtest items 8, 2 and 13; item 12 cut back to Laz's direction (a band in sight is called bandits; no worked-out fight odds). `save::FORMAT` goes from 44 to 45. | `4773b70` | Full suite on the merge result: 53 groups, 437 passed, 0 failed, 1 ignored (cloud container, Linux). | Not seen in the window by the merger. The branch has since moved on (`50f7dc8`); that is its next chunk, not part of this merge. |
| 10 Oct 2026 | `claude/small-fixes` (tip `97bf225`) | Merge commit. Its agent had merged `main` (`2ae3d82`) in. Rest and wake are two orders; coin can't be equipped; the roadside advice. No save change. | `37c496e` | Full suite on the merge result: 54 groups, 439 passed, 0 failed, 1 ignored (cloud container, Linux). | Not seen in the window by the merger. |
| 10 Oct 2026 | `claude/town-work` (tip `581d071`) | Merge commit. Its agent had merged an earlier `main` (`de39a3c`) in; it merged cleanly onto today's. Playtest items 9 and 7. No save change. | `5ec8687` | Full suite on the merge result: 55 groups, 443 passed, 0 failed, 1 ignored. | Not seen in the window by the merger. |
| 10 Oct 2026 | `agent2/crafting` (tip `6d235cf`) | Merge commit. Branched from `main` at `2ae3d82`. A workshop's own forge counts as a forge; buildings named by their real style; a plain list of things to make; town news travels by road and is no longer announced from afar. Two deeds added at the end of the list; no `FORMAT` change. | `cc763dd` | Full suite on the merge result: 56 groups, 449 passed, 0 failed, 1 ignored. Forging a knife in a real Maker's forge was tried by hand in the text tool. | The window's craft panel change was not seen in a screenshot. |
| 10 Oct 2026 | `claude/town-care` (tip `340280a`) | Merge commit. Its agent had merged an earlier `main` (`2ae3d82`) in; it merged cleanly onto `e300424`. Paying a healer to mend the squad or an innkeeper for beds; the sea says why it stops the squad; stilt villages named on the map. `save::FORMAT` goes from 45 to 46 (beds rented at inns). | `ac419e3` | Full suite on the merge result: 57 groups, 451 passed, 0 failed, 1 ignored (cloud container, Linux). | Not seen in the window or played by the merger. The hover line "Would join the squad…" may want moving into `stranger_lines` (its agent's note). |
| 10 Oct 2026 | `claude/sound` (tip `005a901`) | Merge commit. Its agent had merged `main` (`fbc0b4e`) in, so the result is identical to that commit. Sound on: Bevy's `audio` feature in `Cargo.toml` (approved by Laz, R3), 45 placeholder sounds, the first hooks. Nothing in `src/sim` changes. | `01d3482` | Not run on its own by the merger: its agent's run on the same files was 451 passed, 0 failed. Run by the merger together with the next row. | The first build after it recompiles Bevy (about 25 minutes on the cloud container). Not heard by the merger (no sound card there). |
| 10 Oct 2026 | `fix/naming-1` (tip `d46a412`) | Merge commit, onto the sound merge. Fix-list items #1, 12, 17, 21, 24, 25, 26, 35, 36, 39, 41 to 44. Brings in `hunt/naming` and `hunt/buildings` (tests, the hunt's notes, the text tool's `build` and `base` commands). No save change. | `df1f9a2` | Full suite on the result of both merges: 60 groups, 460 passed, 0 failed, 11 ignored (cloud container, Linux). | Not seen in the window. |
| 10 Oct 2026 | `fix/buildings-1` (tip `661b2d1`) | Merge commit. Fix-list items #2, 3, 13, 19, 20, 27, 29, 38. One clash, in the shared text tool (`src/bin/play.rs`, the squad line): `fix/naming-1` clamped the load at 0 and this branch added the carried squadmate after it. Both kept; settled in the merge commit because it is two adjacent lines. No save change. | `2467995` | Run together with the next row. | Not seen in the window. |
| 10 Oct 2026 | `fix/naming-2` (tip `a935500`) | Merge commit, onto the buildings merge. Fix-list items #9, 10, 11, 71 to 75 (base building). No save change. | `6a186a2` | Full suite on the result of both merges: 61 groups, 480 passed, 0 failed, 3 ignored (cloud container, Linux). | Not seen in the window. |
| 10 Oct 2026 | `fix/ruins-gangs-1` (tip `88c010c`) | Merge commit. Fix-list items #4, 5, 6, 7, 8, 15, 16, 18, 22, 28, 30, 37, 45, 46. No clashes. No save change. | `8e84d6d` | Run together with the next row. | Not seen in the window. |
| 10 Oct 2026 | `fix/naming-3` (tip `7004eb3`) | Merge commit. The branch was built on the ruins-gangs merge, so the result is identical to its tip. Fix-list items #14, 23, 31, 32, 33, 47, 68, 69, 70, 93, 129 (animals). No save change. | `e58b8b8` | Full suite on the same files (the branch's tip): 62 groups, 494 passed, 0 failed, 3 ignored (cloud container, Linux). | Not seen in the window. |
| 10 Oct 2026 | `fix/naming-4` (tip `d2a253d`) | Merge commit. Fix-list items #51, 53, 76, 78, 79, 80, 81, 88, 91, 97, 98, 113, 114, 119, 120, 132 (the law and the text tool). Save FORMAT 46 to 47 (`World::hot`). | `ff04978` | Run together with the three rows below. The branch's own run before `main` was merged in: 500 passed, 0 failed. | Not seen in the window. |
| 10 Oct 2026 | `fix/buildings-2` (tip `c79e183`) | Merge commit. Fix-list items #48, 54, 55, 57, 58, 85. No clashes. No save change. | `7f208a8` | Run together with the rows above and below. | Not seen in the window. |
| 10 Oct 2026 | `fix/ruins-gangs-2` (tip `47103b6`) | Merge commit. Fix-list items #49, 56, 59, 60, 61, 62, 63, 64, 66, 67. Save FORMAT 47 to 48 (`World::getting_up`). Two clashes with `fix/naming-4`, settled in the merge commit because each is a line or two: the FORMAT line (48 kept, as agreed on the shared page) and the two new saved fields side by side in `src/sim/world.rs` (both kept, `hot` first). | `bc57258` | Run together with the row below. | Not seen in the window. |
| 10 Oct 2026 | `fix/naming-5` (tip `1fe400e`) | Merge commit. The branch was built on the three merges above, so the result is identical to its tip. Fix-list items #82, 100, 101, 104, 121, 131 (talk and the news; shunning shown). Save FORMAT 48 to 49 (`World::met`). The same two small clashes with `fix/ruins-gangs-2`, settled on the branch. | `1a2eb4f` | Full suite on the same files (the branch's tip): 64 groups, 527 passed, 0 failed, 3 ignored (cloud container, Linux). | Not seen in the window. |
| 10 Oct 2026 | `fix/buildings-3` (tip `4932fec`) | Merge commit. Fix-list items #83, 84, 86, 89, 90, 94, 96, 99, 102, 111, 115, 116 (part), 118. One clash, in `src/sim/chances.rs` (the line said when a town job pays): `fix/naming-5` had shortened the old line and this branch rewrote the pay by hours with its own line; this branch's version kept whole, settled in the merge commit because it is three lines. No save change. | `e1bc84b` | Run together with the row below. | Not seen in the window. |
| 10 Oct 2026 | `fix/naming-6` (tip `5c23a3a`) | Merge commit. The branch was built on the merge above, so the result is identical to its tip. Fix-list items #34, 103, 106, 108, 147, 157, and the unconfirmed U-4 and U-12. Save FORMAT 49 to 50 (`World::unwelcome`). | `9512ffc` | Full suite on the same files (the branch's tip): 65 groups, 546 passed, 0 failed, 3 ignored (cloud container, Linux). | Two screenshots by the merger (software drawing): `GAHT_TALK=1` at dawn opens a talk; the hover over a penned Turiyu, plain and with `GAHT_DEBUG=1`. |

## Consolidation of 10 Oct 2026

Before this, everyone worked on `claude/repo-setup-x72er0` and called it "main". The branch named
`main` had been left at `ce48d51` (17 Sep), 140 commits behind with nothing of its own.

### Branch tips before the consolidation

These were tagged locally as `pre-merge/2026-10-10/<branch>`; the tags could not be pushed from the
session that did the consolidation (the push was refused), so the hashes are recorded here instead.

| Branch | Tip before |
|---|---|
| `main` | `ce48d51` |
| `claude/repo-setup-x72er0` | `edde92d` |
| `agent2/naming` | `e811329` |
| `agent2/weather` | `bfc4a35` |
| `agent2/animals` | `411b1a4` |
| `claude/voice-tool` | `7c0ce72` |
| `claude/mvp-buildings` | `f571e36` |

### What went in

| Step | Branch | How | Commit on `main` | Checked | Left open |
|---|---|---|---|---|---|
| 1 | `claude/repo-setup-x72er0` | Fast-forward: `main` moved from `ce48d51` to `edde92d`. Every commit keeps its hash. | `edde92d` | Full suite on `edde92d`: 51 groups, 403 passed, 0 failed, 1 ignored (cloud container, Linux, rustc 1.97.0). | The branch is frozen. Nothing more should be pushed to it. |
| — | `agent2/animals` | Already in through step 1. | `a95506d` (9 Oct) | Part of step 1's run. | — |
| — | `agent2/weather` | Already in through step 1. | `72f790e` (9 Oct) | Part of step 1's run. | — |
| — | `agent2/naming` | Already in through step 1. | `1c234c7` (10 Oct) | Part of step 1's run, including the 13 native names for the animal items. | — |
| 2 | `claude/voice-tool` | Merge commit, pull request #1. Adds `tools/voice/` and a section in `CLAUDE.md`. No Rust source changes. | `e13461b` | Full suite on the merge result: 51 groups, 403 passed, 0 failed, 1 ignored (cloud container, Linux). | — |
| 3 | `claude/mvp-buildings` | Merge commit, pull request #2. Its agent first merged `main` into the branch and settled the conflicts there (`5ce4b5c`); the merge result is identical to that commit's files. `save::FORMAT` goes from 42 to 43. | `1698c7b` | Full suite on the merge result: 52 groups, 423 passed, 0 failed, 1 ignored (cloud container, Linux). | — |
| 4 | `claude/voice-tool` (follow-up) | Merge commit for one more commit pushed after step 2 (tip `a486d73`). Four files under `tools/voice/` only. | `3b84c34` | Suite not re-run: the only files changed are under `tools/voice/`, which no test or Rust source reads (checked by diff and search). Step 3's run stands. | — |

### Result

Every branch that existed on 10 Oct 2026 is in `main` as of step 4. Branches for new work start from there.

### Still to do

- The `pre-merge/2026-10-10/...` tags are now on GitHub (seen there on 10 Oct; not pushed by the session that did the consolidation). The result itself is not yet tagged `consolidated-2026-10-10`.
- Done: GitHub's default branch is `main` (Laz switched it, 10 Oct).
- Not yet run on Windows. Every check above was on Linux.
- The old branches are left in place. Deleting them is Laz's call.
