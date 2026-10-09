//! What lives cost: memory and time per person (Part 5's budget is about
//! 50–200 extra bytes a person, and work once a day or once an hour).

use std::mem::size_of;

use gahturiyu_sim::sim::{
    history::Known,
    lives::Mind,
    memory::{Feeling, Memory},
    world::{DAY, HOUR},
    worldgen,
};

#[test]
fn lives_stay_within_budget() {
    let mut w = worldgen::generate(1);
    let t0 = std::time::Instant::now();
    let days = 20.0;
    for _ in 0..(days * DAY / HOUR) as usize {
        w.step(HOUR);
    }
    let secs = t0.elapsed().as_secs_f64();
    let people = w.society.minds.len() as f64;
    let s = &w.society;
    // In memory: each mind with its lists, plus each household's purse and
    // feelings and the world-wide stores, shared out per person.
    // (The lists are kept inline, so a mind's size counts them.)
    let minds: usize = s.minds.len() * size_of::<Mind>();
    println!("a mind is {} bytes: memories {} × {}, known events {} × {}", size_of::<Mind>(), gahturiyu_sim::sim::memory::MEMORY_CAP, size_of::<Memory>(), gahturiyu_sim::sim::history::KNOWS_CAP, size_of::<Known>());
    let houses: usize = s.households.iter().map(|h| size_of::<gahturiyu_sim::sim::lives::Purse>() + h.purse.debts.capacity() * 16 + size_of::<Vec<Feeling>>() + h.feelings.capacity() * size_of::<Feeling>()).sum();
    let stores = bincode::serialize(&(&s.history, &s.stories, &s.opps, &s.rings, &s.stolen, &s.contracts, &s.runaways)).unwrap().len();
    let habits = 8 * s.lives.len(); // honour and asking
    let ram = (minds + houses + stores + habits) as f64 / people;
    // Saved: the same, as written to disk.
    let saved = bincode::serialize(&(&s.minds, s.households.iter().map(|h| (&h.purse, &h.feelings)).collect::<Vec<_>>(), &s.history, &s.stories, &s.opps, &s.rings, &s.stolen)).unwrap().len() as f64 / people + 8.0;
    let mems = s.minds.iter().map(|m| m.memories.len()).sum::<usize>() as f64 / people;
    let knows = s.minds.iter().map(|m| m.knows.len()).sum::<usize>() as f64 / people;
    println!("after {days} days: {ram:.0} bytes a person in memory, {saved:.0} saved; {mems:.1} memories and {knows:.1} known events each; whole world {:.0} µs a person a day", secs * 1e6 / days / people);
    assert!(ram < 220.0, "{ram:.0} bytes a person");
    assert!(saved < 160.0, "{saved:.0} bytes a person saved");
}
