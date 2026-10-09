# UI brief: Gahturiyu Sim

What an agent working on the interface needs to know to redesign it and make
mockups. Written 2026-10-09, at save format 24 / 273 tests. The repo's
`README.md` (controls, systems) and `CLAUDE.md` (rules, drawing notes) are
the source of truth; read them too.

## 1. The game, in one paragraph

A Kenshi-meets-Morrowind squad sandbox in Rust. You control a squad of four
(one of each people: Roduro, Horaro, Qotiro, Ṭaḍoro) in a simulated world of
~5,000 people in 30 mixed towns. Everyone has a schedule, a job, a household
purse, memories, grudges and gossip; towns have customs, governments, law,
an economy and (in big towns) a criminal ring. Fights are Kenshi-style (six
body parts, knockouts, lost limbs), magic is Morrowind-style (59 spells, three
styles). Jobs come out of people's lives and are found by **talking**, not on a
quest board. Time runs from real time to 1 game hour per second.

**The designer** is Laz. He isn't a game developer: label things in plain
words, no jargon, and explain choices plainly. He plays on a Windows desktop
and tweaks through conversation with Claude, not in an editor.

## 2. What the player does (the UI must serve these, roughly in order)

1. **Move and command the squad**: select members (cards, F1–F4, Shift adds),
   click ground to walk, click people to talk, bandits to attack, doors to
   enter, things to pick up or gather. Sneak (Z), rest (N), torch (T), carry
   the downed, put them down (X).
2. **Watch their condition**: health per body part, energy (casters), food,
   stamina, rest, carried load, lost limbs, statuses (burning, wet...).
3. **Fight**: mostly automatic; the player picks targets, retreats, casts.
4. **Talk**: conversations assembled from what's on the speaker's mind;
   options (ask more, offer help, report, persuade, pay, threaten) depend on
   who's talking; plus trade, lessons, orders, hall business, jobs.
5. **Manage gear**: pack and worn slots (10), weight, wear and mending,
   eating/drinking, dropping.
6. **Craft** (seven crafts at stations), **cast** (spell book, aiming).
7. **Understand towns**: customs, food, stock and prices, government and law,
   services open now, what's happened lately, the ring, work heard of.
8. **Track jobs** (journal), bounties and standing per town, contracts.
9. **Read the world**: hover anyone/anything for a description; map view.
10. Save/load (F8/F9), speed and pause, graphics settings (O).

## 3. Every screen there is now

All panels are drawn every frame from the world; the screenshots in §8 show them.

| Panel | Where | Opened by | Shows |
|---|---|---|---|
| Side panel (HUD) | top-left | always | day/time, speed, population, light, ambushes, band counts, frame timing, view name, race legend, the 14-line event log |
| Squad cards | bottom-left, 4 × 210×100 px | always | name, state ("Standing", "Down"), carried kg, health bar, energy bar, food/stamina/rest bars with warning words, carrying, lost limbs, torch, held ritual |
| Hover tooltip | by the mouse | hover | person: name/race, traits, stats, skills, gear, spells, per-part HP, might, statuses, work, life (work status, worries, honour, grudge, household purse/debts/feelings), what they're doing now; also groups, items, doors, torches, stations |
| Pack / gear | centre | I, right-click card | worn slots, pack list, weight; click/right-click actions; item tooltips |
| Crafting | centre | K | recipes for the member, inputs, station needed |
| Spell book | centre | M | spells, costs, chances; then click-to-aim |
| Conversation | bottom-centre, above cards | click a person | speaker, regard ("Disposition"), the talk so far (left), topics/options (right), Esc to leave |
| Journal | bottom-left, above cards | J | one line per job |
| Town panel | right, 590 px wide, very tall | P or click a town name | both communities' blend and customs, food paths, store (34 goods with arrows), prosperity, coin, shelf, government (chambers, council, arbiter, matters), law customs, bonds, rite, services open now |
| Town talk ("Lately") | beside the town panel, 470 px | with the town panel | recent events, the ring's leader (once found), work heard of; with L: every opportunity, event and storyline |
| Graphics settings | centre | O | foliage, shadows, lamps, bloom |
| Detail readout | right | L | draw distances and triangle counts (a developer tool) |
| Map | full screen | V | relief, towns, roads, groups, squad, band rings (R) |
| Over-head | in the 3D view | always | health bars, town names, barks (one-line remarks in quotes) |

Text sizes in use: 12–19 px. Colours: dark translucent panels
(`hud::PANEL` ≈ rgba 4,6,6,219), text off-white, dim grey-green for
secondary, gold for headings/emphasis, orange for warnings, a colour per
people (`palette::race_color`: Roduro sand, Qotiro orange, Horaro blue,
Ṭaḍoro lilac), violet for held magic, blue for energy. Font: DejaVu Sans.
No icons yet: everything is text and bars.

## 4. Controls now

