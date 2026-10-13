//! The duel grid (Laz's items design, 0.2 item 10): every weapon against
//! every armour set. For each pair it reports how long an average wielder
//! (Strength 35, skill 30) takes to down someone in that armour who doesn't
//! fight back, and how often the weapon beats a Common Forgeiron longsword
//! when both wear that armour (8 duels, each with its own dice). A report for
//! tuning, not a test: `cargo run --release --bin duel_grid > docs/duel-grid.md`.
//! The fights themselves are `sim::duelgrid` (the test quarter's combat yard
//! runs a weapon's row of them in play).

use gahturiyu_sim::sim::{
    duelgrid::{row, weapons, DUELS, SETS},
    items::{self, item},
};

fn main() {
    let rows: Vec<_> = weapons().into_iter().map(row).collect();
    println!("# The duel grid\n");
    println!("Every weapon (Common, its usual material) against every armour set (the items design, 0.2 item 10). Made by `cargo run --release --bin duel_grid > docs/duel-grid.md`; a report for tuning, not a test.\n");
    println!("- **Seconds to down**: an average wielder (Strength 35, skill 30) against someone in that armour who doesn't fight back (median of 3; `-` if not within the time).");
    println!("- **Wins of {DUELS}**: the weapon against a Common Forgeiron longsword, both wearing that armour, both fighting.\n");
    let head: Vec<&str> = SETS.iter().map(|s| s.0).collect();
    println!("## Seconds to down\n");
    println!("| Weapon | {} |", head.join(" | "));
    println!("|---|{}", "---|".repeat(SETS.len()));
    for r in &rows {
        let cells: Vec<String> = r.secs.iter().map(|s| s.map(|t| format!("{t:.0}")).unwrap_or_else(|| "-".into())).collect();
        println!("| {} | {} |", item(items::id(r.weapon)).name, cells.join(" | "));
    }
    println!("\n## Wins of {DUELS} against a longsword\n");
    println!("| Weapon | {} | Best in |", head.join(" | "));
    println!("|---|{}---|", "---|".repeat(SETS.len()));
    for r in &rows {
        let best = r.wins.iter().copied().max().unwrap_or(0);
        let best_in: Vec<&str> = SETS.iter().zip(&r.wins).filter(|(_, &n)| n == best && n > 0).map(|(s, _)| s.0).collect();
        let cells: Vec<String> = r.wins.iter().map(|n| n.to_string()).collect();
        println!("| {} | {} | {} |", item(items::id(r.weapon)).name, cells.join(" | "), if best_in.is_empty() { "never".to_string() } else { best_in.join(", ") });
    }
    println!("\nNot yet in these fights: the strength each weapon wants (B1), sneak attacks, tight spaces, and fighting in the dark. Ranged weapons start 12 m off, hand weapons close.");
}
