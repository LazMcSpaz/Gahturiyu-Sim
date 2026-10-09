//! Recruiting: a restless townsperson asked to join leaves their town and
//! becomes a full squad member.

use gahturiyu_sim::sim::{dialogue::Topic, items, recruit::MAX_SQUAD, worldgen, World};

/// A world a day and a morning in (the first dawn sorts out who's out of
/// work; by midday everyone's up).
fn world() -> World {
    let mut w = worldgen::generate(1);
    for _ in 0..180 {
        w.step(600.0);
    }
    w
}

/// The nearest person (to the squad) who'd come, with their fee.
fn willing(w: &World, free: Option<bool>) -> (gahturiyu_sim::sim::person::PersonId, u16) {
    let mut best: Option<(f32, u32, u16)> = None;
    for s in &w.settlements {
        for &p in &s.residents {
            if let Some(fee) = w.join_terms(p).filter(|&f| free.is_none_or(|fr| (f == 0) == fr) && !w.is_indoors_asleep(p)) {
                let d = w.person_pos(p).dist(w.squad.pos);
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, p, fee));
                }
            }
        }
    }
    let b = best.expect("someone willing");
    (b.1, b.2)
}

#[test]
fn only_some_would_join() {
    let w = world();
    let all: usize = w.settlements.iter().map(|s| s.residents.len()).sum();
    let free: usize = w.settlements.iter().flat_map(|s| &s.residents).filter(|&&p| w.join_terms(p) == Some(0)).count();
    let paid: usize = w.settlements.iter().flat_map(|s| &s.residents).filter(|&&p| w.join_terms(p).is_some_and(|f| f > 0)).count();
    eprintln!("{all} townsfolk: {free} would come for nothing, {paid} for a fee");
    assert!(free > 0 && paid > 0, "both kinds exist");
    assert!(free + paid < all / 12, "most people have lives they won't leave");
}

#[test]
fn a_recruit_leaves_town_and_joins_the_squad() {
    let mut w = world();
    let (npc, fee) = willing(&w, Some(false));
    assert!(fee > 0);
    let town = w.people[npc as usize].home.unwrap();
    let who = w.squad.members[0];
    w.teleport_squad(w.person_pos(npc));
    assert!(w.order_talk(who, npc));
    w.step(0.1);
    assert!(w.talk.is_some(), "talking");
    assert!(w.topics().contains(&Topic::Join(fee)), "they're asked");
    // Can't pay: they stay.
    let coin = items::id("coin");
    for m in w.squad.members.clone() {
        w.people[m as usize].detail.as_mut().unwrap().gear.bag.retain(|e| e.0 != coin);
    }
    w.ask(Topic::Join(fee));
    assert!(!w.people[npc as usize].in_squad, "no fee, no recruit");
    // With the coin they come.
    w.people[who as usize].detail.as_mut().unwrap().gear.add(coin, fee + 5);
    w.ask(Topic::Join(fee));
    assert!(w.people[npc as usize].in_squad);
    assert!(w.squad.index(npc).is_some());
    assert_eq!(w.squad_count(coin), 5, "the fee was paid");
    assert!(!w.settlements[town as usize].residents.contains(&npc), "gone from the town's roll");
    assert!(w.society.lives[npc as usize].community.is_none());
    assert!(w.people[npc as usize].cond.is_some(), "on the squad's condition timeline");
    assert_eq!(w.topics(), vec![Topic::Goodbye]);
    w.end_talk();
    // They follow orders, live through days and come back from a save.
    let to = w.squad.pos.add(gahturiyu_sim::sim::geo::V2::new(30.0, 0.0));
    w.order_squad(to);
    let k = w.squad.index(npc).unwrap();
    let start = w.squad.at[k];
    for _ in 0..600 {
        w.step(0.1);
    }
    let k = w.squad.index(npc).unwrap();
    assert!(w.squad.at[k].dist(start) > 10.0, "they walk with the squad");
    for _ in 0..60 {
        w.step(600.0);
    }
    assert!(w.squad.index(npc).is_some(), "still with us two days on");
    let back = World::load_bytes(&w.save_bytes()).ok().unwrap();
    assert!(back.squad.index(npc).is_some() && back.people[npc as usize].in_squad);
}

#[test]
fn a_full_squad_takes_no_more() {
    let mut w = world();
    let mut n = 0;
    while w.squad.members.len() < MAX_SQUAD && n < 20 {
        let (npc, fee) = willing(&w, None);
        w.teleport_squad(w.person_pos(npc));
        let by = w.squad.members[0];
        w.people[by as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), fee);
        w.recruit(npc, by).expect("joins");
        n += 1;
    }
    assert_eq!(w.squad.members.len(), MAX_SQUAD);
    let anyone = w.settlements.iter().flat_map(|s| &s.residents).any(|&p| w.join_terms(p).is_some());
    assert!(!anyone, "no room");
}
