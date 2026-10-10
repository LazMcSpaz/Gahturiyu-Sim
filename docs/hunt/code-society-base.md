## message
All findings are from reading only; nothing was run. Paths are under /home/claude/gahturiyu-craft.

## How base building can be reached

- **Window, Build panel:** key B (src/view/app.rs:651) or the Build button on the bottom band (app.rs:1280). Drawn by `baseui::build_panel` (src/view/baseui.rs:68). Its actions are handled at app.rs:1364-1416:
  - `Pick(def)` starts placing.
  - `Store(bid)` calls `World::store_materials`.
  - `Deconstruct(bid,id)` calls `World::deconstruct`. The × is drawn only for sites and ruins, so a standing building cannot be removed from the UI.
  - `Leave(pid,bid)` calls `leave_at_base`.
  - `PickUp(pid)` calls `pick_up`, which for a hired hand calls `dismiss` (the button reads "Let go").
  - `NextJob(pid)` calls `set_base_job` with the next entry of `baselife::JOBS` (Idle, Builder, Farmer, Cook, Crafter, Hauler, Guard).
  - `NextRecipe(pid)` calls `base_recipes` / `set_base_recipe`.
  - `Seal(bid,id)` calls `seal_building`.
- **Window, placing:** `baseui::update_ghost` (baseui.rs:225) calls `World::check_place` and `World::wall_plans`. Shift+wheel turns the ghost. A click goes to `place_click` (app.rs:877), which calls:
  - `found_base(at)` for the camp marker;
  - `place_wall(def, &[last, to])` for walls, one two-point call per click;
  - `place_building(plan)` for everything else.
- **Hiring:** the dialogue topic `Topic::Hire(wage)` is offered when `hire_terms(npc)` is Some, which needs at least one base (src/sim/dialogue.rs:459). Choosing it calls `World::hire(npc)` (dialogue.rs:729). It is reachable from play.rs with `talk pID` then `say N`, but only if the save already holds a base.
- **Flags:**
  - `GAHT_BUILD=1` runs `shot::demo_base(world, false)` (src/view/shot.rs:995). It founds a base in the wilds 80 m or more from the start and places two huts, a lean-to, a field plot, a straight 36 m palisade and a gate. It teaches everyone Carpentry, gives the hunter 60 timber, 16 seareed and 1 ingot, stores them and steps 7 h. The window opens the Build panel with a hut ghost.
  - `GAHT_BUILD=base` does the same, then pushes 10 flatbread straight into the store, leaves members[1] as Farmer and members[2] as Builder, gives the lead 120 coin, hires the nearest willing townsperson as Hauler, runs 30 h and opens the Base tab.
  - `GAHT_HOURS` runs after the demo is built. `GAHT_HOVER=x,y` places the ghost.
- **No window:** headless.rs and play.rs have no base commands, and the window saves only on F8 (`GAHT_SAVE` is headless only). The practical route is a Rust test on the public API:
  - `found_base`, `check_place`, `place_building`, `place_wall`, `wall_plans`, `deconstruct`, `store_materials`;
  - `leave_at_base`, `pick_up`, `dismiss`, `set_base_job`, `base_recipes`, `set_base_recipe`, `seal_building`, `hire_terms`, `hire`;
  - `Base::add_to_store`, plus `World::save_to(path)` to hand a base world to play.rs.
  - tests/base/mod.rs is included by tests/consistency.rs and has `with_base(seed)`, `carpenter`, `supply`, and private `outpost()`, `willing()`, `run_until`.

## Candidate bugs, most serious first

**1. Hiring the first hand a day or more after founding a base rewinds the base's clock: sites get free labour and finish in the past.**
- Where: src/sim/base.rs `next_base_event` 968-971, `base_events` 978-1038, `base_changed_at` 852-928 (`s.since = t` at 908, `*hp_at = t` at 913), `found_base` 690; src/sim/baselife.rs `base_dawn` 634.
- Cause: `dawn_done` is set at founding and only advances while someone with a `Hire` lives there. On the first hire, every missed dawn is offered as due, one by one. Each calls `base_changed_at(bid, past dawn)`, which sets every site's `since` and every building's `hp_at` back to that dawn while keeping the labour and health already counted.
- Scenario: `with_base(1)`, run 72 h, supply and place a hut with the carpenter standing there, `hire(willing)`, `step(1.0)`.
- Expect: the hut is Standing with `stood_at()` about 40 h, though it was placed at about 79 h. The journal line "Hut stands…" is dated days ago. Builders are credited the phantom hours as practice, and thatch loses the lag's rot a second time.
- Should be: nothing changes at hire; the hut stands 8 h/pace after placing.
- Also happens after any spell with no hands, then a new hire. `GAHT_BUILD=base` does not show it because it hires the same day.
- Confidence: read it, sure. tests/base hires 49 h after founding, but only with everything already standing, so only the extra rot occurs there.

