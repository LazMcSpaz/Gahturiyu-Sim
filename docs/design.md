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

**Purity does not require copying what never changes.** The chronicle and the
run of stats are written once per turn and never touched again, so `advance`
carries them by reference into a fresh array rather than deep-copying six
thousand event objects to add one. A turn at year 150 cost three times a turn at
year 50 before that; it is a copy for every purpose the contract cares about,
and `test/run.js` proves an old state cannot be changed by advancing past it.
Any future system that reaches back and edits a written entry breaks this, and
the test is there to say so.

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
  in that faction, weighted by age (`min(age, 60) / 60`). The household's own
  faded deeds count toward **`quarter` only** — they are what the settlement at
  large holds against a house, which is the neighbours' view and not the
  guard's. Adding them to every faction makes all six read the same number and
  hides the only thing worth seeing.
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

## 3. Buildings grow, and what they grow into

> **Built.** Slice 2 is in the code. Three things changed under contact with a
> run: the second growth takes 70 years rather than 50; a great house cannot be
> the fallback role, or most households end up with one and the word stops
> meaning anything; and one tavern to a quarter with about two common halls in
> the settlement, because the devout years otherwise produce a dozen.
>
> One thing the design gets wrong until slice 5. **Third growths accumulate** —
> a century of play leaves most standing buildings at stage 3, because nothing
> knocks one back. Upkeep and falling back a stage are what will thin them, and
> until that exists, age is a ratchet.

A house is not finished when it is finished. Living stone keeps growing for as
long as it is attended, and what a building **is** changes as it gets older. Age
is the settlement's most honest measure of status, because it cannot be bought
or hurried — only kept.

| stage | reached after | what it is |
|---|---|---|
| **1 — home** | 15–20 yr | a dwelling. Where every building starts |
| **2 — home and workshop** | +30 yr (≈45 total) | a dwelling with room to work a trade or keep a shop |
| **3 — third growth** | +70 yr (≈115 total) | **a great house**, **a tavern**, **a common hall**, or — most often — **a large house** |
| **4 — landmark** | rare, and long after | a named thing. The settlement navigates by it |

### Tending never ends

After stage 1 a building needs a tender's attention **every 5 years** — a
season's work, not years of it. Miss the visits and growth stalls where it
stands; it does not fall back, it simply stops.

This is the change that matters most to the existing design. Tenders are
currently scarce and valuable for twenty years per house and then idle. Now a
settlement of forty households with four tenders has a permanent shortage, and
every stalled building is a household that will never have a workshop. The
sharpest pressure in the simulation gets sharper and never lets go.

### What stage 2 does to the class system

**The finer trades need a workshop, and a workshop is a stage-2 building** —
a forty-five-year-old house. Most trades do not: a roper, a quarrier, a mason
and a tender all work where the work is (§4). The ceiling applies only to the
tier that needs a room.

That is the right shape, because it means the settlement **gains complexity as
it ages** rather than starting complete. Four buildings are old enough at
founding; the rest arrive over centuries.

Where it does apply, it concentrates the fine trades in old households without
anyone designing it that way, and it makes a master's workshop the real reason
apprenticeship is gated — rather than a wealth threshold nobody could see. The
ways under it are the ways that should produce stories: marrying into an old
house, being taken by a master with a workshop and no heir, inheriting from a
line that failed, or a household falling far enough that its workshop passes on.

### Choosing what stage 3 becomes

The household decides, weighted by where it stands when the stone gets there:

- **Great house** — composite standing in the **top quartile**. Adds **+10
  `quarter`** and **+6 `government`** a year while it stands
- **Tavern** — the quarter has none. One to a quarter; a second would have
  nobody new to draw. **+6 `quarter`** a year
- **Common hall** — the shrine well kept, and the settlement holds fewer than
  one hall per two quarters. Belongs to the quarter rather than the household,
  and the household is thanked for it in perpetuity: **+15 `quarter`**, decaying
  at a quarter rate
- **A large house** — everything else, and it is the common case

That last row is the one the first draft got wrong. Reaching a third growth is a
century of somebody's attention, but it does not by itself make a household
eminent. With `great` as the fallback, seventeen of thirty-three buildings were
great houses and the standing they carried meant nothing. Eminence has to come
from being eminent already, from keeping a tavern, or from giving the growth
away — everything else is just a big house, and carries no standing at all.

The role can change once, and only through collapse: a great house whose line
fails may be taken over as a tavern or a common hall rather than fall derelict.
A settlement's most-used building being the wreck of a proud family is the kind
of fact this simulation exists to produce.

### Taverns, revised

This supersedes the earlier answer that a tavern is a room in a standing house.
That was a reasonable fix to a bad framing, and stages are a better one: a
tavern is what a **stage-3 building** becomes, which makes it rare, old, and
worth something.

- Fitting out costs **2 timber + 1 stone** and one household member's work
- Draws anyone within **6 tiles' walk** — a feud that shuts a path cuts its
  reach, and the household notices first
- Earns the household a **cut**, and **+6 `quarter`** a year while it runs
- Attendance rises in hard seasons and after storms
- It is a **relationship context** — see §6. That is its real function

It closes when the household stops running it, which needs no machinery: the
keeper dies and nobody takes it up, the family turns poor and wants the hands
back, or nobody comes any more. The building stays; only the trade in it ends.

### Landmarks

Very rare, and it should stay that way — perhaps once in several centuries.

Requirements: stage 3 held **40 years**, condition above **0.9**, beauty above
**0.7**, and a tender still attending. Then **1% a year**.

