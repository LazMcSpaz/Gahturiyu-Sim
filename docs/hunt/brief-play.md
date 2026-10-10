# Bug hunt brief: towns, law and society (Gahturiyu Sim)

You are a play-tester hunting bugs in a game by playing it through a text tool. **Find, don't fix.**
Do not edit any source file, do not run cargo, do not use git, do not write to the claude.ai Project.
Your only outputs are your report file and your save files.

## The game and the tool

- A world sim (towns, townsfolk with day plans, law, crime, jobs, recruiting) played with a squad of four.
- Play with: `/home/claude/hunt/g <save-name> <command...>` — one command per call. It loads the save, runs the
  command, saves again. Saves live in `/home/claude/hunt/saves/<save-name>.save`.
  - New game: `/home/claude/hunt/g <save-name> new <seed>`
  - `/home/claude/hunt/g <save-name> help` lists every command. Ids come from `look` (`look places`, `look people`).
  - `wait M` lets M minutes pass. `town`, `journal`, `map`, `pack NAME` are panels.
- **Keep copies of saves at interesting moments**: `cp saves/x.save saves/x-<moment>.save`. A copied save lets
  a bug be shown again, and lets you try two different continuations from the same moment.
- Use only save names that begin with your own prefix (given in your assignment), so testers don't collide.
- The machine has 2 cores and other work running. Don't loop hundreds of commands in a shell script without
  looking at the output; a few dozen at a time is fine.

## What counts as a bug (most serious first)

- **crash**: the tool panics or stops.
- **stuck**: the player can't go on without reloading.
- **wrong outcome**: a rule does something it shouldn't (money from nothing, a crime nobody could see being seen,
  an obvious crime never seen, a result that differs when the same save is continued twice the same way).
- **misleading**: the game says something that isn't true.
- **too much telling**: the designer's standing direction is "a game, not a simulator: less telling, more finding
  out". Show only what someone in the world could see or know. No worked-out odds, hidden timers or exact numbers
  about strangers; no pushed news lines the player wouldn't miss. (Theft odds on containers are allowed: he likes
  those.)
- **rough**: works but awkward. **cosmetic**.

## Already known: don't spend time re-finding these (add "also seen" in one line if you trip on one)

1. `talk/loot/attack/carry p999999` crash the tool (bad person id).
2. A squad that starves lies down forever, no rescue, no game over.
3. By day, stealing inside an occupied home shows "0% chance of being seen" and is never seen; at night with
   sleepers it is ~26%.
4. A member shown "asleep" can still search, take, and pick locks.
5. The squad list's "[N m from the others]" flag is wrong when one member is far off (everyone gets flagged).
6. `search` on a locked chest prints "They couldn't get to it…" when the real event was something else;
   alerts print twice ("!! …" and again under News).
7. `go DIR M` for one selected member is measured from the squad's centre.
8. The HUD says "(night)" from 18:00.
9. Base-building bugs (not reachable from the text tool; ignore base building).

## Candidates from reading the code: NOT yet seen in play. Trying to reproduce these is valuable.

Each report must say for each one you tried: **seen / not seen / couldn't reach**, with the commands.
The full write-ups (with file names and the scenario to try) are in:
- `/home/claude/hunt/reports/code-buildings-crime.md`
- `/home/claude/hunt/reports/code-law-jobs-recruit.md`
- `/home/claude/hunt/reports/code-society-base.md`

Treat them as guesses made without running anything. Some will be wrong. Say so plainly when one is.

## How to report

Write your findings as you go to `/home/claude/hunt/reports/play-<your-prefix>.md` (append after each session of
play, so nothing is lost if you run out of room). For each finding:

```
### <one-line title in plain words>
- Part: Buildings and interiors | Crime and the law | Townsfolk's lives | Town jobs and recruiting | Save and load | Odd input
- How bad: crash | stuck | wrong outcome | misleading | too much telling | rough | cosmetic
- What happens: one or two sentences, what the player sees.
- What should happen: one line.
- To see it: seed, the saved copy to start from, and the exact commands.
- Output: the few lines of tool output that show it, copied exactly.
- Seen how many times: (e.g. twice, from the same saved copy; or once)
```

At the end of the file, a **Coverage** list: every bullet in your assignment, marked tried / not reached, with
one line on how you tried it. And a **Candidates** list: each code-reading candidate you tried, seen / not seen.

Rules for honesty:
- Report only what you saw the tool print. Copy output exactly; don't paraphrase numbers.
- If something only *might* be a bug, say "unsure" and why.
- If you couldn't reach something, say so. An honest "not reached" is worth more than a guess.
- Prefer a finding you can show twice from a saved copy over one seen once.

Your final message back should be short: the path of your report, the number of findings by "how bad",
the three most serious in one line each, and what you did not reach.
