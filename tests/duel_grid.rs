//! The duel grid (the items design, 0.2 item 10): every weapon against a
//! reference fighter in every armour set, outdoors and indoors, open and
//! from behind. It writes `docs/duel-grid.md` (win rates, and the weapon
//! that does best in each case) and lists any weapon that is never the best
//! choice anywhere. Slow, so it's run by hand:
//!
//! `cargo test --release --test duel_grid -- --ignored --nocapture`
//!
//! Everyone is average (Strength, Agility and Toughness 35, 40 in every
//! weapon skill, Dodge and Block 30). The reference holds a Common
//! Forgeiron longsword. Both fight for themselves (the fight AI). A thrower
//! carries a full stack and a short sword to draw once it's thrown.

use gahturiyu_sim::sim::{
    buildings::door_of,
    combat::{Battle, Fighter},
    geo::V2,
    inventory::Gear,
    items::{self, Kind},
    person::Person,
    race::Race,
    stats::{Attr, Calling, Skill},
    worldgen,
};

const ARMOURS: [(&str, &[&str]); 4] = [
    ("none", &[]),
    ("light", &["padded_vest", "leather_cap"]),
    ("mail", &["mail_shirt", "hide_coat", "iron_helm"]),
    ("plate", &["mail_shirt", "plate_cuirass", "plate_greaves", "full_helm"]),
];
const DUELS: u64 = 16;

fn average(id: u32, seed: u64, hand: items::ItemId, armour: &[&str]) -> Person {
    let mut p = Person::summary(id, seed, Race::Roduro, None);
    p.specialize(Calling::Common, &[], 0.0);
    for a in [Attr::Strength, Attr::Agility, Attr::Toughness, Attr::Intellect, Attr::Willpower] {
        p.stats.set_attr(a, 35.0);
    }
    for k in [Skill::Blade, Skill::Blunt, Skill::Spear, Skill::Marksman, Skill::Unarmed] {
        p.stats.set_skill(k, 40.0);
    }
    p.stats.set_skill(Skill::Dodge, 30.0);
    p.stats.set_skill(Skill::Block, 30.0);
    p.ensure_detail();
    let d = p.detail.as_mut().unwrap();
    let mut g = Gear::default();
    g.add(hand, 1);
    let _ = g.equip(hand);
    if let Some(ammo) = items::item(hand).weapon().and_then(|w| w.ammo) {
        g.add(items::id(ammo), 30);
    }
    // A thrower carries a full stack, and a short sword to draw after.
    if items::item(hand).weapon().is_some_and(|w| w.thrown) {
        g.add(hand, gahturiyu_sim::sim::inventory::stack_of(hand).max(1));
        g.add(items::id("short_sword"), 1);
    }
    for k in armour {
        g.add(items::id(k), 1);
        let _ = g.equip(items::id(k));
    }
    d.gear = g;
    p
}

/// A's share of wins against the reference: (wins, hits landed on B per
/// fight it went down in).
fn duel(w: &gahturiyu_sim::sim::World, at: Option<V2>, weapon: items::ItemId, armour: &[&str], sneak: bool) -> f32 {
    let mut wins = 0.0;
    let longsword = items::variant(items::id("longsword"), gahturiyu_sim::sim::materials::Material::Forgeiron, gahturiyu_sim::sim::materials::Material::None, gahturiyu_sim::sim::materials::Grade::Common).unwrap();
    for seed in 0..DUELS {
        let a = average(1, 100 + seed, weapon, &[]);
        let b = average(2, 900 + seed, longsword, armour);
        let origin = at.unwrap_or(V2::new(0.0, 0.0));
        let wd = items::item(weapon).weapon().copied().unwrap();
        // (Bows from 14 m, throwers from 9: what they're for; the rest close.)
        let gap = if wd.range > 0.0 { 14.0 } else if wd.thrown { 9.0 } else if sneak { 1.0 } else { 3.0 };
        let fa = Fighter::from_person(&a, 0, origin, 0.0);
        let mut fb = Fighter::from_person(&b, 1, origin.add(V2::new(gap, 0.0)), 0.0);
        if sneak {
            fb.aware_at = 2.0;
        }
        let mut bt = Battle::new(0, 4000 + seed * 31 + weapon as u64, 0.0, vec![fa, fb], vec!["A".into(), "B".into()]);
        if at.is_some() {
            bt.around = w.around_fight(&bt);
        }
        while !bt.over {
            bt.tick();
        }
        let (fa, fb) = (&bt.fighters[0], &bt.fighters[1]);
        let a_up = fa.active() && !fa.fleeing;
        let b_up = fb.active() && !fb.fleeing;
        wins += match (a_up, b_up) {
            (true, false) => 1.0,
            (false, true) => 0.0,
            _ => 0.5,
        };
    }
    wins / DUELS as f32
}

