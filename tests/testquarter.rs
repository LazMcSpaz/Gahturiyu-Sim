//! The hidden test quarter (Laz's tutorial island brief, dev-only): found
//! once and kept, walled with a locked gate, dummies that never fight back,
//! and the spawner's calls.

use gahturiyu_sim::sim::{
    animals::Sp,
    duelgrid::{self, SETS},
    factions,
    geo::V2,
    items,
    race::Race,
    world::HOUR,
    worldgen, World,
};

fn opened(seed: u64) -> World {
    let mut w = worldgen::generate(seed);
    assert!(w.test_quarter().is_none(), "nothing of it until the key");
    assert!(w.enter_test_quarter(), "somewhere flat for it");
    w
}

#[test]
fn the_key_finds_it_once_and_brings_the_squad_in() {
    let mut w = opened(1);
    let q = w.test_quarter().unwrap().clone();
    assert!(q.open);
    assert!((q.max.x - q.min.x - 160.0).abs() < 1.0 && (q.max.y - q.min.y - 160.0).abs() < 1.0);
    assert!(q.contains(w.squad.pos), "the squad is inside");
    // Clear of every town.
    assert!(w.settlements.iter().all(|s| !q.contains(s.pos)));
    // Pressed again elsewhere, it's the same quarter.
    w.teleport_squad(w.settlements[0].pos);
    assert!(w.enter_test_quarter());
    assert_eq!(w.test_quarter().unwrap().min, q.min);
}

#[test]
fn walled_round_with_a_gate_that_locks() {
    let mut w = opened(1);
    let q = w.test_quarter().unwrap().clone();
    // The wall stops a walker all round.
    let west = V2::new(q.min.x, (q.min.y + q.max.y) * 0.5);
    assert!(w.blocked_at(west, 0.4).is_some(), "the west wall");
    let north = V2::new((q.min.x + q.max.x) * 0.5, q.min.y);
    assert!(w.blocked_at(north, 0.4).is_some(), "the north wall");
    // Through the open gate, but not the locked one.
    let (inside, outside) = (q.gate().add(V2::new(-4.0, 0.0)), q.gate().add(V2::new(4.0, 0.0)));
    assert!(w.way_clear(inside, outside, 0.4), "open");
    w.lock_test_quarter();
    assert!(!w.way_clear(inside, outside, 0.4), "locked");
    assert!(w.blocked_at(q.gate(), 0.4).is_some());
}

#[test]
fn dummies_take_a_beating_without_fighting_back() {
    let mut w = opened(2);
    let dummies = w.test_quarter().unwrap().dummies.clone();
    assert_eq!(dummies.len(), gahturiyu_sim::sim::testquarter::DUMMIES);
    let d = dummies[0];
    let squad = w.squad.members.clone();
    let hp_before: f32 = squad.iter().map(|&m| w.people[m as usize].wounds.hp_at(&w.people[m as usize].stats, w.time).iter().sum::<f32>()).sum();
    let at = w.person_pos(d);
    w.teleport_squad(at.add(V2::new(2.0, 0.0)));
    assert!(w.attack(&squad, d), "a dummy can be attacked");
    let mut n = 0;
    while w.squad_battle().is_some() && n < 20_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_battle().is_none(), "the fight ends");
    assert!(w.is_down(d) || w.people[d as usize].dead, "the dummy is down");
    let hp_after: f32 = squad.iter().map(|&m| w.people[m as usize].wounds.hp_at(&w.people[m as usize].stats, w.time).iter().sum::<f32>()).sum();
    assert!(hp_after >= hp_before - 0.01, "nobody of the squad was hurt: {hp_before} -> {hp_after}");
    w.dev_reset_dummies();
    assert!(!w.is_down(d) && !w.people[d as usize].dead, "stood up again");
    // They never come looking for a fight on their own.
    w.step(HOUR);
    assert!(w.squad_battle().is_none());
}

#[test]
fn the_spawner_gives_spawns_and_skips() {
    let mut w = opened(1);
    let lead = w.squad.members[0];
    let sword = items::id("longsword");
    let had = w.count_of(lead, "longsword");
    w.dev_give(lead, sword, 2);
    assert_eq!(w.count_of(lead, "longsword"), had + 2);
    let pad = w.test_quarter().unwrap().pad();
    let herd = w.dev_herd(Sp::Shellhen, pad, 3);
    assert_eq!(w.animals.herds[herd as usize].sp, Sp::Shellhen);
    let band = w.dev_band(pad.add(V2::new(-40.0, 0.0)), 3, true);
    assert!(w.is_slaver_camp(band));
    let q = w.dev_person(Race::Horaro, pad);
    assert_eq!(w.people[q as usize].race, Race::Horaro);
    // A faction member: by trade where the faction is one, else seated as its leader.
    let priest = w.dev_faction_member(factions::id("holy_house"), pad);
    assert!(w.factions_of(priest).contains(&factions::id("holy_house")), "{:?}", w.dev_job_of(priest));
    let warden = w.dev_faction_member(factions::id("wardens"), pad);
    assert_eq!(w.factions_of(warden).first(), Some(&factions::id("wardens")));
    let t = w.time;
    w.dev_skip(6.0);
    assert!((w.time - t - 6.0 * HOUR).abs() < 1.0);
}

#[test]
fn a_weapons_row_of_the_duel_grid() {
    let r = duelgrid::row("club");
    assert_eq!(r.secs.len(), SETS.len());
    assert_eq!(r.wins.len(), SETS.len());
    assert!(r.secs[0].is_some(), "a club downs someone in nothing at all");
}

#[test]
fn the_culture_row_is_four_homes_on_the_island_whose_folk_stay_in() {
    let w = worldgen::generate(1);
    let isle = w.island().expect("Salt Isle").clone();
    let town = isle.settlement;
    let homes: Vec<usize> = w.settlements[town as usize].buildings.iter().enumerate().filter(|(_, b)| isle.in_test_quarter(b.pos)).map(|(i, _)| i).collect();
    assert_eq!(homes.len(), 4, "a home for each people");
    let folk: Vec<u32> = w.settlements[town as usize].residents.iter().copied().filter(|&p| w.in_culture_row(p)).collect();
    assert_eq!(folk.len(), 8);
    let peoples: std::collections::BTreeSet<String> = folk.iter().map(|&p| format!("{:?}", w.people[p as usize].race)).collect();
    assert_eq!(peoples.len(), 4, "{peoples:?}");
    for &p in &folk {
        assert!(w.bond_of(p).is_none(), "not one of the compound's debtors");
        let day = World::day_of(w.time);
        let plan = w.day_plan(p, day);
        assert!(plan.segs[..plan.n].iter().all(|s| s.spot == gahturiyu_sim::sim::routine::Spot::Home), "they keep to their homes");
    }
    // The compound's own buildings come first, numbered as the island made them.
    assert!(homes.iter().all(|&i| i >= 6));
}
