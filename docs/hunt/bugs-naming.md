# Bugs found by the naming session (NM-…)

**Part hunted:** towns, law and society, including base building. **Branch:** `hunt/naming` (from `main` `e300424`).
**Format and rules:** `claude/bug-hunt.md`. **Status:** stopped on Laz's word, 10 Oct, before the last step. Four play-throughs are done on three worlds and written up here. **Base building was tried by test only**; the play-throughs of it were about to start and did not run (see section 4). So this is not yet "Hunt finished".

How to read "To see it":
- **Text tool:** `play <save> new <seed>`, then the commands given.
- **Ignored test:** on `hunt/naming`, `cargo test --release --test hunt_base -- --ignored <name>`. The test says what should happen and fails today.
- **Saved moment** (a name like `th21-free-1957`): a save kept at that instant. They are in the naming session's own work area, not in the repo (about 1 GB), and are save format 45 (`main` at `e300424`). Ask the naming session to re-run one. The scenario written beside each is enough to reach it again from a new game.
- **The four play diaries** (every command and the tool's exact output, about 2,300 lines) are in the repo on `hunt/naming` under `docs/hunt/`: `play-th.md` (thief, worlds 21 and 34), `play-rk.md` (reckless, 34 and 3), `play-tr.md` (careful trader, 21 and 3), `play-id.md` (idler, 34 and 21).

Who saw what, in "Found by":
- **[T] [R] [C] [I]**: the thief, reckless, careful-trader and idler play-throughs (run by helper agents for the naming session).
- **re-run**: the naming session ran the recipe again itself from the saved moment and saw the same thing. Entries without it rest on a play-through's own transcript.

Sections:
1. **Seen**: bugs watched happening (in the text tool, in a test, or in a long run). NM-1 to NM-19 are the first write-up; NM-20 on are from the four play-throughs, grouped by part.
2. **Read in the code, not yet seen**: what is left of the code-reading candidates.
3. **Looked for and not found.**
4. **What has been tried**, against the list in the plan.

**Where the worst of it is** (my reading, for whoever compiles):
- **The law has no teeth.** Running always beats being caught (NM-20 to NM-23), theft pays even when caught (NM-24 to NM-26), a bound member can be carried off (NM-35), shunning costs nothing (NM-39).
- **Homes leak.** By day nobody indoors sees anything (NM-11); a walk begun before lock-up ends inside (NM-42); chests are reached through walls (NM-43); nobody minds strangers asleep in their house (NM-41).
- **Distance means nothing** to talk, coin or goods (NM-54, NM-55).
- **People get stuck walking** between some neighbouring towns, and a member with a town job does it every shift (NM-47).
- **The world drifts**: skilled posts never stay filled and "No trade" people pile up (NM-51); town treasuries and grain grow without end (NM-78).
- **Base building** has eight rule bugs shown by test (NM-1 to NM-8). It has not been played yet, so expect more.
- **Too much telling**: NM-19, NM-63, NM-64, NM-65.
- **The text tool itself** (crashes, a broken save, wrong targets): NM-9, NM-68 to NM-76.

---

## 1. Seen

### NM-1 — Hiring a first hand days after founding a base finishes buildings in the past
- Part: Base building
- How bad: wrong outcome
- What happens: found a base, wait three days, lay a hut, then hire a townsperson. One second later the hut is standing, with its finishing time dated before it was even placed (placed at hour 79, "stood" at hour 44).
- What should happen: hiring changes nothing about a building under way; the hut takes its full time.
- To see it: ignored test `nm1_a_hut_placed_days_after_founding_takes_its_full_time_whoever_is_hired` (world 1).
- Why, if known: `base.rs` `next_base_event` / `base_events` / `base_changed_at`. A base's "last dawn handled" only advances while a hired hand lives there. On the first hire every missed dawn is replayed, and each one resets the site's clock back to that dawn while keeping the work already counted.
- Found by: naming session, 10 Oct

### NM-2 — A wall can't turn a corner
- Part: Base building
- How bad: wrong outcome
- What happens: a palisade laid along two 12 m legs at a right angle gets 5 of its 6 pieces; the piece just after the corner is refused as overlapping the one before it, without a word. A closed yard always has gaps.
- What should happen: both legs are laid whole.
- To see it: ignored test `nm2_a_wall_can_turn_a_corner` (world 1).
- Why, if known: `base.rs` `check_ground` / `overlap`: neighbouring wall pieces overlap at any bend, and only towers are let off.
- Found by: naming session, 10 Oct

### NM-3 — A gate doesn't fit a 10 m wall
- Part: Base building
- How bad: wrong outcome
- What happens: on a 10 m palisade a gate is refused everywhere along it ("must sit on a wall").
- What should happen: a gate goes on any wall long enough to hold it.
- To see it: ignored test `nm3_a_gate_fits_a_ten_metre_wall` (world 1).
- Why, if known: `base.rs` `check_place` wants a wall piece of at least 3.49 m; `wall_plans` cuts 10 m into three pieces of 3.33 m. Only some wall lengths work.
- Found by: naming session, 10 Oct

### NM-4 — Someone left at a base starves beside the bread in their own pack
- Part: Base building (members left at a base)
- How bad: wrong outcome
- What happens: a squad member left at a base with 13 flatbread in their pack still has all 13 three days later and is weak with hunger. They only eat from the base's store.
- What should happen: they eat what they carry.
- To see it: ignored test `nm4_someone_left_at_a_base_eats_the_bread_in_their_own_pack` (world 1).
- Why, if known: `condition.rs` `food_for` looks only in the store for a resident. Nothing moves food from a pack to the store either: `store_materials` takes building materials only.
- Found by: naming session, 10 Oct

### NM-5 — A farmer, a cook and a hauler starve beside a full granary
- Part: Base building (residents)
- How bad: wrong outcome
- What happens: an outpost with a field, a kitchen, a farmer, a cook and a hauler. After 14 days all three are at hunger 100, the store holds 76 grain and no bread.
- What should happen: the cook bakes the grain and they eat.
- To see it: ignored test `nm5_a_farmer_a_cook_and_a_hauler_keep_themselves_fed_for_a_fortnight` (world 12).
- Why, if known (a guess from reading): `baselife.rs` `start_cycle`. The hauler fills the store to the brim; the cook checks for free room before taking its grain out, so it never starts; the farmer adds grain with no room check.
- Found by: naming session, 10 Oct

### NM-6 — Letting a hired hand go before dawn makes the day's work free
- Part: Base building (hired hands, wages)
- How bad: wrong outcome
- What happens: hire someone, have them haul for 22 hours, let them go a minute before dawn: nothing is paid. They can be hired again at once.
- What should happen: work done is paid for when they leave.
- To see it: ignored test `nm6_a_day_s_work_is_paid_for_even_if_the_hand_is_let_go_before_dawn` (world 40).
- Why, if known: `baselife.rs` `dismiss` pays only what was left unpaid at earlier dawns; wages are only charged at 06:00.
- Found by: naming session, 10 Oct

### NM-7 — Clicking a resident's job round past "Cook" uses up grain and timber
- Part: Base building
- How bad: wrong outcome
- What happens: the window's only control is "next job". Stepping a resident from Farmer round to Hauler passes through Cook, which starts a bake at once: 2 grain and 1 timber are gone (10 → 8, 10 → 9) and no bread comes of it.
- What should happen: changing someone's job costs nothing.
- To see it: ignored test `nm7_stepping_someone_s_job_round_past_cook_uses_nothing_up` (world 12).
- Why, if known: `baselife.rs` `set_base_job` starts the new job's round immediately and takes its inputs.
- Found by: naming session, 10 Oct

### NM-8 — The travelling squad can go past ten
- Part: Town jobs and recruiting / base building
- How bad: wrong outcome
- What happens: recruit to 10, leave five at a base, recruit five more, pick the five back up: 15 in the travelling squad.
- What should happen: the limit holds however people join.
- To see it: ignored test `nm8_the_travelling_squad_never_exceeds_ten` (world 1).
- Why, if known: the limit is checked in `recruit.rs` only; `baselife.rs` `pick_up` doesn't look.
- Found by: naming session, 10 Oct

### NM-9 — The text tool crashes on a person number that doesn't exist (same as RG-5)
- Part: Odd input
- How bad: crash
- What happens: `talk p999999`, `loot p999999`, `attack p999999` and `carry p999999` each stop the tool with "index out of bounds".
- What should happen: "Can't…" and carry on.
- To see it: any save; the four commands above.
- Also: `cast NAME 1 p999999`, and reading a harmful scroll at a bad number (`use NAME N p999999` with a Scroll of paralysis in slot N), crash the same way (`world.rs:600`). Not crashing: `go p999999`, `use` with food or a draught, any command given a building or container id where a person is wanted.
- Why, if known: no range check before `dialogue.rs:285` (`order_talk`), `world.rs:600` (`person_pos`), `fights.rs:91` (`attack`), `carry.rs:102` (`can_carry`).
- Found by: naming session, 10 Oct (also RG-5); `cast` and the scroll by [R], re-run

### NM-10 — A squad that starves in the middle of a town lies there for ever
- Part: Townsfolk's lives / the body (overlaps RG-21)
- How bad: stuck
- What happens: an idle squad standing in a town with no food goes down on day 8. All four show "down, health 0%, STARVING" and are still so on day 14 and after. Nobody in the town does anything; there is no way to get up and no game over.
- What should happen: something ends it: a townsperson feeds them, they wake weak, or the game says it's over.
- To see it: world 3, `wait` in the starting town for 8 days (save `a.save` kept by the naming session).
- Also: it happens with 155 coin in the squad and a merchant in the same town: nobody buys food by themselves [R]. Once everyone is down nothing stops a long `wait` (see NM-75). A squadmate with food can get a starved member up again with `give` [R]; with all four down there is nobody to do it.
- Why, if known: starvation knocks out but never kills (canon for this slice), and nothing feeds or rouses the downed.
- Found by: naming session, 10 Oct; seen again by [R] (world 3, day 8 to day 22) and [I]

### NM-11 — Stealing in an occupied home by day is never seen
- Part: Crime and the law / containers
- How bad: wrong outcome
- What happens: at 11:00, inside a cottage whose household is at home, things on the floor and a barrel show "0% chance of being seen", and taking everything has no consequence. The same kind of barrel at 23:31 with the household asleep shows 26%. Daylight with people about is the safe time to burgle.
- What should happen: people at home and awake are the likeliest to notice.
- To see it: world 3, town Stilledge, about 11:00: `enter` an occupied home, `search` a container, `takeall`.
- Also: [R] found every home at 0% from about 07:20 to at least 17:35 in four towns; the only times with witnesses were 06:00 to 07:00 (people not yet up) and after the 20:00 lock-up. One exception [T]: a Qotiro "living quarters" by day showed 51 to 59% with someone awake inside, so that kind of building does keep people indoors.
- Why, if known: `pursuit.rs` `spotter` only counts people inside the same building; `routine.rs` `spot_pos` puts anyone awake "at home" out in the yard, so by day nobody is ever indoors.
- Found by: naming session, 10 Oct; [T] [R] [I]

### NM-12 — Someone asleep can still search, take and pick locks
- Part: Buildings and interiors
- How bad: wrong outcome
- What happens: a member shown as "asleep" searched a barrel, took dried fish and mussels, then started on a locked chest and was "seen picking a lock".
- What should happen: a sleeper (or someone knocked out) does nothing until woken.
- To see it: world 3: `rest`, `wait` until "asleep", then `search k…` and `takeall` with that member selected.
- Also: a sleeper picks things up off the floor ("06:06 Moker picks up buckler") [R], and a squad that is all "asleep" opens a conversation at once [I].
- Why, if known: `loot.rs` `source_now` / `take_from` and `buildings.rs` `do_picking` don't check for asleep or down.
- Found by: naming session, 10 Oct; [T] [R] [I]

### NM-13 — "[130 m from the others]" is shown on everyone when one member is far off
- Part: The squad list (my own work, playtest item 10)
- How bad: misleading
- What happens: with one member about 400 m from the other three, each of the three is flagged "[~125 m from the others]" although they stand together.
- What should happen: only the one who has strayed is flagged.
- To see it: world 3: `select` one member, `go n 400`, `wait 10`, `look`.
- Why, if known: `squad.rs` `World::strayed` measures from the average position of the other members, which one far member drags away.
- Found by: naming session, 10 Oct

### NM-14 — `search` on a locked chest reports the wrong thing
- Part: Buildings and interiors (text tool)
- How bad: misleading
- What happens: searching a locked chest printed "They couldn't get to it: they couldn't reach it in time (it may be behind a wall or a locked door)". What had really happened was that the member was seen picking its lock.
- What should happen: the tool says what happened (picking, picked, seen).
- To see it: world 3, a locked chest at night with a sleeper near: `search k…`.
- Other faces of it, all seen: it shows another member's old barrel from a house 30 m away [T]; "Moker has no lockpicks left; the lock holds." for a member who never had any, because the nearest member is sent, not the one with picks [T]; "<a bound member> was caught and is bound to work it off" while the selected member picks the lock without trouble [R]; a locked chest costs 10 minutes before anything is printed [T].
- Why, if known: `play.rs` `search` waits for a looting entry; lock picking is tracked separately, and old looting entries are never cleared.
- Found by: naming session, 10 Oct; [T] [R]

### NM-15 — Every alert is printed twice in the text tool
- Part: Text tool
- How bad: rough
- What happens: "!! Krogdrur of the watch catches Wehu." appears as an alert and again under News in the same reply.
- What should happen: once.
- To see it: any crime that is seen; world 3.
- Why, if known: `pursuit.rs` `wrong_seen` and `post_bounty` push to both the log and the alerts; `play.rs` prints both.
- Found by: naming session, 10 Oct

### NM-16 — "go n 60" for one selected member walks them about 380 m
- Part: Text tool (orders)
- How bad: rough
- What happens: with one member selected and standing apart from the squad, `go n 60` sends them 60 m north of the squad's middle, not of where they stand.
- What should happen: measured from the member(s) being ordered.
- To see it: world 3: `select` one member, move them away, then `go n 60`.
- Why, if known: `play.rs` `go DIR M` uses `squad.pos`.
- Found by: naming session, 10 Oct

### NM-17 — The clock line says "night" at 18:00, in daylight
- Part: Text tool
- How bad: misleading
- What happens: the header reads "(night)" from 18:00; it is light until about 19:00 and doors lock at 20:00.
- What should happen: the word follows the light.
- To see it: any world, `look` at 18:30.
- Why, if known: `play.rs` header.
- Found by: naming session, 10 Oct

### NM-18 — "Sells for…" disappears from the pack of someone standing in town
- Part: Trade (my own work)
- How bad: rough
- What happens: the pack's "sells for N here" lines vanish for a member who is in a town when the rest of the squad is outside it.
- What should happen: the price is for where that member stands.
- To see it: world 3: three members 400 m out of town, one in it; `pack NAME` for the one in town.
- Why, if known: `economy.rs` `sells_for` uses the squad's middle (`squad.pos`).
- Found by: naming session, 10 Oct

### NM-19 — The law's lines tell more than anyone would know
- Part: Crime and the law
- How bad: too much telling
- What happens, each seen:
  - the bond line carries a tutorial sentence: "…led off to work it off. An official there can sell you the bond.";
  - "!! Nolu is seen stealing! Haʻo saw it, but says nothing." names a stranger (often a sleeper the squad never saw) and reports their private choice, as a red alert;
  - "Fehawesis runs to fetch the watch!" names the witness;
  - every door and chest in `look` gives the lock's number, the exact chance a try and the snap odds ("a fair lock (37): Nolu ~47% a try, 6 lockpicks (about 1 in 3 snaps on a miss)"), and each snapped pick repeats the chance;
  - other people's things lying inside locked houses are listed through the walls from 30 to 50 m, each with its odds ("g55 1 × Ring of the Ox on the ground (someone's…) — 37 m west").
