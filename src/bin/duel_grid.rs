//! The duel grid (Laz's items design, 0.2 item 10): every weapon against
//! every armour set. For each pair it reports how long an average wielder
//! (Strength 35, skill 30) takes to down someone in that armour who doesn't
//! fight back, and how often the weapon beats a Common Forgeiron longsword
//! when both wear that armour (8 duels, each with its own dice). A report for
//! tuning, not a test: `cargo run --release --bin duel_grid > docs/duel-grid.md`.

use gahturiyu_sim::sim::{
    combat::{Battle, Fighter},
    geo::V2,
    inventory::Gear,
    items::{self, item, Kind},
    person::Person,
    race::Race,
    stats::{Attr, Calling, Skill},
};

/// The armour sets (the design's list).
const SETS: &[(&str, &[&str])] = &[
    ("none", &[]),
    ("cloth", &["cloth_shirt", "trousers"]),
    ("leather + padding", &["padded_vest", "hide_coat", "trousers", "hide_leggings", "leather_cap", "boots"]),
    ("mail + coat", &["mail_shirt", "hide_coat", "mail_leggings", "mail_coif", "boots"]),
    ("scale", &["cloth_shirt", "scale_hauberk", "trousers", "scale_greaves", "iron_helm", "boots"]),
    ("plate", &["mail_shirt", "plate_cuirass", "mail_leggings", "plate_greaves", "full_helm", "gauntlets", "armoured_boots"]),
    ("stone", &["padded_vest", "stone_cuirass", "trousers", "scale_greaves", "iron_helm", "boots"]),
];

const DUELS: u64 = 8;
/// Ticks a fight may run before it's called off.
const LONGEST: usize = 6000;

/// An average fighter (Strength 35, every weapon skill 30) with this weapon
/// and armour.
fn fighter(id: u32, weapon: &str, set: &[&str]) -> Person {
    let mut p = Person::summary(id, 9000 + id as u64 * 13, Race::Roduro, None);
    p.specialize(Calling::Warrior, &[], 0.0);
    p.ensure_detail();
    p.stats.set_attr(Attr::Strength, 35.0);
    p.stats.set_attr(Attr::Agility, 35.0);
    for s in [Skill::Blade, Skill::Blunt, Skill::Spear, Skill::Marksman, Skill::Unarmed, Skill::Block, Skill::Dodge] {
        p.stats.set_skill(s, 30.0);
    }
    let d = p.detail.as_mut().unwrap();
    d.gear = Gear::default();
    for k in set {
        d.gear.wear(items::id(k));
    }
    let w = items::id(weapon);
    d.gear.wear(w);
    // Something to shoot or throw.
    if let Some(ammo) = item(w).weapon().and_then(|x| x.ammo) {
        d.gear.add(items::id(ammo), 40);
    }
    if item(w).weapon().is_some_and(|x| x.thrown) {
        d.gear.add(w, 5);
    }
    p
}

/// Seconds for `weapon` to down a still target in `set` (None: not in time).
fn time_to_down(weapon: &str, set: &[&str], seed: u64) -> Option<f64> {
    let a = fighter(1, weapon, &[]);
    let d = fighter(2, "club", set);
    let ranged = item(items::id(weapon)).weapon().is_some_and(|w| w.range > 0.0);
    let gap = if ranged { 12.0 } else { 1.2 };
    let mut fs = vec![Fighter::from_person(&a, 0, V2::new(0.0, 0.0), 0.0), Fighter::from_person(&d, 1, V2::new(gap, 0.0), 0.0)];
    fs[1].think_at = f64::INFINITY;
    fs[0].target = Some(1);
    let mut b = Battle::new(0, seed, 0.0, fs, vec!["A".into(), "D".into()]);
    for _ in 0..LONGEST {
        b.tick();
        let t = &b.fighters[1];
        if t.ko || t.dead {
            return Some(b.time);
        }
    }
    None
}

/// Does `weapon` beat a longsword, both in `set`?
fn beats_longsword(weapon: &str, set: &[&str], seed: u64) -> Option<bool> {
    let a = fighter(1, weapon, set);
    let d = fighter(2, "longsword", set);
    let ranged = item(items::id(weapon)).weapon().is_some_and(|w| w.range > 0.0);
    let gap = if ranged { 12.0 } else { 3.0 };
    let fs = vec![Fighter::from_person(&a, 0, V2::new(0.0, 0.0), 0.0), Fighter::from_person(&d, 1, V2::new(gap, 0.0), 0.0)];
    let mut b = Battle::new(0, seed, 0.0, fs, vec!["A".into(), "D".into()]);
    for _ in 0..LONGEST {
        b.tick();
        let (x, y) = (&b.fighters[0], &b.fighters[1]);
        if x.ko || x.dead || x.fled {
            return Some(false);
        }
        if y.ko || y.dead || y.fled {
            return Some(true);
        }
    }
    None
}

fn main() {
    let weapons: Vec<&str> = items::FORMS.iter().filter(|f| matches!(item(items::id(f.key)).kind, Kind::Weapon(_))).map(|f| f.key).collect();
    println!("# The duel grid\n");
    println!("Every weapon (Common, its usual material) against every armour set (the items design, 0.2 item 10). Made by `cargo run --release --bin duel_grid > docs/duel-grid.md`; a report for tuning, not a test.\n");
    println!("- **Seconds to down**: an average wielder (Strength 35, skill 30) against someone in that armour who doesn't fight back (median of 3; `-` if not within the time).");
    println!("- **Wins of {DUELS}**: the weapon against a Common Forgeiron longsword, both wearing that armour, both fighting.\n");
    let head: Vec<&str> = SETS.iter().map(|s| s.0).collect();
    println!("## Seconds to down\n");
    println!("| Weapon | {} |", head.join(" | "));
    println!("|---|{}", "---|".repeat(SETS.len()));
    for w in &weapons {
        let cells: Vec<String> = SETS
            .iter()
            .map(|(_, set)| {
                let mut ts: Vec<f64> = (0..3).filter_map(|s| time_to_down(w, set, 100 + s)).collect();
                ts.sort_by(|a, b| a.total_cmp(b));
                if ts.len() < 2 { "-".into() } else { format!("{:.0}", ts[ts.len() / 2]) }
            })
            .collect();
        println!("| {} | {} |", item(items::id(w)).name, cells.join(" | "));
    }
    println!("\n## Wins of {DUELS} against a longsword\n");
    println!("| Weapon | {} | Best in |", head.join(" | "));
    println!("|---|{}---|", "---|".repeat(SETS.len()));
    for w in &weapons {
        let wins: Vec<u64> = SETS.iter().map(|(_, set)| (0..DUELS).filter(|&s| beats_longsword(w, set, 500 + s) == Some(true)).count() as u64).collect();
        let best = wins.iter().copied().max().unwrap_or(0);
        let best_in: Vec<&str> = SETS.iter().zip(&wins).filter(|(_, &n)| n == best && n > 0).map(|(s, _)| s.0).collect();
        let cells: Vec<String> = wins.iter().map(|n| n.to_string()).collect();
        println!("| {} | {} | {} |", item(items::id(w)).name, cells.join(" | "), if best_in.is_empty() { "never".to_string() } else { best_in.join(", ") });
    }
    println!("\nNot yet in these fights: the strength each weapon wants (B1), sneak attacks, tight spaces, and fighting in the dark. Ranged weapons start 12 m off, hand weapons close.");
}
