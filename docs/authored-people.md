# Authored people: the roster files

Laz decides who lives in a town (`claude/authored-world.md` in the Project;
the brief in `docs/npc-tool-prompt.md`). The engine stays as it is; a town
with a roster file gets exactly the people in it, and nobody seeded. A town
with no roster file works as today.

Two tools write rosters: the **People mode of the Map Marks page** (Laz, on
his phone or iPad: authored characters and crowd rows, exported as text he
hands to Claude) and the **crowd filler** in the game (generates the crowd
from the rows with keyed randomness and stores it person by person). This
file is the format both agree on.

## Where

- `assets/world/towns.ron`: every mark has a stable `key` (lower-case, from
  its label: `imperial_capital`), used everywhere a town is named in data.
- `assets/world/people/<key>.ron`: the town's roster. `wanderers.ron` holds
  the people with no place.
- `data/lines/people/<character key>.txt`: a character's own lines (optional).

## The roster (`assets/world/people/<key>.ron`)

```ron
(
    town: "imperial_capital",
    ruler: Some("deshu_grey"),          // an authored character's key, or None
    characters: [
        (
            key: "deshu_grey",            // stable, unique across every roster
            name: "Ḍeshu of the Grey House",
            people: Roduro,               // Roduro | Qotiro | Horaro | Tadoro
            sex: Man,                     // Man | Woman
            profession: Official,         // a `jobs::Job`
            faction: Some("free_lords"),  // a faction key, or None
            sub_faction: None,
            wealth: Rich,                 // Destitute | Poor | Comfortable | Rich | VeryRich
            traits: (                     // the sim's traits, 0..1 each; omitted ones are rolled from the people
                temperament: Some(0.3), honour: Some(0.8), sociability: None, wanderlust: None, ..
            ),
            home: Some(12),               // a building index in the town's layout, or None
            workplace: Some(2),           // a workplace index (`society.towns[t].places`), or None
            relations: [ (to: "hoqa_wall", kind: Rival) ],   // Spouse | Child | Parent | Sibling | Rival | Friend | Lover | Debtor | Creditor
            background: "Lost his seat at court; runs the island as if it were one.",
            importance: "The first lord the player meets; sets the tone of the Free Lords.",
            schedule: [ (from: 19.0, to: 23.0, at: Place(4)) ],   // optional overrides of the day plan, hours and a spot
            appearance: (style: "court", colours: ["grey", "black"], marks: ["scar over the eye"]),
            lines: Some("deshu_grey"),    // data/lines/people/deshu_grey.txt
        ),
    ],
    crowd_rows: [
        (role: Merchant, count: 10, mix: None),            // mix: None = the town's founders' mix
        (role: Farmer, count: 30, mix: Some((roduro: 0.7, qotiro: 0.1, horaro: 0.1, tadoro: 0.1))),
    ],
    crowd: [                                               // written by the game's filler, keyed by (town key, row, n); edited by hand after if wanted
        (key: "imperial_capital.0.3", name: "...", people: Roduro, sex: Woman, profession: Merchant, household: 7, home: Some(31),
         traits: (..), wealth: Comfortable, appearance: 2, detail: "keeps bees on the roof"),
    ],
)
```

Fields the page can't know (building and workplace indices) are left `None`
by the page and filled in the game's editor or by hand; a character with no
home is housed like a seeded person.

## At world creation

`worldgen::place_people`: for each town, if `assets/world/people/<key>.ron`
exists, its characters and its stored crowd become the town's residents
(`Person` with the stored fields in place of the rolled ones; traits given
override the roll, the rest roll from the people as today); no seeded
residents. Wanderers from `wanderers.ron`. Every fact (profession, faction,
wealth, the personal detail, "authored") is available to the dialogue
system's conditions.
