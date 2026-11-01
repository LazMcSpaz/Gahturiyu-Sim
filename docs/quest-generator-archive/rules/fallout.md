# Fallout Quests — Locked (v2.6)

## 1. What they are
- **A fallout quest is a side quest about what one of the villain's moves did to people outside the main story.** The main story shows a move landing on the heart's chapters. Fallout shows it landing on everyone else.
- **The move shapes the card.** It isn't just the backdrop.

## 2. Relatedness scale (now five levels)

| Level | Name | Shares with the main story | Pays back into it? |
|---|---|---|---|
| **4** | Main | chapters and the Return | — |
| **3** | Minor | a **person** from one chapter | yes, one advantage ending |
| **2** | Fallout | an **event**, one of the villain's moves | no |
| **1** | Echo | a **person** from any story card, main or faction (never cast, a proxy, or the stake); from a main story, prefers people from opening or middle cards, or from chapters in layers of width 2 or more (assembly §8) | no |
| **0** | Independent | the region only | no |

## 3. Rules

### The move
1. **The move fills a requirement.** A fallout card names one source move, and that move must fill one named field on the card:
   - a cause (Restore, rule 1a)
   - a seal type (Acquire)
   - a ban or sickness fact (Passage)
   - a reason
   - a why-here (Threat)
   - a past-event taking (Acquire, Passed on)
   - a debt fact
   - the goods or scheme in a swindle
   - the breach (Dispute, Broken agreement)
   - **Delete test:** if you removed the move and the card would still generate unchanged, the card is rejected.

   1a. **Restore in fallout** takes only three premises, each with set causes:
   - *Lost livelihood*, with a side-effect cause only (*buyers went elsewhere*, *road moved*, *resource ran out*);
   - *Estrangement*, with its normal causes;
   - *Lost good name*, on the mirror only (rule 14).
2. **It never undoes the move.** No fallout ending reverses it.
   - Freeing people a move took is a new event, not undoing the move (rule 7a).
   - **It holds whichever face plays.** A move may have a despite-you face (main-quest, *Wins stick*). Both faces leave the same public fact, and the fallout card stands on that fact, never on the wording of one face.

### How many
3. **The count follows the move's scope:**

| Move's scope | Fallout cards |
|---|---|
| household | 0 |
| community | 1 |
| region | 2 |
| realm | 2–3 |

   - **Late fallout:** each region- or realm-scope move also gets one extra card tagged `checkpoint: k` (`quest-locations.md` §6).
   - **Faction stories** use their own count (`quest-faction-stories.md` §8): a move that reaches beyond the guild gets at least one card, set in a settlement it touched (faction-stories, *Faction fallout is required*); a move that stays inside the guild may get one only if it is region scope; at most 1 per move, so at most 3 per story.
   - **In faction-story fallout,** read "main card" and "main cast" throughout these rules as that faction story's cards and cast (faction-stories §8).
   - If no legal card exists, the count is **0**. That's normal for captive or estranged ordeal events, where everyone affected is already in the main story.

### Who can be in it
4. **Kept out of fallout:**
   - the main cast
   - the stake
   - **every proxy:** anyone linked to a cast member, or any member of the antagonist's faction who appears on a main card
   - any person or place that fills a slot on a main card or a minor quest
   - anyone in a group the move took or still holds, except as the captive of the taking's Rescue (rule 7a)
5. **Bystander members.** A *new* member of the antagonist's faction, with no link to any cast member, may appear in fallout as a **bystander member**.
   - They are **not a proxy**, and they don't count as main-story presence.
   - They may never later fill a slot on a main card.
   - This is a narrow exception to the main-quest rule that every faction member is a proxy.

### Scope and state
6. **Scope** is at most community, and never larger than the source move. Foundational entries are never used (main-quest §4).
7. **No state clashes.** A fallout card (its premise, ways, and endings) may not give any entry used by a main card, a minor quest, or the Return a state of broken, seized, sick, captive, or dead. It may not repair or free any such entry either.

   7a. **The taking's Rescue.** Every move that takes people gets a Rescue downstream (main-quest, *Every taking gets a rescue*). When no main card or minor quest rescues them, a fallout card may be that Rescue, with people the move took as its captives. They must fill no slot on a main card, a minor quest or the Return. It counts toward the move's fallout count and still passes the delete test.

