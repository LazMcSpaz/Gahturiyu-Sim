# The Main Quest — Locked (v3.6)

This builds on `quest-assembly.md` (diamond, cast, roles, plants) and on the generator's faction spec. **This doc extends and overrides assembly's tables for main stories.** Where they differ, this doc wins.

## 1. Goal
- **Always bigger than any side quest:**
  - a wider scope
  - a stake that side quests may never touch
  - an antagonist who acts between chapters
  - a world visibly changed by the end
- **Many parts:** one playthrough path is **15–25 quests**.

## 2. The four big ideas
1. **Chapters.** Each diamond node is a **chapter**: an episode of 2–4 cards built around a single problem.
2. **The antagonist moves.** Between layers, the antagonist's side *does something*, and each chapter starts in the world that move made.
3. **Scale arcs.** Each main story rolls one direction for its scope: **up**, **out**, **in**, or **back**.
4. **Set pieces and factions.** Sieges, successions and embassies exist only in main stories. Schism also appears as a faction story's crisis (`quest-faction-stories.md`).

## 3. Chapters

### Shape
- **Opening card:** states the **chapter problem**, which is how this layer's *move* hits this chapter's place and people.
- **Middle card(s):** each follows from the card before it (*chain and climb*, below). Different doors to the same problem is no longer enough.
- **Turn card:** addresses the problem and carries the node's **role job**.
  - It also carries the layer's **shared plants**, including the revelation's knower.

### Chain and climb
Carried over from faction-stories, *Chain and climb*.
- **Chain:** each card after the opening depends on a plant from the card just before it in the chapter. The chapter reads as *therefore* / *but*, not as side-by-side errands.
- **Climb:** every card records `pressure_level` 1–3. It never falls within a chapter, and the turn card sits above the opening card.
- The chapter payoff rule (below) carries this.

### Default lengths
| Role | Hook | Threshold | Trials | Ordeal | Climax | Return |
|---|---|---|---|---|---|---|
| Cards | 2 | 2 | 3 | 3 | 3 | 2 |

**Path length:**

| L | 5 | 6 | 7 | 8 |
|---|---|---|---|---|
| Quests | 15 | 18 | 21 | 24 |

- **Default:** L=6, W=3.
- **Range:** L 5–8. Exceptions: **Deep** runs L 5–7 (it adds 2 cards), and **Ripple** runs L 6–8 (§6).

### Rules
- **Order, not progress.** Card *j* assumes the earlier cards in its chapter were reached. It never assumes how they ended.
- **Chapter payoff.** Every card after the opening depends on a fact **declared as a plant** by the card just before it (*Chain and climb*); it may also depend on plants from any earlier card in the chapter. Knowledge only.
- **Every card may depend on its chapter's problem fact.**
- **Shared plants** usually sit on the turn card. A fact declared by any card in a chapter counts as planted by that chapter.
- **Revelation needs a fitting card.** The revelation sits only on a card whose family can carry a discovery: Investigate or Reckoning. A card of any other family (a Threat turn card, say) may carry it only if it rolls a way that reaches the revelation through its knower (question the kennel-keeper, read the second books). If the turn card can't carry it, put it on a fitting card earlier in the chapter.
- **Plant-backed ways roll first** on turn cards and climax cards (see rulebook, *Plant-backed ways roll first*).
- **Turning cards are never plain** (rulebook, *No blank card*). Chapter turn cards, the Ordeal, the climax and the Return each carry a non-confirming truth or a twist, or both.
- **Their twists are rolled first** (rulebook, *Roll order*): the climax, the Return, the Ordeal, then chapter turn cards, then the rest. That is the order twists are rolled, not the order cards are written (§10).
- **Moves count as shared facts.** Any card may depend on its own layer's move or an earlier layer's move. Moves don't count toward the document-plant limit.
- **Family limits:**
  - Assembly's role and opposing-slot tables govern **turn cards only**, and **§8–§9 of this doc extend those tables**.
  - Opening and middle cards may use any family or premise whose requirements are met.
- **Variety:**
  - No two cards in a chapter share a family.
  - A chapter's opening card doesn't share a family with the turn card of any chapter linked into it.
  - Assembly's diamond variety rules apply to turn cards.
