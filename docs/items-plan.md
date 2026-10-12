# Items: how the work is split (plan, 11 Oct 2026)

The design is `docs/items-design.md` (Laz's combined items doc; it wins where
anything disagrees). This file is how the two Claude Code sessions on Laz's
PC divide it. **Nothing here is started until Laz says go.** Each step ends
with a short plain report to Laz, as the design asks.

## The two tracks

The split follows the code, so the two sessions rarely touch the same file:

- **Primary** (owns `main`, merges, bumps `save::FORMAT`): the item data and
  how things look — slots, forms, materials, the figure, icons, the shared
  tables.
- **Second**: the rules engines — combat, then containers, then food and
  thirst — each with its visual cues.

| Step | Primary | Second |
|---|---|---|
| 0 (now) | Plan (this file) | Finish walking stage 2 (`fix/solid-world`) |
| 1 | **Ground work**: split `items.rs` into per-kind files so the two tracks don't collide; cleanup (0.1: removals, renames, ingredient mapping); the new slots (body/legs under and over, cloak, belt, brooch, second ring); save format | — (waits for step 1, a few hours) |
| 2 | **Looks**: every form gets a shape in a data table (blade, guard, haft, head; cap, helm, coif; cloak panel; pack…); worn on the figure's parts by layer, held in hand, lying on the ground; **icons** drawn from the same shapes (an off-screen camera, like the outlines), shown in the pack screen | **Combat rules** (0.2 items 1–9): bleeding + bandages, stagger and knockdown, blunt knocks out, keep at bay, tight spaces and sweeps, off-hand parry, sneak multipliers, thrown weapons, concealment — each with its cue |
| 3 | **Weapons and armour as data** (Part 1 B1–B6, Part 2 forms): the 28 weapon and 38 armour forms, materials, grades, weights, prices, costs of wearing, layers; their looks | **The duel grid** (0.2 item 10): the headless test of every weapon against every armour set; tunes the numbers with the primary |
| 4 | **Alchemy** (Part 4, 0.5): ingredients with 4 effects, tasting, experiment, the bench and its tools, toxicity, flasks and blade poisons, recipe sheets | **Containers, fire and water** (0.4): capacity, bulk, soaking and drying on the clock, nesting, drop pack, quick slots, showy jewellery |
| 5 | **Clothing looks and disguise** (Part 3), tools (Part 6), trinkets, mundane goods, valuables (Part 7); NPC kit and loot tables for everything | **Food and thirst** (0.6): spoilage, cooking and the heat-source rules, meal buffs, thirst, alcohol, smokes and the smoke hall |

Why this order: the slots and the split come first because everything else
adds to them. Looks come early because Laz judges by playing, and because
combat cues (a bleed mark, a thrown knife lying on the ground) need item
shapes to draw. Combat rules can start as soon as the slots exist; they read
weapon data through fields that step 3 then fills with real numbers.

## How the two tracks stay out of each other's way

- After step 1, item data lives in separate files by kind (weapons, armour,
  clothing, ingredients, food, tools, trinkets). Each track adds lines to its
  own files.
- `combat.rs`, `ai.rs` and the fight cues are the second session's during
  steps 2–3; the primary only adds data there.
- `items.rs` (the catalogue core), `inventory.rs` (slots), `save.rs` and
  `Cargo.toml` are the primary's; the second asks in `Repos\COORDINATION.md`
  for a field it needs.
- Every merge goes through the primary, one branch at a time, full tests on
  this PC.

## Things that need Laz before or during

- **The terrain has a plateau** (`Terrain::plateau`, the Qotiro south-east)
  and animals live on it (Dustrunner, plateau habitat). The design says no
  desert, steppe or plateau. Reshaping the land is bigger than an item job:
  it moves towns and roads. Proposed: do the words now (CLAUDE.md, animal
  homes), and reshape the land as its own task.
- **Item looks**: code-built placeholder shapes now (proposed), swapped for
  Blender models later at the same attach points — or wait for the asset
  agent's models?
- **The grid inventory** (Kenshi-style) was put off; icons in step 2 work for
  either.