### Secrets
8. **No secret leaks.** A fallout card's truth and cause may name the move's actor only as the move's public text does.
   - **Investigate** is allowed only when the culprit is a bystander (an opportunist), never the move's actor or anyone from their faction.
   - **Reckoning, Betrayal** ("who informed") is excluded, because the informer would be a proxy.

### Variety
9. **No repeating the move's own cards.** A fallout card may not share **family + premise** with any main or Return card that grows from the same move.
   - **Rulebook amendment:** moves are exempt from the "one card per family per past event" limit. This rule replaces it for moves.
10. **Across one story's fallout:** no premise repeats, and no family is used more than **⌈fallout count ÷ 4⌉ times (minimum 2)**. A flat cap of 2 became impossible once a big story had more than 18 fallout cards.
11. **Twist groups:** no twist group repeats across a story's fallout.
    - **Plain cards are allowed.** A fallout card may be plain (a confirming truth and no twist), within the world's share of plain side cards (rulebook, *No blank card*).

### What a story's fallout must include
12. **Three faces.** If a story has 3 or more fallout cards, at least one shows someone who **profits** from a move, one shows someone who **resists** it, and one shows someone who **endures** it.
13. **The enemy's own people.** At least one fallout card makes a bystander member the **requester** or the **wronged** party, not the opposition.
14. **Mirror.** Exactly one fallout card puts a **new** person in the heart's position against the same move.
    - The move's text may name others it struck alongside the heart.
    - If the door's family + premise is already taken by the move's main cards, use the same family with a different premise.
    - They're unlinked to the heart, and their situation uses the same door family the heart's chapter used.
    - It may use the cause *false, spread by someone off the card* (Restore) where the heart's door did.

### Building on earlier fallout
15. **Aftershock.** A fallout card from a later move may reuse a place or person from an earlier fallout card. It describes the state the later move left them in. Like linked pairs, it never assumes how the earlier card ended. Reuse caps still apply.
16. **Linked pairs.** Fallout cards may form linked pairs only with each other.

### Timing
17. A fallout card's premise assumes its move has happened. When it becomes available to the player, late fallout included, is mechanical-layer work (parked).

### Reuse
18. **Reuse:** rulebook §4 caps apply as normal.

## 4. Move kind → likely fallout (weights, not limits)

| The move... | Natural fallout families |
|---|---|
| burned, broke, or flooded something | Restore (*Lost livelihood*: a trade that depended on it), Threat (the damage spreads), Dispute |
| took people | Rescue (*Bound by debt*: families ruined by the loss; or the taken themselves, rule 7a), Restore (*Estrangement*: a side taken), Passage |
| seized a place or road | Reckoning (*Swindle*: opportunists), Investigate (a bystander thief), Acquire, Rescue (*Bound by debt*) |
| banned or disgraced someone | Dispute (*Broken agreement*), Passage (*Unwelcome cargo*), Restore (the mirror) |
| sacked or besieged a place | Threat (*Raiders*: the bands roam on), Acquire (*Passed on*: looted goods sold in good faith), Passage |

The Restore entries above follow rule 1a.

## 5. Sample: fallout from the Ilse story (main-quest §11)
**Moves and counts:**
- hook (community): 1
- threshold (community): 1
- trials (region): 2
- ordeal (realm): 2
- last move (realm): 2

**Total: 8 fallout cards.**

