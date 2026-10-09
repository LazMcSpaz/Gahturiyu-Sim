//! Memory and grudges: what people remember of each other, and how a grudge
//! between households climbs.

use gahturiyu_sim::sim::{
    history::Deed,
    memory::{self, Who},
    world::{DAY, HOUR},
    worldgen, World,
};

fn run(w: &mut World, secs: f64) {
    let n = (secs / HOUR).round() as usize;
    for _ in 0..n {
        w.step(HOUR);
    }
}

fn today(w: &World) -> i32 {
    World::day_of(w.time) as i32
}

/// Two small households of the same town, with no squad members.
fn two_households(w: &World) -> (u32, u32) {
    let ok = |h: usize| {
        let hh = &w.society.households[h];
        (1..=4).contains(&hh.members.len()) && hh.members.iter().all(|&m| !w.people[m as usize].in_squad && !w.people[m as usize].dead)
    };
    for a in 0..w.society.households.len() {
        if !ok(a) {
            continue;
        }
        let ca = w.society.households[a].community;
        if let Some(b) = (a + 1..w.society.households.len()).find(|&b| ok(b) && w.society.households[b].community == ca) {
            return (a as u32, b as u32);
        }
    }
    panic!("no two households");
}

#[test]
fn memories_fade_and_the_list_stays_short() {
    let mut w = worldgen::generate(1);
    let p = w.settlements[0].residents[3];
    let d = today(&w);
    for k in 0..12u32 {
        w.remember(p, Who::Person(100 + k), Deed::Slight, -0.2 - k as f32 * 0.01, d);
    }
    assert_eq!(w.mind(p).memories.len(), memory::MEMORY_CAP);
    // The strongest are the ones kept.
    assert!(w.mind(p).memories.iter().all(|m| m.amount <= -0.25));
    // A second slight by the same hand adds to what's remembered of them.
    w.remember(p, Who::Person(111), Deed::Slight, -0.2, d);
    assert_eq!(w.mind(p).memories.len(), memory::MEMORY_CAP);
    let m = w.mind(p).memories.iter().find(|m| m.about == Who::Person(111)).unwrap();
    assert!(m.amount < -0.4);
    // Wrongs fade, slower for the patient.
    let late = d + 40;
    assert!(m.strength(late, 0.0).abs() < m.strength(late, 1.0).abs());
    assert!(m.strength(late, 0.5).abs() < m.amount.abs() * 0.5);
}

#[test]
fn people_remember_their_dealings_with_those_around_them() {
    let mut w = worldgen::generate(1);
    run(&mut w, 10.0 * DAY);
    let with: usize = w.society.minds.iter().filter(|m| !m.memories.is_empty()).count();
    let n = w.society.minds.len();
    assert!(with > n / 10, "dealings happen: {with} of {n} remember someone");
    assert!(w.society.households.iter().any(|h| h.feelings.iter().any(|f| f.warmth > 0.0)));
    assert!(w.society.households.iter().any(|h| h.feelings.iter().any(|f| f.warmth < 0.0)));
}

/// Slight household `a` from household `b` every day; see what `a` does.
fn slighted(honour: f32) -> Vec<Deed> {
    let mut w = worldgen::generate(1);
    let (a, b) = two_households(&w);
    let town = w.society.communities[w.society.households[a as usize].community as usize].town;
    for &m in &w.society.households[a as usize].members.clone() {
        w.society.lives[m as usize].habits.honour = honour;
        w.people[m as usize].traits.boldness = 0.9;
        w.people[m as usize].traits.patience = 0.2;
    }
    let ours = w.society.households[a as usize].members.clone();
    let them = w.society.households[b as usize].members[0];
    for _ in 0..30 {
        let d = today(&w);
        for &m in &ours {
            w.remember(m, Who::Person(them), Deed::Slight, -0.25, d);
        }
        run(&mut w, DAY);
    }
    let f = w.feeling(a, b).unwrap();
    assert!(f.stage >= 3, "the grudge climbed: {f:?}");
    w.events(town).iter().filter(|e| e.actor.is_some_and(|x| ours.contains(&x))).map(|e| e.deed).collect()
}

const OPEN: [Deed; 6] = [Deed::PublicDispute, Deed::Claim, Deed::Duel, Deed::Shun, Deed::Record, Deed::Brawl];
const COVERT: [Deed; 4] = [Deed::Slander, Deed::Sabotage, Deed::Theft, Deed::Beating];

#[test]
fn the_honourable_take_a_grudge_into_the_open() {
    let deeds = slighted(0.95);
    assert!(deeds.iter().any(|d| OPEN.contains(d)), "{deeds:?}");
    assert!(!deeds.iter().any(|d| COVERT.contains(d)), "{deeds:?}");
}

#[test]
fn the_dishonourable_settle_it_in_the_dark() {
    let deeds = slighted(0.05);
    assert!(deeds.iter().any(|d| COVERT.contains(d)), "{deeds:?}");
    assert!(!deeds.iter().any(|d| OPEN.contains(d)), "{deeds:?}");
}
