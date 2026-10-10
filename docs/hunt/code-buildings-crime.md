## message
Code reading only; nothing was run, edited or built. Tree: /home/claude/gahturiyu-craft.

**Coverage**
- Read fully: CLAUDE.md, src/sim/buildings.rs, containers.rs, pursuit.rs, news.rs, stealth.rs, loot.rs, squad.rs, src/bin/play.rs.
- Read in part: condition.rs (84–741), law.rs (960–1215 plus a function outline), world.rs (60–390, 560–790), routine.rs (451–626), inventory.rs (1–260), ruins.rs (130–246), casting.rs (545–604), torch.rs (lights only), and short snippets of quests.rs, carry.rs, fights.rs, labour.rs, recruit.rs, society.rs (workplace placement), layout.rs (variant table, a few helpers).
- Not read: src/view/*, tests/*, law.rs `dawn_law` and bonds, encounters.rs, most of layout.rs geometry, `day_plan` itself.

**Candidates, most serious first**

1. **Stealing indoors by day is never seen: awake townsfolk are always placed outdoors**
   - Where: pursuit.rs `spotter` 78–106 (same-building test at line 92); routine.rs `spot_pos` 511–537.
   - Why: an awake resident "at home" stands in the yard at `b.size*0.6+2 .. +9` m from the centre, and building half-extents are at most 0.5×size. Workplaces are laid out clear of buildings (society.rs `lay_out`, the `clear` closure). Only sleepers (`around(b.pos, 0, b.size*0.25)`) are inside.
   - Scenario: at midday, `enter bT.N` on a Roduro home with residents, then `search kT.N.S` on an unlocked crate or cupboard. The header prints the chance of being seen.
   - Expect: 0% by day, and `takeall` gives no crime. The same chest at about 23:00 with a sleeper within 6 m shows roughly 10–28%.
   - Should be: daytime burglary with the owners at the door is the riskier case.
   - Confidence: likely. Not checked: `day_plan`. Two exceptions I saw: a Stone Tender on rounds stands at `b.pos + (0.55*size, 0)`, which can fall inside a rotated home; and yard spots can fall inside the corners of very large buildings (temple, size 40).

2. **A door's lock is checked only when the route is written, so a walk that ends after 20:00 goes straight in**
   - Where: buildings.rs `route` 322–325 (the only check); squad.rs `walk_squad` 276–303 follows stored waypoints with no re-check; play.rs `enter` 952–958.
   - Scenario: find a home that `look` shows as "(locked: …)" at night. Next day at about 19:57, from 300 m or more away, `enter bT.N` (or `go` to a point inside it).
   - Expect: the member is reported inside after 20:00 while `look` lists the door as locked; no pick, no trespass.
   - Should be: stop at the door.
   - Related: labour.rs `send` 179–189 pushes the raw target when the route "stops short", so `order_loot` on a body inside a locked building walks through the wall.
   - Confidence: read it, sure.

3. **Sleeping or knocked-out members can still take, put, pick up and pick locks**
   - Where: loot.rs `source_now` 112–122, `take_from` 161, `put_in` 222; squad.rs `do_pickups` 683–717; buildings.rs `do_picking` 440–528 (checks `fighting` only).
   - Scenario A: `search k…` to open a chest, `rest`, `wait 1` (status "asleep"), `take 1`. Expect "Taken." and the item in their pack.
   - Scenario B: a member is out cold beside a ground item; `pickup gN` with them nearest. `order_pickup` calls `do_pickups` at once and the unconscious member "picks up" the item.
   - Should be: refused.
   - Confidence: read it, sure. Not checked: whether the window's loot panel blocks this itself.

4. **`takeall` does not stop when seen: every stack is another roll, another charge, another alert**
   - Where: loot.rs `take_all_from` 255–268; containers.rs `took_from` 419–444; pursuit.rs `wrong_seen` 169–185.
   - Why: the comment says it stops when caught, but the stop test is `source_now(who) != Some(src)`, and `wrong_seen` only queues a pursuit that starts later.
   - Scenario: at night with a sleeper within 6 m, `takeall` on a chest holding 4–6 stacks.
   - Expect: several identical "X is seen stealing from a chest!" lines. Each one adds to the fine, drops standing by fine/4 and adds 1 to `gov.wrongs`. The chance of at least one sighting is 1−(1−c)^N.
   - Should be: stop at the first sighting.
   - Side effect: the tell-or-not roll is keyed per witness, thief and minute, so all sightings in one `takeall` are told or kept quiet together.
   - Confidence: read it, sure.

5. **An out-of-range person id crashes the play tool**
   - Where: play.rs `loot` 783–787 (`w.person_pos(p)` before any bounds test), `talk` 751, `attack` 740, `carry` 965, `cast` 866, `use` 1015.
   - The sim calls index directly: world.rs `person_pos` 600, dialogue.rs `order_talk` 279, fights.rs `attack` 91 (`group_of[enemy]`), carry.rs `can_carry` 102.
   - Scenario: `loot p999999`, `talk p999999`, `attack p999999`, `carry p999999`, `cast NAME 1 p999999`.
   - Expect: index-out-of-bounds panic. Should be: "Can't…". (`go p999999` is guarded in `pos_of`.)
   - Confidence: read it, sure.

6. **`take` with a missing or bad number takes line 1; `pickup`, `gather`, `work` and `butcher` ignore the id's letter**
   - Where: play.rs 892–893 (`parse().unwrap_or(0)` then `saturating_sub(1)`); 908 (`id.get(1..)` drops the first character whatever it is). The same pattern at 862 makes `cast NAME 0` cast spell 1.
   - Scenario: with a chest open, `take` or `take x` steals the first line. `pickup 17` targets g7; `pickup n5` or `pickup d5` targets ground item 5 (possibly owned, so theft); `work h4` works deposit 4.
   - Should be: a usage line or "Can't do that".
   - Confidence: read it, sure.

7. **`search` on a locked chest reports on the wrong person, and says "couldn't get to it" after a successful pick**
   - Where: play.rs 805–829.
   - Why: `looter = w.looting.last()…or_else(picking.last())`, but `order_pick_container` adds to `picking`, not `looting`. Any other member's old looting entry wins, and those entries are never cleared (see 8).
   - Scenario A: member A searches any unlocked chest; later member B is nearest to a locked one and you `search` it. If A is still at their chest the wait loop never runs and A's chest is printed; otherwise it spins 600 s and reports about A.
   - Scenario B, single member: the pick succeeds (News says "X picks the lock."), then the tool prints "They couldn't get to it: they couldn't reach it in time (it may be behind a wall or a locked door)". A second `search` is needed.
   - Confidence: read it, sure.

8. **Reach has no wall test, and a looting entry outlives walking away**
   - Where: loot.rs `source_now` 116–121 (2.16 m from the container, distance only); squad.rs `do_pickups` 694 (1.8 m); squad.rs `order_members` 210–223 clears pickups, picking, giving and more, but not `looting`; play.rs never calls `stop_looting`.
   - Scenario A: by day `search k…` on a chest standing against an outer wall. Walk out and `go X,Y` to the spot outside that wall. `take N` still works, including after 20:00 with the door locked.
   - Scenario B: stand outside the wall by a shelf item, then `pickup gN`. `order_pickup` logs "The door is locked." and then calls `do_pickups()`, which takes it through the wall.
   - Also: `take` and `put` act for the first looting entry that is in reach, not the selected member (play.rs 833, 885).
   - Confidence: sure on the code; likely on the geometry (not checked that a wall-side chest is within 2.16 m of a standable outside point in every variant).

9. **The guard runs through walls but cannot see through them; some charges vanish**
   - Where: pursuit.rs `chase` 256–300, `guard_sees` 249–253, `caught` 309.
   - Scenario A: seen picking a lock or stealing, then walk into any open building. The guard heads for the last outdoor sighting just outside the door, never sees in, and gives up after 120 s: a bounty, never an arrest.
   - Scenario B: seen stealing indoors, then step outside. The guard walks through the wall to the chest and stands in the room.
   - Scenario C: if the guard is in any fight (line 267), or the culprit is no longer in `squad.members`, e.g. left at a base (line 265), the pursuit is removed with no bounty and the fine is lost. `caught` likewise returns with nothing if the culprit became bonded or is in a duel.
   - Should be: the guard uses doors, and a dropped chase becomes a bounty.
   - Confidence: likely. Not checked: how a fight with a guard starts in practice (`attack` refuses non-hostile townsfolk; a harmful scroll or spell does it).

10. **Lock-pick rolls replay from try 1 on every new order**
    - Where: buildings.rs `order_pick` 410 and containers.rs `order_pick_container` 405 (`tries: 0`); `do_picking` 481–485 keys on (seed, who, door, tries), with no night and no order count.
    - Scenario: order a pick and note the sequence (say fail, snap, seen). Walk off and order again, or come back another night: the same sequence, unless Security has risen enough to flip a roll.
    - Expect: a door whose try 2 snaps a pick always snaps one. A door whose try 1 is a quiet miss can be retried forever at no risk, +1 Security each time (exercise at line 486 comes before the witness test).
    - Should be: keyed to something that moves on (the night, or a per-door attempt count).
    - Confidence: sure on the mechanism; likely on how visible it is.

11. **Chest contents depend on when the squad first walks in**
    - Where: containers.rs `stock_building` 275–276 reads `self.keeper(d.id)` at first entry and passes it to `stock`.
    - Why it matters: the file header promises the same things "whenever it's first opened", and `furnish` uses `first_sleepers` for exactly this reason.
    - Scenario: a bench workshop (`roduro_benchroom` / `qotiro_benchyard`) whose keeper dies, changes job or is recruited before the first visit gets generic `BENCH` stock instead of that trade's `BENCH_STOCK`.
    - Confidence: likely. Not checked: how often jobs or keepers change in a normal game.

12. **The play tool hands orders to members who cannot act, and non-move orders skip the bond check**
    - Where: play.rs `nearest` 674 (used by loot, search, pickup, gather, butcher) and the `enter` picker 942 choose among all selected, including the knocked-out and the bonded. `order_search`, `order_pick`, `order_pick_container`, `order_pickup` and `order_loot` never call `free_to_order` or `is_down`; only `order_members` does.
    - Scenario: with one member out cold next to a chest or body, `search`/`loot` picks them. The loop spins 2400 steps (10 game minutes) and prints "They couldn't get to it". For a bonded member, `hold_the_bound` re-routes them next step, with no message.
    - Also: buildings.rs `lock_outlook` 428 names the best hand among all members, down or bound included.
    - Confidence: read it, sure.

13. **With no one left in the squad, every play command panics**
    - Where: play.rs 673 (`unwrap_or(w.squad.members[0])` is evaluated eagerly) and 458.
    - Why: fights.rs 574 removes dead members from `squad.members`, so a wipe leaves it empty.
    - Expect: `look`, `wait` and everything else crash instead of saying the squad is gone.
    - Confidence: sure given an empty squad; a guess that testers will reach one (deaths are rare).

14. **Too much telling in the crime, lock and container text**
    - pursuit.rs 163–164: "{witness} saw it, but says nothing." reveals a hidden roll and names a stranger. Line 198 names the stranger who "runs to fetch the watch".
    - `wrong_seen` and `post_bounty` push every line to both `say` and `alerts`, so play prints each twice ("!! …" and under News).
    - buildings.rs 432 and 508: the lock's exact number ("a fair lock (37)"), exact % per try, "about 1 in 3 snaps", and the % again on every snapped pick.
    - play.rs 433 and 484: "r3 … N things lying inside" from 400 m, and "N things in it" for wild chests before opening.
    - play.rs 455–463: owned items inside buildings within 60 m are listed through walls with a seen-%.
    - play.rs 1267: a ruin cache or bandit stash is titled "taking from it is theft: 0% chance" although it is nobody's.
    - play.rs 344: the HUD says "night" from 18:00; doors lock at 20:00 and it is full daylight until 19:00.
    - Confidence: read it, sure (whether each is unwanted is the designer's call).

15. **A knocked-out or carried townsperson can be the witness; walking into a home is never noticed**
    - Where: pursuit.rs `spotter` 86–104 filters the dead (via `residents_in_band1`) and plan-sleepers, not the knocked out. world.rs `person_pos` 608 puts a carried person at their carrier.
    - Scenario: a town champion left out cold after a duel, lying near a door, "sees" a lock being picked and "runs to fetch the watch". A downed local carried into a house counts as in the room.
    - Related gap: `Wrong::Trespass` is raised only by a seen lock-pick (buildings.rs 489). Walking into any unlocked home at night, standing over the sleepers, or sleeping there for the best rest (condition.rs `shelter_of` 398) draws no notice. Casting Unlock (casting.rs 571–592) is never witnessed.
    - Confidence: sure on the code for the witness; a guess on reachability and on whether the trespass gap is intended.

**Smaller edges, not counted above**
- u16 stacks: loot.rs `put_in` 236 (`x.1 += e.1`) and inventory.rs `Gear::add` 58 are unchecked, while `put_into` saturates; quests.rs `squad_count` 104 sums into u16. Past 65,535 of one thing (coin) this panics in a dev build and wraps in release.
- law.rs `pay_or_bond` 1010–1013 counts only "coin", not 50-coin notes, so a squad rich in notes is bonded for a fine. This is past the arrest and outside my part.
- view/hud.rs 261 iterates the `bounty` HashMap directly, so the display order can change after a load.