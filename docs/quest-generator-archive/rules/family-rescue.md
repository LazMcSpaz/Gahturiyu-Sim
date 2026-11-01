# Family: Rescue — Locked (v3.5)

## 1. Situation
**Someone is held. Getting them out means dealing with why they're held and who is holding them.**
- **vs. Passage:** the person isn't in hand yet; they have to be got out first.
- **vs. Acquire:** the target is a person.
- **vs. Reckoning:** the goal is freedom, not payback.

## 2. Card fields
- **Requester**, with a **reason** (rulebook §7, the shared list; v7): *love*, *dependents*, *obligation*, *duty*, *faith* or *freedom*. Every card fills `reasons.requester`; a hidden motive (§7) sits under it.
- **Captive:** in main-story climaxes, this may be a group, through one representative.
- **Relationships**
- **Captor:** a person, or a creditor's employee acting for them.
- **Place:** where the captive is held, with its access level.
- **Why taken:** Kidnapped only (§5).
- **Truth:** §5.
- **Optional slots:** guard, band member, prisoner (for swaps), representative (for crowd pressure), witness (to the charge), real debtor, a second captive (twist), and any person a way names.
- **Premise, ways through, twist** (+ motive), **possible endings**

## 3. Kinds of ways through
- **Force:** fight, storm, break out
- **Stealth:** slip in, spring them unseen, take them in transit
- **Price:** pay, trade, swap, work it off
- **Leverage:** prove, persuade, pressure, turn someone

## 4. Premises

| Premise | Requirements | Ways through (roll 2–3, ≥2 kinds) | Separation |
|---|---|---|---|
| **Kidnapped:** a lawless band took them | captor is lawless and has a hideout; *why taken* is rolled | fight them at the hideout (force; *removed if fortified*); slip in at night and bring them out (stealth; *removed if fortified*); bribe a guard (price; fills guard); give the band what it took them for: the ransom, or the skill they were taken for, done another way (price; *matches why taken*); turn a band member who wants out (leverage; fills band member) | exempt (crime) |
| **Captured in battle:** held as an honorable prisoner | captor is a noble with a stronghold; captive is Middle+ | raise the ransom (price); arrange a prisoner swap (price; *requires a prisoner the requester's side holds*); spring them during a transfer or a parole walk (stealth); win release on parole by vouching for their honor (leverage); convince the captor the prisoner is worth less than the ransom: disowned, broke, a nobody (leverage) | applies (a ransom is economic) |
| **Imprisoned unjustly:** a legal authority has locked them up | captor holds legal authority | go over the charge with the witness who swore to it, which proves their innocence or shows they did it (leverage; fills witness; *reaches the truth, whichever it is*; kept under *justified captor*, §6); bribe the official (price; *removed if Top*); make holding them cost the official, with their customers, workmates, or neighbors crowding the gate (leverage; fills representative); break them out of the jail (force; *removed if fortified*); spring them during the transfer to trial (stealth) | exempt (authority) |
| **Bound by debt:** held until a debt is paid | captor is the creditor, or the creditor's employee | pay it off (price); work off the debt in their place (price); steal the ledger (stealth; **the family's one document way**; *reaches the truth, whichever it is*); find the real debtor (leverage; *requires truth = misassigned*); prove the debt was paid or inflated, through a witness (leverage; *requires truth = paid or inflated*) | applies |

**Worst case:** Imprisoned, with a Top official, a fortified jail, truth *guilty*, and *justified captor*. Bribe and break-out are removed. Three ways remain: the witness way (leverage; it reaches *guilty*), crowd pressure (leverage) and spring in transit (stealth). That's 3 ways, 2 kinds.

## 5. Truth and why taken

Each value is a truth row, with its shape (rulebook, *Shapes*) beside it.

| Premise | Truth values |
|---|---|
| Kidnapped | *as it looks* (the band's stated reason is the real one) *(confirming)*; *inside help* (someone on the captive's side sold the band the way in; names them) `sold-from-inside`; *a better buyer* (the band already has a higher offer for the captive, and will move them) `another-buyer`. Reached by: turn a band member who wants out. |
| Captured in battle | *honorable* *(confirming)*, or *broke quarter* (killed men who surrendered; only with *justified captor*) `held-rightly` |
| Imprisoned unjustly | *innocent* *(confirming)*, or *guilty* `held-rightly` |
| Bound by debt | *misassigned* `wrong-name`, *paid* `paid-already`, *inflated* `made-bigger`, or *real and fair* `held-rightly` |
| Held for heresy (Holy House signature) | what the charge really is: *a true believer* *(confirming)*; *a grove-goer* (the charge is true) `held-rightly`; *a debt by writ* (it covers a tithe debt the family owes the house) `hidden-aim`. Homed in `quest-guild-kits.md`; the two must match. |
| Left behind on contract (Lantern Company signature) | why the client won't pay: *cover* (the contract was cover for something else), *broke terms* (the captives broke its terms), or *ashamed* (he can't afford it and is ashamed). Shapes in `quest-guild-kits.md`. |

