# Family: Acquire — Locked (v3.3)

## 1. Situation
**The requester wants a specific thing that someone else holds. Getting it means facing why the holder has it and why they won't let it go.**
- **vs. Investigate:** the holder is known from the start.
- **vs. Rescue:** the target is a thing, not a person.
- **vs. Passage:** the player has to obtain the thing; nobody entrusts it to them.
- **vs. Dispute:** nobody asks the player to judge between claims.
- **vs. Reckoning:** Acquire is about a **specific thing that still exists**. Money or standing that's already gone belongs to Reckoning.
- ***Take the readings*** (v7): the thing wanted is a **record made on site** (a survey, a rubbing, a set of soundings, a copy of an inscription) from ground or a fixed thing someone else keeps. The keeper keeps the original; the obstacle is why they won't let anyone measure.
  - **vs. Restore:** nothing need be broken; a survey that mends a broken boundary is still taken here, then used by a Restore card *(Claude)*.

## 2. Card fields
- **Requester**
- **Target:** a thing (heirloom, document, weapon, relic, goods, tool).
- **Holder:** the person who has it.
- **Holding place:** where it's kept, with its access rating.
- **Requester's reason** and **holder's reason** (§5).
- **Seal type:** Under seal only (§5).
- **Truth:** the thing's real history, rolled from §5's truth table.
- **Optional slots:**
  - **authority:** someone who outranks the keeper
  - **insider:** someone inside the holding place
  - **original taker:** Passed on only
- **Premise, ways through, twist, possible endings.**
- ***Take the readings*** (v7) *(Claude)*:
  - **the source** (`details.read_from`): a place, or a thing that can't be moved (a boundary stone, a trench face, a seam, an inscribed wall), with its access;
  - **the keeper** stands in the holder's slot: whoever controls the ground (a landholder, a shrine-keeper, a mine captain, a rival dig's master);
  - **the readings** are the target: a *document* thing the card creates, `held_by` none until made. Making them on site is not "retrieving a document" (§9);
  - **the instrument** (`details.instrument`): what the readings need, a chain and staff, a plumb line, a rubbing cloth, a sounding line, or a person who can read the old hand (an optional slot).

## 3. Kinds of ways through
- **Take:** steal it.
- **Deceive:** con, swap, or mislead.
- **Trade:** buy, barter, or offer surety.
- **Earn:** win it fairly.

## 4. Premises

| Premise | Requirements | Ways through (roll 2–3, ≥2 kinds) | Separation |
|---|---|---|---|
| **Won't sell:** the holder owns it fairly and refuses to part with it | holder owns the thing | give them what the thing does for them some other way: other collateral, a better tool, a different keepsake (earn); borrow it for the one use, under the holder's eye (trade); convince them that keeping it puts them at risk, true or not (deceive); become the person they'd trust it to (earn); take it (take) | exempt: acquisition across classes (rulebook §5 change) |
| **Passed on:** it was taken wrongly at some point before this card, and the current holder got it in good faith (bought it, inherited it, took it as payment) | the old taking is a recorded past-event fact; the original taker is dead (new) or a registry person | trace it back through the people who held it until the holder sees where it came from (earn); repay the holder what it cost them (trade); make the original taker repay the holder (earn; requires the taker alive); swap it (deceive); take it (take) | exempt |
| **Under seal:** an authority holds it under a rule | holder holds legal authority; seal type rolled; holding place is guarded or fortified | meet the condition (earn; when the condition is a fine, paying it counts as trade); offer a surety: something of equal weight held in its place, or the requester works off the term (trade); take it while it's being moved to a hearing or a sale (take); have the authority slot order it released (earn; removed if the keeper is Top); swap it with an insider's help (deceive; fills insider) | exempt (authority) |
| **Unknown worth:** the holder has it but doesn't know what it is | holder is *unaware* (a state, not a reason; the holder also rolls a real reason); thing's value is at least 1 tier above the holder's standing | buy it cheap and say nothing (trade); tell them what it is and offer a fair price (trade); tell them, and help them sell it to the right buyer for a share (earn); swap it for something they'd value more (deceive); take it while they don't care (take) | exempt |
| ***Take the readings*** (v7, C1): field work with an instrument, a survey or a copy made on site *(Claude)* | a source with a keeper who controls access; the instrument is on the card (lent by the requester, or an optional slot who can use it); the keeper rolls a holder's reason (§5: livelihood, sentiment, security, duty or pride) | take the readings at night without leave, with the instrument (take; *removed if the source is fortified*); come as someone allowed: the keeper's own surveyor, a pilgrim, a buyer looking the land over (deceive); pay the keeper for a day on the ground, and let them watch every mark (trade); offer the keeper a fair copy, so the readings serve both (trade); do the work the keeper needs first: mend the wall the stone stands in, read their own old deed for them (earn) | exempt (acquisition) |