**2. A base's store is one-way: only five building materials go in, and nothing comes out.**
- Where: base.rs `store_materials` 806-842 (keys are only what `BUILDINGS.needs` name: timber, seareed, rock, clay, iron_ingot); src/sim/condition.rs `food_for` 436-440; baselife.rs `base_dawn` 646-667, `seal_building` 288-302. No function moves store contents to a pack (grep: `add_to_store` is called only from base.rs, baselife.rs, shot.rs and tests).
- Consequences in real play:
  - A member left at a base eats only from the store and ignores bread in their own pack, so they starve unless a field, kitchen and cook exist.
  - Hired hands are "not fed" every dawn (−0.10 loyalty) and quit on about the fifth dawn even when paid.
  - The store never holds coin, so the panel's "from the store's N coin" is always 0.
  - Pitch cannot be stored and there is no pitch recipe, so "Seal (pitch)" always fails with "no pitch in the store".
  - Weapons and armour a base crafter makes can never be collected.
- Scenario: `leave_at_base` a member carrying flatbread at a base with an empty store and run 3 days: hunger climbs to starving with bread still in the pack.
- Confidence: read it, sure. It may be unbuilt rather than broken, but the test and the demo both push food in directly.

**3. The farmer ignores store room, and the cook checks room before its inputs come out, so a farmer+cook+hauler base jams and starves beside a full granary.**
- Where: baselife.rs `start_cycle` 316 (farmer: no `room_for`), 317-323 (cook), 324-327 (hauler), `finish_cycle` 370-373.
- Cause: the hauler fills the store to within 8 kg of capacity with timber and rock that little uses. Each harvest adds 6 kg regardless of room. The cook needs 1.2 kg free before it removes 4.5 kg of grain and timber. Once the load is above capacity − 1.2 at a harvest, the cook never starts again, and grain grows 6 a day without limit.
- Scenario: the tests' `outpost()` (capacity 160 kg, farmer, cook, hauler), run 14 days, print the store and `hunger_of`.
- Expect: by about day 3-5 flatbread is 0, grain climbs, load is above capacity, the panel says the cook "needs the kitchen, grain and timber", and the residents starve.
- Should be: the cook bakes, since baking frees room.
- Confidence: likely; worked through by hand with the item weights, not run.

**4. Walls cannot turn a corner.**
- Where: base.rs `overlap` 303-320, `check_ground` 657-675 (only tower and wall are exempt from each other), `wall_plans` 721-740, `place_wall` 744-761; app.rs 889-894.
- Cause: at any bend over about 2°, the next leg's first segment (0.6 m thick, 1.0 for rubble) overlaps the previous leg's last one. It is refused as `Overlaps` and silently skipped.
- Scenario: `place_wall(palisade, &[a, a+(12,0), a+(12,12)])`.
- Expect: 5 ids, not 6. A second leg of 4 m or less returns Err outright. Closing a loop also loses the closing segment, so an enclosure always has gaps. In the window the ghost's first segment after a corner is red ("in the way of another building").
- Confidence: read it, sure (geometry by hand). Tests and the demo use straight walls only.

**5. Any town being laid out anew wipes the squad's work-shed stations.**
- Where: src/sim/society.rs `place_stations` 790-802 (`self.stations.clear()`, then only town stations are re-added), called from `dawn_changes` 1146-1149; bases add theirs at base.rs:1000.
- Effect: the squad can no longer craft at its own forge or bench (src/sim/crafting.rs:433 reads `World::stations`), and the station is no longer drawn. Resident crafters are unaffected because they use the building. It is never restored.
- Trigger: a dawn when any community's layout, cooking or own-institution custom changes, marked by the journal lines "…goes over to…" or "…now cooks…". Rare; recruiting or hiring from a small community can cause it (re-blend at a 10% head-count shift, src/sim/culture.rs:360).
- Confidence: sure of the mechanism; I could not force the trigger.

