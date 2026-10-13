# Tutorial island and test quarter (Laz, 2026-10-12, revised)
One small island that is both **the game's opening** and **the dev test environment**. Like RuneScape's Tutorial Island, but it's the real start of the story.

## Laz's decisions
- **A small island offshore.** The player arrives **shipwrecked in a storm** and leaves by ship.
- **The tutorial is the opening.** It replaces the old opening (framed in a town, cleared by favours, rewarded with a plot).
- **No starting plot of land** (Laz). The player leaves the island with a debt or a guild contact, not property.
- **Teaches:** basics, survival, fighting, making, plus sneaking, looting and inventory.
- **Two branching paths** that share the same places, with different set dressing, time of day and people.
- **A hidden test quarter** on the same island: walled off, dev-only, opened with a debug key.
- **The island stays on the map;** the player can come back by ferry later.
- **About an hour** long.
- **Room to grow:** spare space so lessons and test pieces can be added or removed during testing.

## The island
- A labour island run around a **main compound**, where bonded debtors work. The authorities here don't care how you arrived: an outsider with no papers is put to work as a slave.
- **The slavers on the island are Qotiro (the Binders), but a Roduro noble calls the shots** (Laz). The island and the compound belong to **a Roduro Free Lord** (a new character), and the Binders run it for him.
- **He's visiting the island when the player is caught.** The player sees him give the orders, so they learn early that the Qotiro slavers are **tools**, not the whole story, and don't simply come away hating the Qotiro.

## The opening (shared by both paths)
1. **Shipwreck.** You wake on the shore in a storm.
   - **Teaches:** moving, the camera, weather (cold and wet, finding shelter).
2. **Shelter.** You find a hut with a fire and some raw food.
   - **Teaches:** cooking at a fire, the first crafting step, eating.
3. **The stranger arrives,** surprised you're there. He's **Diʻub**, a Roduro man and an **escaped debtor**: a slave hiding from the island's authorities and looking for a way off.
   - He lets you eat what you cooked and asks your story.
4. **Who you were.** Your answers to his questions shape who you were, and each answer gives a **small skill boost**. **Backgrounds are never named or shown as a choice** (Laz): the player just answers questions about their past ("Where were you headed?", "What did you do with your hands?", "Why were you on that ship?"), and the boosts follow quietly from the answers.
5. **His offer:** work together to get off the island.
   - **If you want to go to the authorities** and explain, he advises against it: they won't care, and they'll make you a slave anyway. You can still go, which puts you on the caught path.

## Path A: Caught (the main path)
You try to sneak into the main compound for supplies and get caught, or you went to the authorities. Either way, **you're made a slave.**
- **The Free Lord is on the island when you're caught.** You're brought before the Qotiro overseers, but it's the Roduro lord who decides what happens to you. The Binders defer to him.
- **Teaches, in the compound:**
  - work and tools (labour shifts: quarry, woodlot, fields, docks);
  - inventory and carrying;
  - crafting at a workyard bench (repairs, bandages);
  - fighting (a fight in the yards, or defending the work gang from beasts);
  - wounds, bleeding, bandages, knockouts;
  - the law and bondage (how bonds work, what you owe).
- **The stranger** is caught too, or slips in and out to help you at night *(Claude: he stays free and visits; it keeps him useful)*.
- **The end:** you work your way out. You **show promise**, and you're bought out of your bond by either:
  - **the Lantern Company** (if you fought well), or
  - **the Writers** (if you worked with your head: crafting, reading, alchemy).
  - They give you your freedom, but strongly encourage you to work for them for a while to pay back what they paid. That's the player's first faction relationship, and a natural run of first quests.
  - *(Claude)* Which one buys you is chosen by what you did most on the island.

## Path B: Sneak (harder)
You sneak around the main compound for supplies and **don't get caught.**
- **Teaches, by night:**
  - sneaking, light and shadow, guard routes;
  - looting and inventory (stores, chests, the dead);
  - lockpicking;
  - a disguise (a porter's or a debtor's smock to pass through the compound);
  - fighting when sneaking fails (quiet takedowns, a guard fight);
  - crafting from what you steal.
