# Proposals from the reading

For Laz, 2026-10-27. Five researchers read *Procedural Storytelling in Game Design* (Short & Adams, 2019), Doran & Parberry's quest paper (2011) and Emily Short's "Beyond Branching" (2016). Their notes (`notes-A` to `notes-E`, in this folder) are merged here, checked against today's rules, and ranked. Page numbers are in the notes.

**Terms.**
- A *card* is one quest. Its *truth* is the hidden fact behind it. A *twist* makes it harder than it looks.
- A *shape* is the mechanism a truth turns on ("water spoiled uphill").
- *Turning cards* are where a story turns: chapter ends, the Ordeal, a guild's reveal, a crisis.
- A *plant* is a fact set early for a later card to pay off. A *quiet plant* looks innocent at first.
- A *kit* is a guild's table of parts. The *heart* is the ordinary person a story is for.

**Already closed by today's docs:**
- World C's seven water cards are now one shape, with a cap for the whole world (rulebook §9a). That limits how many, not how close (A1).
- Main moves have despite-you faces, and the Return varies by how the climax ended.
- The kit roller already reports shares over many seeds (`--stats`). Notes D say it was never run, which is wrong. It lacks a pass mark (A10).

**Effort:** S is under an hour of edits plus a small script; M is several docs or a page of kit data; L touches every way template.

**When:** *before D* means it changes how World D's guild and side cards get written. *After D* means it can be judged on D's results. D's main story is already built, so main-only items are *after D*.

---

## A. Fits today's scope: a rule or a check, no new field or a tiny one

**A1. Space out repeats along the road.** *S · before D.* Sources (2): Avellone (Foreword) and Compton (ch. 1), "the illusion of randomness"; Maher (ch. 6).
- *Fixes:* caps count how often, not how close together. In World C, "wants off the ladder" came twice two days apart, and "wants to see him break" twice in a fortnight. The new shape cap (8 at N=220) still fits all eight on one road.
- *Change:* locations §3, new *Spacing* rule. Two cards within about 2 days' travel share no shape, twist or family + premise, unless they are one chain.
- *Check:* a script tests every card pair within 2 days on the map. It should find none.