- **Cast:** each cast member fills **at most one slot per chapter**.
- **Ordering clause:** a state set on any card binds later cards in that chapter, all later layers, and the Return.
- **No secret leaks.** A move or a plant never states an earlier card's **hidden truth**. Only the revelation may publish a secret (the hook's intent, and the antagonist faction's hidden side).
- **Minor quests** attach to a chapter, never to the Return, and sit near it (§7, *Minors sit near their node*).

## 4. Scope
- **Levels:**
  - **household:** one family
  - **community:** one settlement or one group (a guild, a crew)
  - **region:** several settlements, or one lordship
  - **realm:** the whole land, a Top seat, or a realm-wide road or law
- **Chapter scope is at most the move's scope.** The arc sets each chapter's scope. Usually the two are equal, but a move can reach further than any one chapter (for example, one Ripple scheme striking three communities is region scope, while each chapter stays community). The move must earn its scope: a road is realm scope only if it's a realm road.
- **Card scope ≤ chapter scope.** Every card records its own.
- **Only the opening card** has to reach the chapter's scope.
- **Landmark standing by scope:**

| Scope | Landmark |
|---|---|
| household | the heart's home |
| community | Middle-standing place |
| region | High-standing place |
| realm | Top-standing place |

- **The standing is the settlement's, not the site's.** A community chapter's landmark can be a yard or a shed, as long as its town is Middle standing. (Household: the heart's home itself.)
- **Side quests:** capped at community scope, and they may never use an entry tagged **foundational**.

## 5. The antagonist moves
- **Every layer has one move:** a public past-event fact at the move's own scope (at least its chapters' scope), describing what the antagonist's side has done. Sibling chapters share it.
  - **Hook:** the move is the **harm**.
  - **Ordeal:** the move is the **ordeal event**.
  - **Climax:** the move is the **last move**, a *finished* act (taken, sacked, seized, closed). Never an act still in progress.
