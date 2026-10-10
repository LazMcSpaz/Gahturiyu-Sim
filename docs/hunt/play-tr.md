# Play report: the careful trader (prefix `tr`)

Worlds played: seed 21 (`tr21*`), then seed 3 (`tr3*`). All saves are in `/home/claude/hunt/saves/`.
Commands are given as `g SAVE command` meaning `/home/claude/hunt/g SAVE command`.
To re-run a finding without spoiling the saved copy: `cp saves/X.save saves/tr-check.save` and run on `tr-check`.

## Findings

### A talk left open goes on working from any distance: goods sold and bread bought 1.5 km from the merchant
- Part: Town jobs and recruiting (also Odd input)
- How bad: wrong outcome
- What happens: with a merchant's trade topics open, the squad walks 1.5 km out of town (18 minutes). `say N` still sells kelp (from another member's pack) and buys bread; coin and goods change hands with nobody there. The HUD keeps printing "TALKING with …" in the wilds.
- What should happen: the talk ends when the two part (or the tool refuses to move with a talk open).
- To see it: seed 21, copy `tr21-tradeopen` (talk with merchant Siyashifah p3765 open on the trade list, Day 1 07:01). `go e 1500`, `say 8` (sell kelp), `say 6` (buy flatbread), `pack Ḍuhuquʻi`. Result kept as `tr21-tradeopen-after`.
- Output:
  ```
  (18 minutes pass.)
  == Day 1, 07:36 (day) · the wilds ==
  TALKING with Siyashifah (Ṭaḍoro merchant):
      you: Sell all 4 × kelp frond — 4 coin
    » 4 of them — 4 coin. Pleasure.
  ...
      you: Buy flatbread (2 coin)
    » 2 coin. There you are.
   Pack:
      0. 4 × Flatbread [food, 30]  (worth ~3 each)
  ```
- Seen how many times: twice (also with the official p334 from copy `tr21-talkopen`: `go n 600`, `wait 30`, `say 5` still answers 600 m away, 36 minutes later).

### A talk left open recruits from 2 km away two hours later; the recruit appears beside the squad
- Part: Town jobs and recruiting
- How bad: wrong outcome
- What happens: talk to a restless local, don't say goodbye, walk 2 km into the wilds and wait two hours. `say 6` ("Come with us") still signs her on, takes the 28 coin and she is at once standing with the squad (no "[N m from the others]" flag), 2 km from where she was.
- What should happen: the talk closes when you part; no recruiting at a distance.
- To see it: seed 21, copy `tr21-recruit-open` (talk with Sathufiya p4587 open, Day 1 10:00, 146 coin). `go e 2000`, `wait 120`, `say 6`, `look`. Result kept as `tr21-recruit-afar-after`.
- Output:
  ```
  (21 minutes pass.)
  == Day 1, 10:21 (day) · the wilds ==
  (120 minutes pass.)
      you: Come with us — join the squad (28 coin to sign on)
    » 28 coin to my household, and I'm yours. Where are we going?
  == Day 1, 12:21 (day) · the wilds ==
  Squad (5 members, 118 coin between them):
    ...
    Sathufiya  Ṭaḍoro   standing               health 100% stamina  93% load 13/49 kg
  News:
    12:21  Sathufiya of Stillham joins the squad.
  TALKING with Sathufiya (Ṭaḍoro no trade):
  ```
- Seen how many times: once (same mechanism as the finding above, which was seen twice). Also cosmetic: once she has joined, the talk header reads "(Ṭaḍoro no trade)".

### "Sell all" sells what squad members hundreds of metres away are carrying
- Part: Town jobs and recruiting (trade used as a means)
- How bad: wrong outcome (unsure: may be a deliberate shortcut)
- What happens: one member walks 450 m from the woodlot to a merchant while the other three stand at the woodlot with full packs. "Sell all 71 × timber" sells all four packs' timber; the three absent members' loads drop at once.
- What should happen: only what the talker (or members standing with them) carries is sold.
- To see it: seed 21, copy `tr21-sell-absent` (Day 1 09:49, Ḍuhuquʻi talking to merchant Sausai p4231, trade list open, the other three 457 m away). `say 13`, `look`.
- Output:
  ```
      you: Sell all 71 × timber — 80 coin
    » 71 of them — 80 coin. Pleasure.
  Squad (4 members, 80 coin between them):
    Ḍuhuquʻi   Roduro   standing    ... load 18/88 kg [457 m from the others]
    Numeleo    Horaro   standing    ... load 40/72 kg [150 m from the others]
    Ruqox      Qotiro   standing    ... load 26/62 kg [154 m from the others]
    Weyusauthi Ṭaḍoro   standing    ... load 15/61 kg [153 m from the others]
  ```
  (before the sale the loads were 85/88, 70/72, 60/62, 60/61)
- Seen how many times: once.

### Notes are counted in "N coin between them" but nothing can be paid with them
- Part: Crime and the law (money) / Town jobs and recruiting
- How bad: misleading
- What happens: change 102 coin for two notes at the exchanger. The HUD says "116 coin between them" (then 104), but a 28-coin recruit answers "you can't pay what they ask" and a 12-coin hide can't be bought. Only the 16 loose coin count.
- What should happen: notes pay (with change), or the HUD shows coin and notes apart.
- To see it: seed 21, copy `tr21-notes-16coin-2notes` (Day 1 10:01, 16 coin + 2 notes). `talk p4828`, `say 6`, `bye`, `talk p4231`, `say 6`, `say 8`, `say 8`, `look`.
- Output:
  ```
  Squad (5 members, 116 coin between them):
      you: Come with us — join the squad (28 coin to sign on)
    » No — you can't pay what they ask.
      you: Buy hide (12 coin)
    » 12 coin. There you are.
      you: Buy hide (12 coin)
    » You haven't the coin for that — or I've none left.
  Squad (5 members, 104 coin between them):
  ```
- Seen how many times: once each (recruit fee, shop). Fines and bounties not tried (lawful play).
- Also (rough): the exchanger always lists "Change a note for 49 coin" with no note held ("You've no note to change."), and the refusal "No — you can't pay what they ask." is printed as the local's own speech.

