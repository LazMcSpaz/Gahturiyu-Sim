# Play report: the THIEF (prefix `th`)

Worlds: seed 21 (`th21*`), seed 34 (`th34*`). All saves are in `/home/claude/hunt/saves/`.
A copied save needs its `.session` file copied too (it holds "news already read" and who is selected);
without it the first command re-prints old news. That is the tool, not a bug.

Start of seed 21: squad of 4 in Stillham (duel town), 0 coin. Numeleo has 6 lockpicks, Weyusauthi 2.

Summary: about 400 commands over the two worlds. 36 findings: 22 wrong outcome (4 of them marked unsure whether meant),
5 misleading, 6 rough, 3 unsure; no crash, nothing stuck. "Too much telling" items are gathered in the "Small things" lists.
Most serious, in my view:
1. Running beats the law (unless you run toward the guards or stop within 60 m): walk 100 to 120 m or step into the next house and the watch gives up in two minutes; the bounty is smaller than the fine, anyone takes the payment, and no guard ever acts on a bounty.
2. Theft pays even when caught: the loot is kept (45 coin stolen in sight, fine 32), each sighting in one `takeall` is fined again, and taking back your own things from a stranger's chest is fined as theft.
3. Buildings leak: a walk begun at 19:57 ends inside a house locked at 20:00; a chest is reached through the wall of a locked house; the squad sleeps in strangers' homes unnoticed.

## Findings (seed 21, session 1: day 1 morning)

### `takeall` goes on taking after the thief is seen, and can be seen twice in one go
- Part: Crime and the law
- How bad: wrong outcome
- What happens: one `takeall` on a 3-stack cupboard with a sleeper in the house prints two "is seen stealing" alerts and still takes all three stacks. On a 2-stack barrel the first stack was seen and the second was taken anyway.
- What should happen: stop at the first sighting (or roll once for the whole grab).
- To see it: seed 21, `th21-free-cup` (cupboard k5.2.0 open, 07:07 day 1) then `takeall`. Also `th21-barrel` then `takeall`.
- Output:
```
!! Ḍuhuquʻi is seen stealing from a cupboard! Sawasasih runs to fetch the watch!
!! Ḍuhuquʻi is seen stealing from a cupboard!
Took 3 lots.
News:
  07:07  Ḍuhuquʻi takes boots from a cupboard.
  07:07  Ḍuhuquʻi is seen stealing from a cupboard! Sawasasih runs to fetch the watch!
  07:07  Ḍuhuquʻi takes healing draught from a cupboard.
  07:07  Ḍuhuquʻi takes padded jacket from a cupboard.
  07:07  Ḍuhuquʻi is seen stealing from a cupboard!
```
- Seen how many times: twice from `th21-free-cup` (same result both times); once from `th21-barrel` (one sighting, second stack still taken); again in seed 34 (`th34-notetake4`: `takeall`).
- Note: bounty/fine after this was 48 (slipped) / 71 (duel lost) for goods "worth" 15+25+40 = 80. Seed 34 shows each sighting adds its own fine (30 for one sighting, 65 for two, same goods): see "Each sighting inside one `takeall` adds its own fine".

### Stepping into the house next door ends the chase: "slipped the watch", never arrested
- Part: Crime and the law
- How bad: wrong outcome
- What happens: seen stealing in longhouse b5.2, the whole squad walks into the next longhouse b5.4 (15 m away) and stands there. Two minutes later: "has slipped the watch". No guard comes in.
- What should happen: the watch follows through the door, or at least searches.
- To see it: seed 21, `th21-free-cup`: `takeall`, `enter b5.4`, `wait 1`, `wait 1`, `wait 2`.
- Output:
```
Ḍuhuquʻi, Numeleo, Ruqox, Weyusauthi are inside the Grown longhouse.
...
!! Ḍuhuquʻi has slipped the watch in Stillham.
(2 minutes pass.)
== Day 1, 07:11 (day) · Stillham ==
News:
  07:09  Ḍuhuquʻi has slipped the watch in Stillham. Bounty in Stillham: 48.
```
- Seen how many times: twice (`th21-free-chase` and `th21-free-chaseB`, identical output).
- Compare: standing still in b5.2 from the same save (`th21-free-chaseC`: `wait 1`, `wait 1`, `wait 1`) the watch arrives inside a minute and catches the thief.

### Getting away costs less than being caught, and the watch gives up in two minutes
- Part: Crime and the law
- How bad: wrong outcome
- What happens: seen at 06:00, the squad walks 300 m north; at 06:02 "has slipped the watch", bounty 12, while the HUD still says the squad is in Stillham. Staying put from the same save: caught at 06:01, duel lost, "must pay 18", no coin, bound 2 days. Second case: bounty 48 if you step next door, "must pay 71" (all 41 coin taken and a 4-day bond) if you stand still.
- What should happen: running should not be the cheap option; a bounty should not be smaller than the fine.
- To see it: seed 21, `th21-seen` (the instant after being seen). Run: `go n 300`. Stay: `wait 1`, `wait 1`, `wait 2`.
- Output (run):
```
!! Numeleo has slipped the watch in Stillham.
(3 minutes pass.)
== Day 1, 06:03 (day) · Stillham ==
  06:02  Numeleo has slipped the watch in Stillham. Bounty in Stillham: 12.
```
  Output (stay):
```
  06:01  Qotqix of the watch catches Numeleo.
  06:01  Numeleo must answer for theft by duel in Stillham.
  06:02  Numeleo's side loses the duel, and must pay 18.
  06:02  Numeleo is bound to work in Stillham for 2 days: led off to work it off. An official there can sell you the bond.
```
- Seen how many times: twice each way (barrel theft and cupboard theft).