- What should happen: the squad learns that it was seen only when something comes of it; a lock is "simple / fair / hard" until tried; what is indoors is seen from indoors. (Theft odds on a container you are at stay: Laz likes those.)
- To see it: any theft with a witness; `look` near houses at night.
- Why, if known: `law.rs` (the bond line), `pursuit.rs` `wrong_seen`, `buildings.rs` `lock_outlook`, `play.rs` `nearby`.
- Found by: naming session, 10 Oct; [T] [R] [I]

---

## 1b. Seen in the play-throughs: crime and the law

### NM-20 — Step into the house next door, or walk 100 m down the street, and the watch gives up
- Part: Crime and the law
- How bad: wrong outcome
- What happens: seen stealing, the squad walks into the next longhouse 15 m away and stands there; two minutes later "has slipped the watch". No guard comes in. Walking 100 to 120 m along the street and standing in the open does the same; at 30 m or 60 m, or standing still, the thief is caught within a minute.
- What should happen: the watch follows through a door and along a street.
- To see it: world 21, Stillham, saved moment `th21-free-cup`: `takeall`, `enter b5.4`, `wait 1`, `wait 1`, `wait 2`. Also world 3 `rk3-d2-chase`, world 34 Stonehaven.
- Why, if known: `pursuit.rs` `chase` / `guard_sees`: the guard heads for the last place the thief was seen outdoors and can't see into buildings; the chase ends after 120 s. (Oddly, a thief who stays inside a still-locked cottage is caught there: `th21-lockedin-seen`.)
- Found by: [T] [R], 10 Oct; re-run

### NM-21 — Getting away costs less than being caught
- Part: Crime and the law
- How bad: wrong outcome
- What happens: the same theft from the same saved moment. Run: a bounty of 12. Stay: "must pay 18", and with no coin a two-day bond. Another: bounty 48 against "must pay 71" (all 41 coin taken and a four-day bond). Paying a bounty also earns goodwill.
- What should happen: running makes it worse, not better.
- To see it: world 21, `th21-seen`: `go n 300` against `wait 1`, `wait 1`, `wait 2`.
- Why, if known: `pursuit.rs`: the bounty is the plain fine; the duel's loser pays 1.5 times.
- Found by: [T] [R], 10 Oct

