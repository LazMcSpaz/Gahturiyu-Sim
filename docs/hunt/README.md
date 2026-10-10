# The bug hunt: the naming session's findings (towns, law and society)

- **`bugs-naming.md`**: the list, NM-1 to NM-79, in the format of the plan (`claude/bug-hunt.md`).
  The same text is the Project doc `claude/bugs-naming.md`. Start here.
- `play-th.md`, `play-rk.md`, `play-tr.md`, `play-id.md`: four play-throughs in the text tool (thief,
  reckless, careful trader, idler) on worlds 3, 21 and 34: every finding with the commands typed and
  the tool's exact output.
- `code-*.md`: three code-reading passes made before playing. Guesses made without running
  anything; `bugs-naming.md` says which were then seen, which were not, and which are still open.
- `brief-play.md`, `brief-base.md`: what the play-testers were asked to do. `brief-base.md` is for
  base building, which has **not** been played yet.

Also on this branch:
- `tests/hunt_base.rs`: nine tests of base building. Eight fail today and are marked
  `#[ignore = "NM-…"]`: `cargo test --release --test hunt_base -- --ignored`.
- `src/bin/play.rs`: new `build` and `base` commands, so base building can be reached from the text
  tool (`play <save> help`). `build kit` is a testers' cheat, not part of the game.

The saved moments the play diaries name (`th21-free-1957` and so on) are not in the repo: there are
about 660 of them, 2 to 3 MB each, in save format 45 (`main` at `e300424`). The scenario written
beside each finding is enough to reach it again from a new game.
