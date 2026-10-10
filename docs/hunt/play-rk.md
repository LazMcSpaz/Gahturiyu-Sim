# Play report: the RECKLESS player (prefix `rk`)

Worlds: seed 34 (`rk34…`), then seed 3 (`rk3…`). Saves are in `/home/claude/hunt/saves/`.

Note for whoever re-runs these: the tool keeps "which news lines have been shown" in `<name>.session`, not in
the save. When you copy a save, copy the `.session` too (`cp x.save y.save; cp x.session y.session`), or the copy
re-prints old News and TIP lines and a line-by-line compare shows false differences. Where a finding says
"from saved copy X", the commands were run on a fresh copy of X (both files where a `.session` exists).

Town justice customs were looked up with `/home/claude/hunt/headless society 0 <seed>` (the `town` panel in
the text tool does not show customs: see the finding on that).

## Findings

### A duel is over before the player can do anything; the accused never even has to stand there
- Part: Crime and the law
- How bad: rough (unsure whether intended)
- What happens: the watch catches the thief, "must answer for theft by duel" appears, and by the next command (zero game minutes later) the duel is won or lost. There is no moment in which to pick the champion, give an order, drink a draught or walk the accused away. Six duels seen in all: five were decided inside the minute they began; one (a lone accused who was lying down when caught, copy `rk34-sleepduel-pre` → `wait 1` ×3) ran for one more `wait 1` before it was lost.
- What should happen: a duel that lasts long enough to watch or act in (or at least a say in who fights).
- To see it: seed 34, copy `rk34-chase` → `wait 1` (fight starts) → `fight` → `wait 1`.
- Output:
```
!! Yafesuthih of the watch catches Moker.
(1 minutes pass.) A fight has started!
  06:01  Moker must answer for theft by duel in Stonehaven.
FIGHT:
  [yours] p5003   Sawasasih              fighting health 75%
  [them ] p4809   Yafesuthih             fighting health 100%
...
!! Moker is bound to work in Stonehaven for 13 days: led off to work it off. An official there can sell you the bond.
(0 minutes pass.) The fight is over.
  06:01  Moker's side loses the duel, and must pay 105.
```
- Seen how many times: three duels (two from `rk34-chase`, one from `rk34-duel2`), each run twice.
- Because of this the code-reading candidate "accused walks out of town during a lost duel" could not be reached: `select Moker` + `go e 600` right after the fight starts returns at once with the duel already lost (copy `rk34-induel`).

### The squad's duel champion was its frailest member (the Ṭaḍoro with a staff), not the spear or the club
- Part: Crime and the law
- How bad: rough (unsure: the rule may be "highest might", and a caster may rate high)
- What happens: Moker (hatchet and buckler) is accused; all four stand together; the game puts Sawasasih (staff, padded jacket, 52 kg carry) up against the guard. She is at 75% the moment the fight is listed and down the same minute. Later Nolu (spear), alone, beat the same guard at once.
- What should happen: the accused fights, or the player picks, or the best fighter is picked by something the player can see.
- To see it: seed 34, copy `rk34-chase` → `wait 1`.
- Output: as above (`[yours] p5003 Sawasasih fighting health 75%`).
- Seen how many times: twice from the same copy.

### "The downed will come round in an hour or two" — she is up in 12 minutes
- Part: Crime and the law (duels)
- How bad: misleading
- What happens: Sawasasih goes down at 06:01 in the duel; News says "The downed will come round in an hour or two."; at 06:13 she is "standing" with "health 0%" and walks with the squad. No line says she got up.
- What should happen: the line should match what happens (or she should stay down as long as it says).
- To see it: seed 34, copy `rk34-bound1` (06:08, she is down) → `wait 1` five times.
- Output:
```
  06:01  The fight is over: you lost. Down: Sawasasih.
  06:01  The downed will come round in an hour or two.
== Day 1, 06:12 (day) · Stonehaven ==
  Sawasasih  Ṭaḍoro   down                   health   0% stamina 100% load 18/52 kg (Toughness 31 ↑)
== Day 1, 06:13 (day) · Stonehaven ==
  Sawasasih  Ṭaḍoro   standing               health   0% stamina 100% load 18/52 kg (Toughness 31 ↑) [34 m from the others]
```
- Seen how many times: twice (main line and the copy).

### A beaten town champion is back at his post the instant the duel ends, can't be gone through, and greets you as a stranger
- Part: Crime and the law
- How bad: wrong outcome (unsure how much is intended) / rough
- What happens: Nolu is caught 230 m south of town, wins the duel ("Beaten: Yafesuthih"). In the same minute `look people` lists Yafesuthih back beside the other guard in town. `loot p4809` says he isn't a beaten enemy; `attack p4809` is refused; 20 minutes later `talk p4809` opens with "Hello, traveller." as if nothing happened. Two hours later he was the town's champion again (see "sent out at 4% health").
- What should happen: the beaten champion lies where he fell for a while, and the town remembers the duel.
- To see it: seed 34, copy `rk34-duel2` (duel just begun, 06:29) → `wait 1` → `look people` → `loot p4809`.
- Output:
```
  06:29  Nolu's side wins the duel; the matter is closed.
  06:29  The fight is over: you won. Beaten: Yafesuthih.
 *Nolu       Horaro   standing ... [232 m from the others]
  p4809  Yafesuthih — Ṭaḍoro guard — 58 m north
  p1045  Litihegi — Roduro guard — 60 m north
>>> loot p4809
Can't: Only beaten enemies can be gone through; Yafesuthih isn't one.
```
- Seen how many times: twice (main line and the copy).

### A bound member shows as plain "standing"; nothing anywhere says who is bound, where, for how long or the price
- Part: Crime and the law
- How bad: misleading (the code-reading candidate 15: seen)
- What happens: after "Moker is bound to work in Stonehaven for 13 days", `look` lists Moker as "standing" like the others, `journal` says "No jobs taken.", and `town` says nothing about a bond. The only trace is the one News line. The price only appears as a talk topic with the official when he is at the hall (see next).
- What should happen: the squad list should say "bound (Stonehaven, 13 days)" or similar.
- To see it: seed 34, copy `rk34-bound1` → `look`, `journal`, `town`.
- Output:
```
  Moker      Qotiro   standing               health 100% stamina 100% load 34/70 kg [45 m from the others]
Journal:
  No jobs taken.
```
- Seen how many times: every look for the rest of the session.

### "An official there can sell you the bond" — but the official has no such topic until he is at his desk, and says nothing about it
- Part: Crime and the law
- How bad: rough
- What happens: at 06:25 the official (p1356 Ditut) offers only Background / This town / advice / rumours / Bandits / Come with us / Goodbye. At 06:56 (he has reached the hall) the same open talk gains "Buy Moker out of their bond (105 coin)" and the labourer post.
- What should happen: off duty he should at least say "come to the hall".
- To see it: seed 34, copy `rk34-bound1` → `select Ṭelihoḍu` → `talk p1356`; then `wait 30` and `look`.
- Output: see the next finding.
- Seen how many times: once.

