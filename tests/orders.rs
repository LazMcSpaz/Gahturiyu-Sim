//! Round 3's small fixes: rest and wake are two orders, and coin can't be held.

use gahturiyu_sim::sim::{items, worldgen};

/// Rest puts the selected down; it never gets anyone up. Wake gets them up.
/// (Round 3: some had woken on their own, and the same toggle sent the awake
/// to bed and left the sleepers asleep.)
#[test]
fn rest_and_wake_are_two_orders() {
    let mut w = worldgen::generate(1);
    let all = w.squad.members.clone();
    w.order_rest(&all[..2]);
    assert!(w.any_resting(&all[..2]));
    // Rest again, with some already resting: everyone ends up resting.
    w.order_rest(&all);
    assert!(all.iter().all(|&m| w.any_resting(&[m])), "rest never wakes anyone");
    w.order_wake(&all[..1]);
    assert!(!w.any_resting(&all[..1]));
    assert!(w.any_resting(&all[1..2]), "wake only the ones asked");
    w.order_wake(&all);
    assert!(!w.any_resting(&all));
}

/// A coin isn't something to hold (round 3: a mistyped entry put one in a
/// hand, and the squad's coin dropped by one).
#[test]
fn coin_cannot_be_equipped() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), 10);
    let before = w.count_of(m, "coin");
    assert!(!w.equip(m, items::id("coin")));
    assert_eq!(w.count_of(m, "coin"), before);
    let held = w.people[m as usize].detail.as_ref().unwrap().gear.weapon_name();
    assert_ne!(held, "Coin");
}
