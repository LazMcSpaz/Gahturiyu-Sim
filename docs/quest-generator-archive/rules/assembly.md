# Assembly — Locked (v3.7)

How family cards are put together into one main story, the minor quests attached to it, and independent side quests.

> **For main stories, `quest-main-quest.md` extends and overrides this doc.** It turns each node into a chapter, adds the antagonist's moves, scale arcs, scope, set pieces, and the lieutenant, and revises the cast limit (now one slot per chapter), the ordering clause, the proxy definition, and the default diamond size (L=6, W=3).

## 1. Core ideas
- **A node is a card.** Each node is one quest from one family.
- **The main story has a spine:** a small main cast that every main node draws from.
- **Edges are threads.** Two linked nodes share a cast member, a proxy, or a planted fact. Sharing only a place isn't enough.
- **Escalation is built in.** The antagonist gets closer with every layer.
- **The shape locks at generation.** Which nodes the player visits, and what their endings change, is mechanical-layer work.

## 2. The main cast (the spine)

| Cast slot | Spec | Notes |
|---|---|---|
| **Antagonist** | set by the **shape** (below) | the force the story pushes against |
| **Want** | greed, revenge, cover-up, desperation, ambition, or preservation | what the antagonist wants with the stake; the revelation exposes it |
| **Stake** | a place or thing | what the story is about |
| **Heart** | Low or Middle person with a **tie** to the stake (owns it, works it, or lives at it) | who the player is doing this for. No way through may harm or kill the heart, and no card may describe the heart as dead. |
| **Hand** | Middle person with an employer/employee link to the antagonist | the antagonist's one recurring face. Under *Within*, the Hand may instead be linked to the heart, which makes them a possible betrayer. |

### Antagonist shapes
Roll one per story.

| Shape | Antagonist | Preferred climax | Fallback climax |
|---|---|---|---|
| **Above** | High or Top | Reckoning, *Ruin from above* | Reckoning, *Ruin from above* |
| **Beside** | Middle, within one tier of the heart (moneylender, headman, rival trader) | Reckoning, *Swindle* | Reckoning, *Swindle* |
| **Within** | has an ally, family, kin, or employer link to the heart | Reckoning, *Betrayal* | Reckoning, *Betrayal* (accepts kin) |

### Cast proxies
- A **proxy** is an ordinary registry person with a direct link (any rulebook §6 link type) to a cast member. Examples: the antagonist's debtor, the Hand's hired man, the heart's brother.
- A proxy recorded on a card counts as cast presence.
- Proxies follow **normal** reuse rules: the cap of 3 and once-per-family. They are not limited to one slot per chapter.
- A paid agent is linked as **employer/employee**.

### Cast reuse
- Cast members appear only in the main quest, its Return, and its minor quests.
- Within those, they are exempt from the cap and from once-per-family.
- They still fill at most **one slot per chapter** (main-quest §3).
- **The heart's fallback** counts as cast (and is exempt in the same way) whenever it fills a slot in the heart's place.

### Groups
A guild, village, or family stands as a party through one **named representative**.

## 3. The diamond
- **Size:** two numbers, **layers (L)** and **peak width (W)**. W must satisfy **2 ≤ W ≤ (L+1)/2**, rounded down.
- **Width of layer k:** min(k, W, L+1−k). Examples:
  - L=3, W=2: 1-2-1
  - L=5, W=3: 1-2-3-2-1
  - L=7, W=3: 1-2-3-3-3-2-1
  - L=7, W=4: 1-2-3-4-3-2-1
- **Return tail:** one node after the climax, not counted in L.
- **Roles, in order of precedence:** Hook > Climax > Ordeal > Threshold > Trials.

| Role | Layer |
|---|---|
| Hook | 1 |
| Climax | L |
| Ordeal | L−1, only when L ≥ 4 (always width 2) |
| Threshold | 2, if not already assigned |
| Trials | all remaining layers |

- At **L=3**, there is no Trials or Ordeal layer.
- **Revelation layer:** the last layer before the Ordeal. If there's no Ordeal, it's the last layer before the Climax. At L=3 or 4, this is the Threshold layer.

## 4. Role rules
- **Role rules are hard. Family preferences are soft.**
- If no family can meet a role's rules, the variety rules give way first.
- If there's still nothing, use the role's fallback.

### Hook (layer 1)
**Job:** the antagonist's side harms the heart or the stake.

- **The hook card is a normal card of its family.** Its public story is whatever that family produces: an accident, raiders, a dispute, a sickness.
- **The revelation sits above the card.** The secret intent behind the harm (drawn from the want) is stored as a **secret fact on the Hand or proxy who acted**, not in the card's truth. The family's own rules still govern the card. For example, Threat's "sickness is never deliberate" stays true *on the card*, while the secret says someone arranged it. This is a main-quest exception.
- **The harm is a public past-event fact** and counts as the **layer-1 plant**.
- **Don't bench the heart:** the heart is never the captive, cargo, missing person, or accused at the hook. Use the heart's kin or ally instead.