That makes carvers matter beyond decoration — beauty is a precondition, not an
ornament.

**A landmark has no mechanical effect.** It grants no standing, changes no
walking, alters nothing. It is named, it is drawn on the map, the chronicle
records the year it was recognised, and it outlives the family that made it.

That is deliberate. A landmark is proof that some household kept one thing
beautiful and in repair for a century while everyone else let theirs slide, and
attaching a bonus to it would turn that into a strategy. It should only ever be
a fact about the place.

### Falling back

Condition below **0.2** knocks a building **back a stage**. A workshop lost that
way takes the trade with it until the stone recovers, which is decades. Disrepair
should be frightening in proportion to what has been invested.

## 4. Trades

> **A learner is worth what they have learned.** Counted flat at a half each,
> three ten-year apprentices in their first year read as a master and a half,
> and one coast with four tenders aged fifty-seven to sixty-seven read as safe
> — then lost all four inside fourteen seasons and had no working tender until
> the apprentices finished alone. A learner counts for a fifth plus half their
> progress. Across twenty-two seeds no settlement now spends a season without
> a tender, and the lowest the stone gets is two holders and learners together.
>
> **Built.** Slice 3 is in the code, and it cost more corrections than the two
> slices before it put together. In order of how badly each was wrong:
>
> - **A settlement cannot be founded blank.** With nobody holding a craft there
>   are no masters, so nothing can ever be taught and every trade stays empty
>   forever. The founding generation carries all eleven.
> - **The gift must count only for the craft that needs it.** A flat aptitude
>   bonus made aptitude-holders the most attractive apprentice for *every*
>   trade, so ropers and masons took exactly the children the stone needed and
>   the settlement lost the ability to build — without one decision ever looking
>   wrong. When tenders are thin, other masters now leave them alone.
> - **A workshop need not be your own.** Tied to the worker's own household, every
>   fine craft died inside forty years: not because masters died, but because
>   households split and marry, and a smith ends up under a roof with no
>   workshop while three stand empty across the quarter. Your own house, your
>   kin's, or one that thinks well enough of you to let you in.
> - **`want` has to cap as well as encourage.** It only pushed upward when a
>   craft was scarce, and since the rate is per master, a crowded craft kept
>   recruiting by having many masters — one settlement reached twenty-five
>   tanners among eighty-four adults. Past its share, a craft now stops taking
>   anyone.
> - **The tender numbers in this document were too harsh.** Four in five failing
>   over ten years is an absorbing state: a settlement that dips to zero tenders
>   can never recover, and every seed reached zero. The code runs 1.2% a season,
>   which is close to what the simulation managed before any of this.

A person has one trade. It is learned young, usually from kin, and it does not
change because the larder filled up. **Shortage never selects a trade.** It only
shapes choices inside one.

### Tier 1 — unskilled: anyone, no training