#[test]
#[ignore = "slow: the duel grid report, run by hand"]
fn the_duel_grid() {
    let w = worldgen::generate(1);
    // Indoors: the floor of a big enough building.
    let d = (0..w.settlements[0].buildings.len() as u16).find_map(|i| door_of(&w.settlements[0], i).filter(|d| d.half.x.min(d.half.y) >= 3.5)).expect("a big building");
    let inside = w.free_spot(d.centre);
    let weapons: Vec<items::ItemId> = items::ITEMS.iter().filter(|i| matches!(i.kind, Kind::Weapon(_))).map(|i| items::id(i.key)).collect();
    let mut cases: Vec<(String, Option<V2>, &[&str], bool)> = Vec::new();
    for (name, armour) in ARMOURS {
        for (place, at) in [("out", None), ("in", Some(inside))] {
            for sneak in [false, true] {
                cases.push((format!("{name} {place}{}", if sneak { " sneak" } else { "" }), at, armour, sneak));
            }
        }
    }
    let mut table: Vec<(items::ItemId, Vec<f32>)> = Vec::new();
    for &wpn in &weapons {
        let row: Vec<f32> = cases.iter().map(|c| duel(&w, c.1, wpn, c.2, c.3)).collect();
        eprintln!("{:<18} {}", items::item(wpn).name, row.iter().map(|r| format!("{:>4.0}", r * 100.0)).collect::<String>());
        table.push((wpn, row));
    }
    // The best in each case (ties count for all).
    let mut best_somewhere: Vec<items::ItemId> = Vec::new();
    let mut bests: Vec<String> = Vec::new();
    for (k, c) in cases.iter().enumerate() {
        let top = table.iter().map(|r| r.1[k]).fold(0.0, f32::max);
        let who: Vec<items::ItemId> = table.iter().filter(|r| r.1[k] >= top - 1e-4).map(|r| r.0).collect();
        bests.push(format!("| {} | {:.0}% | {} |", c.0, top * 100.0, who.iter().map(|&i| items::item(i).name).collect::<Vec<_>>().join(", ")));
        best_somewhere.extend(who);
    }
    let never: Vec<&str> = weapons.iter().filter(|w| !best_somewhere.contains(w)).map(|&w| items::item(w).name).collect();
    let mut md = String::new();
    md += "# The duel grid\n\nWritten by `cargo test --release --test duel_grid -- --ignored` (items design, 0.2 item 10).\n";
    md += &format!("Each cell: the share of {DUELS} duels the weapon won against an average fighter with a Common Forgeiron longsword wearing that armour (draws count half). Both fighters average (Strength, Agility, Toughness 35; 40 in the weapon skills). \"in\": on the floor of a town building; \"sneak\": the reference doesn't see the first blow coming. Bows start 14 m off, thrown weapons 9 m, the rest 3 m (1 m from behind).\n\n");
    md += &format!("| Weapon | {} |\n|---|{}\n", cases.iter().map(|c| c.0.clone()).collect::<Vec<_>>().join(" | "), "---|".repeat(cases.len()));
    for (wpn, row) in &table {
        md += &format!("| {} | {} |\n", items::item(*wpn).name, row.iter().map(|r| format!("{:.0}", r * 100.0)).collect::<Vec<_>>().join(" | "));
    }
    md += "\n## The best in each case\n\n| Case | Win rate | Weapon |\n|---|---|---|\n";
    md += &bests.join("\n");
    md += "\n\n## Never the best choice\n\n";
    md += &if never.is_empty() { "None: every weapon is the best somewhere.\n".to_string() } else { format!("{}\n", never.join(", ")) };
    std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/duel-grid.md"), md).unwrap();
    eprintln!("never best: {never:?}");
}
