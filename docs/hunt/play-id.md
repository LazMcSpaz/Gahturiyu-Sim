# Play report: the IDLER (prefix `id`)

Worlds: seed 34 (Stonehaven, coastal) then seed 21. Tool: `/home/claude/hunt/g <save> <command>`.
All saves named below are in `/home/claude/hunt/saves/`.

## Findings

### A merchant stands awake in the empty square until midnight, then vanishes on the stroke of 00:00
- Part: Townsfolk's lives
- How bad: wrong outcome (a jump at midnight; the code-reading candidate "evening ran late, never walks home or goes to bed": SEEN)
- What happens: on Day 2 Weyawishis (p4017, Ṭaḍoro merchant, stall 08:30 that day) is still standing outdoors in the town square at 23:00, 23:30 and 23:59 while everyone else but the night guard is gone. At 00:00 exactly they are no longer anywhere (`talk p4017` says "Can't talk to them."). No walk home is ever seen.
- What should happen: they walk home and go to bed like everyone else; nobody should blink out at midnight.
- To see it: seed 34, copy `id34-d2-2359.save` (Day 2, 23:59). `look people`, `wait 1`, `look people`. Earlier copy `id34-d2-2300.save` shows them there from 23:00.
- Output:
```
== Day 2, 23:59 (night) · Stonehaven ==
People about (2; closest first):
  p4017  Weyawishis — Ṭaḍoro merchant — 13 m west
  p2069  Puxtod — Qotiro guard — 61 m north-east
== Day 3, 00:00 (night) · Stonehaven ==
People about (1; closest first):
  p2069  Puxtod — Qotiro guard — 61 m north-east
```
- Seen how many times: twice (main line and again from the 23:00 copy).

### The news feed names every stranger who leaves, arrives or passes, with where they are bound — even at night, out of sight, while the whole squad sleeps
- Part: Townsfolk's lives
- How bad: too much telling
- What happens: standing still in Stonehaven's square the squad gets a pushed line for every party on the road near town: the leader's name, the head count and their destination town ("bound for Windflat", "heading home to Stonehaven", "wandering, crosses your path"). About 10 to 14 a day. Lines arrive while all four are listed "asleep", and for parties that `look people` cannot yet see (night, nobody but the guard within its reach when the line is pushed; the two travellers first show up 4 minutes later, 85 m off).
- What should happen: nothing pushed for passers-by the squad has no reason to notice; at most "a party of three comes in by the east road" for what is seen awake. A stranger's name and destination are things to find out by asking.
- To see it: seed 34, `id34-d3-0131.save` (Day 3, 01:31, squad asleep in the square). `wait 138`, `look people` (only the night guard), `wait 1` (the line arrives at 03:50), `look people` (still only the guard), `wait 2`,`wait 2` (they appear at 03:54, 85 m south-east).
- Output (all of these arrived while the squad stood or slept in one spot):
```
  11:08  Yawashifas and 2 others, bound for Stonehaven.
  12:00  Hihowora and 2 others, bound for Windflat.
  19:44  Hihowora and 2 others, heading home to Stonehaven.
  21:00  Yewifaya (Ṭaḍoro), bound for Windflat.
  01:46  Saisiye (Ṭaḍoro), wandering, crosses your path.      <- squad asleep 22:00-03:58
  22:27  Yiyesh (Ṭaḍoro), wandering, crosses your path.       <- squad asleep
  03:50  Domdem and 1 other, bound for Stonehaven.            <- squad asleep, party not yet in `look people`
  09:00  Shafiha (Ṭaḍoro), bound for Marshshore.
  10:00  Sasih (Ṭaḍoro), bound for Firewood.
  10:00  Sasih and 2 others, bound for Brookgill.
```
- Seen how many times: every day of four days idling; the 03:50 case twice from the saved copy.

### "N times this season, bandits have fallen on folk on the road": an exact world-wide counter in gossip, and it climbs about 45 a day
- Part: Townsfolk's lives
- How bad: too much telling (and a count that grows without end; unsure whether the number itself is right)
- What happens: asking Gahìru (woodworker) for "Latest rumours" gives an exact count that nobody in a village could know: 40 on Day 2 06:01, 93 on Day 3 11:35, 130 on Day 4 11:42. The merchant Mexaq quotes the same exact 130 in other words the same minute. After one day of the world there have already been 40 ambushes "this season".
- What should happen: vague hearsay ("the roads are worse this year"), or a number that is the speaker's own guess; and 40 ambushes on day one sits oddly with "deaths are rare".
- To see it: seed 34, `id34-d2-0601.save` then `talk p492`, `say 4`; `id34-d3-1141.save` (just after) and `id34-d4-1141-A.save` then the same.
- Output:
```
  » 40 times this season, bandits have fallen on folk on the road. In my grandmother's day you could walk to the coast with your purse in your hand.
  » 93 times this season, bandits have fallen on folk on the road. In my grandmother's day you could walk to the coast with your purse in your hand.
  » 130 times this season, bandits have fallen on folk on the road. In my grandmother's day you could walk to the coast with your purse in your hand.
  » Bandits have fallen on travellers 130 times since the season turned. People say the roads aren't what they used to be.
```
  (Cosmetic, same line: "you could walk to the coast" is said in Stonehaven, which is on the coast.)
- Seen how many times: three days running, two speakers.

### Greetings don't follow whether you have met: a stranger says "I was hoping you'd come by, friend", a merchant you traded with says "I don't know you"
- Part: Townsfolk's lives
- How bad: misleading
- What happens: Gahìru's very first words to the squad (never spoken to before) on Day 2 06:01 were "There you are! I was hoping you'd come by. Thelo (peace) to you, oqe (friend)." Doqoḍato, talked to on Days 1 and 2, opens on Day 3 with "You're not from here, are you? Well, you're under our roof now, ḍati (guest)." and on Day 4 is back to "There you are! I was hoping you'd come by." Mexaq, who sold the squad 15 flatbread on Day 1, says on Day 4 "I don't know you. Say who you are and what you want."
- What should happen: the first-meeting line once; a warmer one only after you've actually met.
- To see it: seed 34, `id34-d2-0601.save`: `talk p492` (first ever talk with him). `id34-d4-1141-A.save`: `talk p1467`.
- Seen how many times: four days of daily talks with the same four people.

