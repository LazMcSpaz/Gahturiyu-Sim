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

- Tag the result `consolidated-2026-10-10` and push it with the `pre-merge/...` tags (needs a session that is allowed to push tags).
- GitHub's default branch is still `claude/repo-setup-x72er0`. Laz has said to switch it to `main`; the session that did the consolidation was not allowed to change repository settings.
- Not yet run on Windows. Every check above was on Linux.
- The old branches are left in place. Deleting them is Laz's call.