### A member sent to work from the next town walks on the spot all day and never arrives
- Part: Town jobs and recruiting
- How bad: wrong outcome
- What happens: Ḍuhuquʻi holds a post in Stillham; the squad sleeps in Stillbank (2.5 km north). At 08:00 he sets off alone with no News line. He stops about 930 m out and stays "walking to work" at that same distance until 17:00, stamina draining 100% → 49%, then sleeps alone in the wilds. Nothing is said, and nothing is said next dawn either (0 days paid, no missed-day line).
- What should happen: he reaches the post (or doesn't set off from another town), and the player is told.
- To see it: seed 21, copy `tr21-far-d2-0808` (Day 2 08:08, squad in Stillbank). `wait 240`, `wait 300`, `wait 780`, reading his line each time. (To get there from `tr21-d1-evening`: `go t4`, `go n 500`, `go ne 500`, `go t4`, `wait 600`, `wait 95`.)
- Output:
  ```
  == Day 2, 08:08 (day) · Stillbank ==
    Ḍuhuquʻi   Roduro   walking to work        health 100% stamina  99% load 20/88 kg [789 m from the others]
  == Day 2, 12:08 (day) · Stillbank ==
    Ḍuhuquʻi   Roduro   walking to work        health 100% stamina  60% load 20/88 kg [928 m from the others]
  == Day 2, 17:08 (day) · Stillbank ==
    Ḍuhuquʻi   Roduro   standing               health 100% stamina  49% load 20/88 kg [928 m from the others]
  == Day 3, 06:08 (day) · Stillbank ==
    Ḍuhuquʻi   Roduro   asleep                 health 100% stamina 100% load 20/88 kg [928 m from the others]
  Town work: Ḍuhuquʻi works as labourer and porter ... (0 days paid; shift today from 08:00).
  ```
- Seen how many times: twice from the same copy (once ordering him back at 09:08, when he was also at 928 m; once left alone all day).
- Related (rough): the whole squad's `go t4` from Stillham (`tr21-d1-evening`) stops after 10 minutes at "Stillham 918 m south" with everyone "standing" and no reason given; a second `go t4` prints "(0 minutes pass.)". Only stepping round (`go n 500`, `go ne 500`) gets the squad past. Presumably the straight line meets the sea.

### A worker ordered away in shift hours is shown "walking to work" while walking away from it
- Part: Town jobs and recruiting
- How bad: misleading
- What happens: during his shift Ḍuhuquʻi is ordered 9 km east. From the first step his status reads "walking to work" while the distance from the others grows from 3078 m to 8458 m; only then does he turn back. No line says he left his post or went back.
- What should happen: "walking" while following the order; a line when he heads back by himself.
- To see it: seed 21, copy `tr21-midshift` (Day 2 12:30). `select Ḍuhuquʻi`, `go e 9000`, `wait 1`, `wait 10`, `wait 20`, `wait 30`, `wait 60`, `wait 60`, `wait 60`.
- Output:
  ```
  == Day 2, 13:00 (day) · Stillham ==
   *Ḍuhuquʻi   Roduro   walking to work        health 100% stamina  84% load 20/88 kg [3078 m from the others]
  == Day 2, 13:31 (day) · Stillham ==
   *Ḍuhuquʻi   Roduro   walking to work        health 100% stamina  71% load 20/88 kg [6529 m from the others]
  == Day 2, 14:01 (day) · Stillham ==
   *Ḍuhuquʻi   Roduro   walking to work        health 100% stamina  34% load 20/88 kg [8458 m from the others]
  == Day 2, 15:01 (day) · Stillham ==
   *Ḍuhuquʻi   Roduro   walking to work        health 100% stamina  18% load 20/88 kg [6160 m from the others]
  ```
- Seen how many times: twice (also from `tr21-d2-0815-atwork` with `go e 16000`).
- Also: a worker told to `rest` 300 m from the post during shift hours is back "at work" within minutes; `rest` prints "Resting." and is silently overridden (same copy: `select Ḍuhuquʻi`, `go n 300`, `rest`, `wait 60`).

### Pay is all or nothing on a hidden threshold, and a missed day is never mentioned
- Part: Town jobs and recruiting
- How bad: rough (the pay line itself is misleading: "for a day's work")
- What happens: a worker who was away from 12:30 to about 16:30 (5 hours present out of 9, having walked 17 km in between) is paid the full 19 coin "for a day's work". A worker present for about 20 minutes of the shift gets nothing and no line at all: the journal still says "0 days paid" and never shows missed days.
- What should happen: the player can tell what counted as a day, and is told when a day did not count.
- To see it: seed 21. Five hours: copy `tr21-half-dawn` is the result (from `tr21-midshift`: `select Ḍuhuquʻi`, `go e 9000`, waits to 17:01, `wait 785`). Twenty minutes: copy `tr21-short-dawn` (from `tr21-d2-0815-atwork`: `select Ḍuhuquʻi`, `go e 16000`, `wait 120` ×4, `wait 60`, `wait 740`).
- Output:
  ```
  (five hours present)
  == Day 3, 06:06 (day) · Stillham ==
  Squad (6 members, 109 coin between them)
    06:00  Ḍuhuquʻi is paid 19 coin for a day's work.
  (twenty minutes present)
  == Day 3, 06:05 (day) · Stillham ==
  Squad (6 members, 90 coin between them)
  Town work: Ḍuhuquʻi works as labourer and porter at the food shop in Stillham, 08:00–17:00, 19 coin a day (0 days paid; shift today from 08:00).
  ```
- Seen how many times: once each. A full day pays 19 as promised (`tr21-predawn`, `wait 2`).

### A recruit's kit sells for far more than the sign-on fee, and drifters join for nothing: coin from nothing
- Part: Town jobs and recruiting
- How bad: wrong outcome
- What happens: Sathufiya signs on for 28 coin wearing a kite shield a Stillham merchant buys for 118; Yawashifas (28 coin) wears an iron helm that sells for 101. Take them off, sell: 109 coin becomes 328. From Day 3 five drifters join "for nothing"; one (Shawahis) brings a glaive ("sells for 152 in Stillham") and a scale hauberk ("sells for 220"). The `town` panel lists all of them with their kit, so the player can shop for the richest recruit.
- What should happen: a recruit's fee bears some relation to what they carry, or their own kit can't be stripped and sold at once.
- To see it: seed 21, copy `tr21-sellkit` (Day 3 08:08, Ruqox selected, talking to merchant Goṭu p817): `select Ruqox`, `unequip Sathufiya off`, `unequip Yawashifas head`, `talk p817`, `say 6`, `say 13`, `say 13`, `look`. Result kept as `tr21-sellkit-after`. For the free ones: copy `tr21-ten-full`, `pack Shawahis`.
- Output:
  ```
      you: Sell kite shield (118 coin)
    » I'll give you 118 for the kite shield.
      you: Sell iron helm (101 coin)
    » I'll give you 101 for the iron helm.
  Squad (6 members, 328 coin between them):
  ...
      p4602 Shawahis (drifter), for nothing — 22 m north-east
  Shawahis — carrying 25.2 of 44 kg
     MainHand   Glaive  (worth ~180; sells for 152 in Stillham)
     Body       Scale hauberk  (worth ~260; sells for 220 in Stillham)
  ```
- Seen how many times: once (two recruits sold in the same run).

### Two posts lost after three missed days with no word at all
- Part: Town jobs and recruiting
- How bad: misleading (the player is never told)
- What happens: Ḍuhuquʻi and Numeleo hold labourer posts in Stillham. The squad stays in Stillbank on Days 4, 5 and 6; each morning the two walk off and stick (see the finding above). At dawn on Day 7 both "Town work" lines are simply gone from the HUD and the journal. No News line on any of the missed days or when the posts end.
- What should happen: a line each missed day, and a line when the post is taken away.
- To see it: seed 21, copy `tr21-d6-0958` (Day 6 09:58): `select all`, `wait 1210`, `journal`. Result kept as `tr21-d7-posts-gone`.
- Output:
  ```
  (before, Day 6)
  Town work: Ḍuhuquʻi works as labourer and porter at the food shop in Stillham, 08:00–17:00, 19 coin a day (2 days paid; should be there now).
  Town work: Numeleo works as labourer and porter at the food shop in Stillham, 08:00–17:00, 19 coin a day (1 days paid; should be there now).
  (after, Day 7 06:08)
  Town work: Ruqox works as smith at the smithy in Stillbank, 08:00–17:00, 49 coin a day (2 days paid; shift today from 08:00).
  Town work: Sathufiya works as labourer and porter at the food shop in Stillbank, 08:00–17:00, 19 coin a day (0 days paid; shift today from 08:00).
  News:
    04:37  Numeleo wakes, rested.
    04:38  Ḍuhuquʻi wakes, rested.
    06:00  Hiqìṭuḍe takes up work as alchemist in Stillham.
    ...
    06:00  Ruqox is paid 49 coin for a day's work.
  ```
- Seen how many times: once (both members at the same dawn).

### Between Stillham and Stillbank there is a spot where `go` to a town and "walking to work" both stop dead
- Part: Odd input / Town jobs and recruiting
- How bad: stuck (for a member with a post: they march on the spot every shift; for the squad: rough, a sidestep gets past)
- What happens: about 920–970 m north of Stillham, `go t4` (northbound) and `go t5` (southbound) stop with everyone "standing" and no reason; repeating the order prints "(0 minutes pass.)". Going north, `go n 500` then `go t4` gets through. Going south I found no order that got two members through in several tries (`go s 500`, `go e 400`, `go e 300`, `go se 400`, each followed by `go t5`); the workers' own "walking to work" sticks at the same place on Day 2, 4, 5 and 6.
- What should happen: a route between two neighbouring towns, or a line saying why they stopped.
- To see it: seed 21. Northbound: copy `tr21-d3-leave` (Day 3 17:51, squad in Stillham): `go t4`, `go t4`, `map`. Southbound: copy `tr21-stuck-two` (Day 6 07:36): `select Ḍuhuquʻi Numeleo`, `go t5`, `go t5`.
- Output:
  ```
  (11 minutes pass.)
  == Day 3, 18:02 (night) · the wilds ==
  (0 minutes pass.)
  == Day 3, 18:02 (night) · the wilds ==
    t5  Stillham (342 people) — 918 m south
    t4  Stillbank (138 people) — 1.6 km north
  ```
- Seen how many times: northbound twice (Day 1 and Day 3, same place); southbound four times on the first line, and again on the long line (below).
- More from the long line (Day 12–13): (1) two members sent ahead stick at the same place and `go n 1500`, `go nw 2000`, `go w 2000` all print "(0 minutes pass.)" while `go ne 2000` and `go e 2000` walk (copy `tr21-stuck-north` + `.session`). (2) The whole squad coming back sticks at "Stillham 969 m south"; `go se 300` then `go t5` gets the HUD to "Stillham", but three of the seven are left standing 650 m out (copy `tr21-stuck-south` + `.session`: `go se 300`, `go t5`, `look`). (3) For those three, `go b5.2`, `go e 200`, `go b5.2`, `go t14`, `go t5` each print "(30 minutes pass.)" and nobody moves; the one with a post shows "walking to work" and loses stamina (copy `tr21-three-left-behind`: `select Ḍuhuquʻi Weyusauthi Yawashifas`, then those orders). One of them alone does move with `go 0,0`. So they are walking into something between them and the town, for as long as they are told to, and the tool says only that time passed.

### Quitting a post in the afternoon forfeits the whole day, and quitting is only possible while an official is on duty
- Part: Town jobs and recruiting
- How bad: rough
- What happens: "I'm giving up this work" appears only while an official is at their own work (about 07:30 to 16:00 here; neither official offers it at 16:50 or 17:35). Quitting at 15:50, after nearly eight hours of a nine-hour shift, gives no pay for that day at the next dawn and no line saying so.
- What should happen: the day worked is paid (or the official says it will be lost).
- To see it: seed 21, copy `tr21-quit-before` (Day 3 15:50, Ḍuhuquʻi talking to official Tuʻeṭeqo p334): `say 7`, `bye`, `journal`, `wait 855`.
- Output:
  ```
      you: I'm giving up this work
    » So be it.
  == Day 4, 06:04 (day) · Stillham ==
  Squad (6 members, 128 coin between them):
    06:00  Numeleo is paid 19 coin for a day's work.
  ```
  (109 coin before; only Numeleo's 19 arrives)
- Seen how many times: once.

### Each member eats only from their own pack, `give` hands over the whole stack, and nobody says they are starving
- Part: Town jobs and recruiting (a recruit's food)
- How bad: rough
- What happens: a recruit arrives with 3 flatbread. With none left they go "weak with hunger" then "STARVING" while standing beside squadmates who carry bread and fish and with 328 coin in the squad. The only notice is the tag on the HUD line and one TIP; there is no News line. `give NAME N TO` moves the whole stack (19 flatbread at once), so bread bought by one member can't be shared out: each member has to be the talker and buy their own. (`drop` on the other hand drops one at a time.)
- What should happen: a way to hand over part of a stack, and a line when someone runs out of food.
- To see it: seed 21, copy `tr21-ten-starving` (Day 5 08:22, ten members): `look`, `pack Weyusauthi`, `pack Heyesiyi`. For `give`: copy `tr21-d3-1735`, `select all`, `talk p3765`, `say 6`, `say 6` (buys one; repeat), `bye`, then `give Ḍuhuquʻi N Sathufiya` with N the flatbread line.
- Output:
  ```
    Heyesiyi   Ṭaḍoro   standing               health 100% stamina 100% load 2/46 kg STARVING
  TIP: Someone's hungry. They eat from their pack when they need to: buy food from a merchant, or hunt (click a wild animal) and cut up what you kill.
  ...
  Ḍuhuquʻi gives Sathufiya 19 × flatbread.
  ```
- Seen how many times: once (starving); twice (whole-stack give: 19, then 20).

### The squad limit holds at ten
- Not a bug. Seed 21, copy `tr21-ten-full` (ten members): `talk p4843`, `say 6` gives "There's no room for another of you, by the look of it." The `town` panel then says "Willing to join (0)" and the "restless" tags go. There is no command or topic to send a member away.

### Too much telling: the town panel and `look` list everyone who would join, their price, kit and whereabouts, before anyone has been spoken to
- Part: Town jobs and recruiting
- How bad: too much telling
- What happens: on the first `look` of a new game a stranger 29 m away is tagged "[restless: might join for 28 coin]" with their skills and kit. `town` lists all 14 "Willing to join" in Stillham with price, best skills, weapon, clothes and where they are right now, including "asleep indoors". The opening TIP says so outright.
- What should happen: found out by talking.
- To see it: seed 21, copy `tr21-start`: `look`, `town`.
- Output:
  ```
    p4587  Sathufiya — Ṭaḍoro labourer and porter [restless: might join for 28 coin] — 29 m east
          good at blunt, block, athletics · club and kite shield · wears leather cap, cloth shirt, trousers, boots
    Willing to join (14):
      p385 Huḍaqoḍi (labourer and porter), 28 coin — asleep indoors
          good at spear, block, blunt · spear · wears leather cap, hide coat, leather gloves, trousers, boots
  ```
- Seen how many times: every `look` and `town` in both worlds.

### Too much telling: News lines the squad could not know or would not miss
- Part: Townsfolk's lives
- How bad: too much telling
- What happens: the News feed carries, while the squad stands or sleeps in town:
  - strangers' comings and goings, many an hour, often the same wanderer again and again, also while everyone is asleep;
  - fights out of sight with an exact distance;
  - townsfolk taking jobs, including in a town the squad is not in;
  - "X is paid N coin for a day's work." each dawn for each working member.
- To see it: seed 21, copy `tr21-watch-d2-0700` then `wait 180`; copy `tr21-d6-0958` then `select all`, `wait 1210`.
- Output:
  ```
    07:14  Sawisasis (Ṭaḍoro), wandering, crosses your path.
    07:29  Siwiwesith (Ṭaḍoro), wandering, crosses your path.
    09:25  Sawisasis (Ṭaḍoro), wandering, crosses your path.
    02:10  Fusefeyi (Ṭaḍoro), wandering, crosses your path.          (squad asleep)
    06:30  A Mirejaw falls on travellers 1619 m away.
    06:30  The roadside fight is over.
    06:00  Fauwashuwi takes up work as meal runner in Stillbank.
    06:00  Hiqìṭuḍe takes up work as alchemist in Stillham.         (squad 1 km out of Stillham)
    06:00  Ḍuhuquʻi is paid 19 coin for a day's work.
  ```
- Seen how many times: every day played.

### Too much telling: every local knows the whole world's running count of ambushes
- Part: Townsfolk's lives (rumours)
- How bad: too much telling
- What happens: "Latest rumours" from labourers, drifters and merchants quotes an exact count that ticks up while you go from one person to the next: 87, then 88 a few minutes later (Day 3, Stillham); 129 then 130 (Day 4, Stillbank).
- What should happen: hearsay about something near, without a counter.
- To see it: seed 21, copy `tr21-sellkit`: `bye`, then for several people `talk pID`, pick "Latest rumours", `bye` (p4744, p3765, p956, p4686, p739 gave it).
- Output:
  ```
  p4744 Wasuwaufi:   » Bandits have fallen on travellers 87 times since the season turned. People say the roads aren't what they used to be.
  p956 Theloḍuʻa:   » 87 times this season, bandits have fallen on folk on the road. In my grandmother's day you could walk to the coast with your purse in your hand.
  p4686 Yifauwah:   » Bandits have fallen on travellers 88 times since the season turned. People say the roads aren't what they used to be.
  ```
- Seen how many times: 13 of 38 people asked in two towns (9 of 24 in Stillham, 4 of 14 in Stillbank), plus Stillbank's arbiter.

### Rumours: four lines between 24 people; "today" turns into "yesterday" within the hour
- Part: Townsfolk's lives (rumours)
- How bad: rough / cosmetic
- What happens: of 24 people who answered in Stillham (Day 3), 7 gave "They say the Ṭaḍoro write down everything you tell them…", 9 the ambush count, 6 the camp "3.3 km east", 2 the Stone Tenders. Nothing about neighbours, grudges or anything that happened in Stillham itself. The same person gives the same line when asked twice (fine), except news of a crime, which is reworded each time. The official's greeting says "Trokxuq squeezed money out of Timix, today" at Day 3 16:51 and "…, yesterday" at Day 3 17:35.
- To see it: seed 21, copy `tr21-quit-before`'s parent `tr21-predawn`: `wait 651`, `talk p334` ("today", 16:51); copy `tr21-d3-1735`: `talk p334` ("yesterday", 17:35).
- Output:
  ```
  » ... Word came by the road from Marshport: Trokxuq squeezed money out of Timix, today. That's an hour's walk from here, to the north.
  » ... Word came by the road from Marshport: Trokxuq squeezed money out of Timix, yesterday. That's an hour's walk from here, to the north.
  ```
- Seen how many times: once each.

### "I'll tell the watch." does nothing the player can see
- Part: Crime and the law
- How bad: rough (unsure what it is meant to do)
- What happens: an official who passes on news of a crime in another town ("Word came by the road from Stillham: Yafiwuwi squeezed money out of Goṭu, two days ago") offers the topic "I'll tell the watch." as topic 1. Picking it gives "Good. Let them answer for it." No News line, no journal entry, and nothing anywhere shows the squad's standing, so the player cannot tell whether anything happened. It also shifts every other topic number down by one once used (I took a labourer post by mistake that way).
- To see it: seed 21, copy `tr21-tellwatch` (Day 6 07:36, Sathufiya talking to Stillbank's official p665): `say 1`, `bye`, `journal`, `look`.
- Output:
  ```
      you: I'll tell the watch.
    » Good. Let them answer for it.
  ```
- Seen how many times: twice (Stillham Day 3, Stillbank Day 6).

### Smaller things (seed 21)
- **Standing is shown nowhere** in the text tool (`town`, `journal`, `look`, talks). After a paid job for Ayeyash (25 coin, kelp handed over, as promised) nothing says the town thinks better of the squad. (rough)
- **The town fills a post the squad member holds.** Ruqox takes "smith at the smithy" in Stillbank on Day 4 and is paid 49 coin for Days 5 and 6; the News says "06:00 Goleru takes up work as smith in Stillbank." (Day 5) and "06:00 Qahire takes up work as smith in Stillbank." (Day 6), and the panel's "Work going" changes each day (labourer, smith, armourer, labourer). Copy `tr21-d5-0815`. (unsure: may be several smith places)
- **`talk` sends member 0 even when he is a kilometre away.** With nobody (or everybody) selected, `talk p4547` to someone 14 m from four members prints "(10 minutes pass.) / They're not talking (or you couldn't reach them)." because Ḍuhuquʻi, stuck 745 m off, was sent. Copy `tr21-kelp-before`: `talk p4547`. (rough; candidate 7)
- **The tool keeps the selection and the read-mark for News outside the save** (`NAME.session`). A copied `.save` starts with nobody selected; copy the `.session` too when re-running my copies where one member is selected. (note for whoever re-runs these)
- **Two runs from the same copy with the same commands print the same text but the save files differ in 6–13 bytes** (two entries swapped in order). Seen from `tr21-midshift` (3.5 days) and `tr21-recruit-open`. No visible difference followed. (unsure: something saved in an order that is not fixed)
- **Jobs don't say where the person is.** "Bring Ayeyash in Stillbank 3 × kelp frond": Ayeyash was not within 70 m at five spots round the town nor at the dock at 07:35–07:40; found only at the 18:00 gathering. The board's "It's yours. Go and see them." gives no place. (rough)
- **`look places` never lists the hall, the stalls, the food shop or the smithy** that talk and the journal name ("at the hall, 60 m south of here", "at the food shop"). (rough)
- **Wording:** "(1 minutes pass.)", "1 days paid", "[Done] Done: a job for Ayeyash of Stillbank.", "TALKING with Sathufiya (Ṭaḍoro no trade)". (cosmetic)
- **A stranger greets the squad "There you are! I was hoping you'd come by."** on first meeting when the talker is Roduro (farmer Qari p992, official p334, Day 1 07:0x); the same official greeted a Horaro member "You're not from here, are you?". (unsure)
- **Merchants' hours:** what `town` says matched who would trade every time I tried (Day 1 13:00 and 18:00, three merchants each; Day 3). Sausai's opening time moves day to day (08:30, 07:15, 08:15, 11:00) and she was shut at 13:00 on Day 1 but open at 13:00 on Day 2. Goṭu dropped off the Merchants list altogether at 18:00 and 22:00 on Day 1 and was back next day. (unsure)

## Seed 3 (Stilledge, Windflat)

### Coin and goods are shared across any distance: a recruit's fee is paid from a pack 2.8 km away, and a job is handed in with goods held in another town
- Part: Town jobs and recruiting
- How bad: wrong outcome
- What happens: (a) Wehu, with no coin, stands in Windflat and signs on a guard for 64 coin; the coin leaves Menmuk's pack in Stilledge, 2.8 km away (76 → 12). (b) Wehu in Stilledge tells Aheweyu "About that job..." for "bring 3 × kelp frond"; the three fronds leave Heyuth's pack, who is at work in Windflat 2.3 km away. (c) Seed 21: "Sell all" sold timber carried by members 457 m off (finding above).
- What should happen: only what the talker and those standing with them carry can be paid, sold or handed over.
- To see it: seed 3. (a) copy `tr3-guard-recruit` (+ its `.session`; Day 5 13:5x, Wehu talking to guard Tonqox p1512 in Windflat): `pack Menmuk`, `say 6`, `pack Menmuk`. (b) copy `tr3-kelp-afar` (+ `.session`; Day 6, Wehu selected in Stilledge): `pack Heyuth`, `talk p4507`, `say 6`, `pack Heyuth`.
- Output:
  ```
  (a)   4. 76 × Coin [money]  (worth ~1 each)          <- Menmuk, in Stilledge
        you: Come with us — join the squad (64 coin to sign on)
      » 64 coin to my household, and I'm yours. Where are we going?
        4. 12 × Coin [money]  (worth ~1 each)
  (b)   Heyuth     Ṭaḍoro   at work    ... [2302 m from the others]
        you: About that job...
      » That's all of them. 0 coin — fair's fair.
        2. 1 × Kelp frond   (worth ~2 each; sells for 2 in Stilledge)   <- Heyuth had 4
  ```
- Seen how many times: once each (three different actions, two worlds).
- Also (cosmetic): a job posted as "a favour owed" is closed with "0 coin — fair's fair."

### A labourer who has never touched alchemy is hired as the town's alchemist at 49 coin a day; the post is also given to a local the same dawn
- Part: Town jobs and recruiting
- How bad: wrong outcome (unsure whether intended)
- What happens: Doqe (recruited labourer; `craft Doqe` says "No crafts taken up yet", Alchemy not taken up) is offered and takes "alchemist at the healing house (49 coin a day)". Next dawn the News says "Yeyasa takes up work as alchemist in Stilledge." and Doqe is still "at work" there from 08:00. In seed 21 the same happened with a smith's post (Goleru, then Qahire "takes up work as smith in Stillbank" while Ruqox held and was paid for it).
- What should happen: a skilled post asks for the skill; a post held by a squad member is not also filled.
- To see it: seed 3, copy `tr3-alch-before` (+ `.session`; Day 5 08:48, Doqe talking to official Golegi p237): `craft Doqe`, `say 6`, `journal`.
- Output:
  ```
  What Doqe can make (`make Doqe N`):
   Not taken up yet (a crafter at work or a manual teaches): Handcraft, Smithing, Armoring, Stone-tending, Weaving, Inscription, Alchemy, Carpentry, Masonry.
   No crafts taken up yet.
      6. I'll work as alchemist at the healing house (49 coin a day, 8 till 5)
    » Good, Doqe. You'll work as alchemist at the healing house from tomorrow, 8 till 5, for 49 coin a day, paid each dawn.
    06:00  Yeyasa takes up work as alchemist in Stilledge.
    Doqe       Roduro   at work                health 100% stamina 100% load 6/58 kg
    06:00  Doqe is paid 49 coin for a day's work.          (next dawn; copy `tr3-d7-dawn`)
  ```
- Seen how many times: the unskilled hire once (seed 3); a held post also given to a local three times (alchemist in seed 3; smith on two dawns in seed 21, where Ruqox does have Smithing).

### A merchant lists only seven things for sale, and from Day 5 no food is among them in Stilledge or Windflat
- Part: Town jobs and recruiting (keeping the squad fed; the trade screen itself is another tester's)
- How bad: stuck for a lawful squad (nothing to eat can be bought)
- What happens: every "What have you got?" shows at most seven "Buy" lines and no "what else?" (selling has "What else would you take? (7 more)"). On Day 1 Hafith's seven included flatbread at 2 coin. On Day 5 and 6 both Stilledge merchants and three Windflat merchants show seven lines of arms and clothing only, the same seven for every merchant in a town. The squad ran out of food on Day 5 with 80 coin and could buy none; I fed them from the 3 flatbread each new recruit brings.
- What should happen: all wares can be reached; food can be bought.
- To see it: seed 3, copy `tr3-goon` (+ `.session`; Day 4, Doqe talking to Yaweth p3812): the list above topic "Come with us". Day 1 list: copy `tr3-tradeopen`.
- Output:
  ```
  (Day 1, Hafith)  6. Buy fishskin cap (23 coin)  7. Buy flatbread (2 coin)  8. Buy timber (4 coin)  9. Buy rock feedstock (3 coin)  10. Buy hide (9 coin)  11. Buy ash moss (4 coin)  12. Buy torch (3 coin)
  (Day 6, Yaweth and Hafith alike)  7. Buy iron helm (213 coin);  8. Buy spear (81 coin);  9. Buy nacre scale coat on seareed (510 coin);  10. Buy seareed padded jacket (42 coin);  11. Buy fishskin boots (20 coin);  12. Buy masterwork leather leggings (232 coin);  13. Buy club (25 coin);
  ```
- Seen how many times: five merchants in two towns on two days.

### "I'll tell the watch." is offered by anyone, costs nothing, and the victim then speaks of the squad member as someone who "helped us when nobody else would"
- Part: Crime and the law
- How bad: rough (gratitude for one click; and no sign of it where it happens)
- What happens: in Stilledge, drifters, a labourer and a mason pass on "Texet squeezed money out of Yaweth, yesterday" and each offers "I'll tell the watch." (no guard need be near). Picked once with Heyuth while talking to Yaweth herself: "Good. Let them answer for it." Nothing else is shown. Later Yaweth greets another squad member, Doqe, with a "Go on." topic and says "Heyuth helped us when nobody else would. I won't forget it. Good people, those." She is telling one member of the squad about another as if he were a stranger.
- What should happen: telling the watch means going to the watch; the thanks are shown when earned and addressed to the squad.
- To see it: seed 3, copy `tr3-yaweth` (+ `.session`; Day 4 08:44, Heyuth talking to Yaweth p3812): `say 1`, `bye`. Then copy `tr3-goon` (+ `.session`): `say 1`.
- Output:
  ```
  » ... They come round for their share every few days now. Smile, pay, and don't ask questions.
      you: I'll tell the watch.
    » Good. Let them answer for it.
  ...
      you: Go on.
    » Heyuth helped us when nobody else would. I won't forget it. Good people, those. There aren't so many of them.
  ```
- Seen how many times: once (the thanks); "I'll tell the watch" three times in two worlds, never with any visible result.

### Smaller things (seed 3)
- **Talk left open, again:** copy `tr3-tradeopen` (+ `.session`; Menmuk talking to Hafith): `go e 1200`, `say 7` buys flatbread from 1.2 km ("» 2 coin. There you are.").
- **A second post is refused, as it should be:** Heyuth holds "armourer at the workyard" in Windflat; Stilledge's official then offers him only "I'm giving up this work" (so a post can be quit in another town, but not doubled). Candidate 3's post-plus-post half: not seen.
- **Walking to work from the next town works when the way is clear:** Heyuth, asleep in Stilledge, sets off at 08:00 with no News line and is "at work" in Windflat (2.8 km) by 08:36.
- **`select all` then `go` pulls a worker off a post in another town:** Heyuth walked 2.7 km back to the squad for a 40 m move order, then turned round "walking to work". No warning. (rough)
- **The official offers two posts; the panel lists three or four.** Windflat's panel: smith, armourer, scribe, weaver; the official at the hall offers smith and armourer only. Stilledge Day 6: panel adds woodworker, talk doesn't. One of Windflat's two officials (Sifawuwah p4522, standing 80 m from the hall all of Day 5) offers no work at all at 11:24, 12:14 and 13:33, though `town` says "ask an official or the hall". (rough)
- **`talk` sometimes prints only "(10 minutes pass.)"** with no "A conversation opens.", yet the talk is open and ten minutes are gone; seen on a third and fourth talk in a row with the same merchant (copy `tr3-alch-before`: `bye`, `select Wehu`, `wait 96`, then `talk p4492`, `bye` four times). (rough)
- **"We owe 2 coin, and it isn't getting any smaller, however hard we work. You learn to go without."** (drifter Fewafeyeth p4810, Windflat, Day 5): a lament over two coin, with the exact sum. (cosmetic / too much telling)
- **All crime news in both worlds is the same deed:** nine different "X squeezed money out of Y" stories from seven towns in six days, nothing else. Told as "today", "yesterday", "two days ago" with the walking time and direction. (rough: one story on repeat. In the long Stillham run the official opened with the same Marshport story every morning from Day 3 to Day 9, only the "N days ago" changing.)
- **News from other towns:** "06:00 Siwiwesith takes up work as smith in Woodsands." while the squad sleeps in Stilledge, 2.5 km away. (too much telling)
- **One-off jobs are slow to appear:** in the first six days and about 280 talks across four towns nobody had "Any work?"; the boards held only letters (46, 47, 140 coin and "a favour owed") and "bring 3 × kelp frond". A letter job names the town but the journal never says how far or which way (Mossham is 18 km; Windhead 13.8 km).


## Seed 21 again: the long stay in Stillham (`tr21-long`, Days 4 to 12)

A second line from `tr21-d3-leave`: the squad stays in Stillham, two members keep their labourer posts (paid 19 each at every dawn, Days 4 to 12, no slips), everyone buys their own bread from Siyashifah, and each morning at 07:40 the two officials and three merchants are asked. First job other than a letter or kelp: Day 7 (collect a debt). First "Any work?" from a person: Day 12. No guard, "get back what was stolen", or unlawful job in twelve days.

### Telling the watch through anyone earns "X helped us when nobody else would", said to X's own face, by someone the crime never touched
- Part: Crime and the law / Townsfolk's lives
- How bad: wrong outcome
- What happens: Stillham's official passes on news of a Marshport extortion ("Trokxuq squeezed money out of Timix") and offers "I'll tell the watch." Weyusauthi picks it. Open the talk again and the official's greeting is "Weyusauthi helped us when nobody else would. I won't forget it." It is said to Weyusauthi himself, in the third person, by an official of a different town from the victim's. Another squad member gets the same line about him. Seed 3 has the same from the victim Yaweth, told to squadmate Doqe.
- What should happen: thanks (if any) from the people helped, for something done, addressed as "you" to the one who did it.
- To see it: seed 21, copy `tr21-long-d7-board` (Day 7 08:3x): `select Weyusauthi`, `talk p334`, `say 1`, `bye`, `talk p334`, `bye`, `select Ruqox`, `talk p334`.
- Output:
  ```
    » ... Word came by the road from Marshport: Trokxuq squeezed money out of Timix, four days ago. That's an hour's walk from here, to the north.
      1. I'll tell the watch.
    » Good. Let them answer for it.
    » Good to see you again. Come in. Weyusauthi helped us when nobody else would. I won't forget it. A kindness is like a seed. You never know how big it will grow.
    » There you are! I was hoping you'd come by. Thelo (peace) to you, oqe (friend). Weyusauthi helped us when nobody else would. I won't forget it. ...
  ```
- Seen how many times: three (twice from this copy, once on the long line with Goṭu), plus once in seed 3.

### "Here's something for your trouble (10 coin)" is offered on people who know nothing, with no job taken, and the coin is kept
- Part: Town jobs and recruiting ("find out who wronged X")
- How bad: wrong outcome
- What happens: on Day 12 the merchant Goṭu (robbed in the night) shows two extra topics, "Here's something for your trouble (10 coin)." and "You'll tell me what you know.", though no job about it has been taken or even offered. Paying takes 10 coin and gets "I don't know anything about it." After taking the board job "find out who wronged Xuqud — 15 coin", the same two topics are on Xuqud himself, the man who posted the job; paying him 10 gets "I don't know anything about it.", and so does leaning on him. Nobody else among 50 people asked had the topics, so the job could not be done.
- What should happen: the topics appear on people who might know, once there is something to ask about; the asker is not a suspect in his own case; coin is not taken for "I don't know".
- To see it: seed 21. No job: copy `tr21-long-d12-findout` (Day 12 08:0x): `select Weyusauthi`, `talk p817`, `say 2`, `look`, `journal`. The asker: copy `tr21-findout-asker` (+ `.session`): `talk p1665`, `say 2`, `say 2`.
- Output:
  ```
  TALKING with Goṭu (Roduro merchant):
    » ... Somebody got into my goods shop last night while I was asleep and took spear. I didn't hear a thing. ...
      1. Go on.
      2. Here's something for your trouble (10 coin).
      3. You'll tell me what you know.
      you: Here's something for your trouble (10 coin).
    » I don't know anything about it.
  Squad (6 members, 318 coin between them)        (328 before)
  ...
  TALKING with Xuqud (Qotiro guard):
    » ... I've been robbed. They took today, and I'd only turned my back for a moment. ...
      you: Here's something for your trouble (10 coin).
    » I don't know anything about it.
      you: You'll tell me what you know.
    » I don't know anything about it.
  ```
- Seen how many times: twice (Goṭu with no job; Xuqud the asker).
- Also: picking "Go on." first makes both topics drop out of the list for the rest of that talk; they are back when the talk is opened again (copy `tr21-findout-asker`: `talk p1665`, `say 1`, then read the topics; `bye`, `talk p1665`). And the wording: "took spear", "They took today".

### A creditor pays 15 coin to have a 9-coin debt collected
- Part: Town jobs and recruiting
- How bad: wrong outcome (money sense)
- What happens: the board has "Sausai wants someone to collect 9 coin owed by Saewaha — 15 coin." Asking the debtor twice gets "I don't know what you're talking about." "I'll make it worth your while." costs the squad 10 coin and he answers "Here — 9. Tell them we're square." Sausai then pays 15 "as promised". So Sausai is 6 coin worse off than forgetting the debt, and the debtor is 1 coin up.
- What should happen: a reward smaller than the debt.
- To see it: seed 21, copy `tr21-debt-talk` (+ `.session`; Day 7 08:56, Weyusauthi talking to Saewaha p3809): `say 6`, `say 7`, `bye`, then at 10:05 `talk p4231`, `say 6`.
- Output:
  ```
      you: About that matter...
    » I don't know what you're talking about.
      you: I'll make it worth your while.
    » Here — 9. Tell them we're square.
  Squad (6 members, 153 coin between them)        (163 before)
      you: About that job...
    » You did it? Then here — 15 coin, as promised.
  Squad (6 members, 168 coin between them)
  ```
- Seen how many times: once.
- Also from the same copy: "Tell me, or else." gets "Do your worst. I'm not afraid of you." as often as it is said (seven times), with no witness, no line and no change in the town; only Saewaha's next greeting changes ("I've nothing to say to your sort. Move along.").

### Smaller things (long stay)
- **Two people of one name in a town, and jobs name only the first name.** "For Xuqud of Stillham: find out who wronged Xuqud": Stillham has Xuqud the guard (p1665) and Xuqud the mason (p2393). Also two Tuʻeṭeqo, two Garoṭaqa, two Leshala in Stillham; two Yawewishih in seed 3 (a Woodsands merchant and a Windflat drifter), which makes the gossip about "Yawewishih" unreadable. (rough)
- **"Maybe I can help." and "Any work?" are both listed and do the same thing**; after either, "Maybe I can help." stays in the list beside "I'll do it." (copy `tr21-long-d12-anywork`: `talk p4774`). (cosmetic)
- **A favour letter is offered as "0 coin when it's done — they'll pay you."** (Wuhes p4776, Day 12). (cosmetic)
- **The man who "can't see to it myself" joins the squad for nothing a moment after handing over his letter** and walks along while the squad carries it (copy `tr21-wuhes`: `talk p4776`, pick "Any work?", "I'll do it.", "Come with us"). (rough)
- **Asking round costs hours.** A round of 50 talks took from 10:31 to 14:28: many talks print "(10 minutes pass.)" before opening. (rough)
- **`talk` answers "Can't talk to them."** for a merchant not yet up (Sausai at 08:58, "at their stall from 10:00"), with no word on why or where she is. The job then waits on her opening hours. (rough)

- **With the squad split, `look`, `town` and the HUD header describe where most of the squad is, not where the selected members are.** Two members sent to Stillbank print "Nobody needs to move: they're already there." under a header that still says "Stillham" with Stillham's people; they cannot see anyone in Stillbank, so there are no ids to talk to (copy `tr21-stuck-north` + `.session`: `go ne 1200`, `go t4`, `go t4`, `look people`). (rough; text tool)
- **The letter for "Moanolu in Stillbank" could not be delivered:** Moanolu was not among the 99 people at Stillbank's 18:05 gathering, nor at four spots round the town, nor at the dock at 18:15 and 06:00. Probably out on the stilts (also seen: the known "sent somewhere you can't reach").

## Townsfolk's lives: what I watched (seed 21, Stillham, squad standing still at the centre; copies `tr21-watch-d1-1300` … `tr21-watch-d3-0700`)

`look people` reaches 70 m and does not list people indoors asleep, so "–" means "not within 70 m or indoors".

| Who | D1 07:00 | D1 10:00 | D1 13:00 | D1 18:00 | D1 22:00 | D2 03:00 | D2 07:00 | D2 10:00 | D2 13:00 | D2 18:00 | D2 22:00 | D3 03:00 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Tuʻeṭeqo, official p334 | 68 m S | 68 m S | 67 m S | 20 m NW | – | – | 67 m S | 66 m S | 66 m S | 21 m NW | – | – |
| Siyashifah, merchant p3765 | 55 m SW, trading | 54 m SW, trading | 53 m SW, trading | 68 m W | – | – | 53 m SW, trading | 52 m SW, trading | 53 m SW, trading | 66 m W | 36 m W | – |
| Hushu, scribe p580 | 41 m E | 41 m E | 42 m E | 12 m NW | – | – | 52 m NE | 53 m NE | 55 m NE | 14 m SE | 12 m W | – |
| Ṭeqahuḍu, teacher p261 | 42 m E | 41 m E | 42 m E | 39 m NE | – | – | 41 m E | 43 m E | 44 m E | 43 m NE | 38 m NE | – |
| Doruhulu, priest p1056 | 56 m E | 56 m E | 55 m E | 67 m W | – | – | 56 m E | 54 m E | 53 m E | – | – | – |
| Sefisai, fisher p4732 | 22 m W | 20 m W | 19 m W | 22 m W | – | – | – | – | – | 65 m W | 66 m W | – |
| Qari, farmer p992 | 68 m S | 69 m S | 70 m S | 65 m S | – | – | – | – | – | 9 m W | – | – |
| Weyewesa, exchanger p3829 | 49 m SE | 44 m E | 43 m E | 5 m NW | 6 m N | – | – | – | – | – | – | – (back 45 m E at D3 07:00) |

- It hangs together: a work place by day, somewhere else in the evening, nobody listed at 03:00 except three to five travellers. About 160 people are within 70 m of the centre at 18:00 on both days (an evening gathering), about 100 at 22:00.
- Nobody stood in one spot for 24 hours. Ten labourers were at the same work spot (54–57 m south-west) at 07:00, 10:00 and 13:00 on all three mornings and near their homes at 18:00 and 22:00.
- Odd, unsure: on Day 1 the fisher stood in the middle of town and the farmer stood by the hall all day (07:00–18:00); on Day 2 both were away by day. The exchanger was nowhere within 70 m for the whole of Day 2. A fisher is 177 m from the dock and a farmer 350 m from the fields when standing there, so Day 1 looks like a day they did not go to work.
- Merchants: tried to trade with all three at D1 13:00 and D1 18:00 (copies above): each time the `town` panel's "trading now / at their stall from HH:MM" matched whether "What have you got?" was offered.
- Coast: at Stillham's dock the squad can get no nearer than 46 m east of it ("Nobody needs to move: they're already there."). On Day 3 and Day 6, 16–18 fishers are 50–68 m west at 05:57–06:00 and gone by 06:05: the boats do go at dawn. One fisher is about at midday and none at 22:00. Nothing in `look`, `town` or `map` names the stilt village or says anything about the tide, and `shot` answers "Couldn't take a screenshot.", so the stilts and the tide could not be looked at.

## Coverage

Town jobs
- Find work by talking (officials, people, board): **tried.** Posts from officials in Stillham, Stillbank, Stilledge, Windflat; boards read in Stillham, Stillbank, Stilledge; "Any work?" first seen on Day 12 (two people, both letters).
- Take a post, work a full day, be paid: **tried.** 19 coin as promised (labourer), 49 (smith, alchemist, armourer), every dawn for ten days on the long line.
- There half the day: **tried.** 5 of 9 hours: full pay. 20 minutes: nothing, no line.
- Quit: **tried.** Only while an official is on duty; the day in hand is forfeited.
- A post and a one-off job with the same member: **tried.** Post + letter (Numeleo), post + letter + kelp (Ruqox; kelp handed in, paid). No clash seen. Post + guard work: **not reached** (no guard work was ever offered). A second post: refused, as it should be.
- Take a post, walk the squad to another town, 08:00 next day: **tried** (Stillham→Stillbank, Stilledge→Windflat).
- "Get back what was stolen" job: **not reached** (none offered in 12 + 7 days).
- Unlawful job: **not reached** (none offered). "Tell me, or else." on a debtor was tried on a copy: refused, no consequence.
- Other jobs reached: bring kelp (done twice), collect a debt (done), find out who wronged someone (taken, could not be done), letters (taken; none delivered: Windhead 13.8 km and Mossham 18 km not walked, Moanolu not findable).

Recruiting
- Who joins and for how much: **tried** (28–88 coin; drifters free from Day 3).
- Up to 10 and an 11th: **tried** (refused at ten).
- What a recruit brings: **tried** (own kit, 3 flatbread; kit can be sold at once).
- A recruit's food: **tried** (own pack only; starve beside others' food; `give` moves whole stacks).
- Getting rid of someone: **tried, not possible** (no command, no topic).

Townsfolk's lives
- Five named people over two days at six hours: **tried** (eight people; table above).
- Merchants' hours against `town`: **tried** (matched).
- Gossip, grudges, rumours: **tried** (55 people asked "Latest rumours" twice each in three towns; the greetings of about 190 more checked for news of crimes; "Go on." heard from about 30). Grudges: only "I've nothing to say to your sort" after threats; no grudge between townsfolk heard.
- Coastal town, stilt village, tides: **tried, mostly not reachable** (boats at dawn seen twice; stilts and tide not shown by the tool).

The law, the lawful way
- Standing in a town: **tried, nothing to see** (shown nowhere).
- What officials offer: **tried** (posts, the board, "I'll tell the watch", news; one official teaches).
- A 50-coin note: **tried** (51 coin in, 49 out; not accepted for a recruit's fee or in a shop, though the HUD counts it). Fines and bounties: **not reached** (no crime committed).

Save and load
- With a talk open: **tried** (two copies of `tr21-recruit-open`, same seven commands: same text).
- Mid-shift: **tried** (two copies of `tr21-midshift`, 3.5 days: same text).
- The minute before dawn pay: **tried** (two copies of `tr21-predawn`: same text).
- In all three the save files themselves differed by a handful of bytes (order of two entries); no visible difference followed.

Commands used: well over a thousand across the two worlds (I did not keep an exact count); most were inside the talk-to-everyone loops.

## Candidates (from `code-law-jobs-recruit.md`)

1. Talk stays open while the world runs: **seen** (trade from 1.5 km, recruiting from 2 km after two hours; both worlds).
2. Crash on a bad person id / empty squad: not tried (already known).
3. One member, two jobs, walking back and forth: **not seen.** A second post is refused. A post plus a letter or fetch job causes no walking. Post plus guard work: couldn't reach (no guard work offered).
4. "Get back what was stolen" pays without the thing: **couldn't reach.** For a "bring N" job the item is required ("That's not 3. Come back when it is.", copy `tr21-kelp-before`), but it can be in a pack 2.3 km away.
5. Squad limit passed by picking up from a base: not tried (no base in the text tool). No way to dismiss: **seen.**
6. A member walks off alone to a post from any distance with no message; post gone after three missed days with no line: **seen**, both halves, plus the walking-on-the-spot when the way is blocked. Quitting before dawn forfeits the day: **seen.** "paid 0 coin": not seen.
7. `talk` always sends the first selected member: **seen** (member 0 stuck 745 m away was sent; "They're not talking (or you couldn't reach them)."). The stale talk order blocking a job: not tried.
8. Notes don't count for fees: **seen** for a recruit's fee and a shop. Fines, bounties, buy-outs: couldn't reach.
9. Duel binds the accused wherever they are: not tried (lawful play).
10. Getting away is cheaper than being caught: not tried.
11. Unlawful jobs raise standing: **couldn't reach** (none offered; standing not shown).
12. Rewards paid without looking in the purse: **couldn't reach** directly. Related and seen: a 15-coin reward for collecting a 9-coin debt (the "at least 15" floor).
13. Hiring an officeholder to an outpost: not tried (no base).
14. Too much telling: "X is paid N coin for a day's work." **seen**; ambush count in rumours **seen**; "restless: might join for N coin" before speaking **seen**; `town` giving trade counts, merchants' hours and the whole willing-to-join list from a distance **seen** (from 125 m and from the dock); alerts printed twice, the witness line, the bond lines: not reached (no crime).
15. A town post missing from `journal`: **not seen** (it is there, with days paid and "starts tomorrow / at work now / should be there now / shift over"). Missed days are not shown: **seen.** A confirmation line dropped when stamped the same minute as the last one read: **not seen** (the "joins the squad" line appeared; copy `tr21-recruit-open`: `look`, `say 6`, `look`).

Smaller things from the same file
- "Labourer is always offered, every member can take it in the same town": **seen** (two members took the same Stillham labourer post).
- No eligibility check on skilled posts: **seen** (alchemist).
- The town fills the same post with a local while the squad's contract is paid: **seen** (three dawns).
- A quest giver recruited: **seen** that it can be done (Wuhes joins straight after handing over his letter); what the quest then does: not reached.
- The rest (shunning, bound members, pursuit dropped, arbiter post, guard job with missed days, coin above 65,535, disposition by race): not reached, except that greetings do differ by the talker's people (a Roduro member is greeted "There you are! I was hoping you'd come by." by Roduro strangers who greet a Horaro member "You're not from here, are you?").