- **Each move names a `lever`** from faction-stories' list (debt, law, rumour, theft, force, bribery, blackmail, kin, faith, fire). The story's lever is the last move's lever. The rulebook's *World ledger* (*Reserved levers*) reserves it **and every lever the story uses on 2 or more moves**: other stories may use each at most once, and a side premise or ordeal event built on the same mechanism counts as a use.
- **The villain's want survives the last move.** The last move may not satisfy the antagonist's own want (it can't pay off the very debt that drives them). It raises what the climax decides.
- **Costs are paid.** When the revelation names a cost (someone taken in bond, someone struck off), a later card or move pays it.
- **Moves are past events,** so no card undoes them. The **Return's chapter problem is the aftermath of the last move**, and the Return is set in the world the moves made. Its cards never depend on how the climax ended; only their text varies (*Wins stick*).

### Wins stick
- **A move reads what the player has resolved.** When an earlier card could resolve a move's target (the loan payment made, the pitch won), the move is written with two faces:
  - the **plain face**, and
  - a **despite-you face** that starts from the player's win and plays out another way ("Though the crews made their payment, Dorog had already bought the loan…").
- **Both faces leave the same public fact** for the chapters to stand on, so no card built on the move changes. The mechanical layer shows the face that matches the facts when the layer opens (mechanics §5).
- A move never silently undoes the player's win.
- **The Return has variants** keyed to the climax turn card's ending groups (won / lost at least). Each writes the problem as what's left after the last move and the climax ("the forfeit is broken, but the quay sits empty"). The cards, families and slots stay the same.
- **Return truths may cite the revelation.** It is fixed before the climax, so citing it never depends on how the climax ended.

### Takings and threads
- **Every taking gets a rescue.** A move that takes people (bonded, marched off, seized) gets a Rescue of them somewhere downstream: the climax Rescue (§9), a minor quest, or a fallout card (fallout, *The taking's Rescue*). No two Rescue cards free the same people.
- **The revelation's destination gets a minor quest.** When the revelation names where victims went, a minor quest is set at that place, attached to the climax chapter, or to an Ordeal chapter if that is nearer the place (§7, *Minors sit near their node*). It may be that taking's Rescue.
- **The hook's lost thing is an open thread.** If the hook takes a thing or people (the stolen mules), it is an open `thread` that a later card closes by putting it in a slot (the thing sought, the captive, the cargo), so the player can get it back or learn its fate. The revelation naming where it went points to that card; on its own it doesn't close the thread.
- **Landmarks come back changed.** The climax or Return turn card is set at a landmark from an earlier chapter, in the state a move left it.
  - "Landmark" is a tag the main story gives a place. Every turn card's place gets that tag.

## 6. Scale arcs (one rolled per story)

- **Fresh stories:** the arc and the antagonist shape the last two worlds used are unavailable unless nothing else fits (mechanics, *Fresh stories*). *(v6c)*

**Rising — up** (weight 35%)
- **Scope:** community (hook, threshold) → region (trials) → realm (ordeal, climax)
- **Antagonist:** *Above*, with a faction of realm reach. A sub-faction may borrow its parent faction's realm reach, for this rule and for the scope of its moves.
- **Defining rule:** the enemy gets bigger with each layer: Hand → lieutenant → antagonist. The revelation shows the local harm was one move in a realm-wide plan.

**Ripple — out** (weight 25%, L ≥ 6, W ≥ 3)
- **Scope:** community for every chapter except the climax, which is at the faction's reach. No two trials chapters on the same path share a region.
- **Antagonist:** *Above* or *Beside*; leads a faction
- **Defining rule:** **three faces of one scheme.** Across the trials, the scheme appears where locals profit from it, where they resist it, and where it has **already won**.
  - The "already won" face fills **every chapter of one trials layer**, so every path sees it. The three faces are counted **per story**; every path sees *already won* plus at least one other face.
  - **Trials moves are region scope**, because one scheme strikes several communities. The chapters stay community scope.
  - The climax is at the faction's **seat**. The seat must fill a slot on an earlier main card: a destination or holding-place slot, or the place of a **non-turn** card (so the community landmark rule never clashes with the seat's standing). Its standing follows the faction's reach (rulebook §5).

**Inward — in** (weight 20%)
- **Scope:** region or realm (hook, threshold) → community (trials) → household (ordeal, climax)
- **Antagonist:** *Within* or *Beside*, acting through a faction whose reach is at least the hook's scope
- **Defining rule:** **the war stays on.**
  - The hook's harm card is a **Siege**, and its endings leave out "threat ended."
  - Every later opening card restates the Siege's pressure as a public fact.
  - The household-scope climax faces the act that set off the war, or keeps it going.

**Deep — back** (weight 20%)
- **Scope:** community throughout; household in the ordeal
- **Antagonist:** *Beside* or *Within*
- **Defining rule:** the stake is **foundational**: the community's founding works, charter, or land.
  - Each trials chapter uncovers an **older wrong of the same kind**. Each older wrong is **its own past-event entry**, so none is a second cause of another.
  - The revelation is that the antagonist is heir to that line of wrongs. *Old wound* is preferred.
  - The heart, or her fallback standing in her place, is on every turn card.
  - The Ordeal and Return chapters each get +1 card.

**All arcs:**
- Set pieces are allowed in every arc (cap 3; Deep 1).
- The **Return's scope** is always the heart's home community.

## 7. Epic weight rules
1. **Scope floor:** every chapter is at least community scope. The exceptions are the household chapters the arcs call for.
2. **Pressure on many:** a chapter's **opening card** must threaten more than one household. This is waived in household-scope chapters.
   - The Ordeal, Climax and Return turn cards are exempt from both this rule and the scope floor. They're personal by design.
3. **Foundational stakes** belong to the main story only.
4. **Set pieces** belong to the main story only (§8). **Exception:** Schism, as a faction story's crisis.
5. **Allies are planted, and they have a price.**
   - A trials layer may carry a second shared fact: an **ally plant**. It names a person or faction with a reason to stand with the heart, and what they'd want in return.
   - "Call in your allies" ways on climax cards require that plant, and they name the price.
   - **When an ally plant exists, the climax turn card always rolls its ally way,** so the ally is never left unpaid. It is forced into the **rolled** ways, not just the pool, alongside the other plant-backed ways (rulebook, *Plant-backed ways roll first*). As a forced way, it never gives its place (rulebook §2 rule 3).
6. **Fault line.** When a faction is the antagonist's side, roll one member or sub-faction whose **virtue** the antagonist's want violates. Plant them before the climax as a shared fact. They are a potential ally from inside the enemy.
7. **Sacrifice.** The climax turn card's rolled ways **always include** one **sacrifice way**: winning by giving something up (the stake, a planted ally's goodwill, the heart's standing). It is a forced way and never gives its place; if it shares a kind with the ally way, the next way rolled must be another kind (rulebook §2 rule 3).
8. **Way-pool cap.** A main-story card's pool may grow beyond its premise list by at most:
   - one family-rule way (a shield way, or Old wound)
   - one ally way
   - one sacrifice way (climax turn only)

### Spread and presence (from the world test)
- **The heart is present in every layer, on every path:** whichever chapter the player picks in a layer, the heart (or her fallback) fills a slot on at least one of its cards, or on a **heart-thread card** for it: a minor quest attached to that chapter, in the same region. Ripple's far-flung trials included.
- **Meet them before they matter.** On every path, the antagonist, the lieutenant and anyone the climax needs (the fault line, a climax knower) are **present** on at least one card before the climax chapter: in a slot as themselves, on site.
  - An owner, holder, patron or absent slot (the owner of a dog pack, a preserve owner) doesn't count.
  - Faction stories have the same rule (faction-stories, *Meet them before they matter*).
- **Non-turn families follow the chapter problem.** Opening and middle cards roll their family weighted by what the chapter's move did (a seizure leans toward Acquire and Dispute, a taking toward Rescue). No premise appears more than **3** times in one main storyline.
- **Minor advantages spread out:** no main node is the target of more than **2** minor quests' advantage endings.
- **Minors sit near their node.** A minor quest is set within about **2 days' travel** of its chapter (distances come from the map). A minor at a place a rule fixes (the revelation's destination, a taking's Rescue) attaches to the eligible chapter nearest that place, and may sit further out.
- **Layers start near where the last one ended.** Each chapter's opening card is offered within about **3 days' travel** of the turn card of every chapter linked into it, since the player may arrive from any of them. **Exceptions:** Ripple, whose journey is the point; and the Return, which goes home wherever the climax was.

### Plant what the climax asks for
Ported from faction-stories, *Plant what the crisis asks for*.
- **A heart's secret** is a hidden fact on the heart (a debt she hides, a name she gave up). The heart may carry one only when it is:
  - **planted on a Trials or Ordeal card** (a plant pointing at it, or a knower who can tell it), following the plant rule (assembly §6); and
  - **offered on the Ordeal or the climax**, as a way or a twist. If the climax runs short of way slots, it is offered as the twist (rulebook, *Roll order*).
- **The Return may pay it off, but never shows it first.** A secret first seen in the Return is a payoff with no plant.
- Otherwise the heart is cast without one.

## 8. Set pieces (main-story premises; Schism also as a faction crisis)

| Family | Premise | Requirements | Ways through (roll 2–3, ≥2 kinds) | Separation |
|---|---|---|---|---|
| Threat | **Siege**, in one of two variants: *target* (they want the place) or *in the way* (it stands on their road) | the attacking faction's reach ≥ the chapter's scope; attackers are *rival* or *at war* with the defenders' faction, or the defenders are unaligned; **opposing slot:** the attackers' commander (a representative); a **defending leader** slot | break the siege line (confront); run supplies through the lines (protect); find whoever inside will open a gate, before the enemy does (undermine); turn their hired companies (undermine; *requires a hired-companies fact*); negotiate terms, or offer passage and supply (bargain; *removed if the commander is Top*) | n/a |
| Dispute | **Succession:** a seat below Top is contested by two claimants from the **same** faction (two sub-factions, or a sub-faction against its parent) | the last holder is dead (new) or gone; the seat is a place or a title thing; both claimants are members; in main use, the heart's side backs one claimant | expose what each backer will demand once their claimant sits (truth); learn what the household and garrison will actually accept (truth); broker a divided rule (deal); put forward a compromise claimant (deal; fills a slot); win the household and garrison to one claimant (side) | applies (inheritance) |
| Passage | **Embassy:** carry terms, a hostage, or a token between two factions at war, while hardliners on both sides want it to fail | the two factions are *at war* or *rival*; cargo is a person (a hostage is cargo under escort) or a document; a **hardliner leader** slot; pursuers are hardliners | travel under a neutral faction's protection (arrangement; fills a representative); go as a common courier (cover); take the road neither side watches, guided by someone who has crossed both lines (route; fills the guide); fight off the hardliners (force); win over the hardliner leader (arrangement; *removed if Top*) | n/a |
| Reckoning | **Schism:** the antagonist is made to answer to their own faction or its parent | a fault line planted (§7.6); the antagonist is a member; the **wrong** is personal harm to the heart; **shield:** *faction law* (only the faction can judge its own; adds no extra way; the faction's leaders count as the arbiter whatever their standing, per rulebook §5; **if the antagonist and lieutenant are themselves the faction's leaders, the parent faction's leaders judge**) | bring the proof before the faction's own leaders (justice); turn the betrayed members or sub-faction (ruin); let the faction disown the antagonist quietly, in exchange for silence about its hidden side (amends; *only if the revelation published that hidden side*); make the antagonist answer by the faction's own custom (violence, *contest*) | exempt (authority) |

**Placement (extends assembly's tables):**

| Set piece | As a turn card | On non-turn cards |
|---|---|---|
| Siege | Trials | any chapter; the Hook's opening (harm) card in Inward |
| Succession | Trials | any chapter |
| Embassy | Threshold; Trials | any chapter |
| Schism | Climax (any shape) | — |

- **Hook harm for a Siege:** works broken (*target*), or livelihood collapsed (*in the way*).
- **Twists:** each set piece uses its family's twist table.
  - Siege uses Threat's table. *Passed along* applies only to the *in the way* variant.
  - Succession uses Dispute's table. *Caught in the middle* applies only when the seat is a place.
  - Embassy uses Passage's table. *Pursuer has a point* needs the hardliner leader. *The one left behind* needs a person cargo.
  - Schism has its own column:

| Twist | Schism |
|---|---|
| The wronged don't want it | ✓ |
| Changed | ✗ |
| Hand, not head | ✗ (the antagonist *is* the head) |
| Disproportionate demand | ✓ |

## 9. Role changes for main stories

### Hook chapter
- The **opening card** carries the harm, which is the first move. Assembly's hook table applies to that card.
- **Turn card:** any family except Restore and Contest, as long as it can carry a public lead plant.
  - Assembly §5's premise limits still apply.
  - Investigate's hook filters apply here too: the culprit is the Hand or a proxy.
- **One harm only.** No other card in the hook chapter harms the heart or her kin.
- **Turn card job:** give the first lead, as a public plant pointing at the antagonist's side. This plant is never the revelation.

### Threshold
- The tables are extended by §8: Embassy is allowed.

### Ordeal chapter
- The event is the layer's move.
- **The main Ordeal stays on the heart** (Laz, v6b): its event is *captive*, *disgraced* or *estranged*. Blows on the player or the guild are guild-story only (assembly, *Guild chapter-2 doors*).
- **The turn card must be a door.** This row **replaces** assembly's Disgraced row:

| Event | Doors |
|---|---|
| Disgraced | Dispute (*Public offense*); Passage (*Unwelcome cargo*: the heart is under a ban; **opposing slot: the waypoint headman who bars her**); Restore (*Lost good name*, cause *false, spread by someone off the card*), **barred whenever the Return repairs the disgrace** |

### Lieutenant (cast)
- **Exempt** from the reuse cap, like the rest of the cast.
- **Spec:** High standing; either the antagonist's employee or a member of the antagonist's faction.
- **Trials:** fills the opposing slot on **every turn card of one trials layer**, so every path meets them. Trials opposing slots accept the Hand, the lieutenant, or a proxy.
  - The slot must be one where they are present (§7, *Meet them before they matter*). An owner slot isn't a meeting.
- **Climax:** appears on a non-turn card.

### Climax chapter (the last move)
- The chapter is **one aftermath**. Each card is one front of the last move.
- **Rescue placement:** if a later move took people, a **Rescue** of them is **the card just before the turn card**. Where they're held must be known from a shared plant or a move.
  - It never targets captives who were already the subject of an earlier Rescue card.
  - It never targets the heart.
- **Climax uses the player's path.** A climax card may put someone first met on an earlier card in a slot only if every path into the climax meets them: the main cast, the heart's people, someone on a width-1 layer, or someone on a card in every chapter of a layer. Anyone else needs a `fallback` for the paths that never met them (casting will choose fallbacks).
- **Turn card:**
  - The antagonist fills the opposing slot.
  - It pays off **at least two plants from different layers, and one of them is the revelation.**
  - Its family is the shape's family, or **Schism**.

### Return
- **Chapter problem:** the aftermath of the last move. Its text has variants keyed to the climax ending, and its truths may cite the revelation (§5, *Wins stick*).
- **Repairing the hook's harm** is a new event. It doesn't count as undoing a move.
- **Repairing a disgrace:** the Return may use Restore's cause *false, spread by someone off the card*.
- **Return casting.** The Return reuses kin and people the story has already shown. It creates nobody whose role an established person could fill: no new cousin where the fallback or a known neighbour would do.
- **The heart's secret** may be paid off here, never shown here first (§7, *Plant what the climax asks for*).
- **No minors** attach to the Return (assembly §8).

### Proxies (amends assembly)
A **proxy** is either:
- a person with a direct link to a cast member, or
- a member of the antagonist's faction or its sub-factions (members of the parent faction don't count).
- **Exception:** a *bystander member* on a fallout card is not a proxy (`quest-fallout.md` rule 5).

## 10. Generation order
1. **Spine:**
   - Roll the arc, then a shape the arc allows.
   - Fill the cast: antagonist, want, stake (foundational for Deep), heart (with or without a secret, §7) and fallback, Hand, lieutenant.
   - Add the antagonist's faction, the ally faction, and the fault line.
2. **Diamond:** L and W within the arc's range, roles, scope per layer, chapter lengths.
3. **Moves:**
   - The harm and the revelation. The revelation is a secret stored on the Hand or a proxy. If the antagonist leads a faction, it may include that faction's hidden side.
   - One move per later layer, each with its lever, ending with the last move (a finished act). A move whose target an earlier card could resolve gets its despite-you face (§5).
4. **Twists, then chapters.** Roll the twists in the rulebook's *Roll order*: climax, Return, Ordeal, then chapter turn cards, then the rest. Then write the chapters layer by layer: opening (chapter problem from the move) → middle → turn (role job, plus shared plants).
5. **Checks:**
   - plant rule, including chapter payoffs
   - chain and climb in every chapter
   - no secret leaks
   - climax payoffs; ally and other plant-backed ways rolled
   - revelation on a fitting card
   - document-plant limit
   - epic weight
   - heart and *meet them before they matter*, per path
   - climax cast on the player's path, or a fallback
   - no turning card plain
   - the heart's secret planted on Trials or Ordeal and offered on the Ordeal or climax, or no secret
   - minors near their node; each layer starts near where the last ended
   - every taking rescued; the hook's open thread closed
   - the want survives the last move; revelation costs paid
   - landmarks
   - way-pool cap
6. **Return (cast from people already shown), minor quests, fallout quests (from each move), other side quests, background pass.**

**Casting.** Every slot above, the cast included, is written as a **spec** (`quest-casting.md`). The main story is the realm layer: its specs, its Return's, its minors' and its fallout's are cast before the faction stories are generated (`quest-locations.md` §7a). Until casting exists, each card resolves its own slots in the same order (rulebook §3).

## 11. Sample path: Rising arc, *Above*, L=5 (15 quests)
*Illustrative. Uses the project's factions; the people are invented.*

### Spine
- **Antagonist:** Ostrav, chief of the Binders (High; the faction has realm reach). **Want:** ambition, to buy the Binders a lordship by owning the imperial High Road's northern toll.
- **Lieutenant:** Seyka, master of the northern pens (High, a Binder, *lawless*).
- **Hand:** Drask, a slave-captain (Middle, Ostrav's employee).
- **Heart:** Ilse, a Road Crews carter (Middle).
  - **Stake:** the Weir crossroads (a place on the High Road), where she lives.
  - **Fallback:** her cousin Maro (kin).
- **Ally faction:** the Lantern Company.
- **Fault line:** Old Tamas, a Binder drover.
  - **Why:** the Binders' virtue is a hunger to be *accepted*. Ostrav's plan to take the road by force would make them hated for good.

### Hook (community)
- **Card 1 (harm):** Threat, *Raiders*. Riders burn the Weir waystation.
  - Why-here: *paid*. Patron: a Binder factor (proxy).
  - **Harm (move 1):** *the Weir waystation is burned.*
  - **Revelation** (secret, stored on the factor): *Ostrav means to break the Road Crews and own the northern toll to buy the Binders a lordship, though many Binders hate the trade and see no other door* (the faction's hidden side).
- **Card 2 (turn):** Investigate, *Theft*. The carters' wage chest vanished in the fire. Culprit: the factor's runner, a Binder (a proxy; hidden).
  - **Lead (public plant):** *the raiders were paid in Binder coin.*

### Threshold (community)
- **Move:** *Binder bands are openly taking carters off the north road under false toll warrants.*
- **Card 1:** Passage, *Fugitive*. A carter who witnessed a taking flees the Binder who did it (a proxy).
- **Card 2 (turn):** Rescue, *Kidnapped*. Free a cart crew from the holding pens at Gallow Ford.
  - Captor: Drask (lawless; the pens are his hideout).
- **Shared plants:**
  - *the taken are driven on to the northern pens*
  - **fault line:** *Old Tamas spoke against Ostrav at the ford*

### Trials (region: the High Road's northern stretch, which runs through one lordship)
- **Move:** *Binder-bought wardens, holding purchased imperial warrants, have seized the northern toll-house.*
- **Card 1:** Acquire, *Under seal*. The toll rolls are held at the guarded toll-house by the warden (holds legal authority by warrant; proxy) until a ruling.
  - **Plant:** *the rolls show payments to the northern pens.* This is the diamond's one document plant.
- **Card 2:** Dispute, *Broken agreement*. The Road Crews, through their foreman (Middle), against the toll agent (Middle), over the road-rights agreement.
  - Trade link; one tier apart at most, so separation is OK.
- **Card 3 (turn):** Investigate, *Suspicious death*. A Lantern guard is dead near the pens.
  - Culprit: **Seyka** (lieutenant; the layer's only trials layer, so every path meets her).
  - **Chapter payoff:** uses the rolls.
- **Shared plants:**
  - **Revelation**, attested by the dead guard's partner.
  - **Ally:** *the Lantern Company will stand against the Binders on the High Road, for the road's guard contract for a year.*

### Ordeal (realm)
- **Move = event, *disgraced*:** *an imperial writ, bought by Ostrav, names Ilse a Binder smuggler; Empire towns bar her and anything under her seal.*
- **Card 1:** Reckoning, *Betrayal*. The Road Crews' bookkeeper lent Ilse's seal for the forgery.
  - Wronged: the Road Crews (through the foreman; employer/employee link). Shield: *Protected* (by Seyka, who outranks them).
  - The bookkeeper is a proxy (Seyka paid him).
- **Card 2:** Passage, *Smuggling*. Get Ilse's seal-marked ledgers, forbidden under the writ, past the ban checkpoint into the court town.
- **Card 3 (turn, door):** Restore, *Lost good name*. Subject: Ilse. Requester: Maro. Cause: false, spread by someone off the card.
  - Allowed: the Return repairs the waystation, not the disgrace.

### Climax (realm)
- **Last move:** *Ostrav's bands have sacked the Weir and hold its people at the northern pens, to force the Road Crews to sign the north road over.*
- **Card 1:** Passage, *Embassy*. Carry the Road Crews' terms between the Open Hand council and the Binders, while hardliners on both sides want it to fail.
  - "Travel under the Lantern Company's protection" can roll; its price is named.
- **Card 2:** Rescue, *Kidnapped*. Free the Weir's people from the northern pens.
  - Captor: Seyka (lawless; the pens are her hideout).
  - Location known from the layer-2 plant.
- **Card 3 (turn):** Reckoning, *Schism*. Ostrav must answer to the Binders' own elders.
  - Wronged: Ilse (the writ, the sacking).
  - Shield: *faction law*.
  - Rolled ways include the **sacrifice way**: *give the Binders a share of the Weir's toll for a year in exchange for Ostrav.*
  - The amends way is available, because the revelation published the hidden side.
  - **Payoffs:** fault line (layer 2) + **revelation** (layer 3).

### Return (home)
- **Chapter problem:** the sacked Weir.
- **Card 1:** Dispute, *Shared resource*. The Road Crews and the Sharp Hands both claim the right to rebuild the sacked Weir.
- **Card 2 (turn):** Restore, *Broken works*. The waystation, burned in the hook and sacked in the last move.
  - Cause: *deliberately broken*.
  - Requester: Ilse (reason: pride).
  - Twist: *Can't go back*.
  - Landmark returns changed: the waystation first named in the hook.

### What it shows
- **One problem per chapter:**
  - the burning
  - the takings
  - the toll-house
  - the writ
  - the sacking's aftermath
  - the rebuilding
- **The antagonist acts between every chapter,** and every move is a finished act.
- **Scope rises, and each step is earned:**
  - community
  - region: a lordship's stretch of road
  - realm: an imperial writ, then the realm road
- **The enemy has an inside:** Tamas, and the Binders' hidden side, make the Schism climax possible.
- **The Return** happens in a world the moves made. It never assumes how the climax ended.
- **Count:** 2 + 2 + 3 + 3 + 3 + 2 = **15**.
- **Predates v3.4.** To pass the new rules, Ostrav and Old Tamas would each need to be present on an earlier card (*Meet them before they matter*), the chapters would chain, and the threshold move would get a despite-you face for a player who stops the takings. Under v3.6, the Return's Sharp Hands would speak through someone already shown (*Return casting*).

## 12. Changes from review
- **v3.6 (2026-10-27, v6: World C fixes):**
  - *Plant what the climax asks for:* a heart's secret is planted on Trials or Ordeal and offered on the Ordeal or climax; the Return never shows it first (SC-C4, PD-C3).
  - *Return casting:* the Return reuses people already shown (PD-C3).
  - Turning cards are never plain, and their twists are rolled first (pointers to rulebook *No blank card*, *Roll order*; SC-C3, CM2).
  - *Minors sit near their node*; each layer starts within about 3 days of where the last ended, Ripple and the Return excepted (PD-C4).
  - Rising: a sub-faction may borrow its parent's realm reach (CM1). Landmark standing is the settlement's (CM5). No minors on the Return (CM6).
  - Reserved levers: the last move's lever and every lever used on 2 or more moves, cited by rule name (SC-C5).
  - v6b: Laz's guild changes (27 Oct): the main Ordeal stays on the heart; player and guild targets are guild-story only (§9).
- **v3.5 (2026-10-26, audit fixes):**
  - Each move names a lever; the story's lever is the last move's (core M3).
  - A Siege at the hook sits on the opening (harm) card, not the turn card (core M8).
  - The climax's ally and sacrifice ways are forced ways that never give their place (core M4).
  - Slots are specs, cast layer by layer; the main story is cast before the faction stories (core H6, M5).
- **v3.4 (2026-10-26, v5: world-test fixes):**
  - Chapters chain and climb; "different doors, same problem" is gone (SC6, K2).
  - Heart presence is checked per path, with heart-thread minor quests (PD1, M9).
  - *Meet them before they matter* for the antagonist, lieutenant and climax people; owner slots don't count (SC4, PD1, K3).
  - The climax ally way is forced into the rolled ways; plant-backed ways roll first (SC5, PD3, CO10).
  - *Wins stick:* moves get a despite-you face; the Return has variants by climax ending and may cite the revelation (PD2, CO5).
  - *Takings and threads:* every taking gets a Rescue, the revelation's destination gets a minor quest, the hook's lost thing is an open thread (SC4, CO5, PD3).
  - The revelation sits only on a card that can carry a discovery (PD3).
  - Climax cards use cast from the player's path or carry a fallback (PD4).
  - The villain's want survives the last move; revelation costs are paid (CO8).
- **v2:** chapters with one problem and a payoff, antagonist moves, scope, changed landmarks, arcs as directions, a foundational stake, ally prices, the fault line, sacrifice, the set-piece overhaul with Schism, turn-card-only family limits, the lieutenant as cast, a new ordering clause, proxies via faction.
- **v3.1–v3.2 (full test and final audit):**
  - Chapter scope may be below the move's scope.
  - Ripple trials moves are region scope.
  - Every card may depend on its chapter problem.
  - The hook turn card's families are defined.
  - The *Unwelcome cargo* door has an opposing slot.
  - The lieutenant is accepted in Trials opposing slots.
  - The seat-slot rule is clarified.
  - Return repairs aren't undoing a move.
  - The climax Rescue sits on the card before the turn card and excludes the heart.
  - The Disgraced row replaces assembly's.
  - The Schism arbiter is restated.
- **v3:**
  - **Hook harm:** moved to the opening card, one harm only.
  - **Scope:** comes from the move; only opening cards must reach it; exemptions defined.
  - **Inward:** the threshold Siege removed (it would share a family with the hook).
  - **Succession:** removed from the Climax.
  - **Set pieces:** allowed in every arc.
  - **New rule:** no secret leaks.
  - **Schism:** gains a shield, a twist column and a wrong.
  - **Last move:** must be a finished act; the Return grows from its aftermath.
  - **Ripple:** L ≥ 6, with the "already won" face on every path.
  - **Lieutenant:** fills every turn card of one trials layer.
  - **Way-pool cap.**
  - **Plant placement:** shared plants sit on turn cards.
  - **Return:** may repair a disgrace.
  - **Deep:** older wrongs are separate events.
  - **Mechanical wording:** removed.
  - **Sample:** rebuilt.
