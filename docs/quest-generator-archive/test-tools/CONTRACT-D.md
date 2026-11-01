# Test contract, World D: how every generator agent writes its output

You are one of several agents generating one part of **Test World D**. Generate **by hand, following the rules exactly**.
- This is a paper test of the quest generator's current design: **rules v6** (each doc's version is in its title; card schema v1.11). Laz has frozen the scope.
- Finding where the rules are ambiguous, contradictory or produce dull results is half the point.
- **Don't invent new rules or features.** Where a rule is missing, make the smallest sensible call and log it.

## Inputs (all in /home/claude/quests/)

| File | What it is |
|---|---|
| `rulebook.md` | the shared rules; wins every conflict |
| `card-schema.md` | **v1.11** field names (new in v6: card `truth_row`, `shapes`, `plain`, `reads_context`; context facts carry `when`; `registry_updates`) |
| `casting.md` | **how slots are filled** (§6 the loop, §7 creating): reuse with a reason, otherwise create; seat holders are looked up |
| `assembly.md`, `main-quest.md` | cast, plants, ordeal doors, chapters, moves; **main-quest v3.5's craft rules** |
| `fallout.md` | fallout quests |
| `locations.md` | pinning, the location pass, settlement context, per-place caps, clocks, layers of power (§7a) |
| `faction-stories.md`, `guild-kits.md` | profiles, the **six** faction stories (the five guilds and the Holy House), including §8b (guilds share the big towns), chapter-2 targets (§6), kits with variants |
| `family-*.md` (9) | every family you use, including their **truth tables** |
| `factions.md` | the generator's faction spec |
| `faction-places.md` | fixed places, presences (§5b), guild reach, minor-faction profiles (§7) |
| `world-factions.md` | faction canon: voices, depth |
| `leaders.md` | the leader archetypes and their canon defaults (traits, secrets, goals, foils) |
| `story-rubric.md` | what the reviewers will judge |
| `test/world/map-D.json` (the test island; every large or great town holds a hall of all six joinable factions) | **the island**: places, quotas, regions, `near` lists, roads, in-town presences. Use these place ids. **Lanternhall is inside Carter's Rest.** |
| `test/world/canon-D.json` | factions and canon leaders: the defaults |
| `test/world/D/d-l0.json` | **layers 0–1, already made:** archetype fillers (some canon, some generated), every seat and office holder, and the court tensions. **Reuse these people; never invent a second holder of a seat** (rulebook, *Seats and offices have one holder*). |
| `test/world/voices-ref.md` | the slugs for `people`, `age` and `voice` |

## Output: one JSON file

```json
{
  "agent": "<your prefix>",
  "registry": {"people": [...], "places": [...new places only...], "things": [...]},
  "facts": [...],
  "story": {...},
  "faction_stories": [...],
  "cards": [...]
}
```

- `story` is only for the main-story agent; `faction_stories` only for the guild agents.
- **Ids:** your prefix goes in every id you create (`p-<prefix>-…`, `pl-<prefix>-…`, `t-<prefix>-…`, facts `x-<prefix>-…`, cards `c-<prefix>-…`). Use map, canon and d-l0 ids unchanged.
- **New places** (a farmstead, a hideout, a site in a town): give `kind`, `owner`, `access`, `standing`, `region` and `at` (the map settlement it sits in or beside). A site inside a settlement **reuses an existing site there first**.
- **Person:**
  - fields: `id`, `name`, `standing`, `traits`, `links` [{`type`, `to`, `as`, `visibility`}], `faction` {`id`, `sub`} (**public membership only**; a secret membership is a secret fact), `facts` (ids), `premade`, `dead`, `home`, `dies_at`, `fallback`;
  - **`people`, `age` and `voice` {`manner`, `habit`} for everyone you create** (voices-ref.md);
  - `class` if they come from a class.
- **Fact:**
  - fields: `id`, `text`, `visibility`, `kind` (past event / state / trait / plant / move), `when` (**required on past events**), `published_by` (revelations only), `source`.
  - Facts live once in `facts`; entries and cards hold fact ids.
