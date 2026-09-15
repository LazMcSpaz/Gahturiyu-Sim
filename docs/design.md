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
- **The verbs run out.** Seven goals exist. Raising the cast size just produces
  more people doing the same seven things.
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

1. **Obligation.** A household in the **top quartile** of composite standing is
   expected to give in a season when three or more households are short.
   Failing costs **−12 `quarter`** and **−12 `kin`**. This is the main brake and
   it is deliberately harsh: being on top is a job.
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
−  0.8 × grudge the master holds against that household
−  0.6 × grudge that household's head holds against the master
+  0.5 × bond between the master and that household
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
If either side holds a grudge above **30**, the offer fails and the master moves
to the next candidate. Each refusal is recorded.

> *Nane took the Doḍeʻo boy, having been refused by three houses first.*

Between cascades, a master taking roughly one apprentice a decade, and a small
candidate pool, unlikely outcomes arrive on their own. **Prefer mechanisms that
produce a reason over mechanisms that produce a roll.** This is the governing
principle for the whole design.

---

## 5. The shrine: service, refusal, and bonds

### The term

Everyone serves **one year — 4 seasons — between 16 and 20**. Unpaid. It is the
one ladder wealth does not gate, and it is therefore the settlement's only
social leveller.

- Serving: **+8 `shrine`**, **+3 `quarter`**
- Roughly **6%** of servers stay on afterward, and become candidates for keeper

### Refusing

A person refuses when `piety < 30` **and** `ambition > 70` — about **8%** of
people. Refusal is permitted and expensive:

- **−25 `shrine`**, decaying at **a quarter the normal rate** — it follows you
- **−10 `quarter`**
- A household deed entry at **−6**, fading over the usual sixty-five years

A refusal should be one of the most-cited facts about a household forty years
later. That is the intent.

### Bonds — a grudge with the sign flipped

The engine already stores a grudge as *this person, against that household, this
strongly, for this reason, since this year.* A **bond** is the identical
structure with the sign reversed, and it reuses the same machinery.

During a term, each pair of co-servers forms a bond with probability **25%**,
strength **20–60**. Decay **1% per season** (half-life ~17 years). When the
other person dies, the bond passes to their household at **half strength** —
mirroring how grudges already pass to heirs.

Bonds are read wherever grudges are read: arbitration, fostering, boat lending,
marriage, apprentice choice, relief.

**Why this is the leveller.** Everywhere else in the settlement, people mix with
their own. Service is the only institution that throws a rich child and a poor
child together for a year. Forty years on, the rich one is the arbiter, ruling
on a household he served beside.

And it wires straight into the devotion number that already exists: when the
shrine is kept, the settlement has a leveller and people move between stations.
When it is neglected, terms lapse, no bonds are minted, and the place hardens
into classes — on top of quarrels no longer ending in judgement.

---

## 6. The economy, kept under the floor

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
supply by the same weighting as everything else — need, standing, kin, grudges,
bonds.

---

## 7. Upkeep, decay and beauty

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

## 8. Quarters

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

## 9. Motivation, in three layers

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
weighted by need, the walk, kinship, standing with the relevant faction,
grudges, and bonds.

### The unskilled have numbers, not a ladder

If unskilled labour is only a hole people fall into, it is scenery. It gets the
one thing it actually has: **weight in aggregate**. The unskilled do not rise
individually — they act together, refusing work or swinging behind a side in a
quarrel. That gives the poor real force in the settlement without a ladder that
would contradict the class system.

---

## 10. What the chronicle prints

Unchanged in principle: one line per thing that happened, facts only, no
connective prose. The new systems must respect it.

**Print:** refusals, shortages, a trade lost, a term refused, an unlikely
apprentice and why, a house falling into disrepair, a quarter's beauty crossing
a threshold, an obligation failed.

**Do not print:** transactions, production, standing changes, condition ticks,
anything that happens every season to everyone.

The test that already guards this — `renderTurn` may emit only event lines and
the one "nothing recorded" line — stays. Add: nothing may print a bare number
without saying what it means.

---

## 11. Build order

Specified whole, built in slices. Each slice must leave the simulation running
and the tests passing.

1. **Standing per faction, and bonds.** The substrate, then terms of service,
   refusal, and bonds. Bonds reuse the grudge machinery, so this is cheaper than
   it looks, and it needs nothing from trades.
2. **Trades and apprenticeship.** The master's choice and the refusal cascade.
3. **Economy and upkeep.** Four goods, repairs, the carrier.
4. **Beauty, quarters and class.** The rollups and what falls out of them.

Slice 1 is the next thing built.

## 12. Open, not yet decided

- Whether a household can be **compelled** to give when obligation calls, or
  only shamed for refusing. Currently only shamed.
- Whether **offices** should be sought deliberately. Nobody currently tries to
  gain one, which is strange given they are the real chokepoints.
- Whether the **cast cap** survives at all once ordinary people have trades and
  choices. It may become unnecessary rather than merely larger.
