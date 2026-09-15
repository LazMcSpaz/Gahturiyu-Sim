# Gahturiyu — design

What the settlement is meant to become, written down so the implementation has
something to disagree with. This is not a record of what the code does today;
most of it is not built. Where it contradicts the code, the code is behind.

Numbers here are **starting values**, not conclusions. They are chosen to sit on
the scales the engine already uses (traits 0–99, four turns to a year, deeds
fading over sixty-five years) so they can be dropped in and watched. Expect to
move most of them once a hundred-year run has been read.

## The constraint everything obeys

`advance(state, input) -> new state`, pure given the seed. Every system below
reads and writes only `state` and takes its randomness only from the `r` it is
handed. Rollback, branching and export all depend on this, and they break
quietly rather than loudly. Nothing here is permitted to keep state outside the
state object — no caches, no module-level counters.

---

## 1. The problem this design solves

The settlement currently runs on one number. Every household has a `standing`,
every decision consults it, and so every decision agrees with every other. A
household is liked or it is not.

Three consequences, all visible in a run:

- **Nobody chooses anything.** Figures are assigned goals by a fitness score.
  They are picked for a job; they do not pick.
- **The verbs run out.** Seven goals exist, and only eight people at a time are
  allowed to hold one. Raising that limit would just produce more people doing
  the same seven things, so the limit goes and the verbs multiply instead.
- **There is no ladder and no floor.** Nothing accumulates except reputation,
  and nothing pushes back on it.

The design replaces the single number with **standing held separately by each
faction**, gives people **trades** they keep for life, and makes motivation
operate on *who you work for* rather than *what you do*.

---

## 2. Standing, held per faction

A person holds a standing with each faction, on **−100 to +100**, starting at
**0**.

| faction | who it is | what earns it |
|---|---|---|
| `kin` | your own and allied households | feeding people, fostering, loyalty in a quarrel |
| `quarter` | the neighbours you live among | repairs, upkeep, beauty, settling small things |
| `government` | whoever holds the offices | service, deference, useful rulings |
| `guard` | the captain and the guard | enforcement, courage, standing a watch |
| `shrine` | the keeper and the devout | terms served, offerings, keeping the stone |
| `trade` | the peers of your craft | finishing work, taking apprentices, not undercutting |

Factions are a list, not a fixed set. Anything that later becomes a faction
(a cult, a compact of carriers, a faction around a claimant) gets a row without
schema changes.

**Decay.** Standing moves toward 0 at **1.5% per season** when nothing
reinforces it. Half-life about **46 seasons — 11½ years**. Deliberately faster
than household memory (65 years): regard is current, reputation is history.

**Use the relevant one, not a total.** This is the point of the whole change and
the easiest thing to undo by accident. An arbiter weighs your `government`
standing. A neighbour deciding whether to help weighs `quarter`. A master
choosing an apprentice weighs `trade`. A composite exists **for display only** —
the mean across factions — and no decision may read it.

The stories live in the disagreement: *the richest house on the coast, and
nobody in the guard will speak to them.*

### Flowing up

- **Household** standing in a faction = the mean of its living adults' standing
  in that faction, weighted by age (`min(age, 60) / 60`), plus the household's
  own faded deeds (the existing memory system, unchanged).
- **Quarter** standing = mean of its households' composite × **0.6**, plus
  quarter beauty × **25**, plus quarter condition × **15**.

### What pushes back

Accumulation without a brake ends the simulation at year forty. Three brakes,
in order of how much work they do:

1. **Obligation, and compulsion behind it.** A household in the **top quartile**
   of composite standing is expected to give in a season when three or more
   households are short. Failing costs **−12 `quarter`** and **−12 `kin`**.

   Shame is not the end of it. **The ruling body can compel.** When the shame
   has not worked, whoever holds power — the ruler alone, or the tribunal or
   senate by vote — may levy **25% of stores above 10** from a top-quartile
   household.

   | | |
   |---|---|
   | complying | −stores, **+6 `government`**, **+4 `quarter`**, and a tie toward the compeller worsened by **30** |
   | refusing | **−25 `government`**, **−15 `quarter`**, a dispute opened with the ruling household, and the guard may be set on them |

   Compulsion is deliberately unpleasant from both ends. It fills stores and it
   makes enemies, and the office holder who orders it pays for it later. That
   cost is the point: it is what makes an office worth *having*, which is what
   the next section depends on.