- **The end:** you and the stranger **sneak onto a ship** and get off the island.
  - The stranger goes to work for **the Night Market** (the new thieves' guild) and **offers you a place there.**
- **If you're caught at any point** on this path, you drop onto the caught path. *(Claude)*

## Shared places, different dressing
Both paths use the same locations. What changes is the time of day, who's there and how it looks.

| Place | Caught path | Sneak path |
|---|---|---|
| **Shore and wreck** | Opening (shared) | Opening (shared) |
| **The shelter hut** | Opening, then where the stranger meets you at night | Your hideout and base |
| **Main compound** | Daytime: work gangs, overseers, shifts | Night-time: guards on routes, sleeping workers |
| **Workyard** | Assigned crafting work | Tools and materials to steal |
| **Stores and warehouse** | Hauling loads | Looting and lockpicking |
| **Quarry, woodlot, fields** | Labour shifts | Hiding places, a way round the walls |
| **Yards and pens** | A fight to prove yourself | A guard fight if you're spotted |
| **The dock** | Your buyer's ship arrives | Sneaking aboard |

## The stranger: Diʻub
- **A Roduro man,** an escaped debtor. His name plays on Jiub from Morrowind (Laz).
- **Diʻub is a clipped nickname,** and the closed ending breaks Roduro's open-syllable sound rules on purpose *(Claude)*: a runaway who has dropped his house name and most of his old self. Add it to the naming module's irregulars.
- Wry and wary, and he knows the island and the compound.
- **He's the tutorial's guide on both paths:** he explains things in his own voice instead of pop-up text.
- On the sneak path he ends up in the Night Market, so the player has a friend there.

## The Free Lord *(new character; Claude's notes)*
- **Ḍeshu of the Grey House**, a Roduro Free Lord who owns the island and the compound. The Binders work it for him.
- Polished, bored, and entirely untroubled. To him the island is a ledger.
- He's the first face of the Great House the player meets, and the first lesson that the people who hold the whip aren't the ones who own it.
- He fits the Free Lords' habit of buying from the Binders (`factions.md`). A natural later target for revenge.

## Names *(Claude, 2026-10-12; built with the naming module)*
- **The island: Salt Isle** (Roduro **Shiqagahì**, shee-kah-gah-HIH, "salt island"). The compound works salt pans, a quarry and fields.
  - **What the debtors call it:** "the Binding" (Roduro **Ḍitagahì**, "bind island"). Diʻub uses this name.
- **The Free Lord: Ḍeshu of the Grey House** (DEH-shoo, "sweet"; the house is **Shohihale**, "grey house"). A sweet name for a man who sells people by the cartload.
- **Diʻub** (DEE-oob): the clipped runaway's name, kept as Laz chose it.

## Diʻub's questions *(Claude, 2026-10-12)*
Asked over the meal at the shelter. **The player never sees a background named,** only the questions and their own answers. Each answer quietly adds a **small boost** (about +5) to the skills shown. Skill names are the game's real skills (`src/sim/stats.rs`). Diʻub's reply follows each answer.

**1. "What were you doing on that ship?"**

| Answer | Boost | Diʻub says |
|---|---|---|
| "Working the ropes." | Athletics, Weaving | "Hands like that, I believed you before you said it." |
| "Guarding the cargo. Didn't do much good." | Blade, Block | "Cargo's at the bottom of the sea. You're not. Count that a win." |
| "Keeping the books for the captain." | Structured, Inscription | "A reader. Don't tell anyone here; they'll make you count salt." |
| "I wasn't supposed to be aboard." | Sneak, Security | "Ha. Then we're the same kind of guest." |

**2. "What did you do with your hands, before all this?"**

| Answer | Boost | Diʻub says |
|---|---|---|
| "Built things. Walls, roofs." | Carpentry, Masonry | "The Binding will love you. It always needs walls." |
| "Worked a forge." | Smithing, Armoring | "Don't let them see your arms. They'll chain you to the anvil." |
| "Mixed remedies. Herbs, mostly." | Alchemy | "Then you know what not to eat out here. Good." |
| "Hunted." | Marksman, Spear | "There's goats on the north side. And worse." |

**3. "Someone comes at you in the dark. What do you do?"**

| Answer | Boost | Diʻub says |
|---|---|---|
| "Hit first." | Unarmed, Blunt | "Honest. Stupid, sometimes. But honest." |
| "Get out of the way." | Dodge, Athletics | "That's how I've lasted this long." |
| "Keep still. Let them pass." | Sneak | "Good. Out here, still is alive." |
| "I just... feel it coming. And something answers." | Felt | He looks at you a long while. "Keep that to yourself." |

**4. "Where were you going, before the storm?"**

| Answer | Boost | Diʻub says |
|---|---|---|
| "Home. Whatever's left of it." | Handcraft, Tending | "Then we'll get you there. Or near enough." |
| "Anywhere with work." | Handcraft, Athletics | "Plenty of work here. None of it paid." |
| "To find a teacher." | Structured, Ritual | "The Writers take strays. If you live." |
| "Away. Just away." | Sneak, Dodge | "Away I understand." |

**Notes:**
- Four answers, so four small boosts: enough to tilt a character without labelling them.
- **On the caught path, the answers also nudge who buys you out:** mostly fighting answers lean to the Lantern Company, mostly thinking answers to the Writers. What you actually do on the island still counts most.
- An answer of **"I'd rather not say"** is always allowed: no boost, and Diʻub shrugs ("Fair. Nor would I.").

## Lessons as data
Each lesson is **self-contained**: a trigger, a place, a task, and what it teaches. A lesson for a system that isn't built yet stays switched off until it is, and the tutorial runs with what works.

**Lesson list** (shared ones first, then by path):

| Lesson | Path | Teaches |
|---|---|---|
| Wake in the storm | Both | Moving, camera, weather |
| Find shelter, cook | Both | Shelter, fire, cooking, eating |
| The stranger's questions | Both | Talking, choosing a background |
| Labour shift | Caught | Tools, gathering, carrying |
| Workyard job | Caught | Crafting, repair, bandages |
| Prove yourself in the yard | Caught | Fighting, wounds, knockouts |
| Learn your bond | Caught | Bonds, the law, what you owe |
| Bought out | Caught | First faction (Lantern Company or Writers) |
| Night run to the compound | Sneak | Sneaking, light, guard routes |
| Raid the stores | Sneak | Looting, inventory, lockpicking |
| Pass the gate | Sneak | Disguise |
| Spotted | Sneak | Fighting, fleeing |
| Stow away | Sneak | Escaping; first faction (Night Market) |

## Layout
- **South-east: the shore** where you're wrecked, and the shelter hut.
- **Centre: the main compound** (bunkhouses, overseers' hall, workyard, stores, the yards).
- **Around it:** quarry, woodlot, fields, pens.
- **South-west: the dock.**
- **North: the wild edge,** with a fenced patch of briar the locals avoid *(Claude: a first look at the Overgrowth)*.
- **West, behind a wall: the test quarter** (hidden).
- **Spare ground** in each zone for new lessons.

## The test quarter (dev-only)
Walled off and locked; a debug key opens it and teleports in. Nothing here is part of the story.

| Piece | Use |
|---|---|
| **Empty build plots** | Flat ground to drop in new buildings, stations and props |
| **Dev spawner** | Spawns any item, creature, person or faction member; sets time, weather and season |
| **Combat yard** | Dummies, an arena, and the duel grid (weapons against armour sets) |
| **Culture row** | One Roduro, Qotiro, Horaro and Ṭaḍoro building, with residents, to test kits, routines and voices |
| **Spare plots** | Kept empty for whatever testing turns up |

## Notes for the build
- **The test quarter can start first,** since it's useful for testing straight away.
- **Skip for later playthroughs** *(Claude)*: answer Diʻub's questions, pick an ending (bought out or stowed away), and start on the main island with basic kit.
- **Coming back:** the island stays on the map. On the sneak path you're an escaped slave there, so returning is risky. *(Claude)*

## Open
- Where Salt Isle sits offshore.
