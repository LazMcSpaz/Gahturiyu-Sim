# Family: Dispute — Locked (v4.3)

## 1. Situation
**Two parties both believe they are in the right about the same stake, and the player is pulled in to settle it.**
- vs. Investigate: facts are mostly known; the problem is two partly legitimate claims. Discovery is never the goal.
- vs. Reckoning: no agreed wrongdoer.
- vs. Rescue: no one is held.

## 2. Card fields
- **Requester** — who brings the player in. **Reason** (rulebook §7, the shared list; v7): a requester who is a party takes their party reason (§5); a neutral one takes *duty* or *peace*. Every card fills `reasons.requester`.
- **Party A / Party B** — the claimants.
- **Relationship(s)** — A↔B, and each to the requester.
- **Stake** — must be a registry entry:
  - a **place** or **thing**, or
  - a **right**, stored as a *document thing* or attached to a place (water right → the spring), or
  - **reputation**, stored as a *fact* on the person.
- **Each party's reason** (§5).
- **Truth** — what actually happened, rolled at generation (e.g., "the song really was cruel," "no boundary record survives"). Truth ways only appear if the truth gives them something to find. The truth stays free-form, but the world ledger logs it by **kind** (§5).
- **Arbiter** — who can make a binding ruling, or *absent*.
- **Player position** — advocate (requester is a party), mediator (requester is neutral), judge (requester outranks both parties and grants it).
- **Pressure** — what is at risk while it stays unsettled (drought, a pending flogging, a deadline).
- **Possible endings** — static list, §8.
- **Premise, ways through, twist.**

## 3. Ways-through rule (replaces the fixed truth/deal/side triad)
- Each premise lists **4–5 ways**, tagged *truth*, *deal*, or *side*.
- The generator **rolls 2–3**, covering **at least two kinds**.
- **Truth ways use people and places, not paper.** At most one "retrieve a document" way in the whole family.
- **No Dispute way is a contest.** The champion way was cut in review.
- **Unnamed helpers become slots:** any person a way names (the confidant, the witness, the oldest local, the third party, the patron) is an optional slot, filled only if that way is rolled (rulebook §2.4).
- **Groups as parties:** a guild, village, or family stands as a party through one named representative.

## 4. Premises

