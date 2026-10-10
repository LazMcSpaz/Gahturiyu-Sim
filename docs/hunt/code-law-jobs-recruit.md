## message
All paths are under /home/claude/gahturiyu-craft. Nothing was run or edited; every item is from reading only.

**Coverage**
- Read fully: CLAUDE.md, src/sim/law.rs, recruit.rs, chances.rs, quests.rs, dialogue.rs, pursuit.rs, squad.rs, fights.rs, news.rs, src/bin/play.rs.
- Skimmed (only the parts named): jobs.rs (outline, `pay`, `is_post`), world.rs (`step`, `person_pos`), economy.rs (dawn order, `shelves`, `buy_at`), routine.rs (`day_plan` head, `at_work`), society.rs (`posts_for`, `POST_ORDER`, `assign_jobs`), lives.rs (`place_jobless`, `holds_office`), memory.rs (grudge and `act` paths), stories.rs (`ask_for_help`, `story_hour`), baselife.rs (`leave_at_base`, `pick_up`, `dismiss`, `hire_terms`, `hire`), combat.rs (`winner`, end of fight), inventory.rs (`add`, `take`).
- Not read: talk.rs, containers.rs and loot.rs (how theft is detected), condition.rs (hunger of a bound member), all of src/view except a grep, tests.

**Reachability note for the verifier:** `judge` is only reached through `pursuit::caught` (a guard catching a thief or lock-picker). Fights can only start against hostile bands (`attack` needs `g.hostile`), so the assault and murder branch of `law::fight_wrongs` (law.rs 1149-1205) looks unreachable today. I did not chase bugs inside it.

1. **Play tool: a conversation stays open while the world runs, so `say N` works from any distance and hours later**
   - Where: src/bin/play.rs `pass` (621-667), `"say"` (758-773); src/sim/dialogue.rs `topics` (321), `ask` (482); src/sim/recruit.rs `recruit` (102-104).
   - Why: nothing in the sim ever closes `World::talk` except Goodbye and `end_talk`. The window freezes time while talking (view/app.rs 837-838); play.rs does not. `order_members` clears only `want_talk`.
   - Scenario: `talk pN` with a merchant or a "restless" local, do not `bye`, then `go n 2000` (or `wait 600`), then `look` (still shows "TALKING with …"), then `say <number>`.
   - Expect: Buy and Sell still trade if the merchant is at their stall. Join recruits them and places them beside the talker 2 km away (`squad.add(npc, here + off)`, no distance check). PayBounty, BuyOut, Report, PostWork and QuitWork all still work.
   - Should: the talk ends when the two part or time passes, or play.rs refuses to step with a talk open.
   - Confidence: read it, sure of the mechanism. Not checked: whether each topic's own gate (`is_trading`, `at_work`) still passes after the walk.

2. **Play tool crashes on a person id that doesn't exist, and on every command once the squad is empty**
   - Where: play.rs 673 (`w.squad.members[0]`), 749-751 (talk), 738-740 (attack), 783-786 (loot), 964-965 (carry), 864-866 (cast), 458 (`unwrap_or(w.squad.members[0])` is evaluated eagerly); dialogue.rs 279 (`self.people[npc as usize]`); fights.rs 91 (`self.group_of[enemy as usize]`); world.rs 600.
   - Scenario A: `talk p4000000` (PersonId is u32, so it parses). Same with `attack p4000000` when not in a fight, `loot p4000000`, `cast NAME 1 p4000000`.
   - Scenario B: every squad member dies (fights.rs 574 empties `squad.members`), then any command including `look`.
   - Expect: index-out-of-bounds panic. Should: "Can't talk to them." / a game-over line.
   - Confidence: read it, sure for talk, attack, loot and the empty squad. Likely for carry (`can_carry` body past line 100 not read).

