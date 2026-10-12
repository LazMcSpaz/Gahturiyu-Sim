//! Items step 1 (docs/items-plan.md): the catalogue split by kind, the
//! cleanup of 0.1, the new slots, and each thing's size in a pack's grid.

use gahturiyu_sim::sim::{
    inventory::Gear,
    items::{self, item, Slot, FORMS, ITEMS, SLOTS},
};

#[test]
fn every_key_is_unique_and_every_form_has_its_item() {
    let mut keys: Vec<&str> = ITEMS.iter().map(|d| d.key).collect();
    keys.sort();
    let n = keys.len();
    keys.dedup();
    assert_eq!(keys.len(), n, "two items share a key");
    for f in FORMS.iter() {
        assert!(items::catalogue_has(f.key), "form {} has no item", f.key);
    }
}

#[test]
fn the_cleanup_is_done() {
    for gone in ["harpoon", "sandstone_lamellar", "kite_shield", "ash_moss", "squid_ink"] {
        assert!(!items::catalogue_has(gone), "{gone} should be gone");
    }
    for (key, name) in [("salt_crystal", "Saltcrust"), ("grain", "Hardgrain"), ("wild_berries", "Moorberry"), ("animal_oil", "Tallow")] {
        assert_eq!(item(items::id(key)).name, name);
    }
    for new in ["plate_cuirass", "round_shield", "cloak", "plain_belt", "bronze_brooch", "bloodleaf", "ink"] {
        assert!(items::catalogue_has(new), "{new} missing");
    }
}

#[test]
fn every_thing_has_a_size_in_the_grid() {
    for id in 0..items::catalogue().defs.len() as items::ItemId {
        let (w, h) = items::footprint(id);
        assert!((1..=6).contains(&w) && (1..=6).contains(&h), "{} is {w}x{h}", item(id).name);
        // Every version of a form is the size of the form.
        assert_eq!((w, h), items::footprint(items::info(id).form));
    }
    assert!(items::footprint(items::id("spear")).1 > items::footprint(items::id("knife")).1);
}

#[test]
fn a_shirt_goes_under_a_coat_and_trousers_under_greaves() {
    let mut g = Gear::default();
    for key in ["cloth_shirt", "hide_coat", "trousers", "scale_greaves", "cloak", "small_pack"] {
        g.wear(items::id(key));
    }
    assert_eq!(g.in_slot(Slot::BodyUnder), Some(items::id("cloth_shirt")));
    assert_eq!(g.in_slot(Slot::BodyOver), Some(items::id("hide_coat")));
    assert_eq!(g.in_slot(Slot::LegsUnder), Some(items::id("trousers")));
    assert_eq!(g.in_slot(Slot::LegsOver), Some(items::id("scale_greaves")));
    // The cloak goes over the pack, each in its own slot.
    assert_eq!(g.in_slot(Slot::Cloak), Some(items::id("cloak")));
    assert_eq!(g.in_slot(Slot::Back), Some(items::id("small_pack")));
    assert!(g.bag.is_empty(), "nothing was pushed off: {:?}", g.bag);
}

#[test]
fn two_rings_one_on_each_hand() {
    let mut g = Gear::default();
    g.wear(items::id("gold_ring"));
    g.wear(items::id("ring_might"));
    assert_eq!(g.in_slot(Slot::Ring), Some(items::id("gold_ring")));
    assert_eq!(g.in_slot(Slot::Ring2), Some(items::id("ring_might")));
    // A third replaces the first, which goes in the pack.
    g.wear(items::id("ring_hearth"));
    assert_eq!(g.in_slot(Slot::Ring), Some(items::id("ring_hearth")));
    assert_eq!(g.in_slot(Slot::Ring2), Some(items::id("ring_might")));
    assert!(g.bag.iter().any(|e| e.0 == items::id("gold_ring")));
}

#[test]
fn a_belt_and_a_brooch_have_their_slots() {
    let mut g = Gear::default();
    g.wear(items::id("plain_belt"));
    g.wear(items::id("bronze_brooch"));
    assert_eq!(g.in_slot(Slot::Belt), Some(items::id("plain_belt")));
    assert_eq!(g.in_slot(Slot::Brooch), Some(items::id("bronze_brooch")));
    assert_eq!(SLOTS.len(), 16);
}

#[test]
fn every_armour_piece_has_its_reason() {
    use gahturiyu_sim::sim::items::wears;
    let w = |k: &str| wears(items::id(k));
    // Helms guard against knockouts, the full helm best, and it costs sight.
    assert!(w("full_helm").ko_guard > w("iron_helm").ko_guard && w("iron_helm").ko_guard > w("leather_cap").ko_guard);
    assert!(w("full_helm").sight > 0.0 && w("iron_helm").sight == 0.0);
    // Furs are warmest; metal chills.
    assert!(w("fur_mantle").warmth > w("cloak").warmth && w("mail_shirt").warmth < 0.0);
    // The travelling robe hides the most from a search; soft shoes are quietest.
    let hide_best = ["travelling_robe", "cloak", "hood", "hide_coat"].iter().max_by(|a, b| w(a).hides.total_cmp(&w(b).hides)).unwrap();
    assert_eq!(*hide_best, "travelling_robe");
    assert!(w("soft_shoes").noise < w("boots").noise && w("armoured_boots").noise > w("boots").noise);
    // Padding against blunt, waterproofing for the wet.
    assert!(w("padded_jacket").padding > w("hide_coat").padding);
    assert!(w("oilskin_cape").dry > 0.8 && w("fishskin_boots").dry > w("boots").dry);
    // Plate and stone slow you; nothing else listed is free of a cost or a gift.
    assert!(w("stone_cuirass").slow > w("plate_cuirass").slow && w("plate_cuirass").slow > 0.0);
    // Every slot of the body has something new to wear.
    for k in ["padded_cap", "mail_coif", "full_helm", "padded_vest", "mail_shirt", "fishskin_undercoat", "fur_jerkin", "brigandine", "stone_cuirass", "travelling_robe", "bracers", "gauntlets", "mail_leggings", "plate_greaves", "fishskin_boots", "fur_boots", "armoured_boots", "soft_shoes", "hood", "oilskin_cape", "fur_mantle", "tower_shield", "bandolier_belt", "sash", "fine_belt"] {
        assert!(items::catalogue_has(k), "{k} missing");
    }
}