2. **Decay.** The 1.5% above. Standing not renewed drains.
3. **Resentment.** Households in the bottom quartile accrue **+3 grudge heat per
   year** toward the single highest-standing household. Slow, and it makes the
   top house the natural target of any quarrel that needs a villain.

---

## 3. Trades

A person has one trade. It is learned young, usually from kin, and it does not
change because the larder filled up. **Shortage never selects a trade.** It only
shapes choices inside one.

### Unskilled — anyone, no training

| trade | produces per worker per season |
|---|---|
| `fieldhand` | 3.4 food (scaled by the tile's grazing) |
| `boathand` | 4.2 food (scaled by the tile's fishing) |
| `hauler` | nothing — moves goods, makes a carrier's work possible |

Default for anyone not apprenticed by **18**. Trends to households without
means, because those households need the hands now and cannot spare a child for
five years.

### Skilled — requires a master

| trade | training | produces |
|---|---|---|
| `roper` | 3 yr | 2.5 cordage |
| `quarrier` | 4 yr | 2.0 stone |
| `carrier` | 4 yr | nothing — moves goods between households, takes a cut |
| `mason` | 5 yr | repairs (1 stone + 1 timber → +0.25 condition) |
| `carver` | 6 yr | beauty (+0.15 to a building per season worked) |
| `stone-tender` | 10 yr | grows houses — unchanged, and still the rarest thing |

**Aptitude** stays as it is: rare, innate (13%), cannot be taught in. It gates
`stone-tender` absolutely and gives a bonus elsewhere.

**Dropout: 2% per season**, 4% for stone-tender. Over a five-year training that
is roughly a third who do not finish; over the tender's decade, about four in
five. That matches the world already described.

### Losing a trade

A trade with no living master **cannot be taught**. If the last mason dies
before taking an apprentice, the settlement has no mason until a carrier brings
one or a stranger arrives. This is not a failure state to be designed around —
it is the sharpest pressure available, and it already works this way for
tenders.

---

## 4. Apprenticeship

A master takes **one apprentice at a time**, and may take another **2 years**
after the last finishes or quits. Masters take apprentices between **25 and 60**.
Candidates are **12 to 17**.

### How a master chooses

Score every candidate; take the highest if it clears **60**, otherwise take
nobody this year.

```
  50                                   base
+ aptitude × 0.4                       0 or 40
+ 25  same household
+ 12  same lineage, different household
−  3 × walking distance in tiles       (uses the existing walk, so a feud that
                                        shuts a path really does cost you)
+ W × candidate household's `trade` standing
−  0.8 × any grudge the master holds against that household
−  0.6 × any grudge that household's head holds against the master
+  0.5 × the master's tie to that household  (negative ties subtract)
```

**`W` is the master's own weighting of standing, and its sign is their
character.** This is the whole mechanism for an unlikely apprentice:

| master | W |
|---|---|
| ordinary | **+0.35** — take the well-connected child |
| compassionate (`loyalty > 70` and `avarice < 40`) | **−0.35** — take the one who needs it |
| proud or rebellious (`ambition > 70` and `loyalty < 40`) | **−0.50**, applied only to households above +30 — will not be seen serving the big house |

### Refusal cascades

Better than randomness, because it produces a **reason the chronicle can print**.
If either side's tie to the other is below **−30**, the offer fails and the
master moves to the next candidate. Each refusal is recorded.

> *Nane took the Doḍeʻo boy, having been refused by three houses first.*

Between cascades, a master taking roughly one apprentice a decade, and a small
candidate pool, unlikely outcomes arrive on their own. **Prefer mechanisms that
produce a reason over mechanisms that produce a roll.** This is the governing
principle for the whole design.

---

## 5. Relationships

### One signed tie, not two lists

Grudges and bonds are the same thing at different signs. The engine already
stores a grudge as *this person, toward that party, this strongly, for this
reason, since this year*. A **tie** is that structure with the value allowed to
go either way:

- **−100 to +100.** Zero means no tie, and is not stored.
- Above **+40** is close; below **−40** is an enemy. Crossing either threshold
  is worth a chronicle line. The middle is not.
- Ties point at a **person or a household**. On death, a tie to a person passes
  to their household at **half strength**, which is how grudges already
  descend to heirs.
- Decay **1% per season** toward zero — half-life about **17 years**. A tie
  nobody refreshes fades.

Everywhere the simulation asks "do I hold something against them?", it asks one
question of one number, and a friendship is an answer to it.

**Kept sparse.** Two hundred and fifty people is thirty thousand possible pairs.
Only ties that exist are stored, at most **12 per person** — the weakest drops
when a thirteenth forms. An export stays a few kilobytes, which rollback,
branching and the whole save format depend on.

### Contact is what moves them

Relationships do not change because a system decided to change them. They change
because people **spend time in the same place**. Each season, every shared
context samples **up to 3 pairs** and drifts them.

| context | weight | who it mixes |
|---|---|---|
| same household | ×1.5 | kin, constantly |
| same work crew | ×1.0 | a boat's crew, a quarry gang, masons on one repair |
| the shrine term | ×2.0 | **across class** — compulsory, and the only one that is |
| a tavern | ×1.2 | across quarters — voluntary, and may self-segregate |
| same quarter | ×0.4 | neighbours, weakly |

Drift per sampled pair, before the context weight:

```
affinity = 50
         + (loyalty_a + loyalty_b) / 4
         − |ambition_a − ambition_b| / 3
         − (grudge_a + grudge_b) / 6

drift    = clamp((affinity − 50) / 10, −5, +5)
```

So two loyal people working the same boat drift together; two ambitious ones
grind on each other. Nothing needs to be scripted — put people in a room often
enough and the room does it.

**Work crews matter most here**, because a crew is not chosen at random. It is
the people your trade and your quarter put you beside for thirty years.

### Taverns

A tavern is a **room given over in a house that is already standing** — not a
building of its own. That distinction matters here more than it would anywhere
else: the map holds about thirty house sites, a home takes fifteen to twenty
years to grow, and a tavern that consumed one would never be worth building.
A room and a pair of hands, it is plausible.

- **2 timber + 1 stone** to fit out, and one household member's work from then on
- Draws anyone within **6 tiles' walk** — a feud that shuts a path cuts its
  reach, and the household notices before anyone else does
- Earns the household a **cut** of what passes through, and **+6 `quarter`** a
  year while it runs
- Attendance rises in hard seasons and after storms

It closes when the household stops running it, and that needs no machinery of
its own: the keeper dies and nobody takes it up, the family turns poor and wants
the hands back, or the walk got long enough that nobody comes. The room reverts
to being a room.

Where the shrine forces the classes together for a year, a tavern lets them
choose each other for a lifetime — or lets a quarter close ranks against
outsiders. Both outcomes are worth having.

### When a quarter acts as one

Rarely, and only when two things hold at once:

- **Cohesion** — the mean tie between the quarter's households is above **+25**.
  A quarter divided against itself cannot act.
- **A grievance it shares** — a levy laid on them, feuds cutting their paths,
  their standing sliding against the others.

Both true, and there is an **8% chance a season** that the quarter moves as a
body: refusing a levy together, backing one side of a quarrel as a bloc, or
withholding labour. Once or twice a century, not a standing feature.

This is where the unskilled finally have the weight the design promised them.
They never rise one at a time; they count when a whole quarter stops working.

## 6. The shrine term

The term is one context among several now, not a separate machine. What keeps
it distinctive is that it is **compulsory and it crosses class**, which nothing
else in the settlement does.

### Serving

Everyone serves **one year — 4 seasons — between 16 and 20**. Unpaid.

- **+8 `shrine`**, **+3 `quarter`**
- Ties drift at the strongest weight in the table above
- Roughly **6%** stay on afterward and become candidates for keeper

### Refusing

A person refuses when `piety < 30` **and** `ambition > 70` — about **8%**.
Refusal is permitted and expensive:

- **−25 `shrine`**, decaying at **a quarter the normal rate** — it follows you
- **−10 `quarter`**
- A household deed at **−6**, fading over the usual sixty-five years

A refusal should be among the most-cited facts about a household forty years
later.

### Why this is the leveller

Everywhere else, people mix with their own. Service is the only institution
that puts a rich child and a poor child in the same year of work. Four decades
on, the rich one is the arbiter, ruling on a household he served beside.

It wires into the devotion number that already exists. Kept, the shrine mints
cross-class ties every year and people move between stations. Neglected, terms
lapse, no ties are minted, and the settlement hardens into classes — on top of
quarrels no longer ending in judgement.

## 7. Offices are sought

Offices are the real chokepoints — the boat-holder decides who fishes, the
captain decides who the guard leans on, the ruling body can now compel. Nobody
currently tries to get one; they are filled automatically. That is the largest
missed opportunity in the simulation.

### Standing for a seat

A person puts themselves forward when `ambition > 55` and their standing with
the relevant faction is above **20**. Campaigning is a goal a figure pursues
over years, building standing before the seat falls vacant — which adds a verb
the design badly needs, and one that ordinary ambition produces on its own.

### How the seat is filled, by government form

- **Sole ruler** — appoints. Weighting: kin **+40**, tie to the ruler **× 0.5**,
  candidate's `government` standing **× 0.3**, minus the threat they represent
  (`ambition × 0.4`). A ruler appoints their own, and avoids appointing anyone
  who might replace them.
- **Tribunal, senate** — a vote. Each household casts weight equal to its
  composite standing, for the candidate it has the best tie to, then the
  highest standing, then kin.

Senate seats run **6 years** and may be contested again. The ruler and tribunal
hold for life, as now.

### Losing

A losing candidate takes a tie toward the winner worsened by **25 + ambition/4**.
Elections that recur every six years, in a settlement of forty households, will
therefore accumulate history — which is the point.

## 8. The economy, kept under the floor

Four goods: **food**, **stone**, **timber**, **cordage**.

Trades produce what others cannot make, so exchange is forced rather than
designed. A household short of cordage must deal with a roper, or with the
carrier who supplies one.

**The chronicle never reports a transaction.** No tallies, no "3 timber moved".
It reports the **pinch** — a refusal, a shortage, a dependency, a threshold
crossed:

> *The Rilu would not sell rope to the Deḍu this season.*

The ledger runs underneath and surfaces only when it hurts someone.

### The carrier

The middleman is the strongest lever a household has for accumulating power,
because it is the same shape as the mechanic that already works best: a scarce
capability choosing who benefits. A tender decides which households get to
exist. A carrier decides who gets cordage first, and who waits.

A carrier takes a **10% cut** of what passes through them and chooses order of
supply by the same weighting as everything else — need, standing, kin, ties.

---

## 9. Upkeep, decay and beauty

**Condition**, 0–1, starts at 1.0.

- Falls **0.02 per season**, plus **0.10** in a storm season
- A mason repairs **+0.25** for 1 stone + 1 timber
- Below **0.5** — in disrepair: **−5 `quarter`** per year to the household
- Below **0.2** — at risk of dereliction

**Beauty**, 0–1, starts at 0.

- A carver adds **+0.15** per season worked
- Decays **0.01 per season**
- Contributes **beauty × 15** to the household's `quarter` standing

This is where storms earn their keep twice over. They already exist; letting
them break things gives the settlement a **renewable supply of small jobs**,
which is exactly what the seven-goal list lacks. Nobody has to invent variety —
the weather produces it.

---

## 10. Quarters

Trades cluster where proximity makes sense, not everywhere:

- quarrier, mason, carver — near the crag
- boathand, roper — on the shore
- fieldhand — on the moor
- carrier, hauler — between

Masters **may** take apprentices from outside their quarter, and often do.
That is the dial that decides how rigid the settlement becomes: freely, and
clustering stays occupational; rarely, and quarters calcify into castes. It
starts free, with distance the only friction.

The payoff is that geography bites properly. A feud that shuts a path now
threatens a **supply chain**, not merely a walk. A storm on the shore takes
fishing and cordage in the same season. Quarters acquire class character on
their own, and quarter rivalry becomes class conflict without being written as
such.

---

## 11. Motivation, in three layers

The earlier model — people act on what they are short of — is wrong at the top
layer, and produces nonsense: a farmer who stops farming because the larder is
full.

| layer | timescale | what drives it |
|---|---|---|
| **Trade** | a life | inherited, then fixed. Changes only under force: the trade dies, or nobody will buy |
| **Household strategy** | generations | *we will put the second son with the carver.* Where ambition lives |
| **Daily choice** | a season | **shortage acts only here** |

**Motivation does not pick your job. It picks who you do it for.**

A mason does not stop being a mason; he chooses whose wall to mend. A carrier
chooses who waits. That is already the best mechanic in the simulation — a
tender never stops tending, but chooses which household's stone to work, and
that single choice decides which households exist at all.

Every trade should be that shape: **a scarce skill choosing who benefits**,
weighted by need, the walk, kinship, standing with the relevant faction, and
ties.

### The unskilled have numbers, not a ladder

If unskilled labour is only a hole people fall into, it is scenery. It gets the
one thing it actually has: **weight in aggregate**. The unskilled do not rise
individually — they act together, refusing work or swinging behind a side in a
quarrel. That gives the poor real force in the settlement without a ladder that
would contradict the class system.

---

## 12. What the chronicle prints

Unchanged in principle: one line per thing that happened, facts only, no
connective prose. The new systems must respect it.

**Print:** refusals, shortages, a trade lost, a term refused, an unlikely
apprentice and why, a house falling into disrepair, a quarter's beauty crossing
a threshold, an obligation failed, a levy compelled or defied, a seat sought and
lost, a tie crossing into closeness or enmity.

**Do not print:** transactions, production, standing changes, condition ticks,
every drift of every tie — anything that happens every season to everyone. A
relationship is news when it crosses a threshold, not while it moves.

The test that already guards this — `renderTurn` may emit only event lines and
the one "nothing recorded" line — stays. Add: nothing may print a bare number
without saying what it means.

**There is no cap on how many people may matter.** The eight-figure limit goes.
It was the wrong dial: it limited how many people the simulation would let be
interesting, when the real problem was how much of it reaches the page. That
belongs to **event weight**, which already exists and which the chronicle
already filters at 2 or above. Volume is controlled by what is worth printing,
not by refusing to simulate the ninth person.

Expect "named figure" to stop being a category. Anyone currently pursuing
something is one, and the panel that lists them just reads that.

---

## 13. Build order

Specified whole, built in slices. Each slice must leave the simulation running
and the tests passing.

1. **Standing per faction, and ties.** The substrate: split `standing` by
   faction, then collapse grudges into signed ties and drift them by shared
   context. Household, quarter and the shrine term are contexts enough to start
   — work crews arrive with trades, taverns with the economy. Needs nothing
   from either.
2. **Trades and apprenticeship.** The master's choice, the refusal cascade, and
   work crews as the context that matters most.
3. **Offices sought, and compulsion.** Both depend on faction standing and ties
   being in place, and they give each other teeth.
4. **Economy and upkeep.** Four goods, repairs, the carrier, taverns.
5. **Beauty, quarters and class.** The rollups and what falls out of them.

Slice 1 is the next thing built.

## 14. Open, not yet decided

- **Does anything replace `prominence`?** With the cap gone, the field that
  marks someone as a named figure may be redundant — having a goal is the same
  statement. Worth deleting rather than repurposing, if nothing needs it.
- **What does a quarter acting as one cost the people in it?** A bloc that
  refuses a levy is defying the ruling body collectively. Whether that lands on
  the quarter, on its households individually, or on whoever spoke for it
  decides how often anyone dares.
- **Can a trade organise the way a quarter can?** Every roper refusing to supply
  one household is the same shape as a quarter withholding labour, and the
  `trade` faction already exists to hang it on.
