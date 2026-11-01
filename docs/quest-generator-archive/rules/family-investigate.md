# Family: Investigate — Locked (v3.3)

## 1. Situation
**Something happened, and nobody knows who did it, or whether anyone did.** Or, with *Watch them* (v7), nothing has happened yet, and someone wants eyes on a person or place before it does.
- **vs. Threat (*Go and look*):** *Watch them* watches a known person or place and reports what they do; *Go and look* scouts a place nobody knows and reports what is there.
- **vs. Reckoning:** in Reckoning the wrongdoer is known.
- **vs. Dispute:** in Dispute the facts are mostly known.
- **vs. Restore:** deliberate harm belongs to this family, not Restore.

## 2. Card fields
- **Requester**
- **Victim** (if dead, newly created)
- **Culprit** (hidden; may be *none*)
- **Red herrings** (1–2), each with a reason and a clearing fact
- **Pressure** (optional): one Low/Middle red herring is **already being blamed**. They face dismissal, a beating, or being driven out unless someone clears them.
- **Scene** (a place, with its access level)
- **Truth:** who did it (§5b)
- **Arbiter**
- **Optional slots:** witness, the worker who was blamed last time, plus any person a way names
- **Premise, ways through, twist, possible endings**
- ***Watch them*** (v7) has no victim, culprit, red herrings or blamed worker. In their place:
  - **Subject:** the person watched (the opposing slot), or the keeper of the place watched;
  - **The suspicion** (`details.suspicion`): what the requester fears the subject will do, as the requester tells it;
  - **The place:** where the subject goes, with its access level;
  - **no arbiter:** nothing is accused, so no ending needs one.

## 3. Kinds of ways through
- **Question:** talk to people.
- **Examine:** look at the scene, the body, the damage.
- **Follow:** trails, goods, money, movements.
- **Provoke:** traps, stings, rumors, rewards.

**Clue rule:** every way through is written so that it points at the truth. Each way names the clue it yields.

## 4. Premises

| Premise | Culprit needs | Ways through (roll 2–3, ≥2 kinds) |
|---|---|---|
| **Theft** | access to the scene. A guarded or fortified scene forces an insider. | get the one servant who saw it to talk, though it may cost her her place (question; fills witness); work out how they got in (examine); learn what else went missing, and when: small losses going back months (examine); trace the item through fences and markets (follow); set a trap with a decoy (provoke; *requires a culprit*) |
| **Disappearance** | a link to the missing person | question their associates (question); search the last-seen place (examine); find what they were afraid of: their debts, their quarrels (examine); retrace their last day (follow); put out word of a reward and see who answers (provoke) |
| **Suspicious death** | a motive tied to the victim | examine the body and the scene (examine); question those who stood to gain (question); find who the dead person last argued with (question); follow the money (follow); let a real witness be known, putting them in danger, and watch who moves (provoke; fills witness; *requires a culprit*) |
| **Sabotage** | a rival or grudge link | inspect the damage to see how it was done (examine); question the workers (question); ask the worker who was blamed the last time it broke (question; fills that worker); see who profits from the delay (follow); catch them in the act (provoke; *requires a culprit*) |
| ***Watch them*** (v7, C1): watch a person or place where no crime has happened yet, and report what you see *(Claude)* | **no culprit and no victim.** A subject with a link to the requester (partner, rival, employee, spouse, kin, creditor or debtor), or who keeps a place the requester fears for; the suspicion is a fact the requester holds; the subject is free and at home at this card's time | take work beside the subject at their place, and listen to what's said when they think nobody hears (question; fills the neighbour or master who takes you on; clue: what they say to others); keep watch on the place through a night from somewhere nobody looks, a loft or a window across the way (examine; clue: who comes, and when); follow the subject on their rounds for a day and mark everyone they meet (follow; clue: the one meeting that doesn't fit); go through what they leave behind: the tally board, the ashes in the grate, the mud on the cart wheels (examine; clue: where they've been, what they burned); let a rumour reach them that they'd have to act on, and watch what they do (provoke; clue: what they move, or whom they warn; *risks being seen*) |

- **Separation:** doesn't apply (crime; *Watch them*: no economic pair).
- **With a "no crime" truth:** every premise still has at least 3 ways across at least 2 kinds, even after the culprit-only ways drop out.

