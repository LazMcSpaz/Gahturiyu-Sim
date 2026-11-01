# Family: Threat — Locked (v3.3)

## 1. Situation
**Something is endangering a community, it is getting worse, and the community can't stop it alone.** With *Go and look* (v7), the community has seen a sign of something out there and can't act until someone goes and sees what it is.
- **vs. Investigate (*Watch them*):** *Go and look* scouts a place nobody knows; *Watch them* watches a known person. Neither looks for a culprit.
- **vs. Investigate:** the danger is visible. Finding a *cause* can be one way through; finding a *culprit* never is.
- **vs. Dispute:** the threat doesn't want what the community has. The community is collateral damage, or prey.
- **vs. Rescue:** no one is held.
- **vs. Reckoning:** the harm is ongoing, not finished.

## 2. Card fields
- **Requester:** speaks for the community. **Reason** (rulebook §7, the shared list; v7): *survival*, *dependents*, *duty*, *fear*, *faith* or *legacy*; every card fills `reasons.requester`.
- **Community:** a place (hamlet, village, street, camp, farmstead cluster). Its owner is whoever has authority over it. When the place is reused, it keeps its existing owner.
- **Named members:** 1–2 people besides the requester. *Divided community* needs 2.
- **Threat:** what is endangering them.
- **Truth:** what is actually driving the threat. For Raiders and Encroachment, the truth *is* the why-here (§5), and it is **public**; so is Protected beast's. A public truth never stands in for a twist (§6).
- **Encroachment's side-effect test is judged on the card,** from the encroacher's public purpose. A story-level intent can still sit in a main story's revelation.
- **Pressure:** what will be lost next.
- **Opposing party (required):** raider leader / preserve owner / source owner or travelers' spokesperson / encroacher, depending on the premise.
- **Optional slots,** each filled only when a rolled way or twist needs it:
  - band_lieutenant (Raiders)
  - patron (Raiders, *paid* why-here)
  - keeper (Protected beast, where it is **required**)
  - source owner (Sickness)
  - petition target (Encroachment)
  - helper (*Protector's price*)
  - neighbor headman (*Passed along*)
- Any other person a way names (tenders, a buyer, the petitioned authority) is an optional slot, filled only if that way is rolled (rulebook §2.4).
- **Premise, ways through, twist, possible endings.**
- ***Go and look*** (v7) *(Claude)* adds:
  - **The place out there:** a registry place within near reach that the community can't see into;
  - **The sign** (`details.sign`): what the community has seen, a past-event fact with `when` (smoke, lights at night, goats gone, a herder who came back frightened);
  - **The opposing party is the occupant:** whoever the truth puts there (a band's leader, a beast's owner, squatters' spokesperson); with *Gone bad*, the place's owner, who should tend it;
  - **guide** (optional): the one who has been there.

## 3. Kinds of ways through
- **Confront:** meet it head-on.
- **Undermine:** remove the cause, or weaken it from inside.
- **Protect:** shelter, harden, or move the community.
- **Bargain:** pay, appease, trade, or negotiate.

## 4. Premises

| Premise | Requirements | Ways through (roll 2–3, ≥2 kinds) | Separation |
|---|---|---|---|
| **Raiders:** a lawless band preys on the community | a lawless leader with a camp or hideout nearby; the leader isn't currently held or dead on another card | drive them off in a fight (confront); train and arm the locals (protect); cut off their patron or buyer (undermine; *requires truth = paid*); turn a lieutenant against the leader (undermine; fills band_lieutenant slot); offer tribute or honest work (bargain) | exempt (crime) |
| **Protected beast:** a creature killing herds or people belongs to a powerful neighbor, so killing it is a crime | a High-or-Top person owns an adjacent preserve place; the beast is a fact on that place; a Middle keeper | kill it quietly and hide what happened (confront); catch it alive and return it (confront); make the lord pay for the losses or cull it himself (bargain); get the keeper to admit the fences are down and mend them (undermine); move the herds off the preserve edge, giving up the best grazing (protect) | exempt (authority) |
| **Sickness or blight:** people, herds, or crops are failing | source rolled as the truth: bad water, a sick animal traded in, waste from upstream, spoiled stores, or carried in by travelers. **Never deliberate poisoning.** Carelessness, or shifting a nuisance onto neighbors, doesn't count as deliberate. The source is tied to a named source owner (whose livelihood causes it) or to the travelers' spokesperson | trace the source along the water, trade, or herd (undermine); talk the person behind the source into stopping or moving (bargain); quarantine the sick and find someone willing to tend them (protect); burn or cull what's infected and face the owner who loses it (confront) | n/a |
| ***Go and look*** (v7, C1): scout a dangerous or unknown place and come back with what's there *(Claude)* | a place within near reach the community can't see into (a ruin, a lair, a camp, a cave, a worked-out mine, the Overgrowth's edge), in the registry and **never** a place whose existence is secret, nor offshore while the sea is parked (rulebook §7); a sign at the community; a camp already known as raiders' takes *Raiders* instead | go in openly, armed, and see what stands against you (confront); slip in by the place's own way in, an old drain or a goat track, and look without being seen (undermine; *removed if fortified*); set a named member's herders as lookouts on the paths first, so whatever comes out is seen coming, then go (protect; fills a named member); pay the one who has been there, a charcoal burner or a trapper, to take you where they saw it (bargain; fills guide; names the price); walk in with a gift or a trade, and talk (bargain; *removed when the truth is Gone bad*: no one to talk to) | n/a |
| **Encroachment:** a powerful neighbor's project on their **own** adjacent land will wreck the community as a side effect (a quarry undermining homes, a dam flooding fields downstream) | a High-or-Top person owns the adjacent place; the project is new; the harm is a side effect, not the goal | sabotage the works (confront); petition someone who outranks the encroacher (undermine; *removed when nobody does*: rulebook, *Outranking slots*); prove the project will fail or cost more than it's worth (undermine; *requires truth = the project is flawed*); win compensation or a changed plan (bargain); help the community relocate (protect) | exempt (authority; High/Top vs. Low is the point) |

## 5. Why here (Raiders) / Truth
Each value is a truth row, with its shape (rulebook, *Shapes*).

| Truth | Fits | Shape |
|---|---|---|
| *Hunger:* nothing left where they came from | Raiders (public) | `ran-out` |
| *Driven out* of their old ground (recorded as a fact, not a person) | Raiders (public) | `pushed-here` |
| *Paid* by a patron (a named slot, never hidden) | Raiders (public) | `someone-above` |
| *Opportunity:* the community is weak and unguarded | Raiders (public) | *(confirming)* |
| *The preserve is next door* (fixed) | Protected beast (public) | *(confirming)* |
| *Bad water* | Sickness (secret) | `harm-from-upstream` |
| *Waste from upstream* | Sickness (secret) | `harm-from-upstream` |
| *A sick animal traded in* | Sickness (secret) | `brought-in` |
| *Carried in by travelers* | Sickness (secret) | `brought-in` |
| *Spoiled stores* | Sickness (secret) | `neglected` |
| *Project flawed* | Encroachment (public) | `built-to-fail` |
| *Project sound* | Encroachment (public) | *(confirming)* |

- For **Protected beast**, the why-here is fixed, so its one row is confirming. That's fine: the card always rolls a real twist (§6).
- For **Sickness**, the truth is the source (§4).

**Go and look** (v7, C1) *(Claude)*. The truth is **what's out there**, and it is **secret**: finding it is the job. Every way reaches it, since every way gets close enough to see. Each non-confirming row shows a tell at the community, on the card's face (rulebook rule 8); the column is the pattern, the story layer words it.

| Truth | What it is | Shape | Tell (pattern) |
|---|---|---|---|
| *As feared* | what the sign suggests is there, as bad as feared | *(confirming)* | — |
| *Made bigger* | the sign is real, but blown up: three hungry deserters, not a band | `made-bigger` | the tracks are all one size of boot |
| *Driven here* | what's there was pushed out of somewhere else; names where | `pushed-here` | they carry goods from a town two valleys off |
| *Sent here* | someone pays them to sit there; names who | `someone-above` | fresh-milled flour in a camp that grows nothing |
| *Gone bad* | the place itself is failing (a flooding mine, a cracking dam, the green coming through) and nobody tends it | `neglected` | the stream below it has run brown since spring |

## 6. Twists
- **None.**
  - *None* is never rolled on Raiders, Protected beast or Encroachment, because their truth is public (rulebook, *No blank card*). Those cards always roll a real twist; if none fits, the premise is filtered out for that card. Sickness or blight's source and *Go and look*'s truth are secret facts, so *None* is allowed there.
- **Tells** (rulebook rule 8, v7): *Passed along* (*Someone's hiding something*) names one public fact at the community that doesn't fit the told story in `details.tell`. *One of us caused it*, *Divided community* and *Protector's price* show theirs in the pitch or through every rolled way.
- **One of us caused it:** a named member caused the threat, and the rest of the community doesn't know. This is stated in the card's fixed fields, so it isn't a mystery. The question is what to do about it (expose them, cover for them, make them pay), not who did it.
  - *Requires:* a named member with the trait *has a secret*. The truth links to them: they cheated the raiders, poached the preserve, sold the herd that brought the sickness, or petitioned for the dam.
- **Divided community:** two named members want opposite answers (fight vs. pay, stay vs. leave), and the requester speaks for only one of them.
  - *Requires:* 2 named members with opposing views.
- **Passed along:** a neighboring community solved this by pushing it here. They paid the raiders to move on, drove the beast toward these herds, moved their tannery upstream, or got the project moved off their land. If the raiders' patron is that neighbor, this twist covers it, and "cut off the patron" isn't rolled separately.
  - *Requires:* a neighboring Low/Middle place with a named headman.
  - Shape: `pushed-here`, the same as Raiders' *driven out*, so the ledger sees them as one mechanism (rulebook, *Shapes*).
  - On *Go and look* it takes the row *Driven here*, so it is rolled before the truth (rulebook, *Roll order*).
- **Protector's price:** the only one strong enough to help demands something the community can barely give: their land, their obedience, or a member handed over. This is a demand on the card. What happens if it's paid is parked for the mechanical layer.
  - *Requires:* a helper who outranks the requester. The helper must not be held, a fugitive, or dead on another card.

| Twist | Raiders | Protected beast | Sickness | Encroachment | Go and look |
|---|---|---|---|---|---|
| One of us caused it | ✓ | ✓ | ✓ | ✓ | ✓ (a member goes out there: trades with them, feeds the beast) |
| Divided community | ✓ | ✓ | ✓ | ✓ | ✓ (look, or leave it be) |
| Passed along | ✓ | ✓ | ✓ | ✓ | ✓ (takes *Driven here*) |
| Protector's price | ✓ | ✓ | ✓ | ✓ (a rival lord) | ✓ (only the lord's huntsmen know the ground) |

## 7. Possible endings
Each with its default **`costs`**, who loses by it (rulebook rule 7, v7). What failure endings write is mechanics §4.
- Threat ended (costs: the opposing party; the one ending that costs only the other side)
- Threat driven off elsewhere (costs: with *Passed along*, the neighbor headman, back toward whom it goes; otherwise the requester, since it may come back)
- Community protected, but the threat remains (costs: the named members, who live beside it)
- Community relocated (costs: the requester and the named members: their home)
- Bargain struck (costs: the requester: the price; with *Protector's price*, whoever the price falls on)
- Community lost (costs: the requester)

**Go and look** (v7) *(Claude)* replaces the list above with:
- came back with all of it: the community knows what's there (costs: the occupant; the one ending that costs only the other side)
- came back with part of it: seen, or turned aside partway (costs: the community, who act half-blind)
- dealt with it there: the player ends it on the spot, as *Threat ended* (costs: the requester: the community didn't choose it, and any answer falls on them)
- bargain struck (costs: the requester: the price)
- kept it quiet: the player tells the community less than they found (costs: the requester)
- turned back (costs: the requester; the failure ending)
## 8. Standing effects
- **Low communities can't fund** tribute or compensation. The cost shifts to the player, or is paid in labor.
- **Petition** needs someone who outranks the encroacher. It is removed when nobody does (rulebook, *Outranking slots*), which is always the case for a Top encroacher.
- **The helper outranks the requester.** That is their leverage.

## 9. Registry interactions
- **Creates:**
  - the community place
  - named members
  - raider leader and camp
  - lieutenant
  - patron
  - preserve and its owner and keeper
  - source owner
  - encroacher and project place
  - helper
  - neighbor place and headman
- **Beasts** are a fact on a place. A second Protected-beast card on the same preserve must reuse that fact.
- **Reuse targets:**
  - raider leaders (Rescue's *Kidnapped* captors)
  - lords as preserve owners or encroachers
  - mercenary captains as helpers
- **Always roll a fallback** for a reused raider leader or helper.
- **Secret facts stay secret.** A reused person's secret (such as a hidden motive) can't be stated openly here.

## 10. Test rolls (v2)

**Roll 1: Raiders + Passed along**
- Hask (**reused**, now on his 2nd quest; fallback rolled) raids Millbrook farms (Low).
- Truth: a neighboring hamlet paid Hask to move on from them.
- Ways rolled:
  - train the locals (protect)
  - honest work at harvest (bargain)
  - turn a lieutenant (undermine; band_lieutenant slot filled)
- **Holds.** "Cut off the patron" was filtered out, because the patron is the neighbor and that's the twist's job.

**Roll 2: Protected beast + One of us caused it**
- A lord's preserve borders a herding hamlet.
- Truth: the hamlet's best herder poached a stag last winter and cut the fence to drag it out. That's how the beast got through.
- Ways rolled:
  - catch it alive (confront)
  - get the keeper to mend the fences (undermine)
  - move the herds (protect)
- **Holds.** Mending the fences risks exposing the herder.

**Roll 3: Sickness + Divided community**
- Mining camp. Truth: a new shaft opened bad water. Source owner: the foreman.
- Named members:
  - the foreman (Middle), who wants to keep digging
  - a miner's widow (Low), who wants the shaft sealed
- Ways rolled:
  - trace the source (undermine)
  - talk the foreman into stopping (bargain)
  - quarantine and find tenders (protect)
- **Holds.**

**Roll 4: Encroachment + Protector's price, with a Top encroacher**
- The pre-made governor's dam floods a valley hamlet downstream.
- Petition is removed (no one outranks the governor). Truth: the project is sound, so "prove it will fail" is removed too.
- Remaining ways: sabotage (confront), compensation (bargain), relocate (protect).
- Helper: a neighboring lord (High), who demands the hamlet's obedience.
- **Holds** with 3 ways left. This is the tightest case.

**Roll 5: Sickness (travelers) + Passed along**
- A market town (Middle). Truth: travelers camped outside the walls carried in the sickness. They were turned out of the neighboring town first.
- Ways rolled:
  - quarantine and find tenders (protect)
  - talk the travelers' spokesperson into moving (bargain)
- **Holds.**

**Roll 6: Encroachment, High party sympathetic**
- Lord Edevane (High) is building a flood dam on his own land to save his tenants' fields. That makes him the sympathetic side. It will drown a smaller hamlet's ford downstream.
- Truth: the project is sound.
- Twist (*None* isn't allowed here, §6): *Divided community*: the hamlet's ferryman wants to take the money and go; his sister wants to stay.
- Ways rolled:
  - win a changed plan (bargain)
  - help the hamlet relocate (protect)
- **Holds.** The encroacher is the sympathetic one.

**Roll 7: Go and look, no twist** *(Claude)*
- A charcoal-burners' hamlet (Low) has seen lights in the old lime kilns above it for a week, and a boy's dog came back cut.
- Truth: *Sent here*: four men a timber lord pays sit in the kilns, marking which trees the hamlet cuts. Tell: they burn sea-coal, which nobody up here can buy.
- Ways rolled:
  - slip in by the kiln flues and look (undermine)
  - pay the old burner who went up there to show the way (bargain; guide filled)
- **Holds.** Coming back with all of it names the timber lord; saying less keeps the hamlet out of his way.

## 11. Changes from review
- **Beast → Protected beast.** The old premise was a kill quest.
- **Twists replaced:**
  - "Fleeing something worse" → **Passed along** (the stock twist had no payoff on the card).
  - "Community caused it" → **One of us caused it** (avoids the eco-cliché and gives the player a choice).
- **Encroachment is a side effect on the encroacher's own land.** The deliberate version overlapped Dispute.
- **The truth now drives the why-here.** Ways that need a patron, a flaw, or a lieutenant are filtered out unless the card supplies one.
- **Ownership demands stay demands.** What happens when they're met is parked for the mechanical layer.
- **v3 (final audit):**
  - The opposing party is a required slot.
  - "Never deliberate" is clarified.
  - The patron-neighbor rule moved into the twist.
  - Mechanical wording removed.
  - High-sympathetic roll added.

## 12. Changelog
- **2026-10-27 (v3.3), v7: Laz-approved book rules (A2, A4, A8, C1).** New premise *Go and look*: scouting a dangerous or unknown place and reporting what's there, here rather than in Passage because a scout carries no cargo (rulebook §8); its fields, five ways, secret truth rows with shapes and tells, twists and endings (§1, §2, §4, §5, §6, §7, roll 7); default `costs` on every ending (A4); requester reasons from the shared list (A8); tells (A2).
- **2026-10-27 (v3.2), v6: World C fixes.** One truth table for all four premises (§5), each row with its shape; *opportunity*, *the preserve is next door* and *project sound* marked *(confirming)*; *Passed along* carries the shape `pushed-here` (rulebook, *Shapes*); the petition way follows rulebook *Outranking slots*.
- **2026-10-26 (v3.1), audit fixes.** *None* is never rolled on Raiders, Protected beast or Encroachment, whose truth is public (§6; rulebook, *No blank card*).
