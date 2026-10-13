//! The duel grid (Laz's items design, 0.2 item 10): a weapon against every
//! armour set. For each set: how long an average wielder (Strength 35,
//! skill 30) takes to down someone in it who doesn't fight back, and how
//! often the weapon beats a Common Forgeiron longsword when both wear it.
//! The fights are the full fight rules, each with its own dice. The report
//! (`cargo run --release --bin duel_grid`) and the test quarter's combat
//! yard (`testquarter.rs`, a weapon's row at a time) both use this.

use super::combat::{Battle, Fighter};
use super::geo::V2;
use super::inventory::Gear;
use super::items::{self, item, Kind};
use super::person::Person;
use super::race::Race;
use super::stats::{Attr, Calling, Skill};

/// The armour sets (the design's list).
pub const SETS: &[(&str, &[&str])] = &[
    ("none", &[]),
    ("cloth", &["cloth_shirt", "trousers"]),
    ("leather + padding", &["padded_vest", "hide_coat", "trousers", "hide_leggings", "leather_cap", "boots"]),
    ("mail + coat", &["mail_shirt", "hide_coat", "mail_leggings", "mail_coif", "boots"]),
    ("scale", &["cloth_shirt", "scale_hauberk", "trousers", "scale_greaves", "iron_helm", "boots"]),
    ("plate", &["mail_shirt", "plate_cuirass", "mail_leggings", "plate_greaves", "full_helm", "gauntlets", "armoured_boots"]),
    ("stone", &["padded_vest", "stone_cuirass", "trousers", "scale_greaves", "iron_helm", "boots"]),
];

/// Duels against the longsword for each set, and the fights timed to down.
pub const DUELS: u64 = 8;
pub const TIMED: u64 = 3;
/// Ticks a fight may run before it's called off.
const LONGEST: usize = 6000;

/// Every weapon, by its form key.
pub fn weapons() -> Vec<&'static str> {
    items::FORMS.iter().filter(|f| matches!(item(items::id(f.key)).kind, Kind::Weapon(_))).map(|f| f.key).collect()
}

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

fn ranged(weapon: &str) -> bool {
    item(items::id(weapon)).weapon().is_some_and(|w| w.range > 0.0)
}

/// Seconds for `weapon` to down a still target in `set` (None: not in time).
pub fn time_to_down(weapon: &str, set: &[&str], seed: u64) -> Option<f64> {
    let a = fighter(1, weapon, &[]);
    let d = fighter(2, "club", set);
    let gap = if ranged(weapon) { 12.0 } else { 1.2 };
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
pub fn beats_longsword(weapon: &str, set: &[&str], seed: u64) -> Option<bool> {
    let a = fighter(1, weapon, set);
    let d = fighter(2, "longsword", set);
    let gap = if ranged(weapon) { 12.0 } else { 3.0 };
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

/// One weapon's row: for each set, the median seconds to down (of `TIMED`;
/// None if fewer than two downed it in time) and the wins of `DUELS`
/// against the longsword.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub weapon: &'static str,
    pub secs: Vec<Option<f64>>,
    pub wins: Vec<u64>,
}

pub fn row(weapon: &'static str) -> Row {
    let secs = SETS
        .iter()
        .map(|(_, set)| {
            let mut ts: Vec<f64> = (0..TIMED).filter_map(|s| time_to_down(weapon, set, 100 + s)).collect();
            ts.sort_by(|a, b| a.total_cmp(b));
            (ts.len() >= 2).then(|| ts[ts.len() / 2])
        })
        .collect();
    let wins = SETS.iter().map(|(_, set)| (0..DUELS).filter(|&s| beats_longsword(weapon, set, 500 + s) == Some(true)).count() as u64).collect();
    Row { weapon, secs, wins }
}