### NM-22 — Nobody acts on a bounty
- Part: Crime and the law
- How bad: wrong outcome
- What happens: a wanted squad walks back in three minutes later, stands among six guards, talks to the guard who chased it ("Early, aren't you? The stone's still cold." / "You owe 70. You've got 0. Come back when you can pay."). The same three days later, and after running from a bond. The town's merchant buys the stolen goods meanwhile.
- What should happen: a guard who meets a wanted person, at least one who can't pay, arrests them; people who know of it don't greet them as strangers.
- To see it: world 21, `th21-later-guardtalk`: `say 6`; world 3, `rk3-d2-chase` onward.
- Why, if known: a bounty is only ever a talk topic (`dialogue.rs` PayBounty); nothing in `pursuit.rs` starts a chase from one.
- Found by: [T] [R], 10 Oct

### NM-23 — A bounty can be paid to anyone
- Part: Crime and the law
- How bad: rough
- What happens: a carpenter, a merchant, a farmer, a scribe met in the street at night: each offers "Pay my bounty" and takes the coin ("12 coin. Consider the matter closed — this time.").
- What should happen: paid to the watch or at the hall.
- To see it: world 21, `th21-canpay`: `say 6`.
- Why, if known: `dialogue.rs` offers PayBounty to anyone whose home town has heard of it.
- Found by: [T] [R], 10 Oct

### NM-24 — The thief keeps what was stolen, and it sells in the same town
- Part: Crime and the law
- How bad: wrong outcome
- What happens: 45 coin taken in plain sight; caught a minute later; "a fine of 32"; the 45 stay (84 coin becomes 52). A stolen iron helm survives a lost duel and a 13-day bond; dropped, it lies there as nobody's; a squadmate picks it up and sells it to that town's merchant for 96.
- What should happen: what was seen stolen goes back on arrest; the fine is on top.
- To see it: world 34, Woodsands, `th34-ws-seen`: `wait 1`. Helm: `rk34-0826`.
- Why, if known: nothing in `law.rs` `judge` touches the goods; dropping an item clears its owner.
- Found by: [T] [R], 10 Oct

### NM-25 — "Take all" goes on taking after being seen, and each sighting is fined again
- Part: Crime and the law / containers
- How bad: wrong outcome
- What happens: one `takeall` on a three-stack cupboard prints two "is seen stealing" alerts and still takes all three. The same two things from the same chest: one sighting, "a fine of 30"; two sightings in one go, "a fine of 65".
- What should happen: stop at the first sighting; one grab, one charge.
- To see it: world 21, `th21-free-cup`: `takeall`. Fines: world 34, `th34-note-watch` against `th34-note-watch2`.
- Why, if known: `loot.rs` `take_all_from` only stops if the container is no longer in reach; `pursuit.rs` `wrong_seen` adds a fine per sighting.
- Found by: [T], 10 Oct; re-run

### NM-26 — Taking your own things back out of someone's chest is theft
- Part: Containers / Crime and the law
- How bad: wrong outcome
- What happens: put your own flatbread, fish and torches into a chest in someone's home, take them back: "seen stealing", the watch, a lost duel, "must pay 34". `put` also moves a whole stack, so your bread mixes with theirs.
- What should happen: what you put in is still yours, or `put` warns that it is given away.
- To see it: world 34, Stonehaven, `th34-own1`: `takeall`, `wait 1` ×3.
- Why, if known: ownership is the container's, not the item's (`containers.rs` `took_from`).
- Found by: [T] [R], 10 Oct; re-run

### NM-27 — Notes don't pay for anything, though the purse line counts them
- Part: Crime and the law (money)
- How bad: wrong outcome
- What happens: 6 coin and a 50-coin note; the header says "56 coin between them". Fined 65: the 6 go, the note stays, and the member is bound for 7 days with the header reading "50 coin". The buy-out says "not enough coin", a 130 bounty says "You've got 0", a 28-coin recruit says "you can't pay what they ask", and a 12-coin hide can't be bought.
- What should happen: notes pay (with change), or the purse shows coin and notes apart.
- To see it: world 34, `th34-note-watch2`: `wait 1`, `wait 1`. Shop and recruit: world 21, `tr21-notes-16coin-2notes`.
- Why, if known: `law.rs` `pay_or_bond` and `buy_out`, `recruit.rs`, `economy.rs` count only "coin"; `play.rs` adds 50 × notes to the header.
- Found by: [T] [C], 10 Oct; re-run

### NM-28 — Some sightings post a bounty without saying so, and bounties are shown nowhere
- Part: Crime and the law
- How bad: misleading
- What happens: at night with no guard about, a sighting prints only "!! Nolu is seen stealing from a chest!" and nothing follows. Four of those in ten minutes; next morning a farmer's topics include "Pay my bounty": "You owe 130." No line ever said so, and nothing (`look`, `journal`, `town`) shows a bounty or the squad's standing at any time.
- What should happen: the player can find out that they are wanted and for how much.
- To see it: world 34, Woodsands, `th34-notetake`: `takeall`; then `th34-silentbounty`: `say 6`.
- Why, if known: a guess: the bare line is what `wrong_seen` prints when no guard is free to be fetched.
- Found by: [T] [R] [C], 10 Oct

### NM-29 — Being caught again does nothing about what is already owed
- Part: Crime and the law
- How bad: wrong outcome
- What happens: carrying a 130 bounty in Woodsands, the thief is caught twice more that morning and fined 30 and 65 for the new thefts; the 130 is still owed. A 70 bounty likewise survives an arrest and a bond in the same town.
- What should happen: an arrest settles, or adds, what is owed.
- To see it: world 34, `th34-note-watch` then `th34-bondtalk`: `say 6`.
- Why, if known: `law.rs` `judge` looks only at the new wrong.
- Found by: [T] [R], 10 Oct

### NM-30 — A sleeping squadmate is sent to fight the duel, and the accused sleeps through arrest and trial
- Part: Crime and the law (duels)
- How bad: wrong outcome
- What happens: all four asleep in a house. "The watch catches Ṭelihoḍu" (still shown asleep). The champion is another sleeper, at 11% health on the first line of the fight; he loses and is "asleep" again a minute later.
- What should happen: an arrest wakes people; a sleeper is not sent to fight.
- To see it: world 34, `th34-own-seen`: `wait 1` ×4.
- Why, if known: `law.rs` `start_duel` takes the strongest member within 30 m without looking at their state.
- Found by: [T], 10 Oct

### NM-31 — Who fights a duel makes no sense, and it is over before anything can be done
- Part: Crime and the law (duels)
- How bad: wrong outcome
- What happens: (a) the squad's champion is its frailest member (a staff-carrying mage at 75% health) while the accused and two unhurt fighters stand beside her; she was picked again later as the most hurt of four. (b) The town's champion is the same guard every time, fetched from wherever he is: two hours after losing a duel he is sent out at 4% health, so every later charge that morning is a walkover. (c) Five of six duels were decided in the minute they began: no time to choose, order, heal or watch.
- What should happen: the accused fights or the player chooses; the town sends someone fit; a duel lasts long enough to see.
- To see it: world 34, Stonehaven, `rk34-chase3`: `wait 1`, `fight`. First duel: `rk34-chase`.
- Why, if known: `law.rs` `start_duel`; `fights.rs` 316–327 (the champion is placed beside the accused).
- Found by: [R] [T], 10 Oct; the 4% champion re-run

### NM-32 — A beaten town champion is back at his post the instant the duel ends
- Part: Crime and the law (duels)
- How bad: rough
- What happens: "The fight is over: you won. Beaten: Yafesuthih." In the same minute he is listed 58 m away beside the other guard; he can't be gone through; twenty minutes later he says "Hello, traveller." The same in reverse: champions and arresting guards appear from bed and vanish back.
- What should happen: he lies where he fell for a while; the town remembers the duel.
- To see it: world 34, `rk34-duel2`: `wait 1`, `look people`, `loot p4809`.
- Why, if known: the champion's place comes back from his day plan as soon as the fight ends.
- Found by: [R] [T], 10 Oct

### NM-33 — A wounded member's health reads 31%, then 53% the moment a fight starts
- Part: The body (shown in a duel)
- How bad: misleading
- What happens: "health 31%" at 08:36; at 08:37 the duel starts and both the squad list and the fight list say 53%. No healing, no hit.
- What should happen: one true number.
- To see it: world 34, `rk34-chase3`: `look`, `wait 1`.
- Why, if known: a guess: wounds heal at a steady rate between "settles" (CLAUDE.md rule 8), and the squad list shows the value as of the last settle rather than now; starting a fight settles it.
- Found by: [R], 10 Oct; re-run