| Premise | Requirements | Ways through (roll 2–3) | Separation |
|---|---|---|---|
| **Contested inheritance** | deceased (new) owned the stake; A and B have family, heir, or spouse links to them | the deceased's last confidant tells what was intended (truth); prove who actually cared for the deceased and the property (truth); broker a division (deal); buy one heir out with something they want more (deal; removed if that heir is Top); back one heir before the arbiter (side) | applies |
| **Broken agreement** | A and B have an employer/employee, trade, or creditor/debtor link; the agreement is a document thing *or* had a witness | find the witness to the original handshake (truth); the written contract (truth — the family's one document way); renegotiate terms (deal); find a third party to cover the shortfall (deal); enforce it for one side (side) | applies, **except for employer/employee links**, which cross class by nature |
| **Shared resource** | stake is a place (or a right on it); both parties live nearby | walk the old line with the oldest local who remembers it (truth); prove who used it first from the wear on the land (truth); broker a sharing schedule (deal); find one side an alternative so the stake stops mattering to them (deal); secure it for one side (side) | exempt |
| **Public offense** | the offense happened in public; offended party has reputation to lose | find out what was really said, from the crowd (truth); make mercy cost less than punishment — find what the offended gains by forgiving (deal); arrange a public apology or payment (deal); get the offender out of reach — hide them, get them hired away, or find a patron who outranks the offended (side; the patron option is removed when nobody outranks the offended: rulebook, *Outranking slots*) | exempt |

## 5. Party reasons
These are rows of the shared list (rulebook §7), with this family's requirements.

| Reason | Requires |
|---|---|
| Survival — they can't live without the stake | Low or Middle |
| Dependents — others rely on them | any |
| Legacy — promised by family or tradition | family link to the stake's past |
| Pride — losing publicly would shame them | Middle+, or the Offense premise |
| Greed — they want more | any |
| Obligation — they owe the stake to someone else | creditor/debtor link to a third person (created) |

**Rule (protected):** at least one party has a sympathetic reason (survival, dependents, legacy, obligation). If both roll greed/pride, re-roll one.

**Truth kinds** (for the world ledger; each kind is a truth row, with its shape from rulebook *Shapes*)

| Kind | What it is | Reached by | Shape |
|---|---|---|---|
| *Record survives* | a record, or someone who remembers, settles what happened | the document way (Broken agreement's written contract), the confidant, the oldest local | `record-survives` |
| *No record* | nothing survives to settle it; only a deal or a side can end it | any truth way (it finds that nothing is left), then a deal way | `no-record` |
| *One side lies* | one party's account is false; names which, and what they hide | the witness, the crowd, the confidant | `one-side-lies` |
| *Both partly right* | each claim holds in part | a truth way that hears both sides (the confidant, walking the old line, the wear on the land) | `both-partly-right` |

- **No confirming row:** the requester's account is only one of two sides, so no row just confirms it, and a Dispute card is never plain.

## 6. Twists

- **None.**
- **Caught in the middle** — whoever wins, the people who live or work on the stake lose it. Represented by one named occupant (Low, new or from the background pass). *Requires:* stake is a place with occupants.
- **Hollow prize** — the stake is worth far less than either side thinks (buried in debt, spring failing, cargo spoiled). *Requires:* the stake gets a defect fact.
- **Old wound** — the stake stands in for an older grievance. *Requires:* a grudge link between A and B; the old-wound deceased (if any) is newly created and recorded. **Adds a way through:** "address the old grievance" (reparation, or the truth about what happened). This way is **always among the rolled ways.**
- **The law is on the wrong side** — the party with the legitimate claim has greed or pride; the party who is legally wrong has survival or dependents. The truth ways now hurt the sympathetic party. *Requires:* one greed/pride reason and one survival/dependents reason.

| Twist | Inheritance | Agreement | Resource | Offense |
|---|---|---|---|---|
| Caught in the middle | ✓ | ✓ (only if the agreement covers a place, such as a lease or workshop) | ✓ | ✗ |
| Hollow prize | ✓ | ✓ | ✓ | ✗ |
| Old wound | ✓ | ✓ | ✓ | ✓ |
| Law on the wrong side | ✓ | ✓ | ✓ | ✓ |

- **Tells** (rulebook rule 8, v7): *Hollow prize* (*Not what it seems*) names one public fact that doesn't fit the told story in `details.tell` (the spring's flow, the debt-collector at the door). *Caught in the middle*, *Old wound* and *The law is on the wrong side* show theirs in the pitch, the place, or every rolled way.

**Parked (reserve twist):** *Arbiter wants cover* — the arbiter rules exactly as the player recommends so the loser's grudge falls on the player. Requires an arbiter. Held back to stay within the 4-twist limit.

**Cut:** Third claimant (secret-heir cliché; caused registry contradictions), Instigator (turned Dispute into Investigate), Compromised arbiter (stock).

## 7. Standing effects
- **"Outranks" is the single test** for both arbiter and judge-granting requester.
- **Arbiter must outrank both parties.** If a party is **Top**, the arbiter is **absent**.
- **Top-standing people are never created by a quest.** They're pre-made. If a card needs one and none fits, filter out that combination.
- **Separation rule covers Party A and B only**, never twist characters.

## 8. Possible endings
Each with its default **`costs`**, who loses by it (rulebook rule 7, v7). Where the requester is party A, "the other side" is B.
- A wins (costs: B)
- B wins (costs: A)
- the stake is divided (costs: both parties, each giving part)
- a deal is struck (costs: whichever party gives most; with *Caught in the middle*, the occupant)
- the stake is made worthless or abandoned (costs: both parties)
- a hidden defect is revealed, used, or kept quiet (costs: whoever holds the stake after)
- the player walks away unresolved (costs: the weaker party; failure ending)
- **the player takes the stake** (v7, B7): the selfish choice (costs: both parties). *Requires* a place or thing stake; removed when the stake is a reputation *(Claude)*.
- With the requester a party, only the ending that goes against the other party costs only the other side; on a neutral requester's card, the A-wins and B-wins endings each cost only one party, and a turning card names which party is the other side *(Claude)*.
- What each ending does to the world is the mechanical layer's (mechanics §4, with defaults for the failure and selfish endings).

## 9. Registry interactions
- **Creates:** parties, stake, deceased (inheritance; always new), old-wound deceased (always new), occupant, creditor for Obligation.
- **Reuse:** arbiters are good reuse targets **from other families** (once-per-family still applies).
- **State-sensitive:** a stake or party used by another quest — fallback needed.

## 10. Test rolls
*(Illustrative. Names that recur in other families' rolls are examples of reuse, not authoritative counts.)*

**Roll 1 (rerun) — Shared resource, Old wound, no arbiter, advocate player** *(v1 dead end)*
- Hollows (Low, survival) vs. Rusk (Low, dependents); requester is the Hollows, so player is advocate; no arbiter nearby.
- Ways rolled: walk the old line (truth), sharing schedule (deal) + Old wound adds "address the old grievance."
- **Holds:** the grievance way is always rolled, so there's no dead end.

**Roll 4 (rerun) — Public offense, truth: the song really was cruel**
- Lady Osric (High, pride) vs. Kip (Low, **dependents** — fixed from survival); requester Nell (Low).
- Truth says the song was cruel, so "find out what was really said" is dropped.
- Ways rolled: make mercy cost less (deal), get Kip out of reach (side).
- **Holds:** two kinds, both sensible; the dread stays.

**Roll 6 (new) — Shared resource, Top party**
- The regional governor (Top, **pre-made**, greed) encloses the village pasture; villagers' representative (Low, survival).
- Arbiter: no one outranks Top → **arbiter absent**. Player is advocate.
- Ways rolled: prove first use from the land (truth), find the village an alternative pasture (deal), secure it for the village (side).
- Twist: Law on the wrong side — the governor holds a legal grant.
- **Holds.** With no arbiter, the truth way can't produce a ruling. It remains a valid way through and can lead to the deal ending.

**Roll 7 (new) — Broken agreement, employer/employee across class**
- Lord Aldmere (High, pride) vs. his former groom (Low, dependents), over back wages.
- Separation: exempt (employer/employee link).
- Ways rolled: find the witness (truth), find a third party to cover the shortfall (deal).
- Twist: Hollow prize — the Aldmere estate is broke; there is no money to win.
- **Holds:** a natural servant-vs-noble story the v1 rules blocked.

**Roll 8 (new) — Inheritance, Caught in the middle, reused arbiter**
- Ines vs. Edric over the counting house (both Middle). Occupant: Wick, a clerk (Low) who lives above it.
- Arbiter: a guild master (High) reused from another family. Old Fen (Middle) was rejected because he doesn't outrank the parties.
- Arbiter is High, so it outranks both Middle parties.
- **Holds:** Caught in the middle puts a Low clerk at the center of a merchant-family fight.

**Roll 9 — Inheritance, High party sympathetic**
- Lady Corvane (High, dependents: her late husband's tenants rely on her keeping the estate) vs. her brother-in-law (High, greed).
- Arbiter: a pre-made Top official (authority slot).
- Ways rolled: the confidant tells what was intended (truth), broker a division (deal), back one heir (side).
- Twist: none.
- **Holds:** the noble is the sympathetic party.

## 11. Rules this family introduced
All accepted into the rulebook: the ways-through rule, the truth slot, possible endings, Top pre-made only, add-never-contradict, the employer/employee exemption, and stakes stored as registry entries.

**v4 (final audit):**
- Arbiter is absent for Top parties.
- Old wound's grievance way is always rolled. The old "block deals" rule was a mechanical-layer gate.
- Top filters added.
- Groups are represented by one named person.
- High-sympathetic test roll added.

## 12. Changelog
- **2026-10-27 (v4.3), v7: Laz-approved book rules (A2, A4, A8, B7).** New ending *the player takes the stake* (B7); default `costs` on every ending (A4); requester reasons and party reasons are rows of the shared list (A8); tells (A2).
- **2026-10-27 (v4.2), v6: World C fixes.** Each truth kind carries its shape (§5; rulebook, *Shapes*); no confirming row; the patron way follows rulebook *Outranking slots*.
- **2026-10-26 (v4.1), audit fixes.** The truth stays free-form but is logged in the world ledger by kind (§5: *record survives*, *no record*, *one side lies*, *both partly right*), each naming the ways that reach it.