### A talk left open goes on working from 1.3 km away: new topics appear and a town job is taken at that distance
- Part: Town jobs and recruiting
- How bad: wrong outcome (code-reading candidate 1 in code-law-jobs-recruit: seen)
- What happens: with the talk with the official open, Ṭelihoḍu walks 1,300 m north. `look` still shows "TALKING with Ditut"; the topic list has changed under the player (topic 6 was "Come with us" and is now "Buy Moker out of their bond"); `say 7` hires Ṭelihoḍu as labourer.
- What should happen: the talk ends when you walk off (or the tool refuses to move with a talk open).
- To see it: seed 34, copy `rk34-talkopen` (talk open with Ditut, 06:26) → `go n 1500` → `look` → `say 6` → `say 7` → `journal`.
- Output:
```
 *Ṭelihoḍu   Roduro   walking                health 100% stamina  56% load 18/77 kg [1338 m from the others]
TALKING with Ditut (Qotiro official):
    6. Buy Moker out of their bond (105 coin)
    7. I'll work as labourer and porter at the market square (19 coin a day, 8 till 5)
    you: Buy Moker out of their bond (105 coin)
  » Not so fast: not enough coin.
    you: I'll work as labourer and porter at the market square (19 coin a day, 8 till 5)
  » Good, Ṭelihoḍu. You'll work as labourer and porter at the market square from tomorrow, 8 till 5, for 19 coin a day, paid each dawn.
```
- Seen how many times: once (from the saved copy).

### A bound member sent to talk burns 10 minutes and fails with no reason given; "bye" with no talk open says "You take your leave."
- Part: Odd input / Crime and the law
- How bad: rough (candidate 7 scenario A: seen)
- What happens: `select Moker` (bound) → `talk p4569` (a labourer 10 m off): "(10 minutes pass.) They're not talking (or you couldn't reach them)." Nothing says it is because Moker is bound. `bye` afterwards prints "You take your leave." although no talk was open. `rest`, `sneak` and `torch` are all accepted for the bound member ("Resting.", "Sneaking.", "Moker lights a torch.").
- What should happen: "Moker is bound and can't" at once.
- To see it: seed 34, copy `rk34-bound1` → `select Moker` → `talk p4569` → `bye`.
- Seen how many times: once.

### `go nan,nan` (or `inf,inf`) breaks a squad member for good: they can never be moved again and every later `go` burns 30 minutes
- Part: Odd input
- How bad: stuck
- What happens: Nolu is sent 600 m north, then `go nan,nan`. The tool answers "Nobody needs to move: they're already there." and 30 minutes pass. From then on every `go` for her (a direction, a person, the town) gives the same line and another 30 minutes; the "[N m from the others]" flags vanish for everyone; `enter` fails after 30 minutes. With `select all`, the whole squad's `go` also waits the full 30 minutes each time. The bad position is written into the save. `go inf,inf`, `go n inf` and `go n nan` behave the same way. `cast Nolu 3 nan,nan` is also accepted ("Nolu begins Kindle.").
- What should happen: "Go where?" for anything that isn't a finite number.
- To see it: seed 34, copy `rk34-prenan` (Nolu selected, 599 m north of the others) → `go nan,nan` → `go n 100` → `go s 50` → `go p5003`.
- Output:
```
 *Nolu       Horaro   standing               health 100% stamina  99% load 43/69 kg [599 m from the others]
>>> go nan,nan
Nobody needs to move: they're already there.
(30 minutes pass.)
 *Nolu       Horaro   standing               health 100% stamina 100% load 43/69 kg
>>> go n 100
Nobody needs to move: they're already there.
(30 minutes pass.)
>>> go s 50
Nobody needs to move: they're already there.
(30 minutes pass.)
```
- Seen how many times: three times (two copies with `nan,nan`, one with `inf,inf`).

### `cast NAME N p999999` crashes the tool (same fault as the known talk/loot/attack/carry one)
- Part: Odd input
- How bad: crash
- What happens: a spell aimed at a person id that doesn't exist panics.
- What should happen: "Can't: nobody there."
- To see it: seed 34, copy `rk34-oddbase` → `cast Nolu 1 p999999` (also `p4294967295`).
- Output:
```
thread 'main' (25442) panicked at src/sim/world.rs:600:29:
index out of bounds: the len is 5067 but the index is 999999
```
- Seen how many times: twice. Not crashing with a bad person id: `use NAME N p999999` (non-scroll items), `go p999999`, `carry`/`loot`/`talk`/`attack` with a building or container id, `enter p59`, `pickup p59`, `hunt p59`, `butcher p59`.

### Commands with a missing argument quietly act on squad member 1 or on line 1
- Part: Odd input
- How bad: wrong outcome (code-reading candidate 6 in code-buildings-crime: seen)
- What happens, each seen:
  - `take` (no number) with a crate open takes line 1: a theft, and it was seen (copy `rk34-crate` → `take`): "Nolu takes 6 × fibre from a crate." / "Nolu is seen stealing from a crate!"
  - `dose` (no names): "Ṭelihoḍu gives Ṭelihoḍu a healing draught." The draught is gone from his pack though he is at 100%.
  - `unequip` (no name, no slot): "Taken off." Ṭelihoḍu's club goes to his pack. `unequip Nolu` (no slot) takes off the main hand.
  - `give Nolu 0` (no receiver): "Nolu gives Ṭelihoḍu knife."
  - `pack Nobody` and `pack all` show Ṭelihoḍu's pack; `spells Nobody` shows the selected member's spells; `select ""` selects Ṭelihoḍu; `select Nolu Nolu` prints "Selected: Nolu, Nolu".
  - `cast Nolu 0` and `make Nolu 0` are treated as spell 1 and recipe 1.
  - `wait x` and `wait` wait 10 minutes; `wait 0.5` prints "(0 minutes pass.)" but the clock moves a minute.
- What should happen: a usage line.
- To see it: seed 34, copy `rk34-oddbase` → `dose`, `unequip`, `give Nolu 0`, `pack Nobody`.
- Seen how many times: `take` once; the others twice each.

### The letter of an id is ignored: `pickup x6` steals g6, `gather p59` walks off to plant n59, `work h2` marches the squad out of town to woodlot d2
- Part: Odd input
- How bad: wrong outcome (code-reading candidate 6: seen)
- What happens: `pickup x6` in a home picked up the owner's war-pick (g6) and was seen ("Sawasasih is seen stealing!"). `gather x1` and `gather p59` each sent the squad off for 30 minutes. `work h2` and `work x2` sent the squad to deposit d2 ("the wilds" in the header after 30 minutes). `enter 1.2` enters b1.2 and `search 1.2.2` opens k1.2.2. (`pickup 6`, `pickup 17`, `work 2`, `gather 1` with no letter are refused, so the candidate's `pickup 17` → g7 was not seen.)
- What should happen: "Can't do that."
- To see it: seed 34, copy `rk34-ws-house` (squad in a Woodsands home, war-pick g6 on the floor) → `select Sawasasih` → `pickup x6`. For work: copy `rk34-oddbase` → `work h2`.
- Output:
```
>>> pickup x6
!! Sawasasih is seen stealing! Qahiraqa runs to fetch the watch!
  09:16  Sawasasih picks up war-pick.
>>> work h2
(30 minutes pass.)
== Day 1, 10:54 (day) · the wilds ==
```
- Seen how many times: `pickup x6` once (main line); `gather`/`work` once each.

