# Items: the combined design (Gahturiyu sim, 2026-10-11)

This is one doc for every item category Laz has signed off on, for the coding agents. **It's for after the playable-MVP freeze lifts.** The separate docs it was built from (weapons, armour, potions, ingredients, food, clothing, tools, trinkets) stay in the project as history. Where they disagree, this doc wins.

# Part 0: Engine changes needed (read first)
These are the new rules and systems the items depend on. Everything else in this doc is data (new lines in tables).

**The sim rules from `CLAUDE.md` all still apply:**
- clock-derived and keyed RNG;
- the same outcome in every band;
- condition as values at a moment plus a rate;
- everything in `World` saved, with a bump to `save::FORMAT` and the fingerprint test;
- every new action gets a visual cue.

## 0.1 Canon and data cleanup (do first)
- **Biomes:** rolling plains, forests, coast, bog, rocky mountains (including ravines and chasms), snowy mountains, the Overgrowth. **No desert, steppe or plateau.** (The biome message may already have been sent; see the appendix.)
- **Removals:**
  - harpoon (replace it in kit, loot, recipes and names);
  - `sandstone_lamellar` (becomes the plate cuirass);
  - kite shield (folded into the round and tower shields);
  - `ash_moss`.
- **Renames (keys stay):**
  - `salt_crystal` → Saltcrust;
  - `grain` → Hardgrain;
  - `wild_berries` → Moorberry;
  - `animal_oil` → Tallow.
- **Placeholder ingredients from the old item list** (kelp frond, squid ink, ghostcap, emberroot, storm glass, silk gland, briar heart…): keep only those in the approved list in Part 4. Map or retire the rest, and list what you did.
- **New equipment slots:**
  - body under and body over; legs under and legs over;
  - **cloak** (packs keep Back);
  - **belt**, **brooch**, and a **second ring**.
- Native names for every new item, through the naming module (`assets/lang/things.ron`).

## 0.2 Combat rules (in `combat.rs`, as data)
1. **Bleeding:** cut hits open bleeds that stop only with a **bandage** (new item) or a healing effect. Outside fights it's on the condition timeline.
2. **Stagger and knockdown** from heavy blunt blows.
3. **Blunt knocks out:** blunt damage leans towards knockout rather than death.
4. **Keep at bay:** reach weapons strike first, and can push back anyone closing in.
5. **Tight spaces and sweeps:**
   - weapons have a space need;
   - indoors, alleys and crowds hurt long and sweeping weapons;
   - sweeps hit 2–3 foes.
6. **Off-hand parrying blades:** dirk, knife or short sword.
7. **Sneak multipliers per weapon:** the knife is highest; the sap knocks out.
8. **Thrown weapons:**
   - they fly as projectiles and land where they stop;
   - they're picked up after the fight;
   - **each hit on anything has a chance to break them.**
9. **Concealment (never a guarantee):**
   - every weapon and armour piece has a concealment value, and clothing adds bonuses;
   - noticing is a keyed roll: concealment + clothing + Sneak, against the looker's perception and effort;
   - a full search is harder to beat than a glance.
10. **Weapon physics:**
    - swing time scales with actual weight against the usual weight;
    - there's a **strength minimum** (below it, slower and less accurate);
    - grade adds **balance** (speed and to-hit);
    - fatigue scales with weight;
    - realistic weights;
    - a global damage scale: an unarmoured average person goes down in **4–6 hits** from a Common Forgeiron longsword.
11. **The duel grid:** a headless test of every weapon against every armour set, indoors and outdoors, sneak and open. Every weapon and armour piece must be the best choice somewhere.

## 0.3 Armour, clothing and weather on the body
- **Layers:** the outer layer meets the blow first, then the inner one.
- **Bleed-stopping:** a cut that's ≥60% stopped opens no bleed. Edgeglass and Sandglass slip past light armour.
- **Padding** reduces stagger, knockdown and knockout. **Helms** have a head knockout guard.
- **Costs of wearing,** scaled down by strength:
  - dodge;
  - fight speed;
  - **travel speed, set when a leg is written;**
  - stamina;
  - noise;
  - sight and hearing (full helms).
- **Weather values on worn pieces:** warmth, dryness, wind. Wetness makes cloth heavy. These feed `weather_effects` exposure.
- **Clothing look tags** (people, role):
  - local style lowers suspicion;
  - role garments let you pass as that role until a keyed see-through roll fails;
  - impersonation is a crime where customs say so.

## 0.4 Containers, fire and water
- **Every container has:**
  - capacity;
  - bulk (sneak and dodge, growing with fullness);
  - a move factor;
  - waterproofing and fireproofing (none, resists, or sealed).
- **Nesting one level deep:** inner containers protect their own contents.
- **Water works in two stages, both timed:**
  1. it **soaks through** the container;
  2. then the contents **degrade** over time.
  - **A roof** stops soaking; **a fire** dries faster.
  - Stored as a value plus a rate, solved for, never ticked.
- **Fire** is a keyed roll per fire event.
- **Orders and carrying:**
  - a **drop pack** order;
  - a **duffel carried in both hands**;
  - **belt quick slots**, used faster in a fight than the pack.
- **Showy jewellery** raises how NPCs read your wealth: respect and prices go up, but so does your attraction as a target for thieves.

## 0.5 Alchemy
- **Ingredients carry 4 effects each.** A potion carries the effects shared by ≥2 ingredients, from **at least two different kinds**. Oils count as their own kind.
- **Discovery:** tasting, experiment, or being taught.
- **Remembered mixes,** plus **recipe sheets** (paper items; read to learn; written with Inscription).
- **Brewing only at an alchemy bench.** Remove brewing anywhere with a mortar in the pack.
- **Bench tools:**
  - mortar and pestle (small boost to everything);
  - **retort** (shortens harmful effects);
  - **calcinator** (lengthens helpful effects);
  - **alembic** (raises magnitude).
  - Harmful effects are shortened, never removed.
- **The toxicity meter,** on the condition timeline. It never kills.
- **Thrown flasks and blade poisons.** Poison only works on hits that wound.
- **Oils** are extracted from a base ingredient: three of the base make one oil, which keeps two of its effects, stronger.

## 0.6 Food, thirst and consumables
- **Spoilage:** fresh, stale, spoiled. The rate changes with preservation, container, wetness, cold and storehouses.
- **Thirst (light):** water sources, rain, waterskins. It never kills.
- **Cooking:**
  - a **Cooking** skill and meal grades;
  - **heat source rules:** a campfire gives basic food only (and needs a **cooking pot** for porridge, tea and broth); a **stove** makes full meals;
  - **one meal buff at a time.**
- **Teas and broths;** **smokes and chews with habits and cravings** (NPCs too).
- **Smokes:** a **haze** value beside drunkenness (mellow, hazy, stupor; no hangover; habit chance). **Smoke halls** (a Qotiro building): a dim chamber that raises haze for anyone inside at a rate, an antechamber that doesn't, and a keeper who does rounds on the clock. **A drunk and high meter** shows only while someone is drinking or smoking; it builds up fast and drains slowly. Pipes are personal; water pipes are shared furniture. Qotiro customs: no alcohol outside rites.
- **Alcohol:** 10 drinks (9 sold; the Qotiro sacrament spirit is rites-only), bought only (no brewing yet). Each has a strength and one small effect (one drink effect at a time). Wine and spirits age in storage to a higher grade (value plus rate, solved for); ale and cider spoil. Spirits add a little toxicity.

## 0.7 Tools
- **The mining pick is required** for ore and stone. The other work tools add speed or yield.
- **Tinderbox:** lights fires quickly.
- **Lantern:** can be shuttered for sneaking.
- **Repair kit:** partial field repair.

## 0.8 Suggested build order
1. Cleanup (0.1), the new slots, and the save format.
2. Combat rules (0.2, items 1–9) with their visual cues.
3. Weapon and armour numbers, new forms, layers, costs of wearing, the duel grid (0.2 item 10, 0.3).
4. Containers, fire and water, packs, quick slots, accessories (0.4).
5. Alchemy (0.5).
6. Food, thirst, cooking, consumables (0.6).
7. Clothing looks and disguise, tools, trinkets, mundane goods; NPC kit and loot tables for everything.

Stop after each step and report to Laz in plain language.


# Part 1: Weapons
## Goal
Every weapon in the game should have a **reason to carry it**. No weapon is simply a weaker version of another: each wins in some situation.

The list below is Laz's approved set of **28 forms**. Materials and grades still multiply them, as they do today (`items::FORMS`). Numbers are yours to tune. Keep the niches.

## New rules the list depends on (small, data-driven, in `combat.rs`)
1. **Bleeding:**
   - Cut hits can open a bleed that keeps doing damage over time. Each weapon has its own bleed chance and strength.
   - **A bleed only stops with a bandage or magic.** It doesn't stop by itself.
   - Add a **bandage** item (cheap, stackable, made by any leatherworker or tailor from cloth). Using one takes a few seconds and can't be done while being hit.
   - Healing spells and potions that restore also stop bleeding (an `Effect` rule).
   - Squad members bind themselves or each other. Fight AI uses bandages like it uses potions.
   - Outside fights, bleeding follows the condition timeline (rule 8): a bleed's end is "when bandaged", so solve for knockdown or death rather than ticking.
2. **Stagger and knockdown:**
   - Heavy blunt blows can interrupt a swing (stagger) or floor the target for a moment (knockdown: no dodge or block, getting up takes time).
   - The chance comes from the blunt damage dealt against the target's strength.
3. **Blunt knocks out:**
   - Blunt damage leans towards knockout rather than death. Damage to head and torso from blunt hits counts more towards unconscious and less towards "minus its maximum". So clubs are how you take people alive (bounties, captives, bondage, robbery).
   - Edged weapons keep today's rules.
4. **Keep at bay:**
   - Reach weapons strike first against someone closing in.
   - A hit on an approaching foe can push them back out of their own reach.
   - A foe who gets inside the reach weapon's range takes it off its best use: lower damage and slower recovery until it steps back.
5. **Tight spaces:**
   - Weapons have a **space need** (short, medium, long, sweeping). Indoors (interiors), in alleys and in crowds (allies close by), long and sweeping weapons lose speed and accuracy, and sweeps can hit allies.
   - Short weapons have no penalty.
6. **Concealment (not a guarantee):**
   - Every weapon has a **concealment** value (how easy it is to hide). Weapons marked "hidden" below have a high one.
   - **Nothing is ever undetectable** (Laz). Whether a carried item is noticed (a glance, a sizing-up, a search) is a keyed roll: the item's concealment plus bonuses from clothing (armour doc) and the carrier's Sneak, against the looker's perception and how hard they're looking. A full search is much harder to beat than a glance.
   - A well-hidden weapon doesn't show on the figure. Until it's noticed, it isn't counted when an NPC sizes you up.
   - **There is no "weapon of war" distinction** (Laz): carrying big weapons in town is fine.
