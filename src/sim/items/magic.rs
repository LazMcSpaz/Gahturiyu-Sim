//! Potions, scrolls, spell notes, rites, and the Ṭaḍoro manuals.

use super::*;

pub(super) static ITEMS: &[ItemDef] = &[
    // --- Potions and scrolls --------------------------------------------------
    potion("healing_draught", "Healing draught", 25.0, &[now(Does::Heal, 25.0, Reach::Caster)]),
    potion("greater_healing", "Greater healing draught", 70.0, &[now(Does::Heal, 55.0, Reach::Caster)]),
    potion("mana_tonic", "Mana tonic", 35.0, &[now(Does::Energy, 40.0, Reach::Caster)]),
    // A brewed potion: what it does is the mix in the bottle (`herbs::Mix`).
    potion("brewed_potion", "Potion", 10.0, &[]),
    // A mix written down (`brewing::Sheet`, in the sheet's piece): read to learn it.
    ItemDef { key: "recipe_sheet", name: "Recipe sheet", slot: Slot::MainHand, kind: Kind::Tool, weight: 0.05, value: 8.0, effects: &[] },
    scroll("scroll_heal", "Scroll of mending", "mend", 40.0),
    scroll("scroll_paralyze", "Scroll of paralysis", "paralyze", 60.0),
    scroll("scroll_fireball", "Scroll of fireball", "fireball", 70.0),
    scroll("scroll_lightning", "Scroll of lightning", "lightning_bolt", 60.0),
    // --- Notes and texts: read to learn ------------------------------------
    notes("notes_paralyze", "Notes on paralysis", "paralyze", 90.0),
    notes("notes_fireball", "Notes on fireball", "fireball", 110.0),
    notes("notes_lightning", "Notes on lightning", "lightning_bolt", 90.0),
    notes("notes_blind", "Notes on blinding", "blind", 70.0),
    notes("notes_barrier", "Notes on barriers", "barrier", 70.0),
    notes("notes_haste", "Notes on haste", "haste", 80.0),
    notes("notes_stone_spikes", "Notes on stone spikes", "stone_spikes", 85.0),
    notes("notes_calm", "Notes on calming", "calm", 70.0),
    notes("notes_fear", "Notes on fear", "fear", 75.0),
    notes("notes_sway", "Notes on persuasion", "sway", 60.0),
    notes("notes_hide", "Notes on hiding", "hide", 80.0),
    notes("notes_decoy", "Notes on decoys", "decoy", 75.0),
    notes("notes_disguise", "Notes on disguise", "disguise", 90.0),
    notes("notes_might", "Notes on might", "might", 80.0),
    notes("notes_toughen", "Notes on toughening", "toughen", 70.0),
    notes("notes_resist", "Notes on resisting the elements", "resist", 65.0),
    notes("notes_dispel", "Notes on dispelling", "dispel", 90.0),
    notes("notes_shatter", "Notes on shattering", "shatter", 95.0),
    notes("notes_unlock", "Notes on unlocking", "unlock", 70.0),
    notes("notes_shrink", "Notes on shrinking", "shrink", 70.0),
    notes("notes_enlarge", "Notes on enlarging", "enlarge", 70.0),
    notes("notes_rust", "Notes on rust", "rust", 65.0),
    notes("notes_spirit_beast", "Notes on spirit beasts", "spirit_beast", 110.0),
    notes("notes_pack_spirit", "Notes on pack spirits", "pack_spirit", 70.0),
    notes("notes_wither", "Notes on withering", "wither", 95.0),
    notes("notes_raise_thrall", "Notes on raising the dead", "raise_thrall", 120.0),
    text("text_restore", "Rite of Restoring", "restore", 220.0),
    text("text_grave_call", "Rite of the Grave Call", "grave_call", 320.0),
    text("text_blight", "Rite of Blight", "blight", 280.0),
    text("text_guardian", "Rite of the Guardian", "guardian", 300.0),
    text("text_swarm", "Rite of the Swarm", "swarm", 260.0),
    text("text_transmute", "Rite of Changing", "transmute", 280.0),
    text("text_sanctuary", "Rite of Sanctuary", "sanctuary", 260.0),
    text("text_sustain", "Rite of Sustaining", "sustain", 240.0),
    text("text_regrow", "Rite of Regrowth", "regrow", 600.0),
    text("text_veil", "Rite of the Veil", "veil", 230.0),
    text("text_dominate", "Rite of Mastery", "dominate", 320.0),
    text("text_firestorm", "Rite of the Firestorm", "firestorm", 260.0),
    text("text_tremor", "Rite of the Shaking Ground", "tremor", 240.0),
    // Ṭaḍoro manuals: read one to take up a craft.
    manual("manual_handcraft", "Manual of leather, cloth and wood", Skill::Handcraft),
    manual("manual_smithing", "Manual of smithing", Skill::Smithing),
    manual("manual_armoring", "Manual of armouring", Skill::Armoring),
    manual("manual_tending", "Manual of stone-tending", Skill::Tending),
    manual("manual_weaving", "Manual of weaving and sealing", Skill::Weaving),
    manual("manual_inscription", "Manual of paper and ink", Skill::Inscription),
    manual("manual_alchemy", "Manual of alchemy", Skill::Alchemy),
    manual("manual_carpentry", "Manual of carpentry", Skill::Carpentry),
    manual("manual_masonry", "Manual of masonry", Skill::Masonry),
];

pub(super) static FORMS: &[Form] = &[
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
];
