# Family: Restore — Locked (v3.2)

## 1. Situation
**Something people relied on is broken. Fixing it is hard because of why it broke and who is involved.**
- **vs. Threat:** nothing is actively attacking.
- **vs. Reckoning:** there's no wrongdoer to punish.
- **vs. Acquire:** nothing is taken from anyone.
- **vs. Investigate:** the cause is **never deliberate harm by someone on the card.** That would be Investigate's Sabotage premise.
- **Return-only exception:** a main quest's Return, or a **faction story's epilogue**, may use the cause *deliberately broken* (or *deliberately ruined*). This records a past event, and whoever did it is not on this card (see assembly).
- **As a faction-story reveal:** a Restore card may carry a faction story's reveal only when the harm on it was done **off the card** (a past event whose doer holds no slot here). The reveal may show who; it never puts them on the card.

## 2. Card fields
- **Requester**, with a **requester reason** (rows of the shared list, rulebook §7; v7: survival, dependents, legacy, pride, greed, obligation). Every card fills `reasons.requester`. For *obligation*, the third person owed may be a local creditor already in the registry (a moneylender in the same town) instead of a new one.
- **The broken thing:** a **place** (bridge, mill, well, road), a **reputation** (a fact on a person), or a **bond** (a link between two people).
- **Cause:** the truth, picked from that premise's cause list.
- **Hard part** (§5).
- **Optional slots:**
  - **named member:** someone close to the broken thing
  - **holdout:** the one person who can fix it but won't
  - **profiteer**
- **Premise, ways through, twist, possible endings.**

## 3. Kinds of ways through
- **Craft:** work out why it really broke, and fix *that*. This is never "find the materials."
- **Gather:** rally labor, people, or funds.
- **Persuade:** win over whoever's help or consent is needed.
- **Substitute:** meet the same need another way.

**Cause rule:** a way is only offered if the rolled cause supports it.

## 4. Premises

Each cause is a truth row, with its shape beside it (rulebook, *Shapes*).