| Source move | Fallout card | Field the move fills | Face | Twist |
|---|---|---|---|---|
| Hook: *waystation burned* | **Investigate, Theft.** With the waystation gone, carters camp at the ford. A farrier's tools vanish from the camp. Culprit: a bystander teamster. | the scene (the camp exists because the waystation burned) | endure | none |
| Threshold: *bands take carters* | **Restore, Estrangement.** A Binder drover's wife (bystander member) and her Road Crews sister stop speaking after the sister's husband is taken. Deadline: a niece's wedding. | cause: *a side taken* | **enemy's own people** | *One person pays* (innocents pay) |
| Trials: *toll-house seized* | **Reckoning, Swindle.** A clerk sells fake "pass-papers" to carters desperate to get around the toll-house, then moves on to the next town. | the swindle's goods; shield *Fled* | **profit** | none |
| Trials: *toll-house seized* | **Rescue, Bound by debt.** The tolls ate a carter family's margins. A moneylender (Middle) holds their son (Low) in his yard until they pay. | the debt fact; separation OK | **profit** | none |
| Ordeal: *writ bars Ilse's seal* | **Dispute, Broken agreement.** A chandler (Middle) refuses to pay a weaver (Low) for cloth a middleman shipped under Ilse's seal. The weaver has no link to Ilse. | the breach | endure | *Law on the wrong side*: the chandler is legally right (the other side is right) |
| Ordeal: *writ bars Ilse's seal* | **Restore, Lost livelihood.** Hollin, another carter named in the same writ, watches his customers go elsewhere. Ilse's door was Restore *Lost good name*, so the mirror keeps the family and changes the premise. | cause (the truth): *buyers went elsewhere*; his old buyers quietly signed with a carter the writ doesn't name | **mirror** | none |
| Last move: *the Weir is sacked* | **Threat, Raiders.** With the Weir fallen, a loose Binder band (new leader, bystander member) roams the valley, and the next hamlet decides to fight. | why-here: *opportunity* | **resist** | *Divided community* |
| Last move: *the Weir is sacked* | **Acquire, Passed on.** A Weir survivor (one who escaped, not one of those held) finds her family's cart for sale. A merchant bought it in good faith from a looter. | the past-event taking | endure | none |

**Checks:**
- **Families:** Restore ×2, and every other family once (Investigate, Reckoning, Rescue, Dispute, Threat, Acquire).
- **No repeats:** no premise repeats, and no card shares family + premise with a main card from the same move.
- **Faces:** profit, resist, and endure are all covered. The enemy's own people and the mirror are both present.
- **Twist groups:** innocents pay ×1, the other side is right ×1, they don't want it ×1.
- **Who's in it:** no cast member, proxy, main-card entry, or held person appears.
- **Scope:** every card is community scope or smaller.

## 6. Decisions made without Laz
1. **Scale renumbered** from 0–3 to 0–4.
2. **Bystander members** are an exception to "every faction member is a proxy."
3. **Moves** are exempt from the one-card-per-family limit, with the family + premise rule replacing it.
4. **Count by scope:** fallout count depends on the move's scope.

## 7. Changes
- **v2.1 (full test):**
  - the Dispute *breach* was added to the move-fills list
  - mirror premise flexibility
  - a move's text may name other victims
- **v2.2 (final audit):**
  - echo wording now matches assembly
  - the sample mirror was corrected to the heart's door family
  - the hook fallout card was re-rolled
- **v2.4 (2026-10-26, v5: world-test fixes):**
  - echoes may come from any story card, main or faction (C3)
  - rule 7a: a fallout card may be the Rescue a taking needs (SC4, CO5)
  - fallout holds whichever face of a move plays (PD2)
  - pointer to required faction fallout (PD5, CO1)
- **v2.6 (2026-10-27, v6: World C fixes):**
  - rule 1a: the allowed Restore premises and causes are in the rules, not a sub-list under §4 (CM7)
  - rule 11: fallout cards may be plain, within the world's share (pointer to rulebook *No blank card*; SC-C1)
- **v2.5 (2026-10-26, audit fixes):**
  - rule 3: the faction-story count now matches faction-stories §8 (fallout for any move beyond the guild; at most 1 per move, 3 per story), plus one reading line for faction fallout (core H1, L9)
  - sample: the Hollin mirror has no twist (its truth is its cause), so *They don't want it* is used once, by *Divided community* (core M7)