**A2. Hidden truths show a tell before the choice.** *S · before D.* Sources (3): Epstein (ch. 18) and Rouse (ch. 14); T. Short (ch. 21) and Kim (ch. 24); Murray (ch. 3).
- *Fixes:* outside Investigate, nothing on a card's face jars with the requester's story, so twists feel like dice. And a dilemma twist found by only one way leaves a player on the other ways choosing blind.
- *Change:* rulebook rule 8. A card with a non-confirming truth, or a *Not what it seems* or *Someone's hiding something* twist, names one public fact that doesn't fit the told story (the "kidnapped" daughter's chest is gone). The fact goes in `details.tell`. It points, and never states.
  - *The other side is right*, *They don't want it* and *Innocents pay* show their tell in the pitch, in the place, or through every rolled way.
  - *They don't want it* shows as an act ("she hides from her rescuers").
- *Check:* a script finds the tell in the card's public text. A reviewer passes it if it seems odd and fails it if it gives the truth away.

**A3. Quiet plants: findable, named again, and not the only odd detail.** *S · before D.* Sources (2): T. Short (ch. 21) and Kim (ch. 24); Avellone (Foreword).
- *Fixes:* two risks to the guild reveal, our best part (8/10 in World C). A plant buried in data is missed, and the reveal turns into a lecture. And if only plant cards have odd details, players learn to spot them.
- *Change:* faction-stories §6.
  - The plant sits in the card's text and is concrete.
  - The reveal repeats the routine card's own words for it.
  - Most hall jobs carry a harmless detail of the same kind (`details.noted`).
- *Check:* by script, each reveal matches at least 2 plant nouns, and `noted` is on 80% of hall jobs.

**A4. Every choice on a turning card costs a named person.** *S · before D.* Sources (2): Fijak & Stokalski (ch. 19); Kim (ch. 24).
- *Fixes:* nothing checks that every ending hurts someone. One clean win turns a dilemma into decoration.
- *Change:* rulebook rule 7. Each choice ending on a turning card names `costs`, the slot who loses. At most one ending costs only the other side. Family docs give the defaults.
- *Check:* by script.

**A5. No empty failure.** *S · before D.* Sources (2): Kline (ch. 10), "fail forward", and Ingold (ch. 7); Lumpkin (ch. 20).
- *Fixes:* mechanics §4 gives effects for only two failure endings. The rest read as "nothing happened".
- *Change:* mechanics §4. Every failure, walk-away or unattended ending writes a state on a named person or place: the captive is sold on, the works are abandoned. About one default in five carries `generate`.
- *Check:* by script, no failure ending only closes the card.

**A6. Some hall jobs grow from the story's moves.** *S · before D.* Sources (2): Fijak & Stokalski (ch. 19), "two separate games"; Short (*Beyond Branching*).
- *Fixes:* hall jobs may mention a move, but none comes out of one. The nine jobs feel like a separate game.
- *Change:* locations §4.
  - From the middle band on, each band has a hall job that reads the latest move.
  - One middle job and one late job are made by the move: poached clients want their old contracts honoured.
  - These jobs still use no story cast and no secrets.
- *Check:* by script, 2 such jobs per guild, with no story cast in them.

**A7. The enemy is seen in passing first.** *S · before D.* Source (1): Keenan & Hill (ch. 8), "creating suspicion".
- *Change:* locations §3. In each region a story crosses, at least 2 side cards cast someone from the enemy's faction in an innocent public role, such as a client or a rival bidder. Never a proxy, never a secret.
- *Check:* a script counts those cards opening before the reveal.

**A8. One shared list of requester reasons.** *S · before D.* Source (1): Doran & Parberry.
- *Fixes:* checked today, four guild profiles weight up reasons no family list holds, so those weights do nothing:
  - Writers: *curiosity*;
  - Night Market: *money and freedom*;
  - Dueling Society: *proof and peace*;
  - Holy House: *faith, duty and fear*.
- *Fixes, too:* five families have no requester reason, so the pitch has no "why".
- *Change:* rulebook §7 gets one shared list. Each family names its rows. Every card fills the existing `reasons.requester`.
- *Check:* by script, every motive word is a row, and no reason fills more than 30% of a world.

**A9. A town's cards use the town's own features first.** *S · before D.* Sources (2): Dias (ch. 16); Short (*Beyond Branching*).
- *Fixes:* towns of one kind lean alike, so the ports read the same.
- *Change:* locations §3. A premise this town's own landmark, trade or site can fill rolls at double weight. Each feature is used once before any is used twice.
- *Check:* each `place_tie` names something of its own town.

**A10. A pass mark for kit shares.** *S · before D, the report only.* Sources (4): Compton (ch. 1) and Cook (ch. 5); Dias (ch. 16); E. Short (ch. 28); Short (*Beyond Branching*).
- *Fixes:* a 400-seed run today. The Writers' crisis C8 never rolls, while C7 takes 37% and enemy W8 28%. The Lantern's C3 rolls 2% and L3 3%. Faction-stories §11 still says "a few hundred spines", but the kits measure 25–59.
- *Change:* faction-stories §11, *Share band*. Each enemy and crisis rolls between ⅓ and 2.5 times its even share. A part outside the band gets wider tags first, then a `weight`.
- *Check:* `--stats` covers every pool and lists parts that never roll. Don't reroll D's kits.

**A11. Examples show shape, not wording.** *S · before D.* Source (1): E. Short (ch. 28).
- Our agents copy what they read.
- Rulebook §11: test rolls must differ in premise, twist group, standing and place.
- `CONTRACT-D.md`: never reuse an example's names, objects or openings.
- *Check:* a reviewer finds no two test rolls alike on all four.

**A12. The main story gets quiet plants, one of them an absence.** *S · after D.* Source (1): Epstein (ch. 18).
- *Fixes:* main plants are told, not noticed. World C's Return twist came from "a son I never met".
- *Change:* main-quest §7. On every path, two cards between the Hook and the Trials carry a quiet plant. One is an absence: an empty berth, a name missing from a roll. The revelation depends on both.
- *Check:* a script walks each path and counts them.

**A13. Audit who casting makes villains and victims.** *S · after D, run on D.* Source (1): Cook (ch. 5).
- Over 20 seeds, a checker lists opposing and harmed roles by people and sex.
- It flags any group above 1.5 times its share.
- Canon-fixed roles (kit OH4's Qotiro captives) are reported apart.

**A14. A coverage matrix.** *M · after D.* Sources (2): Horneman (ch. 4); E. Short (ch. 28).
- A script crosses every premise with every place kind, standing and access. It flags cells with under 3 ways, under 2 kinds, or no hidden truth.
- D's main log found four holes like this by hand today. One: *justified captor* removes the only ways to a guilty truth.

---

## B. Small new data: a field or table on cards, kits or people

**B1. Hearts have a want and a past.** *M · before D (guild hearts).* Sources (3): Murray (ch. 3), "shorter was better"; Epstein (ch. 18); Saltsman (ch. 23), T. Short (ch. 21) and Card et al. (ch. 22).
- *Fixes:* each heart is only her tie to the stake. World A's critic said she "has no want of her own on any card".
- *Change:* a person gets two fields: `want` (something she seeks, never "not…") and one `past` event. Each kit heart type gets a want. It shows on her routine card and once more later. It never decides a way.
- *Check:* by script, the want appears in the text of 2 cards and never starts with "not".

**B2. Recurring people carry their history.** *M · before D.* Sources (3): Kazemi (ch. 2); Grinblat (ch. 15); T. Short (ch. 21).
- *Fixes:* reuse across stories reads as coincidence. World C's Agnes buys readers on a main card and a Writers card, and "nothing at Firststone ever asks why". When reuse did connect (Orrin Vane, three times), it was the best moment of the playthrough.
- *Change:* casting §6.
  - The later card records `recalls`: one public fact about the person from the earlier card.
  - A public grudge or debt supplies their reason.
  - Echoes prefer someone a later move touched.
- *Check:* by script, every person reused across stories has `recalls`. A reviewer samples 10 to see if each reads as one life.

**B3. Later cards read how the card before ended.** *M · before D.* Sources (2): Horneman (ch. 4) and Avellone (Foreword); Ingold (ch. 7).
- *Fixes:* rule 11 keeps chained cards silent about results, so chapters read "and then", or get it wrong. World C credited the player with both sibling targets after they had played only one.
- *Change:*
  - Each family sorts its endings into 2–3 ending groups.
  - A card that opens on another's resolution carries `after`: one reading per group.
  - After a story's reveal, its open cards prove the truth instead of discovering it.
- *Check:* by script, every chained card covers each ending of the card before.

**B4. Side cards point onward.** *M · before D.* Sources (2): Ingold (ch. 7); Doran & Parberry (quests nest).
- *Fixes:* World C's Longwall had 7 side quests, and "nothing I played points there".
- *Change:* locations §3.
  - Each side card names a `lead`: a nearby card whose people are linked to its own.
  - A favour asked of a nearby requester names that requester's card.
  - Every town is pointed to.
- *Check:* a script draws the lead graph and lists towns nothing points to.

**B5. The code bends before it breaks.** *S · before D.* Source (1): Fijak & Stokalski (ch. 19).
- One chapter-2 card rolls a lesser code-break of the crisis's kind, such as keeping back one name.
- Rule 3 allows it as an extra way.
- The crisis never assumes the player took it.
- *Check:* by script, exactly one per guild story, of the crisis's kind.

**B6. A sign before each move, and one mark for the enemy.** *M · after D.* Sources (2): Lumpkin (ch. 20); Grinblat (ch. 15).
- Each move gets a `sign`: a smaller public fact, a layer earlier.
- Each enemy gets one public `mark` (a wax colour, a broker's name) on every move.
- *Check:* by script, each sign comes before its move and is read by an earlier card.
- Ask D's reviewers whether moves land cold.

**B7. Endings for the wrong guess and the selfish choice.** *S · before D.* Source (1 reader): Maher (ch. 6), Kline (ch. 10).
- Investigate gains *the wrong one accused*.
- Rescue gains *handed to another*.
- Restore gains *the player profits*.
- Dispute gains *the player takes the stake*.
- Their effects go in mechanics §4.
- *Check:* the endings are in the family lists, with defaults in §4.

**B8. Ladder callbacks.** *S · after D.* Source (1): Short (*Beyond Branching*).
- A later hall job may add one way, locked until an earlier job's ending is true.
- *Check:* by script, never needed for the truth.

**B9. Ways name the game verb and the ending they reach.** *L · after D.* Sources (3): Keenan & Hill (ch. 8); Saltsman (ch. 23); Doran & Parberry.
- Each way template gets `acts`, taken from the sim's orders, and `ends`.
- This is needed before the mechanics layer.
- Laz confirms the verb list.
- *Check:* by script, every kept ending is reachable by a way.

**B10. A story's theme shows on first read.** *M · after D.* Source (1): Cook (ch. 12).
- Each story rolls one canon tension.
- It shows on the story's first card, and a crisis ending decides it.
- *Check:* a reader names each story's theme without seeing the tags.

---

## C. New feature: Laz decides
- **C1. Spying, scouting and field-work quests.** These are the common quest kinds nobody covers yet. Doran & Parberry. *M.*
- **C2. A story's end account.** Three closing lines: the heart, the place, the other side. Rouse (ch. 14); Fijak & Stokalski (ch. 19). *M.*
- **C3. Standing mysteries.** One rumour per region about hidden lore, never resolved. Epstein (ch. 18). *S.*
- **C4. Routine guild jobs in any order,** for less criss-crossing. Short. *S.*
- **C5. The world remembers how you solved it,** by bribe or by fire, for the grudge loop. Kline (ch. 10). *S.*

---

## Recommendation
1. **Before World D's guild and side cards:** A1–A11, plus B1, B5 and B7. All are small, and they shape exactly the cards D is about to write.
2. **Hold B2–B4 one round.** They suit D but take more work. Judge them on D's coherence and map scores, then apply them for World E.
3. **After D:** A12–A14, B6 and B8–B10. C1–C5 go to Laz.

## Appendix: cut
- **Rarity weights:** no sign yet that loud twists wear out.
- **Traces of the past:** wait for the detail pass.
- **Reroll logs:** tooling, with no visible gain.
- **Name banlist:** for the Rust build.
- **★ rubric constraints:** A1–A11 add the scripts that matter.
- **Variants that show:** a review for when kits next widen.
- **Dry-spell test:** belongs to the planned mechanics test.
- **Moved people return:** mechanics.
- **Lever tempo:** low value.
- **Stance in openings:** Lane B's.
- **Several tellings of one event:** risks contradictions.
- **Casting margin:** a later dial.
- **Facts index:** Lane B's.
- **Who owns which quest kinds:** one overview paragraph.
- **Hall jobs shown a few at a time; test climbs:** wait for discovery.
