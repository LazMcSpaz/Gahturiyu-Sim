# Quest generator test tools (Python, not part of the game build)

These are the checkers and viewers used while the quest generator was designed on paper, before the Rust prototype existed. Agents stood in for the generator code. The rules they check live in the claude.ai Project (`claude/quest-*.md`); the overview is `docs/quest-generator.md`.

The Rust generator (`src/questgen/`, to come) replaces these one by one. Until then they are the **reference behaviour**: a Rust checker should flag the same things on `samples/` as these do.

| File | What it does | Run |
|---|---|---|
| `world_check.py` | World-level checks: references, reuse, names, states, loose threads, coverage and span, depth, place ties, variety, clocks, travel, the v5 checks (blank cards, plant-backed ways, main cards per settlement, given names) and guild placement (faction-stories §8b) | `python3 world_check.py samples/world-B.json report.md metrics.json` |
| `fs-check.py` | Faction-story checks (phases, craft rules, ledger, crisis at HQ) | `python3 fs-check.py <faction-stories.json>` |
| `check.py` | Main-story quest-set checks (plants, twists, chapters) | `python3 check.py <questset.json>` |
| `digest.py` | A readable markdown digest of a world, for human or agent readers | `python3 digest.py samples/world-B.json > digest.md` |
| `build_viz.py` + `world-viz-template.html` | The clickable map view (place → quests → card) | `python3 build_viz.py out.html <world.json> <title> <tab> <blurb> <findings.md or -> <check.md or ->` (repeat the six arguments for more worlds) |
| `build_map.py`, `build_canon.py` | Built the hand-made test island (`samples/map.json`) and its canon (`samples/canon.json`) | — |
| `roll_kits.py`, `plan_locations.py`, `merge.py` | Paper-test helpers: rolled guild kits, planned location batches, merged agents' outputs into one world | — |
| `CONTRACT.md`, `LOCATION-BRIEF.md` | The briefs the stand-in generator agents worked to | — |

**World C tools:** `build_map_c.py` (the island with Lanternhall in Carter's Rest and guild halls in every large town), `roll_kits_c.py` (guild kits with variants and the world ledger), `merge.py` (now takes `QMAP`/`QCANON` and add-only `registry_updates`), and the briefs `CONTRACT-C.md` and `LOCATION-BRIEF-C.md`.

**Samples:**
- `samples/world-A.json` is the guild suite, 54 cards.
- `samples/world-B.json` is the full world, 188 cards on the 54-place island.
- `samples/world-C.json` is World C, 152 cards under rules v5.2, schema v1.10 and casting v1.1; it has its check report and `world-C-results.md`.
- `samples/*-check-v5.md` are their check reports.
- World B predates rules v5 and §8b, so it fails some of the newer checks on purpose. The reports show what a pre-v5 world gets wrong.