| Family | Premises allowed at the hook | Harm recorded |
|---|---|---|
| Investigate | all four (*no crime*, *the truth costs the requester*, *wrong target*, and culprit "none" filtered out) | Theft: livelihood · Disappearance: kin missing · Suspicious death: kin or mentor dead (new) · Sabotage: works or livelihood |
| Threat | all four | Raiders: works burned or livelihood · Protected beast: livelihood · Sickness: livelihood, or kin sick (unfixable) · Encroachment: works or livelihood |
| Rescue | all four (the captive is the heart's kin) | kin held |
| Dispute | Contested inheritance, Shared resource (the antagonist's proxy contests the heart's side) | the heart's claim challenged |
| Acquire | Under seal (the stake, or the heart's tools, seized by a proxy authority) | livelihood |
| Passage | Fugitive (the heart's kin flees the Hand) | kin hunted |
| Reckoning | Betrayal (*Within* shape only: the heart is betrayed) | bond broken |

- Restore is never a hook; repair is the Return's job.
- **Fallback:** Investigate, Sabotage.
- **Repair guarantee:** the Return needs something it can fix (§4 Return).
  - **Fixable harms:** works, livelihood.
  - **Unfixable harms:** kin held, missing, dead, hunted, or sick; claim challenged; bond broken.
  - If the hook's harm is unfixable, the ordeal event must be *disgraced* or *estranged*, so the Return can repair the heart instead.

### Threshold
**Job:** the player openly stands against the antagonist's side for the first time.
- The **Hand or a proxy** fills the card's opposing slot (§5).
- The card's place is not the hook's place.
- **Allowed families:** Passage (Smuggling or Fugitive); Dispute (with the player as the heart's side's advocate); Rescue.
- **Fallback:** Passage, Fugitive.

### Trials
**Job:** learn the antagonist's methods; gain allies and proof.
- The Hand, the lieutenant, or a proxy fills the opposing slot, **or** is named in the card's truth.
- **Allowed families:** any except Contest (Contest is never a main turn card). **Fallback:** Investigate, with the Hand or lieutenant named in the truth.

### Shared and unique plants (every layer after the Hook)
- **Shared fact:** every node in a layer plants that layer's shared fact.
  - In the **revelation layer**, the shared fact is the revelation.
  - Each card states it the same way, plus a **knower**: a person slot on that card who can attest it.
    - Example: *"Hale ordered the gates cut, and the engineer can attest it."*
    - The knower differs from card to card.
    - In main stories, the revelation sits only on a card that can carry a discovery (main-quest, *Revelation needs a fitting card*).
- **Unique fact:** each node may also plant one fact of its own. It can seed minor quests, and it must not reveal that card's own hidden truth.

### Ordeal
**Job:** the antagonist wins something, and it costs the heart personally.
- Roll **one ordeal event**. The antagonist is named as its cause in a public fact.
- **Every ordeal node's premise grows from that event.** Different doors, same wound.
- **Sibling ordeal nodes never contradict each other's state,** and they must be different quests (not the same captive behind two names).

| Ordeal event (target: the heart) | Door families (premise) |
|---|---|
| **Captive:** the heart is held as leverage | Rescue (Kidnapped, or Bound by debt): get her out. Acquire (Won't sell or Under seal): obtain what the captor demands for her release, which someone else holds. |
| **Disgraced:** the heart is publicly accused | Dispute (Public offense): the Hand or a proxy demands redress from the heart. Restore (Lost good name, cause *false, spread by someone off the card*). Main-quest §9 replaces this row. |
| **Estranged from kin:** the antagonist turned the heart's kin against her | Reckoning (Betrayal, kin link). Restore (Estrangement). Dispute (Contested inheritance). **Each door uses a different kin member.** |

- **Ordering clause:** a state set at layer k binds only later layers and the Return.
  - If the event is *captive*, the heart may still fill **passive slots**: slots that don't need her to act (wronged, captive, missing person, deceased's heir). Active slots (requester, party, cargo) in later layers go to her **fallback** (someone linked to the heart, chosen by casting).
  - Swapping the heart back in if she's freed is mechanical-layer work.

### Guild chapter-2 doors (Laz, v6b)
A guild story's chapter 2 move is an ordeal event with a **target**, rolled per story (the rule's home is faction-stories §6, *Phase 3*). **Guild stories only:** the main Ordeal stays on the heart (main-quest §9). The event is the door's wound, and its turn card is one door from the rows below.
- **The heart:** the three rows above, with main-quest §9's Disgraced row. In a guild story, *disgraced* means disgraced under the code.
- **The player and the guild:** the rows below *(Claude, from the v6b fix list's suggestions; Laz set the events, not the doors)*. Premises are listed so the planner can deal different ones: chapter-2 turn cards share one phase position, so no two guild stories' doors share a family + premise (faction-stories §9).

| Target | Event | Door families (premise) |
|---|---|---|
| **Player** | **Framed:** accused of a code-break they didn't do | Investigate (*Sabotage* or *Theft*): what the player is blamed for was done by the Hand or a proxy. Reckoning (*Ruin from above*, shield *Bought* or *Protected*): the framer is known; the false charge is the ruin. |
| **Player** | **Stripped of rank:** thrown out, or put down a rank | Restore (*Lost good name*: the subject is the player; cause *false, spread by someone off the card*, or *exaggerated*). Contest (*The ladder*, or *Champion's bout* where the rank was sworn to a ring): win it back before the guild, in whatever contest the guild keeps (a bout, a copying trial). |
| **Player** | **Arrested:** held and judged | Passage (*Fugitive*: the player is the cargo; get clear of the lordship; the pursuer is the Hand, under Passage's Hand-may-pursue exception, extended to this door). Dispute (*Public offense*: before the town's judge; the accuser against the guild, through the heart, who speaks for the player). |
| **Guild** | **Hall seized:** a hall or the headquarters taken or shut | Acquire (*Under seal*: what opens it again, its charter, keys or seal, held by whoever shut it). Dispute (*Broken agreement*: the hall's lease or grant; separation applies). |
| **Guild** | **Members taken:** bonded, arrested or marched off | Rescue, the premise by how they were taken: bonded, *Bound by debt*; arrested, *Imprisoned unjustly*; marched off by a band, *Kidnapped*; taken in a fight by a lord, *Captured in battle*. |
| **Guild** | **Outlawed:** a writ against the guild in one lordship or march | Dispute (*Public offense*: the writ's charge, before that lordship's court). Reckoning (*Ruin from above*: the writ's backer, the enemy or the lord they bought). |
| **Guild** | **Leader struck down:** wounded, killed or disgraced (the kit says which leaders may be) | Killed: Investigate (*Suspicious death*) or Reckoning (*Blood debt*, when the killer is known and walked free). Wounded: Investigate (*Sabotage*: the blow came through something rigged). Disgraced: Reckoning (*Ruin from above*) or Restore (*Lost good name*). |

Where a family's rules didn't fit the suggested door, the door changed *(Claude)*:
- **Acquire takes a thing, not a place,** so a seized hall's Acquire door goes after what opens it (its charter, keys or seal), not the hall.
- **Reckoning's premises need a death for *Blood debt*,** and *Betrayal* needs a link the player doesn't have at generation. So a framing uses *Ruin from above* (its *Bought* or *Protected* shield works at any standing), and a leader only wounded gets Investigate *Sabotage* alone.
- **No Contest door for an arrest.** Where a town's custom is a duel, the sim already fights it (`judge`); a Contest card would fight it twice.
- **Members taken is Rescue only,** but its four premises give the planner room.

**Doors by how the blow fell** (v6d; *Claude*, from the v6c review). An event alone doesn't pick a door: a crew taking a back room is not a lease quarrel, an ambush is not sabotage, and a thrown bout is not a theft. So the plan deals a **row**: how the blow fell. The row sets the blow's means (its `uses` is one of them), its doors, and what fills each door's slots. Where this table and the one above differ, this one holds. The roller reads it (`test/world/roll_kits_d.py`).

| Target | Event | How it fell | Means | Doors (and what fills them) | Key |
|---|---|---|---|---|---|
| heart | *captive* | held by the enemy's men, as leverage | `men` | Rescue *Kidnapped* (captor: {enemy}'s men; why taken: leverage on the guild) | held-by-men |
| heart | *captive* | held for a debt the enemy bought | `coin` | Rescue *Bound by debt* (creditor: {enemy}; the debt: the heart's own, bought up) | held-for-debt |
| heart | *captive* | held under a charge the enemy laid | `paper` or `faith` | Acquire *Under seal* (sealed: the papers of {enemy}'s charge, held by the court that holds her) | held-on-charge |
| heart | *disgraced* | cried down for a code-break; a member heart only | `name` or `inside` | Dispute *Public offense* (offended: whoever {code} wronged); Restore *Lost good name* (the stain: {code}) | cried-down |
| heart | *estranged* | her kin turned against her | `kin` or `name` | Reckoning *Betrayal* (one kin member); Restore *Estrangement* (another); Dispute *Contested inheritance* (a third) | kin-turned |
| player | *framed* | for destroying a text, or throwing a bout | `inside` or `name` | Investigate *Sabotage* (blamed for {code}; done by the Hand); Contest *The fix* (Dueling Society only; fixer: the Hand; fix type: paid fighter); Reckoning *Ruin from above* (framer: {enemy}; shield: {shield}) | framed-damage |
| player | *framed* | for holding back the take | `inside` or `name` | Investigate *Theft* (blamed for {code}; taken by the Hand); Reckoning *Ruin from above* (framer: {enemy}; shield: {shield}) | framed-theft |
| player | *framed* | for taking a side in public, shaming a house, or shaming the house in public | `inside` or `name` | Dispute *Public offense* (blamed for {code}; offended: the side it shamed); Reckoning *Ruin from above* (framer: {enemy}; shield: {shield}) | framed-shame |
| player | *framed* | for abandoning a contract | `inside` or `name` | Dispute *Broken agreement* (the contract the player is said to have left, between its client and the Company); Reckoning *Ruin from above* (framer: {enemy}; shield: {shield}) | framed-contract |
| player | *framed* | for abandoning a comrade, going to the law, or informing | `inside` or `name` | Reckoning *Ruin from above* (blamed for {code}; framer: {enemy}; shield: {shield}) | framed-betrayal |
| player | *stripped of rank* | put down on the enemy's word | `inside` or `name` | Restore *Lost good name* (the stain: what {enemy} said of the player); Contest *The ladder* (the rank, won back in the guild's own contest); Contest *Champion's bout* (Dueling Society only; the rank was sworn to the ring) | stripped |
| player | *arrested* | held on a charge the enemy laid | `paper` or `faith` | Passage *Fugitive* (cargo: the player; pursuer: the Hand, on {enemy}'s charge); Dispute *Public offense* (the charge {enemy} laid, before the town's judge) | arrested-charge |
| player | *arrested* | held in a lord's cells by the enemy's men | `men` (Above) | Passage *Fugitive* (cargo: the player; pursuer: {enemy}'s men) | arrested-men |
| guild | *hall seized* | taken by a lord's men | `men` (Above) | Acquire *Under seal* (sealed: {site}'s keys, under {enemy}'s seizure) | seized-lord |
| guild | *hall seized* | occupied by the enemy's men | `men` (Beside, Within) | Dispute *Shared resource* (the place: {site}, claimed by the guild and by {enemy}) | seized-occupied |
| guild | *hall seized* | shut under a court's seal, or its lease called in | `paper` or `faith` | Acquire *Under seal* (sealed: {site}'s charter, under the court's seal, at {enemy}'s suit); Dispute *Broken agreement* (the agreement: {site}'s lease or grant, called in on {enemy}'s word) | seized-seal |
| guild | *hall seized* | shut by the enemy's own order: a hall he runs, the headquarters only when he leads the guild | `inside` (Within) | Acquire *Under seal* (sealed: {site}'s keys, held by {enemy} under the guild's own rule) | seized-order |
| guild | *members taken* | bonded for a debt | `coin` | Rescue *Bound by debt* (creditor: {enemy}, or whoever the debt was sold to) | taken-bonded |
| guild | *members taken* | arrested on a charge | `paper` or `faith` | Rescue *Imprisoned unjustly* (captor: the court, on {enemy}'s charge) | taken-arrested |
| guild | *members taken* | marched off by a band | `men` (Beside, Within) | Rescue *Kidnapped* (captor: {enemy}'s men) | taken-band |
| guild | *members taken* | taken in a fight by a lord | `men` (Above) | Rescue *Captured in battle* (captor: {enemy}) | taken-fight |
| guild | *outlawed* | a writ against the guild in one lordship or march | `paper` or `faith` | Dispute *Public offense* (the writ's charge, before that lordship's court); Reckoning *Ruin from above* (backer: {enemy}; shield: {shield}) | outlawed |
| guild | *leader struck down* | killed | `men` or `inside` | Investigate *Suspicious death* (the dead: {leader}; who did it: {enemy}'s people); Reckoning *Blood debt* (the dead: {leader}; the killer, {enemy}'s, walked free) | killed |
| guild | *leader struck down* | wounded by men, on the road | `men` | Passage *Fugitive* (cargo: {leader}, wounded; pursuer: {enemy}'s men) | wounded-men |
| guild | *leader struck down* | wounded through something rigged | `inside` | Investigate *Sabotage* (what was rigged: {leader}'s horse, stair or cup, by someone inside) | wounded-rigged |
| guild | *leader struck down* | disgraced | `inside` or `name` | Reckoning *Ruin from above* (the stain on {leader}; wrongdoer: {enemy}; shield: {shield}); Restore *Lost good name* (the stain on {leader}, spread off the card) | disgraced |

- **The blow's `uses` is a means of its row** and of the kit's *Chapter-2 blows* cell, so a leader wounded through someone inside is a rigged thing, never an ambush. The braces are filled from the story: {enemy} the enemy, {code} the code-break, {site} the seized hall, {leader} the struck leader, {shield} the framer's shield (*Bought* with `coin`, otherwise *Protected*).
- **Passage *Unwelcome cargo*** stays the main Ordeal's door for a disgraced heart (main-quest §9), where a ban and a road come with it. A guild story's disgrace has neither, so it isn't a guild door.
- **No step repeats the one before it.** A hall is never seized right after a chapter-1 move that already shut a place (the kits mark them *shuts a place*), and the crisis's family + premise is never the door's.
- **A captive heart stays held** (the ordering clause above): she never gives the crisis, and never fills a crisis the kits mark *herself* (accused, sold, carrying, taken again). Her active slots go to her fallback.

Rules for these doors:
- **The player is never cast** (mechanics §10). The slot the blow falls on (the accused, the subject, the cargo, the wronged) points straight at `p-player`. The **requester** is the heart, or her fallback, or the hall head; never the player.
- **A hidden culprit is the Hand or a proxy** (as main-quest's hook filters). The move's public fact names the blow, not who struck it.
- **The taken are never the heart.** If the heart is taken, the event is the heart's *captive*. A Rescue door frees the members through one named representative (assembly §2, *Groups*). No other card frees the same people.
- **A seized headquarters** is still where the crisis is. The crisis card must hold whether or not the door took it back: it reads the site's state with a `before` face (locations, *Settlement context*) *(Claude)*.
- **A struck-down leader** is never the story's enemy. A leader killed here carries `dies_at` this move, is met on an earlier card (the reveal, say), and can't give the crisis.
- **Arrests run in the sim.** The sim's watch takes the player and its judge sentences them by the town's custom; the door card is what comes of it (mechanics §9, *Blows that fall on the player*).
- **Rank and reputation:** framed and stripped of rank change them through the move and the door's endings (mechanics §9).
- **The heart still matters.** She is in the routine phase, the ally plant and the crisis; only chapter 2's blow moves.
- **Once per world.** Each target + event is used once per world, the main Ordeal's (heart, event) included, and each target by 1–3 guild stories (rulebook, *World ledger*).

### Climax
**Job:** face the antagonist directly.
- The **antagonist** fills the opposing slot (§5).
- **The heart** fills the requester, wronged, or party slot. A captive heart fills *wronged*, and her fallback becomes the requester.
- **The climax pays off at least two plants from different layers,** and the revelation is one of them.
- **The climax wrong is personal harm to the heart** (debts inflated, a false dismissal, a betrayal). It is never just the taking of the stake, which keeps it within Reckoning's rules.
- If the want is *desperation* or *revenge*, the twist can't be "none." Prefer *Changed* or *Disproportionate demand*.
- **Family:** set by shape (§2). Reckoning is the guaranteed fallback.

### Return
**Job:** homecoming, changed.
- **Always a Restore card.**
- **Requester:** the heart, or her fallback if the heart's state still binds. Their requester reason is rolled from Restore's list.
- **Broken thing:** one of
  - the hook's harm, if fixable (works → Broken works; livelihood → Lost livelihood), using Restore's Return-only cause *deliberately broken* or *deliberately ruined* so it matches the record
  - the ordeal's cost: *disgraced* → Lost good name; *estranged* → Estrangement. The Estrangement must use a kin member no Betrayal, Dispute, or Restore door used.
- **Twist:** never "none." Weighted toward *Can't go back*, which needs a legacy or pride reason, and *One person pays*.
- The Return assumes the climax was reached. It never assumes which way the climax ended. In main stories, only its text varies by the climax ending (main-quest, *Wins stick*).

## 5. Opposing slot by family

| Family | Opposing slot | Limits for main use |
|---|---|---|
| Rescue | captor | allowed in: Hook (captive = heart's kin), Threshold, Trials, Ordeal; Climax as a non-turn card (main-quest §9) |
| Investigate | culprit (hidden) | allowed in: Hook, Trials |
| Dispute | the other party | the player must be the heart's side's advocate; money premises still follow the separation rule |
| Threat | raider leader / preserve owner / source owner or travelers' spokesperson / encroacher | Raiders is never valid with a High or Top antagonist in the leader slot. |
| Passage | pursuer, or checkpoint officer / route authority; for the *Unwelcome cargo* ordeal door, the waypoint headman | Smuggling and Fugitive, plus *Unwelcome cargo* as a Disgraced door. The Hand may pursue (Passage's main-quest exception). |
| Acquire | holder | allowed in: Trials; in the Ordeal, only as the *captive* door |
| Reckoning | wrongdoer | — |
| Restore | none | allowed in: Ordeal (Disgraced, Estranged) and Return only |
| Contest | the other principal | non-turn main cards only; faction stories in any phase |

## 6. Plants and payoffs
- **What a plant is:** a card's **declared** shared or unique fact (§4), or the hook's harm. It is written in the card's fixed fields, never in its truth, its ways, or its twist. Other public details on a card (a loan agreement, a charter) are ordinary facts, not plants.
- **Places aren't plants:** a later card may reuse a place from an earlier card (a camp, a keep) under the normal registry rules. A place doesn't need to be planted.
- **Plant rule:** a node may depend on a fact only if that fact was planted in **every node of some earlier layer**. The hook's harm counts as layer 1. Minor quests may also depend on their own chapter's unique plant (§8). This applies to main nodes, the Return, and minor quests.
- **Knowledge, not state:** payoffs depend on what is known, never on who holds something.
- **Revelation exception:** inside the diamond, the revelation layer publishes the hook's secret. This is the one controlled exception to "never expose another card's secret."
- **No ending-dependence:** no main card depends on how another card ends. A move's despite-you face and the Return's text variants change wording only, never the cards built on them (main-quest, *Wins stick*).
- **Plant kinds:** at most **one document plant per diamond.** The rest are about people, places, or non-document things.
- **Not a "use":** a plant about an entry doesn't count toward that entry's reuse cap.

## 7. Variety rules
- No two nodes in the same layer share a family.
- Two linked nodes never share a family.
  - **Exception, Ordeal doors:** every Ordeal node links to the climax (World C showed it with W=2), so this rule doesn't bar an Ordeal door from the climax's family. A Reckoning door (*Betrayal*) may stand under a Reckoning climax.
- **Threads:** for every edge, state the cast member, proxy, or plant it carries. A layer's shared fact works as the thread for every edge leaving that layer.
- **Twist groups:** follow rulebook §9: at most two per group, or one per 10 storyline cards, whichever is more; never two from one group in the same chapter or in linked chapters.
  - A storyline is the main quest, its Return, and its minor quests.
- **Twists go to turning cards first.** Turning cards (turn cards, the Ordeal, the climax, the Return) are never plain (rulebook, *No blank card*). Their twists are rolled before the rest, the climax and the Return first (rulebook, *Roll order*). The generation order (§9) is the order cards are written, and it stays as it is.

## 8. Relatedness scale and linked pairs

| Level | Name | Definition (set by the generation step) |
|---|---|---|
| **4** | Main | a diamond node or the Return. Carries a cast member or proxy. |
| **3** | Minor | attached to one main node and shares a person with it. It may depend on its chapter's unique plant. Declares one **advantage ending**: an entry or fact that will exist for a specific *later* main node. What the advantage *does* is mechanical-layer work. **The Return takes no minors** (nothing follows it). At least 1 in 3 minors has a twist (rulebook, *No blank card*). |
| **2** | Fallout | generated from one of the villain's moves; full rules in `quest-fallout.md` |
| **1** | Echo | reuses a **person** from any **story card**: a main node or a faction story card. Never a cast member (main or faction), a proxy, the stake, or any part of the stake. From a main story, prefers people from opening or middle cards, or from chapters in layers of width 2 or more. No ending may name a story card, and nothing may assume another card's hidden truth. |
| **0** | Independent | shares only the region |

- **Linked pairs** (side quests only): two cards generated together, in a declared order.
  - Card B's premise assumes card A was completed. B may build on A's truth, since both are generated together.
  - B never assumes how A ended.
  - This is a named exception in the rulebook (§3).

## 9. Generation order
1. **Spine:** shape → antagonist → want → stake → heart and tie → Hand → heart's fallback.
2. **Diamond:** set L and W, then assign roles.
3. **Hook:** harm type, revelation, hook card.
4. **Ordeal event.**
5. **Climax card.**
6. **Threshold and Trials:** escalation, threads, shared and unique plants, revelation layer.
7. **Ordeal layer.**
8. **Plant check:**
   - every dependency follows the plant rule
   - the climax pays off ≥2 plants, including the revelation
   - at most one document plant
9. **Return.**
10. **Minor quests:** 0–1 per main node (none on the Return), each with a declared advantage. Main stories also force some (heart-thread minors, the revelation's destination; main-quest §5, §7).
11. **Fallout quests** (from the moves; see `quest-fallout.md`), then **echoes, independents, linked pairs.**
12. **Background pass.**

## 10. Parameters
*For world generation, the location quotas in `quest-locations.md` §3 replace the echo, independent and linked-pair counts below.*


| Setting | Default |
|---|---|
| L / W | 5 / 3 (main stories: 6 / 3, main-quest §3) |
| Minor-quest chance per main node | 50% |
| Echo quests | 3 |
| Independent quests | 3 |
| Linked pairs | 1 |
| Pre-made Top roster | 4 |

## 11. Sample: L=5, W=3 (1-2-3-2-1 + Return), shape *Beside*

### Spine
- **Antagonist:** Brannock Hale, a moneylender (Middle)
- **Want:** ambition
- **Stake:** the Saltmarsh salt pans (place)
- **Heart:** Odile, a salt-boiler (Low). Tie: she works the pans.
- **Hand:** Corm, Hale's debt collector (Middle, employee)
- **Heart's fallback:** Odile's brother Tam (Low, sibling)

### Main nodes

**Layer 1 — Hook**
- **Card:** Investigate, *Sabotage*. The pans' sluice gates keep breaking.
  - Requester: Odile. Red herrings: two boilers who argued with the guild.
  - **Harm (public):** *the pans' works are broken.*
  - Truth: Corm did it. He gains a rival link to Odile.
  - **Revelation (secret):** *Hale ordered the gates cut so the boilers would borrow from him and default.*
- **Cast presence:** Odile (requester); Corm (hidden culprit)
- **Plants:** layer-1 plant = the harm

**Layer 2 — Threshold**
- **Card A:** Passage, *Fugitive*. Wenna, an indebted boiler (Low, debtor to Hale, so a proxy), flees Corm. Reason: runaway debtor.
  - Origin: the boilers' row houses (not the hook's place).
  - Twist: *The one left behind*. Her daughter is left at the row houses.
- **Cast presence:** Corm (pursuer, the creditor's employee; Hand exception); Wenna (proxy)
- **Card B:** Dispute, *Broken agreement*.
  - Parties: the guild, through its elder (Middle; reason: dependents), vs. its treasurer Ferris (Middle; reason: obligation, since he owes Hale).
  - The guild says Ferris broke his duty by signing a loan with Hale without the council's consent. Ferris says the council agreed.
  - Employer/employee link, so separation is exempt. The player advocates for the guild. Arbiter: absent. Twist: none.
- **Cast presence:** Ferris (proxy, a debtor to Hale)
- **Plants (both cards):** shared: *Hale holds most of the boilers' debts*

**Layer 3 — Trials (revelation layer)**
- **Card A:** Acquire, *Won't sell*. A retired engineer (Middle; reason: sentiment) holds the original gate plans. Requester: Odile (reason: legacy). Truth names Corm.
  - Unique plant: *the gate plans survive* (the diamond's one document plant).
  - Shared plant: **revelation**, attested by the engineer.
- **Cast presence:** Odile (requester); Corm (named in truth)
- **Card B:** Investigate, *Theft*. The guild's savings box is stolen. Requester: the guild elder.
  - Culprit: a clerk paid by Corm (employer/employee, so a proxy), named in the truth.
  - Red herring: the night watchman (seen near the scene).
  - Unique plant: *the box vanished the night the gates were inspected*.
  - Shared plant: revelation, attested by the night watchman.
- **Cast presence:** the clerk (proxy)
- **Card C:** Threat, *Raiders*. Toughs burn the boilers' drying sheds. Requester: the drying-yard foreman (new).
  - Why-here: *paid*; patron: Corm. Leader: Garr (lawless, Middle; Corm's employee). Hideout: the old salt store.
  - Ways rolled include "turn a lieutenant," which fills the lieutenant slot.
  - Unique plant: *Garr's crew answers to Corm*.
  - Shared plant: revelation, attested by the lieutenant.
- **Cast presence:** Corm (patron slot); Garr (proxy)

**Layer 4 — Ordeal**
- **Event: captive** (public). *Odile is held at the old salt store, on Hale's orders, as leverage.*
- **Card A:** Rescue, *Kidnapped*. Captor: Garr (lawless, hideout = the old salt store, a reused place). Requester: Odile's mother (new).
- **Cast presence:** Garr (proxy); Odile (captive, a passive slot)
- **Card B:** Acquire, *Won't sell*. Hale demands the guild's pan charter in exchange for Odile.
  - The guild elder holds the charter and refuses (reason: security).
  - Requester: Tam (reason: obligation; Tam owes Hale, a creditor/debtor link).
- **Cast presence:** Tam (fallback, in the heart's place)
- **Plants (both cards):** shared: the ordeal event

**Layer 5 — Climax**
- **Card:** Reckoning, *Swindle*. The debts Hale bought were inflated.
  - Wronged: Odile (a passive slot, since she's still captive). Requester: Tam (fallback).
  - Shield: *Bought*. Arbiter: the town magistrate (High, minor gentry; owes Hale, a creditor/debtor link).
- **Cast presence:** Hale (wrongdoer); Odile (wronged)
- **Payoffs:** *Hale holds the debts* (layer 2) + **revelation** (layer 3)

**Return**
- **Card:** Restore, *Broken works*. Rebuild the sluice gates.
  - Cause: *deliberately broken*, which matches the hook's record.
  - Requester: Tam (fallback; reason: legacy, since their grandfather built the gates).
  - Twist: *Can't go back*. Tam insists on the old design, but only a new one will hold.
- **Cast presence:** Tam
- **Plants:** relies on the harm (layer 1)

### Edges and threads
| Edge | Thread |
|---|---|
| L1 Investigate → L2 Passage, L2 Dispute | the harm (layer-1 plant) |
| L2 Passage → L3 Acquire, L3 Threat | *Hale holds the debts* (layer-2 shared) |
| L2 Dispute → L3 Investigate | the same |
| L3 Acquire → L4 Rescue; L3 Threat → L4 Rescue | revelation (layer-3 shared); Garr also links Threat → Rescue |
| L3 Investigate → L4 Acquire | revelation |
| L4 Rescue, L4 Acquire → L5 Reckoning | the ordeal event (layer-4 shared) |
| L5 → Return | the harm |

- No linked pair shares a family.
- Every node has at least one edge in and one edge out.

### Minor and side quests
- **Minor (on Trials A):** Restore, *Estrangement*.
  - The engineer and his son haven't spoken since the son sided with Hale in the guild's quarrel (cause: *a side taken*). The son is Hale's clerk, which makes him a proxy, and that's allowed in minor quests.
  - Deadline: the engineer leaves the Saltmarsh at season's end.
  - Neither is an acknowledged wrongdoer.
  - **Advantage ending:** *the son is known to be willing to testify at the climax*.
- **Echo:** Restore, *Lost good name*.
  - The night watchman from Trials B is shunned as the suspected thief. Truth: *false*, his own hidden truth.
  - He's off the most direct path and isn't a proxy.
- **Independent:** Threat, *Protected beast*, in the next valley.
- **Linked pair:** Investigate, *Disappearance* (a ferry girl is missing; truth: smugglers took her) → Rescue, *Kidnapped*. Unrelated to the main quest.

### Checks
- **Escalation:**
  - Hook: Corm, hidden.
  - Threshold: Corm openly.
  - Trials: Corm's men and methods.
  - Ordeal: Hale named as the *cause of harm* for the first time. Earlier plants named him only as a creditor.
  - Climax: Hale himself.
- **Cast per chapter:** Corm fills one slot in layers 1–3. Odile fills one slot in layers 1, 3, 4, and 5.
- **Reuse counts (non-cast):**
  - The guild elder: 3 cards (Dispute, Investigate, Acquire). That's at the cap, each in a different family.
  - Garr: 2 (Threat, Rescue).
  - The night watchman: 2 (Investigate, Restore).
  - Tam fills only fallback slots, which are exempt.
- **Twist groups:**
  - *The one left behind* (innocents pay)
  - *Can't go back* (they don't want it)
  - No group is used twice.
- **Ordeal doors:** breaking her out vs. persuading the guild to give up its charter. Two different quests.

## 12. Changes from review
- **v3.7 (2026-10-27, v6: World C fixes):**
  - §7: an Ordeal door may share the climax's family, so a Reckoning door is allowed under a Reckoning climax (CM3).
  - §7: pointer to rulebook *No blank card* and *Roll order*: turning cards are never plain and get their twists first; §9's writing order is unchanged (SC-C3, CM2).
  - §8: the Return takes no minors; at least 1 in 3 minors has a twist (CM6, SC-C3).
  - v6d: coherence joins (27 Oct, from the v6c review): §4 *Doors by how the blow fell*: the plan deals a row (held by men, for a debt or on a charge; wounded by men or through something rigged; a hall taken, occupied, sealed or shut by order), which sets the blow's means, its doors and their slot fills; *Unwelcome cargo* is the main Ordeal's only; no step repeats the one before; a captive heart never gives the crisis or fills a *herself* crisis.
  - v6b: Laz's guild changes (27 Oct): §4 *Guild chapter-2 doors*: door rows for the player targets (framed, stripped of rank, arrested) and the guild targets (hall seized, members taken, outlawed, leader struck down), with premises; where a family's rules didn't fit, the door changed, and why; the player is never cast; arrests run in the sim; each target + event once per world.
- **v3.6 (2026-10-26, audit fixes):**
  - The twist cap follows rulebook §9 (one per 10 storyline cards when that is more) (core M9).
  - The sample's *Can't go back* is filed under *They don't want it* (core M7).
  - The override banner names main-quest's default diamond size; the heart's fallback is chosen by casting (core L9, H6).
- **v3.5 (2026-10-26, v5: world-test fixes):**
  - Echoes may come from any story card, main or faction (C3).
  - Pointers to main-quest's new rules: revelation on a fitting card, *Wins stick* (move faces, Return variants), forced minor quests.
- **v2 (creative and logic reviews):**
  - Added antagonist shapes, want, the Hand, escalation, and the revelation.
  - Role rules are now hard.
  - One ordeal event per story.
  - Return variety.
  - Public plants.
  - Cast proxies.
  - Diamond math.
  - Opposing-slot table.
  - Ordering clause.
  - Minor, echo, and linked-pair definitions.
- **v3 (final audit):**
  - **Hook:** narrowed to harms the Return can repair and families that can carry the antagonist's intent. Rescue dropped as a hook.
  - **Return:** uses Restore's new *deliberately broken/ruined* causes.
  - **Ordeal doors:** rebuilt so siblings are different quests and Investigate is gone from the Ordeal.
  - **Climax:** the heart or her fallback, with slots matched to the family. The climax wrong is personal.
  - **Revelation plants:** each names a knower.
  - **Proxies:** defined.
  - **Groups:** represented by one named person.
  - **Short diamonds:** the shared-fact rule now applies to them too.
  - **Sample:** rebuilt so every card passes its own family's rules.
- **v3.3 (full test and final audit):**
  - "Per layer" wording removed.
  - The lieutenant is accepted in Trials.
  - Bond and kin-sick harms are classed as unfixable.
  - The disgrace cause wording is unified.
  - The minor-quest plant exception is cited.
  - The Threat opposing slot is completed.
- **v3.2:** the hook is open to seven families, and the secret intent lives on the Hand or proxy. A repair guarantee was added for the Return.
- **v3.1 (verification pass):**
  - Beside's climax is Reckoning *Swindle* only. Dispute can't hold a known wrongdoer.
  - A captive heart fills passive slots, so she is the climax's *wronged*.
  - Ordeal doors use different kin members, and the Disgraced door has its own Restore cause.
  - Threat-hook revelations are stored on the patron.
  - "Plant" is limited to declared facts. Places are reused freely.
  - Every edge states its thread.
  - The sample was corrected again.