| Premise | Causes (truth) | Ways through (roll 2–3, ≥2 kinds) |
|---|---|---|
| **Broken works:** a bridge, mill, well, or road a community depends on has failed | worn out and neglected `neglected`; built badly `built-to-fail`; the ground moved `ground-moved`; stones carted off for someone's wall `stripped-for-parts`; *deliberately broken* (Return or faction epilogue only) `broken-on-purpose` | find the old builder and convince them to come back (persuade; fills holdout); find out why it really failed before rebuilding it to fail again (craft); trade labor with the next village, who help now and are owed at harvest (gather); shame the neglectful owner into paying in front of the people he needs (persuade; removed if the requester is the owner or 2+ tiers below them); find another crossing or source (substitute); *deliberately broken only:* rebuild it so it can't be broken the same way again (craft). With *deliberately broken*, "find out why it really failed" is removed, because the cause is already known. |
| **Lost good name:** someone's reputation was ruined | the stain is false *(confirming)*; true but long past `true-but-past`; exaggerated `made-bigger`; *false, spread by someone off the card* `rumour-planted` (only on a main-story Ordeal door, a main-story Return, a fallout mirror, a faction story's chapter-2 door, or a faction story's epilogue; the source way is filtered out) | find the people who remember what really happened (gather; not if true but past); find where the story started and answer it at the source (craft; not if true but past; whoever first repeated it did so carelessly, never maliciously, because malice would make it a Reckoning); a public act that shows who they are now (craft); get someone respected to vouch (persuade); help them start fresh where nobody knows them (substitute) |
| **Estrangement:** two people who were close no longer speak, and there's a deadline (a departure, a wedding) | a missed deathbed `absent-when-needed`; a spouse one of them disapproved of `disapproved-match`; a side taken in someone else's quarrel `took-a-side`; a secret that got told `secret-told`. **Never** a stake that could still be fought over (that would be Dispute), and neither of the two is an acknowledged wrongdoer (that would be Reckoning's Betrayal) | carry words between them (persuade); find the memory or thing that matters to both (craft); arrange a meeting neither can walk out of (persuade); bring in the people who loved them both (gather); help them part on decent terms (substitute) |
| **Lost livelihood:** the trade a place lived on has collapsed | the master died with the method `lost-knowledge`; the resource ran out `ran-out`; the trade road moved `trade-moved`; buyers went elsewhere `trade-moved`; *deliberately ruined* (Return or faction epilogue only) `broken-on-purpose` | find the apprentice who learned half the method (persuade; *master died*); rebuild the method by trial (craft; *master died*); find a new seam or source using the old hands' knowledge (craft; *resource ran out*); talk a merchant into routing back through (persuade; *road moved* or *buyers went*); organize the town to put up what a new start needs (gather); find a new trade suited to what they have (substitute) |

- **Separation rule:** doesn't apply. This family has no opposed economic pair.
- **Every cause leaves at least 3 ways across 2 or more kinds.** Pools keyed to a cause may list up to 6 ways, as long as any single cause leaves 3–5 (rulebook §2.3). For *deliberately ruined*, the gather and substitute ways apply, plus *win back the buyers or trade the ruin drove off (persuade)*.
- Any other person a way names (the voucher, the merchant, the neglectful owner) is an optional slot, filled only if that way is rolled (rulebook §2.4).

## 5. Hard parts
| Hard part | Fits |
|---|---|
| The one person who can fix it won't | any; fills the holdout slot |
| Burned once: an earlier fix failed or was a swindle, so nobody will give labor or coin again | Broken works, Lost livelihood |
| Nobody can agree who pays | Broken works, Lost livelihood |
| Pride: those involved won't admit it's broken | Lost good name, Estrangement |
| A deadline | any |

## 6. Twists
- **None.**
- **Someone wants it broken:** a profiteer benefits from it staying broken (the ferryman since the bridge fell, a rival who took the trade). They profit from it, but never caused it.
  - *Requires:* a profiteer who owns a substitute service or has a rival link to the requester.
  - Shape: `still-profiting`, the same mechanism as Reckoning's *still profiting* (rulebook, *Shapes*).
- **It broke for a reason:** the old version did harm. Maybe the bridge was built on taken land, the good name hid something real, or the estrangement protects someone.
  - *Requires:* a harm fact in the truth. For Lost good name, the cause must be *true but past* or *exaggerated*, so there the twist is rolled before the cause (rulebook, *Roll order*).
- **Can't go back:** only a changed version is possible, and the requester insists on the old one.
  - *Requires:* the requester's reason is legacy or pride.
- **One person pays:** the fix demands a real sacrifice from the named member (their land for the bridge footing, their savings, their secret).
  - *Requires:* a named member.

- **Tells** (rulebook rule 8, v7): *Someone wants it broken* (*Someone's hiding something*) names one public fact that doesn't fit the told story in `details.tell` (the ferryman's new boat). *It broke for a reason*, *Can't go back* and *One person pays* show theirs in the pitch, the place, or every rolled way.

| Twist | Broken works | Good name | Estrangement | Livelihood |
|---|---|---|---|---|
| Someone wants it broken | ✓ | ✓ | ✓ | ✓ |
| It broke for a reason | ✓ | ✓ | ✓ | ✓ |
| Can't go back | ✓ | ✓ | ✓ | ✓ |
| One person pays | ✓ | ✗ | ✓ | ✓ |

## 7. Possible endings
Each with its default **`costs`**, who loses by it (rulebook rule 7, v7). Restore has no opposing party on most cards, so its "other side" is the profiteer or the holdout, when there is one. What failure endings write is mechanics §4.
- Restored as it was (costs: the profiteer, when there is one; otherwise the named member or the requester, who gave labour or coin)
- Restored in a changed form (costs: the requester, who loses the old form)
- Replaced by a substitute (costs: the named member, or whoever the old thing served)
- Left broken (costs: the requester)
- Abandoned (people move on) (costs: the requester's community)
- **the player profits** (v7, B7): it is fixed, but on the player's terms: a toll, a share, the gathered funds kept (costs: the requester). Broken works and Lost livelihood only *(Claude)*.

## 8. Standing effects
- **Shaming the owner** is removed if the requester is the owner or is 2+ tiers below them (the same tier-gap test used for contest ways).
- **Low communities gather labor, not coin.**

## 9. Registry interactions
- **Creates:** works places, named members, the holdout, and the profiteer.
- **State conflict rule (rulebook):** a Restore card can't mark a place as broken if another card needs it working.
- **Reputation stains** can be reused only when the stain is a **public** recorded fact.
- **Bonds** need an existing link between two registry people.

## 10. Test rolls
*(Illustrative only; no shared registry.)*

**Roll 1 — Broken works + Someone wants it broken**
- **Setup:** A river bridge washed out. Cause: built badly.
- **Profiteer:** a ferryman (Low), now the only way across.
- **Ways rolled:**
  - why it really failed (craft)
  - trade labor (gather)
  - another crossing (substitute)
- **Holds.**

**Roll 2 — Broken works, High requester (sympathetic)**
- **Setup:** Lord Penrick (High; reason: dependents) asks for help with his own failing mill dam. The tenants who grind there depend on it.
- **Hard part:** burned once. His last mason took the money and left.
- **Ways rolled:**
  - find the old builder (persuade)
  - why it really failed (craft)
- **What the filters did:** "Shame the owner" was removed because the requester is the owner.
- **Holds.**

**Roll 3 — Lost good name + It broke for a reason**
- **Setup:** A retired watch sergeant is shunned as a coward. Cause: true but past — he ran from a fire that killed two.
- **Effect:** the truth-hunting ways are removed.
- **Ways rolled:**
  - public act (craft)
  - someone respected to vouch (persuade)
- **Holds.**

**Roll 4 — Estrangement + One person pays**
- **Setup:** Two brothers (Middle) haven't spoken since one sided with their father's creditor in a quarrel. Deadline: the elder sails in a month.
- **Named member:** the younger brother's wife, who must admit she carried the message that started it.
- **Ways rolled:**
  - carry words (persuade)
  - bring in the people who loved them both (gather)
- **Holds.** There's no stake, so it isn't a Dispute.

**Roll 5 — Lost livelihood, resource ran out**
- **Setup:** A quarry town's best stone is gone.
- **Ways rolled:**
  - new seam from the old hands' knowledge (craft)
  - organize the town (gather)
  - new trade (substitute)
- **What the filters did:** the apprentice and merchant ways were filtered out by cause.
- **Holds.**

## 11. Changes from review
- **Craft is redefined** as "work out why it broke." It no longer means fetching materials.
- **Ways are now keyed to causes.** Each premise has a cause list.
- **New card fields:** requester reason, named member, holdout, profiteer.
- **The profiteer never caused the break.** That keeps Restore from overlapping Investigate.
- **Estrangement has its own causes** and can't be over a stake, which kept it from becoming a Dispute.
- **New hard part: "burned once."** It replaced "materials on someone else's land," which was a fetch quest in disguise.
- **"A thing" was dropped as a broken-thing kind.** No premise used it.
- **v3 (final audit):**
  - Added the Return-only *deliberately broken/ruined* causes.
  - Estrangement excludes acknowledged wrongdoers.
  - A rumor's source is careless, never malicious.

## 12. Changelog
- **2026-10-27 (v3.2), v7: Laz-approved book rules (A2, A4, A8, B7).** New ending *the player profits* (B7); default `costs` on every ending (A4); requester reasons are rows of the shared list (A8); tells (A2).
- **2026-10-27 (v3.1), v6: World C fixes.** A Restore card may be a faction-story reveal only when the harm was done off the card (§1); *obligation* may reuse a local creditor (§2); every cause carries its shape, and *the stain is false* is marked *(confirming)* (§4; rulebook, *Shapes*); *Someone wants it broken* carries the shape `still-profiting`; *It broke for a reason* on Lost good name is rolled before the cause.