## 5. Lists

**Requester's reason** (rows of the shared list, rulebook §7; v7: *curiosity*, *money* and *duty* added; collecting is *pride*)
| Reason | Requires |
|---|---|
| Legacy: it was the family's | family link to a past owner |
| Survival | Low or Middle |
| Evidence: it proves something | a fact the thing could prove |
| Obligation: they owe it to someone | creditor/debtor link to a third person |
| Pride (or collecting) | Middle+ |
| Greed | any |
| Curiosity | any; leans to *Take the readings* and *Unknown worth* |
| Money: it is their trade (a dealer, a surveyor paid by the job) | any |
| Duty: an office or a master sends them | holds an office, or is the employee of someone who does |

**Holder's reason**
| Reason | Requires |
|---|---|
| Livelihood | thing kind: tool or goods |
| Sentiment | any |
| Security: it protects them | any |
| Duty | Under seal, or the keeper of a *Take the readings* source |
| Pride | Middle+ |

**Sympathy rule:** at least one of requester and holder has a sympathetic reason (legacy, survival, evidence, obligation, livelihood, sentiment, security, duty). *Curiosity* and *money* are not sympathetic on their own.

**Truth** (a secret fact; each value is a truth row in the world ledger, with its shape from rulebook *Shapes*)

| Truth | What it is | Reached by | Shape |
|---|---|---|---|
| *As told* | the thing and its history are as the requester and holder say | any way that deals with the holder openly (borrow it, repay them, meet the condition) | *(confirming)* |
| *Promised elsewhere* | the holder has already pledged it to someone; names who | a trade or earn way with the holder (borrow it, offer a surety, become the person they'd trust it to): they say why they can't | `another-buyer` |
| *A copy* | the thing held is not the original; names where that is | a way that handles the thing (swap it, trace it back through its holders, tell them what it is) | `a-copy` |

**Take the readings** (v7, C1) *(Claude)*. The row is **what the ground really shows**. Each non-confirming row shows a tell on the card's face before the choice (rulebook rule 8); the column is the pattern, the story layer words it.

| Truth | What it is | Reached by | Shape | Tell (pattern) |
|---|---|---|---|---|
| *As told* | the ground reads as the requester expects (the family's row) | any way that takes the readings openly (pay for a day, do the keeper's work) | *(confirming)* | — |
| *Promised elsewhere* | the keeper has already promised the ground, or the readings, to someone else; names who (the family's row) | a trade or earn way with the keeper: they say why they can't | `another-buyer` | the keeper's best room made ready for a guest |
| *Already taken* | someone else took these readings first, and has them; names who | a way with the keeper, or coming as someone allowed: they mention the last one | `a-copy` | fresh stake holes beside the old ones |
| *The marks were moved* | the stone or the line was shifted, and the readings show it | any way that actually takes the readings | `damning-record` | the turf round the stone is younger than the turf beside it |
| *The ground moved* | the land itself has shifted (a slip, a river's new bed), so the old marks no longer hold | any way that actually takes the readings | `ground-moved` | the lower well has gone dry, and the upper field is wet |

- **Never closed.** Acquire has only two non-confirming rows on its first four premises. When both are full, each gains one (rulebook, *World ledger*), so an Acquire card never goes without a truth.
- *Promised elsewhere* shares `another-buyer` with Rescue's *a better buyer*. When the one it's pledged to is a High or Top person seen only in the truth, they fill that role on one card per world (rulebook, *Background people in truths*).

**Seal types (Under seal)**
| Seal type | Note |
|---|---|
| Fine unpaid | "meet the condition" means paying it |
| Ownership unproven | the condition is proving it's yours |
| Held as surety for someone else's debt | links to a debtor (created or reused) |
| Held until a ruling | the condition is a hearing |

## 6. Twists
- **None.**
- **Load-bearing:** the thing protects many people beyond the holder: a deed that shields tenants, a charter, a seal that vouches for others' goods. *Requires:* thing kind is document or relic, plus a community place it protects.
- **Wants to be rid of it:** the holder secretly wants it gone but can't be seen giving it up (a promise to a dead parent, family pride, a duty they're tired of). The refusal is for show. *Requires:* holder reason is sentiment, duty, or pride. On *Take the readings*: the keeper wants the ground measured, and can't be seen to allow it.
- **Meant for destruction:** the requester wants it gone, not owned. *Requires:* requester reason is evidence or legacy.
- **Rightful keeper:** the holder has the better claim, and the requester's story is one-sided. *Requires:* requester reason is legacy, evidence, or obligation.

| Twist | Won't sell | Passed on | Under seal | Unknown worth | Take the readings |
|---|---|---|---|---|---|
| Load-bearing | ✓ | ✓ | ✗ | ✓ | ✓ (the unmeasured line shields a community; the readings would settle it against them) |
| Wants to be rid of it | ✓ | ✓ | ✓ (a tired keeper) | ✗ | ✓ |
| Meant for destruction | ✓ | ✓ | ✓ | ✓ | ✓ (the requester wants the readings to bury them) |
| Rightful keeper | ✓ | ✗ (the old wrong is fact) | ✓ | ✗ | ✓ (sacred or rightly kept ground) |

- **Tells** (rulebook rule 8, v7): *Wants to be rid of it* (*Not what it seems*) and *Meant for destruction* (*Someone's hiding something*) each name one public fact that doesn't fit the told story in `details.tell`. *Load-bearing* and *Rightful keeper* show theirs in the pitch, the place, or every rolled way.

## 7. Possible endings
Each with its default **`costs`**, who loses by it (rulebook rule 7, v7). What failure endings write is mechanics §4. On *Take the readings*, "obtained" means the readings are made, and "left with the holder" means they never are.
- Obtained and handed over (costs: the holder; the one ending that costs only the other side)
- Obtained and kept or redirected by the player (costs: the requester)
- Destroyed (costs: the requester; under *Meant for destruction*, the holder, so the card checks rule 7 again)
- Left with the holder (costs: the requester)
- Holder fairly compensated (costs: the requester, who pays)
## 8. Standing effects
- **A fortified holding place** removes "take it" ways that require entering the place. Taking it in transit is still possible.
- **Trade ways need funds.** With a Low requester, the cost shifts to the player.
- **Ordering a release** needs an authority slot that outranks the keeper. That way is removed if the keeper is Top.

## 9. Registry interactions
- **Creates:** the target, holder, holding place, authority, insider, original taker, and the community a Load-bearing thing protects.
- **Reuse targets:** things and holders from other families.
- **Documents as targets:** a target that happens to be a document does not count toward the "retrieve a document" limit. That limit applies to ways through. *Take the readings*' readings are such a target, made on site.
- **Readings are on the record** once made: a registry thing, `held_by` whoever ends up with them, reused by any later card that needs them (rulebook, *Things are on the record*).

## 10. Test rolls
*(Illustrative only. They don't share a registry with other families' rolls.)*

**Roll 1: Won't sell + Wants to be rid of it (High holder, sympathetic)**
- **Holder:** Lord Fennimore (High), reason: sentiment. He keeps his late father's war sword, and secretly hates it.
- **Requester:** a smith's guild (Middle). Reason: legacy; the guild forged the sword.
- **Ways rolled:**
  - become the person he'd trust it to (earn)
  - borrow it for the guild's anniversary (trade)
- **Holds.** The High party is the sympathetic one.

**Roll 2: Passed on + Load-bearing**
- **Holder:** a village headwoman (Middle). She holds a land charter her mother bought in good faith.
- **Past event:** the charter was stolen from a fishing hamlet 40 years ago. The original taker is dead (new). **Truth:** *as told*.
- **Load-bearing:** the charter shields the village's tenants from their lord's rent rises.
- **Ways rolled:**
  - trace it back through its holders (earn)
  - repay the holder what it cost (trade)
- **Holds.** Both sides of the charter protect ordinary people.

**Roll 3: Under seal (Top keeper, fortified), worst case**
- **Holder:** the governor (Top). A relic is held "until a ruling" in a fortified vault.
- **Effect:** ordering release is removed (Top keeper), and breaking in is removed (fortified).
- **Ways left:**
  - meet the condition (earn)
  - surety (trade)
  - take it in transit to the hearing (take)
  - swap via an insider (deceive)
- **Holds.** v1 left only 1 way here.

**Roll 4: Unknown worth**
- **Holder:** Old Bess (Low). She keeps seed in an iron strongbox that is actually a guild's lost strongroom box. Her real reason: livelihood.
- **Requester:** a collector (Middle). Reason: greed.
- **Ways rolled:**
  - buy it cheap and say nothing (trade)
  - help her sell it for a share (earn)
- **Holds.** The sympathy rule passes through Bess's livelihood.

**Roll 5: Under seal + Rightful keeper**
- **Holder:** a magistrate (Middle). He holds a carter's wagon. Seal type: fine unpaid.
- **Truth:** *as told*: the fine is just; the wagon crushed a stall.
- **Requester:** the carter (Low). Reason: obligation. He owes his master the wagon's value (creditor/debtor link).
- **Ways rolled:**
  - meet the condition by paying the fine (trade)
  - work off the term (trade)
  - take it in transit (take)
- **Holds.**

**Roll 6: Take the readings + Rightful keeper** *(Claude)*
- **Keeper:** the warden of a hill shrine (Middle), reason: duty. A standing stone in the shrine yard is cut with old boundary marks.
- **Requester:** a valley reeve (Middle), reason: evidence. His lord needs the marks copied as proof of a grazing line.
- **Truth:** *the marks were moved*: someone recut one mark a generation ago. Tell: one groove is sharper than the rest.
- **Ways rolled:**
  - come as a pilgrim, with a rubbing cloth under the cloak (deceive)
  - do the warden's work first: re-set the fallen yard wall (earn)
- **Holds.** The shrine's claim to keep the stone unread is the better one, and the readings help the reeve's lord more than the valley.

## 11. Changes from review
- **Ill-gotten → Passed on.** Ill-gotten duplicated Reckoning's Swindle. In Passed on, the holder is innocent.
- **What's inside → Wants to be rid of it.** The old twist was stock, and it brought back the secret-heir cliché.
- **Each premise now has its own ways through,** aimed at that premise's obstacle. The shared heist menu is gone.
- **Under seal gained a seal-type list and an in-transit way,** so it can't drop to 1 way.
- **"Unaware" is now a state, not a reason.** The holder still rolls a real reason.
- **Load-bearing is narrowed** to things that protect many people.
- **Separation exemption for acquisition** has been moved to the rulebook for Laz's review.
- **v3 (final audit):** Roll 5's reason is fixed, since *Rightful keeper* requires legacy, evidence, or obligation.

## 12. Changelog
- **2026-10-27 (v3.3), v7: Laz-approved book rules (A2, A4, A8, C1).** New premise *Take the readings*: field work with an instrument, a survey or a copy made on site, here rather than Restore because nothing need be broken and the obstacle is the ground's keeper (rulebook §8); its fields, five ways, truth rows with shapes and tells, twists and endings (§1, §2, §4, §5, §6, §7, §9, roll 6); default `costs` on every ending (A4); requester reasons are rows of the shared list, with *curiosity*, *money* and *duty* added (A8); tells (A2).
- **2026-10-27 (v3.2), v6: World C fixes.** Each truth row carries its shape, and *as told* is marked *(confirming)* (§5; rulebook, *Shapes*); the ledger never closes the family.
- **2026-10-26 (v3.1), audit fixes.** A truth table (§5: *as told*, *promised elsewhere*, *a copy*), each row naming the ways that reach it; test rolls tagged with their row.
