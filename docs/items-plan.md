# Items: how the work is split (plan, 11 Oct 2026)

The design is `docs/items-design.md` (Laz's combined items doc; it wins where
anything disagrees). This file is how the two Claude Code sessions on Laz's
PC divide it. Each step ends with a short plain report to Laz, as the design
asks.

## Laz's answers (11 Oct)

- **Go** for step 1.
- **Item looks**: simple shapes built in code, at fixed attach points on the
  figure (final models can replace them at the same points later).
- **The grid inventory** (Kenshi-style) is part of this chunk.
- **The plateau**: leave it. Laz is making a map himself.
- **Work on a branch**, so Laz can play `main` while this chunk is under way.

## Branches: `main` stays playable

- **`next/items`** holds this chunk. Every step of either session is merged
  there (by the primary, `git merge --no-ff`, full tests at 8 threads), not
  into `main`.
- `main` gets only fixes to what Laz is playing now (a bug he hits, the
  walking slowdown). When `main` moves, the primary merges it into
  `next/items`.
- At the end of the chunk, once Laz has played it, `next/items` is merged into
  `main` in one merge.
- Laz plays `main` from `Repos\Gahturiyu-Sim` as always. The chunk so far can
  be played from **`Repos\Gahturiyu-Sim-next`** (`next/items`, its own build
  folder, its own saves and settings), rebuilt by the primary after each merge
  while that game isn't running. Saves made there may not load in `main`.
- Work branches start from `next/items`: the primary's `items/<step>` in
  `Repos\Gahturiyu-Sim-wt\items`, the second's `items2/<step>` in
  `Repos\Gahturiyu-Sim-wt2\...`. Sync one by merging `origin/next/items` in.

## The two tracks

The split follows the code, so the two sessions rarely touch the same file:

- **Primary** (merges, bumps `save::FORMAT`): the item data and how things
  look: slots, forms, sizes, materials, the figure, icons, the grid
  inventory, the shared tables.
- **Second**: the rules engines: combat, then containers, then food and
  thirst, each with its visual cues.

| Step | Primary | Second |
|---|---|---|
| 0 (now) | Branches and this plan | The walking slowdown (`headless 3 1` 29 s vs 16.5 s): track when solid things change. A fix to `main` |
| 1 | **Ground work**: split `items.rs` into per-kind files so the two tracks don't collide; cleanup (0.1: removals, renames, ingredient mapping); the new slots (body and legs under and over, cloak, belt, brooch, second ring); each item's footprint in grid cells; save format | **Combat rules** (0.2 items 1–9), started on `next/items` at once: bleeding and bandages, stagger and knockdown, blunt knocks out, keep at bay, tight spaces and sweeps, off-hand parry, sneak multipliers, thrown weapons, concealment, each with its cue. What needs a new item field is a placeholder in `combat.rs` until step 1 lands, then read from the item data |
| 2 | **Looks**: every form gets a simple shape in a data table (blade, guard, haft, head; cap, helm, coif; cloak panel; pack…), worn on the figure's parts by layer, held in hand, lying on the ground; **icons** drawn from the same shapes (an off-screen camera, like the outlines) | Combat rules, continued |
| 3 | **The grid inventory**: every pack and container is a grid; things take their footprint's cells and can be turned; the pack screen becomes grids with icons, dragged between members and containers | **Containers, fire and water** (0.4) on the grid: capacity, bulk, soaking and drying on the clock, nesting one deep, drop pack, quick slots, showy jewellery |
| 4 | **Weapons and armour as data** (Part 1 B1–B6, Part 2 forms): the 28 weapon and 38 armour forms, materials, grades, weights, prices, costs of wearing, layers; their looks | **The duel grid** (0.2 item 10): the headless test of every weapon against every armour set; tunes the numbers with the primary |
| 5 | **Alchemy** (Part 4, 0.5): ingredients with 4 effects, tasting, experiment, the bench and its tools, toxicity, flasks and blade poisons, recipe sheets | **Food and thirst** (0.6): spoilage, cooking and the heat-source rules, meal buffs, thirst, alcohol, smokes and the smoke hall |
| 6 | **Clothing looks and disguise** (Part 3), tools (Part 6), trinkets, everyday goods, valuables (Part 7); NPC kit and loot tables for everything | — |

Why this order: the slots and the split come first because everything else
adds to them. Looks come next because Laz judges by playing, because combat
cues (a bleed mark, a thrown knife lying on the ground) need item shapes, and
because the grid's icons are drawn from them. Containers follow the grid
because a container is a grid.

## How the two tracks stay out of each other's way

- After step 1, item data lives in separate files by kind (weapons, armour,
  clothing, ingredients, food, tools, trinkets). Each track adds lines to its
  own files.
- `combat.rs`, `ai.rs` and the fight cues are the second session's; the
  primary only adds data there.
- `items.rs` (the catalogue core), `inventory.rs` (slots, the grid),
  `view/packui.rs`, `save.rs` and `Cargo.toml` are the primary's; the second
  asks in `Repos\COORDINATION.md` for a field it needs.
- Every merge goes through the primary, one branch at a time, full tests on
  this PC at half the CPU (`CARGO_BUILD_JOBS=8 RUST_TEST_THREADS=8`; the PC
  has been losing power under long all-core loads).
