# Family: Passage — Locked (v3.3)

## 1. Situation
**Someone or something has to get from here to there, and the hard part is the cargo itself, not just the road.**
- **vs. Rescue:** the cargo is already in hand.
- **vs. Acquire:** the player is entrusted with the cargo, not taking it.
- **vs. Threat:** the danger follows the cargo, not a community.
- **Anti-escort rule:** every premise makes the *cargo* the problem: it's forbidden, hunted, unwilling, or unwelcome. A cargo that just needs carrying past danger is a failure.

## 2. Card fields
- **Requester (sender)**, with a **reason** (rulebook §7, the shared list; v7): *love*, *obligation*, *duty*, *money*, *freedom* or *faith*. Every card fills `reasons.requester`.
- **Cargo:** a person or a thing.
  - A person cargo must not be held or dead on another card.
- **Recipient:** optional. A fugitive may only need a haven. In that case the **haven owner** stands in as the recipient for twists.
- **Origin and destination:** places. A reused place keeps its existing owner.
- **Route:** a place of kind *road, pass, ford, river, or wilds*. **Sea routes are parked** while the sea is off-limits to the squad: a coastal Passage goes by the shore road (rulebook, *Entry specs*).
- **Waypoints:** 0–2 places along the route. A waypoint gets a named headman whenever a way or twist needs one. Waypoints count toward the reuse cap.
- **Obstacle:** who or what stands in the way.
- **Optional slots:**
  - pursuer
  - **route authority** (Smuggling: whoever issued the ban)
  - **checkpoint officer** (Smuggling: the Middle-standing officer at the gate)
  - guide
  - the one left behind (twist)
  - any other person a way names (haven owner, host, outlying farmer, the trusted carrier, someone they love). These are filled only if that way is rolled (rulebook §2.4).
- **Why they flee / why they resist:** for person cargo (§5).
- **Truth:** rolled at generation from §5b's truth table.
- **Pressure:** a deadline or the cost of delay.
- **Premise, ways through, twist, possible endings.**

## 3. Kinds of ways through
- **Route:** find or choose a path. Every route way names a person who holds the path, and the price they ask.
- **Cover:** disguise, hide, or decoy.
- **Force:** fight through or outrun.
- **Arrangement:** bribe, negotiate, or call in a favor.

## 4. Premises