| trade | produces per worker per season |
|---|---|
| `fieldhand` | 3.4 food (scaled by the tile's grazing) |
| `boathand` | 4.2 food (scaled by the tile's fishing) |
| `quarrier` | 2.0 stone |
| `miner` | 1.5 ore |
| `woodcutter` | 2.2 timber |
| `hauler` | nothing — moves goods, makes a carrier's work possible |

Default for anyone not apprenticed by **18**. Trends to households without
means, because those households need the hands now and cannot spare a child for
five years.

Quarrying sits here deliberately. **Cutting stone is not growing it.** Anyone
can take rock out of a hillside; only a tender can make a hillside into a house,
and the distinction is the whole reason this settlement is shaped as it is.

### Tier 2 — skilled, worked where the work is

A master is required. A workshop is not — these are done on the hillside, at the
quarry, on the water, or at the wall being mended.

| trade | training | produces |
|---|---|---|
| `roper` | 3 yr | 2.5 cordage |
| `tanner` | 4 yr | leather, from the herds' hides |
| `carrier` | 4 yr | nothing — moves goods between households, takes a cut |
| `mason` | 5 yr | repairs (1 stone + 1 timber → +0.25 condition) |
| `stone-tender` | 10 yr | grows houses — unchanged, and still the rarest thing |

### Tier 3 — skilled, and needing a room

Fine work needs somewhere to do it: a **stage-2 building** (§3), roughly a
forty-five-year-old house.

| trade | training | produces |
|---|---|---|
| `weaver` | 4 yr | cloth |
| `tailor` | 4 yr | garments, from a weaver's cloth |
| `herbalist` | 5 yr | remedies — the only answer to illness the settlement has |
| `joiner` | 5 yr | fitted timber — furnishings, fittings, a tavern's fit-out |
| `carver` | 6 yr | beauty (+0.15 to a building per season worked) |
| `smith` | 7 yr | worked metal, from a miner's ore |

**This tier is meant to grow, and to grow late.** A settlement begins with a
handful of workshops and gains more over centuries as its houses age, so trades
that need a room can only appear once there are rooms. Complexity arrives on its
own schedule rather than being present at founding.

### How hard it is to survive

Measured, and it was not hard at all: the median larder sat at seventeen with a
cap of forty-four, eleven households of thirty-one were **at** the cap, the
common store ran at ninety-four per cent full, and five households in a
settlement of a hundred and fifty were ever short.

Worse, **a storm season brought in a hundred and twenty per cent of a normal
one.** Storms cut fishing and nothing else, and this settlement is mostly
pastoral — weather that touches a fifth of the work is not weather.

Three changes, and the numbers they were measured against:

| | was | now |
|---|---|---|
| larder cap | 44 — a third of a year's eating, so everybody filled up | **22** |
| a storm | fishing only | also **0.72×** on grazing |
| relief from the store | three consecutive short seasons | **two** |

And gratitude no longer accumulates without limit. Fed every season through a
crisis, households pushed the settlement's regard for its rulers to thirty and
legitimacy to a hundred and ten *while people were starving*. Being fed once is
gratitude; being fed constantly is dependency, and it now buys almost nothing.

The result is a settlement that can fail. Twenty years of storm, blight and
fever now cost a fifth to a quarter of the population, and on one seed in three
they drove the common store to eight, legitimacy to twenty-two, and **the coin
out of use** — which is the collapse loop of §9c firing on its own rather than
being proved in a test.

### Food, until the economy exists

**This is a placeholder and slice 5 should delete it.** The food model assumes
every pair of hands fishes or herds. That held when there were four kinds of
work; with sixteen it starves the settlement — a run diversified into crafts and
fell from a hundred and thirty adults to fifteen, because a quarrier, a smith
and a tanner all produced nothing anyone could eat.

Until goods and exchange exist, everybody also feeds themselves: a fieldhand or
boathand at full rate, the other unskilled trades at **0.55**, a taught craft at
**0.4**, and one needing a workshop at **0.3**. It is a lie the shape of the
truth. When a smith's metal can be traded for a fieldhand's food, none of it is
needed.

### What these trades imply, and is not yet settled

One gap the list opens, belonging to the economy in slice 5:

- **Four goods is no longer enough.** A smith needs `ore` and makes `metal`; a
  tanner needs hides and makes `leather`; a weaver makes `cloth` and a tailor
  turns it into garments; a herbalist makes `remedies`. The design's four goods —
  food, stone, timber, cordage — will have to become roughly nine, or several
  of these trades produce nothing anyone can use.

Neither blocks apprenticeship, which is slice 3 and only needs the trades to
exist and be learnable.

**Aptitude** stays as it is: rare, innate (13%), cannot be taught in. It gates
`stone-tender` absolutely and gives a bonus elsewhere.

**Dropout: 2% per season**, 4% for stone-tender. Over a five-year training that
is roughly a third who do not finish; over the tender's decade, about four in
five. That matches the world already described.

### A craft the settlement is losing

A town without a smith puts a high price on getting one, and the first build
did not. Counting heads is not counting a craft: **five tenders aged
forty-five to seventy-four look like plenty and are not.** They were seeded
together, stopped teaching because five seemed enough, then died within a
decade of each other — and every half-trained apprentice lost their master
partway through a ten-year training. That is how a settlement loses the ability
to build without anybody making a mistake.

So a holder counts for **as much working life as they have left**. An old smith
is most of a smith today and almost none of one in ten years, and a settlement
that can see that trains a replacement while there is still somebody to do the
training.

When a craft falls below about half what it ought to number here, the
settlement behaves differently:

- the old master teaches anyway — the window widens to seventy-eight
- somebody finds them a room, wherever one stands in the settlement
- they take an apprentice they would otherwise have turned away
- another holder takes over an orphaned apprentice rather than lose the craft
- and a half-taught apprentice past two fifths finishes alone far more often,
  because a poor smith is a great deal better than none

Across nine long runs that took losses from four-to-eight crafts down to none
or one, and the stone — the craft everything else depends on — now holds
between two and four at its worst on eight of nine seeds.

### Losing a trade

A trade with no living master **cannot be taught**. If the last mason dies
before taking an apprentice, the settlement has no mason until a carrier brings
one or a stranger arrives. A trade can also be lost by **losing the room** —
every workshop that held it fallen back a stage — which is slower, sadder, and
harder to reverse. This is not a failure state to be designed around —
it is the sharpest pressure available, and it already works this way for
tenders.

---

## 5. Apprenticeship

A master takes **one apprentice at a time**, and may take another **2 years**
after the last finishes or quits. Masters take apprentices between **25 and 60**.
Candidates are **12 to 17**. A master with no workshop cannot take anyone.

The ways under the forty-five-year ceiling run through here. A master with a
workshop and no heir is the most consequential person in the settlement's class
structure, and the sim should let you watch them decide.

### How a master chooses

Score every candidate; take the highest if it clears **40**, otherwise take
nobody this year.

Measured against a real settlement, the first version of this let through **5%**
of master-and-child pairs and fifteen of the sixteen were the master's own
household — so every craft became kin-only and died with any master who had no
child of the right age. The floor is **40**, not 60, and the walk costs **1.5** a
tile rather than 3, which was swamping every other term. That gives 13% clearing
with two thirds from another house, which is what the refusal cascade needs in
order to have anything to refuse.

```
  50                                   base
+ aptitude × 0.4                       40, but only for a craft that needs it;
                                       −45 for any other while tenders are thin
+ 25  same household
+ 12  same lineage, different household
−  1.5 × walking distance in tiles     (uses the existing walk, so a feud that
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

## 6. Relationships

> **Built.** Slice 1 is in the code. Three numbers moved when it ran, and the
> document now carries what the run taught rather than what was guessed:
> traits average 31, not 50, so the affinity baseline is 68; contact must fall
> on the same faces repeatedly or nothing accumulates; and household deeds
> count toward `quarter` alone, because adding them to all six made every
> faction read the same number.

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
affinity = 68
         + (loyalty_a + loyalty_b − 62) / 3
         − |ambition_a − ambition_b| / 3
         − (grudge_a + grudge_b − 62) / 5

drift    = clamp((affinity − 50) / 10, −5, +5) × context weight
```

Traits are rolled as `pow(r, 2.1) × 100`, so they average about **31**, not 50.
Every term is measured against that, and the base is set so a shade more people
drift together than apart: about **57% up, 37% down**, the rest too slight to
store. The first version assumed a midpoint of 50 and quietly made every
relationship in the settlement worse.

**Sampling is not enough.** Three random pairs out of a quarter of forty is one
chance in two hundred and fifty seasons that any particular pair meets, so
nothing accumulates — while a single quarrel lands sixty points of ill will at
once. Instead **each person sees somebody each season**, and **60% of the time
it is a face they already know**. Relationships have to deepen with the same
people to get anywhere, which is also how they work.

**Generosity has to mint ties the way quarrels do**, or the ledger only ever
runs one way. Taking in a child, feeding a household, speaking for one nobody
else will — each is a positive tie, sized like the negative ones.

**Work crews will matter most here**, because a crew is not chosen at random. It
is the people your trade and your quarter put you beside for thirty years.

### Taverns

Specified in §3, because what a tavern *is* belongs to the building that becomes
one. What matters here is what it **does**: it is the only context that mixes
quarters by choice rather than obligation.

Where the shrine forces the classes together for a year, a tavern lets them
choose each other for a lifetime — or lets a quarter close ranks against
outsiders. Both outcomes are worth having, and a settlement with no stage-3
building has neither.

### When a quarter acts as one

Rarely, and only when two things hold at once:

- **Cohesion** — the mean tie between the quarter's households is above **+25**.
  A quarter divided against itself cannot act.
- **A grievance it shares** — a levy laid on them, feuds cutting their paths,
  their standing sliding against the others.

Both true, and there is an **8% chance a season** that the quarter moves as a
body: refusing a levy together, backing one side of a quarrel as a bloc, or
withholding labour. Once or twice a century, not a standing feature.

**Whether it works is a question of means, not a roll.** A quarter can only
refuse what it can afford to refuse. Withholding labour while your own stores
are empty is a gesture that lasts one season. The test is whether the quarter
can outlast the settlement's need of it — its combined stores against what
refusing costs it, and how badly the rest depend on what it is withholding.

A poor quarter that stops working folds. A quarter holding the only rope on the
coast does not.

**The quarter pays, and the quarter benefits.** The consequence lands on the
whole of it, not on whoever spoke:

- **It works** — the levy withdrawn, the backed side wins, the demand met:
  **+15 `quarter`** to every household in it, and cohesion rises by **10**
- **It fails** — **−20 `government`** to every household in it, the guard may be
  set on the quarter as a whole, and cohesion falls by **15**

Collective action is therefore a gamble taken together, and a quarter that loses
one is a worse place to live for a generation. That is why it should happen
twice a century and not twice a decade.

**Watch this before tuning it.** The interaction between cohesion, means and
grievance is the least predictable thing in the design, and the numbers above
are a starting position to observe rather than a balance to defend.

This is where the unskilled finally have the weight the design promised them.
They never rise one at a time; they count when a whole quarter stops working.

### And a trade can do the same

A trade organises on exactly the same terms, against the `trade` faction instead
of the quarter: cohesion above **+25** among its practitioners, a shared
grievance, **8% a season**. Every roper refusing to supply one household is the
same act as a quarter withholding labour, and the faction already exists to hang
it on.

The same means test decides it, and a trade usually passes where a quarter would
fail: four ropers agreeing costs the settlement its rigging, and the ropers can
eat. That makes it more dangerous than the quarter version — a trade is a
**monopoly** where a quarter is only a neighbourhood.

The same payoffs apply with `trade` in place of `quarter`, and a trade that
loses such a fight tends to lose its apprentices next, because no household will
place a child where the work has become a liability.

This makes trades political actors rather than merely economic ones, which is
cheap given everything else already in place.

## 7. The shrine term

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

## 8. Offices are sought

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

## 9. The economy

Large enough to need its own build order. Four parts, in dependency order: what
is made and stored, how it changes hands, what money is and when it appears, and
what happens when a debt is not paid.

**The chronicle never reports a transaction.** No tallies of goods, no prices
paid. It reports the **pinch** — a refusal, a shortage, a dependency, a debt
called in:

> *The Rilu would not sell rope to the Deḍu this season.*
> *Nobody would take a Deḍu stick this season.*

The ledger runs underneath and surfaces only when it hurts somebody.

### 9a. Goods, larders and the common store

> **Built.** Production, the tithe and the store are in the code. Two things the
> run showed. The store opening had to stop being news every time — reported on
> every occurrence it fired most seasons, three identical lines a season, and
> turned an emergency into routine; it now speaks when many households are
> involved or when it has been quiet a while. And **goods pile up where they are
> made**: a household holding sixty stone and an empty larder, while the smith
> two quarters over has no ore. That is not a fault in 9a, it is 9a working —
> nothing can move until 9b exists, and the settlement says so.

Ten goods, each made by a trade and needed by somebody else:

| good | made by | wanted by |
|---|---|---|
| food | fieldhand, boathand | everyone, every season |
| stone | quarrier | mason, carver |
| ore | miner | smith |
| timber | woodcutter | mason, joiner |
| cordage | roper | boathand, upkeep |
| leather | tanner | tailor, upkeep |
| metal | smith | joiner, mason, and the mint |
| cloth | weaver | tailor |
| garments | tailor | everyone, slowly |
| remedies | herbalist | the sick and the old |

Masons, carvers, carriers and tenders make no good — they do work, and are paid
for it like anyone else.

**Food is stored twice.** A household fills its own larder first. What is left
over is surplus and can be sold. But **the government takes its cut before any
of that** — off the top, before a larder is filled — and that cut goes to a
common store.

The common store does three things and only three: it feeds the offices and the
guard, it is opened in an emergency, and it is the thing a government is judged
on when the emergency comes. A store that is empty when the settlement is
starving should cost more standing than any other single failure in the design.

### 9b. Exchange: barter, credit, and the stick

> **Built.** Goods move, debt accrues, and the chains that made nothing in 9a
> now complete — metal, garments and fittings all went from zero. One
> correction: **a tally being cut is not news.** Reported every time, it filled a
> sixth of the chronicle with bookkeeping, a hundred and fifty lines a century
> all saying the same thing. A stick is only worth a line when the debt behind
> it is a millstone. What survives is the pinch the design asked for — a house
> refused, a stick passed on at a discount, a household nobody will extend to
> any further.

Before money, every exchange is a negotiation between two households, and the
price is a **range**, not a number. Four things set it:

- **What it costs to make** — training years and inputs consumed
- **How scarce it is right now** — what exists against what is wanted
- **How badly the buyer needs it** — a hungry house pays more
- **What the two households are to each other** — standing, and the tie between
  them

That last is the point, and it runs on machinery already built. A grudge costs
you. Standing buys you a better price. In barter the spread is wide — the same
rope can cost two households very different amounts, and the chronicle can say
so.

**Everyday exchange is credit, remembered.** Neighbours do not hand over goods
for goods; they remember. That keeps the ledger quiet and puts trust where it
belongs, on reputation.

**A tally is the official record of a debt** — a split stick, half to each side.
Two things separate it from a remembered debt: it does not fade, and it does not
die with the debtor. An heir inherits a stick they never agreed to.

**Tallies are transferable.** Once a stick can change hands, its worth depends on
whether the settlement believes the debtor will pay — so a tally on a
well-regarded household trades near its face and one on a bad household trades
at a discount. The settlement is putting a number on creditworthiness, using the
standing it already keeps.

Two things follow, both wanted:

- **Creditor households.** Holding other people's paper is power without land or
  craft — the middleman again, in a third form.
- **Cascades.** A household that fails ruins everyone holding its sticks. In a
  settlement of two hundred that is a chain reaction, and the most dramatic
  economic event available.

### 9c. Coin, and the mint

> **Built.** Two of six seeds invent money over a hundred and fifty years; four
> never do. One of those never had a smith at all, which is the design working
> exactly as written — a settlement without the craft cannot have money however
> well it is governed.
>
> It was four of six until 9d landed, and the reason is worth keeping: hunger
> now tells against a government whether or not the store answers it, which
> took six to ten points off every settlement's legitimacy. The thresholds were
> tuned against the old scale and had to come down with it — 72 and 52 became
> **64 and 46**. What matters is the gap between them: wide enough that a coin
> outlives a bad decade, not so wide that it survives a collapse.
>
> Two corrections. The first threshold let coin appear inside forty years on
> every seed and it never once failed, so it was raised until only a genuinely
> well-run settlement reaches it. And **legitimacy was measuring nothing**: it
> read households' `government` standing, but only office-holders earned any,
> so the number was a near-constant made of filled seats and a stocked store.
> The governed now have an opinion — fed from the store, left to starve, kept
> off the water, or the guard set on you all move it.
>
> **The collapse loop does not fire on its own**, and this is honest rather than
> hidden: over a hundred and fifty years, including twenty-five of storm,
> blight and fever, the store never emptied and no more than two households
> were ever short at once. The settlement is simply hard to break, which is a
> balance property older than this slice. The path itself is proved directly in
> `test/run.js` — put a government in the state a bad one would be in and the
> coin stops being taken the next season, and the settlement is told why. What
> is missing is a settlement that can actually fail.

**Currency is an achievement, not a given.** The settlement starts without it,
and it appears only when the government is strong and believed enough to
guarantee a standard. That makes legitimacy mechanically consequential rather
than decorative, and it means a run can show you money being invented.

**The unit never changes; only what moves does.** From the first day there is a
standard measure of grain — the **ḍaqu**. When coin comes, a coin *is* the
weight of metal that buys one ḍaqu, and takes the same name. Common people go on
meaning grain by it while the government means metal.

**The threshold** is legitimacy, read from things that already exist: mean
`government` standing across households, whether levies are obeyed or defied,
and how long the present form has held without collapsing.

**Minting is a contract, one batch at a time.** The government names a smith for
each batch; that smith keeps a portion as payment. Per batch, not for life — so
there is no minting dynasty, and the favour is granted fresh every time. What
the chronicle records is not one contract but a habit:

> *The Rilu have had the mint four batches running.*

This is the most valuable thing a government owns, and it is what makes an
office worth wanting.

**Money can go bad, and the same dial does it.** A government short of coin
orders another batch; too many batches and the coin is believed less; being
believed less is legitimacy falling; and below the threshold the coin stops
being money at all. Prices widen, barter returns, and everyone holding coin
learns it was only ever a promise. The government that abused the money
destroyed it, and no new machinery was needed to say so.

**The mint is hostage to one named person.** Lose the contracted smith and there
are no more batches until another is named. Lose the *craft* and there are none
at all — and smiths are a tier-3 trade that dies on most seeds.

### 9d. Default

> **Built.** The form of government now produces visibly different settlements
> over a century, which is the thing this slice was for. Over four hundred
> seasons from the same seed:
>
> | | forbear | seize | gaol | kill | rescued | too proud |
> |---|---|---|---|---|---|---|
> | **senate** | 303 | 38 | 0 | 0 | 25 | 11 |
> | **tribunal** | 214 | 68 | 46 | 0 | 22 | 14 |
> | **sole ruler** | 290 | 49 | 16 | 1 | 28 | 10 |
>
> Every form mostly lets people off, and the ladder is a cap rather than a
> prediction — a sole ruler who is not cruel behaves like a senate, which is
> the point of picking within the cap by character.
>
> Four numbers moved after the first run. **Forbearance paid +12 government
> standing per member**, so mercy raised legitimacy straight through a famine;
> it is +5. **`creditTight` decayed faster than it accrued** and sat at zero
> all century, so mercy cost nothing; it decays at 0.004 a season against
> +0.22 a forbearance, and a forgiving settlement now reaches 0.99. **A tally
> being cut was reported every time**, which was a hundred and fifty lines of
> bookkeeping a century; only debts well past the threshold surface. And the
> store opening was the same problem — three lines a season, all identical.
>
> The slice also exposed three things that were not about default at all.
> **Feeding people was a legitimacy engine**: famine opened the store, opening
> it paid regard, and twenty years of storm, blight and fever left a government
> of ninety-five survivors better thought of than the same government over a
> calm run of a hundred and twenty. Going hungry now tells against whoever runs
> the settlement whether or not the store answers it, so relief softens the
> blow instead of turning it into a profit. **A great house was chosen once and
> never revisited**, so greats only ever accumulated — nine of the eleven
> standing at year 120 belonged to families that had died out. The word is
> checked against who the household now is, with a gap between rising into it
> and falling out of it so it does not flicker. And **a craft whose room went
> away kept its holder forever**: a carver with nowhere to work stayed a carver
> for forty years, made nothing, and counted as a holder, so the settlement
> read itself as supplied and never trained anyone. Three years without a room
> and the craft is given up.

What happens to a household that cannot pay is set by **the form of government,
which caps how cruel it may be, and the character of whoever holds the seat, who
picks within that cap.** This is what finally makes a sole ruler and a senate
produce visibly different settlements over a century.

The ladder, mildest first:

| | what it does |
|---|---|
| **forbearance** | they pay what they can without losing the home or the means to earn |
| **seizure** | the home is taken and they work off the rest |
| **imprisonment** | a worker is removed from the household |
| **execution** | rare, and it should nearly always be a mistake |

**Seizure is the most violent act in this world**, and it should be written that
way. A home takes fifteen to twenty years to grow and a workshop forty-five.
Taking one is not foreclosure — it is taking a century of somebody's family and
handing it to a creditor, and it cannot be replaced in a lifetime. The
settlement should answer it closer to how it answers a killing than a debt.

**Mercy has to cost something**, or it is a free win and the choice is not a
choice. It costs credit: a government that never enforces dries up lending,
because nobody extends what they will not get back. A merciful settlement is
gentler and poorer, thrown back on barter and favours.

**A neighbour can pay it off.** Someone with a warm tie and the means steps in.
It costs them real stores and leaves the rescued household owing something that
is not money. It is the warmest mechanic in the design.

**And a household can refuse rescue.** A proud house would rather lose the home
than be beholden, and that is one trait check for one of the better stories
available.

## 10. Upkeep, decay and beauty

> **Three findings from the sweep, a year on.** Half the seeds had forty per
> cent of their stone in disrepair. Classifying every unmended house by why
> gave three causes and none was the wear rate. A mason took the top three
> jobs on their list and moved on whether or not the household could pay, so a
> season went on households with nothing while houses with a stone and a
> timber on the step went unmended — twelve a season on one coast; a mason
> now does three jobs, not looks at three. Mason households held whatever they
> happened to hold, so on a coast where none had materials the credit path was
> closed too; a mason's house keeps stone and timber in. And the weight on
> repute closed a loop: disrepair costs quarter standing, standing is repute,
> and a house of low repute was a house no mason would walk to — seven of
> sixteen failing houses a season on a declining coast, and the decline fed
> itself. A mason is paid at the door and cares less than a tender what the
> house is known for. Disrepair fell from ten houses a settlement to one or two
> across twenty-two seeds, with the poor household that cannot pay still
> visible. Barter now exists for materials — the fallback the design promised
> when credit tightens — and only for materials: bartering anything had a
> household hand over six lots of grain for a lot of garments it used to get
> for a debt that faded, and whole coasts starved on finery.
>
> **Built.** The bill arrives every season and the settlement can be seen
> paying it or failing to. A mason mends a house for a stone and a timber; a
> household that cannot find them is carried by the mason and owes for it,
> which is the class mechanic the design wanted — the poor do not live in
> worse houses because a rule says so, they live in houses they are still
> paying for. That debt goes into the same ledger 9d reads, so a house mended
> on credit for thirty years is a house that can be taken.
>
> **The world had no wood in it.** Timber was in the goods list from 9a and a
> joiner needed it to make anything, but no tile ever carried any, so the
> settlement held exactly zero lots of timber for a hundred and twenty years
> and nothing in the simulation noticed until the stone needed mending.
> Copses now sit in about a third of the slopes and moors. The first fix
> over-corrected — a household put every spare hand into the copse and
> thirty-seven of a hundred adults came out as woodcutters holding nine
> hundred lots of timber nobody wanted — so a household works its wood with
> one pair of hands after it has somebody on food.
>
> Four numbers were wrong in the same direction, and all four made the stone
> come down faster than a settlement could raise it:
>
> - **People patch their own roofs**, which the design did not say. Without it
>   a quarter no mason would walk to simply came down: twenty to forty growths
>   lost a century, and three seeds in nine ended with no third growth at all.
> - **A flat rate is not a recovery.** Self-mending at 0.022 a season against
>   wear of 0.02 nets 0.002, so a house knocked under the line by three storms
>   took twenty-five years to climb back over it. Every workshop in the
>   settlement was gone by year twenty-four and the crafts that need a room
>   with them. It is a share of the gap now — the worst of it first.
> - **Poverty and abandonment are different stories.** A growth is lost after
>   two years past mending *and* ten years with nobody touching the stone at
>   all. A household that simply cannot afford its mending lives in a squalid
>   house, which is common; the stone only comes down when a house is given up.
> - **A sound empty house does not stand empty.** A family that died out used
>   to leave its stone to rot for twenty years while the household next door
>   spent fifteen growing a new one. The count of workshops fell from thirty to
>   eleven over a century. A homeless household now takes the empty house
>   before it claims new ground.
>
> Two further things the slice forced. **A settlement now answers what it is
> short of**: masons are wanted in proportion to the stone that needs mending
> rather than to the population, and a household with crag or copse puts a
> hand on it when the settlement has no stone or timber to mend with. Both
> needed ceilings — unchecked, a settlement of seventy-eight people answered
> failing stone with twelve masons, a seventh of its adults doing work that
> makes nothing while the quarry it needed went uncut. And **a settlement is
> founded with a few workshops in it**: stage is drawn from a building's age
> and one seed in ten drew badly enough to found twenty-seven households
> around a single workshop, which meant no smith, no joiner and no herbalist
> for as long as the place stood, because a lost craft is never re-invented.
> A quarter of the founding stone is old stone, and the crafts that need a
> room are seeded before the ones that do not.
>
> Storms earn their keep exactly as written: a storm season takes five quiet
> seasons off every roof in the settlement, and that is what keeps the masons
> in work.

**Condition**, 0–1, starts at 1.0.

- Falls **0.02 per season**, plus **0.10** in a storm season
- A mason repairs **+0.25** for 1 stone + 1 timber
- Below **0.5** — in disrepair: **−5 `quarter`** per year to the household
- Below **0.2** — falls **back a stage** (§3), taking any workshop with it
- A stage-1 building below 0.2 is at risk of dereliction as now

**Beauty**, 0–1, starts at 0.

- A carver adds **+0.15** per season worked
- Decays **0.01 per season**
- Contributes **beauty × 15** to the household's `quarter` standing

This is where storms earn their keep twice over. They already exist; letting
them break things gives the settlement a **renewable supply of small jobs**,
which is exactly what the seven-goal list lacks. Nobody has to invent variety —
the weather produces it.

---

## 11. Quarters

> **Built** (with §12's blocs and the landmark of §3). Four things, and three of
> them had to be measured before they would work at all.
>
> **The levy came first, because without it a bloc has nothing to be angry
> about.** The ruling body compels a top-quartile household when three or more
> are short; a tribunal or a senate has to agree, so a body divided against
> itself takes nothing. Eight to thirty of these a century depending on how
> often the settlement goes hungry, and about a quarter of them refused — by a
> household with the character *and* the standing to survive saying no, which
> is not the same as a roll.
>
> **Cohesion was measured wrong, and it made the whole mechanism dead.** The
> design said the mean tie between a quarter's households; nobody holds a tie
> to more than twelve households, so most pairs in a quarter of eight have
> never had anything to do with each other, and those zeroes dragged the mean
> to nine against a threshold of twenty-five. In sixteen hundred
> quarter-seasons the test passed thirty-one times and **never once while the
> quarter had a grievance**. It is now the mean over the pairs that know each
> other, plus a floor on how many of them do — a neighbourhood is not close
> because everybody knows everybody, it is close when the people who deal with
> each other get on. A quarter's grievance also needed the other two things
> the design names: a feud line drawn across it, and its standing sliding
> against the rest.
>
> Blocs now run three or four a century, quarters and trades alike, and **they
> fold more often than they work** — which is the means test doing what it was
> for. A trade is the more dangerous of the two exactly as written, because
> the settlement's dependence on a roper is not a matter of how many ropers
> there are.
>
> **A new quarter could never have formed.** On a coast twenty-two tiles wide
> with four founding centres, the farthest lived-in house in a century sits
> about five and a half tiles from the nearest of them — so the threshold had
> to come down to four and a bit, which is the far edge of this map and a real
> walk. One settlement in seven gains a quarter, one in seven loses one to
> emptiness and keeps the name on the hillside.
>
> **The landmark needed two hundred years and the right gate.** Requiring
> sound stone and fresh carving in the same season was requiring a state that
> holds for about five seasons after a mason calls: the best-kept house on the
> coast accumulated a hundred qualifying seasons in two hundred and fifty years
> and no settlement ever raised anything. It is sustained care instead — a
> mason inside the last six years, a carver's work still showing, the stone
> never allowed to slide — and two seeds in eight raise one, at years 136 and
> 161. One at a time on a coast, and it is never demoted back to a house,
> though it can still be abandoned and fall in.

### Quarters are fixed, and that is the point

The centres are chosen **once, at founding**, from where the houses stood then,
and never redrawn. Everything afterward is filed under whichever founding centre
is nearest.

Two consequences follow, and both are worth keeping:

- **A quarter can empty.** Households die out, houses go derelict, and the
  centre stays where it was with nothing around it. Ṭìquḍa becomes the name of
  a hillside where people used to live. That is how real place names behave and
  the chronicle should be allowed to say it plainly.
- **Growth in a new direction has no name.** A settlement that expands away from
  all four centres files those houses under a quarter they are nowhere near.

The second is a genuine flaw rather than a feature. The fix is not to redraw the
old quarters — that would erase the names, which are the valuable part — but to
let a **new centre form** when enough standing houses sit far enough from every
existing one. A settlement should be able to gain a quarter and keep its ghosts.

### Trades cluster where proximity makes sense

Not everywhere:

- quarrier, mason, carver — near the crag
- boathand, roper — on the shore
- fieldhand — on the moor
- carrier, hauler — between

> **On clustering:** the ground-worked trades already cluster without being
> told to — a quarrier measures nought tiles from a crag against a settlement
> average of half a one, a fieldhand sits nearer the moor than any house does.
> The taught trades do not, and the roper was the proof: rope is rigging, and
> ropers came out six to eleven tiles from the water on a coast where the
> average house is six. Masters now lean toward an apprentice who already
> lives where the work is. On a coast that supports two or three ropers the
> effect is under the noise, and this is recorded as measured rather than
> claimed.

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

## 12. Motivation, in three layers

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

## 13. What the chronicle prints

Unchanged in principle: one line per thing that happened, facts only, no
connective prose. The new systems must respect it.

**Print:** refusals, shortages, a trade lost, a term refused, an unlikely
apprentice and why, a house falling into disrepair, a quarter's beauty crossing
a threshold, an obligation failed, a levy compelled or defied, a seat sought and
lost, a tie crossing into closeness or enmity, a building reaching a new stage
and what it became, a building falling back one, a quarter or a trade acting as
one and how it went, and — once in a long while — a landmark.

A building reaching stage 3 is among the largest events the settlement can
produce: it is a century of somebody's attention arriving all at once. It should
read like one.

> **Measured, and trimmed.** The first sweep of eleven seeds after slice 7
> showed the rendered chronicle at six to seven lines a season, up from five,
> and one kind — a figure taking up a goal — a fifth of everything printed.
> Four routine lines were the whole of the drift: a goal taken up, a goal
> still being pursued, a quarrel pressed again, and an apprenticeship simply
> starting. None is a threshold crossed. They sit at weight one now, below the
> floor and still in the record; an apprenticeship prints only when it is the
> unlikely one the design names — a master refused first, a craft down to its
> last hands taking who it can, or a child from a house well below the
> master's. Five lines a season across twenty-two seeds, and the chronicle
> reads as what happened rather than what everyone was busy at.

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

"Named figure" stops being a category. **`prominence` is deleted**, not
repurposed — having a goal says everything the field said, and the panel that
lists figures just reads that. Nothing else should be given the job of deciding
who counts.

---

## 14. Build order

Specified whole, built in slices. Each slice must leave the simulation running
and the tests passing.

1. **Standing per faction, and ties.** The substrate: split `standing` by
   faction, then collapse grudges into signed ties and drift them by shared
   context. Household, quarter and the shrine term are contexts enough to start
   — work crews arrive with trades, taverns with stages. Delete `prominence`
   and the cast cap here too; both are small and both are in the way.
2. **Building stages.** Stage 2 and 3, tender check-ins, falling back. Trades
   need workshops to exist in, so this comes before them. It also stands alone:
   even with nothing else, buildings that keep growing change the map and the
   chronicle.
3. **Trades and apprenticeship.** The master's choice, the refusal cascade,
   workshops as the gate, and work crews as the context that matters most.
4. **Offices sought, and compulsion.** Both depend on faction standing and ties
   being in place, and they give each other teeth.
5. **The economy**, which is large enough to need its own order (§9):
   - **5a** goods, larders, the government's cut, the common store. Deletes the
     food placeholder in §4, which is the thing currently holding trades up.
   - **5b** prices as ranges, barter, credit, and the transferable tally.
   - **5c** legitimacy, the threshold, minting by batch contract, tax in coin,
     and money going bad.
   - **5d** default: the ladder, ruthlessness by form and by person, rescue and
     the refusal of it. *Built.*
6. **Upkeep and repairs.** Condition, masons, falling back a stage, and beauty.
   *Built* — see §10.
7. **Quarters, trades organising, landmarks.** The rollups, the blocs, and the
   rarest thing in the design. Beauty moved forward into 6, because a carver
   with nothing to carve was the only trade in the list doing nothing at all.
   *Built* — see §11. The levy of §2 was built with it, because a bloc with
   nothing to be angry about never acts.

Every slice is built. What the build order was for is finished; what follows
is tuning, and the open questions in §15.

One thing the whole of it has not answered: **a settlement thins out over two
centuries.** A hundred and fifty people at founding are a hundred and thirty at
year a hundred and twenty and between thirty and a hundred and twenty at year
two hundred and fifty, and the spread between seeds is wider than any effect
measured here. It predates slice 6 and slice 7 alike and it is the largest
unexamined thing in the simulation.

## 15. Open, not yet decided

- **The coast fills up, and then it empties.** This is the two-century decline,
  diagnosed. A new house needs a site at least 2.15 tiles from every standing
  one, and on a coast twenty-two by fifteen that is thirty-five to forty
  houses. Once they stand, every household that splits off waits for ground
  that never comes and *leaves the coast* — eleven households in the last
  seventy years of one run, and a settlement of a hundred and seventy-five
  down to forty-five. Nothing in the simulation is broken; the map is full.
  Whether the answer is closer building, lodging with kin instead of leaving,
  a second coast to go to, or simply a bigger map is a design decision and not
  a tuning one.

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