### NM-34 — A town that holds someone to work doesn't feed them
- Part: Crime and the law (bonds)
- How bad: wrong outcome
- What happens: a bound member with no food goes hungry, starves and is down on the third or fourth day, with 109 coin in her pack. She lies there for the rest of the bond, and the buy-out price keeps falling, so lying starved counts as working. No line marks any stage. (A squadmate can walk food over: `give` works on a bound member.)
- What should happen: the town feeds its bound workers, or takes it from their coin; someone who is down isn't working off the debt.
- To see it: world 34, `id34-bond-d4-2009`: `wait 600`, then `wait 1440` ×4. Also `rk34-ws-nofood`.
- Why, if known: a bound member's food comes only from their own pack (`condition.rs`); `law.rs` counts days, not work.
- Found by: [R] [I], 10 Oct

### NM-35 — A bound member can be carried out of town, and then 60 coin to any guard clears it
- Part: Crime and the law (bonds)
- How bad: wrong outcome
- What happens: a bound member refuses every move order, but once down (from the duel, or starved) a squadmate can pick her up and walk out: "Sawasasih has run from their bond in Stonehaven." Nobody comes after her. Walk back in, talk to a guard ("Hello, traveller."), "Pay my bounty": 60 coin, and a 105-coin, 13-day bond is gone.
- What should happen: carrying off someone bound is stopped or is itself a crime; a runaway who walks back in is seized; the price of running is more than what was owed.
- To see it: world 34, `rk34-downbound`: `select Nolu`, `go p5003`, `carry p5003`, `go t1`.
- Why, if known: `carry.rs` has no bond check; `law.rs` `check_runaways` adds a flat 60.
- Found by: [R] [T], 10 Oct; re-run

### NM-36 — Nothing shows that someone is bound, for how long, or when it ends
- Part: Crime and the law (bonds)
- How bad: misleading
- What happens: a bound member is listed as "standing"; `journal` says "No jobs taken."; days left and the buy-out price appear nowhere. The bond ends without a line. "For 2 days" lasts 2 days 6 hours, "for 7 days" about 7 days 9 hours, "for 1 days" was still on 27 hours later. A bound member left alone never sleeps ("worn out" for three days); told to `rest`, sleeps through the working day. A member bound in one town with a job in another flips between "walking to work" and "standing" all day.
- What should happen: "bound in Stonehaven, 5 days left" on the member; a line when they are free.
- To see it: world 21, `th21-d3noon`; world 34, `rk34-bound1`: `look`, `journal`, `town`.
- Why, if known: `play.rs` `status` (the window does show it, `view/hud.rs` 394); no line in `law.rs` when a bond ends.
- Found by: [T] [R] [I], 10 Oct

### NM-37 — "An official there can sell you the bond", but often he can't
- Part: Crime and the law (bonds)
- How bad: misleading
- What happens: the buy-out topic only exists while an official is at his own post, and he says nothing about it otherwise (no topic at 06:41, there at 07:02). In Woodsands (world 34) the official was asked on three days running with 215 coin in the squad and never offered it: his own words put him "90 m south of the hall" each time. His "Any work?" answer is "Break that camp and I'll pay 0 coin."
- What should happen: the official says when and where; a town with an official can always sell a bond.
- To see it: world 34, `rk34-ws-d3eve`: `select Nolu`, `talk p4417`.
- Why, if known: `dialogue.rs` gates BuyOut on `at_work`; a guess that this official's day plan never puts him at the hall.
- Found by: [R] [T] [I], 10 Oct

### NM-38 — Fines take coin without saying what was taken
- Part: Crime and the law
- How bad: misleading
- What happens: (a) 18 coin in hand, "a fine of 75", "bound for 7 days": the 18 go without a word (7 days matches the 57 left, not 75). (b) In towns that judge "by the record", the only line is "…is written into the record; it will follow them." and 5 or 7 coin leave the purse, from another member's pack, with no sum named; a bond there is "for 1 days" with no debt named. (c) "A fine of 12" took 11. (d) What "it will follow them" means is shown nowhere.
- What should happen: "pays 18 of 75; bound 7 days for the rest."
- To see it: world 34, `rk34-ws-chase`: `wait 1` ×2; Newshaw, `th34-newshaw-seen`: `wait 1` ×2.
- Why, if known: `law.rs` `pay_or_bond` takes what there is silently.
- Found by: [R] [T], 10 Oct

### NM-39 — Shunning costs nothing
- Part: Crime and the law
- How bad: wrong outcome
- What happens: Windflat (world 3) shuns. "Windflat turns its back on Wehu for theft: no one will trade with them." No fine, nothing taken back; she is left at the same barrel, empties it two minutes later, is caught again: the same line and nothing more. A squadmate standing beside her sells the loot to the town's merchant. To Wehu the merchant says only "I don't know you." Nothing shows that she is shunned or when it ends (day 7, silently).
- What should happen: shunning bites the squad she travels with, or at least the loot; a second offence while shunned costs more; the state can be seen.
- To see it: world 3, Windflat, `rk3-wf-chase`: `wait 1` ×3, `takeall`, `wait 1` ×2; then `rk3-wf-0703`.
- Why, if known: shunning only blocks the convicted member's own trading (`economy.rs` 447–451).
- Found by: [R], 10 Oct; re-run

### NM-40 — In some towns nobody ever fetches the watch, and no guard walks any town at night
- Part: Crime and the law
- How bad: rough
- What happens: Woodsands (world 3): seven sightings in one cottage between 07:32 and 14:13, never "runs to fetch the watch", only "Bounty in Woodsands: 75 … 132", with two guards standing 36 m and 59 m away. In every town tried there was no guard on the street at night, so doors were picked in the open unseen; the only night witnesses are sleepers indoors.
- What should happen: a town with guards near sends one; someone keeps watch at night.
- To see it: world 3, `rk3-ws-together`.
- Why, if known: a guess: a witness only fetches a guard who is "at work" at the guard post, and these guards' hours or post put them elsewhere.
- Found by: [R] [T], 10 Oct

### NM-41 — Nobody minds strangers in their house
- Part: Buildings and interiors / Crime and the law
- How bad: wrong outcome
- What happens: the squad walks into homes by day, in the evening, at 01:00; stands there for a day; beds down at 22:00 beside the sleeping household, in locked and unlocked homes, including one it robbed that morning while wanted for it. Nobody notices. A door locks at 20:00 with the squad inside and nothing is said; walking out is free. `look` never lists the people of the house (the only sign of sleepers is the "26% chance of being seen").
- What should happen: someone at home reacts to strangers on the floor; `look` says who is in the room.
- To see it: world 21, `th21-free-inlocked`: `wait 180`; world 34, `id34-d5-0100-inhome`: `enter b0.7`, `rest`, `wait 240`.
- Why, if known: trespass is only ever raised by a lock-pick being seen (`buildings.rs` 489).
- Found by: [T] [I], 10 Oct

## 1c. Seen in the play-throughs: buildings and interiors

### NM-42 — A walk begun at 19:57 ends inside a house that locked at 20:00
- Part: Buildings and interiors
- How bad: wrong outcome
- What happens: from 300 m away at 19:57, `enter` a cottage. At 20:01 all four "are inside the Grown cottage" and the same `look` lists it as locked. No pick, no notice. Ordered at 19:59 from the square instead, the same `enter` has the lock picked in front of the evening crowd, and the picker is seen, duelled and bound.
- What should happen: stop at the locked door.
- To see it: world 21, Stillham, `th21-free-1957`: `enter b5.18`, `look places`. World 34, `id34-c2-1957`: `enter b0.5`.
- Why, if known: `buildings.rs` `route` checks the lock once, when the walk is planned.
- Found by: [T] [I], 10 Oct; re-run

### NM-43 — A chest can be emptied and filled through the wall of a locked house
- Part: Buildings and interiors
- How bad: wrong outcome
- What happens: a member who opened a chest steps 2.2 m outside the wall. `take 1`: "Taken." `put 0`: "Put away." `search` on the same chest from the same spot says he isn't in the building. Having walked right away and come back near the wall, `take` works again without searching.
- What should happen: no reach through walls; walking away closes the chest.
- To see it: world 21, `th21-wall5-base`: `go se 2.2`, `take 1`, `search k5.18.0`, `put 0`.
- Why, if known: `loot.rs` `source_now` is distance only (2.16 m); a looting entry is never cleared by walking away.
- Found by: [T], 10 Oct; re-run

### NM-44 — Ordering the same lock-pick again replays the same result
- Part: Buildings and interiors
- How bad: wrong outcome
- What happens: a hard chest. Each fresh order gives the very same first try: the pick snaps and the same sleeper sees it. Three orders, three snapped picks, three sightings. The reverse also follows: a quiet miss on the first try can be repeated for ever at no risk.
- What should happen: a new attempt is a new roll.
- To see it: world 21, `th21-free-prechest`: `search k5.2.1`, `wait 1`, three times over.
- Why, if known: `buildings.rs` `order_pick` / `containers.rs` `order_pick_container` start `tries` at 0, and the roll is keyed on (who, door, try) only.
- Found by: [T] [R], 10 Oct; re-run