### `town` says a merchant is "at their stall from HH:MM tomorrow" when tomorrow is their day off
- Part: Townsfolk's lives
- How bad: misleading
- What happens: Stonehaven's two merchants trade on alternate days (Mexaq Days 1 and 3, Weyawishis Days 2 and 4). On the evening of a merchant's own trading day the panel promises them "tomorrow"; next day they don't trade and the panel says "tomorrow" again.
- What should happen: "the day after tomorrow", or no promise.
- To see it: seed 34. `id34-d1-1700.save`: `town` shows "p1467 Mexaq — at their stall from 06:45 tomorrow". `id34-d2-0840.save` (Day 2, 08:40): `town` shows the same line again and `look people` shows Mexaq with no "[trading at their stall]"; he trades on Day 3. Same for Weyawishis: `id34-d2-2300.save` says "from 08:30 tomorrow"; on Day 3 at 08:40 (`id34-d3-0131.save`, `wait 429`) it says "from 08:30 tomorrow" again.
- Output:
```
Day 1 17:00   p1467 Mexaq — at their stall from 06:45 tomorrow
Day 2 08:40   p1467 Mexaq — at their stall from 06:45 tomorrow
              p4017 Weyawishis — trading now
Day 2 23:00   p4017 Weyawishis — at their stall from 08:30 tomorrow
Day 3 08:40   p1467 Mexaq — trading now
              p4017 Weyawishis — at their stall from 08:30 tomorrow
```
- Seen how many times: twice (once for each merchant).

### People who have arrived stay "traveller" in `look`; `talk` gives their real trade
- Part: Townsfolk's lives
- How bad: cosmetic
- What happens: Yawetheyis arrived at 13:47 on Day 1 and is still listed "Ṭaḍoro traveller" at 17:00; `talk p3937` titles them "Ṭaḍoro labourer and porter". Domdem and Gorgex arrive at 03:56 on Day 3 and stand as "traveller" 83 m and 92 m west, on the same spots, until after 10:04.
- What should happen: one label; and visitors who arrive at 04:00 might go indoors rather than stand six hours in one spot.
- To see it: seed 34, `id34-d1-1700.save`: `look people` (p3937 traveller), `talk p3937`.
- Seen how many times: twice.

### A fight 1114 m away is announced, to the metre, to a squad standing inside a workshop in town
- Part: Townsfolk's lives
- How bad: too much telling
- What happens: while the squad stood inside the Bench house (b0.4) in Stonehaven overnight, the news said a beast had fallen on travellers "1114 m away", and in the same minute that the fight was over.
- What should happen: nothing, unless someone could see or hear it; never an exact distance.
- To see it: seed 34, `id34-d5-1201-inhome.save`: `enter b0.4`, then `wait 118`, `wait 180`, `wait 180`, `wait 180`, `wait 420`.
- Output:
```
  05:00  A Mirejaw falls on travellers 1114 m away.
  05:00  The roadside fight is over.
```
- Seen how many times: once.

### "X takes up work as alchemist in Stonehaven": strangers named for taking a post at dawn (code-reading candidate: SEEN)
- Part: Town jobs and recruiting
- How bad: too much telling
- What happens: at 06:00 on Day 7, with the squad standing on the shore 150 m west of town and nobody in sight but three fishers, two lines name townsfolk who changed job.
- What should happen: not pushed; found out by seeing a new face at the healing house or by asking.
- To see it: seed 34, `id34-d6-1159-shore.save`: `wait 180`, `wait 210`, `wait 150`, `wait 180`, `wait 177`, `wait 124`, `wait 59`, `wait 2`.
- Output:
```
== Day 7, 06:01 (day) · Stonehaven ==
News:
  06:00  Siyuth takes up work as alchemist in Stonehaven.
  06:00  Ranira takes up work as weaver and sealer in Stonehaven.
```
- Seen how many times: once (one dawn out of six had such lines).

### An `enter` ordered at 19:57 from 350 m away walks straight into a cottage that locked at 20:00 (code-reading candidate: SEEN)
- Part: Buildings and interiors
- How bad: wrong outcome
- What happens: the squad stands at the woodlot at 19:57 and is told `enter b0.5`. Five minutes later all four "are inside the Grown cottage" while `look` lists that cottage as locked. No lock picked, no lockpick used, nobody notices. Ordered two minutes later from the square instead, the same `enter` makes Nolu pick the lock in front of the evening crowd, and she is seen, caught and bound.
- What should happen: stop at the locked door.
- To see it: seed 34, `id34-c2-1957.save` (Day 4, 19:57, at the woodlot): `enter b0.5`, `look places`. For the contrast: `id34-lock-2001.save` (inside b0.5 when it locked): `go n 30`, `wait 1`, `enter b0.5`.
- Output:
```
(5 minutes pass.)
Ṭelihoḍu, Nolu, Moker, Sawasasih are inside the Grown cottage.
== Day 4, 20:02 (night) · Stonehaven ==
...
  b0.5  Grown cottage (locked: a fair lock (38): Nolu ~42% a try, 6 lockpicks (about 1 in 3 snaps on a miss)) — 5 m north-east
```
- Seen how many times: once (one try, worked first time).

### Nobody minds the squad living in their house: day, evening, asleep beside the sleepers, or walking in at 01:00 (code-reading candidate: SEEN)
- Part: Buildings and interiors
- How bad: wrong outcome (unsure whether intended), and misleading because `look` never shows the household
- What happens: (a) The squad walked into the Grown cottage b0.6 at 11:45 and stood there until the next noon. At 20:00 its door did not lock (some homes never lock), the squad `rest`ed at 20:05, slept to 23:45 beside the household, and nobody said a word. (b) At 01:00 the squad walked into the unlocked Great house b0.7, where the ground items show "26% chance of being seen" (so sleepers are within reach), lay down and slept: no notice. (c) Locked into the cottage b0.5 at 20:00 with the family home: no message that the door has locked, nobody minds, and the squad walks out freely at 20:01. In every case `look` and `look people` list no one in the house: at 01:00 "People about (1)" is the night guard 47 m away, yet something asleep within 6 m gives the 26%.
- What should happen: someone at home notices strangers on the floor (asks them to leave, or the watch comes); and `look` should say who is in the room, asleep or awake.
- To see it: seed 34. (a) `id34-d4-2005-inhome.save`: `rest`, `wait 115`, `wait 180`. (b) `id34-d5-0100-inhome.save`: `enter b0.7`, `look places`, `look people`, `rest`, `wait 240`. (c) `id34-lock-1959.save`: `wait 2`, `look places`, `go n 30`.
- Output (b):
```
Ṭelihoḍu, Nolu, Moker, Sawasasih are inside the Great house.
  g8  1 × Notes on fireball on the ground (someone's: taking it is theft, 26% chance of being seen) — right here
...
People about (1; closest first):
  p2069  Puxtod — Qotiro guard — 47 m east
Resting.
(240 minutes pass.)
== Day 5, 05:01 (night) · Stonehaven ==
```
- Seen how many times: three nights, three houses.

