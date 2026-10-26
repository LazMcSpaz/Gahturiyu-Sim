# Test Contract: how every generator agent writes its output

You are one of several agents generating parts of one **test world** for the quest generator's shape layer. Generate **by hand, following the rules exactly**. Finding places where the rules are ambiguous, contradictory or produce dull results is half the point.

## Inputs (all in /home/claude/quests/)
| File | What it is |
|---|---|
| `rulebook.md` | the shared rules; wins every conflict |
| `card-schema.md` | field names |
| `assembly.md`, `main-quest.md` | cast, plants, ordeal doors, chapters, moves |
| `fallout.md` | fallout quests |
| `locations.md` | pinning, location pass, clocks, late fallout |
| `faction-stories.md`, `guild-kits.md` | profiles, faction stories, kits |
| `family-*.md` (9) | every family you use |
| `factions.md` | the generator's faction spec |
| `faction-places.md` | fixed places' landmark, trade and hook; presences §5b |
| `world-factions.md` | faction canon: voices, depth |
| `story-rubric.md` | what the reviewers will judge (coherence, span, depth, story quality) |
| `test/world/map.json` | **the island**: places, size, quota, territory, region, coordinates, `near` lists, roads. Use these place ids. |
| `test/world/canon.json` | factions and **pre-made leaders**. Reuse these ids; never invent leaders. Top roster: Gahìru only. |

## Output: one JSON file
```json
{
  "agent": "<your prefix>",
  "registry": {"people": [...], "places": [...new places only...], "things": [...]},
  "facts": [...],
  "story": {...},
  "faction_stories": [...],
  "cards": [...],
  "log": "path to your log .md"
}
```
- `story` is only for the main-story agent; `faction_stories` only for guild agents. Omit what isn't yours.

**Ids and names:**
- **Your prefix** (given in your brief) goes in every id you create: `p-<prefix>-<name>`, `pl-<prefix>-<name>`, `t-<prefix>-…`, `f…` facts as `x-<prefix>-…`, cards `c-<prefix>-…`. Use map and canon ids as they are.
- **New places** (a farmstead, a hideout, a camp) go in `registry.places` with `kind`, `owner`, `access`, `standing`, `territory`, `region`, and `at` (the map place it sits in or beside). Its `x`/`y` are within 0.6 of that place.
- **Names:**
  - full names are unique in your output;
  - follow rulebook §3 (loosened): no shared first-name stem within one card, chain or location;
  - use culture-neutral, plain fantasy names; vary their sounds.

**Person:** `id`, `name`, `standing`, `traits`, `links` [{`type`, `to`, `as`, `visibility`}], `faction` {`id`, `sub`} or null, `facts`, `premade` (true/false), `dead` (true/false), `home` (place id), `fallback`.

**Fact:** `id`, `text`, `visibility` (public/secret), `kind` (past event / state / trait / plant / move), `published_by` (revelation only), `source` (card id).

**Card:** every field in `card-schema.md` that applies, including:
- `id`, `family`, `premise`, `variant`, `relatedness`
- `scope`, `place`, `offered_at`, `giver`, `giver_faction`, `signature`
- `slots`, `opposing_slot`, `reasons`, `motive`
- `truth` (a fact id), `pressure`, `separation`, `twist` {`name`, `group`}
- `ways_pool` [{`text`, `kind`, `source`, `requires`, `names_slot`, `price`, `code_forbidden`}], `ways_rolled`, `ways_removed`
- `possible_endings`, `plants` [{`fact`, `type`, `knower`}], `details`, `depends_on`
- `clock`, plus the story fields: main `chapter` / `position` / `move` / `door`; faction `faction_story` / `phase` / `odd_job` / `private_side` / `far`; side `attached_to` / `advantage` / `source_move` / `move_fills` / `face` / `echo_of` / `linked_pair`
- **`place_tie`**: one line on how the card uses what's distinctive about its location (the landmark, trade, hook or presence), or `null` if it doesn't

**Main story** (`story`): exactly the fields in `card-schema.md` §3. **Faction stories:** exactly the fields in `faction-stories.md` §13, plus `id`.

## Log (markdown, beside your JSON)
1. Rolls and rerolls (use Python `random` with your given seed; record them).
2. A one-line summary per card.
3. **Rule problems:** every place a rule was ambiguous, contradictory or impossible, citing doc and section.
4. **Story notes:** where following the rules produced something dull, repetitive or incoherent, and what would fix it.

## Self-check before you finish
- Run `python3 /home/claude/quests/test/fs-check.py <your.json>` if you made faction stories. It needs `faction_stories`, `cards`, `registry` and `facts`, and reads canon from your registry, so **copy the canon people and factions you use into your registry**.
- Main story: run `python3 /home/claude/quests/test/check.py <your.json>`.
- Fix what they flag. If you think a complaint is a checker bug, leave it and explain in the log.