### NM-45 — After a lock is picked the tool never says who got in; someone inside is told "It's locked"
- Part: Buildings and interiors (text tool)
- How bad: rough
- What happens: `enter` with the lock-picker: "(5 minutes pass.)", "picks the lock", and no "is inside". With the squad already inside a locked cottage, `enter` for a member without picks says "It's locked…"; for the picker it picks the lock of the house he is standing in.
- What should happen: say who is inside; no picking from the inside.
- To see it: world 21, `th21-free-prepick`: `enter b5.14`; `th21-free-inlocked`.
- Why, if known: `play.rs` `enter`.
- Found by: [T], 10 Oct

### NM-46 — A workshop's crafter never comes indoors; the hall, stalls and desks can't be found
- Part: Buildings and interiors
- How bad: rough
- What happens: the squad stood in a Bench house for 21 hours: the woodworker was always 11 to 13 m outside, never at the bench. "Has to be done at a scribe's desk", but `look places` lists only houses and out-of-town works; the hall, market, stalls, food shop, smithy and scribe's room have no entry. Guessing a hidden id works (`go w2.17`, then the scroll can be made).
- What should happen: people work in their workshops; places that talk and jobs name can be found.
- To see it: world 34, `id34-d5-1201-inhome`: `enter b0.4` and wait; `rk34-prefireball`.
- Why, if known: `routine.rs` `spot_pos` puts workers at a yard spot (the same cause as NM-11); `play.rs` `nearby` doesn't list town workplaces.
- Found by: [I] [R] [C], 10 Oct

## 1d. Seen in the play-throughs: town jobs and recruiting

### NM-47 — Between some neighbouring towns people walk on the spot, and a member with a job does it every shift
- Part: Town jobs and recruiting / travel
- How bad: stuck
- What happens: a member with a post in Stillham sleeps in Stillbank (world 21). At 08:00 he sets off alone, stops about 930 m out and is "walking to work" at that same distance until 17:00, stamina falling from 100% to 49%; nothing is said. The whole squad's `go` to the town stops dead at the same place ("(0 minutes pass.)" on a second try); coming back, three of seven members could not be ordered into town by any order. The same north of Woodsands on the way to Stonehaven (world 34): "[3180 m from the others]" all day. In world 3 the same kind of walk (Stilledge to Windflat, 2.8 km) arrives in half an hour, so it is the place, not the job.
- What should happen: a way between two neighbouring towns, or a line saying why they stopped; a worker who can't get there gives up and says so.
- To see it: world 21, `tr21-far-d2-0808`: `wait 240`, `wait 300`. World 34, `rk34-d3-0835`: `wait 10`, `wait 60`, `wait 300`. Squad: `tr21-d3-leave`: `go t4`, `go t4`.
- Why, if known: a guess: the straight line meets the sea or something else that can't be walked, and nothing routes round it. On today's `main` the squad is told "The water stops them" (town-care); a worker's own walk still says nothing. To be re-checked on `main`.
- Found by: [C] [R], 10 Oct; re-run (both worlds)

### NM-48 — A member with a town job walks off alone, wherever they are, without a word
- Part: Town jobs and recruiting
- How bad: rough
- What happens: at 08:00 the member leaves the squad for the post, from another town or from 3 km off, with no line. Told to `rest` in shift hours, they are back "at work" within minutes. Ordered away during the shift they show "walking to work" while walking 8 km in the other direction, then turn round. A whole-squad `go` pulls a worker 2.7 km back from the post for a 40 m move.
- What should happen: a line when someone leaves for work; an order from the player wins, and says what it costs.
- To see it: world 21, `tr21-midshift`: `select Ḍuhuquʻi`, `go e 9000`, then waits.
- Why, if known: `chances.rs` `work_contracts` re-routes an idle worker in shift hours with no distance limit and no line.
- Found by: [C] [R], 10 Oct

### NM-49 — Missed days are never mentioned, a post vanishes after three, and a day's pay is all or nothing
- Part: Town jobs and recruiting
- How bad: misleading
- What happens: present five hours of nine (having walked 17 km in between): the full 19 coin "for a day's work". Present twenty minutes: nothing, and no line; the journal shows days paid but never days missed. After three missed days both posts are simply gone from the journal, with no line on any of those days. Quitting at 15:50 after nearly eight hours forfeits the day, and quitting is only possible while an official is on duty.
- What should happen: a line for a day that didn't count and for a post taken away; the player can tell what a day is.
- To see it: world 21, `tr21-d6-0958`: `select all`, `wait 1210`, `journal`; `tr21-quit-before`.
- Why, if known: `chances.rs` `dawn_contracts`: `missed` never resets and a post ends with no `say`.
- Found by: [C], 10 Oct

### NM-50 — Anyone can take a skilled post, and the town fills it with a local as well
- Part: Town jobs and recruiting
- How bad: wrong outcome
- What happens: a recruited labourer with no crafts at all is taken on as "alchemist at the healing house (49 coin a day)" and paid. Next dawn "Yeyasa takes up work as alchemist in Stilledge." as well. In world 21 a member held and was paid for a smith's post while locals "took up work as smith" on two dawns. Two members took the same labourer post.
- What should happen: a skilled post asks for the skill; a post a squad member holds is not also filled.
- To see it: world 3, Stilledge, `tr3-alch-before`: `craft Doqe`, `say 6`.
- Why, if known: `chances.rs` `take_post_work` / `vacant_posts` have no skill check and don't mark the post as taken.
- Found by: [C], 10 Oct; re-run

### NM-51 — Skilled posts never stay filled: every dawn new people "take up" smith, scribe, alchemist, and "No trade" piles up
- Part: Townsfolk's lives / jobs
- How bad: wrong outcome
- What happens: in Stillham over 14 days "No trade" went 0, 5, 15, 29 and the same vacancy was advertised again and again; yesterday's smith is "no trade, will join for nothing" a week later. Across the whole world, with the squad standing still: nobody has "No trade" on day 1; about 150 people on day 10, 280 on day 30, 360 on day 60 (worlds 3, 21 and 34 alike).
- What should happen: a post once taken stays taken unless something happens to its holder; someone who leaves a post goes back to ordinary work.
- To see it: `headless society 1 21`, then `10`, `30`, `60`: add up "No trade" over the towns. In play: world 21, `id21-d8-0607-shore`: `town`, `wait 1440`, `town`, six times.
- Why, if known: not looked into. A guess: the dawn step that fills vacant posts (`lives.rs` / `society.rs`) and the one that drops people from posts disagree about who holds what.
- Found by: [I], 10 Oct; counted by the naming session on three worlds

### NM-52 — Recruits are coin from nothing
- Part: Town jobs and recruiting
- How bad: wrong outcome
- What happens: a recruit signs on for 28 coin wearing a kite shield the same town's merchant buys for 118; another's helm sells for 101. From day 3 drifters join for nothing, one carrying a glaive and a scale hauberk worth 372 to that merchant. `town` lists every would-be recruit with their kit, so the richest can be picked.
- What should happen: a recruit's own kit isn't the squad's to sell at once, or the fee follows what they bring.
- To see it: world 21, `tr21-sellkit`: `unequip Sathufiya off`, `unequip Yawashifas head`, `talk p817`, `say 6`, `say 13`, `say 13`.
- Why, if known: `recruit.rs` `join_terms` prices the person, not the kit.
- Found by: [C], 10 Oct

### NM-53 — Who will join makes little sense, and recruits starve without a word
- Part: Town jobs and recruiting
- How bad: rough
- What happens: six people recruited in twelve minutes in a town whose watch had just caught the squad twice, among them a guard of that watch and the priestess; a man who "can't see to it myself" hands over a letter and then joins for nothing. Each recruit brings three flatbread; from day 5 they are weak, then starving, then down, standing beside squadmates with bread: no line, nobody leaves or eats in their own home town. `give` hands over a whole stack only, and there is no way to send anyone away. Seen once, unsure: "Thuwah takes up work as weaver and sealer in Windflat." at dawn, about a recruit who was in the squad and starving.
- What should happen: a line when someone runs out of food; a way to share and to dismiss.
- To see it: world 3, `rk3-ten`: `town`, `wait 1440` ×6.
- Why, if known: each member eats only from their own pack (`condition.rs` `food_for`); overlaps RG-21.
- Found by: [R] [C] [I], 10 Oct

### NM-54 — A talk never closes: sell, take a job or recruit from two kilometres away
- Part: Talk (text tool) / jobs and recruiting
- How bad: wrong outcome
- What happens: open a talk, don't say goodbye, walk away. From 365 m, 1.5 km and 2 km, minutes or hours later, `say N` still sells and buys, takes a town post, tries to buy out a bond, and recruits: "28 coin to my household, and I'm yours." and she is at once standing beside the squad 2 km from where she was. The talk was still answering 15 hours later with the merchant long in bed. A guard on a chase also opens a talk and makes his arrest 60 m away in the same minute.
- What should happen: a talk ends when the two part; nothing is done at a distance.
- To see it: world 21, `tr21-recruit-open`: `go e 2000`, `wait 120`, `say 6`, `look`. World 34, `id34-far-woodlot`: `say 19`.
- Why, if known: the window freezes time while a talk is open (`view/app.rs` 837); the text tool doesn't, and nothing in the sim closes `World::talk` or checks distance (`recruit.rs` 102: the recruit is placed beside the talker).
- Found by: [R] [C] [I] [T], 10 Oct; re-run