## 5. Lists
- **Requester reasons** (rulebook §7, the shared list; v7): *love*, *duty*, *fear*, *pride*, *curiosity*, *obligation*. Every card fills `reasons.requester`.
- **Culprit motives** (on *Watch them*, the subject's, when the truth gives one):
  - greed
  - revenge
  - cover-up
  - desperation
  - ambition
  - preservation
- **Red herring reasons:**
  - seen near the scene
  - argued with the victim
  - benefits from the outcome
  - lying about something unrelated (needs a defined secret)
- **Clearing fact:** every red herring has one.

## 5b. Truth: who did it
The truth is a secret fact naming the culprit and their motive, or *no crime*. The row is **who the culprit is to the case**. The motive is its own field (`motive`), never part of the row: "a bystander (greed)" and "a bystander (cover-up)" are one row, with one shape. Every way reaches the truth (the clue rule, §3).

| Truth | What it is | Shape |
|---|---|---|
| *A bystander* | someone at the edge of it with no tie to the victim (a hand, a lodger, a lay brother) | `bystander-did-it` |
| *Known to the victim* | someone the victim knew: kin, a partner, a rival, someone they turned off | `close-to-the-victim` |
| *Close to the requester* | someone the requester loves or depends on; only with *The truth costs the requester* | `close-to-home` |
| *The story's own* | the Hand, the lieutenant or a proxy; main and faction stories only (§10; guild odd jobs) | `the-enemy-did-it` |
| *No crime* | an accident, someone who left by choice, something mislaid; only with *No crime* | `no-crime` |

- **No confirming row** in the table above. A hidden culprit always tells the player something new, so a card on the first four premises is never plain. *Watch them*'s *As feared* is the family's one confirming row (below).

**Watch them** (v7, C1) *(Claude)*. The row is **what the subject is really doing**. Each non-confirming row shows a tell on the card's face before the choice (rulebook rule 8); the tell column is the pattern, the story layer words it. Every way reaches the truth (the clue rule).

| Truth | What it is | Shape | Tell (pattern) |
|---|---|---|---|
| *As feared* | the subject is doing what the requester suspects | *(confirming)* | — |
| *Shielding someone* | they hide it to protect someone; names who | `protects-someone` | they buy or carry more than their own house needs |
| *Someone above* | they act for a backer or buyer off the card; names who | `someone-above` | coin or goods they couldn't come by on their own |
| *They know* | someone on the requester's side warned them; names who | `sold-from-inside` | their habits changed the day the watching was asked for |
| *Close to home* | their partner in it is someone the requester loves or relies on; only with *The truth costs the requester* | `close-to-home` | something of the requester's own household at the subject's door |
| *Nothing to find* | they do what they say, and the fear was wrong; only with *No crime* | `no-crime` | the one odd thing they do has a plain reason anyone could ask |

- On main and faction cards the subject may be the Hand, the lieutenant or a proxy. The truth never states the story's secret before its revelation (rulebook §3, *No secret leaks*).

## 6. Twists
- **None.**
- **The truth costs the requester:** the culprit is someone the requester loves or depends on. They asked for the truth, and they won't want it.
  - *Requires:* a family, spouse, dependent, or employer link from the requester to the culprit.
- **Sympathetic culprit:** the culprit's reason is understandable.
  - *Requires:* motive desperation, revenge, or preservation.
- **No crime:** an accident, someone who left by choice, or something misplaced.
  - Culprit is *none*. Culprit-only ways are removed.
  - On *Watch them*: the subject is doing nothing wrong, so the truth is *Nothing to find*.
- **Wrong target:** the culprit meant to harm someone else.
  - *Requires:* a second person the harm was meant for.
  - On *Watch them*: what the subject plans is aimed at someone the requester never thought of.
- **Tells** (rulebook rule 8, v7): *No crime* and *Wrong target* (*Not what it seems*) each name one public fact that doesn't fit the told story in `details.tell`; on this family it is usually one of the clues.
- **Roll order:** *The truth costs the requester* and *No crime* decide the truth row (on *Watch them* too: *Close to home*, *Nothing to find*), so they are rolled before it (rulebook, *Roll order*).
- A twist's detail can carry a shape of its own (rulebook, *Shapes*): a *wrong target* set as a step or rail loosened in the night is `rigged-to-fall`.

| Twist | Theft | Disappearance | Suspicious death | Sabotage | Watch them |
|---|---|---|---|---|---|
| The truth costs the requester | ✓ | ✓ | ✓ | ✓ | ✓ |
| Sympathetic culprit | ✓ | ✓ | ✓ | ✓ | ✓ (the subject's reason) |
| No crime | ✓ | ✓ | ✓ | ✓ | ✓ (*Nothing to find*) |
| Wrong target | ✗ | ✓ | ✓ | ✓ | ✓ |

## 7. Possible endings
Each with its default **`costs`**, who loses by it (rulebook rule 7, v7). What failure endings write is mechanics §4.
- accuse before the arbiter (costs: the culprit; the one ending that costs only the other side)
- confront privately (costs: the culprit and the requester: no public answer)
- blackmail (costs: the culprit and the requester: the wrong is never answered)
- cover it up (costs: the requester, or the victim's kin)
- let it go (costs: the requester)
- truth not found (costs: the blamed red herring; with none, the requester)
- **the wrong one accused** (v7, B7): the player names a red herring, and it sticks (costs: the accused). *Requires* a red herring; not on *Watch them*.

**Watch them** (v7) *(Claude)* replaces the list above with:
- report what was seen (costs: the subject; the one ending that costs only the other side)
- report it shaded: the player tells the requester less, or other, than they saw (costs: the requester)
- warn the subject (costs: the requester)
- confront privately (costs: the subject and the requester)
- blackmail (costs: the subject and the requester)
- seen and lost: the subject spots the watcher (costs: the requester; the failure ending)
- With *The truth costs the requester*, *report what was seen* costs the requester too.
## 8. Standing effects
- **Accusing upward:** if the culprit outranks the **requester**, the "accuse before the arbiter" ending is offered only when at least one rolled way is tagged *hard proof*. Examine, follow, and provoke ways can carry that tag. Accusing someone below the requester has no such requirement. (v1 measured this against the player; it now measures against the requester.)
- **No arbiter outranks the culprit:** the "accuse before the arbiter" ending is unavailable.

## 9. Registry interactions
- **Creates:** victim, culprit, red herrings, scene, arbiter, witness.
- **Secret:** the culprit's identity is a **secret fact**.

## 10. Main-story use
- **Hook:**
  - *The truth costs the requester* is filtered out, along with *no crime* and *wrong target*, and culprit "none."
  - The culprit is the Hand or a proxy.
- **Trials:** *no crime* is allowed only if the truth still names the Hand, the lieutenant, or a proxy. Example: an accident the Hand covered up.
- ***Watch them*** is filtered out on the Hook (no harm has happened yet) *(Claude)*.
- **Never** a Threshold or Climax turn card, because the culprit is hidden.

## 11. Test rolls
*(Illustrative.)*

1. **Theft, guarded scene.**
   - The culprit is an insider (a footman).
   - Pressure: a kitchen girl is blamed and will be dismissed.
   - Ways rolled: get the servant who saw it to talk (question); learn what else went missing (examine).
   - **Holds.** Both ways put an ordinary person at risk.
2. **Disappearance + No crime, High requester sympathetic.**
   - Lady Wren (High) fears her daughter was taken.
   - Truth: the daughter left to escape a match.
   - Ways rolled: find what she was afraid of (examine); put out word of a reward (provoke).
   - **Holds.**
3. **Suspicious death + The truth costs the requester.**
   - A miller asks who killed his brother.
   - Culprit: the miller's own son (family link).
   - Ways rolled: examine the body (examine); follow the money (follow).
   - **Holds.**
4. **Sabotage + Wrong target.**
   - The sluice was cut to flood the neighbor's field. It flooded the requester's.
   - Ways rolled: ask the worker blamed last time (question); see who profits (follow); catch them in the act (provoke).
   - **Holds.**
5. **Theft, culprit outranks the requester.**
   - Hard proof is needed.
   - Ways rolled: trace through fences (follow); set a trap (provoke).
   - **Holds.**

6. ***Watch them*, no twist** *(Claude)*.
   - A ferry-master (Middle) asks the player to watch his new partner, who goes up to the top landing after dark.
   - Truth: *Someone above*: the partner works for a toll lord who means to buy the ferry out. Tell: he pays his share in new-struck coin.
   - Ways rolled: follow him on his rounds (follow); go through the ashes in his grate (examine).
   - **Holds.** Reporting it ends the partnership; shading it keeps the ferry's peace for a season.

## 12. Changes from v1
- **Ways through:** 5 tagged ways per premise, across 4 kinds. v1 had 3 fixed ways.
- **Ways that cost someone:** *the servant who saw it*, *the real witness*, and *the worker blamed last time*.
- **Provoke ways:** need a culprit.
- **Removed:** "inspect records" (Sabotage) and "who suddenly has money" (Theft). Money-tracking was repeating across premises.
- **New twist:** *the truth costs the requester* replaces *requester did it*, which was the stock detective twist.
- **New field:** optional scapegoat pressure.
- **Accusing upward:** now measured against the requester.
- **Main-use filters added.**

## 13. Changelog
- **2026-10-27 (v3.3), v7: Laz-approved book rules (A2, A4, A8, B7, C1).** New premise *Watch them*: spying where no crime has happened yet, with its fields, five ways, truth rows with shapes and tells, twists and endings (§1, §2, §4, §5b, §6, §7, §10, test roll 6); *the wrong one accused* (B7); default `costs` on every ending (A4); requester reasons from the shared list (A8); tells on *Not what it seems* twists (A2).
- **2026-10-27 (v3.2), v6: World C fixes.** A truth table (§5b): the row is who the culprit is to the case, never the motive, each with its shape (rulebook, *Shapes*); no confirming row; *The truth costs the requester* and *No crime* are rolled before the truth; *wrong target* by a loosened step is `rigged-to-fall`.