### In a workshop all day and night: the crafter never comes in, and the squad is locked in at 20:00 without a word
- Part: Buildings and interiors
- How bad: rough
- What happens: the squad stood inside the Bench house b0.4 from Day 5 12:02 to Day 6 09:00. Gahìru the woodworker is listed 11 to 13 m east or west of the squad (outside) at 12:02, 14:00, 17:00 and 06:00, never "right here"; from 17:00 the building is ringed by the evening crowd. The door locks at 20:00 with the squad inside, the squad beds down at 22:00, and nothing is said.
- What should happen: the crafter works at the bench by day; someone locking up notices four strangers.
- To see it: seed 34, `id34-d5-1201-inhome.save`: `enter b0.4`, then `wait 118`, `wait 180`, `wait 180`, `wait 180`, `wait 420`, `wait 180`, with `look people` each time.
- Seen how many times: once.

### Trading sells goods out of the packs of squad members who are 86 m away
- Part: Odd input (trade)
- How bad: rough (unsure: may be meant as a shared squad purse)
- What happens: one member walks to the merchant; the "Sell" lines cover everything the other three carry. Moker, alone at the stall and "[86 m from the others]", sold "all 4 × hide", which were in Nolu's pack inside the Bench house; Nolu's load dropped from 44 to 36 kg at that moment. On Day 1 Ṭelihoḍu sold Moker's three iron ingots from 66 m away the same way.
- What should happen: only what the trader (or those standing with them) carries.
- To see it: seed 34, `id34-d6-0900-shop.save`: `select Moker`, `talk p1467`, `say 12`, `look`, `pack Nolu`, `say 22`, `look`, `pack Nolu`.
- Output:
```
 *Moker      Qotiro   standing               health 100% stamina 100% load 23/70 kg hungry [86 m from the others]
  » 4 of them — 20 coin. Pleasure.
Nolu — carrying 35.8 of 69 kg        (was 43.7, with "7. 4 × Hide")
```
- Seen how many times: twice.

### Two squad members go hungry beside a squadmate carrying 12 flatbread; `give` can only hand over the whole stack
- Part: Odd input
- How bad: rough
- What happens: members eat only from their own pack. `give NAME N TO` moves the entire stack, so feeding four people from one stack means passing it round by hand and waiting for each to eat.
- What should happen: share within the squad when standing together, or let `give` take a count.
- To see it: seed 34, `id34-d6-0900-shop.save`: `look` (Moker and Sawasasih "hungry"), `pack Nolu` ("12 × Flatbread").
- Seen how many times: Days 6 and 7.

### `go w 100` at the shore answers "Nobody needs to move: they're already there."
- Part: Odd input
- How bad: misleading
- What happens: standing 46 m east of the fishing dock, an order to walk 100 m west (into the sea, which is off limits) is answered as if the squad were already at the target.
- What should happen: "They can't go into the sea."
- To see it: seed 34, `id34-d6-1159-shore.save`: `go w 100`.
- Seen how many times: twice (also `go w 3`).

### A member bound to work off a fine is not fed by the town; left alone she starves to the ground on the third day, and the days lying there still count as worked
- Part: Crime and the law
- How bad: wrong outcome (unsure: the squad can walk food over with `give Ṭelihoḍu 0 Nolu`, which works on a bound member; nothing tells the player they must)
- What happens: Nolu lost a trespass duel at Day 4 20:05 and was "bound to work in Stonehaven for 8 days". She had two dried fish. Nobody feeds her and she takes nothing from the squad: "hungry" on Day 6, "STARVING" health 67% on Day 7, "down … health 0% STARVING" at Day 8 02:01, and still down on Day 14. The buy-out price keeps falling meanwhile (56 coin on Day 5, 40 on Day 7), so lying starved counts as working.
- What should happen: the town feeds its bound workers (or the squad is told to), and a worker who is down does not work the debt off.
- To see it: seed 34, `id34-bond-d4-2009.save` (just after the duel): `wait 600`, then `wait 1440` four times with `look`. Later state: `id34-bond-d8-0201.save`.
- Output:
```
== Day 7, 09:10 (day) · Stonehaven ==
  Nolu       Horaro   standing               health  67% stamina 100% load 38/69 kg STARVING [93 m from the others]
== Day 8, 02:01 (night) · Stonehaven ==
  Nolu       Horaro   down                   health   0% stamina 100% load 38/69 kg STARVING
```
- Seen how many times: twice from the saved copy (same result).

### A bond ends without a word, and nothing shows that a member is bound or for how long
- Part: Crime and the law
- How bad: rough
- What happens: after "Nolu is bound to work in Stonehaven for 8 days" the squad list shows her as plain "standing … [85 m from the others]"; `journal` says "No jobs taken."; the days left appear nowhere. The only signs are "Nobody moves: Nolu is bound to work off a bond." when she is ordered, and the official's topic "Buy Nolu out of their bond (56 coin)", which is offered at 09:08 on Day 5 but not at 06:08 the same morning (on Day 6 at 09:09 my `talk` showed no bond line either; unsure why). When the 8 days are up (Day 12, 20:05) no news line comes; the official's topic just disappears.
- What should happen: a "bound, N days left" mark on the member and a line when they are free.
- To see it: seed 34, `id34-bond-d8-0201.save`: `wait 1440` eight times (the wait stops twice for members going down), reading News each time; then `talk p1356`.
- Seen how many times: once start to end; the silent ending twice.

### News lines with nothing after the comma: "Yafiyayuh and 1 other, ."
- Part: Townsfolk's lives
- How bad: cosmetic
- What happens: some party lines have an empty ending.
- To see it: seed 34, `id34-bond-d8-0201.save`: `wait 1440` six times; the lines come on Day 12 and Day 13.
- Output:
```
  02:14  Yafiyayuh and 1 other, .
  06:35  Yawayisa and 1 other, .
```
- Seen how many times: twice (two different parties), repeatable from the copy.

