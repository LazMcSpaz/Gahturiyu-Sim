# Family: Reckoning — Locked (v3.3)

## 1. Situation
**A wrong was done and everyone knows who did it, but normal justice failed. Someone wants a reckoning.**
- **vs. Investigate:** the wrongdoer is known.
- **vs. Dispute:** the wrong is acknowledged.
- **vs. Threat:** the harm is finished.
- **vs. Acquire:** what was lost is money, standing, or a life — not a specific thing that still exists.

## 2. Card fields
- **Requester:** the wronged, or someone acting for them, with a **reason** (rulebook §7, the shared list; v7): *love*, *honor*, *pride*, *need*, *duty* or *fear*. Every card fills `reasons.requester`; the demand (§5) says what they want, the reason why.
- **Wronged:** if dead, either newly created or a recorded death (rulebook past-event rule).
- **Wrongdoer**
- **The wrong:** the premise.
- **Shield:** why justice failed (§5).
- **Demand:** what the requester wants.
- **Truth:** rolled from §5's truth table.
- **Optional slots:**
  - *arbiter* — needed by justice ways and the Bought shield
  - *answers-to* — the person a Rank-shielded wrongdoer still answers to
  - *protector* — Protected shield
  - *the head* — the Hand-not-head twist
  - any other person a way names (whoever the arbiter fears, the other victims, the witnesses). These are filled only if that way is rolled (rulebook §2.4).
- **Premise, ways through, twist, possible endings.**

## 3. Kinds of ways through
- **Justice:** take it to someone with authority.
- **Ruin:** take away their standing, money, or reputation.
- **Amends:** restitution, confession, or apology.
- **Violence:** hurt, frighten, destroy, or kill.

**The shield adds its own way.** Each card adds its shield's way (§5) to the premise's pool before rolling.

**Notes, not rules:**
- Since cards roll ≥2 kinds, every card has at least one non-violent way.
- A demand that matches none of the rolled ways is deliberate tension. For example, the requester wants blood, but every open path is non-violent.

## 4. Premises

| Premise | Requirements | Ways through (roll 2–3, ≥2 kinds) | Separation |
|---|---|---|---|
| **Betrayal:** a partner, friend, or spouse sold them out, cheated, or abandoned them | ally, spouse, employer/employee, family, or kin link between the wronged and the wrongdoer | expose the betrayal to the people whose trust they still need (ruin); take back what the betrayal gained them (amends); bring the betrayed and the betrayer face to face (amends); bring it before the arbiter (justice); a beating, or a duel (violence; duel is *contest*) | exempt |
| **Ruin from above:** someone more powerful ruined them personally (a false dismissal, a debt called in early against its terms, a son forced into service). A debt the wrongdoer was legally entitled to call belongs to Dispute, not here | the wrongdoer outranks the wronged, **or** their power is bought or borrowed: shield *Bought* or *Protected* (§5). So Middle against Middle works when the wrongdoer has the arbiter in his pocket or a protector behind him. **Not** land seizure, which Dispute and Threat cover | turn the wrongdoer's own servants or allies (ruin); shame them publicly where it costs them (ruin); use leverage to force amends (amends); destroy what they built from it (violence) — plus the shield's way | exempt (authority) |
| **Blood debt:** someone was killed and the killer walked free | the dead person is new or a recorded death that matches the past-event rule; the killer is known; shield is Acquitted, Fled, or Protected | bring a new witness to the arbiter (justice; *requires truth = a witness exists*); take away what the killing got them — the post, the land, the inheritance (ruin); force a public confession (amends); the killer pays the blood-price and supports the dead person's dependents (amends); blood for blood (violence) | exempt (crime) |
| **Swindle:** they were cheated out of money or standing that is now gone | the wrongdoer is Middle+; shield is No law, Bought, or Fled | con the con artist (ruin); find the others he cheated and act together (ruin); make him repay before witnesses he can't ignore (amends); frighten him out of town before the next mark (violence) — plus the shield's way | exempt (crime) |

## 5. Lists

**Shields (why justice failed)**