7. **Sneak attacks:** each weapon has its own sneak-attack multiplier. The knife's is the highest by far, and the sap knocks out instead of wounding.
8. **Off-hand parrying:**
   - A dirk, knife or short sword can go in the off hand.
   - It gives a parry bonus (less than a shield's block) plus an occasional quick off-hand strike. It can't be combined with a shield or a two-handed weapon.
9. **Thrown weapons:**
   - Thrown items fly as projectiles (the existing arrow flight). They **land where they stop**, on the target's body or the ground, as items lying in the world.
   - Picking them up after the fight is part of looting. The squad AI should offer "gather thrown weapons".
   - **Each time one hits something (target, wall, ground, shield) it has a chance to break.** The chance depends on what it hit and its material; a broken one is gone.
   - Thrown weapons use the Marksman skill.
10. **Sweeps:** sweeping weapons can hit up to 2–3 foes in an arc in front of the wielder (allies too, in tight spaces).

Each new rule needs its **visual cue** (CLAUDE.md): a bleed mark, a stagger jolt, a fall, a push-back, a parry flash on the off-hand blade, thrown weapons flying and lying on the ground.

## The 28 weapons

**Blades** (Blade skill)

| Form | Niche | Flags |
|---|---|---|
| Knife (existing) | Best sneak attack and finishing blow; an off-hand option; a tool (skinning, cutting nets) | hidden, short |
| Dirk (new) | The off-hand parrying blade | hidden, short |
| Short sword (existing) | Fast, no penalty in tight spaces; the indoor sidearm | short |
| Cutlass (new) | Biggest one-handed bleed, fast, poor against armour | medium |
| Longsword (existing) | All-rounder with a good parry | medium |
| Greatsword (new) | Two-handed sweeping cuts that bleed | sweeping |
| Hatchet (existing) | Cheap mix of cut and blunt; throwable; woodcutting tool | short, throwable |
| Battle axe (new) | Two-handed; huge cut and bleed, slow | sweeping |

**Blunt** (Blunt skill, the knockout family)

| Form | Niche | Flags |
|---|---|---|
| Sap (new) | Knocks out from a sneak attack | hidden, short |
| Club (existing) | Cheap, knocks out | medium |
| Mace (new) | Staggers; one-handed, pairs with a shield | medium |
| War-pick (existing) | Gets through armour | medium |
| Stone maul (existing) | Knockdown; slow and heavy | sweeping |
| Staff (existing) | Reach plus a good parry; looks like a walking stick | long |
| Knuckle guards (new) | Boosts unarmed blows; knocks out | hidden, short |

**Spears** (Spear skill)

| Form | Niche | Flags |
|---|---|---|
| Spear (existing) | Reach, keeps foes at bay, cheap | long |
| Short spear (new) | One-handed with a shield; throwable | medium, throwable |
| Trident (existing) | Keeps foes at bay; its prongs catch blades (parry bonus) | long |
| Glaive (existing) | Reach plus a sweep | long, sweeping |

**Ranged and thrown** (Marksman skill)

| Form | Niche | Flags |
|---|---|---|
| Short bow (existing) | Fast shots, light | — |
| Longbow (new) | Longest range, slow draw, needs strength | — |
| Crossbow (existing) | Hits hard and gets through armour, slow reload, easy to learn | — |
| Hand crossbow (new) | One strong opening shot | hidden |
| Sling (existing) | Cheap ammo found anywhere; blunt, so it can knock out at range | hidden |
| Throwing knives (new) | Several quick hits before melee | hidden, throwable |
| Javelins (new) | A strong opening volley that gets through armour | throwable |
| Weighted net (existing net, made throwable) | Tangles a target at range | throwable |

**Removed:** the harpoon. Replace it in NPC kit, loot tables, recipes and the names table with the trident or short spear. No pike.

## Data and other systems
- New forms go in `items::ITEMS` / `FORMS` with materials, so materials and grades multiply them as now.
- Add recipes to `crafting::RECIPES` (bandages included).
- Add native names through the naming module (`assets/lang/things.ron`, by recipe). `tests/names.rs` will flag missing ones.
- Add NPC kit and loot weights so every form actually turns up.
- Bump the save format (rule 12).
- **Balance test:** a headless duel grid (every weapon against every other, plus against armour sets, indoors and outdoors, sneak and open). Report where each weapon wins. **Every weapon must win somewhere.** Flag any weapon that's never the best choice in any matchup.

## Weapon numbers (materials, damage, quality, weight, cost, speed)

Laz's decisions (2026-10-10):
- **Weight changes speed.**
- **Kenshi-style strength minimum.**
- **Grade improves damage, durability and balance.**
- **Kenshi-steep prices, set by the economy.**
- **Realistic weights.**
- **4–6 hits** to down an unarmoured person with a Common longsword.
- Approved the material personalities below.

All numbers are **starting points**. The duel grid (Part A, stage 4) tunes them. The niches must survive tuning.

### B1. Speed, weight and strength
- **Swing time** = the form's base windup and recover × `(actual weight / usual weight)^0.5` × grade balance × the agility factor (as today).
  - So a light Edgeglass longsword swings about 15% faster than a Forgeiron one, and a Ringstone maul is slower than a wooden one.
- **Strength wanted** = the form's base × `(actual weight / usual weight)`.
  - **Below it**, for each point short: +1% swing time and −0.5% to hit, capped at +50% and −25%. Fatigue per swing rises too.
  - **At or above it**, strength adds damage as now. The cap of that bonus is up to you.
- **Fatigue per swing** scales with weight (it currently scales with windup only).
- Thrown and ranged weapons: a bow's **draw** wants strength (longbow especially). A crossbow wants little, which is its point.

### B2. Damage scale
- Set one global scale so that a **Common Forgeiron longsword**, swung by an average fighter (strength 35, skill 30) at an **unarmoured** average person, downs them in **4–6 hits** on average.
- Everything else is relative to that. Report the hits-to-down for each weapon in the duel grid.

### B3. Form table (usual material, Common grade, average wielder)
Speed is windup / recover in seconds. "Space" is short, medium, long or sweep (N = max targets). Bleed is the chance a cut hit opens a bleed. Stagger is the chance of interrupting, or KD for knockdown.

| Form | Usual material | kg | Str | Windup/recover | Reach m | Cut | Blunt | Bleed | Stagger | Parry | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Knife | plain iron | 0.25 | 10 | 0.30/0.30 | 0.6 | 6 | 1 | 0.30 | – | 0.05 | hidden; sneak ×4; finishing ×3; off-hand ok |
| Dirk | plain iron | 0.5 | 15 | 0.35/0.35 | 0.75 | 7 | 1 | 0.30 | – | 0.35 off-hand | hidden; off-hand parry blade |
| Short sword | bronze | 1.0 | 25 | 0.40/0.40 | 0.9 | 10 | 2 | 0.40 | 0.05 | 0.20 | short; off-hand ok |
| Cutlass | plain iron | 1.2 | 30 | 0.42/0.42 | 0.95 | 12 | 2 | 0.70 | 0.05 | 0.15 | best one-handed bleed |
| Longsword | Forgeiron | 1.5 | 35 | 0.55/0.55 | 1.1 | 15 | 3 | 0.50 | 0.10 | 0.30 | all-rounder |
| Greatsword | Forgeiron | 3.0 | 55 | 0.85/0.75 | 1.5 | 22 | 5 | 0.60 | 0.25 | 0.20 | two-handed; sweep 3 |
| Hatchet | plain iron | 0.8 | 20 | 0.45/0.45 | 0.7 | 8 | 4 | 0.30 | 0.10 | 0.05 | throwable; woodcutting tool |
| Battle axe | Forgeiron | 2.5 | 50 | 0.85/0.80 | 1.3 | 24 | 6 | 0.80 | 0.30 | 0.05 | two-handed; sweep 2 |
| Sap | leather | 0.4 | 10 | 0.35/0.35 | 0.5 | 0 | 5 | – | 0.15 | 0 | hidden; sneak knockout |
| Club | wood | 1.2 | 25 | 0.50/0.50 | 0.9 | 0 | 10 | – | 0.25 | 0.10 | knockout |
| Mace | bronze | 1.6 | 35 | 0.55/0.55 | 0.9 | 0 | 13 | – | 0.40 | 0.10 | staggers; one-handed |
| War-pick | Forgeiron | 1.8 | 40 | 0.60/0.60 | 1.0 | 8 | 11 | 0.20 | 0.25 | 0.10 | +0.3 extra pierce |
| Stone maul | Ringstone | 5.0 | 65 | 1.00/0.90 | 1.3 | 0 | 24 | – | KD 0.60 | 0.10 | two-handed; sweep 2 |
| Staff | wood | 1.8 | 20 | 0.45/0.45 | 1.8 | 0 | 7 | – | 0.15 | 0.35 | long; reads as a walking stick |
| Knuckle guards | plain iron | 0.3 | 10 | unarmed | 0.6 | 0 | +5 | – | 0.15 | 0 | hidden; adds to unarmed |
| Spear | wood (iron tip) | 2.2 | 30 | 0.55/0.50 | 2.2 | 10 | 3 | 0.40 | 0.10 | 0.20 | long; strong keep-at-bay |
| Short spear | bronze | 1.4 | 25 | 0.45/0.45 | 1.6 | 9 | 2 | 0.40 | 0.05 | 0.15 | one-handed; throwable |
| Trident | Nacre | 2.5 | 35 | 0.55/0.55 | 2.0 | 11 | 2 | 0.40 | 0.10 | 0.30 | catches blades |
| Glaive | Forgeiron | 3.0 | 50 | 0.80/0.70 | 2.4 | 16 | 4 | 0.50 | 0.15 | 0.25 | long; sweep 2 |

| Ranged / thrown | Usual material | kg | Str | Draw/reload | Range m | Cut | Blunt | Notes |
|---|---|---|---|---|---|---|---|---|
| Short bow | wood | 0.8 | 25 | 0.8/0.5 | 30 | 12 | 0 | fast |
| Longbow | wood | 1.2 | 55 | 1.4/0.5 | 45 | 18 | 0 | longest range |
| Crossbow | Forgeiron | 4.0 | 20 | 0.4/2.6 | 38 | 18 | 4 | high pierce; easy to learn (skill counts less) |
| Hand crossbow | bronze | 1.2 | 10 | 0.3/3.5 | 15 | 10 | 1 | hidden; one opening shot |
| Sling | leather | 0.1 | 15 | 0.6/0.8 | 22 | 0 | 9 | hidden; knockout at range; free stones |
| Throwing knives | plain iron | 0.15 each | 10 | 0.4/0.4 | 10 | 6 | 0 | hidden; stack of up to 6 |
| Javelin | wood (iron head) | 1.0 each | 35 | 0.7/0.6 | 20 | 14 | 2 | pierce; carry 3 |
| Weighted net | Seareed | 2.0 | 20 | 0.8/1.0 | 8 | 0 | 0 | tangles a target |

**Ammo:**
- arrows and bolts with wood, bronze, Forgeiron or **Sandglass** heads (Sandglass: sharpest, extra bleed, no pierce, always breaks);
- sling stones (free) or Hearthclay shot (cheap, always breaks, a little more blunt).

### B4. Material personalities (multipliers against the usual material)
Keep today's columns and add **bleed** and **chips on armour**. Weight comes from density and drives speed and strength (B1).

| Material | Personality | Key numbers to set |
|---|---|---|
| Plain iron | Cheap all-rounder | edge 0.9, mass 1.0, pierce 0.10 |
| Bronze | Cheap, softer, wears fast; any smith mends it | edge 0.9, durability low-ish, pierce 0.15 |
| Forgeiron | Heavy, gets through armour; the war standard | pierce 0.35, density 1.0 |
| Edgeglass | Sharpest and lightest; heaviest bleeding; no pierce; chips when it strikes armour; can't be mended, only regrown | edge 1.4, bleed ×1.6, density 0.7, chips on armour (extra wear) |
| Nacre | Sharp and light, bleeds well, cracks under blunt blows | edge 1.15, bleed ×1.3, brittle 1.3 |
| Ringstone | Heaviest; best blunt and knockdown; slow; nearly unbreakable | mass 1.3, stagger ×1.4, density 1.3, durability 300 |
| Wood | Light, cheap, weak | edge 0.7, mass 0.9, density 0.7 |
| Sandglass | Razor-sharp but shatters: arrowheads, throwing knives | edge 1.3, bleed ×1.5, breaks on almost any hit |
| Gold | Show pieces for wealth and gifts; poor in a fight | worth 6, soft, heavy |
| Leather / cloth / Seareed / Fishskin | Saps, slings, nets, wraps | as now |

- Allow only sensible form and material pairs (no Gold maul, no wooden glaive).
- Main plus backing pairings stay. A backing material averages weight and adds a share of its traits, as today.

### B5. Grade
| Grade | Damage | Durability | Balance (swing time, to hit) | Worth |
|---|---|---|---|---|
| Crude | ×0.80 | ×0.6 | +5% time, −5% to hit | ×0.5 |
| Common | ×1.00 | ×1.0 | – | ×1.0 |
| Fine | ×1.15 | ×1.4 | −4% time, +4% to hit | ×2.0 |
| Masterwork | ×1.35 | ×2.0 | −8% time, +8% to hit | ×4.0 |

Maker's marks still add a premium on top.

### B6. Prices (from the economy)
- **Base price** = materials used (at the local store price) + the maker's hours × the town's crafter wage, then × material worth × grade worth.
- Each town's scarcity moves it, as Part 2 prices already do (`price_factor_at`).
- Give every form its **labour hours** (knife ~4, longsword ~30, greatsword ~50, crossbow ~45, Edgeglass items add their grow time through the Tender order).
- **Targets, in a labourer's daily wage** (check against the real wage in the economy):
  - Common plain iron knife or club: 1–2 days;
  - Common Forgeiron longsword: about 2 weeks;
  - Fine Forgeiron greatsword: about 2 months;
  - Masterwork Edgeglass blade: 6–12 months, a real goal to save up for, steal or loot.
- Report the actual price spread in the duel-grid report.

# Part 2: Armour, accessories and containers
## Goal
Every piece of armour and clothing should have a **reason to wear it**: a trade between protection and what it costs you (speed, stamina, noise, sight, weather, visibility). The list below is Laz's set of **38 forms** (35 wearables + 3 shields), plus accessories, packs and containers (sections below). Materials and grades multiply them as weapons do (`items::FORMS`). Numbers are starting points; the duel grid tunes them, but the niches must survive.

Read alongside `weapons.md` (Parts A and B). Armour uses the same grade table, the same price formula from the economy and realistic weights.

## Laz's decisions (2026-10-10)
- **Two layers** on body and legs: an under layer plus an over layer.
- **Heavy armour costs you:** dodge and speed (in fights and on the road), stamina, noise (sneaking) and weather (cold metal, wet padding, warm furs).
- **Against the new weapon effects:**
  - armour that mostly stops a cut stops the bleed;
  - padding reduces stagger, knockdown and knockout;
  - helms sharply cut knockouts from head blows;
  - pierce gets through armour by its share (as now).
- **Concealment is a bonus, never a guarantee** (Laz): no clothing makes anything undetectable. Hidden under layers and cloaks improve the odds of things going unnoticed.
- **Shields:** buckler, round shield, tower shield.
- **Weather clothing in this pass:** cloaks, oilskins, furs, hoods.
- **Strength eases heavy armour's costs.** There's no hard minimum.

## Rules
1. **Slots:**
   - Head;
   - Body under, Body over;
   - Hands;
   - Legs under, Legs over;
   - Feet;
   - **Cloak**: a new slot, because packs already use Back;
   - Off hand (shields).
   Bump the save format.
2. **Layers:**
   - A blow on a covered part meets the over layer first (its coverage roll), then the under layer (its own roll).
   - Each takes its share off cut and blunt in turn, and each wears.
   - Pierce applies to each layer.
3. **Bleeding:** a cut hit opens no bleed if the armour on that part stopped at least 60% of its cut damage. Edgeglass (bleed ×1.6) and Sandglass ignore that rule against light armour (leather, cloth, wraps).
4. **Padding** (a per-form value) reduces stagger, knockdown and knockout chances from blunt blows to the parts it covers.
   - **Helms** have a separate **head knockout guard**. A sap or club to a helmed head rarely knocks out.
   - A sneak-attack knockout against a helmed target needs a strong blow.
5. **Costs of wearing** (summed over everything worn, scaled down by strength as in B1 of `weapons.md`):
   - **dodge penalty** (as now);
   - **move speed** in fights;
   - **travel speed**: set when a leg is written (rule 3: legs stay analytic), never per step;
   - **stamina**: fatigue per swing and per hour of marching;
   - **noise**: a sneak penalty, with metal and stone loudest;
   - **sight and hearing**: full helms reduce spotting sneakers and seeing at night.
6. **Weather** (from `weather_effects`, read once per decision):
   - **warmth**: furs and cloaks against cold and snow; metal is chilling in cold and gales;
   - **waterproofing**: oilskin and pitch-sealed Seareed, Fishskin;
   - **wetness**: wet cloth and padding get heavier (more weight and stamina cost) until dry.
   - Feed exposure into condition as the weather hooks describe. Where the hooks aren't wired yet, store the values on the items.
7. **Concealment (bonuses only):**
   - Hidden-friendly under layers have a high concealment value of their own (a mail shirt under a coat is hard to spot).
   - Clothing also gives a **concealment bonus** to what's carried beneath it.
   - A **cloak** gives a large bonus to weapons up to medium size, and to the armour beneath.
   - A **travelling robe** gives the best bonus against **searches** (theft, arrest, robbery). It still never guarantees anything.
   - Whether something is noticed is the keyed roll in `weapons.md` (rule 6): the item's concealment + clothing bonuses + the carrier's Sneak, against the looker's perception and effort.
   - Until noticed, items aren't counted when NPCs size up your might, and don't show on the figure.
8. **Durability:**
   - Each piece wears per blow it meets.
   - Zero durability destroys it (Part 2 rule).
   - It's mended by its craft.
   - Brittle materials (Slatewing, Nacre) wear faster under blunt blows; Ringstone hardly wears.
9. **Visual cue:** every worn layer shows on the figure (except hidden ones), plus a sound and flash on a stopped blow.

## The 38 forms
Key: P = protection (low, med, high, top); cut and blunt marked where they differ; Pad = padding; Hid = high concealment (hard to spot, never impossible).

**Head (6)**

| Form | Usual material | Why wear it |
|---|---|---|
| Hood | cloth / fishskin | Rain and cold; shades the face (harder to recognise); almost no armour |
| Padded cap | cloth | Cheap knockout guard; quiet |
| Leather cap | leather | Light cut guard; quiet |
| Mail coif | plain iron | Good vs cut, poor vs blunt; a bit noisy |
| Open helm | bronze / Forgeiron | Strong knockout guard; keeps sight and hearing |
| Full helm | Forgeiron / Slatewing | Best head protection; **cuts spotting and night sight** |

**Body, under (6)**

| Form | Usual material | Why wear it |
|---|---|---|
| Shirt | cloth / tentsilk | Base layer; a little warmth |
| Silk wraps | tentsilk | Hid; lightest cut guard; best for sneaks and casters |
| Padded vest | cloth | Hid; anti-knockout and anti-blunt; gets heavy when wet |
| Mail shirt | plain iron / Forgeiron | Hid; strong vs cut, weak vs blunt; noisy |
| Fishskin undercoat | fishskin | Hid; light; sheds water (no wet weight) |
| Fur jerkin | leather (fur) | Warmth for snow and mountains; light blunt guard |

**Body, over (7)**

| Form | Usual material | Why wear it |
|---|---|---|
| Padded jacket | cloth / seareed | Cheap all-round; strong padding |
| Hide coat | leather / fishskin | Cheap, quiet, decent vs both |
| Brigandine | leather with iron plates | Medium protection while staying fairly quiet |
| Scale coat | Forgeiron / bronze / Nacre / Slatewing scales | Strong vs cut, flexible (smaller dodge penalty) |
| Plate cuirass | Forgeiron / bronze / Slatewing | Top protection on the torso only, arms free (keeps dodge and swing) |
| Stone cuirass | Ringstone | Top vs blunt, nearly unbreakable; very heavy and slow |
| Travelling robe | tentsilk / cloth | Warm and light; the best bonus to keep items unfound in a search |

**Hands (3)**

| Form | Usual material | Why wear it |
|---|---|---|
| Gloves | leather | Grip and warmth (no slipping in rain); small cut guard |
| Bracers | leather / bronze | Forearm guard; small parry bonus |
| Gauntlets | Forgeiron | Strong hand and arm guard; adds blunt to unarmed; **−10% marksman accuracy** |

**Legs (5)**

| Form | Layer | Usual material | Why wear it |
|---|---|---|---|
| Trousers | under | cloth | Base layer |
| Mail leggings | under | plain iron | Hid; cut guard; noisy |
| Hide leggings | over | leather / fishskin | Cheap, quiet |
| Scale greaves | over | Forgeiron / Nacre / Slatewing | Strong vs cut |
| Plate greaves | over | Forgeiron / bronze | Top protection; heavy (slows travel most) |

**Feet (5)**

| Form | Usual material | Why wear it |
|---|---|---|
| Soft shoes | cloth / leather | **Quietest**; best for sneaking; worse travel on rough ground |
| Boots | leather | Standard; good travel on rough ground |
| Fishskin boots | fishskin | No penalty in mud and wet ground; good at low tide |
| Fur-lined boots | leather (fur) | Warmth; snow travel |
| Armoured boots | Forgeiron | Foot and shin guard; noisy and heavy |

**Cloak slot (3)**

| Form | Usual material | Why wear it |
|---|---|---|
| Cloak | cloth / leather | Warmth and light rain; **big concealment bonus for a medium weapon and the armour beneath** |
| Oilskin cape | seareed sealed with pitch | Best against rain and spray; rots if the seal wears off |
| Fur mantle | leather (fur) | Most warmth for mountains and snow; bulky (small dodge penalty) |

**Shields (3), off hand**

| Form | Usual material | Why carry it |
|---|---|---|
| Buckler | wood / bronze / Nacre | Light; parries rather than blocks; no noise or sneak penalty; pairs with any one-handed weapon |
| Round shield | wood / Forgeiron | All-round block; moderate weight |
| Tower shield | wood / Forgeiron / Ringstone | Blocks most arrows and bolts from the front; heavy, slow, big dodge penalty |

Remove `sandstone_lamellar`. Its role becomes the plate cuirass (canon: Qotiro never use sandstone). Fold the old kite shield into the round and tower shields.

## Accessories (decor and enchanting, little armour)
New slots: **Belt**, **Brooch** (cloak pin), and a second **Ring** (two rings). **Neck** stays. Accessories are mainly for **enchantments** (the effect system already supports worn effects) and **show**.
- **What visible decor does:** gold and gems raise how NPCs read your wealth and standing (respect in talks, better first prices with some sellers). They also make you a **target** for thieves and bandits (they read it as wealth). Plain ones don't.
- **Belt** has a small practical side: a belt has **quick slots** (2–4). Potions, bandages, throwing knives and scrolls in quick slots are used faster in a fight than from a pack.

| Form | Slot | Usual material | Why wear it |
|---|---|---|---|
| Plain belt | Belt | leather | 2 quick slots |
| Bandolier belt | Belt | leather | 4 quick slots; visible |
| Sash | Belt | cloth / tentsilk | 1 quick slot; high concealment bonus for a small blade tucked in it |
| Fine belt | Belt | leather with gold or bronze fittings | Show; enchantable; 2 quick slots |
| Ring | Ring ×2 | bronze / gold / Nacre / Edgeglass | Enchanting and show |
| Amulet / torc | Neck | bronze / gold / Ringstone / Nacre | Enchanting and show (existing amulets move here) |
| Brooch | Brooch | bronze / gold / Nacre | Pins the cloak; enchanting and show; a maker's or house mark can show allegiance |

## Packs and containers (fire, water, bulk)
Laz: the player should choose containers for **size, sneaking and protection**. Kenshi does size; we add **fire and water**, which already harm carried things here (`elements.rs`: paper burns or soaks, pitch-sealed gear scorches) and will get worse with rain and wading (weather).

### Every container has
- **capacity** (kg);
- **bulk**: the sneak penalty and dodge penalty while worn or carried, growing with how full it is;
- **move factor**: fight speed and travel speed (travel set when a leg is written);
- **waterproofing** and **fireproofing**: each is *none*, *resists* (cuts the chance and amount of damage) or *sealed* (contents safe until the container itself is damaged or its seal wears off);
- **where it goes**: Back, Belt, carried in the hands, or **inside another container**;
- weight, price and concealment, like everything else.

### What fire and water do to contents
- **Wet:**
  - paper, notes, scrolls and manuals blur or are ruined;
  - bread and dried food spoil faster;
  - powders and reagents (salt, ash, herbs) clump and lose value;
  - unsealed Seareed rots faster.
- **Fire:**
  - paper, cloth, Seareed, tentsilk and food burn;
  - potions can crack;
  - metal and stone are safe.
- **Outer layer first:**
  - Rain, spray, wading, a fall in water, a fire spell or burning ground reaches the outermost container first. Only what gets through reaches the contents.
  - **Inner containers protect their own contents** even when the pack around them is soaked or singed.
  - Nesting goes one level deep: a pouch in a pack, not a pouch in a pouch.
- **Water takes time, in two stages** (Laz):
  1. **Soaking through:** rain must first work through the container. Each container has a soak-through time that depends on rain strength and its waterproofing (none: minutes in a downpour; resists: much longer; sealed: never, unless damaged).
  2. **Degrading:** only then does it start reaching the contents, and the contents degrade over further time (paper blurs, then is ruined; food spoils by stages).
  - **Under a roof** (inside a building, a tent, a cave) rain stops adding wetness.
  - **Near a fire** things (and people) dry out faster; in the open with no rain they dry slowly.
  - Store wetness like condition (rule 8): a value at a moment plus a rate (soaking or drying), with the moments it crosses each stage solved for, not ticked. The rate changes when rain starts or stops (weather lookup), when you go under a roof, or near or away from a fire.
  - **Wading or falling in water** soaks through at once (scaled by waterproofing), but contents still degrade over time.
- **Fire** is quicker: a keyed roll per fire event (a spell, burning ground) by the fire's strength and the container's fireproofing.

### Worn and carried

| Form | Where | Usual material | Capacity | Bulk / sneak | Water | Fire | Why carry it |
|---|---|---|---|---|---|---|---|
| Belt pouch | Belt (uses quick slots) | leather | tiny | none | resists | none | Keeps a few small things on you with no sneak cost |
| Satchel | hip (Back slot free for nothing else) | leather | small | low | resists | none | The sneak's bag: little penalty, quick to reach into |
| Small pack | Back | leather / seareed | medium | moderate | none (seareed: resists if sealed) | none | Everyday carrying |
| Large pack | Back | leather / tentsilk | large | high | none | none | Big trips; drop it before fighting or sneaking |
| Sea bag | Back | fishskin / seareed + pitch | medium | moderate | **sealed** | none (pitch burns: fire damages the seal) | Coast, rain and wading; scrolls stay dry |
| Duffel / sack | **carried in both hands** | cloth / seareed | very large | very high | none | none | Hauling loot or goods; you can't fight or use hands while holding it; drop it instantly |
| Porter's frame | Back | wood + rope | huge | extreme | none | none | Trade and base hauling; slows travel; useless in a fight |
| Quiver | Back or Belt | leather | ammo only | low | resists | none | Ammo drawn at full speed, worn with any pack |

### Inner containers (go inside a pack, or in a quick slot)

| Form | Usual material | Holds | Water | Fire | Why carry it |
|---|---|---|---|---|---|
| Oilskin wrap | seareed + pitch | a few small items | sealed | none | Cheap way to keep notes and scrolls dry |
| Scroll case | bronze / leather / Ringstone | papers only (scrolls, notes, manuals, letters) | sealed | **resists** (bronze), **sealed** (Ringstone) | Protects the most fragile valuables; Ringstone is heavy |
| Clay jar | Hearthclay | powders, reagents, food | sealed when stoppered | resists | Cheap and fireproof-ish, but **breaks** on a hard fall or blunt hit |
| Potion belt case | leather with stoppered slots | potions only | resists | resists | Potions don't crack in fire or falls; can sit in a quick slot |
| Strongbox | Forgeiron / Ringstone | anything, small | sealed | sealed | Lockable (Security skill to pick); heavy; for coin and valuables, at bases and for caravans |

### Other rules
- **Drop the pack** with one order before a fight or a sneak, and pick it up after. It lies on the ground and can be stolen.
- **Bulk grows with fullness:** an empty large pack sneaks better than a full one.
- **Containers are items:** they can be damaged (fire burns a leather pack, a hard hit breaks a clay jar). A damaged seal loses its protection until mended (pitch reseals seareed).
- **The squad's AI:** offer "stow papers in the scroll case" and warn before wading or entering rain with unprotected paper.
- **The base storehouse** keeps its contents dry; a burned storehouse burns its contents (base-building fire rules).

## Armour materials (personalities)

| Material | As armour |
|---|---|
| Cloth | Padding; warmth; soaks and gets heavy when wet |
| Leather | Quiet, cheap, decent all-round |
| Fishskin | Light, sheds water, quiet; modest protection |
| Seareed | Cheap padding and capes; rots unless sealed (as now) |
| Tentsilk | Lightest; good vs cut; hidden-friendly; poor vs blunt |
| Plain iron | Cheap mail; strong vs cut, weak vs blunt; noisy |
| Bronze | Cheaper metal plates; softer, wears faster |
| Forgeiron | Strong vs both; heavy; noisy |
| Slatewing | Light stone plates; great vs cut; cracks under blunt |
| Nacre | Very good vs cut; light; brittle vs blunt |
| Ringstone | Best vs blunt; heaviest; nearly unbreakable; loud |
| Gold | Show armour: status and gifts; soft and heavy |

Allow only sensible pairs (no gold mail, no Ringstone gloves).

## Numbers to set (starting points; the duel grid tunes them)
For each form: coverage (parts and chance), cut and blunt share stopped, padding, head knockout guard (helms), dodge penalty, move and travel factor, stamina factor, noise, sight and hearing, warmth, waterproofing, weight (realistic: mail shirt ~9 kg, plate cuirass ~12 kg, stone cuirass ~20 kg, padded jacket ~3 kg), labour hours, its concealment value, and its concealment bonus to what's beneath.
- **Targets:**
  - full Forgeiron plate stops most cuts from plain iron and bronze, but Forgeiron picks and crossbow bolts still get through;
  - leather and padding together roughly halve hits from a club;
  - a sneaker in silk wraps and soft shoes keeps nearly all of their sneaking;
  - a fully plated fighter travels about 20–25% slower than an unarmoured one (less with high strength).
- **Prices:** the same formula as weapons (B6). A Common hide coat is ~2–3 days' wages; a Common Forgeiron plate cuirass ~1 month; Masterwork Slatewing or Nacre armour is a long-term goal.

## Duel grid additions
- Run every weapon against these armour sets: none, cloth, leather + padding, mail + coat, scale, full Forgeiron plate, Ringstone.
- Report hits-to-down, bleed rate, knockout rate, and sneak and travel penalties per set.
- **Every armour form must be the best choice somewhere** (a fight, a sneak, the weather, a disguise).

# Part 3: Clothing
## Laz's decisions
- **Clothing is for weather, and for disguise and belonging.** Status and work bonuses aren't part of it for now.
- **Clothes are the under layer**, using the armour layers in `armour.md`: shirts, tunics and trousers go under armour; coats and cloaks go over it.
- **Regional styles:** each people has its own cut. Anyone can wear any style.
- **Dyes come later;** plain colours for now.

## What a piece of clothing carries
- **Weather:**
  - **warmth** (cold and snow);
  - **dryness** (how well it sheds rain and spray);
  - **wind** (gales on the coast and in the mountains).
  - These feed the exposure values in `weather_effects`, as the armour doc already describes.
- **Look:** which **people** it reads as (Roduro, Qotiro, Horaro, Ṭaḍoro, or plain), and for some pieces a **role** (guard, priest or priestess, merchant, labourer).
- Little or no protection. That's armour's job.

## Disguise and belonging (rules)
- **Dressing like the locals helps.** In a community, strangers dressed in its people's style draw less suspicion. Guards and townsfolk are slower to notice them as outsiders, and slightly more open in talk.
- **Role garments let you pass as that role** (a guard's tabard, a priest's vestment). People treat you as the role until someone sees through it.
- **Seeing through a disguise** is a keyed roll: the looker's perception and how well they know the real person or role, against the wearer's Sneak and how fitting the rest of their kit is (a priest with a greatsword stands out). Close friends, guards of the same post and officials are the hardest to fool.
- **Impersonation** of a guard or a priest or priestess is a crime where customs say so (law reads customs, never race). Caught in a role garment means suspicion, then the law.
- Clothing never makes anything undetectable (Laz's concealment rule). It only shifts the odds.

## The clothes

**Everyday under layer (body)**

| Piece | Look | Weather | Notes |
|---|---|---|---|
| **Plain shirt and trousers** | Plain | Low warmth, poor in rain | The default; fits anywhere, belongs nowhere |
| **Banded tunic** | Roduro | **Warm**, fair in rain | Thick wool, grey-brown, stitched in rings that echo grown stone |
| **Sash robe** | Qotiro | **Cool**: poor warmth, poor in rain | The desert cut they brought with them. Out of its element here, so Qotiro wear it under a wool overcoat |
| **Sea wrap** | Horaro | Low warmth, **sheds water, dries fast** | Wrapped fishskin and seareed, made for diving and spray |
| **Travelling robe** | Ṭaḍoro | Fair warmth, fair in rain | Light, many pockets. Already in `armour.md` (concealment bonus in a search) |

**Outer layer (cloak slot): weather first**

| Piece | Look | Weather | Notes |
|---|---|---|---|
| **Wool cloak** | Plain | Warm, fair in rain, fair in wind | The everyday cloak (the armour doc's "Cloak") |
| **Oilskin cape** | Horaro and coast | **Best in rain and spray** | Pitch-sealed; rots if the seal wears off |
| **Fur mantle** | Mountain folk | **Warmest**, good in wind | Bulky |
| **Wool overcoat** | Qotiro (in Roduro land) | Warm, good in wind | What Qotiro wear over the sash robe here |

**Head and feet**

| Piece | Look | Weather | Notes |
|---|---|---|---|
| **Hood** | Plain | Fair in rain; shades the face | Harder to recognise (a small disguise bonus) |
| **Wool cap** | Roduro | Warm | |
| **Brimmed rain hat** | Horaro and coast | Good in rain | |
| **Soft shoes, boots, fur-lined boots, fishskin boots** | Plain | As in `armour.md` | Already defined there |

**Role garments (disguise)**

| Piece | Reads as | Notes |
|---|---|---|
| **Guard's tabard** | The town watch | Each town's colours. Impersonating the watch is a serious crime where customs say so |
| **Vestment** | Priest or priestess | Opens shrines and rites. Note that a Qotiro priest is always a woman (canon), so a man in a Qotiro vestment is spotted at once |
| **Merchant's coat** | Merchant | Easier talk at stalls and caravans; draws thieves |
| **Labourer's smock** | Labourer or porter | Nobody looks twice: a good disguise for slipping through work yards and docks |

## Visual variants (cosmetic only)
Each variant uses the same numbers as its base piece. They're there for looks and so crowds don't all dress alike. Townsfolk are dressed from these by their people, trade and wealth.

**Plain (anyone)**
- Shirts: linen shirt, laced shirt, collarless work shirt, sleeveless vest over a shirt.
- Trousers: straight trousers, gathered breeches, patched trousers, rolled-cuff work trousers.
- Cloaks: short shoulder cape, long travelling cloak, patched cloak, hooded cloak.

**Roduro (banded tunic family)**
- Long banded tunic to the knee, belted.
- Short banded tunic with stone-grey leggings.
- Elder's tunic: dense ring stitching from collar to hem (the more rings, the older the wearer looks).
- Tender's tunic: ash-stained, sleeves bound back for working stone.
- Wool caps: ringed cap, ear-flap cap, simple felt cap.

**Qotiro (sash robe family)**
- Long sash robe with a broad wrapped sash.
- Split robe over loose trousers, for riding and drilling.
- Short robe with layered shoulder wraps.
- Sun-disc robe: a gold sun disc stitched on the back (worn at rites).
- Overcoats: high-collared wool overcoat, long belted overcoat, sheepskin-lined overcoat.

**Horaro (sea wrap family)**
- Shoulder wrap with knotted cords.
- Full body wrap, crossed and tied at the hip.
- Diver's short wrap, bare-armed, with a woven belt for tools.
- Net shawl: an open-woven shawl over the wrap.
- Rain hats: wide brim, peaked brim, woven-reed hat.

**Ṭaḍoro (travelling robe family)**
- Long robe with many sewn pockets.
- Short robe with a satchel strap across the chest.
- Scholar's robe: wide sleeves, ink-stained cuffs.
- Road robe: hitched up at the waist over boots.
- Hoods: deep hood, pointed hood, hood with a face scarf.

**Mountain and coast extras**
- Fur mantles: grey fur, brindled fur, fur with a hood.
- Oilskin capes: short cape, long cape with a hood, sleeved oilskin coat.

**Role garments**
- Guard's tabard: town colours in a few patterns (halved, quartered, banded).
- Vestments: each people's own form of the priestly robe.
- Merchant's coat: short coat, long coat, coat with a fur collar.
- Labourer's smock: plain, patched, leather-aproned.

## Overlap with `armour.md`
These pieces are defined in both docs: shirt, trousers, hood, travelling robe, cloak, oilskin cape, fur mantle, and the shoes and boots. **This doc owns their look and weather; the armour doc owns their protection and concealment.** Merge them into one item each when built.

## Later
- **Dyes** from the ingredients (Moorberry blue, Ashberry orange, Nightberry black, Stormbell rose, Vinecap amber, Gold thread gold).
- **Status** (fine clothes and rags) and **work** bonuses, if wanted.

# Part 4: Alchemy
## Laz's decisions (2026-10-10)
- **Morrowind-style mixing:**
  - each ingredient carries hidden effects;
  - mixing ingredients that share an effect makes a potion with that effect;
  - fixed recipes go away as a rule, though shops and NPCs still sell finished potions.
- **Discovery:** by **tasting** (eat a raw ingredient), by **experiment** (mixing reveals shared effects) and by being **taught** (alchemists, herbalists, Ṭaḍoro notes and manuals).
  - Alchemy skill does **not** reveal effects. It sets strength and duration, together with the bench's tools.
- **Roles:** healing and recovery; boosts; thrown flasks and blade poisons.
- **Side effects are shortened by equipment and skill** (Laz, 2026-10-11). Ingredients carry harmful effects too. A well-equipped bench and good skill make them brief, so cheap potions keep them and good ones barely do.
- **A toxicity meter** instead of a fixed limit.
- **Mixes are remembered** as named recipes, repeatable in one click, and can be sold or taught.
- **Regional ingredients, quality grades, cultural specialities.**

## Rules
1. **Ingredients:**
   - Each has **4 effects** (some good, some harmful), stored as data, the same in every world.
   - Weight, base value, a **region and source** (a plant in a biome, a sea pickup, a part from a species), and a **shelf life** (fresh, dried, preserved).
   - Wet or burned ingredients spoil (container rules in `armour.md`).
2. **Tasting:**
   - Eating a raw ingredient reveals **one** of its unknown effects (a keyed roll), and you feel a weak dose of it. It might be the harmful one.
   - Each taste adds a little toxicity.
3. **Experiment:** mixing 2–4 ingredients reveals **every effect they share**, known or not, even if the result is useless or harmful.
4. **Taught:**
   - Alchemists and herbalists sell knowledge of effects ("ghostcap helps you see in the dark").
   - Ṭaḍoro manuals and notes list effects. Recipes can be bought.
   - What the squad knows is per person, and shareable by teaching within the squad.
5. **Brewing (Laz, 2026-10-11): only at an alchemy station.**
   - **No brewing anywhere else.** The mortar and pestle no longer works from the pack.
   - Choose 2–4 ingredients from **at least two different kinds**. The potion carries every effect shared by **at least two** of them.
   - **The alchemy bench** is the station: in alchemists' shops, Ṭaḍoro houses, healing houses, and buildable at a base. A bare bench brews weak, short potions whose harmful effects last long.
   - **Tool upgrades** fitted to a bench (each a crafted item, with its own grade):

     | Tool | What it does |
     |---|---|
     | **Mortar and pestle** | Grinds ingredients: a small boost to everything. The basic kit, fitted to the bench. |
     | **Retort** | **Shortens harmful effects.** At a high grade with good skill they become a brief flicker. |
     | **Calcinator** | **Lengthens helpful effects.** |
     | **Alembic** | **Raises magnitude** (how strong each effect is). |

   - **Alchemy skill** scales all of them.
   - A town alchemist's bench has the tools their trade can afford. A player bench is upgraded one tool at a time.
   - Harmful effects are never removed outright. They're shortened, so cheap potions keep noticeable side effects and fine ones barely any.
   - **Grade** (Crude to Masterwork) comes from the final strength and how short the harmful effects are. It sets the price.
6. **Recipes remembered:**
   - A brewed mix is saved under an auto name from its main effect ("Draught of Night Sight"), and can be renamed.
   - It can be repeated in one click with the needed ingredients, and sold or taught as a note.
7. **Toxicity meter:**
   - Every potion adds toxicity by its strength and impurity. Tasting adds a little too.
   - It drains over hours.
   - Stored like condition (rule 8): a value plus a rate, with the moments it crosses each threshold solved for, never ticked.
   - **The stages:**
     1. Fine.
     2. **Queasy:** stamina regenerates slower.
     3. **Sick:** attribute penalties, chance to vomit, which cancels the newest potion.
     4. **Collapse:** knocked out for a while.
   - **Never kills** (consistent with starvation).
   - A **Cleanse** potion lowers it.
8. **Thrown flasks:**
   - A potion with harmful effects can be bottled in a **throwing flask** (Sandglass or clay: made to shatter).
   - It bursts on impact in a small area, using the existing projectile flight and the thrown-weapon rules (no pickup: it's gone).
   - Burn sets fire (`elements.rs`); Smoke makes a cloud that blocks sight for a while; Slumber or Daze as gas; Poison as a splash.
9. **Blade poisons:**
   - A harmful potion can **coat a weapon** for N hits (more with skill).
   - It only takes effect on hits that wound, so armour that stops the blow stops the poison.
   - Works on arrows and throwing knives too.
   - Using poison is a crime in some towns' customs (a law hook, read from customs, never from race).
10. **Potions break.** Flasks in packs can crack from falls, fire and blunt hits (container rules; the potion case protects them).
11. **Shops and NPCs:**
    - Town alchemists brew from what their store holds (the making rules, rule 17).
    - Cheap potions on sale often carry side effects; Fine and better don't.
    - Fight AI uses potions by their effects (rule 13), and the throwing and coating rules too.

## What each effect does

**Helpful**

| Effect | Does |
|---|---|
| Heal | Restores health (as `Does::Heal`) |
| Stanch | Stops bleeding (alongside bandages) |
| Restore stamina | |
| Restore energy | Mana |
| Rouse | Wakes a knocked-out ally; clears daze |
| Steady | Resists stagger and knockdown |
| Cleanse | Cures poison; lowers toxicity |
| Strength | Temporary strength |
| Swiftness | Move and swing speed |
| Shadow | Sneak bonus |
| Night sight | See in the dark (light rules) |
| Warmth | Resists cold and wet exposure (weather) |
| Resist fire | |
| Toughen | Skin like armour (`Does::Attr`/toughness) |
| Clear mind | Easier Structured casting, energy regeneration |
| Keen senses | Better at noticing hidden things, sneakers and concealed weapons |
| Deep breath | Hold breath long (Horaro speciality; for when the sea opens) |

**Harmful** (poisons and side effects)

| Effect | Does |
|---|---|
| Poison | Damage over time |
| Weakness | Strength down |
| Sluggish | Slower movement and swings |
| Daze | Short stun |
| Slumber | Knockout |
| Blind | Sight greatly reduced |
| Nausea | Stamina drain plus toxicity |
| Thin blood | Wounds bleed more |
| Burn | Fire (flasks) |
| Smoke | A cloud that blocks sight (flasks) |

## Data and other systems
- Ingredient effects live in one data table. Brewing reads effects, never names (as magic does).
- Existing potions (healing draught, greater healing, mana tonic) become **known mixes** that town alchemists start with.
- Native names come from the naming module.
- Stations and tools: the alchemy bench (required to brew) with fitted tools: mortar and pestle, retort, calcinator, alembic.
- Discovery state (which effects each person knows, and saved recipes) is saved (rule 12).
- The UI shows each ingredient's known effects and "?" for unknown ones. The brew panel previews the effects the mix will carry (only the known ones, plus "unknown shared effects").


## Ingredients (all approved)

Goes with `potions.md` (the mixing rules). The earlier 51-ingredient draft was dropped as too much too fast.

### Ground rules (Laz)
- **Three per category for now**, so they're easy to remember. More may come later.
- **A potion needs at least two different kinds of ingredient** (berry + flower, flower + root…). Two berries alone make nothing.
- **Scope:** each ingredient's effects, where it's found, and its food or utility value. **Recipes come last**, once the ingredients are set.
- **Gathering mechanics are out of scope** (no gathering spots, tools or timing). Ingredients come from shops, stores, hunting and loot like other materials.
- **Biomes (Laz, 2026-10-11): the only ones on the map are**
  - rolling plains;
  - forests;
  - coast;
  - bog;
  - rocky mountainous ground, including ravines and chasms;
  - snowy mountains;
  - the Overgrowth.
  Every ingredient is placed in one of these.
- **The map has no desert or steppe.** The Qotiro came from one, but here they live in Roduro land, out of their element. (Any leftover "plateau" in code is listed in the appendix.)

### Effects in use (from `potions.md`)
- **Helpful:** Heal, Stanch, Restore stamina, Restore energy, Rouse, Steady, Cleanse, Strength, Swiftness, Shadow, Night sight, Warmth, Resist fire, Toughen, Clear mind, Keen senses, Deep breath.
- **Harmful:** Poison, Weakness, Sluggish, Daze, Slumber, Blind, Nausea, Thin blood, Burn, Smoke.

### Berries (approved)

| Berry | Where found | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Moorberry** | Rolling plains, on low creeping shrubs among the grass; common | Restore stamina, Heal, Clear mind, *Nausea* | Good: fresh, jam, dried for travel (replaces "wild berries") | Dark-blue dye |
| **Ashberry** | Small trees on rocky slopes and ravine sides, and at forest edges; common in autumn | Warmth, Rouse, Resist fire, *Nausea* | Poor raw (sour); good as jelly or with meat | Hung over doors as a ward (custom only); orange dye |
| **Nightberry** | Forests, in deep shade; uncommon | Night sight, Shadow, *Poison*, *Thin blood* | Not food: eating it raw risks poisoning | Black dye, ink darkener; a poisoner's staple |

### Flowers (approved)

| Flower | Where found | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Stormbell** | Coast: pink bell flowers on sea cliffs and headlands; uncommon (said to open after gales) | Heal, Clear mind, Steady, *Daze* | None | Rose dye; picked for rites and omens |
| **Moorbell** | Bog edges and damp plains: small blue bells; common | Rouse, Warmth, Keen senses, *Daze* | Tea (a common everyday drink) | Scent for linen |
| **Mirebloom** | Bog pools: a floating white flower; uncommon | Shadow, Cleanse, *Slumber*, *Poison* | Not food | A sleeping draught in tiny doses (folk use); funeral flower |


### Leaves (approved)

| Leaf | Where found | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Bloodleaf** | Forest understory: broad dark leaves with red veins; common | Stanch, Heal, Cleanse, *Sluggish* | Edible boiled as bitter greens | Wrapped over wounds as a folk dressing; wraps food for cooking |
| **Frostleaf** | Snowy mountains: low silver plant with furry leaves; uncommon | Warmth, Steady, Restore energy, *Weakness* | Tea for the cold | Stuffed into boots and gloves as lining |
| **Creeperleaf** | The Overgrowth: glossy leaves of its spreading vines; common where it grows | Strength, Swiftness, Keen senses, *Poison* | Not food for people (Turiyu graze it) | Cutting back the Overgrowth yields it |


### Roots (approved)

| Root | Where found | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Sweetroot** | Rolling plains: a pale, knobbly tuber under tall grass; common, also grown in fields | Restore stamina, Heal, Rouse, *Daze* | **Staple food**: roasted, boiled, in stews | Brewed into the Roduro root spirit |
| **Emberroot** | Rocky mountains and ravines: a red root that grows in cracks and feels warm to the touch; uncommon | Strength, Resist fire, Restore stamina, *Burn* | Spice (small amounts) | Qotiro value it as a reminder of home heat |
| **Peatroot** | Bog: long black roots dug from peat; common | Cleanse, Restore energy, *Slumber*, *Nausea* | Famine food only (bitter, boiled twice) | Tanning agent for leather; burns slow when dried |


### Minerals (approved)

| Mineral | Where found | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Saltcrust** | Coast: white crust left by sea spray in rock hollows; common | Cleanse, Restore stamina, Deep breath, *Weakness* | Seasoning | **Preserving** fish and meat (salting); trade good (existing `salt_crystal`) |
| **Storm glass** | Coast: glassy lumps fused where lightning struck sand on headlands; uncommon | Swiftness, Keen senses, Clear mind, *Blind* | Not food | Charms; flux for glassmaking (existing `storm_glass`) |
| **Rimestone** | Snowy mountains: pale blue crystal from ice caves and high ledges, always cold to the touch; uncommon | Stanch, Night sight, Steady, *Sluggish* | Not food | Laid in cellars and stores to keep food cool |


### Fungi (approved)

| Fungus | Where found | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Ghostcap** | Forest floors and ravine caves: pale mushroom that glows faintly in the dark; uncommon | Restore energy, Night sight, Clear mind, *Daze* | Not food (bitter, makes you dizzy) | Its glow marks paths in caves; existing `ghostcap` |
| **Puffcap** | Rolling plains: round white puffball; bursts into a brown cloud when ripe; common | Keen senses, Rouse, *Smoke*, *Blind* | Young ones are edible, fried | Tinder; smoking pests out of stores |
| **Vinecap** | The Overgrowth: orange shelf fungus growing on its dead vines; common where it grows | Toughen, Heal, Restore energy, *Nausea* | Not food | Amber dye; its spread on dead vine marks where the Overgrowth was cut back |

### Mosses (approved)

| Moss | Where found | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Weepmoss** | Bog: thick red moss that holds water like a sponge; common | Swiftness, Restore stamina, *Thin blood*, *Poison* | Not food | Packing and insulation; keeps roots and cuttings damp in transit |
| **Greymoss** | Coast: shaggy grey moss hanging from sea cliffs where the spray reaches; common | Stanch, Warmth, Deep breath, *Smoke* | Not food | **Smoking fish and meat** (it smoulders with thick smoke); bedding |

Fungi and mosses are now **two separate kinds** (so a mushroom + a moss counts as two kinds).


### Animal parts (approved)
All from creatures already in the game (`creatures.md`), placed in the new biomes. Turiyu are never used.

| Part | From (where) | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Wallowback fat** | Wallowback, hunted in bog and on the coast; common | Warmth, Restore stamina, Toughen, *Burn* | Cooking fat | Lamp fat; waterproofing leather; base for salves |
| **Tidepicker roe** | Tidepicker egg clusters from rock pools on the coast; common | Restore energy, Heal, Deep breath, *Nausea* | A delicacy (spoils fast) | — |
| **Ridgehound musk** | Ridgehound gland, hunted on plains and forest edges; uncommon | Keen senses, Rouse, Strength, *Nausea* | Not food | Hunters rub it on to mask their scent |
| **Silk gland** | Silkcrawler, from cliff colonies in the rocky mountains (Silk Mother guards them); uncommon | Steady, Stanch, *Slumber*, *Weakness* | Not food | Thread glue for tentsilk weaving (existing `silk_gland`) |
| **Briar heart** | Core of a slain Briarback in the Overgrowth; uncommon and dangerous | Strength, Toughen, Steady, *Poison* | Not food | Trophy (existing `briar_heart`) |
| **Cragmaw venom** | Venom sac of a Cragmaw, rocky mountains; rare, apex hunt | Swiftness, *Poison*, *Sluggish*, *Daze* | Not food | The poisoner's prize |


### Farm crops (approved)
Grown in town fields and gardens, mostly on the rolling plains (terraced on slopes). Cheap and always in stock, so they make the everyday base of common potions. Sweetroot is also grown in fields but stays a root.

| Crop | Where grown | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Hardgrain** | Fields on the plains; very common | Restore stamina, Steady, Warmth, *Sluggish* | **Staple**: bread, porridge, biscuit | Brewing ale; animal feed (existing `grain`) |
| **Stonebean** | Fields and terraces; common | Restore stamina, Restore energy, Warmth, *Nausea* | Stews; dried beans keep for months | Rotated with grain to rest the fields |
| **Hearthleaf** | Kitchen gardens in every town; common | Cleanse, Heal, Keen senses, *Daze* | Cooking herb; seasoning | Hung to dry in every kitchen; strewn on floors to freshen rooms |
| **Stoneapple** | Orchards on sheltered slopes; small, hard apples that ripen late; common in autumn | Rouse, Clear mind, Heal, *Thin blood* | Fresh fruit; dried rings keep; pressed for cider | Cider is a trade good |
| **Bluestem** | Fields of tall blue-flowered stalks on the plains; common | Stanch, Steady, Restore stamina, *Weakness* | Seeds added to porridge and bread | **Fibre for cloth and rope**; seeds pressed for lamp oil and wood finish |
| **Firepod** | Sheltered walled gardens, mostly in Qotiro quarters: a red pod plant brought from their desert home, hard to grow in the damp; uncommon | Resist fire, Warmth, Strength, *Burn* | Hot spice in Qotiro cooking | A taste of home for the Qotiro; a pricey trade crop |
| **Brinekale** | Sea-facing plots on the coast: a tough grey-green cabbage that shrugs off salt spray; common | Deep breath, Cleanse, Restore stamina, *Weakness* | Everyday greens; pickled for winter | Grown and traded by Horaro and coastal towns |


### Imported spices (approved)
Not grown on the island: they come by ship from far lands and are sold mainly by merchants in port towns. Always **expensive**. Each is a cooking spice first and an alchemy ingredient second.

| Spice | Where from | Effects (harmful in *italics*) | Food value | Utility |
|---|---|---|---|---|
| **Sunbark** | Curled red-brown bark from the desert lands the Qotiro came from | Swiftness, Warmth, Clear mind, *Thin blood* | Sweet-hot spice for breads and warm drinks | Qotiro households buy it for festivals: a taste of the old country |
| **Duskseed** | Tiny black seeds from a far southern shore | Shadow, Night sight, Restore energy, *Slumber* | Seasoning for fish and stews | Prized by those who work at night |
| **Gold thread** | Fine golden threads picked from a rare flower, sold by the pinch | Heal, Restore energy, Keen senses, *Daze* | Colours and scents rice, bread and sauces gold | **Gold dye**; the most expensive thing in a spice merchant's box |


### Oils (approved): secondary ingredients
An oil is **extracted from a base ingredient** at the alchemy table (seeds pressed, roots steeped, fat rendered). Proposed rules, kept simple:
- **Three of the base make one oil.**
- An oil keeps **two** of its base's effects, **stronger** than in the base. It drops the others, often including the harmful one.
- An oil is its **own kind**. Mixing an oil with its own base counts as two kinds, so it makes a strong two-effect potion.
- Oils keep a long time and travel well, so they're good trade goods.

| Oil | Extracted from | Effects kept (stronger) | Food value | Utility |
|---|---|---|---|---|
| **Bluestem oil** | Bluestem seeds, pressed | Stanch, Restore stamina | Cooking oil | Lamp oil; wood finish |
| **Ember oil** | Emberroot, steeped and pressed | Resist fire, *Burn* | Not food (too hot) | Warming rub for cold joints; fuel for firepots |
| **Tallow** | Wallowback fat, rendered | Warmth, Toughen | Cooking and frying fat | Candles; waterproofing leather and boots (replaces `animal_oil`) |


### Balance review (2026-10-11)
Goal (Laz): **powerful effects rare, common effects common.** Every ingredient was scored for availability (common 3, uncommon 2, rare or costly 1), and each effect was checked against its tier.

**Tiers:**
- **Common** (should be easy to get): Heal, Restore stamina, Warmth, Cleanse, Rouse, Stanch, Nausea, Daze, Weakness, Sluggish.
- **Moderate:** Restore energy, Steady, Keen senses, Clear mind, Resist fire, Deep breath, Thin blood, Smoke, Blind, Burn.
- **Powerful** (should be scarce): Strength, Swiftness, Toughen, Shadow, Night sight, Poison, Slumber.

**What was wrong, and the fixes:**

| Problem | Fix |
|---|---|
| **Toughen** came from 5 common things (Bloodleaf, Saltcrust, Vinecap, Wallowback fat, Stonebean) | Removed from Bloodleaf (now Cleanse), Saltcrust (now Restore stamina) and Stonebean. It stays in Vinecap (Overgrowth, risky), Wallowback fat and Tallow (hunting), and Briar heart (rare). |
| **Strength** came from cheap crops and fungi (Stonebean, Vinecap) | Removed from both. It now comes from Creeperleaf (Overgrowth), Emberroot, Ridgehound musk, Briar heart and Firepod: risky, hunted or pricey. |
| **Swiftness** from a farm crop (Bluestem and its oil) | Removed. Bluestem is now Stanch, Steady, Restore stamina, *Weakness*; its oil keeps Stanch and Restore stamina. |
| **Slumber** (a knockout poison) from a farm crop (Bluestem) | Removed (Bluestem's harmful effect is now Weakness). |
| **Shadow** from a common plains puffball (Puffcap) | Removed. Puffcap is now Keen senses, Rouse, *Smoke*, *Blind*. Shadow comes only from Nightberry, Mirebloom and the imported Duskseed. |
| Stonebean was too strong for a bean | Now Restore stamina, Restore energy, Warmth, *Nausea*: a filling, everyday base. |

**Result (sources per effect, after the fixes):**

| Tier | Effect | Ingredients carrying it | Of which common | Kinds |
|---|---|---|---|---|
| Common | Restore stamina | 11 | 10 | 7 |
| Common | Heal | 9 | 7 | 8 |
| Common | Warmth | 10 | 7 | 8 |
| Common | Nausea | 7 | 6 | 5 |
| Common | Rouse | 6 | 5 | 6 |
| Common | Cleanse | 6 | 5 | 5 |
| Common | Stanch | 6 | 4 | 6 |
| Common | Daze | 7 | 3 | 6 |
| Common | Weakness | 5 | 3 | 4 |
| Common | Sluggish | 4 | 2 | 4 |
| Moderate | Steady | 7 | 2 | 5 |
| Moderate | Keen senses | 7 | 4 | 7 |
| Moderate | Restore energy | 8 | 4 | 6 |
| Moderate | Clear mind | 6 | 2 | 6 |
| Moderate | Deep breath | 4 | 4 | 4 |
| Moderate | Resist fire | 4 | 1 | 4 |
| Moderate | Thin blood | 4 | 2 | 4 |
| Moderate | Burn | 4 | 1 | 4 |
| Moderate | Smoke | 2 | 2 | 2 |
| Moderate | Blind | 2 | 1 | 2 |
| Powerful | Poison | 6 | 2 | 5 |
| Powerful | Strength | 5 | 1 | 4 |
| Powerful | Swiftness | 5 | 2 | 5 |
| Powerful | Toughen | 4 | 3* | 3 |
| Powerful | Slumber | 4 | 1 | 4 |
| Powerful | Night sight | 4 | 0 | 4 |
| Powerful | Shadow | 3 | 0 | 3 |

\* Toughen's "common" sources are Vinecap (only in the dangerous Overgrowth), and Wallowback fat and its Tallow (from hunting).

**Remaining notes:**
- **Poison** has two common carriers (Creeperleaf, Weepmoss). That's left as is: weak poisons should be easy, and the strong ones (Cragmaw venom) rare.
- **Smoke and Blind** have only two carriers each. That's fine for harmful flask effects.
- **Every effect is still in at least two different kinds.**

### Recipes (Laz, 2026-10-11)
- **No fixed recipe book is needed.** Any mix of two different kinds that share an effect makes a potion, so players can work them out.
- **Recipes exist as items:** a **recipe sheet** (paper) that, once read, teaches the mix. It works like the existing spell notes.
- **Reading a sheet** teaches the mix and also reveals the effects it relies on in those ingredients.
- **Sheets turn up** in shops (alchemists, Ṭaḍoro scribes), as loot, and as rewards.
- **Writing one:** a squad member who knows a mix can copy it onto paper with the Inscription skill, to sell or to teach the rest of the squad.
- **Paper burns and soaks,** so sheets follow the container rules for fire and water.

# Part 5: Food and consumables
## Laz's decisions (2026-10-11)
- Food does more than fill hunger: **spoilage**, and **cooked meals are better than raw**. No morale or variety system, and no cultural tastes for now.
- **Cooking: simple known recipes** at a fire, hearth or kitchen. Better cooks make better meals.
- **Thirst: light.** Water is easy to find at wells, streams and rain, but it matters on long marches and when fighting.
- **Also:** alcohol, teas and broths, and smokes and chews (stimulants with a habit risk).

## Rules
1. **Spoilage:**
   - Every food has a **shelf life** in stages: **fresh → stale → spoiled**.
   - Stale fills less. Spoiled gives **Nausea** (the potion effect, with a little toxicity) and fills little.
   - Shelf life follows the clock from when the food was made or gathered. Store it as a value at a moment plus a rate, with the stage changes solved for (rule 8); never tick it.
   - **The rate changes with:**
     - **preservation** (dried, salted, smoked, pickled: long; fresh: short);
     - **container** (clay jar and strongbox slow it; a sealed sea bag keeps it dry);
     - **wetness** (wet food spoils faster, per the container rules in `armour.md`);
     - **cold** (weather temperature: snow and cold slow it);
     - **town stores and base storehouses** (slower).
   - Each change settles the item's freshness and starts a new rate.
2. **Cooking:**
   - **Cooking needs a heat source** (Laz, 2026-10-11):
     - a **campfire** (needs fuel) only does **basic food**: roast meat or fish, toasted bread, boiled water, tea and broth, simple porridge;
     - a **stove** (kitchens in homes, inns, town kitchens, and the base kitchen) makes **full meals**: stews, pies, baked bread, every dish on the meals list.
   - Meal buffs come from full meals. Campfire food fills, but gives at most a small buff (a warm drink, warmth).
   - **Recipes are known lists** (no discovery). Squad members learn more from cooks, innkeepers and manuals, like other crafts.
   - Add a **Cooking** skill. Skill sets the meal's **grade** (Crude → Masterwork), which scales how much it fills and how strong its buff is.
   - Cooking takes time (minutes to an hour) and fits the condition timeline.
3. **Meal buffs:**
   - A cooked meal fills more than its raw ingredients and gives **one meal buff** for some hours: warmth, faster stamina regeneration, faster healing, cold resistance, or a little extra carry weight.
   - Only **one meal buff** at a time; the newest replaces it. Raw food gives none.
   - **Hot meals** give a bigger warmth buff in cold weather.
4. **Thirst (light):**
   - A **thirst** value beside hunger, on the same condition timeline.
   - It drains faster with exertion (marching, fighting) and alcohol.
   - **Refilling:** wells, streams, springs (any fresh water source in the terrain data), rain (anyone outdoors in rain slowly refills, and an open waterskin catches rain), drinks, broths and juicy fruit.
   - **Waterskins** (leather or fishskin) carry water; the bigger the skin, the heavier.
   - Seawater doesn't count; drinking it adds thirst.
   - **Dehydration stages:** thirsty (slower stamina) → parched (attribute penalties) → collapse (knocked out). **It never kills** (like starvation).
   - The squad AI drinks automatically from carried water and nearby sources, and warns before a long march with little water.
5. **Alcohol** (drinks are listed under "Alcohol" below):
   - A **drunkenness** value that drains over hours.
   - **Tipsy:** a little warmth, resists fear (nerve), a slight boost to persuasion in talk.
   - **Drunk:** worse aim, dodge and sneaking; slurred talk (lower persuasion).
   - **Very drunk:** stumbling, chance to pass out.
   - Leaves a **hangover** after (slower stamina, worse aim for some hours).
   - Strong spirits add a little toxicity (the potion meter).
   - Alcohol is a trade good and part of inn life (rounds, drinking with locals as a way into talk).
6. **Teas and broths:** small, safe, warm boosts (warmth, rest, clear head, a little healing). They also quench thirst.
7. **Smokes and chews (stimulants):**
   - Short boosts (alertness, stamina, calm nerves).
   - Each use has a small **habit** chance (a keyed roll per person, with a per-person susceptibility rolled like traits).
   - A habit makes a **craving**: mild penalties when too long without (irritable, worse focus), stronger over time. It fades after enough days without.
   - **NPCs can have habits too**, which gives dialogue hooks, a trade, and a reason to steal.
8. **Town life:**
   - Town kitchens, inns and the meal runners already cook. They should cook these dishes from what the store holds (making rules, rule 17).
   - Inns sell meals and drinks.
   - Caravans carry preserved food; fresh food stays local.
9. **Fights:**
   - Eating and drinking in a fight is slow, except from belt quick slots (trail cake, a flask).
   - The AI never drinks alcohol in a fight unless ordered.

## Food recipes: 15 dishes from the approved ingredients
These replace the earlier draft meals list. Ingredients are from `ingredients.md`, plus everyday foods: meat (Brushleaper, Wallowback), fish (Silverling, Slatefin, Ribbonback), mussels, Shellhen eggs, and Mossback milk and cheese.

**Campfire (basic food: fills you, small buff at most)**

| Dish | Ingredients | Effect |
|---|---|---|
| **Roast meat** | Any meat (+ Saltcrust to season) | Fills well; no buff |
| **Spit-roast fish** | Any fish | Fills; no buff |
| **Fried puffcaps** | Young Puffcaps, Tallow | Light snack |
| **Camp porridge** | Hardgrain, water (+ Moorberries) | Small stamina-recovery buff |
| **Moorbell tea** | Moorbell, water | Small warmth; quenches thirst |

**Stove (full meals with a proper buff)**

| Dish | Ingredients | Buff |
|---|---|---|
| **Sweetroot stew** | Meat, Sweetroot, Hearthleaf | Faster healing |
| **Fish and kale broth** | Fish, Brinekale, Saltcrust | Cold resistance; quenches thirst |
| **Mussel pot** | Mussels, Brinekale, Bluestem oil | Faster stamina recovery |
| **Stonebean stew** | Stonebean, Sweetroot, Tallow | Fills the most; carry a little more |
| **Hardgrain bread** | Hardgrain, Saltcrust (baked) | No buff; keeps for weeks (travel staple) |
| **Shellhen bake** | Shellhen eggs, Bloodleaf greens, Mossback cheese | Faster healing (small); quick to make |
| **Ashberry-glazed meat** | Meat, Ashberries (as jelly) | Warmth |
| **Trail cake** | Hardgrain, dried Moorberries, Tallow (baked) | Small stamina buff; keeps for months; fits a belt quick slot |
| **Stoneapple cake** | Hardgrain, Stoneapples, Mossback milk (+ Sunbark) | Faster stamina recovery; a festival dish |
| **Firepod stew** | Meat, Stonebean, Firepod | Strong warmth; the Qotiro mess-hall favourite |

**Notes:**
- **Spices raise a dish's grade and value** without changing its buff. Sunbark, Duskseed and Gold thread are optional extras in any stove dish.
- **Brewing (ale, cider, spirits) is skipped for now** (Laz, 2026-10-11). Drinks are bought only; see "Alcohol" below.

## Alcohol (Laz, 2026-10-11)
**Decisions:**
- **Bought only for now.** Inns, taverns and traders sell drinks; the player doesn't make them yet. Brewing and distilling come later.
- **Each drink has one small effect,** a weak version of one effect from its main ingredient, on top of drunkenness.
- **Aging raises grade.** Wine and spirits get better in storage; ale and cider don't age and go off like food.
- **About 10 drinks to start.**

**Rules:**
1. **Drunkenness** works as in rule 5 above (tipsy, drunk, very drunk, then a hangover). Each drink has a **strength**: mild (ale, cider), medium (wine), strong (spirits).
2. **The small effect:**
   - It's a potion effect at low strength, lasting about an hour.
   - Only **one drink effect at a time**; the newest replaces it, as with meal buffs.
   - It doesn't stack with a potion of the same effect; the stronger one wins.
   - A drink's grade scales the effect a little.
3. **Aging:**
   - Wine and spirits go up a grade after enough time in storage: weeks for wine, months for spirits. **Wine tops out at Fine; spirits can reach Masterwork.**
   - Stored as a value at a moment plus a rate (rule 8), with the grade changes solved for, never ticked.
   - Aging only happens in **storage**: a cellar, storehouse, strongbox or clay jar. Carrying it around in a pack doesn't count.
   - **Ale and cider spoil** on the food shelf-life rules (S to M). Spoiled ale gives Nausea.
4. **Price** follows the existing grade multipliers (Crude ×0.5, Common ×1, Fine ×2, Masterwork ×4). An aged bottle is a good trade good and a nice thing to steal.
5. **Where it's sold:**
   - Inns and taverns mostly sell Common, plus whatever's local.
   - Richer inns and traders have aged bottles.
   - Each people's own drink is cheap in its own towns and dearer elsewhere.
6. **Thirst:** ale and cider quench a little thirst. Wine and spirits add to thirst.
7. **Bottles are fragile:** glass and clay crack on the potion rules (falls, fire, blunt hits); the potion case protects them.

**The drinks:**

| Drink | Type | Strength | Small effect | Who drinks it / where | Ages? |
|---|---|---|---|---|---|
| **Hardgrain ale** | Ale | Mild | Restore stamina | Everyone, every inn; the common drink | No (spoils) |
| **Stonebean dark** | Ale | Mild | Warmth | A thick, dark ale; labourers' and porters' drink | No (spoils) |
| **Stoneapple cider** | Cider | Mild | Rouse | Plains towns and orchards; festival drink | No (spoils) |
| **Moorberry wine** | Wine | Medium | Clear mind | Purple-blue, everyday wine across the plains | Yes, to Fine |
| **Ashberry wine** | Wine | Medium | Warmth | Orange, sharp; mountain inns, a winter drink | Yes, to Fine |
| **Nightberry wine** | Wine | Medium | Shadow | Near-black, rare and costly; favoured by thieves and the night trade | Yes, to Fine |
| **Brine wine** | Wine | Medium | Deep breath | **Horaro.** Salty, pale-green, from fermented Brinekale; an acquired taste | Yes, to Fine |
| **Stone spirit** | Spirit | Strong | Steady | **Roduro.** Clear Sweetroot spirit, slow-sipped from small stone cups | Yes, to Masterwork |
| **Sacrament spirit** | Spirit | Strong | Resist fire | **Qotiro, rites only.** Firepod spirit; the recipe came with them from the desert. Never sold (see below) | Yes, to Masterwork |
| **Gold thread cordial** | Cordial | Mild | Restore energy | **Ṭaḍoro.** Sweet, golden and pricey; made with imported Gold thread; scholars' and merchants' drink | Yes, to Fine |

Spirits also add a little toxicity (rule 5). Native names come from the naming module.

**Qotiro and alcohol (Laz, 2026-10-11):**
- **The Qotiro don't drink, except at their sacraments.** Inns in Qotiro towns and quarters serve no alcohol.
- Drinking openly there draws disapproval (talk goes worse, and guards watch you). This is a **custom**, read from customs and never from race, so a Qotiro raised elsewhere might drink.
- **Sacrament spirit** (the old Fire spirit) is poured only at rites, by priestesses. It's **never sold**. A stolen bottle is valuable, and its theft is sacrilege where customs say so.

## Smokes (Laz, 2026-10-11)
**Decisions:**
- **The Qotiro smoke instead of drink.** Their towns have **smoke halls**: large halls where incense that gets you high is burned for everyone at once. Smoking there is a communal thing.
- **Roduro and Ṭaḍoro smoke now and then,** from pipes (Roduro) and water pipes (Ṭaḍoro).
- **Bought only,** like drinks. Each smoke has **one small effect**, on the same rules as drinks (one at a time, about an hour, scaled a little by grade).

**Rules:**
1. **Haze** is its own value, beside drunkenness. It drains over hours, stored as a value plus a rate with the stage changes solved for (rule 8).
   - **Mellow:** resists fear, slightly better talk with the people you're smoking with, slower stamina recovery.
   - **Hazy:** slower reactions (a little worse swing and dodge), worse aim, worse at noticing hidden things.
   - **Stupor:** dozes off. It's like being knocked out but light: any nudge or Rouse wakes them.
   - **No hangover.** Instead, each smoke carries the **habit** chance from rule 7: a craving if you go too long without, which fades after enough days.
2. **Smoke halls (Laz, 2026-10-11):**
   - **Two rooms:**
     - **The chamber:** dim (low light under the light rules), with incense burners. Most guests are in here, just hanging out. **Being inside raises your haze** at the chamber's rate, whether or not you came to smoke.
     - **The antechamber:** the way in. You can stand and talk here **without partaking**; it doesn't raise haze.
   - **Leaving the chamber** starts haze draining. This is solved like any other rate, never ticked.
   - **The keeper** (the hall's "bartender") works from the antechamber. They pop into the chamber now and then to check on guests and swap spent burners for fresh ones. Their rounds are on the clock, like other town life.
   - **The communal side:** sitting in the chamber with Qotiro is the way into talk with them, as drinking with locals is elsewhere. Staying a while builds a little standing. Turning it down and staying in the antechamber is fine.
   - A hall burns its incense from dusk into the night. The keeper buys it with the town's stores (making rules, rule 17).
   - **Building:** the smoke hall is a Qotiro town building (a note for the town forge, and for Qotiro quarters in Roduro towns). It needs an antechamber, a chamber, and a back store for incense.
3. **Pipes and water pipes:**
   - **A pipe** is a personal mundane item (it's already in the trinkets list). Smoking it gives a small haze and the blend's effect.
   - **A water pipe** is furniture in Ṭaḍoro houses and teahouses. It's shared by up to four people at once, and has the same talk bonus as a smoke hall, on a small scale.
4. **The drunk and high meter (Laz, 2026-10-11):**
   - Each person shows **how drunk** and **how high** they are, as a meter on their portrait or panel.
   - **It only appears when it matters:** it shows up once someone drinks or smokes (or sits in a chamber), and disappears again once they're sober. Someone who never drinks never sees it.
   - It's one meter with the current stage named (tipsy, drunk, very drunk; mellow, hazy, stupor). If someone is both drunk and high, it shows two bars.
   - **It builds up faster than it fades.** Each drink is a quick jump, and a chamber raises haze quickly. Draining takes hours. Both are a value plus a rate, solved for (rule 8).
   - **Visual cue** (rule: every action has one): a drunk or hazy person sways or moves slower, so you can spot it in the world without opening the meter.
5. **AI:** the squad never smokes in a fight. NPCs with a habit go looking for a smoke, which gives dialogue hooks and a reason to steal.

**The smokes:**

| Smoke | How it's used | Ingredients | Haze | Small effect | Who and where |
|---|---|---|---|---|---|
| **Hall incense** | Burned in smoke halls | Greymoss, Duskseed | Mild | Steady | **Qotiro.** The everyday evening blend |
| **Ghost incense** | Burned in smoke halls | Ghostcap, Greymoss | Strong | Clear mind | **Qotiro.** Late-night halls; strange, vivid thoughts. Dearer |
| **Sun incense** | Burned at rites only | Sunbark, Gold thread | Mild | Rouse | **Qotiro.** Burned with the sacrament spirit. Not sold |
| **Drift smoke** | Pipe | Dried Moorbell, Greymoss | Mild | Warmth | **Roduro**, and anyone. The plain pipe blend sold everywhere |
| **Elder's blend** | Pipe | Moorbell, Duskseed | Medium | Steady | **Roduro.** Dark and slow; elders smoke it at the hearth |
| **Sweet water-pipe blend** | Water pipe | Sunbark, dried Stoneapple | Mild | Clear mind | **Ṭaḍoro.** Smoked in teahouses over long talk and ledgers |

Sold as cones and bricks (incense) or pouches (pipe blends). Native names come from the naming module.

**Later:**
- Brewing at a base or kitchen (ale and cider), and a still for spirits.
- Drinking with locals as a way into talk (already a hook in rule 5).

## Data and other systems
- Foods, meals, drinks and recipes are data (one recipe table, as crafting has).
- Thirst, drunkenness and cravings join the condition timeline. Update the save format (rule 12).
- The kitchen buildings (base hearth kitchen, town kitchens, inns) use the cooking recipes.
- The squad AI eats the best available meal, drinks when thirsty, and eats spoiling food first.

# Part 6: Tools
Kept light: each tool has one clear job, using systems that already exist or are already planned. Tools are carried in the pack and used, not worn (`Kind::Tool`). Better grades work faster or more reliably.

## Work and gathering

| Tool | What it does |
|---|---|
| **Woodcutter's axe** | Faster timber at woodlots (the hatchet still works, but slower) |
| **Mining pick** | Needed to work ore seams and rock for stone; faster with better grades |
| **Sickle** | Faster harvesting of crops and plants |
| **Fishing line** | Catch fish from the shore |
| **Skinning knife** | More hide and meat from a carcass (any knife works, but yields less) |

## Camp and travel

| Tool | What it does |
|---|---|
| **Tinderbox** | Lights campfires and torches; without it, lighting a fire is slow |
| **Cooking pot** | Lets a campfire make porridge, tea and broth; without it, only roast and spit food |
| **Waterskin** | Carries water (food doc) |
| **Lantern** | A longer-lasting light than a torch. It can be shuttered, which hides its light, for sneaking |
| **Repair kit** | Field repair of gear: partial durability back. A full repair still needs a station and the right craft |

## Thieving

| Tool | What it does |
|---|---|
| **Lockpick** (existing) | Picks locks (Security skill); can snap |

## Already covered elsewhere
- **Alchemy tools:** mortar and pestle, retort, calcinator, alembic (potions doc).
- **Bandages** (weapons doc).
- **Containers:** pouches, scroll case and so on (armour doc).

# Part 7: Trinkets, mundane goods and valuables
Small things worn or carried whose value isn't fighting. Three groups: jewellery, folk charms, valuables. (Tokens of the peoples were dropped.)

## Jewellery
Slots: two rings, neck, brooch, belt (from `armour.md`, accessories).
- **Enchanted pieces give small buffs for now**, through the existing effect system. For now they're found or bought only; making them waits for enchanting.
- **Every piece also has a mundane side**, from its material and how showy it is:
  - **Fine jewellery** (gold, gems, carved Nacre) improves how people react to you: respect in talk, slightly better first prices with some sellers.
  - **But it makes you a juicier target.** Thieves, pickpockets and bandits read it as wealth, and pick you out.
  - **Plain jewellery** (bronze, bone, Ringstone) does neither.

**Starter set** (the five existing pieces plus a few, so each slot has options):

| Piece | Slot | Small buff | Showy? |
|---|---|---|---|
| Ring of Swiftness (existing) | Ring | Slightly faster | Plain |
| Ring of the Ox (existing) | Ring | Slightly stronger | Plain |
| Hearthstone Ring (existing) | Ring | Small warmth | Plain |
| Ring of Warding | Ring | Slight resistance to fire | Fine (gold) |
| Wellspring Amulet (existing) | Neck | Slightly faster energy recovery | Fine |
| Amulet of the Clear Mind (existing) | Neck | Easier casting | Fine |
| Steady Torc | Neck | Small resistance to stagger | Plain (Ringstone) |
| Keen-eye Brooch | Brooch | Slightly keener senses | Fine (Nacre) |
| Traveller's Brooch | Brooch | A little less tiring on long marches | Plain |
| Fine Belt of Bearing | Belt | Carry a little more | Fine |

## Mundane goods
Everyday household and personal things. They have no special use: they're **sold, traded, stolen**, and found in homes, packs and loot so the world feels lived-in. They're worth little each, but add up.

**Household**
- Clay cup, clay bowl, Hearthclay jug.
- Wooden spoon, ladle, carved trencher (a wooden plate).
- Tallow candle, candle stub, rushlight.
- Iron pot hook, bronze lamp.
- Woven basket, reed mat, wool blanket.
- Broom.

**Personal**
- Bone comb, horn comb.
- Pipe and a pouch of drift smoke.
- Dice (bone), a set of game stones.
- Sewing kit, spool of thread, bone needles.
- Small mirror (polished bronze).
- Purse (leather, empty).

**Writing and reading (Ṭaḍoro trade)**
- Blank sheets, a bundle of reed paper, ink pot, quill.
- Letter (unsealed, someone's private news).
- Ledger (a merchant's or a household's accounts).
- Old map fragment (just a curio for now).
- Book of songs, book of prayers.

**Work leftovers**
- Bent nails, iron scrap, broken blade.
- Net fragment, frayed rope.
- Cracked pot, worn tool handle.

## Folk charms
No real magic. They're for decoration, trade and flavour; people notice them in conversation. Cheap and common, so they make loot feel lived-in.

**Roduro**
- **Ring-stone pebble:** a naturally banded pebble, kept for long life.
- **Home chip:** a flake of grown stone from a family home, carried when travelling.
- **Carved Turiyu:** a small figure of the revered animal, in wood or bone.

**Horaro**
- **Knotted sea cord:** sailor's knots tied for a safe return.
- **Shell rattle:** small shells strung on cord, shaken at the tideline.
- **Drowned bell:** a tiny bronze bell found in the shallows, thought lucky.

**Qotiro**
- **Sun charm:** a small brass disc, a cheap copy of the temple sun discs.
- **Ember bead:** a red glass bead, for keeping the "home fire" in mind.
- **Sand vial:** a pinch of sand from the old desert, passed down.

**Ṭaḍoro**
- **Folded prayer:** a tiny folded paper with a written blessing.
- **Feather quill charm:** an old quill tied with thread.
- **Road token:** a stamped bronze coin-like token from a far town.

**Anyone**
- **Ashberry ward:** a sprig of orange berries tied over a door.
- **Stormbell locket:** a pressed pink Stormbell behind glass.
- **Lucky stone:** a stone with a hole worn through it.
- **Moorbell posy:** dried blue flowers given as a keepsake.

## Valuables
They exist to be **sold, stolen or used as bribes**: a lot of value in very little weight, and a common target for thieves. Listed roughly cheapest to dearest.

- **Storm glass bead:** polished.
- **Bronze bangle:** heavy, plain.
- **Silver ring:** a plain band.
- **Silver bar:** small.
- **Pearl:** existing item.
- **Carved Nacre:** a shell carving of a fish or a wave.
- **Gold nugget:** existing item.
- **Cut gem:** a red or a green stone.
- **Gold chain.**
- **Edgeglass sliver:** an offcut of grown glass. Pretty, rare, and very sharp.
- **Engraved gold cup:** a household's pride; its theft is big news.
- **Black pearl:** very rare, from the deep beds.

# Appendix: biome changes for other game docs
## Game docs to update for the biome list (for the agents)
- **Code notes:** `CLAUDE.md` says the south-east inland is "Qotiro country, a raised arid plateau". Remove it: there's no desert or plateau.
- **`creatures.md` and the animals module:**
  - **Dustrunner** lives on the "Qotiro plateau". It needs a new home (rolling plains) or should be dropped.
  - **Wallowback** ("river valleys, coastal marsh") moves to bog and coast.
  - **Mirejaw** ("marsh and river edges") moves to bog.
  - **Brushleaper** ("hills, scrub") moves to plains and forest edges.
  - **Crag grazer and Cragmaw** ("massif") fit rocky mountains.
  - **Chasm lurker** fits ravines.
- **Weather climate regions** (`data/weather/climate.ron`): check that they match the biomes.
- **Terrain:** make sure the world generator only produces these biomes.