### The same posts are "taken up" again and again at dawn (alchemist four times in eight days)
- Part: Town jobs and recruiting
- How bad: wrong outcome (unsure: could be people really quitting, but it looks like a post that never stays filled)
- What happens: the dawn lines for Stonehaven, idling Days 4 to 14: Day 7 "Siyuth takes up work as alchemist", "Ranira takes up work as weaver and sealer"; Day 10 "Halelu takes up work as alchemist", "Tamax takes up work as leatherworker"; Day 11 "Ranira takes up work as weaver and sealer" (again); Day 12 "Hotorìṭo takes up work as scribe"; Day 14 "Qeḍa takes up work as alchemist", "Fewafauseh takes up work as scribe", "Nemomeno takes up work as weaver and sealer". On Day 6 `town` listed "Work going: Alchemist at the healing house — 49 coin a day" and a "No trade 1" in the trades.
- What should happen: a post once filled stays filled unless something happens to its holder.
- To see it: seed 34, `id34-bond-d8-0201.save`, `wait 1440` six times; or the main line `id34-d6-1159-shore.save` for the Day 7 lines.
- Seen how many times: nine lines over eight days.

### "A Mirejaw falls on travellers" about 1.1 km away, again and again at about 05:00
- Part: Townsfolk's lives
- How bad: too much telling (and a repeating line)
- What happens: Day 6 05:00 "1125 m away", Day 8 06:00 "1122 m away", Day 9 05:10 "1182 m away", Day 12 05:07 "1105 m away", each followed the same minute by "The roadside fight is over." The squad is asleep or standing in town each time.
- To see it: seed 34, `id34-bond-d4-2009.save` or `id34-bond-d8-0201.save` and `wait 1440` repeatedly.
- Seen how many times: four mornings.

### `wake` says "Up." for a member who is down from starvation, and `go` for her waits 30 minutes with no message
- Part: Odd input
- How bad: misleading
- To see it: seed 34, `id34-bond-d14.save` (Day 14, bond over, Nolu down): `select Nolu`, `wake` ("Up."), `go e 5` ("(30 minutes pass.)"), `look` (Nolu still "down").
- Seen how many times: once.

### A conversation stays open at any distance and for any time: sell to a merchant from the woodlot 365 m away
- Part: Odd input
- How bad: wrong outcome
- What happens: with a talk open, `go`, `enter`, `search` and `wait` all still work and the talk is never closed. The squad opened Mexaq's goods at his stall, walked to the woodlot ("t0 Stonehaven — 365 m north", Mexaq not in `look people`), and from there sold a leather for 8 coin and bought a cloth shirt for 12. After `wait 900` (now 23:05, the merchant long gone to bed) the talk is still open and still answers "Latest rumours". In a second run a talk with the cook stayed open while all four walked into the Bench house, then 100 m out of town, and "Go on." was still answered.
- What should happen: walking away ends the talk (or the order is refused while talking); no trade unless the talker stands at the stall.
- To see it: seed 34, `id34-far-woodlot.save` (Day 7 08:05, squad at the woodlot, talk with Mexaq open): `look`, `say 19` (sell leather), `say 9` (buy cloth shirt), `pack Ṭelihoḍu`. From scratch: `id34-ch-start.save`: `talk p1467`, `say 6`, `select all`, `go d0`, `wait 6`, `say 19`.
- Output:
```
  d0  Woodlot of Stonehaven — 80 timber left; ... — right here
  t0  Stonehaven (122 people) — 365 m north
    you: Sell leather (8 coin)
  » I'll give you 8 for the leather.
    you: Buy cloth shirt (12 coin)
  » 12 coin. There you are.
Squad (4 members, 16 coin between them):
```
- Seen how many times: twice (merchant; cook).

### `talk` to a second person while a talk is open: ten minutes pass and the first talk is silently swapped
- Part: Odd input
- How bad: rough
- What happens: with the cook's talk open, `talk p875` prints "(10 minutes pass.)" (no "A conversation opens.") and the screen is now the hunter's talk. An ordinary `talk` to someone at that range takes 0 to 1 minute.
- To see it: seed 34, `id34-ch-start.save`: `talk p718`, `talk p875`.
- Seen how many times: once.

### A resting ("asleep") squad can open a conversation at once
- Part: Odd input
- How bad: rough (same family as known 4, also seen for `talk`)
- To see it: seed 34, `id34-ch-start.save`: `rest`, `wait 1` (all four "asleep"), `talk p718` gives "(0 minutes pass.) A conversation opens."
- Seen how many times: once.

### `enter` for a bound or downed member says nothing and burns five minutes each time
- Part: Odd input
- How bad: rough
- What happens: `go` for a bound member answers "Nobody moves: Nolu is bound to work off a bond." but `enter b0.5` for the same member prints only "(5 minutes pass.)". Ten in a row cost 50 minutes with no word. For a member who is down, `go e 5` costs 30 minutes the same way.
- To see it: seed 34, `id34-bond-d4-2009.save`: `select Nolu`, `go n 5`, then `enter b0.5` several times.
- Seen how many times: ten times in a row in one run.

### `wait` with a bad number waits anyway
- Part: Odd input
- How bad: rough
- What happens: `wait abc` and bare `wait` let 10 minutes pass; `wait -5` and `wait 0` let 0 pass; `wait 1.5` lets 2 pass; `wait 99999999999999999999` ran until someone collapsed from hunger 3546 minutes later. None prints a usage line.
- To see it: seed 34, `id34-ch-start.save`: each command above.
- Seen how many times: once each.

### With the squad split up, `look` describes the empty spot in the middle of them
- Part: Odd input
- How bad: rough
- What happens: with one member at the dock, one at the iron seam, one in a cottage and one in the hall, "Near you" and "People about (61)" list what is round the point between them (a cottage "6 m north" that no member is near).
- To see it: seed 34, `id34-ch-start.save`: `go d0`, `search k0.1.2`, `select Nolu`, `enter b0.5`, `select Sawasasih`, `go d1`, `select Ṭelihoḍu`, `go w0.8`, `select all`, `wait 60`.
- Seen how many times: once.