### A wanted squad can stand in the middle of town; nobody acts on the bounty
- Part: Crime and the law
- How bad: wrong outcome (unsure: may be meant, but then a bounty has no teeth)
- What happens: three minutes after slipping the watch the squad walks back, stands 30 minutes in the open, walks to where a guard stood. Nothing happens. The merchant in the same town buys the stolen mussels.
- What should happen: a guard who sees a wanted person tries to arrest them.
- To see it: seed 21, `th21-slipped` (bounty 12, 06:08): `go s 300`, `wait 10`, `go p2455`, `wait 20`.
- Output: only `(10 minutes pass.)` / `(20 minutes pass.)`, no news.
- Seen how many times: once here; again later with a bounty of 48 (see below).

### The bounty can be paid to anyone: a carpenter, a merchant
- Part: Crime and the law
- How bad: wrong outcome / rough
- What happens: any townsperson offers "Pay my bounty". The merchant took 12 coin and said the matter was closed.
- What should happen: pay the watch or an official.
- To see it: seed 21, `th21-paytalk` (talking to carpenter p903, 0 coin): `say 6`. `th21-canpay` (talking to merchant p3765 with 24 coin): `say 6`.
- Output:
```
TALKING with Ḍuhushì (Roduro carpenter):
    6. Pay my bounty
    you: Pay my bounty
  » You owe 12. You've got 0. Come back when you can pay.
...
TALKING with Siyashifah (Ṭaḍoro merchant):
    you: Pay my bounty
  » 12 coin. Consider the matter closed — this time.
```
- Seen how many times: once each.

### The thief keeps everything stolen after being caught, tried and bound
- Part: Crime and the law
- How bad: wrong outcome
- What happens: after the lost duel and the bond, the 4 berries and 8 mussels are still in Numeleo's pack; a squadmate then sells the mussels to the town merchant. Same later with boots, healing draught, padded jacket (and a 120-coin iron helm stolen unseen).
- What should happen: what was seen stolen goes back to its owner on arrest.
- To see it: seed 21, `th21-bound` (06:31, Numeleo bound): `pack Numeleo`.
- Output:
```
   11. 4 × Wild berries [food, 8]  (worth ~1 each; nobody in Stillham pays for it)
   12. 8 × Mussels [food, 14]  (worth ~2 each; sells for 1 in Stillham)
```
- Seen how many times: twice (two separate arrests).