- **The captor can be right.** *Guilty* and *real and fair* are ordinary rows, rolled with or without a twist. Together they come up on about **1 in 3** Imprisoned and Bound-by-debt cards; the other values share the rest. On Kidnapped, only *as it looks* can make the band's side the fair one (a healer taken for a camp's sick children).
  - On such a card the requester believes, or says, the captive is wronged. Finding out otherwise is the card's turn.
  - *Justified captor* (§6) goes further: the captor's case is the better one, and the card is built around it.
- **World cap.** Every row here is capped by the world ledger, and its shape with it (rulebook, *World ledger*). *Innocent* is a confirming row: the requester already says so, so with twist *none* the card is plain (rulebook, *No blank card*).
- **Left behind on contract** is homed in `quest-guild-kits.md` (Lantern Company); its row is listed here so §6 can read it. The two must match.

**Why taken (Kidnapped):**
- **for ransom**
- **for their skill:** a healer, a smith, a ferryman. The band's need for that skill stays a public fact on the card.
- **as leverage** over someone
- **to silence them** ("give the band what it took them for" is removed: there's nothing to give)

## 6. Twists
- **None.**
- **Unwilling captive:** the captive has a reason to stay.
- **Hidden motive:** the requester has a secret (§7).
- **Justified captor:** the captor has the better case. *Requires* one of these truths: *broke quarter*, *guilty*, *real and fair*, *broke terms* or *a grove-goer* (§5; the last two from their kits' twist rows), so it is rolled before the truth (rulebook, *Roll order*). It removes the "prove" ways, but never a way that reaches the truth whichever it is: Imprisoned's witness way stays, so *The truth is reachable* holds (rulebook, rule 8). It adds no shape of its own: the row's shape counts.
- **Not alone:** a second captive (Low, new) is held with them. Stealth and price ways are tagged *one captive only*. *Requires* Kidnapped or Imprisoned. Twist group: **Innocents pay**.

| Twist | Kidnapped | Captured in battle | Imprisoned unjustly | Bound by debt |
|---|---|---|---|---|
| Unwilling captive | ✓ | ✓ | ✓ | ✓ |
| Hidden motive | ✓ | ✓ | ✓ | ✓ |
| Justified captor | ✗ | ✓ (broke quarter) | ✓ | ✓ |
| Not alone | ✓ | ✗ | ✓ | ✗ |

- **Signature premises** carry their own twist rows in their kit docs (*Left behind on contract*: `quest-guild-kits.md`).

## 7. Hidden motives
*Hidden motive* is a *Someone's hiding something* twist, so its tell goes in `details.tell` (rulebook rule 8, v7). The table's tell column is that tell.

| Motive | Requires | Tell |
|---|---|---|
| Wants the captive's silence | the captive knows something damaging | rushes; wants the captive's belongings returned |
| Wants the captive's claim | captive Middle+; family/heir link | legal papers in the requester's hands |
| Wants an item the captive carries | a thing linked to the captive | asks about the item, not the person |
| Wants revenge on the captor | grudge link | rejects peaceful ways |
| **Put them there** | the requester's testimony or unpaid debt caused the capture | wants it undone before anyone asks how it happened |
| Wants the credit | ally or rival link | insists on bringing them home personally |
| Owes the captor | Low requester | desperate, fearful |
| Afraid of retaliation | Low requester | demands secrecy |
| Needs the captive's support back | Low requester; the captive supported them | focuses on lost income |

## 8. Possible endings
Each with its default **`costs`**, who loses by it (rulebook rule 7, v7). What failure endings write is mechanics §4.
- freed (costs: the captor; the one ending that costs only the other side. With *Not alone*, a one-captive way also costs the second captive)
- freed but harmed or late (costs: the captive)
- captive stays by choice (costs: the requester)
- exchanged for something (costs: the requester, who gives it)
- moved elsewhere (costs: the captive)
- rescue fails (costs: the captive)
- **handed to another** (v7, B7): the player gives the captive to someone else who wants them (costs: the captive and the requester). *Requires* such a person on the card: the better buyer (truth *a better buyer*), the party a hidden motive points at, or the captor's rival; removed otherwise (rulebook rule 9) *(Claude)*.

## 9. Standing effects
- **Low requester:** can't fund ransoms or debts, so the cost shifts to the player, or it's paid in labor.
- **Top captor:**
  - Bribe ways are removed (rulebook §4).
  - Ways that need someone outranking the captor fall away on their own.
- **Separation:** applies to *Captured in battle* and *Bound by debt*.

## 10. Main-story use
- **Hook:**
  - The captive is the heart's kin, never the heart.
  - *Justified captor* is filtered out.
  - *Hidden motive* is filtered out when the requester is the heart.
- **Ordeal (captive door):**
  - Kidnapped (why taken = *as leverage*) or Bound by debt only.
  - "Give the band what it took them for" is removed, because the sibling Acquire door is that quest.
  - *Justified captor* is filtered out.
- **Climax:**
  - Rescue sits on a non-turn card.
  - The captive may be a group, through one representative.

## 11. Test rolls
*(Illustrative.)*

1. **Kidnapped, taken for their skill.** A band holds a village healer in a ruined mill.
   - Ways rolled: give the band what it took her for, by finding them another healer (price); turn a band member (leverage).
   - **Holds.** Freeing her leaves the band hunting for the next healer.
2. **Captured in battle, High party sympathetic.** Lord Arnhelm (High) holds a mercenary captain honorably.
   - Requester: the captain's sister (**Low**), so the cost shifts to the player.
   - Ways rolled: parole by vouching (leverage); spring him during a transfer (stealth).
   - **Holds.**
3. **Captured in battle + Justified captor.** Truth: the captain *broke quarter* at the ford.
   - "Prove" ways: none apply to this premise.
   - Ways rolled: raise the ransom (price); convince the captor he's worth less (leverage).
   - **Holds.** The honorable captor is holding a dishonorable prisoner.
4. **Bound by debt.** A moneylender's yard-boss holds a carter's daughter. Truth: *inflated*.
   - Ways rolled: prove it through a witness (leverage); work it off (price).
   - **Holds.**
5. **Imprisoned unjustly: Top official, fortified jail, truth *innocent*.**
   - Bribe and break-out are removed.
   - Ways remaining: go over the charge with the witness, which proves innocence (leverage); crowd pressure (leverage); spring during the transfer to trial (stealth).
   - **Holds.**
6. **Bound by debt, truth *real and fair*, no twist.** A mill-owner's steward holds a ferryman's son for a debt the boy really ran up at dice.
   - Ways rolled: steal the ledger (stealth; it shows the debt is fair); work it off (price).
   - **Holds.** The player learns the boy owes it. Freeing him now means paying or working, or stealing a fair man's ledger.

## 12. Changes from v1
- **Bigger pools:** 5 tagged ways per premise, across 4 kinds. v1 had 3 fixed ways.
- **Truth table** added, with *why taken* for Kidnapped.
- **Fortified filter** now applies to entry by stealth too.
- **Imprisoned:** bribe and pressure are now separate. The pressure way became crowd pressure, and a way to spring them in transit was added.
- **Captured in battle:** "break out" is replaced by "spring them during a transfer." The petition-a-superior way became "worth less than the ransom."
- **Justified captor** now works for *Captured in battle*.
- **New twist:** *Not alone*.
- **Hidden motive:** *wants them to stay captured* became *put them there*.
- **Main-use filters** added.
- **Unchanged:** the creditor's-employee captor carries over from v1.

## 13. Decisions made without Laz
1. **The witness way stays under *justified captor*** (v6b audit). "Go over the charge with the witness who swore to it" reaches the truth either way, and on *Imprisoned unjustly* with *guilty* it is the only way that does. Removing it would break *The truth is reachable* (§4, §6).

## 14. Changelog
- **2026-10-27 (v3.5), v7: Laz-approved book rules (A2, A4, A8, B7).** New ending *handed to another* (B7); default `costs` on every ending (A4); requester reasons from the shared list (A8); hidden motives' tells are the card's `details.tell` (A2).
- **2026-10-27 (v3.4), v6: World C fixes.** Every truth row carries its shape, and *as it looks*, *honorable* and *innocent* are marked *(confirming)* (§5; rulebook, *Shapes*); "plain truths" renamed ordinary rows, since *plain* now means a card with a confirming truth and no twist; the world-cap note follows the new ledger; *justified captor* is rolled before the truth. v6b audit: *justified captor*'s required truths include *a grove-goer*, as the Holy House kit's *Held for heresy* row says. v6b audit decision: the witness way is kept under *justified captor*, since it reaches the truth either way (§4, §6; new §13 *Decisions made without Laz*).
- **2026-10-26 (v3.3), audit fixes.** Kidnapped gets a truth row (*as it looks*, *inside help*, *a better buyer*), reached by turning a band member, so a Kidnapped card with twist *none* is never blank (rulebook, *No blank card*).
- **2026-10-26 (v3.2), v5: world-test fixes.** *Guilty* and *real and fair* become ordinary rows (about 1 in 3), with the innocent and not-owed rows capped by the world ledger; a truth row for *Left behind on contract*; *prove their innocence* becomes a witness way that reaches the truth either way; test roll 6.