### Small ones seen in passing
- cosmetic: "People about (N; closest first)" is not closest first: the "[restless: might join]" people are always listed on top (51 m and 55 m ahead of 13 m).
- cosmetic: `select Moker Moker Moker` answers "Selected: Moker, Moker, Moker".
- cosmetic: members can be "asleep [sneaking: half pace]" holding lit torches (`sneak`, `torch`, `rest`, `wait 5`).
- cosmetic: names collide. Two people called Ditut (p1356 official, p1689 labourer), two farmers called Rodo (p837, p269), two Heyefuya, two Hulilegu, two Punmor; and a passer-by with a squad member's name: "19:16  Sawasasih (Ṭaḍoro), wandering, crosses your path."
- cosmetic: each language has its own name for the town ("Doqoteḍo (Stonehaven)" from a Roduro, "Sewethehe (Stonehaven)" from a Ṭaḍoro). Possibly meant.
- too much telling (already listed as code-reading candidate 14, all SEEN): exact lock numbers and odds for every door within about 45 m ("locked: a fair lock (38): Nolu ~42% a try, 6 lockpicks (about 1 in 3 snaps on a miss)"); owned things lying inside other people's houses listed through walls with a seen-% ("g0  1 × Knife on the ground (someone's: taking it is theft, 0% chance of being seen) — 54 m north-west"); "Ḍuʻathuhu runs to fetch the watch!" names the witness; `town` lists a recruit as "asleep indoors" and gives each recruit's best skills and gear before they are met.
- too much telling (unsure): "There's a band of 4 camped about 3.2 km east of here" from a farmer on Day 1: exact head count and distance.
- rough: a `wait 1440` shows only the last 14 or so news lines; the earlier part of the day is gone and nothing says so.
- rough: nothing is said when a door locks at 20:00 with the squad inside (cottage, workshop, hall).
- also seen (known 5): "[N m from the others]" on everyone when one member walks off. Also seen (known 8): "(night)" from 18:00.
- tool note for whoever re-runs: a `.save` copied without its `.session` file replays the last 14 old news lines and the first-day TIPs on the next command. Copy both files when comparing output line by line.

### All gossip in ten days is one sentence about another town: "X squeezed money out of Y"; "I'll tell the watch" does nothing you can see
- Part: Townsfolk's lives
- How bad: rough (with one misleading line)
- What happens: the same people were asked every day. Days 1 to 4: no gossip at all; the farmer and the cook give the bandit-camp line word for word every day, the woodworker and the merchant the ambush counter. From Day 7 gossip appears in the greeting and under "Latest rumours", and every item, from five tellers, is the same crime in some other town: "Word came by the road from Stillshaw: Pontriq squeezed money out of Yeiwaufe, yesterday. That's an hour's walk from here, to the south-east." (also Rainwood: Qotxem/Yaḍoʻe; Marshshore: Ritmag/Weyusesa; Brookgill: Shafuwu/Shoga; Stillshaw: Pontriq/Heyesiyi). Nothing about Stonehaven itself in ten days (not the squad sleeping in houses, not the new alchemists). The age counts up properly ("yesterday", "two days ago", … "a while back"). The other wording of the same item is "Have you heard? Ritmag squeezed money out of Weyusesa, six days ago. The whole street's talking about it.", said of a six-day-old matter in a town a few hours away. Each item adds a topic "I'll tell the watch."; choosing it gets "Good. Let them answer for it." and then no news line, no journal entry, nothing.
- What should happen: some local talk; more than one kind of story; "tell the watch" either leads somewhere or isn't offered for another town's business.
- To see it: seed 34, `id34-d7-0820.save`, `id34-d8-0820.save`, `id34-d9-0820.save`, `id34-d10-0819.save`: `select Nolu`, `talk p563` / `talk p718` / `talk p3907`, pick "Latest rumours"; on `id34-d7-0820.save` `talk p718`, `say 1`.
- Seen how many times: four days running, five tellers.

### Talking again to someone you have just left costs ten minutes each time
- Part: Odd input
- How bad: rough
- What happens: `talk p1467`, `bye`, `talk p1467`: the second and every later `talk` prints "(10 minutes pass.)" with no "A conversation opens." (the talk is open all the same). The first one took 1 minute.
- To see it: seed 34, `id34-d9-0820.save`: `select Nolu`, `talk p1467`, `bye`, `talk p1467`, `bye`, `talk p1467`, with `look` between (08:38, 08:48, 08:58 in my run).
- Seen how many times: three times in a row.