### Trading reaches every squad member's pack at any distance, the bound one included
- Part: Crime and the law (bonds) / Odd input
- How bad: wrong outcome
- What happens: one member talks to the merchant; "Sell all 8 × mussels" sells the mussels out of the pack of a bound member standing 60 to 138 m away, and the coin lands in that far member's pack.
- What should happen: only the talker's pack (or members standing by) trade.
- To see it: seed 21, `th21-bound`: `select Ruqox`, `wait 30`, `talk p3765`, `say 6`, `say 8` ("Sell all 8 × mussels — 8 coin"), `bye`, `look`, `pack Numeleo`. (Checked again on a fresh copy, `th21-tradecheck`.)
- Output:
```
  » 8 of them — 8 coin. Pleasure.
  Numeleo    Horaro   standing               health 100% stamina 100% load 41/72 kg [100 m from the others]
   12. 8 × Coin [money]  (worth ~1 each)
```
- Seen how many times: four times (twice on the bound line, once on the run line where the talker was "[138 m from the others]", once in seed 34 where the exchanger changed a note out of the bound member's pack 65 m away).

### The bond can only be asked about when the official is at his post, and nothing says so; bond state is shown nowhere
- Part: Crime and the law
- How bad: rough / misleading
- What happens: the news says "An official there can sell you the bond." At 06:41 the official's topic list has no bond line (and no work line). At 07:02 the same official shows "Buy Numeleo out of their bond (18 coin)". In Woodsands: no bond line at 10:43 on day 8 (official 41 m away from where he sat before), "Buy Nolu out of their bond (9 coin)" at 13:43 the same day. Off his post he says nothing about it ("come back later" or the like). The price and the days left are shown nowhere else (`look`, `journal`, `town`); the bound member's status reads "standing".
- What should happen: the official says when to come back; the squad list says who is bound and for how long.
- To see it: seed 21, `th21-bound`: `select Ruqox`, `talk p334` (topics: Background, This town, A little advice, Latest rumours, Bandits, Come with us, Goodbye). `th21-buyout` is the same talk at 07:02.
- Output (07:02):
```
    6. Buy Numeleo out of their bond (18 coin)
    7. I'll work as labourer and porter at the food shop (19 coin a day, 8 till 5)
```
- Seen how many times: twice (Stillham, Woodsands).
- Not a bug: the buy-out price falls as days are served (59 on the day of the bond, 9 on the sixth day), and buying out frees the member at once (`th21-buyout`: `say 6`).

### Orders to a bound member fail silently or waste ten minutes
- Part: Crime and the law
- How bad: rough
- What happens: with the bound member selected, `work d6`, `pickup g2` and `enter b5.1` print "(0 minutes pass.)" or "Nobody got inside…" with no reason; `talk p334` burns 10 minutes then "They're not talking (or you couldn't reach them)." `go` and `give` do explain ("is bound to work off a bond"). `rest`, `sneak`, `use`, `drop` and picking up something at their feet all work.
- What should happen: one clear refusal for every order.
- To see it: seed 21, `th21-bound`: `select Numeleo`, then each command.
- Seen how many times: once each.

### The save file is not the same after two identical continuations
- Part: Save and load
- How bad: unsure (output was identical line for line; only the file differs)
- What happens: `th21-free-chase` and `th21-free-chaseB` were made from one save and given the same six commands. Printed output is identical, but the saves differ in three places: a saved list of the three buildings the squad has been inside is written in a different order ((5,18),(5,2),(5,4) against (5,2),(5,4),(5,18)).
- What should happen: byte-identical saves (CLAUDE.md rule 12: nothing may depend on a hash map's order). If anything ever walks that list in order, two loads could diverge.
- To see it: `cmp -l saves/th21-free-chase.save saves/th21-free-chaseB.save` (bytes 2070938, 2070942, 2070946). With only one building visited (`th21` against `th21-seenB`) the saves were identical.
- Seen how many times: once (one pair).

### Small things (session 1)
- misleading: after stealing from an unlocked barrel (no lock touched), a carpenter's "Latest rumours" says "Someone's been at the locks round here lately." (`th21-paytalk`, `say 4`).
- too much telling: owned things lying inside houses are listed from 30 to 50 m away through walls, with their odds ("g0 1 × Leather gloves on the ground (someone's: taking it is theft, 28% chance of being seen) — 49 m north").
- too much telling: "Fehawesis runs to fetch the watch!" names a stranger (a sleeper the squad never saw; they never appear in `look people`).
- rough: the town's duel champion (p2067 Qotqix, a night guard) and the guard who made the arrest are gone from `look people` the instant the duel ends (`th21-free-chaseC` after three `wait 1`): they seem to appear from bed and vanish back.
- rough: the squad's duellist was the staff-carrying mage Weyusauthi while two unhurt fighters stood beside him (`th21-free-chaseC`).
- rough (unsure): `enter b5.2` from 10 m away printed "(0 minutes pass.)" yet everyone lost about 19% stamina; then `go n 300` (3 minutes) changed nobody's stamina at all.
- also seen (known 3): by day, chest k5.18.0 (31 coin, silk wraps 70, iron helm 120) shows 0% and `takeall` is never seen.
- also seen (known 5): "[N m from the others]" flags on everyone when one member is away.
- also seen (known 6): alerts print twice.


## Findings (seed 21, session 2: bonds, the 20:00 lock, night, walls)

### A walk that starts at 19:57 ends inside a house that locked at 20:00
- Part: Buildings and interiors
- How bad: wrong outcome
- What happens: from 300 m away at 19:57, `enter b5.18`. At 20:01 all four "are inside the Grown cottage", and the same `look` lists the cottage as locked. No pick, no trespass.
- What should happen: stop at the locked door.
- To see it: seed 21, `th21-free-1957` (19:57, squad 300 m south of the cottage): `enter b5.18`, `look places`.
- Output:
```
(4 minutes pass.)
Ḍuhuquʻi, Numeleo, Ruqox, Weyusauthi are inside the Grown cottage.
== Day 1, 20:01 (night) · Stillham ==
  k5.18.0  a chest — right here
  b5.18  Grown cottage (locked: a simple lock (17): Numeleo ~79% a try, 6 lockpicks (about 1 in 3 snaps on a miss)) — 5 m north
```
- Seen how many times: once.

### The squad beds down in a stranger's locked home beside the sleepers and nobody minds
- Part: Buildings and interiors / Crime and the law
- How bad: wrong outcome (unsure if meant)
- What happens: standing in the cottage from 20:01, the squad "beds down for the night" at 22:00 by itself. By 23:01 the owners are home asleep (the kelp frond on the floor now reads 28% instead of 0%). No one notices four strangers asleep in the room, then or at any time.
- What should happen: residents coming home to strangers in their house react (trespass).
- To see it: seed 21, `th21-free-inlocked` (20:01 inside the locked cottage): `wait 180`, `look`.
- Output:
```
  22:00  Ḍuhuquʻi beds down for the night.
  22:00  Numeleo beds down for the night.
  ...
  g3  1 × Kelp frond on the ground (someone's: taking it is theft, 28% chance of being seen) — right here
```
- Seen how many times: once. Saved after it: `th21-free-asleepin`.

### A chest can be emptied (and filled) from outside the wall of a locked house
- Part: Buildings and interiors
- How bad: wrong outcome
- What happens: at 23:01 with the cottage door locked, a member who opened the chest inside steps 2.2 m south-east. `take 1` says "Taken."; `search` on the same chest from the same spot says he is not in the building. `put 0` works from there too. At 2.0 m `search` still works (inside); at 2.5 m `take` fails (out of reach). So the wall is between, and reach goes through it.
- What should happen: no reach through walls.
- To see it: seed 21, `th21-wall5-base` (Ḍuhuquʻi selected, chest k5.18.0 open with 2 flatbread in it): `go se 2.2`, `take 1`, `search k5.18.0`, `put 0`.
- Output:
```
>>> take 1
Taken.
>>> search k5.18.0
Can't go through that (is it locked, or not in a building you're in?).
>>> put 0
Put away.
Ḍuhuquʻi opens a chest (taking from it is theft: 26% chance of being seen):
  1. 2 × Torch  (~4 coin each)
```
- Seen how many times: once at 2.2 m (`th21-w2.2`), with controls at 1.5, 2.0, 2.5, 2.8 m (`th21-w1.5` … `th21-w2.8`).
- Also: in `th21-w2.8` the member went out of reach ("Nobody is going through anything right now"), came back toward the wall (`go se 1.5`, `go se 1.4`) and `take 1` said "Taken." again: the open chest stays "open" for him after he has walked away. By day at 20 m and more the entry was not usable (`th21-chestopen`: `go b5.27`, `take 1`).

### Ordering the same lock-pick again replays the same result
- Part: Buildings and interiors
- How bad: wrong outcome
- What happens: the hard chest k5.2.1 (3% a try). Each fresh `search k5.2.1` gives the very same first try: the pick snaps and Shihuwaufi sees it. Three orders, three snapped picks, three sightings.
- What should happen: a new attempt is a new roll.
- To see it: seed 21, `th21-free-prechest` (Numeleo selected, inside b5.2 at 23:11): `search k5.2.1`, `wait 1`, `search k5.2.1`, `wait 1`.
- Output:
```
  23:12  Numeleo's lockpick snaps (3% a try at a hard lock).
  23:12  Numeleo is seen picking a lock! Shihuwaufi saw it, but says nothing.
...
  23:12  Numeleo's lockpick snaps (3% a try at a hard lock).
  23:13  Numeleo is seen picking a lock! Shihuwaufi runs to fetch the watch!
```
- Seen how many times: twice in a row on each of two copies (`th21-pickA`, `th21-pickB`) and once more on `th21-free-chase` at 23:16.
- Note: the two doors picked that night (b5.14 at 49%, b5.2 at 38%) both went "snaps, then picks the lock"; could be chance.

### After a lock is picked, `enter` never says whether anyone got in; a member already inside is told "It's locked"
- Part: Buildings and interiors
- How bad: misleading / rough
- What happens: `enter b5.14` with the lockpicker: "(5 minutes pass.)", news "picks the lock", and no "is inside" or "nobody got inside" line. With the squad already inside the locked cottage, `enter b5.18` for a member without picks says "It's locked, and nobody selected has a lockpick."; for the lockpicker it "picks the lock" of the house he is standing in.
- What should happen: say who is inside; don't pick a lock from the inside.
- To see it: seed 21, `th21-free-prepick`: `enter b5.14`. And `th21-free-inlocked`: `select Ḍuhuquʻi`, `enter b5.18`; `select Numeleo`, `enter b5.18`.
- Seen how many times: twice (two doors); the inside case once each.

### `search` on the locked chest shows another member's old barrel
- Part: Buildings and interiors
- How bad: misleading (also seen: known 6)
- What happens: Numeleo is sent to the locked chest k5.2.1 in the longhouse; the tool prints "Ruqox opens a barrel" (the barrel Ruqox opened earlier in another house 30 m away). The pick goes on unseen by the player until the next `wait`.
- To see it: `th21-free-prechest`: `search k5.2.1`.
- Seen how many times: five times.

### Walking 120 m down the street is enough to "slip the watch"; 60 m is not
- Part: Crime and the law
- How bad: wrong outcome
- What happens: the thief alone walks 120 m north from the house and stands in the open, still in town, squad still in the house: "has slipped the watch" two minutes after the sighting. Walking the whole squad 30 m or 60 m: caught at 06:01.
- What should happen: a chase that follows someone in plain sight.
- To see it: seed 21, `th21-seen`: `select Numeleo`, `go n 120`, `wait 1`, `wait 1`. Compare `th21-seen`: `go n 60`, `wait 1`.
- Output:
```
  06:02  Numeleo has slipped the watch in Stillham. Bounty in Stillham: 12.
```
- Seen how many times: once each (`th21-duelrun`, `th21-walk30`, `th21-walk60`).

### Days later the bounty is still there, and guards do nothing about it, even asked to their face
- Part: Crime and the law
- How bad: wrong outcome
- What happens: three days away, back on day 4 with the 12-coin bounty and no coin. The squad walks into a knot of six guards and stands five minutes: nothing. Talking to guard Qotqix (the one who chased them): "Pay my bounty" → "You owe 12. You've got 0. Come back when you can pay." and that is all. Earlier the same squad stood 12 hours in town with a 48 bounty (`th21-free-chase`, `wait 735`): nothing.
- What should happen: the watch arrests a wanted person it meets, at least one who can't pay.
- To see it: seed 21, `th21-later-guardtalk` (talking to the guard, day 4 07:02): `say 6`.
- Seen how many times: once after days; twice same day.

### A bond ends without a word, hours after the stated days, and the bound member never sleeps
- Part: Crime and the law
- How bad: rough (I first took this for a day too many; it is not)
- What happens: bound at 06:02 on day 1 "for 2 days". Still bound at 12:00 on day 3 ("Nobody moves: Numeleo is bound to work off a bond."), free at 12:02. In seed 34, "for 7 days" from 07:16 on day 2: still bound at 16:13 on day 9, free at 17:03. So the stated days are rounded down (2 days is 2 days 6 hours; 7 days is about 7 days 9 hours). No news line says the bond is over, on either world. Through the whole bond the member's line reads "standing … worn out" day and night (the free members bed down at 22:00 each night), unless he is told to `rest`.
- What should happen: a line when it ends; the bound sleep like anyone; say "2 days and a bit" or round up.
- To see it: seed 21, `th21-d3noon` (day 3 11:31, Numeleo selected): `go n 50` (refused), `wait 29`, `go n 5` (refused), `wait 2`, `go n 5` (moves), `look` (no news about it).
- Seen how many times: once each world, bisected from saved copies.

### A fine takes every coin the squad has, then binds anyway, and the old bounty is left standing
- Part: Crime and the law
- How bad: unsure (may be meant; reported for the numbers)
- What happens: "must pay 71" with 41 coin → 0 coin and "bound … for 4 days". "must pay 60" with 41 coin → 0 coin and "bound … for 2 days". With 0 coin and "must pay 18" → 2 days. The coin is taken from all four packs wherever they stand. In the second case another member (Ḍuhuquʻi) still carried a 48 bounty in the same town; the arrest of Numeleo did nothing about it.
- To see it: `th21-free-chaseC` (three `wait 1`); `th21-free-chest2` (`search k5.2.1`, `wait 2`, `wait 2`).
- Seen how many times: once each.

### A downed, bound member can be picked up and carried off by a squadmate
- Part: Crime and the law
- How bad: rough (unsure)
- What happens: Numeleo loses his own duel, is down and bound. `carry p5001` works; the carrier walks north; five minutes later Numeleo comes round and is "put down" about 400 m from where he was bound and just stands there (not pulled back, not called a runaway, 30 minutes on).
- To see it: seed 21, `th21-free-downbound`: `select Ḍuhuquʻi`, `carry p5001`, `go n 900`, `wait 10`, `wait 30`.
- Seen how many times: twice.

### "The downed will come round in an hour or two" but they are up in 5 to 25 minutes
- Part: Crime and the law
- How bad: misleading
- What happens: Numeleo down at 23:19, "puts Numeleo down" / standing at 23:26. Ḍuhuquʻi down at 06:02, standing at 06:28.
- To see it: `th21-free-downbound`: `wait 10`.
- Seen how many times: twice.

### Small things (session 2)
- too much telling: "Numeleo is seen picking a lock! Shihuwaufi saw it, but says nothing." (names a hidden sleeper and her private choice.)
- too much telling: exact lock numbers and odds on every door and chest in `look` ("a fair lock (42): Numeleo ~37% a try, 5 lockpicks (about 1 in 3 snaps on a miss)"), and the odds again in each snap line.
- too much telling: a steady stream of pushed lines about strangers: "Sayasesas (Ṭaḍoro), wandering, crosses your path." (dozens a day in the wilds), "Aniruo and 1 other, bound for Oldedge." (hourly in town), "Fauwashuwi takes up work as meal runner in Stillbank." (a town 1 km off), "Stillbank comes into view." repeated at 05:59 each day while standing still.
- rough: the accused can be given orders and walk off while his champion fights the duel (`th21-duelstart`: `select Numeleo`, `go n 500`); he got 82 m before the duel ended and was then led back. I could not get him out of town in time, so "bound while away" (candidate 9 in the law report) was not reached.
- rough: `talk p2067` to a guard listed "right here" took "(10 minutes pass.)" before the talk opened.
- rough: `put 0` then `take 1` of your own flatbread from a stranger's chest is logged as "takes 2 × flatbread from a chest" under "taking from it is theft: 26% chance of being seen" (not seen being charged; unsure).
- rough: by day the lock line named "Weyusauthi ~3% a try, 2 lockpicks" for the hard chest while Numeleo (6 picks, the better hand on doors) stood beside him.
- also seen (known 4): a member shown "asleep" searched a barrel (`th21-free-asleepin`: `search k5.18.1`).
- also seen (known 7): `go n 800` for three members with the fourth far away moved them 250 m the second time.

## Findings (seed 34: Stonehaven is a duel town, Woodsands is judged by its elders)

Start: squad of 4 in Stonehaven, 0 coin. Nolu has 6 lockpicks, Sawasasih 2.

### Some sightings post a bounty without ever saying so
- Part: Crime and the law
- How bad: misleading (the player is never told he is wanted)
- What happens: at night in Woodsands (no guard about), a sighting prints only "!! Nolu is seen stealing from a chest!" with no witness, no "fetch the watch", no "says nothing", and nothing follows. Four of those in ten minutes. Next morning a farmer's topic list has "Pay my bounty": "You owe 130." No line ever said a bounty was posted, and no panel (`look`, `journal`, `town`) shows a bounty at any time.
- What should happen: say that a bounty was posted (as the "slipped the watch … Bounty in X: N" line does), and show bounties somewhere.
- To see it: seed 34, `th34-notetake` (23:05, Nolu at chest k1.4.1 holding his spear and leggings in it): `takeall`. Then `th34-silentbounty` (07:08 next day, talking to farmer p2280): `say 6`.
- Output:
```
!! Nolu is seen stealing from a chest!
Took 2 lots.
...
    you: Pay my bounty
  » You owe 130. You've got 36. Come back when you can pay.
```
- Seen how many times: four bare sightings in one night (130 owed). One bare sighting alone leaves 30 owed (`th34-note-seen`: `select all`, `wait 485`, `talk p2280`, `say 6` → "30 coin. Consider the matter closed — this time."). In seed 21 the bare second line of a double sighting came with a chase already on and left no separate bounty after the arrest. My guess: the bare line is what prints when there is no guard free to fetch.

### Notes do not count for fines, bonds or bounties, though the HUD counts them as coin
- Part: Crime and the law
- How bad: wrong outcome
- What happens: the squad holds 6 coin and one 50-coin note; the HUD says "56 coin between them". Fined 65: the 6 coin go, the note stays, and Nolu is "bound … for 7 days" with the HUD still reading "50 coin between them". The official: "Buy Nolu out of their bond (59 coin)" → "Not so fast: not enough coin." Bounty: "You owe 130. You've got 0."
- What should happen: notes pay, or the HUD stops calling them coin.
- To see it: seed 34, `th34-note-watch2` (seen at 07:15 day 2, 6 coin + 1 note): `wait 1`, `wait 1`. Then `th34-bondtalk` (talking to official p4417): `say 7`, `say 6`.
- Output:
```
Squad (4 members, 56 coin between them) — selected marked *:
  07:16  Nolu is judged by the elders of Woodsands for theft: a fine of 65.
  07:16  Nolu is bound to work in Woodsands for 7 days: led off to work it off. An official there can sell you the bond.
Squad (4 members, 50 coin between them) — selected marked *:
```
- Seen how many times: once (bond); twice (bounty lines, with 36 coin + note and with 0 coin + note).

### Each sighting inside one `takeall` adds its own fine
- Part: Crime and the law
- How bad: wrong outcome
- What happens: the same two things (spear ~50, hide leggings ~40) taken from the same chest by the same thief. One sighting: "a fine of 30". Two sightings in one `takeall`: "a fine of 65".
- What should happen: one grab, one charge.
- To see it: seed 34, `th34-note-watch` (one sighting): `wait 1`, `wait 1`. `th34-note-watch2` (two sightings): `wait 1`, `wait 1`.
- Output:
```
!! Nolu is seen stealing from a chest! Roshu runs to fetch the watch!
!! Nolu is seen stealing from a chest!
Took 2 lots.
  07:16  Nolu is judged by the elders of Woodsands for theft: a fine of 65.
```
- Seen how many times: once each.

### Stealing coin in plain sight pays even when caught: 45 coin stolen, fine 32, coin kept
- Part: Crime and the law
- How bad: wrong outcome
- What happens: Moker is seen taking 45 coin and a sheet of reed paper from a chest. Caught a minute later and judged by the elders: "a fine of 32". The 45 coin stay with the squad (84 → 52). In Stonehaven: 42 coin stolen unseen, then seen at the barrel; duel lost, "must pay 22", 20 left over.
- What should happen: what was stolen is given back, and the fine is on top.
- To see it: seed 34, `th34-ws-seen` (07:11, 84 coin): `wait 1`.
- Output:
```
  07:12  Ḍeshu of the watch catches Moker.
  07:12  Moker is judged by the elders of Woodsands for theft: a fine of 32.
Squad (4 members, 52 coin between them):
```
- Seen how many times: once each town.

### Taking your own things back out of a stranger's chest is theft, and is fined
- Part: Buildings and interiors / Crime and the law
- How bad: wrong outcome
- What happens: `put 0` four times puts the squad's own flatbread, dried fish, torches and healing draught into a chest; `takeall` takes them back; a sleeper sees it; the watch comes; duel lost; "must pay 34".
- What should happen: your own things stay yours (or `put` into someone's chest warns that you are giving them away).
- To see it: seed 34, `th34-own1` (00:01 day 2, four own stacks in chest k0.2.1): `takeall`, `wait 1`, `wait 1`, `wait 1`.
- Output:
```
!! Ṭelihoḍu is seen stealing from a chest! Heʻaḍoʻa runs to fetch the watch!
Took 4 lots.
  00:02  Ṭelihoḍu must answer for theft by duel in Stonehaven.
  00:03  Ṭelihoḍu's side loses the duel, and must pay 34.
```
- Seen how many times: once in Stonehaven; again in Stillham (`th21-lockedin-pre`: `takeall`, three `wait 1`: "must pay 60").

### A sleeping squadmate is made the duel champion, and the accused sleeps through his own arrest and trial
- Part: Crime and the law
- How bad: wrong outcome
- What happens: all four are "asleep" in the house. The watch "catches Ṭelihoḍu" (still shown asleep). The champion is Sawasasih, asleep; he is at 11% health on the first line of the fight, loses, and ends up "[43 m from the others]", listed "standing … health 0%", then "asleep" again a minute later. Ṭelihoḍu never wakes.
- What should happen: an arrest wakes people; a sleeper is not sent to fight.
- To see it: seed 34, `th34-own-seen`: `wait 1`, `wait 1`, `wait 1`, `wait 1`.
- Output:
```
  Ṭelihoḍu   Roduro   asleep                 health 100% stamina 100% load 17/77 kg
  Sawasasih  Ṭaḍoro   fighting               health  11% stamina 100% load 15/52 kg
...
  Sawasasih  Ṭaḍoro   standing               health   0% stamina  25% load 14/53 kg (Toughness 31 ↑) [43 m from the others]
```
- Seen how many times: once.

### An arrest ignores the bounty the thief already carries
- Part: Crime and the law
- How bad: wrong outcome
- What happens: Nolu carries a 130 bounty in Woodsands. Ḍeshu of the watch catches him twice more that morning; the elders fine him 30 and then 65 for the new thefts; the 130 is still owed afterwards. In Stonehaven Nolu's 15 bounty was untouched when squadmate Ṭelihoḍu was caught and tried there.
- What should happen: being caught settles (or adds) what is already owed.
- To see it: seed 34, `th34-note-watch`: `wait 1`, `wait 1`; then `th34-bondtalk`: `say 6` ("You owe 130.").
- Seen how many times: twice.

### A bound member told to `rest` sleeps through the working day
- Part: Crime and the law
- How bad: rough
- What happens: Nolu, bound at 07:16 and "led off to work it off", is told `rest` at 10:20 and is "asleep" until evening. Left alone, a bound member never sleeps at all (seed 21: "worn out" for three days).
- To see it: seed 34, `th34-bondtalk`: `bye`, `select Nolu`, `rest`, `wait 30`.
- Seen how many times: once.

### Walk-in at night, unlocked house: the wanted squad sleeps among the people it robbed that morning
- Part: Buildings and interiors
- How bad: wrong outcome (unsure if meant)
- What happens: Stonehaven longhouse b0.2 does not lock at night. The squad (bounty 15 for robbing this house at 06:02) walks in at 20:01, beds down at 22:00, and sleeps beside the residents (chest odds 25%). Nobody notices.
- To see it: seed 34, `th34-1957`: `go k0.2.1`, `wait 240`, `search k0.2.1`.
- Seen how many times: once (and once in seed 21 in a locked cottage).

### The watch catches a thief inside a house whose door is still locked
- Part: Crime and the law
- How bad: unsure
- What happens: the squad is inside locked cottage b5.18 at 23:03 (it got in at 20:01). Seen stealing; two minutes later "Krogdod of the watch catches Ḍuhuquʻi"; `look` still lists the cottage as locked. Yet when a thief steps into an unlocked house next door the watch never comes in (see "Stepping into the house next door").
- To see it: seed 21, `th21-lockedin-seen`: `wait 1`, `wait 1`, `wait 1`, `look places`.
- Seen how many times: once.

### Small things (seed 34)
- rough: `search` on a locked chest sends the nearest member even if he has no picks, though `look` advertises the one who has: "Moker has no lockpicks left; the lock holds." (He never had any.) The same stale line then prints after Nolu's successful picks (`th34-inhouse`: `search k0.2.1`).
- rough: `search` on a locked chest cost 10 minutes of game time before printing (10:05 → 10:15, `th34` in b1.4).
- rough: `pickup g4` (a club) printed nothing; the club was in the pack. News lines stamped in the same minute as the last one read are dropped (also after `takeall` at 23:05: the "takes …" lines never showed).
- too much telling: "Moker is seen stealing! Theloḍuʻa saw it, but says nothing."; "Bandits fall on travellers 1433 m away." (at night, exact metres).
- also seen (known 6): "Nolu picks the lock." followed by "They couldn't get to it: they couldn't reach it in time…".
- also seen (trading at a distance): the exchanger changed the note out of bound Nolu's pack 65 m away; the 49 coin landed in the talker's pack.
- mid-chase save/load (seed 34): `th34-runA` and `th34-runB` from `th34-seen`, ten identical commands, output identical; the two save files again differ (8 bytes).
- Stonehaven, same pattern as seed 21: seen at 06:02, the squad walks 100 m east and stands; "slipped the watch" at 06:04, bounty 15. Standing still from the same save: caught at 06:03, "must pay 22".

## Findings (seed 34, later: Newshaw, the long bond, odd input)

### Newshaw's judgement takes 5 coin without saying so
- Part: Crime and the law
- How bad: misleading
- What happens: Newshaw's custom is the record. Caught for picking up a kelp frond, the only line is "Moker's theft in Newshaw is written into the record; it will follow them." The squad's coin goes from 50 to 45 in the same minute (out of Nolu's pack; Nolu was not the one tried). No line mentions a fine or the 5 coin. What "it will follow them" means is not shown anywhere afterwards.
- What should happen: say what was taken and from whom.
- To see it: seed 34, `th34-newshaw-seen` (07:48 day 2, 50 coin): `wait 1`, `wait 1`.
- Output:
```
Squad (4 members, 50 coin between them):
...
!! Purekem of the watch catches Moker.
Squad (4 members, 45 coin between them):
News:
  07:49  Purekem of the watch catches Moker.
  07:49  Moker's theft in Newshaw is written into the record; it will follow them.
```
- Seen how many times: once.

### The guard chasing a thief chats with the thief's friend 60 m away and makes the arrest in the same minute; the talk never closes
- Part: Crime and the law
- How bad: wrong outcome / rough
- What happens: while Ḍeshu of the watch is on his way to Moker, another member walks 60 m to Ḍeshu and `talk`s: "A conversation opens." One minute later "Ḍeshu of the watch catches Moker" back at the house, and the talk with Ḍeshu is still open and stays open through `wait 1`, `wait 2`, `wait 5`.
- What should happen: a guard on a chase doesn't stop to chat; a talk ends when the two part.
- To see it: seed 34, `th34-ws-seen`: `talk p507`, `wait 1`, `wait 2`, `wait 5`.
- Output:
```
(1 minutes pass.) A conversation opens.
TALKING with Ḍeshu (Roduro guard):
...
!! Ḍeshu of the watch catches Moker.
  07:12  Moker is judged by the elders of Woodsands for theft: a fine of 32.
TALKING with Ḍeshu (Roduro guard):
```
- Seen how many times: once.

### The same witness fetches the watch for one theft and "says nothing" about the next one a minute later
- Part: Crime and the law
- How bad: rough
- What happens: Womuwira (awake, 11 m away, by day) sees Moker take a kelp frond: "runs to fetch the watch!". He does not move (still listed 12 m away a minute later). The next minute he sees Nolu take salted meat: "saw it, but says nothing."
- To see it: seed 34, `th34-newshaw-day` (07:47 day 2 in living quarters b10.2): `pickup g2`, `pickup g1`, `look people`.
- Output:
```
  07:47  Moker is seen stealing! Womuwira runs to fetch the watch!
  07:48  Nolu is seen stealing! Womuwira saw it, but says nothing.
  p2992  Womuwira — Horaro labourer and porter — 12 m south-east
```
- Seen how many times: once.
- Note for known item 3: here the odds by day were 51 to 59% (an awake person inside a Qotiro living quarters), so daytime theft can be seen in that kind of building.

### `take 0` and `take -1` steal line 1
- Part: Odd input
- How bad: wrong outcome (small)
- What happens: with a chest open, `take 0` takes line 1 (42 coin) and `take -1` takes the next line 1 (trousers). `take 99` and `take x` say "Couldn't take that." (`take x` was tried on an empty chest only.)
- To see it: seed 34, `th34-chestopen`: `take 0`, `take -1`.
- Output:
```
>>> take 0
Taken.
  06:02  Nolu takes 42 × coin from a chest.
```
- Seen how many times: once each.
- Other odd input was refused cleanly: `put 99`, `put -1`, `put x`, `search k0.2.99`, `search kx`, `pickup g999`, `enter b0.999`, `enter w0.8`, `enter`, `search`; `carry`/`loot`/`attack`/`talk` on a townsperson asleep indoors (p3248).

### Small things (later)
- misleading: with three free members selected, `search k1.2.0` printed "Nolu was caught and is bound to work it off (see News)." (Nolu was not selected and was 70 m away; a stale entry from the night before). `th34-bondwait`: `enter b1.2`, `search k1.2.0`, `search k1.2.1` (it printed on the second search when re-run from the copy).
- rough: after two convictions the Woodsands merchant says "I've nothing to say to your sort. Move along." to every squad member, including one never accused, and cannot even be paid the bounty; the exchanger next to him deals as usual. (May be meant.)
- rough: Moker's status read "going through things" for seven days after his last `takeall`.
- too much telling: "A Mirejaw falls on travellers 2273 m away."; "Punpeg takes up work as armourer in Woodsands." (and four more of the kind at later dawns); the `town` panel lists who would join, by id, with "asleep indoors".
- not a bug: a bounty is per town (wanted in Stonehaven, no bounty topic in Newshaw); sneaking lowers the shown odds a little (26% → 22% at the same barrel).
- observation: no guard was ever on the street at night in either world (Stillham 23:01, Woodsands 23:27, lists of 5 to 7 travellers only), so doors were picked in the open unseen (b5.14, b5.2); the only night witnesses were sleepers inside.

## Coverage

Buildings and interiors
- entering and leaving: tried (dozens of `enter`; leaving a locked house from inside is free).
- locked doors (20:00): tried (`th21-free-1957`; seed 34 houses b0.2 and b1.4 never showed as locked at night).
- picking locks by day: tried (chests only: doors are open by day; k0.2.1, k1.3.0, k1.4.1, k1.4.0, k10.3.0).
- picking locks at night: tried (doors b5.14, b5.2, b5.18; chest k5.2.1 seen by a sleeper).
- lockpick source: the squad starts with 8 (6 + 2); two more were lying in houses (g7, g10 in Woodsands) and were stolen.
- sleeping in beds: tried (the squad beds down by itself at 22:00 inside a locked and an unlocked home; `rest` ordered for a bound member). Nobody minds.
- containers `search`, `take N`, `takeall`, `put N`, stealing: tried, all of them, many times.

Crime and the law
- witnesses: tried (sleepers by day and night; one awake witness in Newshaw).
- the watch: tried (caught more than a dozen times across both worlds).
- chases: out of town (slipped), into a building (slipped), down the street 120 m (slipped), 30 m and 60 m (caught), toward the guards (caught), standing still (caught).
- bounties: tried (12, 15, 30, 48, 130); paying: to a carpenter (refused, no coin), a merchant (paid 12), a farmer (paid 30), a guard and an official (offered).
- arrest: tried.
- judging: three customs met: duel (Stillham, Stonehaven), elders (Woodsands), the record (Newshaw).
- fines with coin: tried (paid in full; paid in part then bound). Fines with none: tried (bound 2 days). Fines with notes only: tried (bound 7 days).
- bonds: bound member's orders tried; the others leaving town tried (nothing happens to them or to him); buying out tried (works; price falls with days served); serving it out tried twice (it ends on time, a few hours past the stated days, with no message); running from it: NOT reached as a real escape. A downed bound member was carried 400 m and left standing there; the accused walked 82 m off during his duel; neither was called a runaway.
- coming back the same day: tried (3 minutes later, and 12 hours in town). Days later: tried (seed 21, day 4).

Save and load at awkward moments
- at the moment of being seen: tried (`th21` against `th21-seenB`): output identical, saves identical.
- mid-chase: tried twice (`th21-free-chase`/`chaseB`; `th34-runA`/`runB`): output identical, saves differ by a few bytes.
- mid-lock-pick: tried (`th21-pickA`/`pickB`, then 18 more hours): output identical, saves differ (66 bytes).
- long twin run through two arrests and a day (`th34-twinA`/`twinB`, 19 commands): output identical, saves differ (14 bytes).

Not reached
- a knocked-out or carried townsperson as witness (needs a won duel; every duel was lost).
- a pursuit dropped because the guard is in a fight or the culprit left the squad.
- bound while out of town after a lost duel (the duels last under a minute).
- shunning as a judgement; assault or murder charges.
- the screenshot command (`shot` fails here: "Couldn't take a screenshot."), so wall positions were worked out from the tool's own "not in a building you're in" line.

## Candidates

From `code-buildings-crime.md`
- 2 (door lock checked only when the walk starts): SEEN. `th21-free-1957`: `enter b5.18`.
- 4 (`takeall` doesn't stop; several sightings, several fines): SEEN. `th21-free-cup`: `takeall`; fines 30 against 65 in `th34-note-watch` / `th34-note-watch2`.
- 7 (`search` on a locked chest reports the wrong person / "couldn't get to it" after a good pick): SEEN, both forms (`th21-free-prechest`, `th34-prepick`).
- 8 (reach through walls; looting entry outlives walking away): SEEN for take and put through the wall at 2.2 m (`th21-wall5-base`). The entry coming back to life after leaving reach was seen once (`th21-w2.8`); by day from 20 m it did not work. Scenario B (`pickup` through a wall) not tried at the wall; from 15 m it only logged "The door is locked."
- 9 (guard and walls): A SEEN (step into any open building: slipped, bounty, no arrest). B unsure: the watch did catch the thief inside a still-locked cottage (`th21-lockedin-seen`). C not reached.
- 10 (lock-pick rolls replay on a new order): SEEN (`th21-free-prechest`: snap and sighting on each fresh order).
- 15 (walk-in at night never noticed; knocked-out witness): the walk-in and sleeping beside the residents SEEN in a locked and an unlocked home; the knocked-out or carried witness not reached.
- 3 (also tried): SEEN, known item 4. 6 (also tried): `take 0` / `take -1` SEEN; the id-letter part not tried. 14 (also tried): SEEN, listed under small things.

From `code-law-jobs-recruit.md`
- 8 (notes don't count): SEEN for a fine, a bond buy-out and a bounty (`th34-note-watch2`, `th34-bondtalk`).
- 10 (getting away is cheaper; bounty paid to anyone): SEEN (12 against 18 plus a bond; 15 against 22; 48 against 71; paid to a merchant and a farmer).
- 1 (a talk stays open while the world runs): SEEN in passing (`th34-ws-seen`: `talk p507`, then waits).
- 7 (`talk` by a bound member burns 10 minutes): SEEN (`th21-bound`: `select Numeleo`, `talk p334`).
- 9 (lost duel binds the accused wherever he is): NOT reached; only that the accused can be ordered about during the duel.
- 15 (bond state hidden; news lines dropped): SEEN.
