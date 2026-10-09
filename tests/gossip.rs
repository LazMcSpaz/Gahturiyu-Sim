//! History and gossip: who knows what, and how word gets about.

use gahturiyu_sim::sim::{
    history::{self, Deed},
    world::{DAY, HOUR},
    worldgen, World,
};

fn run(w: &mut World, secs: f64) {
    let n = (secs / HOUR).round() as usize;
    for _ in 0..n {
        w.step(HOUR);
    }
}

/// Two townsfolk of the biggest town, from different households.
fn pair(w: &World) -> (u16, u32, u32) {
    let town = (0..w.settlements.len()).max_by_key(|&s| w.settlements[s].residents.len()).unwrap() as u16;
    let folk: Vec<u32> = w.settlements[town as usize].residents.iter().copied().filter(|&p| !w.people[p as usize].in_squad && w.society.lives[p as usize].household.is_some()).collect();
    let a = folk[10];
    let b = *folk.iter().find(|&&b| w.society.lives[b as usize].household != w.society.lives[a as usize].household).unwrap();
    (town, a, b)
}

#[test]
fn an_event_is_known_to_those_who_saw_it_until_word_gets_about() {
    let mut w = worldgen::generate(1);
    run(&mut w, DAY);
    let (town, a, b) = pair(&w);
    let t = w.time;
    let id = w.note(Deed::Theft, Some(a), Some(b), town, t, false);
    let e = w.event(id).unwrap().clone();
    let mut first = w.knowers(id);
    let mut expect: Vec<u32> = vec![a, b];
    expect.extend(e.witnesses.iter().copied());
    expect.sort_unstable();
    first.sort_unstable();
    assert_eq!(first, expect, "only those involved and those who saw");
    // The same day, still only them.
    run(&mut w, 10.0 * HOUR);
    assert_eq!(w.knowers(id).len(), first.len());
    run(&mut w, 6.0 * DAY);
    let later = w.knowers(id);
    assert!(later.len() > first.len() + 3, "word got about: {} then {}", first.len(), later.len());
    // Those who heard think less of the thief, but it isn't their grievance.
    let heard = later.iter().copied().find(|p| !first.contains(p) && w.mind(*p).memories.iter().any(|m| m.heard)).expect("someone thinks less of them");
    assert!(w.mind(heard).memories.iter().any(|m| m.about == gahturiyu_sim::sim::memory::Who::Person(a) && m.amount < 0.0));
}

#[test]
fn news_travels_with_travellers_and_violence_makes_folk_fearful() {
    let mut w = worldgen::generate(1);
    run(&mut w, DAY);
    let (town, a, b) = pair(&w);
    let t = w.time;
    let id = w.note(Deed::Feud, Some(a), Some(b), town, t, false);
    let before: f32 = w.settlements[town as usize].residents.iter().map(|&p| w.mind(p).needs[2]).sum();
    run(&mut w, 12.0 * DAY);
    let away = w.knowers(id).into_iter().filter(|&p| w.people[p as usize].home.is_some_and(|h| h != town)).count();
    assert!(away > 0, "another town heard of it");
    let after: f32 = w.settlements[town as usize].residents.iter().map(|&p| w.mind(p).needs[2]).sum();
    assert!(after > before, "the town feels less safe: {before} then {after}");
    // Nobody knows more than they can hold.
    assert!(w.society.minds.iter().all(|m| m.knows.len() <= history::KNOWS_CAP));
}