- **Ledger fields on every card (rulebook, *World ledger* and *No blank card*):** `truth_row` ("<Family or signature premise>: <row>" exactly as the truth table names it), `shapes` (the shape slugs of its truth row and twist, from the rulebook's shape list and the truth tables), `plain` (true when the truth row is marked *(confirming)* and the twist is *none*), `reads_context` [{`fact`, `before`}] when the card's text uses a settlement or site context fact.
- **Updates to entries a higher layer made** go in a top-level `registry_updates` list (casting §1: add-only).
- **Thing:** `id`, `name`, `kind`, `owner`, `value`, `location`, `held_by`, `since`. "The only X" is created once.
- **Names:** follow the rulebook's *Name check*:
  - full names unique world-wide;
  - stems checked locally;
  - given names unique in a region among people on story cards;
  - kin share a family name.

  Check against everyone in d-l0.json and in the earlier layers' files you are given.

## Card

Every field in `card-schema.md` §2 that applies:
- **Identity:** `id`, `family`, `premise`, `variant`, `relatedness`, `scope`, `place`, `offered_at`, `giver`, `giver_faction`, `signature`, `clock`, `place_tie`, **`theme`**.
- **Who and what:** `slots`, `opposing_slot`, `fallbacks`, `reasons`, `motive`, `factions`, `truth`, `pressure`, `pressure_level` (chapter and phase-3 cards), `separation`, `twist` {`name`, `group`} (the group from rulebook §9, **always filled when a twist is set**), `open_threads`.
- **Ways:** `ways_pool` [{`text`, `kind`, `source`, `requires`, `names_slot`, `price`, `pays`, `code_forbidden`, **`forced`** (true for always-rolled ways), **`reaches_truth`** (true when it reaches the card's truth)}], `ways_rolled`, `ways_removed`.
- **Endings and knowledge:** `possible_endings` (a string, or {`text`, `state`}), `plants` [{`fact`, `type`, `knower`}], `details`, `depends_on`.
- **Story fields:**
  - main: `chapter`, `position`, `move`, `door`, `cast_present`, `proxies_present`, `landmark`;
  - faction: `faction_story`, `phase`, `odd_job`, `private_side`, `faction_endings`, `guild_meeting`, `shared_town`;
  - side: `attached_to`, `advantage`, `source_move`, `move_fills`, `face`, `aftershock_of`, `checkpoint`, `echo_of`, `linked_pair`.
- **`cast_log`** (test-only field): for every slot, {`how`: reused / created / looked up / cast (the story's own cast), `why`: one short line, such as "linked to the requester (+3), same region (+2)" or "no one fits: created"}. This is how the test measures casting (`casting.md` §6). **Aim for 25–45% reuse** among non-cast slots, and only where a reuse has a real reason.

Rules that bite often (all in the docs; listed so you don't miss them):
- ***No blank card*:** a truth or a real twist on every card. Threat *Raiders*, *Protected beast* and *Encroachment* never roll *none*.
- ***Ways name what's on the card*,** in the roll order, with ***Plant-backed ways roll first***. ***The truth is reachable.***
- ***Premise must fit the place*; *Card text only says what's true then*:** no directions or travel times written freehand. Say "a day's travel" only from the map's `near` and coordinates (1 unit = 1 day).
- **The world ledger** (rulebook, *World ledger*) counts premises, truth rows, **shapes**, twists, levers, ordeal events and way wording across all passes. The planned world size is **N = 220 cards**; the caps follow from it. You get the earlier passes' cards, so count them (`python3 ledger.py <merged.json> 220` prints what's used and left). **If your brief gives you a reserved share** of a nearly-spent cap, stay inside your share.
- ***No blank card*:** turning cards (main turn cards, Ordeal, climax, Return, reveals, phase-3 turn cards, crises) are never plain; side cards may be plain up to about 1 in 4; at least 1 minor in 3 has a twist.
- **The guild ladder (v6b):** faction-story cards carry `rank_gate` (mechanics, *The guild ladder*); **hall jobs** (locations §4) carry `hall_job` {`guild`, `rank`}. A guild story's chapter-2 blow has a `target` (heart / player / guild) and its door (assembly §4, *Guild chapter-2 doors*).
- **No ids or notes** in anything the player reads (pressure, ways, details, place tie, endings).
- **Examples show shape, not wording** (v7: Laz-approved book rules, A11). Never reuse an example's names, objects or openings: not from the family docs' test rolls, the kits' examples or this contract. Take the shape (who wants what, what's in the way) and write your own people, things and first words. (Reusing **registry** people and places by the casting rules is not copying; it's required.)

## Log (markdown, beside your JSON)
1. Rolls and rerolls. Use Python `random.Random(<your seed>)` and record them.
2. A one-line summary per card.
3. **Rule problems:** every place a rule was ambiguous, contradictory, impossible or missing, citing doc and section.
4. **Story notes:** where following the rules produced something dull, repetitive or incoherent.

## Don't read
Don't open earlier test worlds, their reviews, the findings ledger, fix lists or results docs (anything under `test/world/A`, `B`, `C`, or named `review-*`, `findings-*`, `fixlist-*`, `*-results*`). Follow the rules as written; the test is whether *they* produce a good world.

## Self-check before you finish

Merge your file with the earlier layers and run the world checker:

```
cd /home/claude/quests/test/world && QMAP=map-D.json QCANON=canon-D.json python3 merge.py /tmp/claude-0/-home-claude/c3cb521d-c58b-5458-b7ce-de226df0d054/scratchpad/<prefix>-check.json D/d-l0.json <earlier layers…> D/<prefix>.json && QN=220 python3 world_check.py /tmp/claude-0/-home-claude/c3cb521d-c58b-5458-b7ce-de226df0d054/scratchpad/<prefix>-check.json /tmp/claude-0/-home-claude/c3cb521d-c58b-5458-b7ce-de226df0d054/scratchpad/<prefix>-check.md
```

- Fix every issue your cards cause. Ignore coverage of places outside your part.
- Guild agents also run `python3 /home/claude/quests/test/fs-check.py D/<prefix>.json`.
- If you think a complaint is a checker bug, leave it and explain in the log.
- **Write the JSON with a Python build script, and save it early and often.**
