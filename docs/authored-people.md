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
                wanderlust: None, boldness: Some(0.3), sociability: None, patience: None,
                honour: Some(0.8), asks: None,   // honour: open ways (high) or covert (low); asks: how readily they ask an outsider for help
            ),
            home: Some(12),               // a building index in the town's layout, or None
            workplace: Some(2),           // a workplace index (`society.towns[t].places`), or None
            relations: [ (to: "hoqa_wall", kind: Rival) ],   // Spouse | Child | Parent | Sibling | Rival | Friend | Lover | Debtor | Creditor
            background: "Lost his seat at court; runs the island as if it were one.",
            importance: "The first lord the player meets; sets the tone of the Free Lords.",
            schedule: [ (from: 19.0, to: 23.0, at: Place(4)) ],   // optional overrides of the day plan, hours and a spot: Home | Work | Hearth | Place(workplace) | Building(index)
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

Every field but `key`, `name`, `people`, `sex` and `profession` may be left
out (a character's `wealth` is then Comfortable, its traits rolled). The
whole file reads back as written (`Roster::to_ron`, pretty, one field a line).

Fields the page can't know (building and workplace indices) are left `None`
by the page and filled in the game's editor or by hand; a character with no
home is housed like a seeded person.

The town's key is the mark's `key` in `towns.ron` (made from the label when
left out: `Imperial capital` → `imperial_capital`).

## At world creation

In `worldgen` (stage 1, built): for each town, if `assets/world/people/<key>.ron`
exists, its characters and its stored crowd become the town's residents,
and nobody seeded: anyone the seed would have sent there goes to another
town (a second pick, so nobody else's moves). Each is an ordinary `Person`:

- people and sex as given (their seed is one whose reading is that sex,
  as names and offices read it), name as given, the traits given (the rest
  rolled from the people), their stats worked out from those;
- home: the building given, if the town's layout has it, else housed like
  anyone; the layout is still made from who lives there, so adding people
  can move building numbers;
- once the town's life is founded: profession as given; workplace as
  given if the town has it, else the first of that trade's; honour and
  asks over the rolled habits; the household's purse at the richest of its
  authored members' `wealth` (Destitute none, Poor 2 days' keep,
  Comfortable 7, Rich 30, VeryRich 120).

Wanderers from `wanderers.ron` start on the road like any wanderer. The
whole entry is kept on the person (`Person::authored`, saved), so every fact
(profession, faction, wealth, background, the personal detail, relations,
schedule, looks, lines, and "authored" itself) is there for the dialogue's
conditions (stage 4).

Not yet: households from the crowd's own numbers (the town's customs still
group people), the schedule and relations acting on anything, the ruler
taking the seat, Salt Isle (it isn't a mark; its people are made in code).
