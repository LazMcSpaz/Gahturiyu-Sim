//! Town alchemists brew for the shelf (Part 4, rule 11): cheap bottles may
//! carry a side effect, Fine and better never do, and a bottle bought is
//! the very mix that was on the shelf.

use gahturiyu_sim::sim::{
    apothecary::KNOWN_MIXES,
    economy::Table,
    items::{self, herbs::{Kit, Mix}, ItemId},
    jobs::{Good, Job},
    making::Shelved,
    materials::{Grade, Piece},
    person::PersonId,
    world::{DAY, HOUR},
    worldgen, World,
};

fn until_hour(w: &mut World, hour: f64) {
    let now = w.time.rem_euclid(DAY) / HOUR;
    let wait = (hour - now).rem_euclid(24.0);
    for _ in 0..(wait * 2.0).round() as usize {
        w.step(HOUR / 2.0);
    }
}

fn bottle() -> ItemId {
    items::id("brewed_potion")
}

/// Every bottle on every town's shelf.
fn shelved(w: &World) -> Vec<Mix> {
    w.society.towns.iter().flat_map(|t| t.shelf.iter()).filter(|s| s.item == bottle()).filter_map(|s| s.piece.and_then(|p| p.mix)).collect()
}

/// A world whose stores are stocked with what alchemists brew from, run
/// for some days.
fn brewed_world(seed: u64, days: f64) -> World {
    let mut w = worldgen::generate(seed);
    for tl in &mut w.society.towns {
        for g in [Good::Herbs, Good::Game, Good::Wares] {
            tl.stock[g.index()].base += 60.0;
        }
    }
    let end = w.time + days * DAY;
    while w.time < end {
        w.step(HOUR);
    }
    w
}

#[test]
fn the_known_mixes_are_real_and_some_are_cheap() {
    let plain = Kit { mortar: Some(Grade::Common), ..Kit::default() };
    let (mut clean, mut cheap) = (0, 0);
    for m in KNOWN_MIXES {
        for k in m.ings {
            assert!(items::ingredient(items::id(k)).is_some(), "{k} is an ingredient");
            assert!(gahturiyu_sim::sim::jobs::good_of(k).is_some(), "{k} is something a store holds");
        }
        let mix = m.brewed(40.0, plain);
        assert!(!mix.is_empty(), "{} makes something", m.key);
        if mix.harms() { cheap += 1 } else { clean += 1 }
    }
    assert!(clean >= 6 && cheap >= 3, "{clean} clean, {cheap} cheap");
    // Healing is among the clean ones.
    let heal = KNOWN_MIXES.iter().find(|m| m.key == "healing").unwrap().brewed(40.0, plain);
    assert!(!heal.harms() && heal.name().contains("heal"), "{}", heal.name());
}

#[test]
fn alchemists_fill_the_shelves_and_fine_bottles_never_harm() {
    let w = brewed_world(1, 4.0);
    let alchemists = w.society.lives.iter().filter(|l| l.job == Job::Alchemist).count();
    assert!(alchemists > 0, "the world has alchemists");
    let bottles = shelved(&w);
    assert!(bottles.len() >= 4, "{} bottles on the shelves after four days", bottles.len());
    let names: std::collections::BTreeSet<String> = bottles.iter().map(|m| m.name()).collect();
    assert!(names.len() >= 2, "more than one kind: {names:?}");
    for m in &bottles {
        assert!(!(m.harms() && m.grade() >= Grade::Fine), "a Fine bottle with a side effect: {} {:?}", m.name(), m.grade());
    }
    eprintln!("{} bottles: {names:?}; {} carry harm", bottles.len(), bottles.iter().filter(|m| m.harms()).count());
}

/// A merchant at their stall who sells off the town's shelf, with two
/// different bottles on it.
fn two_bottles(seed: u64) -> (World, PersonId, Mix, Mix) {
    let mut w = worldgen::generate(seed);
    until_hour(&mut w, 11.0);
    let merchant = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).find(|&p| w.life(p).job == Job::Merchant && w.at_work(p, w.time) && w.offer(p, bottle(), None).is_some()).expect("a merchant who sells the shelf");
    let town = w.people[merchant as usize].home.unwrap();
    let plain = Kit { mortar: Some(Grade::Common), ..Kit::default() };
    let a = KNOWN_MIXES.iter().find(|m| m.key == "healing").unwrap().brewed(40.0, plain);
    let b = KNOWN_MIXES.iter().find(|m| m.key == "warmth").unwrap().brewed(40.0, plain);
    let shelf = &mut w.society.towns[town as usize].shelf;
    shelf.clear();
    for mix in [a, b, a] {
        shelf.push(Shelved { item: bottle(), piece: Some(Piece::bottle(mix)) });
    }
    (w, merchant, a, b)
}

#[test]
fn each_mix_is_its_own_line_and_buying_gets_that_mix() {
    let (mut w, merchant, a, b) = two_bottles(1);
    let lots: Vec<_> = w.stall(merchant).into_iter().filter(|l| l.item == bottle()).collect();
    assert_eq!(lots.len(), 2, "two lines: {lots:?}");
    assert_eq!(lots.iter().find(|l| l.mix == Some(a)).unwrap().n, 2);
    assert_eq!(lots.iter().find(|l| l.mix == Some(b)).unwrap().n, 1);
    // Coin enough, standing by the merchant.
    let me = w.squad.members[0];
    w.people[me as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), 200);
    let at = w.person_pos(merchant);
    for k in 0..w.squad.members.len() {
        w.squad.at[k] = at;
    }
    w.squad.pos = at;
    assert!(w.order_talk(me, merchant));
    for _ in 0..40 {
        if w.talk.is_some() {
            break;
        }
        w.step(0.25);
    }
    let held = |w: &World| -> Vec<Mix> { w.people[me as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == bottle()).filter_map(|e| e.2.and_then(|p| p.mix)).collect() };
    let price = lots.iter().find(|l| l.mix == Some(b)).unwrap().price;
    assert!(w.buy_lot(merchant, bottle(), Some(b), price));
    assert_eq!(held(&w), vec![b], "the warming one, not the healing");
    // Across the table too.
    let mut t = Table::default();
    t.add_get_mix(bottle(), Some(a), 2);
    let d = w.deal_table(merchant, &t);
    assert!(d.stuck.is_none(), "{d:?}");
    assert_eq!(held(&w).iter().filter(|&&m| m == a).count(), 1);
    // Sold back, it keeps its mix on the shelf.
    let town = w.people[merchant as usize].home.unwrap();
    assert!(w.sell_from_mix(merchant, me, bottle(), Some(b)).is_some());
    assert!(w.society.towns[town as usize].shelf.iter().any(|s| s.piece.and_then(|p| p.mix) == Some(b)));
    assert!(held(&w).iter().all(|&m| m == a));
}
