//! Gendered roles match the names (Laz): a Qotiro priestess has a woman's
//! name, the men who administer under her have men's names, and a Qotiro
//! priest by trade is always a woman.

use gahturiyu_sim::sim::{culture::Rule, jobs::Job, names::gender, race::Race, world::DAY, worldgen};
use gahturiyu_sim::names::Gender;

#[test]
fn gendered_roles_go_to_the_right_people() {
    let mut w = worldgen::generate(3);
    while w.time < 1.5 * DAY {
        w.step(600.0);
    }
    let mut priestesses = 0;
    let mut priests = 0;
    for t in 0..w.settlements.len() as u16 {
        let g = w.government(t);
        for c in &g.chambers {
            if c.rule != Rule::Priestesses {
                continue;
            }
            for &p in &c.holders {
                assert_eq!(gender(w.people[p as usize].seed), Gender::Female, "a priestess with a man's name");
                priestesses += 1;
            }
            for &p in &c.administrators {
                assert_eq!(gender(w.people[p as usize].seed), Gender::Male);
            }
        }
        for &p in &w.settlements[t as usize].residents {
            if w.life(p).job == Job::Priest && w.people[p as usize].race == Race::Qotiro {
                assert_eq!(gender(w.people[p as usize].seed), Gender::Female, "a Qotiro priest by trade who isn't a woman");
                assert_eq!(w.life(p).job.title(w.people[p as usize].seed), "Priestess");
                priests += 1;
            }
        }
    }
    eprintln!("{priestesses} priestesses in office, {priests} Qotiro priestesses by trade");
    assert!(priestesses + priests > 0, "some to check");
}