See README › Controls for the full table. Keys in use: V (3D/map), F1–F4,
` / Esc, X, N, Z, T, I, K, M, G, J, P, O, F8/F9, Q/E, WASD, C, Space, 1–5
(speed), R (rings), L (detail), B (debug bandits). Mouse: left-click acts,
right-click opens a card's pack / cancels aiming / drops items, right-drag
turns, middle-drag pans, wheel zooms.

## 5. Known problems worth solving

- **Too much text, little hierarchy.** The town panel is one long column of
  small text; the hover tooltip for a person can run 15+ lines.
- **Panels overlap.** Town panel + town talk + detail readout + log all
  compete; there's no layout system, just fixed rectangles.
- **Nothing tells the player what's going on with people** at a glance: who's
  troubled, who has work, who's hostile. It's all in tooltips.
- **Conversation** shows options as a plain list; no sense of who you're
  talking to (portrait, mood, regard trend) or which options are gated and why.
- **Jobs** are one line each; no tracking of where to go or what's next.
- **No notifications** beyond the 14-line log (arrests, thefts nearby, pay
  received, a contract shift starting).
- **Squad cards** are dense; statuses and conditions are words, not icons.
- The map is functional but bare (no legend, filters or labels for jobs).

## 6. Hard constraints (how the UI is built)

- **Engine**: Bevy 0.19.1 (pinned) with bevy_egui 0.42.0. Panels are drawn
  with egui's *painter*, not egui widgets, through `view/hud.rs`'s `Canvas`:
  pixel coordinates from the top-left, text placed by its baseline,
  `rect`, `text`, `centred`, `panel`, `circle`, `line`, `triangle`. A new
  look can keep this approach or move to egui widgets — say which, and why.
- **Clicks** on panels are hit-tested against the boxes (`Bx`) drawn the
  frame before; a click on a panel is not a world order. Panels return an
  `Action` (or a `Topic` for talk) that `app.rs` carries out.
- **The window never contains world rules** (CLAUDE.md rule 6). It reads
  `World` and calls its order methods (`order_squad`, `order_talk`, `ask`,
  `take_opportunity`, `press`, `toggle_torch`, ...). Anything purely visual
  (caches, which panel is open, barks) lives in `Game` in `view/app.rs`.
- Two cameras: 3D (HDR, bloom) and a 2D egui camera on top (also HDR). Don't
  change that or the 3D view goes black.
- **Screenshots without a screen**: every panel can be put on screen by
  `GAHT_` flags (CLAUDE.md lists them: `GAHT_INV`, `GAHT_CRAFT`, `GAHT_BOOK`,
  `GAHT_TALK`, `GAHT_CONVO`, `GAHT_TOWN`, `GAHT_GUARD`, `GAHT_FEUD`,
  `GAHT_HOVER=x,y`, `GAHT_SELECT`, ...):
  `GAHT_SHOT=out.png GAHT_FRAMES=40 GAHT_CONVO=1 xvfb-run -a -s "-screen 0 1600x1000x24" ./target/release/gahturiyu`.
  Each new panel should get a flag so it can be checked this way. Rendering
  is software (~4 fps) in the container; ignore the fps readout.
- Target screen: 1600×1000 for checks; Laz plays on a Windows desktop
  (assume 1920×1080 and up). Should scale.

## 7. What can be shown (the data is there)

All `pub` on `World` (in `src/sim/`):

- **People**: `people[pid]` (race, traits, stats, skills, wounds, might,
  gear via `detail`), `life(pid)` (job, workplace, household, habits incl.
  honour), `mind(pid)` (work status, needs via `.needs()`, memories, known
  events), `why_not_working`, `top_need`, `top_grudge`, `regard_of(npc, member)`,
  `on_mind(npc)` (what they'd talk about), `doing_now`, `day_plan`.
- **Households**: purse, debts, feelings toward others (`society.households`),
  `household_need`.
- **Towns**: `government`, `gov_words`, customs, food, `stock_now`, prices
  (`price_factor`, `dearest`), shelf, `service_open`, `events(town)`,
  `ring(town)` (`found` = squad knows the leader), `known_opps(town)`,
  `hall_board(town)`, `story_cap`, `drama`, `would_join(town)` (recruitment, later).
- **Squad**: members, positions, conditions (`cond`), carried, torches,
  bounties and `bounty_known_in`, `standing(member, town)`, `contract_of`,
  `quests` + `quest_line`, `squad_count(item)`.
- **Talk**: `talk` (the conversation: lines, concerns, tried options),
  `topics()`, `options(...)`, `bark(npc, member)`.
- **Fights**: `battles`, `fighter(pid)` (and `hud::battle_lines` in the view).

## 8. Mockups wanted (a suggestion; agree the list with Laz)

1. **The main screen**: squad cards, a slimmer status/time bar, notifications,
   minimap or compass; a calm default with panels on demand.
2. **A conversation**: speaker portrait/name/people/job, regard and mood,
   what's on their mind, gated options (greyed with the reason), trade and
   job offers as their own tabs.
3. **A town screen**: tabs (People & customs / Food & store / Government &
   law / Lately & work), replacing the one long column.
4. **A person inspector**: the hover tooltip as a pinned card — body diagram
   for wounds, gear, mood, household, grudges.
5. **Journal and jobs**: what's next, where, deadline, who asked.

Show them as static images (HTML or drawn), sized 1600×1000, using the
existing palette and font, then build the chosen ones in Rust with a `GAHT_`
flag each and compare against real screenshots. Current screenshots: run the
flags in §6 (`GAHT_CONVO`, `GAHT_TOWN=1`, `GAHT_FEUD=1`, `GAHT_GUARD=1`,
`GAHT_INV=0`, `GAHT_BOOK=3`, `GAHT_HOVER=…`).

## 9. Working rules

- Read `CLAUDE.md` first (rules 1–19; "Drawing notes").
- Don't change anything in `src/sim` for looks; if a panel needs data that
  isn't exposed, add a read-only `pub fn` there and say so.
- Keep every keyboard shortcut working, or list what changed.
- Run the tests before every commit (`cargo test --release`); never commit
  untested code. Don't run `cargo fmt` over the repo.
- Commit messages end with the co-author lines the repo uses.

## 10. Questions to put to Laz

- Look and feel: keep the dark translucent panels, or a more themed look
  (parchment, stone, per-people styles)?
- Icons and portraits: is placeholder art OK, and where does final art come
  from?
- How much should be visible by default versus on demand?
- Mouse-first, keyboard-first, or both equally? Controller ever?
- Screen sizes to support (laptop? ultrawide?).
