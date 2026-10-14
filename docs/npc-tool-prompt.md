# Prompt: the NPC tool and the crowd filler (authored people)

You're working on Gahturiyu Sim (`LazMcSpaz/Gahturiyu-Sim`), a Kenshi-style squad sandbox in Rust and Bevy 0.19. Read `README.md` and `CLAUDE.md` first: they hold the rules this work must keep (keyed randomness, nothing depends on bands or step size, everything in `World` is saved, bump `save::FORMAT` and the fingerprint in `tests/save.rs` when the save changes). Build on the latest atlas branch (`world/atlas` or whatever main has become); it has `assets/world/towns.ron`, `atlas.rs` and `leaders.rs`.

## Why
Laz (the designer) feels the world is too simulated: towns are filled with people rolled from the seed, and they feel hollow. The engine stays as it is (routines, economy, condition, combat, weather, animals). What changes is **who decides who lives in a town: Laz does**, using two tools you'll build:

1. **The NPC tool:** a dev editor to make an important character by clicking options.
2. **The crowd filler:** set counts per role for a town ("10 merchants, 30 peasants, 20 guards") and generate them, with small variations so they aren't clones.

The design is in `claude/authored-world.md` (copied below in short). Laz decided these points; don't change them without asking.

## Ground rules
- **Don't author any town or place yourself.** Laz authors locations one at a time with these tools. Your test fixtures can use a throwaway town, but ship no authored content.
- **Don't add new procedurally generated locations** (no new seeded ruins, camps, towns). Existing ones stay as they are for now.
- **A town with no roster file works exactly as today** (seeded residents). A town with a roster file uses the roster instead. So nothing breaks before Laz has written anything.
- Plain language in everything Laz reads (panel labels, docs, your reports). He isn't a programmer; explain terms.

## 1. What a roster is (data)
One file per town: `assets/world/people/<town key>.ron`, written by the tools and readable by hand. The town key is the town's entry in `towns.ron` (add a stable `key` field there if it hasn't one).

A roster holds:
- **Authored characters**, each with:
  - `key` (stable id), `name` (generated or typed), `people` (Roduro, Qotiro, Horaro, Ṭaḍoro), `sex`;
  - `profession` (from the jobs list), `faction` and sub-faction (optional);
  - `wealth`: destitute, poor, comfortable, rich, very rich;
  - `traits`: the trait set the sim already uses (temperament, honour and the rest), set by picking options that map to values;
  - `home` and `workplace` (building indices in the town layout);
  - `relations`: links to other characters in any roster (spouse, child, parent, sibling, rival, friend, lover, debtor, creditor);
  - `background`: short free text;
  - `importance`: free text: why they matter, what they connect to;
  - `schedule`: optional overrides of the routine (e.g. "at the smoke hall every night from dusk");
  - `appearance`: clothing style, colours, a few look options;
  - `lines`: optional key to a hand-written lines file (`data/lines/people/<key>.txt`).
- **Crowd groups:** each is a role, a count, and optionally a mix of peoples (default: the town's founders' mix). Plus **the generated crowd itself**, stored person by person once generated (name, people, sex, profession, household, home, traits, wealth step, appearance variant, one personal detail). **Stored, not re-rolled:** the same town always has the same people, and Laz can edit any one of them.
- **Town-level fields** Laz may set later (ruler, mood, shop stock); leave room, but you don't need to build them now beyond a ruler link to an authored character.

**At world creation:** a town with a roster gets exactly its authored characters and its stored crowd, and nobody seeded. Everyone is still an ordinary `Person` to the sim: they eat, work, sleep, trade, gossip and fight through the existing systems. Their stored fields replace what the seed would have rolled.