### NM-55 — Coin and goods are shared over any distance (more cases of RG-1 and RG-20)
- Part: Trade / jobs and recruiting
- How bad: wrong outcome
- What happens: a recruit's 64-coin fee is paid from a pack 2.8 km away; a "bring 3 kelp" job is handed in with kelp held by a member in another town; "Sell all 71 × timber" empties the packs of three members 450 m off; a member bound in another town 2.2 km away has her things sold and her coin spent on another's buy-out; a fine takes coin from all four packs wherever they stand.
- What should happen: only the talker and those standing with them.
- To see it: world 3, `tr3-guard-recruit`: `pack Menmuk`, `say 6`, `pack Menmuk`.
- Why, if known: `squad_count` / `take_from_squad` span the whole squad.
- Found by: [C] [R] [T] [I], 10 Oct; re-run

### NM-56 — "I'll tell the watch": one click with anyone earns "X helped us when nobody else would"
- Part: Talk / Crime and the law
- How bad: wrong outcome
- What happens: anyone who passes on news of a crime in another town offers "I'll tell the watch." Picking it gives "Good. Let them answer for it." and nothing else is shown. Open the talk again and the same person says "Weyusauthi helped us when nobody else would. I won't forget it.", to Weyusauthi's own face, in the third person, though the crime was in another town and never touched the speaker.
- What should happen: telling the watch means going to the watch; thanks come from the people helped, for something done, said as "you".
- To see it: world 21, `tr21-long-d7-board`: `select Weyusauthi`, `talk p334`, `say 1`, `bye`, `talk p334`.
- Why, if known: not looked into (the reward for reporting is given to the hearer, and the line template names the helper).
- Found by: [C], 10 Oct; re-run

### NM-57 — "Here's something for your trouble (10 coin)" is offered where nobody knows anything, and the coin is kept
- Part: Jobs ("find out who wronged X")
- How bad: wrong outcome
- What happens: a merchant robbed in the night shows "Here's something for your trouble (10 coin)." and "You'll tell me what you know." though no job has been taken. Paying costs 10 coin: "I don't know anything about it." With the board job "find out who wronged Xuqud" taken, the same two topics are on Xuqud himself, who posted it; nobody else among 50 asked had them, so the job could not be done.
- What should happen: the topics appear on people who might know, once there is something to ask; no coin for "I don't know".
- To see it: world 21, `tr21-long-d12-findout`: `select Weyusauthi`, `talk p817`, `say 2`.
- Why, if known: `chances.rs` `press`: the topic is offered on the victim of the story.
- Found by: [C], 10 Oct; re-run

### NM-58 — Small things about jobs
- Part: Town jobs and recruiting
- How bad: rough
- What happens, each seen by [C]:
  - a creditor pays 15 coin to have a 9-coin debt collected (the reward has a floor of 15); "Tell me, or else." can be said seven times with no consequence;
  - a job never says where its person is; a letter's journal line gives no distance or direction (one was 18 km); two people of the same name in one town and jobs give only the first name;
  - in six days and about 280 talks across four towns nobody had "Any work?"; the boards held only letters and "bring 3 kelp"; the first other job came on day 7;
  - the panel lists three or four posts going, the official offers two; one official offers none;
  - "a favour owed" is paid as "0 coin — fair's fair."; a letter is offered for "0 coin when it's done — they'll pay you."
- To see it: world 21, `tr21-debt-talk`, `tr21-long-d12-anywork`.
- Found by: [C], 10 Oct