### After the squad wakes, "23:45 Ṭelihoḍu wakes, rested." is printed again after every command
- Part: Save and load
- How bad: cosmetic (unsure whether it is only the text tool's "news seen so far" mark)
- What happens: once the four wake lines have been shown, each later `wait 1` shows the last of them again as fresh News until some newer line arrives (20 repeats in 20 minutes). The world itself is the same.
- To see it: seed 34, `id34-rested-2005.save` (squad told to `rest` inside the cottage at 20:05): `wait 219`, then `wait 1` six times.
- Output:
```
== Day 4, 23:46 (night) · Stonehaven ==
News:
  23:45  Ṭelihoḍu wakes, rested.
== Day 4, 23:47 (night) · Stonehaven ==
News:
  23:45  Ṭelihoḍu wakes, rested.
```
- Seen how many times: twice from the saved copy.

## Seed 21 (Stillham, coastal, 344 people; Fieldhearth, inland, 330 people)

### Craft posts are a revolving door: every dawn new people "take up" smith, scribe, weaver…, and the pile of "No trade" people grows by two or three a day
- Part: Town jobs and recruiting
- How bad: wrong outcome (a count that grows without end)
- What happens: idling in Stillham for 14 days. `town` on Day 1: "Armourer 2 … Labourer and porter 143 … Scribe 4, Smith 3 …", no "No trade". Day 5: "No trade 5", Smith 1, no Armourer at all. Day 8: "No trade 15", Labourer 132. Then, with the dawn lines: Day 9 "No trade 20" (Trergux and Fiha take up smith, Yiyahi armourer, Ṭuʻaṭuḍu scribe); Day 10 "No trade 21" (Pikix smith, …); Day 11 "No trade 22", Smith back to 1; Day 12 "No trade 24" (Qeḍugeʻu and Yisausai smith); Day 13 "No trade 26" (Shohiqa smith; "Luwelao takes up work as weaver and sealer", who already took it on Day 10); Day 14 "No trade 29", Labourer 122, Scribe 2. The people who held a post yesterday show up as "(no trade), for nothing" under "Willing to join", which itself grows 14 → 20 → 26 → 30. Redakan, a smith asking 72 coin on Day 1, is "Redakan (no trade), for nothing" on Day 8. The same vacancy ("Smith at the smithy — 49 coin a day") is advertised on Days 8, 9, 11 and 12 in between. Same pattern in Stonehaven (seed 34: alchemist taken on Days 7, 10 and 14; the first alchemist Huḍaliʻi is "no trade" from Day 6) and Fieldhearth (smith taken Day 6 by Ḍoʻidodì, Day 7 by Roxnor; scribe Day 6 once, Day 7 twice).
- What should happen: a post once taken stays taken; people who leave a post go back to being labourers (or leave town), not into an ever-growing "No trade".
- To see it: seed 21, `id21-d8-0607-shore.save`: `town`, then six times `wait 1440` and `town`. Earlier copies for the start of it: `id21-start.save`, `id21-d4-2000-inn.save`, `id21-d5-0600-inhome.save` (`town` on each).
- Output:
```
Day 1   Trades: … Armourer 2 … Labourer and porter 143 … Scribe 4, Smith 3 …            Willing to join (14)
Day 5   Trades: … Labourer and porter 143 … No trade 5 … Scribe 4, Smith 1 …             Willing to join (23)
Day 8   Trades: … Armourer 1 … Labourer and porter 132 … No trade 15 … Scribe 3, Smith 1  Willing to join (26)
Day 14  Trades: … Armourer 1 … Labourer and porter 122 … No trade 29 … Scribe 2, Smith 3  Willing to join (30)
== Day 13, 06:07 (day) · Stillham ==
  06:00  Shohiqa takes up work as smith in Stillham.
  06:00  Laʻedaqo takes up work as mason in Stillham.
  06:00  Luwelao takes up work as weaver and sealer in Stillham.
```
- Seen how many times: every dawn from Day 5 to Day 14 in Stillham; also Stonehaven and Fieldhearth.

### In the wilds, 2.2 km from one town and 0.8 km from another, dawn brings five named job changes from both (code-reading candidate: SEEN)
- Part: Town jobs and recruiting
- How bad: too much telling
- What happens: the squad stands in open country ("· the wilds", "People about (0)"). At 06:00 it is told who took which post in Stillham (2.2 km west) and Fieldhearth (799 m south-east). Through the same day it also gets about sixty lines naming every party on the roads and five "Bandits fall on travellers 1473 m away." lines.
- To see it: seed 21, `id21-wilds-d7-0722.save`: `map`, `wait 700`, `wait 659`.
- Output:
```
== Day 8, 06:01 (day) · the wilds ==
  06:00  Hauyai takes up work as leatherworker in Stillham.
  06:00  Henenuwa takes up work as weaver and sealer in Stillham.
  06:00  Yisausai takes up work as armourer in Fieldhearth.
  06:00  Mextron takes up work as weaver and sealer in Fieldhearth.
  06:00  Shafiha takes up work as weaver and sealer in Fieldhearth.
```
- Seen how many times: twice from the saved copy (identical).

### "Bandits fall on travellers 1729 m away": every ambush within about 1.8 km is announced with its distance, five a day near Fieldhearth
- Part: Townsfolk's lives
- How bad: too much telling (and the same event over and over)
- What happens: standing in Fieldhearth's square, or asleep there: Day 5 16:38 "1729 m away", Day 6 01:03 "1649 m away", 05:32 "1759 m away", Day 7 03:47 "1644 m away"; from the wilds on Day 7: 07:54, 12:11, 16:17, 20:32 and 02:14. Each is followed within a minute by "The roadside fight is over." Nobody in town mentions any of it; the only echo is the ambush counter in gossip (47 on Day 2, 170 on Day 5).
- To see it: seed 21, `id21-d5-0650-field.save`: `wait 600`, `wait 780`.
- Seen how many times: nine lines in three days.

### `talk` to someone who is not in town sends a member walking off across country after them
- Part: Odd input
- How bad: rough
- What happens: `talk p293` (the innkeeper, who was away that day) prints "(10 minutes pass.) They're not talking (or you couldn't reach them)." and leaves Ḍuhuquʻi "walking … [726 m from the others]". He stops about a kilometre out and stands there; at 22:00 the other three bed down and he does not.
- What should happen: "They aren't here."
- To see it: seed 21, `id21-d4-2000-inn.save`: `talk p293`, `look`, `wait 30`, `wait 120`.
- Seen how many times: once (also `talk p817` on Day 2: 10 minutes, same message).

### Visitors stand in the street all night as "traveller" in a town that has an inn
- Part: Townsfolk's lives
- How bad: rough
- What happens: at midnight in Stillham the only people about are seven or eight "traveller"s standing 64 to 148 m off, on the same spots at 23:30, 23:59, 00:00 and 00:01; in Stonehaven two arrivals stood on one spot from 03:56 to past 10:04.
- To see it: seed 21, `id21-d2-2300.save`: `wait 30`, `look people`, `wait 29`, `look people`.
- Seen how many times: three nights.

### The Merchants list drops a merchant who is away, after promising them for "tomorrow"
- Part: Townsfolk's lives
- How bad: misleading (second form of the "tomorrow" finding)
- What happens: Stillham, Day 1, 06:00 to 14:00: "p817 Goṭu — at their stall from 07:00 tomorrow". From 17:00 Goṭu is not in the list at all (two merchants shown, "Merchant 3" in the trades), nor at 08:00 on Day 2; `talk p817` cannot reach them. They are back in the list at 12:00 on Day 2 ("from 12:30"). In Fieldhearth "p4266 Yihewasa — at their stall from 09:00" on Day 5 is gone from the list on Day 6.
- To see it: seed 21, `id21-d1-0837.save`: `town`; `id21-d2-0800.save`: `town`, `talk p817`.
- Seen how many times: twice.

### New in town, greeted as an old friend: "It's good to see a face I can trust."
- Part: Townsfolk's lives
- How bad: misleading (same as the seed 34 greeting finding)
- What happens: five minutes after first walking into Fieldhearth: the smith: "There you are! I was hoping you'd come by. Thelo (peace) to you, oqe (friend)."; the cook: "Ton (sun) on your road. It's good to see a face I can trust."; the farmer and the guard, the same minute: "You're not from here, are you?"
- To see it: seed 21, `id21-d5-0645-field.save`: `select Ḍuhuquʻi`, `talk p407`, `bye`, `talk p1798`.
- Seen how many times: once each.

### `equip NAME N` on food puts a flatbread in the main hand; `dose X X` at full health uses up the draught
- Part: Odd input
- How bad: rough
- What happens: `equip Ḍuhuquʻi 0` (entry 0 is "12 × Flatbread") answers "Done."; the pack then shows "MainHand   Flatbread  (worth ~3)" and the club has gone to the pack. `dose Ruqox Ruqox` at 100% health answers "Ruqox gives Ruqox a healing draught." and his only draught is gone.
- To see it: seed 21, `id21-d2-0800.save`: `equip Ḍuhuquʻi 0`, `pack Ḍuhuquʻi`; `dose Ruqox Ruqox`, `pack Ruqox`.
- Seen how many times: once each.

## What the six followed people did (no finding unless said)

Seed 34, Stonehaven, squad standing in the square, `look people` every 2 to 3 hours, Day 1 06:00 to Day 3 01:30:
- Merchant Mexaq (p1467): at the stall ("[trading at their stall]", 67 to 70 m west) 07:20 to 14:00 on Day 1; at 17:00 near the stall without the tag and `talk` offers no goods; 20:00 and 22:00 at 49 m south-west (home side); gone by 23:59. Day 2 (not his trading day) he stands 49 to 52 m south-west in the morning and 65 to 70 m west from 12:00, never trading.
- Merchant Weyawishis (p4017): trading Day 2 from 08:30; that night stands in the square till midnight (the finding above).
- Farmer Doqoḍato (p563): in the square at 06:00; gone to the fields by 07:20; at 12:00 on Day 1 listed as "traveller" 16 m north and left with "Hihowora and 2 others, bound for Windflat", back at 19:44, at the evening meal at 20:00; asleep (not listed) by 22:00; in the square again 05:59. Four hours earlier she had said "I've a living here, and I'm not leaving it."
- Guards Litihegi (p1045) and Yafesuthih (p4809): on one post 60 to 67 m north-east from 05:59 to 17:00 both days; at the evening meal (10 to 16 m) at 20:00; not listed 22:00 to 05:00. Guard Puxtod (p2069) holds the same post 20:00 to 06:00 every night and is not seen by day. At 05:59 all three stand there together. Nobody on watch for 24 hours.
- Official Ditut (p1356): 52 to 57 m east 07:20 to 14:00 (work), 52 to 57 m west at 06:00, 17:00 and 20:00 (home side); the bond and work topics are offered at 09:08 but not at 06:08.
- Woodworker Gahìru (p492): 10 to 12 m east at 06:00, 68 m south-west 07:20 to 12:00, back by 17:00, at the meal 20:00, not listed at night.
- Drifter Thehewayis (p3866, no job): wanders 7 to 34 m round the square all day, at the meal at 20:00, still out at 22:00, gone by 23:59.
- Arbiter Siyi (p3907) and cook Ḍuʻathuhu (p718): steady work spots by day (39 to 43 m south; 37 to 44 m north), other spots from 17:00, not listed at night; the cook is out again at 05:00.
- At 00:00 and at 06:00 nothing jumped except Weyawishis at midnight on Day 2. No news line arrived at 06:00 on Days 2 to 6.

Seed 21, Stillham, same method, Day 1 08:36 to Day 2 23:00: merchants Siyashifah and Sausai trade when `town` says so (a 12:00 to 12:30 break shows in both `town` and `look`); teacher 41 to 44 m east 08:00 to 14:00; exchanger 40 to 49 m east by day and in the square in the evening; officials 59 to 70 m south by day; farmer Ṭaqi away by day, 4 to 5 m off at 17:00 and 20:00; five guards at the evening gathering 20:00 to 22:00, none within reach at midnight; innkeeper at the inn spot 12:00 to 23:00 on Days 1 to 3, walking home at 23:30, away from town all of Day 4.

## Merchants' hours (does `town` agree with who will trade?)

- Stonehaven (seed 34): 05:00 nobody about; 07:20 and 10:00 to 14:00 Mexaq trades and `town` says "trading now"; 17:00 `talk` gives no goods and `town` says "from 06:45 tomorrow"; 20:00, 23:00 no trade. They agree hour by hour; the only fault is the "tomorrow" promise (finding above).
- Stillham (seed 21): 05:00 "Can't talk to them."; 08:00 both "trading now" and both sell; 12:00 Sausai sells, Siyashifah "from 12:30"; 17:00 and 20:00 `talk` opens but no goods; 22:56 "Can't talk to them.". `town` and `look` agreed every time.
- Both merchants in a town offer the same goods at nearly the same price (Stonehaven Day 6: both list seareed padded jacket 43, short bow 97/96, fine wood bow 194/191 …), and Day 1's flatbread was never on sale again in Stonehaven (Days 6 to 10 checked), so a squad cannot buy food there after the first day. Unsure whether meant.

## The shore, the boats and the tide

- Stonehaven (Days 6/7 and 10/11) and Stillham (Days 3 and 7/8), standing 46 m east of the fishing dock (as far west as the squad can go).
- Boats: yes. A line of Horaro fishers stands 50 to 63 m west from about 05:50, is 6 to 8 m further out at 06:01, and is gone by 06:07 (Stillham: about twenty of them; Stonehaven: three). Two to four fishers of other peoples work the dock itself (44 to 49 m west) from about 06:10 to mid afternoon; nobody is on the shore from dusk to 05:50.
- Tide: nothing in `look`, `town`, `map` or anyone's talk mentions the tide or shows the stilt village (no building ids offshore, no place line). The fishers' spots at the same clock time were the same to within 2 m four or five days apart, so I could not see the tide change anything, and saw nothing that contradicts itself. The one day-to-day difference: at Stillham about fifteen labourers stood on the shore 33 to 70 m east at 06:07 on Day 3 and only three on Day 8.
- `shot` answered "Couldn't take a screenshot.", so the window's view of the tide was not reachable.

## Coverage (my assignment's bullets)

- Idle in a town square three or four days with long waits, reading every news line: **tried.** Seed 34 Days 1 to 4 in Stonehaven's square (`wait 600`, `wait 1440`, and 2 to 3 hour steps); seed 21 Days 5 to 7 in Fieldhearth's square and Days 8 to 14 at Stillham with six `wait 1440`. Findings: the passer-by feed, far fights with distances, dawn job lines, the ambush counter, the growing "No trade".
- Idle inside someone's home overnight and over a day: **tried.** Seed 34 cottage b0.6 Day 4 11:45 to Day 5 12:01 (unlocked all night), cottage b0.5 across the 20:00 lock, Great house b0.7 entered at 01:00; seed 21 longhouse b5.15 entered at 20:00. Slept on the floor each time. Nobody minds; no line when the door locks; walking out of a locked house is free; walking back in needs the lock picked (and was seen at once); `look` never shows the household. Sleeping in a bed: **not reached** (the tool has no bed command; `rest` only).
- Idle inside a workplace or shop through a working day and a night: **tried.** Seed 34 Bench house b0.4, Day 5 12:02 to Day 6 09:00; seed 21 the inn's spot, Day 3 18:32 to Day 4 20:00 (the inn, food shop and market have no building id in the text tool).
- Six named people with different jobs, every two or three hours for two days: **tried**, both seeds (summary above).
- Merchants' hours at 05:00, 08:00, 12:00, 17:00, 20:00, 23:00: **tried**, both seeds (summary above).
- Gossip, grudges and rumours, same people every day for four days: **tried.** Seed 34 Days 1 to 4 and again Days 7 to 10 (farmer, woodworker, cook, merchant, arbiter); seed 21 Days 2, 5, 6, 7. Grudges: none ever came up.
- A coastal town's stilt village and the tides: **tried** as far as the text allows (section above).
- Everything at once: **tried.** `rest` then `go` (fine); `enter` with a talk open (walks in, talk stays open); `search` while walking away (the searcher walks back 370 m and opens it); `talk` to a second person with a talk open (10 minutes, silent swap); `sneak` + `torch` + `rest` (asleep, sneaking, torches lit); one member indoors and others far, then `wait` (fine, but `look` centres on nobody); `wait 0`, `-5`, `abc`, none, `1.5`, `100000`, a 20-digit number; `select` nobody/unknown/the same name three times; `enter` ten times; about forty bad ids and missing arguments (no crash; all answered, list in the odd-input findings).
- Save and load at awkward moments: **tried.** Copies at 23:59 (both seeds), 05:59 (both seeds), asleep indoors (seed 34 20:05), door locked on the squad (seed 34 20:01, which goes on into the lock-pick, the duel and the bond), Fieldhearth with a talk, the wilds. Each copy was continued twice with the same 5 to 14 commands and the full output compared: **identical every time** (eight pairs, 114 to 549 lines each).
- Stepping, `wait 60` once against `wait 1` sixty times: **tried.** Seed 34 from 17:00 Day 1; across dawn from 05:59; across midnight (120 against 120 × 1); across the 20:00 lock; asleep indoors (240 against 240 × 1); 1440 against 24 × 60; 1140 against 19 × 60 over a dawn with job changes; 600 against 20 × 30 with a bound member. Seed 21: 120 × 1 across dawn; 1360 against 34 × 40 in the wilds. `look`, `look people`, `town`, packs and `journal` were **the same every time**. The only differences were in what the tool prints as News (the 14-line cut, and the repeated last line listed above).

## Candidates (from the code-reading reports)

- News lines about every town's change of custom ("…goes over to…", "…now cooks…", "…lives by … now"): **not seen** in 14 days of seed 34 (Stonehaven) or 14 days of seed 21 (Stillham, Fieldhearth, the wilds). I grepped every long wait for them.
- Strangers named for taking a post at dawn in towns up to 2.5 km away: **seen** (shore of Stonehaven; the wilds 2.2 km from Stillham and 0.8 km from Fieldhearth).
- The `town` panel giving exact treasury, unrest and stock numbers and a "Lately" list: **couldn't reach.** The text tool's `town` prints trades, out-of-town places, vacancies, recruits and merchants only. What it does over-tell: every recruit's price, best skills, gear and "asleep indoors", for a town just walked into.
- People who never walk home or go to bed because their evening ran late: **seen** once (Weyawishis, Stonehaven, Day 2: in the square to 23:59, gone at 00:00). Not seen on Days 4 or 6 there, nor for Stillham's innkeeper (who walks home at 23:30).
- Walking into an unlocked home at night and sleeping beside the household with no notice: **seen** (four houses, both seeds).
- code-society-base 14, base events in the journal: **couldn't reach** (no base). 15, treasuries without bound: **couldn't reach** (no treasury shown).
- code-society-base "smaller things", stale household numbers / wiped stations after a custom change: **couldn't reach** (no custom change happened).
- code-buildings-crime 2, a door's lock checked only when the route is written: **seen** (`id34-c2-1957.save`).
- code-buildings-crime 14, too much telling in crime, lock and container text: **seen** for lock numbers and odds, owned items listed through walls with a seen-%, and the named witness "runs to fetch the watch". "saw it, but says nothing" **not seen** (I stole only once, by day, at 0%).
- code-buildings-crime 15, a knocked-out witness: **not tried.** Its "related gap" (no trespass for walking into a home): **seen.**
- code-buildings-crime 12, non-move orders skip the bond check: **seen** for `enter` (silent, 5 minutes each).
- code-law-jobs-recruit 9 (duel champion is the strongest member within 30 m, town champion appears beside the accused): **seen in passing**: "Nolu must answer for trespass by duel" and the fight was Sawasasih against Yafesuthih.
- Known list: 3 (0% by day indoors), 4 (asleep members act; also `talk`), 5, 8 **also seen.** 2 (starved squad) also seen, in the bond branch and after `wait 100000`.

## Saved copies worth keeping (all in `/home/claude/hunt/saves/`, each with its `.session` unless noted)

Seed 34: `id34-start` (no session), `id34-d1-1700`, `id34-d2-0559`, `id34-d2-0601`, `id34-d2-0840`, `id34-d2-2300`, `id34-d2-2359`, `id34-d3-0131`, `id34-d3-1141`, `id34-d4-1141-A` (these early ones have no session file), `id34-d4-1147`, `id34-d4-2005-inhome`, `id34-rested-2005`, `id34-d5-0100-inhome`, `id34-d5-1201-inhome`, `id34-d6-0900-shop`, `id34-d6-1159-shore`, `id34-d7-0749-shore`, `id34-d7-0820` … `id34-d10-0819`, `id34-d10-2145-shore`, `id34-lock-1931`, `id34-lock-1959`, `id34-lock-2001`, `id34-c2-1957`, `id34-bond-d4-2009`, `id34-bond-d8-0201`, `id34-bond-d14`, `id34-ch-start`, `id34-far-woodlot`.
Seed 21: `id21-start`, `id21-d1-0837`, `id21-d1-2359`, `id21-d2-0559`, `id21-d2-0800`, `id21-d2-2300`, `id21-d3-1830-shore`, `id21-d4-2000-inn`, `id21-d5-0600-inhome`, `id21-d5-0645-field`, `id21-d5-0650-field`, `id21-d6-0548-field`, `id21-d7-0700-field`, `id21-wilds-d7-0722`, `id21-d8-0607-shore`, `id21-d14-0607-shore`.