**6. A change of household custom renumbers every household in the world, and a hand's stored household number goes stale.**
- Where: society.rs `form_all_households` 472-554, called from `dawn_changes` 1143-1145; baselife.rs `Hire.household` 484, `base_dawn` 657-659, `dismiss` 235-237, `hand_leaves` 603-608.
- Effect on hands: wages go to an unrelated household, the hand "goes home" into the wrong one, or the index is past the end and the game panics at dawn.
- World-wide side effects of the same call: every household-to-household debt is cancelled with the lender credited in full from nowhere (477-496, 520-527); all Grudge, Con, DodgeDebt and Steal storylines are dropped and every ring's `paying` is cleared (546-550).
- Trigger: the journal line "…lives by … now". Rare.
- Confidence: sure of the mechanism; I did not check which other state holds household numbers (law, chances).

**7. Residents' meals are not on the base's timeline, so results depend on step size when food or room is tight.**
- Where: condition.rs `update_conditions` 478-584 runs after the event loop (src/sim/world.rs:468) and takes food from the store as it stands at the end of the step, back-dated; base events ran earlier in the step.
- Scenario: a store with exactly 2 flatbread, a resident due to eat at 05:55 and a hand's dawn meal at 06:00. With 60 s steps the resident eats one and the hand is "not fed". With one 1 h step the hand takes both and the resident finds none.
- Also: a meal frees room but is not a base change, so a blocked hauler or cook is not re-checked until an unrelated event.
- Breaks rules 1 and 22. The existing tests pass because food is plentiful.
- Confidence: likely.

**8. Free labour by dismissing before dawn.**
- Where: baselife.rs `dismiss` 228-242 pays only `owed` (unpaid from earlier dawns); wages are charged only at 06:00 to hands already arrived (639). `hand_leaves` sends them home with no ill will, and as a Labourer `hire_terms` takes them straight back, at a labourer's wage.
- Scenario: hire at 07:00, "Let go" at 05:59 next day, re-hire: 23 h of hauling, farming or cooking for 0 coin, repeatable.
- Side effect: a carpenter or mason loses the trade on the first dismissal (job becomes Labourer, 601), so they cannot build when re-hired.
- Confidence: read it, sure.