### With one member's container open, nobody else can take from theirs: `take`/`takeall` go to whoever opened one first, or to a different container than the one just searched
- Part: Buildings and interiors / Odd input
- How bad: wrong outcome (code-reading candidate 8 "also": seen)
- What happens: Nolu has emptied crate k0.4.0 and still stands by it. `select Ṭelihoḍu`, `search k0.4.1` lists "1 × Fibre"; `takeall` answers "Took 0 lots." and shows Nolu's empty crate. `select Sawasasih`, `search k0.4.3` lists flatbread and kelp; `take 1` → "Couldn't take that." After Nolu steps away, Sawasasih's `search k0.4.3` + `takeall` takes Ṭelihoḍu's fibre from the other crate instead ("Ṭelihoḍu takes fibre from a crate."). Only when both others have been walked off does the barrel's `takeall` work.
- What should happen: `take` acts on the container just searched by the selected member.
- To see it: seed 34, copy `rk34-twoopen` → `select Sawasasih` → `search k0.4.3` → `take 1`.
- Output:
```
>>> search k0.4.3
Sawasasih opens a barrel (taking from it is theft: 25% chance of being seen):
  1. 8 × Flatbread  (~3 coin each)
  2. 6 × Kelp frond  (~2 coin each)
>>> take 1
Couldn't take that.
Nolu opens a crate (taking from it is theft: 26% chance of being seen):
  (take N, or takeall)
```
- Seen how many times: three times in the main line.

### Anything can be put in the main hand: flatbread, a tent, a standing torch
- Part: Odd input
- How bad: cosmetic
- What happens: `equip Nolu 6` (3 × flatbread) → "Done." and the pack shows "MainHand   Flatbread"; her spear goes to the pack. The pack is re-ordered after each equip, so the numbers from the last `pack` are stale at once.
- To see it: seed 34, copy `rk34-oddbase` → `equip Nolu 6` → `pack Nolu`.
- Seen how many times: once.

### Trade and sign-on fees reach across any distance: a member 2.2 km away, bound in another town, has her things sold and her coin spent
- Part: Town jobs and recruiting / Crime and the law
- How bad: wrong outcome
- What happens: the merchant's Sell list is built from every squad member's pack wherever they are. In Stonehaven, Nolu sold the buckler of Moker (bound, 77 m off, who "can't give") and the 27 coin landed in Moker's pack. In Leafshore, Nolu sold the mortar and pestle that was in the pack of Sawasasih, who was bound in Woodsands 2.2 km away; the 15 coin appeared in Sawasasih's pack there. A 72-coin sign-on fee was paid with Nolu's 71 coin plus 1 coin out of Sawasasih's pack at that same distance (her 110 became 109). A bound member's coin also paid another member's bond buy-out.
- What should happen: only what the talker (or members standing with them) carries can be sold or spent.
- To see it: seed 34, copy `rk34-leaf-trade` (trade open with p3800 in Leafshore; Sawasasih is in Woodsands) → `pack Sawasasih` → `say 18` (Sell mortar and pestle) → `pack Sawasasih`.
- Output:
```
    you: Sell mortar and pestle (15 coin)
  » I'll give you 15 for the mortar and pestle.
    9. 124 × Coin [money]  (worth ~1 each)          <- Sawasasih's pack, was 109
  Sawasasih  Ṭaḍoro   standing ... [2239 m from the others]
```
- Seen how many times: three separate sales/fees (Stonehaven, Woodsands, Leafshore).

### With the talk left open you can walk 1.5 km from the merchant and still sell
- Part: Town jobs and recruiting
- How bad: wrong outcome (code-reading candidate 1: seen for Sell as well as for taking a post)
- What happens: trade is opened with the Leafshore merchant, Nolu walks 1,500 m east for 15 minutes without `bye`, then `say 13` sells the amulet for 168 coin.
- To see it: seed 34, copy `rk34-leaf-trade` → `go e 1500` → `look` (still "TALKING with Heyesiyi") → `say 13`.
- Output:
```
 *Nolu       Horaro   standing               health 100% stamina  94% load 40/69 kg [1465 m from the others]
TALKING with Heyesiyi (Ṭaḍoro merchant):
    you: Sell amulet of the clear mind (168 coin)
  » I'll give you 168 for the amulet of the clear mind.
```
- Seen how many times: once from the copy.

