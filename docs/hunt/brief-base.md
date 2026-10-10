# Bug hunt brief: base building (Gahturiyu Sim)

Read `/home/claude/hunt/BRIEF.md` first: the rules, what counts as a bug, and the report format are the same.
Three things differ for this part:

1. **Use the newer tool:** `/home/claude/hunt/g2 <save-name> <command...>` (not `g`). Saves go in
   `/home/claude/hunt/saves46/`. Old saves from `g` do not load in it. When you copy a save, copy its
   `.session` file too (it holds who is selected and which news has been shown).
2. **New commands** (see `g2 <save> help`):
   - `build` lists what can be built, what each needs and who in the squad can build it.
   - `build camp` founds a base (a camp marker) where the first selected member stands.
   - `build KEY [DIR M] [DEG]` lays a building site DIR M metres from that member (e.g. `build hut n 8`,
     `build field_plot e 15 90`).
   - `build wall KEY DIR M [DIR M ..]` lays a wall from that member's spot along the legs
     (e.g. `build wall palisade e 12 n 12`). `gate` and `watchtower` snap onto a wall.
   - `base` shows every base: buildings and sites, the store, who lives there, and its log.
   - `base store` puts building materials from the packs of members at the base into its store.
   - `base leave NAME` leaves a squad member to live at the base; `base fetch NAME` brings them back
     (for a hired hand it lets them go).
   - `base job NAME JOB` (idle, builder, farmer, cook, crafter, hauler, guard); `base recipe NAME [N]`.
   - `base seal ID`, `base down ID` (take a building down).
   - Hiring a townsperson: `talk` to them; if they are willing and you have a base, a topic
     "Come and work at my outpost" appears.
   - `build kit` is a **testers' cheat, not part of the game**: it teaches the whole squad carpentry
     and masonry and gives the first selected member 80 timber, 50 rock, 30 seareed, 12 clay, 3 iron
     ingots, 300 coin, 20 flatbread and 10 grain. Use it so you don't spend the session gathering.
     Anything that only happens *because of* the kit's own oddities (one person carrying far too much)
     is not a finding.
   These commands were written this afternoon. If one of them itself misbehaves (prints nonsense,
   crashes), report it under "Part: the new build commands" and say whether you think the fault is the
   command's or the game's.
3. **How the game is meant to work here** (so you can tell a bug from a rule):
   - A base is founded with a camp marker; buildings can be placed within the base's reach, which
     grows with buildings standing. Sites need their materials delivered (from the store) and
     builder-hours from members standing at the base who know the craft (or residents set to Builder).
   - Members left at a base, and hired hands, do a job in rounds: a farmer harvests a field once a day,
     a cook bakes grain and timber into flatbread at a kitchen, a hauler brings timber and rock, a
     crafter makes a recipe at a work shed, a builder builds and mends.
   - Hired hands are paid at each dawn (06:00) and want food and a bed; unhappy ones quit.
   - Thatch rots unless sealed with pitch.
   - CLAUDE.md rules that apply: nothing may depend on how time is stepped (`wait 60` against sixty
     `wait 1`), on whether a save was loaded, or on how near the squad is.

## Already known in base building (shown by tests; one line "also seen in play" is welcome, no more)

1. Hiring a first hand days after founding a base makes buildings under way finish at once, dated in the past.
2. A wall can't turn a corner (the piece after the bend is refused without a word).
3. A gate doesn't fit a 10 m wall ("must sit on a wall").
4. Someone left at a base doesn't eat the bread in their own pack, only from the store.
5. A farmer, a cook and a hauler starve beside a full granary within a fortnight.
6. Letting a hired hand go just before dawn makes the day's work free.
7. Changing a resident's job to "cook" and away again uses up grain and timber.
8. The travelling squad can go past ten by fetching people from a base.

## Candidates from reading the code, not yet seen (trying these is valuable)

Full write-ups: `/home/claude/hunt/reports/code-society-base.md`, items 2, 5, 6, 10, 11, 12, 13, 14 and
the "Smaller things" list. In short:
- the store is one-way: only building materials go in and nothing comes out; hired hands are
  "not fed" each dawn and quit; "seal" can never work (no pitch can be stored); what a base crafter
  makes can't be collected;
- placement checks only town buildings: can a hut go on a town's field, market, road, a bandit camp,
  a ruin, a woodlot? Can a camp marker go in the middle of a town?
- residents and hands work round the clock whatever their state (starving, down, asleep);
- an office holder (an official, an elder) or a guard can be hired and keeps the post;
- every base event goes to the squad's news wherever the squad is (too much telling);
- leaving the last living member at a base strands everyone; a base founded between 00:00 and 06:00
  skips that day's dawn tally; a hand docked for "no bed" while a bed stands empty.