| Shield | Requires | Way it adds |
|---|---|---|
| Rank | no one but the Top roster outranks the wrongdoer (the emperor doesn't hear such matters). *Before 2026-10-26 this read "no registry person", which one Top emperor made impossible.* | find the one person they still answer to — a creditor, a parent, a patron (justice; fills *answers-to*) |
| Bought | the arbiter has a creditor or ally link to the wrongdoer | expose the payment to whoever the arbiter fears (justice) |
| No law against it | Swindle or Betrayal | warn the next mark and dry up their trade (ruin) |
| Acquitted for lack of proof | Blood debt | *(none; the witness way covers it)* |
| Fled | the wrongdoer has a second place | follow them to their new life and decide there (amends) |
| Protected | a protector who outranks the requester | make shielding them cost the protector more than it's worth (ruin) |

**Truth** (a secret fact; each value is a truth row in the world ledger, with its shape from rulebook *Shapes*)

| Truth | What it is | Reached by | Shape |
|---|---|---|---|
| *As told* | the wrong happened as the requester tells it | any way that makes the wrongdoer answer: face to face, a public confession, repaying before witnesses | *(confirming)* |
| *A witness exists* | someone saw the wrong and hasn't spoken; names them. Unlocks Blood debt's witness way | Blood debt's witness way; elsewhere, a way that turns or gathers people (turn the servants or allies, find the others he cheated, expose the payment) | `unheard-witness` |
| *Shared blame* | the wronged, or the requester, had a hand in it; names how | an amends way: the wrongdoer's answer names it (face to face, a public confession, repaying before witnesses) | `shared-blame` |
| *Still profiting* | the wrongdoer still lives off the wrong; names what | a way that goes after the gains (take back what the betrayal gained them, take what the killing got them, destroy what they built from it, con the con artist) | `still-profiting` |

**Demand:**
- death (violence)
- confession (amends)
- repayment (amends)
- ruin (ruin)
- formal punishment (justice)
- exile (justice or violence)

## 6. Twists
- **None.**
- **The wronged don't want it:** the requester acts for someone who wants it left alone. *Requires:* the requester is not the wronged, and the wronged is alive.
- **Changed:** the wrongdoer now runs the mill, ferry, or shop the requester's own neighbors depend on. *Requires:* the wrongdoer owns or works a place the requester's community uses.
- **Hand, not head:** the wrongdoer acted on orders from someone above them. That person is untouched and outranks everyone involved. *Requires:* an employer/employee link from the wrongdoer to a *head* slot that outranks the requester. Shape: `someone-above` (rulebook, *Shapes*).
- **Disproportionate demand:** the demand far exceeds the wrong. *Requires:* the demand is death or ruin, and the premise is Swindle or Betrayal.

- **Tells** (rulebook rule 8, v7): *Hand, not head* (*Someone's hiding something*) names one public fact that doesn't fit the told story in `details.tell` (the steward's new horse came from a stable he never served). *The wronged don't want it*, *Changed* and *Disproportionate demand* show theirs in the pitch, the place, or every rolled way.

| Twist | Betrayal | Ruin from above | Blood debt | Swindle |
|---|---|---|---|---|
| Wronged don't want it | ✓ | ✓ | ✗ | ✓ |
| Changed | ✓ | ✓ | ✓ | ✓ |
| Hand, not head | ✓ | ✓ | ✓ | ✓ |
| Disproportionate demand | ✓ | ✗ | ✗ | ✓ |

## 7. Possible endings
Each with its default **`costs`**, who loses by it (rulebook rule 7, v7). What failure endings write is mechanics §4.
- Justice done formally (costs: the wrongdoer; the one ending that costs only the other side)
- Wrongdoer ruined (costs: the wrongdoer; with *Changed*, the requester too, whose community relies on the wrongdoer's place)
- Amends made (costs: the requester, who gives up the demand)
- Wrongdoer killed or driven off (costs: the wrongdoer and the requester, whose name carries it)
- Requester talked down, or forgives (costs: the requester, or the wronged)
- Player sides with the wrongdoer (costs: the requester)
- Reckoning fails (costs: the requester)

## 8. Standing effects
- **Justice ways need an arbiter who outranks the wrongdoer.** If none exists, the Rank shield applies.
  - **Exception:** under *faction law* (the main-story Schism), the faction's own leaders count as the arbiter, whatever their standing.
- **Contest ways are removed** when the parties are 2+ tiers apart.

## 9. Registry interactions
- **Creates:** the wrongdoer, the wronged, the arbiter, the answers-to person, the protector, and the head.
- **Reuse:** a wrongdoer from another family is allowed only for a **public** recorded wrong. Another card's hidden truth is never treated as a known wrong.
- **Past-event rule:** a recorded death or wrong must match every card that describes it, and no new card may give it a second cause.

## 10. Test rolls
*(Illustrative only; no shared registry.)*

**Roll 1 — Betrayal + Hand, not head (High wronged, sympathetic)**
- Lady Maudry (High) was betrayed by her steward, who sold her family's letters to a rival house. The rival's lord (High) gave the order, and he is the head.
- Shield: Protected — the rival lord shields the steward.
- Ways rolled:
  - expose him to those whose trust he needs (ruin)
  - face to face (amends)
  - make shielding him cost the rival more than it's worth (ruin; shield way)
- **Holds.**

**Roll 2 — Ruin from above, Rank shield**
- A Top official called in a weaver's family debt early to clear their workshop.
- No registry person outranks him, so the shield is Rank. The answers-to slot is filled by his own creditor, a moneylender's house.
- Ways rolled:
  - find the creditor he answers to (justice)
  - shame him publicly (ruin)
- **Holds.** In v1, a Rank shield coexisted with an outranking arbiter, which contradicted itself; that's now impossible.

**Roll 3 — Blood debt + Changed**
- The killer of a carter (dead, new) was acquitted. He now runs the only ferry the carter's village uses.
- Ways rolled:
  - blood-price and support (amends)
  - take what the killing got him (ruin)
  - blood for blood (violence)
- **Holds.**

**Roll 4 — Swindle, Fled shield**
- A land agent sold void leases to six families, then moved two towns over.
- Ways rolled:
  - find the others and act together (ruin)
  - follow him to his new life (amends; shield way)
- **Holds.** The demand is repayment, which the amends way matches.

**Roll 5 — Betrayal + Disproportionate demand**
- A miller's partner skimmed the till for a year. The miller (Middle) demands his ruin and his family's eviction.
- Shield: No law.
- Ways rolled:
  - take back what he gained (amends)
  - warn the next mark (ruin; shield way)
- **Holds.**

## 11. Changes from review
- **The shield now shapes the ways through** (its own column). The generic "find someone who outranks them" way is gone.
- **Arbiter, answers-to, protector, and head are slots.**
- **Violence ways vary, and Blood debt is less kill-leaning:**
  - Blood debt's duel was replaced with the blood-price.
  - "Strip what they love" was replaced with "take what the killing got them."
- **Answered wrong → Hand, not head.** Answered wrong repeated Dispute's old wound and Threat's "one of us caused it."
- **Changed** no longer uses the orphan-house cliché.
- **Ruin from above is personal ruin.** Land seizure is left to other families.
- **The anti-kill and demand-match rules are now notes.** Neither one filtered anything.
- **v3 (final audit):**
  - Betrayal accepts kin links.
  - Ruin from above excludes debts that could lawfully be called.
  - The shield's way counts as the one family-rule addition (rulebook §2.3).

## 12. Changelog
- **2026-10-27 (v3.3), v7: Laz-approved book rules (A2, A4, A8).** Default `costs` on every ending (A4); requester reasons from the shared list (A8); tells (A2).
- **2026-10-27 (v3.2), v6: World C fixes.** *Ruin from above* needs a standing gap **or** a Bought or Protected shield, so Middle against Middle works (§4); each truth row carries its shape and *as told* is marked *(confirming)* (§5); *Hand, not head* carries the shape `someone-above`.
- **2026-10-26 (v3.1), audit fixes.** A truth table (§5: *as told*, *a witness exists*, *shared blame*, *still profiting*), each row naming the ways that reach it; Blood debt's witness way requires *a witness exists*.