### NM-59 — A merchant shows only seven wares, the same seven as the next merchant, and after day 1 often no food
- Part: Trade (found while keeping a squad fed)
- How bad: stuck (for a squad that won't steal or hunt)
- What happens: "What have you got?" lists at most seven things to buy and no "what else?". On day 1 one of them was flatbread; from day 4 or 5 both merchants in Stilledge and three in Windflat show the same seven arms and clothes and no food. Stonehaven never sold bread again after day 1 (days 6 to 10 checked). A squad with 80 coin could buy nothing to eat.
- What should happen: all wares can be reached; a town sells food.
- To see it: world 3, `tr3-goon`: read the Buy lines; day 1 for comparison, `tr3-tradeopen`.
- Why, if known: `dialogue.rs` takes the first seven of the shelf (`.take(7)`).
- Found by: [C] [I], 10 Oct; re-run. For the ruins-gangs session's list (trade).

## 1e. Seen in the play-throughs: townsfolk's lives

### NM-60 — A merchant stands in the empty square until midnight, then is gone on the stroke of 00:00
- Part: Townsfolk's lives
- How bad: wrong outcome
- What happens: at 23:00, 23:30 and 23:59 she stands outdoors 13 m away with everyone else gone; at 00:00 she is nowhere. No walk home.
- What should happen: she walks home and goes to bed.
- To see it: world 34, Stonehaven, day 2, `id34-d2-2359`: `look people`, `wait 1`, `look people`.
- Why, if known: `routine.rs` `Plan::push`: when the evening starts after bedtime the walk home and the sleep are dropped, so the plan ends at midnight.
- Found by: [I], 10 Oct; re-run

### NM-61 — "At their stall from 08:30 tomorrow" when tomorrow is their day off
- Part: Townsfolk's lives (town panel)
- How bad: misleading
- What happens: Stonehaven's two merchants trade on alternate days. On the evening of a trading day the panel promises "tomorrow"; next day they don't trade and it says "tomorrow" again. A merchant who is away drops off the list altogether after being promised.
- What should happen: the real next time, or no promise.
- To see it: world 34, `id34-d1-1700`: `town`; `id34-d2-0840`: `town`.
- Why, if known: the panel looks one day ahead only.
- Found by: [I] [C], 10 Oct

### NM-62 — Greetings don't follow whether you've met
- Part: Townsfolk's lives / talk
- How bad: misleading
- What happens: a stranger's first words are "There you are! I was hoping you'd come by… oqe (friend)." A merchant who sold the squad 15 flatbread says three days later "I don't know you. Say who you are and what you want." The same person switches between the two from day to day. Roduro strangers greet a Roduro member as a friend and a Horaro member with "You're not from here, are you?" in the same minute.
- What should happen: the first-meeting line once; warmth after you've met. (The split by people may also sit against CLAUDE.md rule 15.)
- To see it: world 34, `id34-d2-0601`: `talk p492`; `id34-d4-1141-A`: `talk p1467`.
- Why, if known: `dialogue.rs` `disposition` adds for same people (+10) and doesn't know about earlier meetings.
- Found by: [I] [C], 10 Oct

### NM-63 — The news pushes at the squad what it couldn't know and wouldn't miss
- Part: Townsfolk's lives
- How bad: too much telling
- What happens: standing still in a square, asleep, or indoors, the squad gets:
  - a line for every party on the roads, 10 to 60 a day: leader's name, head count and where they are bound ("Hihowora and 2 others, bound for Windflat."; "Saisiye (Ṭaḍoro), wandering, crosses your path." while all four sleep);
  - fights out of sight to the metre ("A Mirejaw falls on travellers 1114 m away." / "The roadside fight is over."), the same one most mornings;
  - who took which post at dawn, in two towns at once, from open country 2.2 km away ("Yisausai takes up work as armourer in Fieldhearth.");
  - "X is paid 19 coin for a day's work." per working member per dawn;
  - "Stillbank comes into view." each morning while standing still.
- What should happen: nothing pushed for what isn't seen or wouldn't be missed.
- To see it: world 21, `id21-wilds-d7-0722`: `wait 700`, `wait 659`. World 34, `id34-d3-0131`.
- Why, if known: `news.rs` (parties in view), `society.rs` 1166 (posts within 2.5 km), `chances.rs` 506 (pay).
- Found by: [I] [C] [R] [T], 10 Oct

### NM-64 — Gossip is a world-wide counter and one story
- Part: Townsfolk's lives (rumours)
- How bad: too much telling
- What happens: "Bandits have fallen on travellers 87 times since the season turned." The number is the whole world's running count, the same from everyone, and it ticks up between one person and the next (87, 88); it climbs about 45 a day (40 on day 2, 93 on day 3, 130 on day 4 in world 34). Apart from that, a farmer gives "a band of 4 camped about 3.2 km east". Every piece of news of a deed in two worlds and ten days is the same one, "X squeezed money out of Y", always about another town; nothing about the town you are in.
- What should happen: hearsay about something near, without a count; more than one kind of story; some local talk.
- To see it: world 34, `id34-d2-0601`: `talk p492`, `say 4`; again on later days.
- Why, if known: `dialogue.rs` 623–626 quotes `stats.ambushes`.
- Found by: [I] [C], 10 Oct

### NM-65 — `look` and `town` say who would join, for how much, and what they own, before a word is spoken
- Part: Town jobs and recruiting
- How bad: too much telling
- What happens: on the first `look` of a new game a stranger 29 m away is tagged "[restless: might join for 28 coin]" with skills and kit. `town` lists all 14 "Willing to join" with price, best skills, weapon, clothes and whereabouts, including "asleep indoors". The opening tip says so outright.
- What should happen: found out by talking.
- To see it: any new game: `look`, `town`.
- Why, if known: `play.rs` 401–405 and 1202–1225; the window's hover has the same tag (`view/hud.rs` 302).
- Found by: [C] [R] [I], 10 Oct

### NM-66 — Visitors stand in the street all night
- Part: Townsfolk's lives
- How bad: rough
- What happens: at midnight the only people about are seven or eight "traveller"s on the same spots hour after hour, in a town with an inn; two arrivals stood on one spot from 03:56 to past 10:04. Someone who has arrived stays "traveller" in `look` while `talk` gives their real trade.
- What should happen: visitors go indoors; one label.
- To see it: world 21, `id21-d2-2300`: `wait 30`, `look people`, `wait 29`, `look people`.
- Found by: [I], 10 Oct

### NM-67 — Small things in the text
- Part: Townsfolk's lives / text tool
- How bad: cosmetic
- What happens, each seen:
  - "Yafiyayuh and 1 other, ." (a party line with nothing after the comma);
  - "(1 minutes pass.)", "1 days paid", "for 1 days", "[Done] Done: a job for…", "TALKING with Fusa (Ṭaḍoro no trade)" after he joins;
  - "took spear", "They took today" (missing words in a robbery line);
  - "Someone's been at the locks round here lately." after a theft with no lock touched;
  - "People about (N; closest first)" lists the "restless" first, not the closest;
  - "The downed will come round in an hour or two." and they are up in 5 to 25 minutes;
  - "you could walk to the coast" said in a coastal town;
  - "We owe 2 coin, and it isn't getting any smaller…" (a lament over two coin, with the sum).
- Found by: [R] [I] [C] [T], 10 Oct

## 1f. Seen in the play-throughs: the text tool and odd input

### NM-68 — `go nan,nan` breaks a squad member for good
- Part: Odd input
- How bad: stuck
- What happens: after `go nan,nan` (also `inf,inf`, `go n nan`), every later `go` for that member says "Nobody needs to move: they're already there." and burns 30 minutes; the "[N m from the others]" flags vanish for everyone. It is written into the save. `cast NAME 3 nan,nan` is accepted too.
- What should happen: "Go where?" for anything that isn't a real number.
- To see it: world 34, `rk34-prenan`: `go nan,nan`, `go n 100`, `go s 50`.
- Why, if known: `play.rs` `go` parses "nan" as a number; the member's goal becomes not-a-number.
- Found by: [R], 10 Oct; re-run

### NM-69 — A missing or zero argument quietly acts on member 1 or line 1
- Part: Odd input
- How bad: wrong outcome
- What happens: `take`, `take 0` and `take -1` steal line 1 (42 coin, seen and charged). `dose` with no names uses up a draught on someone at full health. `unequip` takes off member 1's weapon. `give Nolu 0` hands a knife to member 1. `cast X 0` and `make X 0` do number 1. `pack Nobody` shows member 1. `wait x` waits ten minutes.
- What should happen: a usage line.
- To see it: world 34, `th34-chestopen`: `take 0`; `rk34-oddbase`: `dose`, `unequip`, `give Nolu 0`.
- Why, if known: `play.rs`: `parse().unwrap_or(0)` then `saturating_sub(1)`; a missing name falls back to the first member.
- Found by: [R] [T], 10 Oct; `take 0` re-run

### NM-70 — The letter of an id is ignored
- Part: Odd input
- How bad: wrong outcome
- What happens: `pickup x6` in a home picked up the owner's war-pick (g6) and was seen stealing. `gather p59` sent the squad off to plant n59. `work h2` marched everyone out of town to woodlot d2. `enter 1.2` enters b1.2.
- What should happen: "Can't do that."
- To see it: world 34, `rk34-ws-house`: `select Sawasasih`, `pickup x6`.
- Why, if known: `play.rs` 908 drops the first character whatever it is.
- Found by: [R], 10 Oct

### NM-71 — `take` and `put` work whichever container was opened first, even one 2.5 km away
- Part: Containers (text tool)
- How bad: wrong outcome
- What happens: with one member still "going through things" at a barrel, another member's `search` lists her own barrel, but `takeall` says "Took 0 lots." and `take 1` "Couldn't take that."; a third member's `takeall` took the second's things. From another town 2.5 km away, twelve rounds of `put` and `take` moved things in and out of the first member's barrel, using her pack's numbering. A member's status stays "going through things" for seven days.
- What should happen: `take` acts on what the selected member just opened; walking away closes it.
- To see it: world 34, `rk34-twoopen`: `select Sawasasih`, `search k0.4.3`, `take 1`. World 3, `rk3-farloot`: `put 5`.
- Why, if known: `play.rs` 833/885 use the first looting entry in reach of its own looter, and looting entries are never cleared (the same root as NM-43).
- Found by: [R], 10 Oct

### NM-72 — Orders to someone who can't act fail silently or burn up to half an hour
- Part: Odd input
- How bad: rough
- What happens: for a bound member `go` and `give` explain; `talk` burns 10 minutes ("They're not talking (or you couldn't reach them)."); `enter` prints only "(5 minutes pass.)" each time; `work` and `pickup` print "(0 minutes pass.)". For a member who is down, `go` costs 30 minutes with no word, also when they are one of a whole squad ordered; `wake` answers "Up." and they stay down. `rest`, `sneak` and `torch` are accepted from the bound.
- What should happen: one clear refusal, at once.
- To see it: world 21, `th21-bound`: `select Numeleo`, then each order. World 34, `id34-bond-d14`: `select Nolu`, `wake`, `go e 5`.
- Why, if known: only `order_members` checks `free_to_order` / `is_down`; the tool waits out its full time limit.
- Found by: [T] [R] [I], 10 Oct

### NM-73 — `talk` picks the wrong person to send, and talking again costs ten minutes each time
- Part: Talk (text tool)
- How bad: rough
- What happens: `talk` always sends the first selected member (member 1 if none), even one stuck 745 m away or bound, while three others stand 14 m from the person. `talk` to someone who isn't in town sends that member walking a kilometre across country, where he stays. Talking again to someone just left prints "(10 minutes pass.)" with no "A conversation opens." (it is open); a round of 50 talks took four hours. `talk` to a second person with a talk open silently swaps. "Can't talk to them." for a merchant not yet up, with no reason.
- What should happen: the nearest free member talks; "they aren't here"; a second talk is as quick as the first.
- To see it: world 21, `tr21-kelp-before`: `talk p4547`; `id21-d4-2000-inn`: `talk p293`. World 34, `id34-d9-0820`: `talk p1467`, `bye`, three times.
- Why, if known: `play.rs` 673 / 751; a stale `want_talk`.
- Found by: [C] [I] [R], 10 Oct

### NM-74 — With the squad split up, `look`, `town` and the header describe the empty middle
- Part: Text tool
- How bad: rough
- What happens: with one member in another town, the lists centre on the ground between; a lone member's surroundings, and the people they could talk to, can't be seen at all. Two members sent to Stillbank are told they've arrived under a header that still says "Stillham".
- What should happen: what's round the selected members.
- To see it: world 21, `tr21-stuck-north`: `go ne 1200`, `go t4`, `go t4`, `look people`.
- Why, if known: `play.rs` uses `squad.pos` (the same root as NM-13, NM-16 and NM-18).
- Found by: [C] [R] [I], 10 Oct

### NM-75 — `wait` takes anything, and has no limit
- Part: Odd input
- How bad: rough
- What happens: `wait abc` and bare `wait` pass ten minutes; `wait 1.5` passes two; `wait 99999999` did not return in two minutes and had to be killed (it would run for more than a day of real time once the squad is down and nothing stops it).
- What should happen: a usage line; a sane upper limit.
- To see it: any save.
- Found by: [R] [I], 10 Oct

### NM-76 — News lines come late, twice, or not at all
- Part: Text tool
- How bad: cosmetic
- What happens: "23:45 Ṭelihoḍu wakes, rested." is printed again after every command for twenty minutes; a line from one member's command turns up under the next member's; lines stamped in the same minute as the last one read never show (a `pickup` that printed nothing); a `wait 1440` shows only the last 14 lines of the day.
- To see it: world 34, `id34-rested-2005`: `wait 219`, then `wait 1` ×6.
- Why, if known: `play.rs` `news` filters on time later than the last line seen.
- Found by: [I] [R] [T], 10 Oct

### NM-77 — Flatbread in the main hand (same as RG-4), and a draught wasted on the healthy
- Part: The pack
- How bad: cosmetic
- What happens: `equip NAME N` on food, a tent or a standing torch says "Done." and the weapon goes to the pack. `dose X X` at full health uses up the draught. A torch put out stays in the off hand and the buckler it replaced stays in the pack.
- To see it: world 21, `id21-d2-0800`: `equip Ḍuhuquʻi 0`; `dose Ruqox Ruqox`.
- Found by: [I] [R], 10 Oct

## 1g. Seen in long runs and in the save files

### NM-78 — Town treasuries and grain stores grow without end
- Part: Townsfolk's lives (the economy)
- How bad: wrong outcome
- What happens: with the squad standing still, all the town treasuries together hold about 10,300 coin on day 1, 20,000 on day 10, 43,000 on day 30 and 80,000 on day 60; the richest town goes from 1,250 to 15,750. "Grain and greens" in store goes from 17,000 to 55,000 over the same time. The same on worlds 3, 21 and 34. (Unpaid guard pay, which the code reading expected to grow, stayed at 0 everywhere.)
- What should happen: something spends a treasury and eats or sells a surplus.
- To see it: `headless society 1 21`, then `10`, `30`, `60`: the "treasury" and "stock" figures.
- Why, if known: `economy.rs` `dawn`: tax comes in every day and only guards are paid from it.
- Found by: naming session, 10 Oct

### NM-79 — Two identical play-throughs give save files that differ by a few bytes
- Part: Save and load
- How bad: rough
- What happens: from one saved moment, the same commands twice: the printed output is identical line for line (about 25 such pairs across the four play-throughs), but the two save files differ in 6 to 66 bytes: a saved list (the buildings the squad has been inside) is written in a different order each time.
- What should happen: identical saves, so that "same save" can be checked by comparing files.
- To see it: `cmp -l` on `th21-free-chase.save` and `th21-free-chaseB.save`.
- Why, if known: `World::furnished` is a `HashSet` and is saved in whatever order it iterates (`world.rs` 134); the same would go for the other hash maps in `World`. Nothing was seen to depend on the order, but `view/hud.rs` 261 walks the bounty map directly, so the order bounties are listed in could change after a load.
- Found by: [T] [C], 10 Oct; cause read by the naming session

---

## 2. Read in the code, not yet seen

What is left of the code-reading candidates after the play-throughs. Likely, not certain; file names are given so a fixer can look.

**Crime and buildings**
- A chase dropped because the guard is in a fight, or the culprit has left the squad (left at a base), loses the fine altogether: no bounty (`pursuit.rs` 265–269).
- What is in a workshop's chests depends on who keeps it at the moment the squad first walks in (`containers.rs` `stock_building`).
- A knocked-out or carried townsperson can be the witness who "runs to fetch the watch" (`pursuit.rs` `spotter`).
- Casting Unlock is never witnessed.
- With nobody left in the squad every text-tool command crashes (`play.rs` 673). Nobody died in any play-through, so not reached.
- A lost duel binds the accused wherever they now are; out of town they are at once "run from their bond" (`law.rs` `duel_over`). Tried three ways and not reachable in practice: duels end in the minute they begin (NM-31).

**Jobs**
- A post plus guard work for the same member, walking back and forth and earning neither (`chances.rs` `take_opportunity` has no "already has work" check). Guard work was never offered in 19 days of play. A second *post* is refused, as it should be.
- "Get back what was stolen" pays in full without the thing being handed over (`chances.rs` `finish_opp`). Never offered.
- Unlawful jobs raise standing; lawful ones count twice (`quests.rs` `reward`, `chances.rs` `finish_opp`). None offered, and standing is shown nowhere.
- Rewards are paid without looking in the giver's purse; a purse can go negative (`chances.rs` 701–713).
- The Arbiter's post can be taken again and again and replaces the town's own arbiter (`law.rs` `take_post`).
- A job stays at "Report back" for ever if its giver dies or is recruited.

**Society and base building** (base building was not played: see section 4)
- When any town re-lays itself out at dawn, the squad's own work-shed stations are wiped (`society.rs` `place_stations`). No town changed custom in 28 days of idling, so not reached.
- A change of household custom renumbers every household in the world: a hired hand's wages go to strangers, all debts are cancelled, grudges and cons are dropped (`society.rs` `form_all_households`). Not reached, for the same reason.
- Office holders and ring members can be hired to an outpost and keep their seat (`baselife.rs` `hire_terms`).
- A base's store is one-way: only five building materials go in, nothing comes out; hired hands are "not fed" every dawn and quit; "Seal (pitch)" can never work; things a base crafter makes can't be collected.
- Residents and hands work round the clock whatever their state.
- Placement checks only town buildings: a hut can go on a town's field or market.
- Every base event is logged in the main news wherever the squad is.

---

## 3. Looked for and not found

- **Save and load.** About 25 times, a save was copied at an awkward moment and both copies were continued with the same commands: at the instant of being seen, mid-chase, mid-lock-pick, during a duel, with a talk open, two members bound over a night and a dawn, ten members with a job, a bond and a bounty over a day, one minute before midnight, before 06:00 and before pay, asleep indoors, locked in. The printed output was the same every time. (The files themselves differ a little: NM-79.)
- **Stepping.** Ten comparisons of one long wait against many short ones (`wait 60` against sixty `wait 1`, up to 1440 against 24 × 60, across midnight, dawn, the 20:00 lock-up, asleep, with a bound member): the townsfolk's places, the packs, the journal and the town panel were the same every time.
- **Long runs.** `headless 20` on worlds 3, 21 and 34: no crash, steady speed (about 63 µs a step, 110 s in all). `headless society` to 60 days on the same three: no crash, time grows in a straight line with days. What does grow: NM-51 and NM-78.
- **Day plans.** Fourteen named people followed for two days in two towns: each has a work place by day and somewhere else in the evening; guards change over at 06:00 and 20:00; nobody stood on one spot for 24 hours (except NM-60 and NM-66). The boats do go out at dawn. Merchants trade exactly when the town panel says "trading now".
- **The squad limit** holds in talk: the eleventh recruit is refused. (Past it through a base: NM-8.)
- **A second town post** is refused. **A town post does show in the journal**, with days paid.
- **A bond's buy-out** works and its price falls with days served. **A bounty is per town**, and a town that has heard of another's collects both.
- **Unpaid guard pay** stayed at 0 in every town to day 60.
- **No town changed a custom** in 28 days of idling, so the news lines for that were never seen.
- **Odd input refused cleanly:** `put 99`, `put -1`, `search k0.2.99`, `pickup g999`, `pickup 17`, `enter b0.999`, `enter` and `search` with nothing, a person id where a building is wanted and the other way round, `carry` / `loot` / `attack` / `talk` on someone asleep indoors.
- **Attacking townsfolk or the watch** can't be done at all: `attack`, harmful spells and a harmful scroll are refused on anyone who isn't already an enemy. So nothing a town does about assault or killing could be reached (and that part of `law.rs` looks unreachable today).

**Could not be looked at through the text tool:** the stilt villages and the tide (nothing in `look`, `town` or `map` mentions either); the town panel's treasury, unrest and "Lately" (window only: see RG-7); a bed as such (the tool has only `rest`).

---

## 4. What has been tried

Each bullet in the plan, and in which kinds of play: **T** thief (worlds 21, 34, and 3 by the naming session), **R** reckless (34, 3), **C** careful trader (21, 3), **I** idler (34, 21, and 3 by the naming session).

| Bullet in the plan | Tried in | Left |
|---|---|---|
| Buildings: entering and leaving | T R C I | — |
| Buildings: locked doors | T R I | — |
| Buildings: picking locks, by day and at night | T R | — |
| Buildings: sleeping in beds | T I (on the floor of homes; the squad beds down by itself at 22:00) | a rented inn bed and the healer (new on `main` since `fbc0b4e`): not tried |
| Containers: search, take, put, stealing | T R I | — |
| Law: witnesses, the watch, chases | T R | — |
| Law: bounties, arrest, judging | T R (duel, elders, the record, shunning: all four customs) | — |
| Law: fines, bonds, duels, shunning | T R I | — |
| Law: running away, coming back, days later | T R | — |
| Lives: day plans, who is at work | C I | — |
| Lives: merchants' hours | C I | — |
| Lives: gossip, grudges, rumours | C I (no grudge between townsfolk ever came up) | — |
| Lives: stilt villages and tides | C I (boats at dawn only; the tool shows nothing else) | screenshots of the window (`GAHT_SOCIETY=tides`): not done |
| Jobs: taking a job, being paid, quitting | C R | guard work and "get it back" jobs were never offered |
| Recruiting: up to 10, firing, a recruit's food | C R (no way to fire anyone exists) | — |
| Base building: placing, sites finishing | by test (NM-1 to NM-3); a first try in play (an L-shaped palisade laid 4 of its 6 pieces) | **not played.** `build` and `base` commands for the text tool are on `hunt/naming`, ready for it |
| Base building: hired hands, wages, quits | by test (NM-6) | **not played** |
| Base building: members left at a base | by test (NM-4, NM-5, NM-7, NM-8) | **not played** |
| Save and load at awkward moments | T R C I | with a base |
| Long runs | worlds 3, 21, 34: `headless 20`, `headless society` to 60 days | — |
| Odd input | R I T | — |