| Premise | Requirements | Ways through (roll 2–3, ≥2 kinds) | Separation |
|---|---|---|---|
| **Smuggling:** the cargo is forbidden where it's going | a route authority with legal power over the route; a guarded checkpoint place with a checkpoint officer | hide it inside ordinary goods (cover); bribe the checkpoint officer (arrangement; removed if the officer is Top); find the one person who still knows the smugglers' path and pay their price (route; fills guide); stage a distraction at the gate (cover); have it carried by someone the guards won't search (arrangement) | exempt (crime) |
| **Fugitive:** a person is being hunted and must reach safety | person cargo; a pursuer linked to them (master, creditor or the creditor's employee, grudge, or someone they witnessed). The pursuer is never another card's hidden culprit. **Main-quest exception:** the Hand may pursue, as long as this card never refers to the hidden act | disguise them (cover); lay a false trail (cover); reach land where the pursuer has no authority, if its owner will take them in (route; the haven owner is the person); hire guards or fight off the hunters (force); buy the pursuer off, or trade them something they want more (arrangement; removed if the pursuer is Top) | exempt (crime and authority) |
| **Unwilling passenger:** the person being moved doesn't want to go | person cargo with a reason to resist; the requester has a family or kin link to them and is not outranked by them, unless the requester is their parent | earn their trust on the road (arrangement); keep them under close escort (force; they are escorted, not held); let them pick the route and the pace (route; they are the person, and the price is time); bring along someone they love (arrangement) | exempt (authority) |
| **Unwelcome cargo:** the cargo brings danger or disgrace to anyone who lets it in (a sick person going to a healer, grain from a quarantined district, someone under a lord's ban whose hosts would be punished) | a person cargo with a sickness or ban fact, *or* goods from a place with a sickness fact; 1–2 waypoints, each with a headman who can bar the road | find a host willing to take the risk, and learn what it costs them (arrangement); go around every settlement, at the cost of time the cargo may not have (route; a waypoint's outlying farmer knows the way); hide what the cargo is, risking the hosts without telling them (cover); force the barricade (force); win a headman over with something they need (arrangement) | n/a |

## 5. Why they flee / why they resist
| Reason | Fits |
|---|---|
| Witnessed something | Fugitive |
| Escaped servant or apprentice | Fugitive (employer/employee link) |
| Deserter | Fugitive (soldier) |
| Runaway debtor | Fugitive (the pursuer is the creditor, or the creditor's employee) |
| Accused, rightly or wrongly | Fugitive |
| Fears what waits at the destination | Unwilling |
| Has someone they'd be leaving behind | Unwilling |
| Expects punishment on arrival | Unwilling |
| Wants to choose their own path | Unwilling |

## 5b. Truth
A secret fact; each value is a truth row in the world ledger, with its shape (rulebook, *Shapes*).

| Truth | What it is | Reached by | Shape |
|---|---|---|---|
| *As declared* | the cargo is what the sender said, and the trouble is as it looks | a way that handles the cargo closely (hide it inside goods, disguise them, earn their trust on the road) | *(confirming)* |
| *Not what it seems* | the cargo carries more than was said (a second letter, a debt, a reason to flee they never gave); names what | the same close ways; with person cargo, earn their trust on the road or let them pick the route | `more-than-said` |
| *The pursuer was told* | someone at the origin informed whoever stands in the way (the pursuer, the checkpoint, a headman); names who | a way that deals with them directly (bribe the checkpoint officer, buy the pursuer off, win a headman over) | `sold-from-inside` |

## 6. Twists
- **None.**
- **Not wanted at the end:** the recipient or haven owner refuses the cargo. Taking it would implicate them, or they've changed their mind. The player now holds something nobody will take.
  - *Requires:* a recipient or haven owner.
- **The pursuer has a point:** whoever is trying to stop the cargo is in the right, because the cargo really is dangerous.
  - *Requires:* a route authority, checkpoint officer, or waypoint headman with a legitimate reason.
  - Not used on Fugitive. "The fugitive really did it" duplicates Rescue's *justified captor*.
- **Need along the way:** a waypoint needs the cargo as badly as the recipient does.
  - *Requires:* a useful cargo (medicine, food, coin, goods) and a waypoint.
- **The one left behind:** when the cargo leaves, a named person at the origin pays for it. The fugitive's sister may be held in their place, or the porter who signed for the goods may take the blame.
  - *Requires:* a named Low/Middle person at the origin with a family or employer link to the cargo.

- **Tells** (rulebook rule 8, v7): a card whose truth is *not what it seems* or *the pursuer was told* names one public fact that doesn't fit the told story in `details.tell` (the cargo weighs more than it should; the checkpoint already knows the cart). *The pursuer has a point*, *Not wanted at the end*, *Need along the way* and *The one left behind* show theirs in the pitch, the place, or every rolled way.

| Twist | Smuggling | Fugitive | Unwilling | Unwelcome |
|---|---|---|---|---|
| Not wanted at the end | ✓ | ✓ | ✓ | ✓ |
| Pursuer has a point | ✓ | ✗ (duplicates Rescue) | ✗ | ✓ (the headman is right to fear it) |
| Need along the way | ✓ | ✗ | ✗ | ✓ (goods only) |
| The one left behind | ✓ | ✓ | ✓ (not "someone they'd be leaving behind") | ✓ |

## 7. Possible endings
Each with its default **`costs`**, who loses by it (rulebook rule 7, v7). What failure endings write is mechanics §4.
- Delivered (costs: the obstacle, or the pursuer; the one ending that costs only the other side. With *The one left behind*, that person too)
- Delivered late, damaged, or partly (costs: the recipient)
- Seized or lost (costs: the requester)
- Kept or redirected by the player (costs: the requester)
- Handed to the pursuer (costs: the cargo slot: a person cargo, or the sender's goods)
- Released (unwilling passenger) (costs: the requester)

## 8. Standing effects
- **Bribes and paid passage need funds.** With a Low requester, the cost shifts to the player.
- **Buying off the pursuer** is removed if the pursuer is Top.
- **Unwilling passenger:** the requester has a family or kin link and is not outranked by the passenger, unless the requester is the parent.

## 9. Registry interactions
- **Creates:** cargo, route, waypoints and their headmen, checkpoint and its officer, guide, pursuer, recipient, the one left behind.
- **Places are the best reuse targets.** They connect quests across the map.
- **A Passage truth may not create a new claimant to another card's stake.** It also may not depend on how another quest ends. Such links go to assembly as **linked pairs** (for example, a ransom run for a Rescue, or a medicine run that answers a Threat).
- **Always roll a fallback** for a reused person cargo, pursuer, or recipient.

## 10. Test rolls (v2)

**Roll 1: Fugitive + The one left behind**
- **Cargo:** Sela, a scullery maid (Low), fleeing Lady Osric's house (Osric's 2nd quest). Reason: accused.
- **Truth:** *not what it seems*: she did take a ring, for her mother's medicine.
- **Left behind:** her younger brother, a pot-boy in the same house.
- **Ways rolled:**
  - disguise her (cover)
  - reach the free town if its reeve will take her (route)
  - buy off the steward (arrangement)
- **Holds.**

**Roll 2: Smuggling + Need along the way**
- **Cargo:** fever medicine, headed for a town under a lord's ban.
- **Waypoint:** the sick market town from Threat Roll 5 (reused, its 2nd quest).
- **Ways rolled:**
  - hide it in grain (cover)
  - pay the old smuggler's price (route)
- **Holds.** Answering the Threat card's sickness is logged as a candidate **linked pair**.

**Roll 3: Unwilling passenger + Not wanted at the end**
- **Cargo:** a young journeyman (Middle), summoned home by his mother (Middle; parent). Reason: wants to choose his own path.
- **Truth:** *not what it seems*: his mother's new husband has said he won't have him in the house.
- **Ways rolled:**
  - let him pick the route and pace (route)
  - earn his trust (arrangement)
- **Holds.** The v1 version, an uncle claiming Dispute's stake, is now blocked by the no-new-claimant rule.

**Roll 4: Unwelcome cargo + Pursuer has a point**
- **Cargo:** a coughing child (Low) being taken to a healer two valleys over.
- **Waypoint:** a hamlet whose headman bars the road.
- **Truth:** *as declared*: the cough is catching.
- **Ways rolled:**
  - go around every settlement (route; the outlying farmer knows the way)
  - find a willing host (arrangement)
  - force the barricade (force)
- **Holds.** The headman is right, and the child still needs the healer.

**Roll 5: Fugitive, runaway debtor, with a Top creditor**
- **Cargo:** a ferryman (Low) who owes the governor's tax farm.
- **Pursuer:** the governor's bailiff (Middle), acting for the governor (Top).
- **Effect:** "buy off the pursuer" is checked against the bailiff, not the governor. The bailiff is the pursuer slot, so the way stays.
- **Ways rolled:**
  - false trail (cover)
  - buy off the bailiff (arrangement)
- **Holds.**

**Roll 6: Unwilling passenger, High party sympathetic**
- **Requester:** Lady Harrowgate (High), the cargo's parent. She's the sympathetic side here: her estate's tenants need an heir present to keep their leases under the law.
- **Cargo:** her son (High), who wants to choose his own path.
- **Ways rolled:**
  - let him pick the route and pace (route)
  - bring along someone he loves (arrangement)
- **Holds.** The High requester has the sympathetic reason.

## 11. Changes from review
- **Magnet cargo → Unwelcome cargo.** Magnet was an escort quest where the danger was the road, not the cargo.
- **"Wrong hands" → Not wanted at the end.** "Wrong hands" was a stock trap twist.
- **"You're the decoy" → The one left behind.** The decoy twist made the player's choices pointless.
- **Every route way names a person and a price.** They no longer read as one way under three names.
- **New card fields:** truth, waypoints, checkpoint officer, and guide.
- **Added the Runaway debtor reason.**
- **Tightened the authority-to-send rule.**
- **No cross-card claimants:** a Passage card can't create a claimant to another card's stake, or depend on how another card ends.
- **Pursuer has a point no longer applies to Fugitive.** A cross-family review found it duplicated Rescue's *justified captor*.
- **v3 (final audit):**
  - The route authority and the checkpoint officer are now separate slots.
  - A creditor's employee can pursue.
  - Top filter added on bribes.
  - "Under guard" is now an escort, not a held state.
  - High-sympathetic roll added.

## 12. Changelog
- **2026-10-27 (v3.3), v7: Laz-approved book rules (A2, A4, A8).** Default `costs` on every ending (A4); requester reasons from the shared list (A8); tells (A2). *Go and look* went to Threat, not here: a scout carries no cargo, so it would break the anti-escort rule (rulebook §8, C1).
- **2026-10-27 (v3.2), v6: World C fixes.** Each truth row carries its shape, and *as declared* is marked *(confirming)* (§5b; rulebook, *Shapes*); sea routes parked until Laz opens the sea, so a coastal Passage goes by the shore road (§2).
- **2026-10-26 (v3.1), audit fixes.** A truth table (§5b: *as declared*, *not what it seems*, *the pursuer was told*), each row naming the ways that reach it; test rolls tagged with their row.
