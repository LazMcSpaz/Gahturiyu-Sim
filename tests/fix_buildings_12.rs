//! U-14: a change of household custom forms households again, and that
//! renumbers them. What named a household by its old number (a memory of
//! it, a job it was giving) now names the household its people live in.

use gahturiyu_sim::sim::{
    chances::Giver,
    culture::Belonging,
    memory::{Memory, Who},
    worldgen,
};

#[test]
fn households_formed_again_keep_who_is_who() {
    let mut w = worldgen::generate(1);
    // A day and a half on, so people have jobs going.
    for _ in 0..36 {
        w.step(3600.0);
    }
    // A household of several, somewhere past the first few (so renumbering
    // moves it), under a custom that will change.
    let (h, a) = w
        .society
        .households
        .iter()
        .enumerate()
        .skip(5)
        .find(|(_, x)| x.members.len() >= 2)
        .map(|(k, x)| (k as u32, x.members[0]))
        .expect("a household of two or more");
    // Someone elsewhere remembers that household.
    let p = w.society.lives.iter().enumerate().find(|(i, l)| l.household.is_some_and(|x| x != h) && !w.people[*i].dead).map(|(i, _)| i).unwrap();
    let before = w.society.minds[p].memories.len();
    w.society.minds[p].memories.push(Memory { about: Who::Household(h), amount: -5.0, ..Memory::NONE });
    // And that household is giving a job.
    let mut o = w.society.opps.first().cloned().expect("some job going");
    o.giver = Giver::Household(h);
    o.id = 999_999;
    w.society.opps.push(o);

    // Every community lives by another custom now.
    for c in &mut w.society.communities {
        c.customs.belonging = if c.customs.belonging == Belonging::Lodging { Belonging::Lineage } else { Belonging::Lodging };
    }
    w.form_all_households();

    let now = w.society.lives[a as usize].household.expect("still in a household");
    assert_ne!(now, h, "the test wants the number to change");
    let m = w.society.minds[p].memories[before];
    assert_eq!(m.about, Who::Household(now), "the memory follows the people");
    let o = w.society.opps.iter().find(|o| o.id == 999_999).unwrap();
    assert_eq!(o.giver, Giver::Household(now), "the job follows the people");
}