### Stolen goods stay with the thief after conviction, lose their "someone's" mark when dropped, and sell at full price in the town they were stolen from
- Part: Crime and the law
- How bad: wrong outcome (unsure: may be a design choice, but it makes being caught profitable)
- What happens: Moker is caught with the iron helm, loses the duel, is bound for 13 days over a 105-coin debt — and keeps the helm. `drop Moker 7` puts it on the ground as plain "1 × Iron helm on the ground" (no "someone's"); Nolu picks it up without a crime and sells it to the Stonehaven merchant for 96. In Woodsands, Sawasasih was fined 75 for a war-pick; next day the same town's merchant paid 110 for it. In both towns the sale more than covered the fine.
- What should happen: the watch takes back what was stolen (or the merchant won't touch it).
- To see it: seed 34, copy `rk34-0826` → `pack Moker` → `drop Moker 7` → `look places` → `select Nolu` → `pickup g5` → `talk p1467` → `say 6` → `say 13`.
- Output:
```
    7. 1 × Iron helm   (worth ~120 each; sells for 96 in Stonehaven)
  g5  1 × Iron helm on the ground — 36 m south-east
  08:34  Nolu picks up iron helm.
    you: Sell iron helm (96 coin)
  » I'll give you 96 for the iron helm.
```
- Seen how many times: twice (helm in Stonehaven, war-pick in Woodsands).

### A part-payment of a fine is taken without a word
- Part: Crime and the law
- How bad: misleading
- What happens: the squad has 18 coin; the elders fine Sawasasih 75. The News says "a fine of 75" and "bound to work in Woodsands for 7 days"; the header goes from 18 coin to 0. Nothing says 18 was taken or that 57 is still owed (7 days matches 57, not 75).
- What should happen: "pays 18 of 75; bound 7 days for the rest."
- To see it: seed 34, copy `rk34-ws-chase` → `wait 1` twice.
- Output:
```
Squad (4 members, 0 coin between them) — selected marked *:
  09:17  Sawasasih is judged by the elders of Woodsands for theft: a fine of 75.
  09:17  Sawasasih is bound to work in Woodsands for 7 days: led off to work it off. An official there can sell you the bond.
```
- Seen how many times: once (plus the copy).

### With any member bound, `search` on a locked chest says "<bound member> was caught and is bound…" though the selected member picks the lock fine
- Part: Odd input / Buildings and interiors
- How bad: misleading (a new face of known item 6)
- What happens: Ṭelihoḍu and Sawasasih are bound; Nolu (free, selected, 6 lockpicks) does `search k1.2.1`. The tool prints "Ṭelihoḍu was caught and is bound to work it off (see News)." Two minutes later the News shows "Nolu picks the lock." and a second `search` opens it.
- To see it: seed 34, copy `rk34-ws-lockbound` → `search k1.2.0` → `wait 1` → `search k1.2.0`.
- Seen how many times: three times.

### The eleventh recruit is refused cleanly
- Not a bug: at ten members "Come with us" answers "There's no room for another of you, by the look of it." and the town panel shows "Willing to join (0)". Copy `rk34-ten` → `talk p992` → `say 6`.

### A member walking alone to a town job freezes on the road all day: "walking to work", not moving, stamina draining
- Part: Town jobs and recruiting
- How bad: stuck (for that member, until given a new order) / wrong outcome
- What happens: Ṭelihoḍu took the labourer post in Stonehaven on day 1, then the squad went south. On day 3 at 08:00 he sets off alone from Woodsands with no line in the News (code-reading candidate 6, the silent departure: seen). By 08:35 he is "[3180 m from the others]" and from then until 17:00 that number never changes while his status stays "walking to work" and his stamina falls from 94% to 36%. At 17:00 he turns to "standing" in the same spot; the journal says "0 days paid". A plain `go` order frees him at once (he then covered 3 km in 30 minutes).
- What should happen: he reaches the job (or the game says why he can't), and a line says he has left.
- To see it: seed 34, copy `rk34-d3-0835` → `wait 10` six times → `wait 300`.
- Output:
```
== Day 3, 08:45 (day) · Leafshore ==   Ṭelihoḍu   Roduro   walking to work        health 100% stamina  93% load 21/77 kg [3180 m from the others]
== Day 3, 09:35 (day) · Leafshore ==   Ṭelihoḍu   Roduro   walking to work        health 100% stamina  85% load 21/77 kg [3180 m from the others]
== Day 3, 14:35 (day) · Leafshore ==
  Ṭelihoḍu   Roduro   walking to work        health 100% stamina  36% load 21/77 kg [3180 m from the others]
Town work: Ṭelihoḍu works as labourer and porter at the market square in Stonehaven, 08:00–17:00, 19 coin a day (0 days paid; should be there now).
```
- Seen how many times: twice (main line for one hour, the copy for the whole day), and a third time later the same day: after being walked back to Woodsands he set off again and froze at "[885 m from the others]". Not every such walk fails: in seed 3 Ḍuhushì walked alone from Windflat to his post in Stilledge (2.8 km) in 30 minutes and showed "at work" (copy `rk3-wf-job` → `wait 10` ×4). So it is that spot: about 950 m north of seed 34's Woodsands, by its woodlot (d2), on the way to Stonehaven. Walking the others up to him (copy `rk34-d3-0835` → select the other eight → `go p5000` ×3) prints "Bandits in sight." on the way, so the bandits by that road may be what stops him; he still shows "walking to work" and nothing tells the player.

### A member who is bound in one town and has a job in another flips between "walking to work" and "standing" all day; nothing is said about either
- Part: Crime and the law / Town jobs and recruiting
- How bad: rough
- What happens: Ṭelihoḍu (post in Stonehaven from day 2) is bound in Woodsands on day 1 "for 1 days". All through day 2 his status alternates "walking to work" / "standing" while he stays in Woodsands; `go` still says he is bound at 12:26 on day 2, 27 hours after a "1 days" bond began. The journal says "should be there now". On day 3 he simply walks off (no line that the bond ended).
- What should happen: a bound member's job is put on hold with a line; a line when the bond ends; "1 days" should mean what it says.
- To see it: seed 34, copy `rk34-d2-0851` → `wait 5`, `wait 30`, `wait 60`, `wait 120` → `go n 50`.
- Output:
```
== Day 2, 08:56 (day) · Woodsands ==
 *Ṭelihoḍu   Roduro   walking to work        health 100% stamina 100% load 21/77 kg
== Day 2, 09:26 (day) · Woodsands ==
 *Ṭelihoḍu   Roduro   standing               health 100% stamina 100% load 21/77 kg
>>> go n 50
Nobody moves: Ṭelihoḍu is bound to work off a bond.
```
- Seen how many times: once (main line), four looks.

### A bounty shows nowhere, and any passer-by takes the money for it at night
- Part: Crime and the law
- How bad: rough / wrong outcome (code-reading candidate 10: seen)
- What happens: picking a door at 20:07 gives "Nolu is seen picking a lock! Bounty in Leafshore: 40." (no watch comes at night). `journal` and `town` say nothing about a bounty. A scribe standing 9 m away greets Nolu with "Hello, traveller…" and offers "Pay my bounty"; `say 6` → "40 coin. Consider the matter closed — this time." Five minutes later the very next pick attempt on the same door is seen again on its first try ("Bounty in Leafshore: 40." again; the code-reading candidate about lock-pick rolls replaying from try 1 fits this, seen twice in a row).
- What should happen: the bounty is listed somewhere; it is paid at the hall or to the watch; people who know of it don't greet you as a stranger.
- To see it: seed 34, copy `rk34-leaf-night` → `select Nolu` → `enter b2.8` → `journal` → `town` → `talk p4014` → `say 6` → `bye` → `enter b2.8`.
- Output:
```
  20:07  Nolu is seen picking a lock! Bounty in Leafshore: 40.
Journal:
  [Town work] Ṭelihoḍu works as labourer and porter ...
    6. Pay my bounty
    you: Pay my bounty
  » 40 coin. Consider the matter closed — this time.
  20:13  Nolu is seen picking a lock! Bounty in Leafshore: 40.
```
- Seen how many times: once (main line).

### "X is seen stealing! Y saw it, but says nothing." is pushed as a red alert
- Part: Crime and the law
- How bad: too much telling (code-reading candidate 14: seen)
- What happens: the squad is told the name of a stranger who saw the theft and that they chose to keep quiet; it comes as a "!!" alert and again in News. Nothing follows from it. Seen with Haʻo (Woodsands, twice in one minute for two items) and Yeyahetha (Leafshore).
- To see it: seed 34, copy `rk34-ws-pre3` → `pickup g10` → `pickup g11`.
- Output: `!! Nolu is seen stealing! Haʻo saw it, but says nothing.`
- Seen how many times: three times.

### Other telling noticed in towns (one line each)
- How bad: too much telling
- "Bandits fall on travellers 1337 m away." / "The roadside fight is over." arrive in the News while the squad stands inside a town, with the exact metres, several times a day.
- "X (Ṭaḍoro), wandering, crosses your path." fills the News (ten or more a day while standing still in town).
- "06:00  Kereg takes up work as alchemist in Leafshore." and "A bad omen in Leafshore; the arbiter rules it was no true sign." are pushed at dawn.
- `town` lists everyone "Willing to join" with their price, skills, weapon and clothes and how far away they are (one was 403 m off on the boatyard and could not be reached: "They're not talking (or you couldn't reach them)"), and each merchant's next opening time; `look people` tags them "[restless: might join for 40 coin]" before anyone has spoken to them.
- `look` lists owned things lying inside locked houses at night through the walls ("g55  1 × Ring of the Ox on the ground (someone's…) — 37 m west").
- Lock lines give the lock's number, the exact % a try and the snap odds ("a fair lock (37): Nolu ~47% a try, 6 lockpicks (about 1 in 3 snaps on a miss)"); "a fair lock (45)" and "a hard lock (45)" both appear in one list.
- One News line is broken: "06:26  Ditdod and 1 other, ." (cosmetic).

### A bound member with no food starves where she works, with 109 coin in her pack; the town that holds her does not feed her
- Part: Crime and the law / Townsfolk's lives
- How bad: wrong outcome
- What happens: Sawasasih is bound in Woodsands for 7 days. Her food is dropped on day 2. Nobody feeds her and she cannot buy or fetch anything: "hungry" on day 3, "weak with hunger" that night, "STARVING" on day 4 (health 73% → 25%), "down / health 0%" at 04:27 on day 5, with 109 coin in her pack the whole time. None of the stage changes makes a News line. (A squadmate handing her fish gets her up again: `give Nolu 4 Sawasasih` → "Sawasasih eats some dried fish." ×3 → "standing health 12%". So known item 2 has a rescue when another member has food.)
- What should happen: a town holding someone to work feeds them (or takes it from their coin).
- To see it: seed 34, copy `rk34-ws-nofood` (day 2, 13:16; her food already dropped) → `wait 720` four times.
- Output:
```
== Day 4, 10:15 (day) · Leafshore ==
  Sawasasih  Ṭaḍoro   standing               health  73% stamina 100% load 18/52 kg STARVING [2149 m from the others]
== Day 5, 04:27 (night) · Leafshore ==
  Sawasasih  Ṭaḍoro   down                   health   0% stamina 100% load 18/52 kg STARVING
```
- Seen how many times: once (on the copy `rk34-starve`; `rk34-starved-bound` is the moment she goes down).

### The only way out of a bond is to be carried out unconscious; then "has run from their bond", and 60 coin to any guard wipes it
- Part: Crime and the law
- How bad: wrong outcome (code-reading candidate 10 "running costs a flat 60": seen) / rough
- What happens: a bound member refuses every move order, so "run from a bond" cannot be done on foot. With Sawasasih starved and down, `carry p5003` works on her and Nolu walks out of Woodsands: "05:01  Sawasasih has run from their bond in Woodsands." (a News line only, no alert; she is unconscious on someone's back). Walking straight back in and standing there for 30 minutes draws no one. Three hours later the whole squad stands in town; a guard greets them "Hello, traveller…" and offers "Pay my bounty" → "60 coin. Consider the matter closed — this time." She is then free; the four unworked days of the bond are forgotten. For the 105-coin bond in Stonehaven this route would have been the cheaper one.
- What should happen: carrying a bound member off should be stopped or treated as the carrier's crime; a runaway who walks back in should be seized; the bounty should not be less than what was owed.
- To see it: seed 34, copy `rk34-starved-bound` → `select Nolu` → `go p5003` (twice) → `carry p5003` → `go t0` → (News) → `go t1` twice → `select all` → `go t1` twice → `select Nolu` → `talk p4972` → `say 6`.
- Output:
```
 *Nolu       Horaro   carrying Sawasasih     health 100% stamina  81% load 45/69 kg [3512 m from the others]
  05:01  Sawasasih has run from their bond in Woodsands.
TALKING with Wases (Ṭaḍoro guard):
  » Hello, traveller. I'd ask where you've come from, but I'll guess instead, if you don't mind. I'm usually right.
    6. Pay my bounty
    you: Pay my bounty
  » 60 coin. Consider the matter closed — this time.
```
- Seen how many times: twice. The second time straight after a lost duel: Sawasasih, alone, loses, is "down" and "bound to work in Stonehaven for 13 days" (105 owed) at 06:11; Nolu picks her up at 06:13; "06:21  Sawasasih has run from their bond in Stonehaven." (copy `rk34-downbound` → `select Nolu` → `go p5003` → `carry p5003` → `go t1`). Copies `rk34-carrybound`, `rk34-ranbond`, `rk34-backatonce` mark the steps of the first.

### In Woodsands the official never offers to sell the bond
- Part: Crime and the law
- How bad: misleading (unsure of the cause)
- What happens: the bond line says "An official there can sell you the bond." In Stonehaven the official offered "Buy Moker out of their bond (105 coin)" once at the hall. In Woodsands the official (p4417 Siyashifah) was asked on day 2 at 13:10, on day 3 from 11:26 to 17:25 (talk left open, looked at every 30 minutes), and on day 4 at 09:01: the topic never appeared, with 215 coin in the squad and Sawasasih bound there. His Background says "I'm an official at the hall, 90 m south of here" each time: he is never at the hall. His "Any work?" answer is "Break that camp and I'll pay 0 coin."
- What should happen: the buy-out can be had, or the official says when.
- To see it: seed 34, copy `rk34-ws-d3eve` → `select Nolu` → `talk p4417` → `say 2`.
- Seen how many times: three days running.

### A lost duel with coin in hand: the fine is simply paid
- Not a bug, for the record: Leafshore, squad of ten with 237 coin; Yafeyisith is caught, Weyusatha (longsword and kite shield) fights Girix and loses; "Yafeyisith's side loses the duel, and must pay 22." and the purse drops to 215. Copy `rk34-leaf-preduel` → `select Moker` → `pickup g44` → `wait 1`.
- But in that same run `select Yafeyisith` + `go e 900` during the duel shows why the "accused leaves town during a lost duel" candidate can't be reached: the duel ends in the same step.

### The town's duel champion is sent out at 4% health two hours after losing the last duel
- Part: Crime and the law
- How bad: wrong outcome
- What happens: Stonehaven's guard Yafesuthih is beaten in a duel at 06:29. At 08:37 a different guard (Litihegi) catches Nolu, yet the champion who appears is again Yafesuthih, at "health 4%"; the squad's own champion (Sawasasih, picked again though she was the most hurt member and Nolu and Ṭelihoḍu stood beside her unhurt) wins in the same minute and "the matter is closed". So after one won duel, every later charge that morning is a walkover.
- What should happen: the town sends someone fit (the catching guard was), or the hurt champion yields.
- To see it: seed 34, copy `rk34-chase3` → `wait 1` → `fight` → `wait 1`.
- Output:
```
  08:37  Litihegi of the watch catches Nolu.
  08:37  Nolu must answer for theft by duel in Stonehaven.
FIGHT:
  [yours] p5003   Sawasasih              fighting health 53%
  [them ] p4809   Yafesuthih             fighting health 4%
  08:37  Nolu's side wins the duel; the matter is closed.
```
- Seen how many times: twice from the same copy.

### A wounded member's health jumps from 31% to 53% the moment a duel starts
- Part: Crime and the law (fights)
- How bad: wrong outcome (unsure which number is the true one)
- What happens: Sawasasih shows "health 31%" at 08:36. One minute later the duel begins and she shows "health 53%" in the squad list and the fight list; she takes no hit, and is at 53% after it and 55% ten minutes later. Either the squad list under-reports a healing member, or entering a fight heals.
- To see it: seed 34, copy `rk34-chase3` → `look` → `wait 1` → `wait 1` → `wait 10`.
- Output:
```
== Day 1, 08:36 (day) · Stonehaven ==
 *Sawasasih  Ṭaḍoro   standing               health  31% stamina 100% load 18/52 kg
(1 minutes pass.) A fight has started!
 *Sawasasih  Ṭaḍoro   fighting               health  53% stamina  97% load 18/52 kg
```
- Seen how many times: twice from the same copy.

### Ordering a downed member (or a squad that includes one) costs the full wait with no reason given
- Part: Odd input / orders to people who can't act
- How bad: rough
- What happens: `select Weyusatha` (down) → `go n 10`: "(30 minutes pass.)" and nothing else. `select all` → `go t1` with a downed member in the squad: everyone else arrives in a few minutes but the command returns only after 30. `talk` from a bound member takes 10 minutes and says "They're not talking (or you couldn't reach them)."; `work d0` and `pickup g3` from a bound member print "(0 minutes pass.)" and nothing else.
- What should happen: "Weyusatha is down." at once.
- To see it: seed 34, copy `rk34-leaf-down` → `select Weyusatha` → `go n 10`.
- Seen how many times: three times.

### Reading a harmful scroll at a person id that doesn't exist crashes the tool
- Part: Odd input
- How bad: crash
- What happens: with a Scroll of paralysis in the pack, `use Sawasasih 10 p999999` panics. (With food or a draught in that slot the bad id is ignored and the item is simply used.)
- To see it: seed 34, copy `rk34-hasscroll` → `use Sawasasih 10 p999999`.
- Output:
```
thread 'main' (11977) panicked at src/sim/world.rs:600:29:
index out of bounds: the len is 5067 but the index is 999999
   3: <gahturiyu_sim::sim::world::World>::person_pos
```
- Seen how many times: once.

### There is no way to raise a hand against townsfolk or the watch
- Not a bug as such, for the record (the assignment asked): `attack p…` on a farmer, on a guard just beaten in a duel, and on squadmates → "Can't attack that (too far, or not someone to fight)."; `cast Nolu 1 p59` (Spark) → "Can't: not an enemy"; `cast Sawasasih 6 p4208` (Fireball at a traveller 5 m away) → "Can't: only of use in a fight: aim it at enemies"; a Scroll of paralysis read at a cook 8 m away (`use Sawasasih 10 p273`) → "Can't: read it at enemies (usage: use NAME N pID)". The help line says a harmful scroll "is read at pID: it starts the fight", which is only true for people who are already enemies. So nothing a town does about assault could be reached.

### "Has to be done at a scribe's desk" — and nothing shows where one is
- Part: Buildings and interiors
- How bad: rough
- What happens: `make Sawasasih 75` → "Can't make scroll of paralysis: has to be done at a scribe's desk." `look places` lists houses and the out-of-town works only; the scribe's room, hall, market and guard post have no entry. Standing next to the scribe (`go p4014`) is not enough. Guessing the hidden id works: `go w2.17` (Leafshore) and then `make Sawasasih 75` starts the scroll; `go w25.30` walks to Windflat's hall.
- To see it: seed 34, copy `rk34-prefireball` → `select all` → `go t2` twice → `select Sawasasih` → `make Sawasasih 75` → `go w2.17` → `make Sawasasih 75`.
- Seen how many times: once.

### Shunning costs nothing: the thief keeps stealing in front of the watch, and a squadmate sells the loot to the same town
- Part: Crime and the law
- How bad: wrong outcome (unsure how much is meant; code-reading "smaller things" note on shunning: seen)
- What happens: Windflat (seed 3) shuns. Wehu is seen stealing, caught: "Windflat turns its back on Wehu for theft: no one will trade with them." No fine, nothing taken back, she stays "going through things" at the same barrel. Two minutes later she empties it, is seen and caught again: the very same line again and nothing more. `look`, `journal` and `town` never show that she is shunned. To Wehu the merchant just says "I don't know you. Say who you are and what you want." with no trade topic and no word of why. Menmuk, standing with her, then sells the silk wraps she had just stolen there (59 coin, which lands in Wehu's pack) and five more lots; minutes later all four merchants offer those silk wraps for 124. On day 7 Wehu can trade again; no line says the shunning ended.
- What should happen: shunning should bite the squad that travels with her (or at least the loot), a second offence while shunned should cost more, and the state should be visible.
- To see it: seed 3, copy `rk3-wf-chase` → `wait 1` ×3 → `takeall` → `wait 1` ×2 → (07:03, copy `rk3-wf-0703`) `talk p1610` → `bye` → `select Menmuk` → `talk p1610` → `say 6` → `say 13`.
- Output:
```
  06:36  Windflat turns its back on Wehu for theft: no one will trade with them.
  06:38  Wehu is seen stealing from a barrel! Waweyisis runs to fetch the watch!
  06:39  Hafith of the watch catches Wehu.
  06:39  Windflat turns its back on Wehu for theft: no one will trade with them.
TALKING with Tuqgor (Qotiro merchant):          <- Wehu
  » I don't know you. Say who you are and what you want.
    6. Come with us
TALKING with Tuqgor (Qotiro merchant):          <- Menmuk
    13. Sell silk wraps (59 coin)
  » I'll give you 59 for the silk wraps.
```
- Seen how many times: once (main line), end of shunning seen on the copy `rk3-days`.

### "Written into the record" justice: bound "for 1 days" with no sum named; a bond ends without a word
- Part: Crime and the law
- How bad: misleading
- What happens: Stilledge (seed 3, justice by record). "Heyuth's theft in Stilledge is written into the record; it will follow them." then "Heyuth is bound to work in Stilledge for 1 days". No fine is named, so the player can't tell what is owed or what a buy-out would cost. On the copy run through the week no line ever says she is free; by day 6 a `go` order simply works again. The same in seed 34: Ṭelihoḍu's "1 days" bond of day 1 still held at 12:26 on day 2 and was gone on day 3, silently.
- What should happen: name the debt; say when the bond is over.
- To see it: seed 3, copy `rk3-chase` → `wait 1` ×2. For the silent end: copy `rk3-ten` → `wait 1440` ×5 → `select Heyuth` → `go n 5`.
- Seen how many times: twice (one in each world).

### A hired guard, priestess and caravaner walk off with the squad the same morning; the posts are advertised at once; recruits starve quietly beside 9 coin
- Part: Town jobs and recruiting
- How bad: rough (the first part may be intended) / wrong outcome for the last line
- What happens: in Windflat six people were recruited in twelve minutes for 284 coin, among them a guard of the watch (Tonqox, 64 coin) and the priestess (Raama, 60), minutes after the same watch had caught a squad member twice. The town panel then lists "Guard at the guard post — 43 coin a day", "Priest at the shrine", "Caravaner at the market square" as work going. On the copy left to run: each recruit brought 3 flatbread; "weak with hunger" on day 5, "STARVING" on day 6, all six "down" by day 7. No News line marks any of it, none of them leaves, asks, or eats in the town they come from.
- Unsure, seen once: at dawn on day 6 the News says "06:00  Thuwah takes up work as weaver and sealer in Windflat." Thuwah is one of the recruits (p4966), in the squad and starving; nobody else of that name is among the 87 people in view. It looks as if the town gave a job to someone who had already joined the squad.
- To see it: seed 3, copy `rk3-ten` (day 1, 07:20, ten members, 9 coin) → `town` → `wait 1440` ×6, reading the News and the squad list.
- Seen how many times: once.

### A runaway's bounty is still open three days later, costs more, and the guards still say "Hello, traveller"
- Part: Crime and the law
- How bad: rough
- What happens: after the carry-out above, the squad stays away until day 8 and comes back. Both Woodsands guards open with a friendly greeting and offer "Pay my bounty"; the price is now "100 coin" (it was 60 on the day; 40 was owed in Leafshore for a picked lock, so it looks as if a Woodsands guard also collects Leafshore's bounty). Nobody comes for the runaway in the three days or on return.
- To see it: seed 34, copy `rk34-ranbond` → `wait 720` until day 8 → `select all` → `go t1` ×2 → wait to 08:00 → `select Nolu` → `talk p4972` → `say 6`.
- Seen how many times: once.

### Elders' fine with enough coin: "a fine of 12" and 11 coin leave the purse
- Part: Crime and the law
- How bad: cosmetic (unsure: perhaps rounding)
- What happens: squad has 18 coin; "Ṭelihoḍu is judged by the elders of Woodsands for theft: a fine of 12."; the header then shows 7 coin. No line says it was paid. He is not bound and stays "going through things" at the barrel.
- To see it: seed 34, copy `rk34-fine-chase` → `wait 1` ×3.
- Seen how many times: once.

### Taking your own food back out of someone's barrel is "seen stealing", and you are fined for it
- Part: Buildings and interiors / Crime and the law
- How bad: wrong outcome
- What happens: Heyuth puts her own 2 × dried fish (from her starting pack) into a barrel in a Stilledge cottage with `put`, then takes them back with `take 1`: "Heyuth is seen stealing from a barrel!" The third time a witness fetches the watch: "Heyuth's theft in Stilledge is written into the record; it will follow them." and the purse drops from 19 to 12 with no sum named. (`put` also moves the whole stack, so mixing your own flatbread with stolen flatbread and putting the stack back hands your own bread over too.)
- What should happen: what you put in is still yours (or `put` warns that you are giving it away).
- To see it: seed 3, copy `rk3-d3-dawn` → `select Heyuth` → `enter b4.6` → `search k4.6.1` → `pack Heyuth` (find Dried fish, entry 1) → then repeat: `put 1` (the fish's entry number), `wait 1`, `search k4.6.1`, `take 1` until the alert (it came on the 1st, and again on the 4th try with the watch fetched) → `wait 1` ×2.
- Output:
```
!! Heyuth is seen stealing from a barrel! Halelu saw it, but says nothing.
!! Heyuth is seen stealing from a barrel! Halelu runs to fetch the watch!
  06:53  Ḍoqari of the watch catches Heyuth.
  06:53  Heyuth's theft in Stilledge is written into the record; it will follow them.
Squad (10 members, 12 coin between them)        <- was 19
```
- Seen how many times: twice seen-stealing with her own fish, once caught.

### Step into the house next door and the watch gives up: "slipped the watch", and the guard then chats and lets you owe it
- Part: Crime and the law
- How bad: wrong outcome (code-reading candidate 9A in code-buildings-crime: seen; candidate 10 in code-law: seen)
- What happens: Tonqox (a guard of Windflat hired the day before) is seen stealing in Stilledge at 06:02. He walks into the forge 17 m away (`enter b4.8`) and waits. At 06:05: "Tonqox has slipped the watch in Stilledge. Bounty in Stilledge: 70." He then walks up to the guard who was after him; the guard says "Early, aren't you? The stone's still cold." and offers "Pay my bounty" → "You owe 70. You've got 0. Come back when you can pay." Twenty minutes later the same guard catches him for a new theft: he is bound "for 1 days" for that one alone, and the 70 is still only a talk topic ("Pay my bounty" with the official).
- What should happen: the watch looks indoors; a wanted man in front of a guard is taken; being caught settles what is already owed.
- To see it: seed 3, copy `rk3-d2-chase` (Tonqox seen, watch fetched, 06:02) → `enter b4.8` → `wait 1` ×4 → `go p54` → `talk p54` → `say 6`.
- Output:
```
  06:05  Tonqox has slipped the watch in Stilledge. Bounty in Stilledge: 70.
TALKING with Ḍoqari (Roduro guard):
  » Early, aren't you? The stone's still cold. Go on, then, what is it?
    6. Pay my bounty
  » You owe 70. You've got 0. Come back when you can pay.
```
- Seen how many times: once (main line; copies `rk3-d2-bounty`, `rk3-d2-chase2` mark the later steps).

### "Come with us" through an open talk from 1.5 km away: he joins and is at your side at once
- Part: Town jobs and recruiting
- How bad: wrong outcome (code-reading candidate 1, the Join case: seen)
- What happens: the talk with Fusa is open in Woodsands. Nolu walks 1,500 m east without `bye`, then `say 6`: "Nothing keeps me here. I'll get my things — lead on." The squad list shows Fusa "[1123 m from the others]" right beside Nolu's "[1125 m …]": he crossed 1.5 km in no time.
- To see it: seed 34, copy `rk34-prerecruit` → `go e 1500` → `look` → `say 6` → `look`.
- Seen how many times: once.

### A News line can come a command late, or twice
- Part: Odd input (the tool's News)
- How bad: cosmetic (code-reading candidate 15, the News filter: seen in this form)
- What happens: `select Fusa`, `pickup g53` prints "06:04  Fusa picks up flatbread." / "Fusa eats some flatbread."; `select Textrim`, `pickup g54` prints "06:04  Fusa picks up flatbread." again above Textrim's line. In the main line the first command printed no News at all and the line turned up under the next command.
- To see it: seed 34, copy `rk34-leaf-dawn` → `select Fusa` → `pickup g53` → `select Textrim` → `pickup g54`.
- Seen how many times: twice.

### Small things, one line each
- cosmetic: `bye` with no talk open answers "You take your leave."; a recruit's talk header reads "TALKING with Fusa (Ṭaḍoro no trade)" after he joins; `say -1` and `say x` answer "No topic 0."
- rough: topic numbers shift after every sale or buy-out, so repeating the same `say N` does a different thing (the second `say 16` sold a buckler instead of ingots; a second `say 6` quit the job just taken).
- rough: `look` lists what is near the middle of the whole squad. With one member bound in another town the list is centred on empty ground between the towns, so a lone member's surroundings (and the people they could talk to) can't be seen at all (seed 3, `rk3-ten`: the hall's officials never showed; seed 34: Nolu alone in Woodsands saw Leafshore's guards). Also seen: known item 5 (every member flagged "[N m from the others]").
- rough: the merchant's opening time in `town` changed overnight ("at their stall from 09:30 tomorrow", then on the day "from 12:45").
- misleading (unsure): a recruit advertised as "good at … inscription" (Hetheweyi) shows Inscription under "Not taken up yet" in `craft`.
- rough: a torch put out stays in the off hand; the buckler it replaced stays in the pack.
- rough: `wait` has no upper limit and, once the whole squad is down (known 2), nothing stops it: `wait 99999999` did not return in two minutes and had to be killed (20,000 minutes take about 23 seconds, so that one would run for more than a day). Copy `rk3-long` is a starved squad of four lying in the middle of Stilledge from day 8 to day 22 with nobody doing anything about them.
- also seen (known 3): by day every home showed "0% chance of being seen" from about 07:20 to at least 17:35 in four towns; the only windows with witnesses were 06:00–07:00 and after the 20:00 lock-up.
- also seen (known 4): members shown "asleep" picked things up ("06:06 Moker picks up buckler").
- also seen (known 6): "They couldn't get to it…" after a lock pick that was really seen; "Ṭelihoḍu has no lockpicks left; the lock holds." while Nolu picked it.
- also seen (known 2): a squad with 155 coin standing in a town with a merchant starved; nobody buys food.

### In one town nobody ever fetches the watch: seven sightings in a morning all became a bounty at once
- Part: Crime and the law
- How bad: rough (unsure whether intended: the guards were off duty)
- What happens: Woodsands in seed 3 (elders, irregular hours). Between 07:32 and 14:13 Wehu and then Raama were "seen stealing from a barrel!" seven times in the same cottage (the same 10 wild berries, put back and taken again). Not once did anyone "run to fetch the watch"; each sighting printed "Bounty in Woodsands: 75 … 90 … 105 … 120 … 132". Two guards stood 36 m and 59 m away the last two times; their Background says the guard post is 90–120 m off. The bounty is one number for the whole squad (Raama's first offence took it from 120 to 132), and a guard there offers "Pay my bounty" to a squadmate while chatting. The same happened at night in Leafshore (seed 34). So in such a town the bond and the elders' judgement can't be reached at all: only a bounty you pay when you like.
- What should happen: unsure; at the least a town with guards 36 m away should send one.
- To see it: seed 3, copy `rk3-ws-together` (14:06, whole squad in Woodsands, Wehu at the barrel) → `select Wehu` → repeat: `put` (the Wild berries entry from `pack Wehu`), `wait 1`, `search k3.6.1`, `take 1`.
- Output:
```
  07:45  Wehu is seen stealing from a barrel! Bounty in Woodsands: 75.
  11:27  Wehu is seen stealing from a barrel! Bounty in Woodsands: 105.
  14:08  Wehu is seen stealing from a barrel! Bounty in Woodsands: 120.
  14:13  Raama is seen stealing from a barrel! Bounty in Woodsands: 132.
```
- Seen how many times: seven sightings, one town, one day.

### `take` and `put` can be working a container 2.5 km away without saying so
- Part: Odd input / Buildings and interiors
- How bad: wrong outcome (the same fault as the two-containers finding, at long range)
- What happens: Heyuth was left "going through things" at a barrel in Stilledge. Wehu, alone in Woodsands 2.5 km north, does `search k3.6.1` (10 wild berries listed) and `takeall`: "Took 0 lots." Twelve rounds of `put N` / `take 1` then answer "Taken." each time while Wehu's berries stay in the barrel: the commands were moving things in and out of Heyuth's barrel in the other town. Whenever Wehu's `search` named a locked or missing container the tool printed "Heyuth opens a barrel (taking from it is theft: 0% chance of being seen)". Only after Heyuth was given a move order did `take 1` reach Wehu's barrel ("Wehu takes 10 × wild berries from a barrel.").
- To see it: seed 3, copy `rk3-d3-chase` (Heyuth at the Stilledge barrel) → `wait 3` → `select Wehu Menmuk Raama Totpok Doqoḍo` → `go t3` twice → `select Wehu` → `enter b3.6` → `search k3.6.1` → `takeall` → `put 5`. Or start from copy `rk3-farloot` (just after that `takeall`) → `put 5`: entry 5 of Wehu's pack is dried fish, but what goes into the barrel is entry 5 of Heyuth's pack.
- Output:
```
Wehu opens a barrel (taking from it is theft: 26% chance of being seen):
  1. 10 × Wild berries  (~1 coin each)
>>> takeall
Took 0 lots.
Heyuth opens a barrel (taking from it is theft: 0% chance of being seen):
 *Wehu       Horaro   going through things   ... [1384 m from the others]
  Heyuth     Ṭaḍoro   going through things   ... [1367 m from the others]
>>> put 5
Put away.
Heyuth opens a barrel (taking from it is theft: 0% chance of being seen):
  1. 2 × Ghostcap  (~4 coin each)
```
- Seen how many times: twice (main line, twelve commands in a row; and again from the copy).

## Coverage

Crime and the law
- steal in front of people at noon: tried. At noon every home is 0% (known 3); thefts in view were done at 06:00–07:00 (Stonehaven, Woodsands, Leafshore, Stilledge, Windflat).
- pick locks at noon in the market: tried in homes at 13:11 (two locks, unseen); the market has no id in the tool, so not at the market itself.
- caught on purpose, several times in the same town: tried. Stonehaven three times (duel), Woodsands twice (elders), Windflat twice (shunning), Stilledge three times (record).
- refuse to stop / keep walking: tried twice (caught after 230 m; caught within a minute). Stepping into a building instead: slipped the watch.
- caught with coin and with none: tried. None → bound (duel town, elders, record); part of the fine → taken silently and bound; enough → paid (elders 12, duel 22, record 7).
- fines, bonds: tried, including buy-out (works in Stonehaven and Stilledge, never offered in Woodsands).
- duels: tried, three in Stonehaven and one in Leafshore; won two, lost two.
- shunning: tried in Windflat (seed 3), twice on the same member, plus trading round it and waiting it out.
- run from a bond: tried. Not possible on foot; done by carrying the bound member out unconscious.
- come back at once: tried (nothing happens; 60 coin to a guard). Come back days later: tried (three days; 100 coin; same greeting).
- attack townsfolk and the watch: tried, always refused. Harmful spell: refused. Harmful scroll: made one, refused on townsfolk (and it crashes on a bad id).

Town jobs and recruiting
- recruit everyone, up to 10 and an 11th: tried in both worlds; the 11th is refused cleanly.
- what a recruit eats, no food: tried (three flatbread each; starve from day 5; nobody leaves).
- get rid of one: tried; there is no way in the tool (no command, no talk topic). Walking away leaves them in the squad list.
- town job: tried (works in the same town, paid at dawn; freezes on the road when the job is in another town).

Orders to people who can't act
- the bound: tried (go, enter, talk, attack, search, pickup, work, rest, sneak, torch, use, drop, give, cast).
- the downed: tried (go, talk; fed and carried).
- the asleep: tried (talk, cast, give, use, go: all work and wake them).
- the dead: not reached (nobody died in either world).
- the same order twice; an order to one while another is mid-something: tried (see the two-containers finding, the repeated `say N`, the repeated `go`).

Odd input: tried on every command in `help` except `shot` (select, go, attack, talk, say, bye, loot, search, take, takeall, put, spells, cast, pickup, gather, work, hunt, butcher, enter, carry, putdown, sneak, torch, rest, wake, pack, use, equip, drop, give, dose, unequip, craft, make, journal, map, town, wait, fight, look).

Save and load at awkward moments: tried five times, each time the save and a copy were continued with the same commands and the output compared with `diff`: mid-chase (same, once the `.session` file is copied too), during a duel (same), with a talk open (same), two members bound over a night and a dawn (same), ten members with a job, a bond and a bounty over a full day (same). No difference found.

Whole squad dead or gone: not reached.

## Candidates

From `code-law-jobs-recruit.md`
- 1 (talk stays open at any distance): seen, for taking a post, selling and Join.
- 2 (crash on a bad person id; crash with an empty squad): seen for `cast` and for a harmful scroll with `use`; empty squad not reached.
- 3 (two jobs at once): couldn't reach; nobody in Stilledge offered a second job on day 1 (14 people asked).
- 5 (limit of 10 only checked when recruiting): couldn't reach (needs a base). The limit itself holds in talk.
- 6 (silent walk to a far job): seen, and worse: he freezes on the road.
- 7 (talk sends member 0 / the selected even if bound): scenario A seen with the bound member selected; scenario B not tried.
- 9 (lost duel while the accused has left town): not seen. Tried three ways (accused sent off the moment the fight is listed; whole squad running south when caught, copy `rk34-run-duel`; ten-member squad in Leafshore): the duel was over before the accused had gone anywhere, and afterwards the bound member turns back toward town while the others walk on. Related notes seen: the town champion is fetched from wherever he is, even at 4% health.
- 10 (getting away is cheaper; bounty paid to anyone): seen.
- 14 (too much telling): seen: "saw it, but says nothing", the bond line's "An official there can sell you the bond", restless tags and prices, the town panel, alerts printed twice, "X is paid 19 coin for a day's work." The repeated "is bound… and can't leave" line: seen once per new move order when other news has come between, not on every order.
- 15 (bond state hidden; news lines dropped): bond state hidden: seen. Town post missing from the journal: not seen (the journal does list it). News line late or doubled: seen.

From `code-buildings-crime.md`
- 6 (`take` with no number; letter of the id ignored): seen for `take`, `pickup x6`, `gather p59`, `work h2`. `pickup 17` → g7: not seen (refused).
- 9 (guard can't see indoors; charges vanish): A seen (slipped the watch from the next building). B and C not tried.
- 12 (orders handed to members who can't act): seen in part (bound: talk burns 10 minutes, work/pickup silent; downed: go burns 30 minutes). The tool does refuse `search` for a bound member with a clear line.
- 13 (everything crashes with nobody left): not reached.