## 2. The NPC tool (authored characters)
A dev editor in the game window, opened with a free F-key (check the README's key table), in the same style as the F10 land editor (egui panel, Esc to leave).

- **Pick a town** (from `towns.ron`), then **list its characters** and **New character**.
- **The character form:** every field above as clickable options (drop-downs, toggles, sliders where a value fits), plus text boxes for background and importance.
- **Name: a Generate button.** Uses the naming module (`sim::names`): a given name by people and sex, plus the people's byname shape (Roduro house, Qotiro rank or deed, Horaro mother's line or sea-place, Ṭaḍoro teacher or road; profession can feed the byname). **Reroll** until it fits; or type a name.
- **Home and workplace:** pick a building by clicking it in the town view (or from a list of the town's buildings if clicking is a big job; say which you did).
- **Relations:** pick another character from a list (any town), and the link type.
- **Save** writes the roster file. **Undo** for the last change if it's cheap; otherwise say so.
- **Persuasion leanings are not a field.** Laz: "the things that persuade them are derived from every other trait." Show them read-only in the form (moved, neutral, cold, offended for each of the seven appeals in `claude/persuasion.md`), worked out from people, faction, profession, wealth, traits and background, so Laz sees the result of his choices. If the persuasion system isn't built yet, build only the derivation function and its display; don't build the persuasion gameplay.

## 3. The crowd filler
In the same tool, a **Crowd** tab for the town:
- **Role counts:** a list of rows, `role` and `count` (merchant, peasant, labourer, guard, porter, priest or priestess, fisher, crafter, servant, beggar, and the town's other jobs), and a peoples mix (sliders, defaulting to the founders' mix).
- **Generate** makes the people: name, people, sex, profession, household (group them into households of plausible size), home (fill the town's homes), and **small variation:** one or two traits shifted from the people's average, a wealth step up or down from the role's usual, a clothing variant (`claude/clothing.md` lists variants per people), and **one personal detail** from a data list (a habit, a worry, a hobby, a small loss) that their dialogue can mention.
- **Regenerate** a row or the whole crowd (asks first if it would replace edited people).
- **Promote:** open any crowd member in the character form; they become an authored character, keeping their name and fields.
- All randomness is keyed (`Rng::from_keys`) by the town key and the row, so generating twice with the same inputs gives the same people.

## 4. How they talk
- **Authored characters:** every field becomes a fact the dialogue system can match lines on (profession, faction, wealth, traits, the personal detail, and "authored" itself). Their own lines file, if any, outranks generated lines. The importance text is a note for Laz, not shown to the player.
- **The crowd talks less** (Laz: "keep, but quieter"): a crowd member draws from a small pool (a greeting, a remark about their work or their personal detail, an occasional rumour, a farewell). No deep conversations, but they can still be robbed, hired, recruited and gossip.
- Check the faction-line issue noted in Laz's inventory: "a faction's line outranks a plain one, so in a held town everyone greets alike". Fix it so a person's own facts (profession, personal detail) can win over a blanket faction greeting.

## 5. Saving and tests
- Everything that changes `World` is saved; bump `save::FORMAT` and the fingerprint in `tests/save.rs`.
- **Tests:**
  - a town with no roster file gives the same world as before (the existing tests still pass);
  - a town with a roster has exactly its authored and crowd people, with their stored fields;
  - generating a crowd twice with the same inputs gives identical people;
  - a promoted crowd member keeps their name and fields;
  - the persuasion derivation is stable and changes when a trait changes;
  - a roster round-trips through its file unchanged.

## Build in stages (stop after each and report to Laz in plain language, with a screenshot of the tool where there is one)
1. **The roster data and loading:** the file format, reading it at world creation, a town with a roster using it, no-roster towns unchanged. A tiny hand-written test roster in the tests only.
2. **The NPC tool, characters:** the panel, the form, name generation, home and workplace, relations, save. Read-only persuasion leanings.
3. **The crowd filler:** role counts, generation with variation, households and homes, regenerate, promote.
4. **Dialogue:** facts from the fields, per-character lines files, the quieter crowd pool, the faction-greeting fix.
5. **Docs:** a README section on using the tool (written for Laz), and the controls table.

## What not to do
- Don't write any town's roster, characters or crowd: that's Laz's job, one location at a time.
- Don't change the routines, economy, combat or condition systems beyond what loading authored people needs.
- Don't generate new locations.