**9. Stepping the job button through "Cook" eats materials.**
- Where: baselife.rs `set_base_job` 245-260 (clears the round, then `base_changed` starts the new job's round at once, taking its inputs); app.rs 1393-1399 (the only control is "next job").
- Scenario: a base with a free kitchen, grain and timber; click a resident from Farmer round to Hauler: 2 grain and 1 timber vanish each pass through Cook. A farmer's 24 h round also restarts.
- Confidence: read it, sure.

**10. Office holders and ring members can be hired, and stay in office or in the ring while living at the base.**
- Where: baselife.rs `hire_terms` 531-548 lacks the `holds_office`, ring and `is_bonded(now)` checks that `recruit::join_terms` has (src/sim/recruit.rs:44). `hire` leaves `people.home` and `dwelling` unchanged.
- Effect: `law::seated` (src/sim/law.rs:355) still passes, ring.rs:143 does not drop them, they still count as living in their old home for chest ownership (src/sim/containers.rs:235), and a storyline they are in keeps acting (src/sim/stories.rs:487).
- Scenario: find an arbiter or councillor with wanderlust > 0.6 or money need > 0.4 and hire them; the town panel still lists them.
- Confidence: likely; I did not read law.rs or ring.rs beyond those lines.

**11. A gate often has no wall segment it fits.**
- Where: base.rs `check_place` 563-574 (needs a segment ≥ 3.49 m) against `wall_plans` 730 (segment = len / ceil(len/4)).
- Effect: legs under 28 m fit a gate only at lengths in 3.49-4, 6.98-8, 10.47-12, 13.96-16 and so on. A 10 m wall gives 3 × 3.33 m and the gate says "must sit on a wall". A gate also cannot go on a segment that carries a tower (Overlaps).
- Confidence: read it, sure (arithmetic).

**12. Placement checks only town buildings.**
- Where: base.rs `check_ground` 602-677.
- Not checked: town workplaces (fields, market, yards, kitchens in `society.towns[].places`), bandit camps and stashes, ruins, deposits, pens, standing torches, gather nodes. A camp marker can also go in the middle of a town between homes (`Land::Unclaimed`, by design until Stage 4).
- Scenario: found a base beside a town's Fields and place a hut on `places[k].pos`.
- Confidence: sure the checks are absent; whether that is wanted is a design call.

**13. Residents and hands work round the clock whatever their state.**
- Where: baselife.rs `start_cycle` 310 (checks only dead and on-the-way); base.rs 875 (resident builders are not checked for knocked out, unlike `present` at 1049).
- Effect: a starved, knocked-out or sleeping resident keeps hauling, farming and building, 24 h a day (a hauler brings 16 timber and 8 rock daily). A round at a building that is deconstructed or falls in still pays out.
- Confidence: read it, sure; partly a design gap.

**14. Too much telling.**
- Every base event goes to the main journal wherever the squad is: "X took iron ingot, coin on the way out", "X quits…", "Hut stands…" (base.rs `base_note` 1066-1078 with loud=true; baselife.rs 700, 615).
- society.rs:1132 logs every custom change in every town in the world, with no distance check.
- society.rs:1166 names each stranger who takes a post at dawn in any town within 2.5 km.
- The town panel's "Lately" (src/view/townui.rs:296-311) lists the town's real record with names and minute, not what the squad has heard. The panel also gives exact treasury, unrest out of 100, merchants' coin and every stock figure (139-171).
- Confidence: sure the lines exist. townui.rs was only skimmed.

**15. Town treasuries and unpaid guard pay have no bound.**
- Where: src/sim/economy.rs `dawn` 315-319. Tax is 0.6 × heads × (0.5..1) a day; guards cost 12 each, at least 2.
- Effect: a town under 40 people can never cover its two guards, so `owed` grows forever and the watch is on 40% pay permanently (src/sim/lives.rs:369). A prosperous town over about 60 gains treasury forever with no sink but squad contracts.
- Check: `headless society 30` against `300` and compare treasury and owed.
- Confidence: likely. economy.rs was outside my list and only its dawn was read.

## Smaller things noticed, not ranked

- `leave_at_base` counts dead members as "someone to travel" (baselife.rs:178). Leaving the last living member strands everyone, since `pick_up` needs a living member present.
- `base_bed` (430-444) orders all residents before the squad, including hands still on the road. `base_dawn` (636-638) gives beds to the squad first. A hand can be docked for "no bed" while its slot in `base_bed` goes unused.
- A hand who quits unpaid is recorded as the wrongdoer (`note(WorkQuarrel/Theft, actor = hand)`, baselife.rs 702-706). Gossip sours the town on the hand; the squad suffers nothing.
- `hand_steals` adds gear without `recompute_might` (rule 5).
- Day plans can drop the walk home and bedtime when the evening starts after bed (routine.rs `Plan::push` 109-127 with 443-446). Examples: a market-day merchant, or a late winter worker with garden, market and visit. They stay at the evening spot until midnight and `is_indoors_asleep` is false.
- A base founded between 00:00 and 06:00 skips that day's dawn tally.

## Coverage

- **Read fully:** CLAUDE.md; sim/base.rs, baselife.rs, society.rs, routine.rs, tide.rs, history.rs, memory.rs, lives.rs, culture.rs, world.rs, few.rs, progress.rs; view/baseui.rs; tests/base/mod.rs.
- **Read in part:** economy.rs (60-460); condition.rs (380-640); encounters.rs (1-200); recruit.rs; squad.rs (add/remove); view/app.rs (640-700, 868-912, and grep hits for 1315-1416); view/shot.rs (flag wiring and `demo_base`); view/townui.rs (284-343).
- **Grep hits only:** law.rs, ring.rs, stories.rs, chances.rs, dialogue.rs, making.rs, wear.rs, containers.rs, crafting.rs, items.rs, jobs.rs, play.rs (help text).
- **Not reached:** talk.rs, labour.rs, save.rs bodies, view/interact.rs, data/lines.
- **Clean in what I read closely:** no HashMap or shared running RNG in my files; history, tidings, memories, feelings and the base log are all capped; tide and season maths have no bad edges.