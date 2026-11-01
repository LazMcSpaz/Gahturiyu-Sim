# d-main log: Test World D, layer 2 (the realm): main story, Return, minors, fallout

Seed `random.Random(6601)` for the spine and plan (rolls.py, in order); keyed rolls `K[6601, …]` (casting §8) for people and ways.
Rules v6, card schema v1.11. Build: `D/d-main-src/` (rolls.py → people.py → lib.py → registry.py → cards_main.py → cards_side.py → build.py).

## Spine

- Arc **Rising**, shape **Above** (forced by Rising), **L=6 × W=2** (diamond 1-2-2-2-2-1 + Return). Antagonist Litiṭaʻa of the Great House (Throne), want **preservation**.
- Heart **Kemed Oath-keeper** (Middle miller of the Gate Mill, Crownsgate; earlier-wave Qotiro); fallback Xipid Road-warden; stake **the Gate Mill**; no heart secret (rolled no).
- Hand Trerxuq Fire-walker (crown clerk of the sealing office); lieutenant Laʻetalu of the Bark-Field House; ally Haʻahoqo the General (price: the company's arms and bondsmen's oaths); fault line Ḍuhuḍì of the Spring-Cliff House.
- Moves: m1 kin (the will) · m2 debt (the sealing tenth) · m3 force (company in Longwall's towers) · m4 bribery (the Great Bridge re-granted, the house's advance) · m5 law (the writ: disgraced) · m6 force (Crownsgate seized; the company named the emperor's guard). Reserved lever: force.
- Revelation (layer 4, published by c-d-main-15 and c-d-main-18): Litiṭaʻa seals in the emperor's name because his hands can no longer hold the seal, to raise a company that holds Highcrossing when he dies.

## Rolls (seeded, in order)

- arc: **Rising** (r=0.2604, of ['Rising', 'Ripple', 'Inward', 'Deep'])
- shape: **Above** — Rising allows only Above (main-quest §6)
- L: **6** (r=0.3683, of ['5', '6', '7', '8']) — Rising range 5-8, uniform (main-quest §3)
- W: **2** (r=0.3615, of ['2', '3']) — 2 <= W <= floor((L+1)/2)
- antagonist side: **f-throne** (r=0.1613) — equal weights
- want: **preservation** (r=0.8709, of ['greed', 'revenge', 'cover-up', 'desperation', 'ambition', 'preservation']) — the Throne's own want (preservation) x2
- hook settlement: **pl-crownsgate** (r=0.3858, of ['pl-oathford', 'pl-crownsgate', 'pl-saltmarket', 'pl-carters-rest'])
- hook family: **Dispute** (r=0.5471, of ['Investigate', 'Threat', 'Rescue', 'Dispute', 'Acquire', 'Passage'])
- hook premise: **Contested inheritance** (r=0.1718, of ['Contested inheritance', 'Shared resource'])
- ordeal event: **disgraced** (r=0.1763, of ['disgraced', 'estranged'])
- climax premise: **Ruin from above** (r=0.4848, of ['Ruin from above', 'Schism'])
- heart has a secret: **False** (r=0.8191, p=0.5)
- hook turn family: **Reckoning** (r=0.9573, of ['Investigate', 'Threat', 'Passage', 'Reckoning'])
- hook turn premise: **Swindle** (r=0.9964, of ['Betrayal', 'Ruin from above', 'Blood debt', 'Swindle'])
- threshold turn A family: **Rescue** (r=0.7326, of ['Passage', 'Dispute', 'Rescue'])
- threshold turn B family: **Dispute** (r=0.8473, of ['Passage', 'Dispute'])
- trials-4 chapter A family: **Investigate** (r=0.3902, of ['Investigate', 'Reckoning'])
- trials-3 turn A family: **Acquire** (r=0.9082, of ['Threat', 'Passage', 'Acquire'])
- trials-3 turn B family: **Passage** (r=0.9626, of ['Threat', 'Passage'])
- threshold A (Rescue) premise: **Imprisoned unjustly** (r=0.0853, of ['Kidnapped', 'Captured in battle', 'Imprisoned unjustly', 'Bound by debt']) — the move takes men for an unpaid tenth: Bound by debt x2; no lawless band or battle on the card's ground
- threshold B (Dispute) premise: **Broken agreement** (r=0.3628, of ['Broken agreement', 'Shared resource']) — Contested inheritance is the hook's, Public offense the Ordeal door's
- trials-3 A (Acquire) premise: **Won't sell** (r=0.017, of ["Won't sell", 'Passed on', 'Under seal', 'Unknown worth']) — the lieutenant holds by crown warrant: Under seal x2
- trials-3 B (Passage) premise: **Fugitive** (r=0.546, of ['Smuggling', 'Fugitive'])
- trials-4 A (Investigate) premise: **Sabotage** (r=0.8946, of ['Theft', 'Disappearance', 'Suspicious death', 'Sabotage'])
- trials-4 B (Reckoning) premise: **Betrayal** (r=0.077, of ['Betrayal', 'Ruin from above', 'Blood debt']) — Swindle is the hook turn's; Ruin from above is also the climax's (allowed, premise <= 3)
- which ordeal chapter gets which door: **A Passage / B Dispute** (r=0.8291, of ['A Dispute / B Passage', 'A Passage / B Dispute'])
- ch2a opening family: **Acquire** (r=0.4058, of ['Passage', 'Acquire', 'Investigate', 'Threat', 'Restore', 'Dispute', 'Contest']) — m2 takes men and levies a tenth: Passage, Acquire lean
- ch2b opening family: **Rescue** (r=0.1544, of ['Rescue', 'Passage', 'Acquire', 'Investigate', 'Threat', 'Restore', 'Contest']) — a taking leans Rescue; a levy Acquire
- c3a-open family: **Restore** (r=0.7304, of ['Threat', 'Passage', 'Investigate', 'Restore', 'Reckoning', 'Contest']) — m3 is force on the road: Threat x2; Acquire (turn), Rescue and Dispute (linked turns) out
- c3a-mid family: **Reckoning** (r=0.7312, of ['Dispute', 'Rescue', 'Threat', 'Passage', 'Investigate', 'Reckoning', 'Contest'])
- c3b-open family: **Restore** (r=0.6411, of ['Acquire', 'Threat', 'Investigate', 'Restore', 'Reckoning', 'Contest']) — carts stopped and goods taken: Acquire x2; Passage (turn), Rescue, Dispute out
- c3b-mid family: **Threat** (r=0.4478, of ['Dispute', 'Acquire', 'Rescue', 'Threat', 'Investigate', 'Reckoning', 'Contest'])
- c4a-open family: **Contest** (r=0.9371, of ['Dispute', 'Threat', 'Rescue', 'Restore', 'Reckoning', 'Contest']) — m4 re-grants tolls and buys keepers: Dispute x2; Investigate (turn), Acquire, Passage (linked turns) out
- c4a-mid family: **Acquire** (r=0.0972, of ['Acquire', 'Dispute', 'Threat', 'Rescue', 'Restore', 'Reckoning', 'Passage'])
- c4b-open family: **Dispute** (r=0.1784, of ['Dispute', 'Threat', 'Rescue', 'Restore', 'Investigate', 'Contest']) — Reckoning (turn), Acquire, Passage out
- c4b-mid family: **Passage** (r=0.8156, of ['Acquire', 'Threat', 'Rescue', 'Restore', 'Investigate', 'Passage', 'Contest'])
- c5a-open family: **Restore** (r=0.5468, of ['Acquire', 'Dispute', 'Restore', 'Rescue', 'Threat', 'Contest']) — the writ bars her papers: Acquire x2; Passage (door), Investigate, Reckoning (linked turns) out
- c5a-mid family: **Rescue** (r=0.4163, of ['Acquire', 'Dispute', 'Rescue', 'Threat', 'Contest', 'Investigate', 'Reckoning'])
- c5b-open family: **Acquire** (r=0.1784, of ['Acquire', 'Passage', 'Restore', 'Rescue', 'Threat', 'Contest']) — Dispute (door), Investigate, Reckoning out
- c5b-mid family: **Reckoning** (r=0.9974, of ['Passage', 'Restore', 'Rescue', 'Threat', 'Contest', 'Investigate', 'Reckoning'])
- c6-open family: **Threat** (r=0.3349, of ['Acquire', 'Threat', 'Restore', 'Investigate', 'Contest']) — m6 seizes a village: Acquire, Threat x2; Rescue (the forced middle), Reckoning (turn), Dispute, Passage (linked turns) out
- ret-open family: **Passage** (r=0.8858, of ['Acquire', 'Dispute', 'Threat', 'Investigate', 'Rescue', 'Passage', 'Contest']) — the aftermath of a seizure: Acquire, Dispute x2; Restore (turn), Reckoning (linked climax turn) out
- c2a-open premise: **Passed on** (r=0.3391, of ["Won't sell", 'Passed on', 'Under seal', 'Unknown worth']) — crown clerks hold Bogford's goods for the tenth: Under seal x2; nothing of unknown worth there
- c2b-open premise: **Imprisoned unjustly** (r=0.2864, of ['Kidnapped', 'Imprisoned unjustly', 'Bound by debt']) — men taken for an unpaid tenth: Bound by debt x2
- c3a-open premise: **Lost livelihood** (r=0.9821, of ['Broken works', 'Estrangement', 'Lost livelihood']) — riders on the road drive trade off it: Lost livelihood x2
- c3b-open premise: **Broken works** (r=0.3783, of ['Broken works', 'Estrangement', 'Lost livelihood']) — carts dodging the riders load the quiet crossing: Broken works x2
- c3a-mid premise: **Betrayal** (r=0.1991, of ['Betrayal', 'Ruin from above', 'Blood debt', 'Swindle'])
- c3b-mid premise: **Raiders** (r=0.052, of ['Raiders', 'Protected beast', 'Sickness or blight', 'Encroachment']) — a Binder raid camp lies within reach; no preserve by the march road
- c4a-open premise: **The fix** (r=0.2676, of ["Champion's bout", 'The fix', 'First blood or last', 'The ladder'])
- c4a-mid premise: **Unknown worth** (r=0.9596, of ['Passed on', 'Under seal', 'Unknown worth']) — Won't sell is the trials-3 turn's
- c4b-open premise: **Shared resource** (r=0.2256, of ['Shared resource', 'Contested inheritance', 'Broken agreement']) — Public offense is the Ordeal door's
- c4b-mid premise: **Unwilling passenger** (r=0.5272, of ['Smuggling', 'Unwilling passenger']) — Fugitive is the trials-3 turn's; Unwelcome cargo the Ordeal door's
- c5a-open premise: **Estrangement** (r=0.3382, of ['Broken works', 'Estrangement', 'Lost livelihood']) — Lost good name left to the Return, which repairs the disgrace
- c5a-mid premise: **Kidnapped** (r=0.3711, of ['Kidnapped', 'Bound by debt']) — Imprisoned unjustly already twice (threshold A turn, threshold B opening): out
- c5b-open premise: **Passed on** (r=0.3798, of ["Won't sell", 'Passed on', 'Under seal', 'Unknown worth']) — the writ bars her papers: Under seal x2
- c5b-mid premise: **Swindle** (r=0.8105, of ['Betrayal', 'Ruin from above', 'Blood debt', 'Swindle'])
- c6-open premise: **Sickness or blight** (r=0.5739, of ['Raiders', 'Sickness or blight', 'Encroachment']) — no preserve near Oathford
- ret-open premise: **Unwilling passenger** (r=0.7413, of ['Smuggling', 'Fugitive', 'Unwilling passenger', 'Unwelcome cargo'])
- c27 twist?: **True** (r=0.6282, p=0.7) — turning card
- c27 twist: **the wronged don't want it** (r=0.1109, of ["the wronged don't want it", 'changed'])
- c25 twist?: **False** (r=0.8298, p=0.7) — turning card
- c26 twist?: **False** (r=0.7593, p=0.7) — turning card
- c29 twist?: **True** (r=0.4038, p=1.0) — forced
- c29 twist: **someone wants it broken** (r=0.324, of ['someone wants it broken'])
- c28 twist?: **True** (r=0.2514, p=0.7) — turning card
- c28 twist: **the one left behind** (r=0.5431, of ['the one left behind'])
- c21 twist?: **True** (r=0.2123, p=0.7) — turning card
- c21 twist: **the one left behind** (r=0.6438, of ['pursuer has a point', 'the one left behind'])
- c24 twist?: **True** (r=0.5598, p=0.7) — turning card
- c24 twist: **law on the wrong side** (r=0.675, of ['old wound', 'law on the wrong side'])
- c19 twist?: **True** (r=0.6019, p=0.7) — turning card
- c19 twist: **someone wants it broken** (r=0.364, of ['someone wants it broken', 'it broke for a reason'])
- c20 twist?: **True** (r=0.1706, p=0.7) — turning card
- c20 twist: **none** — every eligible group spent or used in a linked chapter
- c22 twist?: **False** (r=0.984, p=0.7) — turning card
- c23 twist?: **True** (r=0.6514, p=0.7) — turning card
- c23 twist: **hand not head** (r=0.8114, of ['changed', 'hand not head'])
- c2 twist?: **False** (r=0.7851, p=0.7) — turning card
- c4 twist?: **True** (r=0.5741, p=0.7) — turning card
- c4 twist: **unwilling captive** (r=0.1485, of ['unwilling captive', 'justified captor', 'not alone'])
- c6 twist?: **True** (r=0.3943, p=0.7) — turning card
- c6 twist: **hollow prize** (r=0.4597, of ['caught in the middle', 'hollow prize', 'old wound', 'law on the wrong side'])
- c9 twist?: **False** (r=0.8691, p=0.7) — turning card
- c12 twist?: **False** (r=0.8452, p=0.7) — turning card
- c15 twist?: **True** (r=0.6202, p=0.7) — turning card
- c15 twist: **wrong target** (r=0.8064, of ['wrong target'])
- c18 twist?: **True** (r=0.023, p=0.7) — turning card
- c18 twist: **the wronged don't want it** (r=0.7833, of ["the wronged don't want it"])
- c1 twist?: **True** (r=0.27, p=0.35)
- c1 twist: **caught in the middle** (r=0.3105, of ['caught in the middle', 'old wound', 'law on the wrong side'])
- c3 twist?: **False** (r=0.8935, p=0.35)
- c5 twist?: **True** (r=0.3164, p=0.35)
- c5 twist: **justified captor** (r=0.6941, of ['justified captor'])
- c7 twist?: **False** (r=0.9701, p=0.35)
- c8 twist?: **True** (r=0.3176, p=0.35)
- c8 twist: **none** — every eligible group spent or used in a linked chapter
- c10 twist?: **False** (r=0.9741, p=0.35)
- c11 twist?: **True** (r=0.6071, p=1.0) — forced
- c11 twist: **one of us caused it** (r=0.2678, of ['one of us caused it'])
- c13 twist?: **True** (r=0.0453, p=0.35)
- c13 twist: **none** — every eligible group spent or used in a linked chapter
- c14 twist?: **False** (r=0.979, p=0.35)
- c16 twist?: **False** (r=0.5296, p=0.35)
- c17 twist?: **False** (r=0.6057, p=0.35)
- c27 truth row: **Reckoning: as told** (r=0.1537, of ['Reckoning: as told', 'Reckoning: a witness exists', 'Reckoning: shared blame', 'Reckoning: still profiting'])
- c25 truth row: **Threat: a sick animal traded in** (r=0.5299, of ['Threat: bad water', 'Threat: waste from upstream', 'Threat: a sick animal traded in', 'Threat: carried in by travelers', 'Threat: spoiled stores'])
- c26 truth row: **Rescue: real and fair** (r=0.7814, of ['Rescue: misassigned', 'Rescue: paid', 'Rescue: inflated', 'Rescue: real and fair'])
- c28 truth row: **Passage: not what it seems** (r=0.5745, of ['Passage: as declared', 'Passage: not what it seems', 'Passage: the pursuer was told'])
- c21 truth row: **Passage: the pursuer was told** (r=0.991, of ['Passage: as declared', 'Passage: not what it seems', 'Passage: the pursuer was told'])
- c24 truth row: **Dispute: no record** (r=0.3043, of ['Dispute: record survives', 'Dispute: no record', 'Dispute: one side lies', 'Dispute: both partly right'])
- c19 truth row: **Restore: a secret that got told** (r=0.875, of ['Restore: a missed deathbed', 'Restore: a spouse one of them disapproved of', "Restore: a side taken in someone else's quarrel", 'Restore: a secret that got told'])
- c20 truth row: **Rescue: a better buyer** (r=0.8885, of ['Rescue: inside help', 'Rescue: a better buyer'])
- c22 truth row: **Acquire: a copy** (r=0.6194, of ['Acquire: promised elsewhere', 'Acquire: a copy'])
- c23 truth row: **Reckoning: a witness exists** (r=0.2661, of ['Reckoning: as told', 'Reckoning: a witness exists', 'Reckoning: shared blame', 'Reckoning: still profiting'])
- c2 truth row: **Reckoning: shared blame** (r=0.6279, of ['Reckoning: a witness exists', 'Reckoning: shared blame', 'Reckoning: still profiting'])
- c4 truth row: **Rescue: guilty** (r=0.7253, of ['Rescue: innocent', 'Rescue: guilty'])
- c6 truth row: **Dispute: record survives** (r=0.1853, of ['Dispute: record survives', 'Dispute: no record', 'Dispute: one side lies', 'Dispute: both partly right'])
- c9 truth row: **Acquire: a copy** (r=0.9729, of ['Acquire: promised elsewhere', 'Acquire: a copy'])
- c12 truth row: **Passage: the pursuer was told** (r=0.7897, of ['Passage: not what it seems', 'Passage: the pursuer was told'])
- c18 truth row: **Reckoning: a witness exists** (r=0.2968, of ['Reckoning: as told', 'Reckoning: a witness exists', 'Reckoning: shared blame', 'Reckoning: still profiting'])
- c1 truth row: **Dispute: one side lies** (r=0.7416, of ['Dispute: record survives', 'Dispute: no record', 'Dispute: one side lies', 'Dispute: both partly right'])
- c3 truth row: **Acquire: promised elsewhere** (r=0.4949, of ['Acquire: as told', 'Acquire: promised elsewhere', 'Acquire: a copy'])
- c7 truth row: **Restore: the trade road moved** (r=0.0712, of ['Restore: the trade road moved', 'Restore: buyers went elsewhere'])
- c8 truth row: **Reckoning: shared blame** (r=0.7169, of ['Reckoning: as told', 'Reckoning: a witness exists', 'Reckoning: shared blame', 'Reckoning: still profiting'])
- c10 truth row: **Restore: built badly** (r=0.3516, of ['Restore: worn out and neglected', 'Restore: built badly', 'Restore: the ground moved', "Restore: stones carted off for someone's wall"])
- c11 truth row: **Threat: driven out** (r=0.2737, of ['Threat: hunger', 'Threat: driven out', 'Threat: paid', 'Threat: opportunity'])
- c13 truth row: **Contest: the money** (r=0.0746, of ['Contest: the money', 'Contest: someone above'])
- c14 truth row: **Acquire: as told** (r=0.1976, of ['Acquire: as told', 'Acquire: promised elsewhere', 'Acquire: a copy'])
- c16 truth row: **Dispute: both partly right** (r=0.9886, of ['Dispute: record survives', 'Dispute: no record', 'Dispute: one side lies', 'Dispute: both partly right'])
- c17 truth row: **Passage: not what it seems** (r=0.6029, of ['Passage: as declared', 'Passage: not what it seems', 'Passage: the pursuer was told'])
- c2 shield: **Fled** (r=0.7367, of ['No law against it', 'Fled']) — Bought needs the arbiter linked to the clerk: no such link
- c8 shield: **No law against it** (r=0.1882, of ['No law against it', 'Fled']) — Rank and Protected fail on standing
- c18 shield: **Bought** (r=0.4805, of ['No law against it', 'Bought', 'Protected'])
- c23 shield: **No law against it** (r=0.4554, of ['No law against it', 'Fled'])
- c27 shield: **Rank** — only the emperor outranks Litiṭaʻa, and he doesn't hear such matters (fixed by standing)
- minor on chapter h: **False** (r=0.6748, p=0.5)
- minor on chapter 2a: **False** (r=0.7619, p=0.5)
- minor on chapter 2b: **False** (r=0.5174, p=0.5)
- minor on chapter 3a: **True** (r=0.2746, p=0.5)
- minor on chapter 3b: **True** (r=0.475, p=0.5)
- minor on chapter 4a: **False** (r=0.5728, p=0.5)
- minor on chapter 4b: **False** (r=0.5468, p=0.5)
- minor on chapter 5a: **True** (r=0.047, p=0.5)
- minor on chapter 5b: **False** (r=0.585, p=0.5)
- minor on chapter 6: **False** (r=0.7677, p=0.5)
- fallout count, ordeal move (realm): **2** (r=0.1044, of ['2', '3'])
- fallout count, last move (realm): **2** (r=0.4813, of ['2', '3'])
- minor 3a twist: **one of us caused it** — Threat Encroachment never rolls none; only You started it is open to 3a
- minor 3a truth (Encroachment, public): **Threat: project flawed** (r=0.0388, of ['Threat: project flawed', 'Threat: project sound'])
- minor 3b truth (Under seal): **Acquire: a copy** (r=0.726, of ['Acquire: promised elsewhere', 'Acquire: a copy']) — no open twist group: a non-confirming row
- minor 3b seal type: **Fine unpaid** (r=0.15, of ['Fine unpaid', 'Ownership unproven', "Held as surety for someone else's debt", 'Held until a ruling'])
- minor 5a twist: **none** (r=0.8199, of ['law on the wrong side', 'old wound', 'none'])
- minor 5a truth (Broken agreement): **Dispute: no record** (r=0.3873, of ['Dispute: no record', 'Dispute: one side lies']) — record survives and both partly right are at 1 each; spread the rows
- fallout F6a twist?: **True** (r=0.1185, p=1.0) — Raiders never none
- fallout F6a twist: **one of us caused it** (r=0.1252, of ['one of us caused it', 'divided community', 'passed along', "protector's price"])
- fallout F1 twist?: **False** (r=0.6493, p=0.4)
- fallout F2a twist?: **False** (r=0.9899, p=0.4)
- fallout F2b twist?: **True** (r=0.3448, p=0.4)
- fallout F2b twist: **meant for destruction** (r=0.403, of ['wants to be rid of it', 'meant for destruction', 'rightful keeper', 'load-bearing'])
- fallout F2late twist?: **False** (r=0.8491, p=0.4)
- fallout F3a twist?: **False** (r=0.5486, p=0.4)
- fallout F3b twist?: **False** (r=0.6976, p=0.4)
- fallout F3late twist?: **True** (r=0.0482, p=0.4)
- fallout F3late twist: **needs it more** (r=0.2739, of ['needs it more'])
- fallout F4a twist?: **False** (r=0.8612, p=0.4)
- fallout F4b twist?: **False** (r=0.5948, p=0.4)
- fallout F4late twist?: **False** (r=0.9996, p=0.4)
- fallout F5a twist?: **False** (r=0.4733, p=0.4)
- fallout F5b twist?: **True** (r=0.2354, p=0.4)
- fallout F5b twist: **wants to be rid of it** (r=0.814, of ['wants to be rid of it'])
- fallout F5late twist?: **False** (r=0.5849, p=0.4)
- fallout F6b twist?: **False** (r=0.8382, p=0.4)
- fallout F6late twist?: **False** (r=0.9887, p=0.4)
- fallout F2a truth row: **Rescue: misassigned** (r=0.2366, of ['Rescue: misassigned', 'Rescue: paid', 'Rescue: inflated', 'Rescue: real and fair'])
- fallout F2b truth row: **Acquire: as told** (r=0.2084, of ['Acquire: as told', 'Acquire: promised elsewhere'])
- fallout F2late truth row: **Passage: the pursuer was told** (r=0.6572, of ['Passage: not what it seems', 'Passage: the pursuer was told'])
- fallout F3a truth row: **Restore: a secret that got told** (r=0.9565, of ['Restore: a missed deathbed', 'Restore: a spouse one of them disapproved of', "Restore: a side taken in someone else's quarrel", 'Restore: a secret that got told'])
- fallout F3late truth row: **Contest: the stated reason** (r=0.1896, of ['Contest: the stated reason', 'Contest: a cover', 'Contest: a way out'])
- fallout F4a truth row: **Reckoning: shared blame** (r=0.6355, of ['Reckoning: a witness exists', 'Reckoning: shared blame', 'Reckoning: still profiting'])
- fallout F4b truth row: **Passage: not what it seems** (r=0.4708, of ['Passage: not what it seems'])
- fallout F4late truth row: **Contest: a way out** (r=0.7459, of ["Contest: the sponsor's price", 'Contest: a way out'])
- fallout F5a truth row: **Dispute: record survives** (r=0.2281, of ['Dispute: record survives', 'Dispute: no record', 'Dispute: one side lies', 'Dispute: both partly right'])
- fallout F5b truth row: **Acquire: as told** (r=0.1701, of ['Acquire: as told', 'Acquire: promised elsewhere'])
- fallout F5late truth row: **Reckoning: still profiting** (r=0.7326, of ['Reckoning: a witness exists', 'Reckoning: still profiting'])
- fallout F6a truth row: **Threat: driven out** (r=0.2979, of ['Threat: hunger', 'Threat: driven out', 'Threat: paid', 'Threat: opportunity'])
- fallout F6late truth row: **Dispute: one side lies** (r=0.5914, of ['Dispute: record survives', 'Dispute: no record', 'Dispute: one side lies', 'Dispute: both partly right'])

## Way rolls (keyed K[6601, card, 'ways'])

- c-d-main-1: target 3; forced [], truth 0, random 1, random 4 → rolled [0, 1, 4]
- c-d-main-2: target 3; forced [], truth 2, random 1, random 4 → rolled [1, 2, 4]
- c-d-main-3: target 3; forced [], truth 1, random 4, random 3 → rolled [1, 3, 4]
- c-d-main-4: target 2; forced [], truth 0, random 1 → rolled [0, 1]
- c-d-main-5: target 3; forced [], truth 2, random 1, random 0 → rolled [0, 1, 2]
- c-d-main-6: target 2; forced [], truth 1, random 0, kinds 2 → rolled [0, 1, 2]
- c-d-main-7: target 3; forced [], truth 0, random 1, random 2 → rolled [0, 1, 2]
- c-d-main-8: target 3; forced [], truth 2, random 5, random 3 → rolled [2, 3, 5]
- c-d-main-9: target 2; forced [], truth 1, plant-backed 0 → rolled [0, 1]
- c-d-main-10: target 2; forced [], truth 1, random 2 → rolled [1, 2]
- c-d-main-11: target 3; forced [], random 2, random 1, random 0 → rolled [0, 1, 2]
- c-d-main-12: target 2; forced [], truth 4, random 3 → rolled [3, 4]
- c-d-main-13: target 3; forced [], truth 3, random 1, random 2 → rolled [1, 2, 3]
- c-d-main-14: target 2; forced [], truth 1, random 2 → rolled [1, 2]
- c-d-main-15: target 2; forced [], truth 0, random 2 → rolled [0, 2]
- c-d-main-16: target 2; forced [], truth 0, random 2 → rolled [0, 2]
- c-d-main-17: target 2; forced [], truth 2, random 3 → rolled [2, 3]
- c-d-main-18: target 2; forced [], truth 0, plant-backed 1 → rolled [0, 1]
- c-d-main-19: target 3; forced [], truth 1, random 0, random 2 → rolled [0, 1, 2]
- c-d-main-20: target 3; forced [], truth 4, random 0, random 2 → rolled [0, 2, 4]
- c-d-main-21: target 3; forced [], truth 4, random 3, random 2 → rolled [2, 3, 4]
- c-d-main-22: target 2; forced [], truth 0, random 2, kinds 1 → rolled [0, 1, 2]
- c-d-main-23: target 3; forced [], truth 1, random 2, random 0 → rolled [0, 1, 2]
- c-d-main-24: target 2; forced [], truth 0, plant-backed 1 → rolled [0, 1]
- c-d-main-25: target 3; forced [], truth 0, random 2, random 1 → rolled [0, 1, 2]
- c-d-main-26: target 3; forced [], truth 2, random 4, random 0 → rolled [0, 2, 4]
- c-d-main-27: target 2; forced [5, 6], truth 2, plant-backed 0 → rolled [0, 2, 5, 6]
- c-d-main-28: target 3; forced [], truth 2, random 3, random 0 → rolled [0, 2, 3]
- c-d-main-29: target 3; forced [], truth 0, random 2, random 1 → rolled [0, 1, 2]
- c-d-main-minor-1: target 2; forced [], truth 1, random 2 → rolled [1, 2]
- c-d-main-minor-2: target 3; forced [], truth 1, random 2, random 0 → rolled [0, 1, 2]
- c-d-main-minor-3: target 2; forced [], truth 0, random 1 → rolled [0, 1]
- c-d-main-fall-1: target 2; forced [], truth 0, random 1 → rolled [0, 1]
- c-d-main-fall-2a: target 2; forced [], truth 2, random 0 → rolled [0, 2]
- c-d-main-fall-2b: target 3; forced [], truth 1, random 2, random 3 → rolled [1, 2, 3]
- c-d-main-fall-2late: target 2; forced [], truth 2, random 1 → rolled [1, 2]
- c-d-main-fall-3a: target 2; forced [], truth 1, random 0, kinds 3 → rolled [0, 1, 3]
- c-d-main-fall-3b: target 2; forced [], truth 3, random 0 → rolled [0, 3]
- c-d-main-fall-3late: target 2; forced [], truth 1, random 2 → rolled [1, 2]
- c-d-main-fall-4a: target 2; forced [], truth 1, random 2 → rolled [1, 2]
- c-d-main-fall-4b: target 2; forced [], truth 3, random 0 → rolled [0, 3]
- c-d-main-fall-4late: target 3; forced [], truth 1, random 0, random 2 → rolled [0, 1, 2]
- c-d-main-fall-5a: target 2; forced [], truth 0, random 3 → rolled [0, 3]
- c-d-main-fall-5b: target 3; forced [], truth 0, random 1, random 2 → rolled [0, 1, 2]
- c-d-main-fall-5late: target 3; forced [], truth 1, random 0, random 2 → rolled [0, 1, 2]
- c-d-main-fall-6a: target 3; forced [], truth 2, random 0, random 3 → rolled [0, 2, 3]
- c-d-main-fall-6b: target 3; forced [], truth 0, random 2, random 1 → rolled [0, 1, 2]
- c-d-main-fall-6late: target 3; forced [], truth 0, random 2, random 1 → rolled [0, 1, 2]

## People (keyed K[6601, id, 'make', …]: people, age, manner, habit; names from the naming system, stems unique)

- p-d-main-heart: Kemed Oath-keeper (qotiro, grown, weary/eat)
- p-d-main-uncle: Pakrek Stair-builder (qotiro, old, blunt/decide)
- p-d-main-brother: Xipid Road-warden (qotiro, grown, proud/no_goodbye)
- p-d-main-cousin: Giktrok Spear-thrower (qotiro, grown, wry/decide)
- p-d-main-widow: Krekem Way-finder (qotiro, old, wry/eat)
- p-d-main-millhand: Mutuk Dawn-runner (qotiro, old, careful/decide)
- p-d-main-hand: Trerxuq Fire-walker (qotiro, grown, careful/decide)
- p-d-main-bridgekeeper: Aheh of the Silk Road (tadoro, grown, blunt/writes_it_down)
- p-d-main-bridgeson: Shasauthu of the Silk Road (tadoro, young, warm/corrects_self)
- p-d-main-sergeant: Lawinuu of the Reed Cove (horaro, grown, weary/we_for_all)
- p-d-main-sisterinlaw: Dayahi of the Six-Ring House (roduro, young, weary/so_first)
- p-d-main-carter: Doqoṭuḍì of the Grass-Ledge House (roduro, young, proud/stone_keep_you)
- p-d-main-stallwife: Qeḍa of the Gull-Brow House (roduro, old, careful/counts_seasons)
- p-d-main-towerwarden: Hiqìṭuḍe of the Rain-Hollow House (roduro, grown, careful/traveller)
- p-d-main-crossingkeeper: Texar Ore-finder (qotiro, grown, warm/eat)
- p-d-main-raider: Laʻedaqo of the Spring-Ridge House (roduro, grown, proud/traveller)
- p-d-main-informer: Dortik First-striker (qotiro, young, blunt/eat)
- p-d-main-fortsergeant: Hirilela of the Quiet Reef (horaro, old, careful/have_you_eaten)
- p-d-main-deserter: Nuqug Wall-warden (qotiro, young, careful/decide)
- p-d-main-tollreeve: Ḍoʻiqiʻi of the Ember-Ledge House (roduro, grown, warm/names_the_dead)
- p-d-main-paymaster: Yiheyaeh of the East Road (tadoro, grown, solitary/what_would_you_do)
- p-d-main-ladderkeeper: Oqi of the Moss-Field House (roduro, young, blunt/counts_seasons)
- p-d-main-oldkeeper: Tasho of the Herb-Field House (roduro, old, wry/so_first)
- p-d-main-paidfighter: Thelohoʻu of the Herb-Field House (roduro, young, careful/names_the_dead)
- p-d-main-sapper: Ruqred Iron-breaker (qotiro, grown, wry/decide)
- p-d-main-hurtcarter: Dodìli of the Thorn-Hollow House (roduro, grown, blunt/traveller)
- p-d-main-drover: Saiwasa of the White Road (tadoro, old, warm/have_you_noticed)
- p-d-main-chclerk: Nuihina of the Blue Reef (horaro, young, careful/we_for_all)
- p-d-main-ferrywoman: Dorì of the Vine-Cliff House (roduro, grown, wry/traveller)
- p-d-main-ferryson: Giqo of the Vine-Cliff House (roduro, young, weary/so_first)
- p-d-main-rivalferry: Thahuh of the Wind Road (tadoro, grown, blunt/corrects_self)
- p-d-main-lad: Gurile of the Tree-Ledge House (roduro, young, careful/traveller)
- p-d-main-bandleader: Ḍuqi of the Bark-Ridge House (roduro, grown, wry/stone_keep_you)
- p-d-main-paperdealer: Nami of the Rain Bay (horaro, grown, weary/we_for_all)
- p-d-main-notary: Fewash of the Old Road (tadoro, young, solitary/writes_it_down)
- p-d-main-saltcarter: Maiwunu of the White Stack (horaro, grown, weary/waits_for_a_sign)
- p-d-main-apprentice: Ponor Far-seer (qotiro, young, warm/curses_the_damp)
- p-d-main-factor: Weyeh of the Hill Road (tadoro, grown, warm/writes_it_down)
- p-d-main-fordwarden: Qahituḍe of the Moon-Ledge House (roduro, young, weary/so_first)
- p-d-main-remounter: Numeleo of the Kelp Pool (horaro, young, proud/we_for_all)
- p-d-main-tenthclerk: Yaḍoraqo of the Vine-Forest House (roduro, grown, weary/so_first)
- p-d-main-oldcarter: Pontriq Crag-runner (qotiro, old, blunt/eat)
- p-d-main-merchant: Truqged Shield-bearer (qotiro, grown, warm/eat)
- p-d-main-veteran: Huḍalego of the Spring-Crag House (roduro, old, proud/speaks_of_the_house)
- p-d-main-reedcutter: Yithaya of the Star Road (tadoro, old, solitary/have_you_noticed)
- p-d-main-raidsecond: Leshe of the Moon-Crag House (roduro, old, careful/stone_keep_you)
- p-d-main-olddrover: Ruʻalaʻo of the Herb-Brow House (roduro, old, weary/speaks_of_the_house)
- p-d-main-sweetheart: Ḍaqoʻara of the Rain-Bank House (roduro, young, blunt/so_first)
- p-d-main-writrider: Roshaʻo of the Gull-Stream House (roduro, young, careful/stone_keep_you)
- p-d-main-towerguard: Luweholo of the Pearl Cove (horaro, young, warm/another_day)
- p-d-main-bridgedaughter: Hethe of the Silk Road (tadoro, young, wry/what_would_you_do)
- p-d-main-wallreeve: Eheheya of the High Road (tadoro, old, careful/guesses_where_from)
- p-d-main-f1-baker: Shohiqa of the Tree-Hollow House (roduro, old, proud/traveller)
- p-d-main-f1-breadcarter: Shawasus of the Dawn Road (tadoro, grown, wry/listen_first)
- p-d-main-f2a-wife: Sayashuwi of the West Road (tadoro, grown, careful/corrects_self)
- p-d-main-f2a-debtor: Feiwahe of the West Road (tadoro, young, proud/have_you_noticed)
- p-d-main-f2a-keeper: Xutat Wall-leaper (qotiro, grown, proud/eat)
- p-d-main-f2a-neighbour: Ṭuḍiqahu of the Moon-Cliff House (roduro, grown, warm/stone_keep_you)
- p-d-main-f2b-broker: Mokpur Bone-breaker (qotiro, grown, proud/decide)
- p-d-main-f2b-daughter: Ritder Spear-breaker (qotiro, young, warm/no_goodbye)
- p-d-main-f2l-grainwife: Heyesiyi of the Sea Road (tadoro, grown, blunt/have_you_noticed)
- p-d-main-f2l-carrier: Huliro of the Moon-Brow House (roduro, grown, blunt/stone_keep_you)
- p-d-main-f2l-postguard: Sausuhethes of the Far Road (tadoro, grown, proud/writes_it_down)
- p-d-main-f3a-rider: Tonod Shield-breaker (qotiro, young, warm/eat)
- p-d-main-f3a-brother: Dapmut Gate-holder (qotiro, grown, careful/eat)
- p-d-main-f3b-carter: Ḍeḍedoqi of the Spring-Hill House (roduro, young, careful/names_the_dead)
- p-d-main-f3b-potboy: Wuwesh of the Rain Road (tadoro, young, careful/writes_it_down)
- p-d-main-f3b-drover: Aheweyu of the North Road (tadoro, young, weary/guesses_where_from)
- p-d-main-f3l-freeman: Tortopox Hall-builder (qotiro, grown, warm/curses_the_damp)
- p-d-main-f3l-rider: Ḍuʻa of the Bark-Crag House (roduro, young, weary/so_first)
- p-d-main-f3l-keeper: Legera of the Frost-Cliff House (roduro, grown, proud/traveller)
- p-d-main-f4a-boss: Limawu of the Rain Stack (horaro, grown, proud/talks_tide)
- p-d-main-f4b-tollwife: Ṭeqatiqe of the Nine-Ring House (roduro, old, weary/traveller)
- p-d-main-f4b-brother: Ṭelihoḍu of the Nine-Ring House (roduro, grown, wry/counts_seasons)
- p-d-main-f4b-tollman: Qoḍu of the Vine-Bank House (roduro, grown, weary/speaks_of_the_house)
- p-d-main-f4l-fighter: Ini of the Gull Bay (horaro, young, weary/we_for_all)
- p-d-main-f4l-opponent: Thuta of the Ember-Brow House (roduro, grown, proud/traveller)
- p-d-main-f4l-keeper: Quhoro of the Frost-Hollow House (roduro, old, weary/counts_seasons)
- p-d-main-f5a-sealcutter: Ḍoʻiḍole of the Tree-Hill House (roduro, grown, weary/names_the_dead)
- p-d-main-f5a-halfbrother: Yero of the Tree-Hill House (roduro, grown, wry/names_the_dead)
- p-d-main-f5a-father: Thuhe of the Tree-Hill House (roduro, old, wry/stone_keep_you)
- p-d-main-f5b-weaver: Suwai of the Stone Road (tadoro, old, solitary/writes_it_down)
- p-d-main-f5b-reeve: Wehu of the Deep Shore (horaro, old, proud/out_of_the_wind)
- p-d-main-f5l-sister: Ditxen Stone-breaker (qotiro, grown, proud/one_word)
- p-d-main-f5l-taker: Limu of the Deep Mouth (horaro, young, warm/another_day)
- p-d-main-f5l-dead: Kotxuq Sand-walker (qotiro, old, careful/no_goodbye)
- p-d-main-f6a-farmwife: Gexem Torch-bearer (qotiro, old, blunt/decide)
- p-d-main-f6a-son: Pokik Hound-hunter (qotiro, young, warm/eat)
- p-d-main-f6a-leader: Gedmag Pass-holder (qotiro, grown, blunt/decide)
- p-d-main-f6b-widow: Ḍoqaquhe of the Moss-Ridge House (roduro, grown, weary/stone_keep_you)
- p-d-main-f6b-dead: Menput Night-walker (qotiro, old, weary/curses_the_damp)
- p-d-main-f6b-culprit: Wasiwihu of the Long Road (tadoro, old, proud/writes_it_down)
- p-d-main-f6l-captain: Hononaha of the Quiet Island (horaro, grown, blunt/have_you_eaten)
- p-d-main-f6l-quartermaster: Sawis of the Salt Road (tadoro, young, warm/writes_it_down)
- p-d-main-f3a-mother: Raqtod Beast-hunter (qotiro, old, proud/one_word)

## Cards

- **c-d-main-1** (main L1 opening) Dispute/Contested inheritance at the Gate Mill (Crownsgate): truth *Dispute: one side lies*, twist Caught in the middle, ways [truth; truth; side].
- **c-d-main-2** (main L1 turn) Reckoning/Swindle at the crown gate-house (Crownsgate): truth *Reckoning: shared blame*, twist none, ways [ruin; amends; amends].
- **c-d-main-3** (main L2 opening) Acquire/Passed on at the Free Lord's yard at Bogford (Bogford): truth *Acquire: promised elsewhere*, twist none, ways [trade; deceive; take].
- **c-d-main-4** (main L2 turn) Rescue/Imprisoned unjustly at the lord's tower at Oathford (Oathford): truth *Rescue: guilty*, twist Unwilling captive, ways [leverage; price].
- **c-d-main-5** (main L2 opening) Rescue/Imprisoned unjustly at the Last Well (the Last Well): truth *Rescue: guilty*, twist Justified captor, ways [price; leverage].
- **c-d-main-6** (main L2 turn) Dispute/Broken agreement at the Long Shed (Carter's Rest): truth *Dispute: record survives*, twist Hollow prize, ways [truth; truth; deal].
- **c-d-main-7** (main L3 opening) Restore/Lost livelihood at the Broken Shrine (the Broken Shrine): truth *Restore: the trade road moved*, twist none, ways [persuade; gather; substitute].
- **c-d-main-8** (main L3 middle) Reckoning/Betrayal at the general's hall inside the inner wall (Longwall): truth *Reckoning: shared blame*, twist none, ways [amends; justice; ruin].
- **c-d-main-9** (main L3 turn) Acquire/Won't sell at Longwall's outer towers (Longwall): truth *Acquire: a copy*, twist none, ways [earn; trade].
- **c-d-main-10** (main L3 opening) Restore/Broken works at the Reed Crossing (the Reed Crossing): truth *Restore: built badly*, twist none, ways [craft; gather].
- **c-d-main-11** (main L3 middle) Threat/Raiders at the carters' camp under the Brennock road fort (Brennock road fort): truth *Threat: driven out*, twist One of us caused it, ways [confront; protect; undermine].
- **c-d-main-12** (main L3 turn) Passage/Fugitive at the breach at Breakwall (Breakwall): truth *Passage: the pursuer was told*, twist none, ways [force; arrangement].
- **c-d-main-13** (main L4 opening) Contest/The fix at the tavern-yard ring at Southroad (Southroad): truth *Contest: the money*, twist none, ways [stakes; rules; edge] — plain.
- **c-d-main-14** (main L4 middle) Acquire/Unknown worth at the west toll hut of the Great Bridge (Southroad): truth *Acquire: as told*, twist none, ways [trade; earn] — plain.
- **c-d-main-15** (main L4 turn) Investigate/Sabotage at the Great Bridge (Southroad): truth *Investigate: the story's own*, twist Wrong target, ways [examine; question].
- **c-d-main-16** (main L4 opening) Dispute/Shared resource at the river meadows below the Empty Gallery (Highcrossing): truth *Dispute: both partly right*, twist none, ways [truth; deal].
- **c-d-main-17** (main L4 middle) Passage/Unwilling passenger at the crown road from Highcrossing to Crownsgate (Highcrossing): truth *Passage: not what it seems*, twist none, ways [route; arrangement].
- **c-d-main-18** (main L4 turn) Reckoning/Betrayal at the Scales Hall (the Silver Road Counting-House): truth *Reckoning: a witness exists*, twist The wronged don't want it, ways [ruin; amends].
- **c-d-main-19** (main L5 opening) Restore/Estrangement at the Oathford ferry stair (Oathford): truth *Restore: a secret that got told*, twist Someone wants it broken, ways [persuade; craft; persuade].
- **c-d-main-20** (main L5 middle) Rescue/Kidnapped at the burned toll-tower on the Elderhollow road (Oathford): truth *Rescue: a better buyer*, twist none, ways [force; price; leverage].
- **c-d-main-21** (main L5 turn) Passage/Unwelcome cargo at the archive of the Ninth Ring (the Ninth Ring): truth *Passage: the pursuer was told*, twist The one left behind, ways [cover; force; arrangement].
- **c-d-main-22** (main L5 opening) Acquire/Passed on at the paper-dealer's shop on the salt quay (Saltmarket): truth *Acquire: a copy*, twist none, ways [earn; trade; earn].
- **c-d-main-23** (main L5 middle) Reckoning/Swindle at the notary's stall under the salt-hall arches (Saltmarket): truth *Reckoning: a witness exists*, twist Hand, not head, ways [ruin; ruin; amends].
- **c-d-main-24** (main L5 turn) Dispute/Public offense at the Unchained Quay (Openharbour): truth *Dispute: no record*, twist Law on the wrong side, ways [truth; deal].
- **c-d-main-25** (main L6 opening) Threat/Sickness or blight at the muster field below the lord's tower (Oathford): truth *Threat: a sick animal traded in*, twist none, ways [undermine; bargain; protect].
- **c-d-main-26** (main L6 middle) Rescue/Bound by debt at the crown gate-house (Crownsgate): truth *Rescue: real and fair*, twist none, ways [price; stealth].
- **c-d-main-27** (main L6 turn) Reckoning/Ruin from above at the Empty Gallery (Highcrossing): truth *Reckoning: as told*, twist The wronged don't want it, ways [ruin; amends; violence; amends].
- **c-d-main-28** (main L7 opening) Passage/Unwilling passenger at the Crownsgate toll bridge (Crownsgate): truth *Passage: not what it seems*, twist The one left behind, ways [arrangement; route; arrangement].
- **c-d-main-29** (main L7 turn) Restore/Lost good name at the crown gate-house (Crownsgate): truth *Restore: false, spread by someone off the card*, twist Someone wants it broken, ways [gather; craft; persuade].
- **c-d-main-minor-1** (minor → ch-d-main-3a) Threat/Encroachment at the wall-village under Longwall's inner wall (Longwall): truth *Threat: project flawed*, twist One of us caused it, ways [bargain; undermine].
- **c-d-main-minor-2** (minor → ch-d-main-3b) Acquire/Under seal at the breach at Breakwall (Breakwall): truth *Acquire: a copy*, twist none, ways [earn; trade; deceive].
- **c-d-main-minor-3** (minor → ch-d-main-5a) Dispute/Broken agreement at the Oathford ferry stair (Oathford): truth *Dispute: no record*, twist none, ways [truth; deal].
- **c-d-main-fall-1** (fallout m-d-main-1 endure) Restore/Lost livelihood at the gate oven at Crownsgate (Crownsgate): truth *Restore: buyers went elsewhere*, twist none, ways [craft; substitute].
- **c-d-main-fall-2a** (fallout m-d-main-2 endure) Rescue/Bound by debt at the Sedge pens (the Sedge pens): truth *Rescue: misassigned*, twist none, ways [price; leverage].
- **c-d-main-fall-2b** (fallout m-d-main-2 profit) Acquire/Won't sell at the debt broker's counting room at the Chainmarket (the Chainmarket): truth *Acquire: as told*, twist Meant for destruction, ways [trade; deceive; take].
- **c-d-main-fall-2late** (fallout m-d-main-2 resist checkpoint 1) Passage/Smuggling at Breakwall's grain market (Breakwall): truth *Passage: the pursuer was told*, twist none, ways [route; cover].
- **c-d-main-fall-3a** (fallout m-d-main-3 endure) Restore/Estrangement at the riders' row at Highcrossing (Highcrossing): truth *Restore: a secret that got told*, twist none, ways [persuade; persuade; substitute].
- **c-d-main-fall-3b** (fallout m-d-main-3 endure) Investigate/Theft at the carters' inn at Southroad (Southroad): truth *Investigate: a bystander*, twist none, ways [examine; follow].
- **c-d-main-fall-3late** (fallout m-d-main-3 resist checkpoint 1) Contest/First blood or last at Breakwall's ladder yard (Breakwall): truth *Contest: the stated reason*, twist Needs it more, ways [stakes; rules].
- **c-d-main-fall-4a** (fallout m-d-main-4 profit) Reckoning/Ruin from above at the crew boss's wagon yard at Carter's Rest (Carter's Rest): truth *Reckoning: shared blame*, twist none, ways [amends; violence].
- **c-d-main-fall-4b** (fallout m-d-main-4 resist) Passage/Fugitive at the carters' loft at Wainyard (Wainyard): truth *Passage: not what it seems*, twist none, ways [cover; arrangement].
- **c-d-main-fall-4late** (fallout m-d-main-4 endure checkpoint 2) Contest/The ladder at the tavern-yard ring at Carter's Rest (Carter's Rest): truth *Contest: a way out*, twist none, ways [prepare; edge; rules].
- **c-d-main-fall-5a** (fallout m-d-main-5 mirror) Dispute/Contested inheritance at the seal-cutter's shop on Longwall's wall street (Longwall): truth *Dispute: record survives*, twist none, ways [truth; side].
- **c-d-main-fall-5b** (fallout m-d-main-5 endure) Acquire/Under seal at the market-seal house at Reefhold (Reefhold): truth *Acquire: as told*, twist Wants to be rid of it, ways [trade; earn; deceive].
- **c-d-main-fall-5late** (fallout m-d-main-5 profit checkpoint 2) Reckoning/Blood debt at the lodging-house at Ashring (Ashring): truth *Reckoning: still profiting*, twist none, ways [justice; ruin; amends].
- **c-d-main-fall-6a** (fallout m-d-main-6 resist) Threat/Raiders at the outlying farms of Firststone (Firststone): truth *Threat: driven out*, twist One of us caused it, ways [confront; undermine; bargain].
- **c-d-main-fall-6b** (fallout m-d-main-6 endure) Investigate/Suspicious death at the river wharf at Highcrossing (Highcrossing): truth *Investigate: a bystander*, twist none, ways [examine; question; follow].
- **c-d-main-fall-6late** (fallout m-d-main-6 enemy's own checkpoint 3) Dispute/Shared resource at the old palace guard-yard (Highcrossing): truth *Dispute: one side lies*, twist none, ways [truth; deal; side].

Cast how: {'cast': 31, 'created': 146, 'reused': 49, 'looked up': 7}; reuse rate among reused+created: 25%.

## Rule problems (doc and section)

1. **main-quest §6 (Rising journey) + locations §3:** each layer's chapters must open farther than *near* from the last turn's place and within ~3 days; on map D the Crossing is so tight that layer 5 and the climax had one legal opening each (Oathford ~3.06 days). The two constraints together nearly force geography.
2. **main-quest §6, landmark standing:** community landmarks are 'Middle' exactly, but most villages' best site is Low or High; exact match had no solution, so I read it as a floor.
3. **main-quest §8 (heart's secret):** no odds are given for whether the heart has a secret; I rolled 50%.
4. **family-rescue §6 (Justified captor) vs rulebook (truth reachable):** on Imprisoned unjustly the twist removes the prove ways, which are the only ways that reach a *guilty* truth (c-d-main-5). I kept a crowd way and marked it truth-reaching; the family needs a non-prove way that can surface guilt.
5. **family-rescue (Kidnapped, why taken: as leverage) + assembly §2 (no way harms the heart):** 'give them what they took him for' would hand over the heart, so it is removed (c-d-main-20). The family should say what replaces it.
6. **fallout §3 rule 8 + family-investigate:** a bystander culprit fits Theft and Suspicious death but not Disappearance (needs a link) or Sabotage (needs a rival or grudge), so fallout Investigate has two premises.
7. **rulebook §4 once-per-family vs main-quest (ally planted on a Reckoning card before a Reckoning climax):** the crisis exception covers faction stories only; the general's two Reckoning uses (c-d-main-8, c-d-main-27) are legal only because c-d-main-8 uses him in an authority slot.
8. **world_check vs rulebook (World ledger):** the checker sets caps from the cards it sees (N≈48 here: premise 2, row 3, shape 4), not the planned N=220 (premise 9, row 6, shape 8). I held to the tighter caps.
9. **main-quest §9 (Ordeal is realm) vs door cards:** c-d-main-19 (Estrangement) is personal by nature; I made it realm through the writ's proof (mother and son's quarrel is what the writ cites).
10. **locations §2 (Settlement context) vs layer 0:** the Long Shed court state from layer 0 has no `when`, so it can't sit in a context list; I restated it as x-d-main-ctx-long-shed with `when`.
11. **casting (fallback in the heart's place):** whether the fallback counts as cast when he isn't acting for the heart is unstated; I treated him as cast everywhere (he's in story.cast).
12. **main-quest §5 (move scope):** a move's scope is not said to follow its layer; m2 reaches two regions from community chapters, so I set its scope by reach (region) and counted fallout 2 + late.
13. **No blank card / plain:** whether main non-turn cards may be plain is unstated; c-d-main-13 and c-d-main-14 rolled confirming truths and no twist in the same chapter (both plain).
14. **fallout §3 rule 4 + family-reckoning (Rank, answers-to):** on fallout Ruin from above the one the wrongdoer answers to holds a main card, so the family-rule way can't be offered (c-d-main-fall-4a).
15. **locations §6:** `checkpoint: k` has no numbering; I used 1 for the threshold and trials-3 moves, 2 for trials-4 and the ordeal, 3 for the last move.
16. **world_check reuse (one person, two slots) vs the families:** requester + party is allowed, but requester + estranged party (Restore, Estrangement) or requester + cargo (Passage, Fugitive) is flagged, though the families' natural requester is that person. I moved the requester to someone else on three cards (c-d-main-19, -fall-3a, -fall-4b).
17. **main-quest §9 (Return reuses shown people):** no shown person can be the bridge-keeper's son who comes home in c-d-main-28, so the cargo (and the toll-bridge site and river road) are created in the Return; the requester and the one left behind are shown people.

## Story notes

- The will (m1) and the seal-proofs (c-d-main-2) carry the same new seal: the lead plant, paid at the Long Shed's strongbox (c-d-main-6).
- Layer 3 is the lieutenant's: he holds the commission (c-d-main-9) and hunts the deserter (c-d-main-12); he returns as captor at Crownsgate (c-d-main-26), behind the notary (c-d-main-23) and as the Return's profiteer.
- The revelation is published twice, once per path: by Shahayuh at the Great Bridge (his own grant bears the emperor's true hand) and by the cousin at the Scales Hall.
- The climax rolled 4 ways by overflow: the forced ally and sacrifice ways plus the fault line and revelation ways.
- Minors: the veterans' ditch (Longwall), the tenth tally under Kotqot's seal (Breakwall), the ford-wardens' handshake (Oathford). Advantages land on the climax and on 4a.
- Fallout: 16 cards (m1 1; m2–m6 2 each + 1 late), faces profit/resist/endure/enemy's own/mirror all present (enemy's own: the company quartermaster in the guard-yard dispute); mirror is the Longwall seal-cutter named in the same writ (Dispute, Contested inheritance).