3. **A member can hold two jobs at once and then walks back and forth between them, earning neither**
   - Where: chances.rs `take_opportunity` (333-370) has no "already has work" check; `take_post_work` (398) does. `work_contracts` (446-461). dialogue.rs 450-454 hides PostWork when a contract exists, but Accept (683) and TakeJob (708) are not gated.
   - Scenario: member takes a town post at the hall ("I'll work as labourer here"). Same member then accepts guard work from a robbed crafter or merchant ("Any work?" → "I'll do it."), or takes one off the Board. From the next day, watch them between 08:00 and 17:00.
   - Expect: on arriving at place A they count as idle for contract B and are sent to B, and back again. Presence only accrues in the last 14 m of each approach, so both count as missed. After three misses the post vanishes silently and the guard job fails ("X let Y down", −0.4 memory). If the two places are within about 28 m, both pay.
   - Should: refuse the second job, or serve one.
   - Also: `contract_of` returns only the first contract, and QuitWork ends both.
   - Confidence: sure the check is missing; the ping-pong is likely (traced by hand).

4. **"Get back what was stolen" pays in full without the thing being handed back, and the thing is conjured**
   - Where: chances.rs `press` Recover (601-611), `finish_opp` (724-728); quests.rs `reward` (122-135).
   - Scenario: take a Recover job, press the thief until they give way (item added to the talker's pack). Sell or drop the item. Talk to the asker → "About that job...".
   - Expect: full reward, standing and thanks; `finish_opp` only takes the item if the squad happens to hold one of that kind, else nothing. Any other item of the same kind satisfies it too.
   - Also: `press` adds a fresh plain item (`gear.add(it, 1)`); nothing leaves the thief, and the stolen `piece` (maker's mark, wear) is lost.
   - Should: Report requires the item.
   - Confidence: read it, sure.

5. **The squad limit of 10 is only checked when recruiting; picking members up from a base goes past it**
   - Where: recruit.rs 15, 37 (`squad.members.len() >= MAX_SQUAD`, the travelling squad only); baselife.rs `pick_up` (204-224) calls `squad.add` with no limit.
   - Scenario: recruit to 10, leave 9 at an outpost, recruit 9 more, go back and pick the 9 up. Travelling squad is 19.
   - Also: there is no way to dismiss a recruit (`dismiss` returns "one of the squad"); leaving them at a base is the only removal.
   - Confidence: read it, sure. Needs a base to verify (`GAHT_BUILD=base` in the window; I saw no base commands in play.rs).

6. **A member with a town job walks off to it alone at shift start from any distance, with no message; a post is lost silently after three missed days ever**
   - Where: chances.rs `work_contracts` (449-461): `c.sent_day != day` re-routes the member whatever they were doing; no distance limit, no log line. `dawn_contracts` (507-514): `missed` never resets, and a post (`opp: None`) ends with no `say`.
   - Scenario: one member takes "I'll work as labourer here", then the whole squad walks to another town. At 08:00 next day (`wait`), that member turns round and walks back alone. In shift hours, each time they stand idle anywhere else they set off for the post again.
   - Expect: silent departure; later the post is gone with no line.
   - Should: at least a line, and probably no auto-walk when far from the town.
   - Related: quitting after a full day's shift but before dawn forfeits that day's pay (pay is only at the dawn after). With an empty treasury or purse the line reads "X is paid 0 coin for a day's work."
   - Confidence: read it, sure of the code path; whether the auto-walk at distance is intended is the designer's call.

7. **Play tool: `talk` always sends the first selected member (member 0 if none), even when bound, down or asleep, and the stale talk order then blocks their job**
   - Where: play.rs 673 (`lead`), 751 (`order_talk(lead, p)`); dialogue.rs `order_talk` (278-291) has no `free_to_order` or `is_down` check; law.rs `hold_the_bound` (1065-1081) pulls a bound member back each step; chances.rs 449 skips a contract while `want_talk` names the member.
   - Scenario A: member 0 is bound (or knocked out), nothing selected, `talk pN` → after 10 minutes "They're not talking (or you couldn't reach them)." Nobody else is tried (`loot` picks the nearest; `talk` doesn't).
   - Scenario B: `want_talk` stays set after a failed `talk` (cleared only by a move order to that member or by the talk opening). A member with a contract and a stale `want_talk` gets no presence counted even standing at their post, so days are missed. Later the talk can open by itself when the two happen to pass.
   - Confidence: likely. Not checked: how the window picks the talker.

8. **Notes (50 coin each) don't count when a fine, bounty, fee or buy-out falls due, though the play HUD counts them**
   - Where: law.rs `pay_or_bond` (1009-1020) and `buy_out` (1035) use `squad_count(coin)` only; dialogue.rs PayBounty (770); recruit.rs 70; play.rs 346 (`coin + 50 × note`).
   - Scenario: change most coin for notes at an exchanger, then get caught stealing in an elder-judgement town.
   - Expect: "a fine of 40", then bound for days although the HUD says "300 coin between them". PayBounty says "You've got 0."
   - Should: notes pay, or the HUD stops counting them.
   - Confidence: sure of the mechanism; whether notes are meant to be unspendable is a design question (shops also take coin only, economy.rs 626).

9. **A lost duel binds the accused wherever they now are; if they've left town they are at once "run from their bond"**
   - Where: law.rs `start_duel` (1105-1127: champion is the strongest fit member within 30 m, the accused is not in the fight and stays orderable), `duel_over` (1130-1143), `pay_or_bond`, `check_runaways` (1084-1101).
   - Scenario: steal and be caught in a duel-custom town, carrying little coin, with a stronger squadmate within 30 m. While they fight, `select` the accused and `go` more than the town's reach + 150 m. Let the duel be lost.
   - Expect: "must pay N", then "bound … for D days", and in the same step "has run from their bond": +60 bounty, −15 standing, +1 record. The 1.5× fine is never paid.
   - Related, same code:
     - The town's champion is taken from anywhere in town (asleep, on night watch) and placed 4 m east of the accused (fights.rs 316-327).
     - With nobody fit within 30 m, a knocked-out accused "fights" and loses at once.
     - A duel that times out or ends with both down counts as lost.
     - A bandit attack during a duel starts a second fight using `squad_fit()`, which doesn't exclude the duellist (fights.rs 235-243), so one person is in two battles.
   - Confidence: likely for the main scenario; the last bullet is a guess.

10. **Getting away is always cheaper than being caught, and a bounty can be paid to any resident**
    - Where: pursuit.rs 287-292 (slipped: bounty = the plain fine); law.rs 1016 (bond capped at 30 days), 1095 (running costs a flat 60), 1028 (buy-out up to 240); dialogue.rs 342-346 (PayBounty offered by anyone whose home town has heard, not only officials or guards) and 766-781.
    - Scenario: steal, get seen, walk out past the town edge + 150 m ("has slipped the watch"), walk back, talk to any farmer → "Pay my bounty".
    - Expect: pay exactly the fine, get +2 standing for paying, no duel or bond risk. Caught in an elder town you pay the same fine with no standing back and risk a bond.
    - Also: fine and bounty coin just vanishes (not to the treasury or the victim). Standing drops by fine/4 the moment the witness tells (pursuit.rs 169) and is not restored if you later win the duel.
    - Confidence: sure of the numbers; intent is a guess.

11. **Unlawful jobs raise standing, and lawful ones count twice**
    - Where: quests.rs `reward` (138-141: +5 in the giver's town, unconditional, plus +20 regard); chances.rs `finish_opp` (697-699: +5 only if `o.legal`).
    - Scenario: do "a quiet killing" or "put a scare into someone" and report.
    - Expect: +5 standing in that town for the unlawful job; +10 for any lawful one. Five lawful jobs reach COUNCIL (50), which opens government posts.
    - Confidence: read it, sure; which number was intended I can't tell.

12. **Rewards are paid without looking in the giver's purse, and a guard wage can come from the wrong pocket**
    - Where: quests.rs 122-128 (coin handed over first); chances.rs 701-713 (`purse.coin -= reward`, unchecked, can go negative; same for a ring's purse); chances.rs 487-499 (if the asker has no household the guard wage comes from the town treasury); `can_reward` (265: reward raised to at least 15 even when `spare` is less).
    - Scenario for the treasury case: take guard work from X, then recruit X ("Come with us") or hire X to the outpost; their household becomes None. Each dawn's wage comes from the treasury.
    - Scenario for the negative purse: accept a job from a poor household, wait several days, then report.
    - Also: a job done but not reported for 10 days sets the opportunity Lapsed but leaves the quest at "Report back"; reporting later still pays.
    - Confidence: sure of the code; how often a purse actually goes negative is unchecked.

13. **Hiring to the outpost lacks the checks recruiting has: a sitting elder, priestess, arbiter or ring member can be hired, and keeps the seat**
    - Where: baselife.rs `hire_terms` (531-548) versus recruit.rs `join_terms` (44); law.rs `seated` (355-358).
    - Why: `hire_terms` has no `holds_office`, no ring check, and uses `mind.work == Bonded` (set at dawn) rather than `is_bonded`. `hire` removes them from `residents` but leaves `home`, so `seated` stays true and they hold office from the outpost.
    - Scenario: with a base, talk to an officeholder who is poor or has high wanderlust; "Come and work at my outpost" appears. Hire them and check the town panel next dawn.
    - Also: someone bound that same day (after the dawn tally) is hireable until the next dawn.
    - Confidence: sure the checks are missing; the kept seat is likely.

14. **Too much telling, and chatter, in this area**
    - pursuit.rs 161-165: "{line} {name} saw it, but says nothing." names the witness and reports a private choice the squad couldn't know, as a log line and an alert.
    - squad.rs 190-197: "X is bound to work off a bond and can't leave." is logged again on every whole-squad move order unless it is still the newest line; with one member bound this repeats for the whole bond.
    - chances.rs 506: "X is paid N coin for a day's work." per member per dawn (ten labourers give ten lines in a 14-line log).
    - law.rs 788: the bond line carries a tutorial sentence ("An official there can sell you the bond.").
    - dialogue.rs 623-626: "Latest rumours" can quote `stats.ambushes`, the whole world's ambush count.
    - play.rs 136-138 plus `news`: every alert prints twice ("!! line" and again under News).
    - play.rs 401-405 and view/hud.rs 302: every local within 70 m is tagged "restless: might join for N coin" before anyone has spoken to them.
    - play.rs 1202-1225: `town` gives the nearest town's exact trade counts and merchants' hours from any distance.
    - Confidence: sure the lines exist; which ones the designer wants gone is his call.

15. **Play tool hides the state of a bond or a job, and can drop confirmation lines**
    - Where: play.rs `status` (292-329), `"journal"` (1161-1169), `news` (503-514).
    - Bond: a bound member shows as "standing"; days left and buy-out price appear nowhere (the window shows them, view/hud.rs 394). A town post makes no journal entry (`take_post_work` creates no Quest), and hours and missed days are not shown.
    - News: the filter is `t > log_seen`. A line stamped at the same instant as the last one read is never shown. Example: `look`, then `say N` (Accept gives "New job: …", Join gives "… joins the squad.", TakePost gives "… takes up the post …") with no time passing, then `look` again. This only bites when the newest line already read carries exactly the current time.
    - Confidence: sure for the missing displays; likely for the dropped lines.

**Smaller things, unranked, each from reading only**
- Shunning as a judgement has no fine at all, lasts 6 days, and only stops the convicted member from trading with that community's merchants (economy.rs 447-451); another squad member trades freely.
- "I'll work as priest here" has no eligibility check (chances.rs `vacant_posts` and `take_post_work`), while NPC priests follow the canon rule (society.rs `assign_jobs`). Labourer is always offered and never counted, so every member can take it in the same town. The town may also fill the same post with a local while the squad's contract goes on being paid.
- A bound member cannot fetch food. I did not read condition.rs to confirm they starve; a squadmate can still hand them food (`order_give` only checks the giver).
- A pursuit is dropped with no bounty if the culprit leaves the travelling squad mid-chase (pursuit.rs 265-269: left at a base, or dead).
- `take_post` (law.rs 938-972): Arbiter can be re-taken repeatedly (no "already holds it"), silently replaces the NPC arbiter, and ignores `fallen`. The TakePost topic shows even where the town has no such chamber.
- A guard job with up to two missed days still finishes as done with +5 standing (chances.rs 517-524).
- A quest stays at "Report back" forever if the giver dies or is recruited (`order_talk` refuses the dead and squad members).
- `squad_count` sums into a u16 (quests.rs 104) and `Gear::add` does `e.1 += n` on a u16: above 65,535 coin this wraps in release and panics in a dev build. Unlikely in normal play.
- `dialogue::disposition` (262-266) reads race directly (+10 same people, +6 Roduro–Horaro); this may sit against rule 15.
- Taking a camp job after already beating that camp completes it on the next step (quests.rs 87).