//! Losing is a story: bandits who beat the squad rob the downed, and what
//! they take can be won back from them.

use gahturiyu_sim::sim::{geo::V2, items, stats::SKILLS, worldgen, World};

fn coin_of(w: &World, who: &[u32]) -> u32 {
    who.iter().map(|&m| w.count_of(m, "coin") as u32).sum()
}

#[test]
fn bandits_rob_a_beaten_squad() {
    let mut w = worldgen::generate(1);
    // A feeble squad with money on them, against a hard band that won't run.
    let squad = w.squad.members.clone();
    for &m in &squad {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 1.0);
        }
        w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), 50);
        w.people[m as usize].recompute_might();
    }
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 6, false);
    let foes: Vec<u32> = w.group(band).unwrap().members.clone();
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
        for k in SKILLS {
            w.people[f as usize].stats.set_skill(k, 70.0);
        }
        w.people[f as usize].recompute_might();
    }
    let had = coin_of(&w, &squad);
    let foes_had = coin_of(&w, &foes);
    assert!(w.attack(&squad, foes[0]));
    let mut n = 0;
    while w.squad_battle().is_some() && n < 40_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_battle().is_none(), "the fight ended");
    assert!(w.squad_fit().is_empty(), "the squad lost");
    let alive: Vec<u32> = squad.iter().copied().filter(|&m| !w.people[m as usize].dead).collect();
    assert!(!alive.is_empty());
    assert_eq!(coin_of(&w, &alive), 0, "every coin taken from the living");
    assert!(coin_of(&w, &foes) >= foes_had + alive.len() as u32 * 50, "and it's in the bandits' purses");
    assert!(had > 0);
    assert!(w.log.iter().any(|l| l.1.starts_with("Beaten.")), "{:?}", w.log);
}

#[test]
fn a_beaten_squad_is_left_alone_until_dawn() {
    use gahturiyu_sim::sim::encounters::{next_dawn, CAMP_REST};
    let mut w = worldgen::generate(1);
    let squad = w.squad.members.clone();
    for &m in &squad {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 1.0);
        }
        w.people[m as usize].recompute_might();
    }
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 6, false);
    let foes: Vec<u32> = w.group(band).unwrap().members.clone();
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
        for k in SKILLS {
            w.people[f as usize].stats.set_skill(k, 70.0);
        }
        w.people[f as usize].recompute_might();
    }
    assert!(w.attack(&squad, foes[0]));
    let mut n = 0;
    while w.squad_battle().is_some() && n < 40_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_fit().is_empty(), "the squad lost");
    let ended = w.time;
    let camp = w.camps.iter().find(|c| c.group == band).unwrap();
    assert!(camp.ready_at >= ended + CAMP_REST, "they rest first");
    assert_eq!(camp.ready_at, next_dawn(camp.ready_at), "and stay home till a dawn");
    // Right beside their camp, the squad comes round and rests: no second
    // beating before that dawn.
    let until = camp.ready_at;
    while w.time < until - 60.0 {
        w.step(30.0);
        assert!(w.squad_battle().is_none(), "attacked again at {:.1} h (safe till {:.1} h)", w.time / 3600.0, until / 3600.0);
    }
}

#[test]
fn the_downed_come_round_within_a_couple_of_hours() {
    use gahturiyu_sim::sim::body::{knocked_out, Part};
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let t = w.time;
    let max = w.people[m as usize].stats.max_hp(Part::Torso);
    // Torso at minus half its health: out cold.
    w.people[m as usize].wounds.lost[Part::Torso as usize] = max * 1.5;
    w.people[m as usize].wounds.at = t;
    assert!(knocked_out(&w.people[m as usize].wounds.hp_at(&w.people[m as usize].stats, w.time)));
    let mut n = 0;
    while knocked_out(&w.people[m as usize].wounds.hp_at(&w.people[m as usize].stats, w.time)) && n < 2000 {
        w.step(60.0);
        n += 1;
    }
    let hours = (w.time - t) / 3600.0;
    eprintln!("came round after {hours:.2} h");
    assert!(hours < 2.0, "out for {hours:.1} h");
    // And the rest heals at the ordinary rate: no faster than that.
    let hp = w.people[m as usize].wounds.hp_at(&w.people[m as usize].stats, w.time)[Part::Torso as usize];
    assert!(hp < max * 0.5, "not healed outright: {hp:.0} of {max:.0}");
}

/// Round 3 (Jo): beaten right beside the gang's camp, nobody given an order,
/// and four of five lay at 0% for five and a half hours. The promise is "within
/// an hour or two"; at least one should be up inside the first hour.
#[test]
fn beaten_beside_a_camp_someone_is_up_within_the_hour() {
    use gahturiyu_sim::sim::world::HOUR;
    for seed in 1..=4 {
        let mut w = worldgen::generate(seed);
        let squad = w.squad.members.clone();
        for &m in &squad {
            for k in SKILLS {
                w.people[m as usize].stats.set_skill(k, 1.0);
            }
            w.people[m as usize].recompute_might();
        }
        // The camp 10 m off: the squad lies beside it after.
        let at = w.squad.pos.add(V2::new(10.0, 0.0));
        let band = w.spawn_bandits(at, 6, false);
        let foes: Vec<u32> = w.group(band).unwrap().members.clone();
        for &f in &foes {
            w.people[f as usize].traits.boldness = 1.0;
            for k in SKILLS {
                w.people[f as usize].stats.set_skill(k, 70.0);
            }
            w.people[f as usize].recompute_might();
        }
        assert!(w.attack(&squad, foes[0]));
        let mut n = 0;
        while w.squad_battle().is_some() && n < 40_000 {
            w.step(0.1);
            n += 1;
        }
        assert!(w.squad_battle().is_none(), "seed {seed}: the fight ended");
        let lost = w.log.iter().any(|l| l.1.starts_with("Beaten."));
        let downed = squad.iter().filter(|&&m| w.is_down(m)).count();
        eprintln!("seed {seed}: lost {lost}, {downed} down, fit {:?}", w.squad_fit());
        assert!(lost && downed >= 2, "seed {seed}: the squad lost");
        let ended = w.time;
        let camp = w.camps.iter().find(|c| c.group == band).unwrap().pos;
        let mut first_up = None;
        while w.time < ended + 2.0 * HOUR {
            w.step(30.0);
            for &m in &squad {
                if let Some(k) = w.squad.index(m) {
                    assert!(w.member_pos(k).dist(camp) < 60.0, "seed {seed}: nobody moved them");
                }
            }
            if first_up.is_none() && squad.iter().any(|&m| !w.people[m as usize].dead && !w.is_down(m)) {
                first_up = Some(w.time);
            }
        }
        let up = first_up.map(|t| (t - ended) / HOUR);
        eprintln!("seed {seed}: first up after {up:?} h; still in a fight: {}", w.squad.members.iter().any(|m| w.fighting.contains_key(m)));
        for &m in &squad {
            let p = &w.people[m as usize];
            eprintln!("  {m}: down {} rate {} rally {} activity {:?}", w.is_down(m), p.wounds.rate, p.wounds.rally, p.cond.as_ref().map(|c| c.activity));
        }
        assert!(up.is_some_and(|h| h <= 1.0), "seed {seed}: nobody up within the hour ({up:?})");
    }
}

/// Beat a feeble squad with a strong band right by its camp (world 1).
/// Returns the world, the band and the camp's middle.
fn beaten_by_a_camp() -> (World, u32, V2) {
    let mut w = worldgen::generate(1);
    let squad = w.squad.members.clone();
    for &m in &squad {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 1.0);
        }
        w.people[m as usize].recompute_might();
    }
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 6, false);
    let foes: Vec<u32> = w.group(band).unwrap().members.clone();
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
        for k in SKILLS {
            w.people[f as usize].stats.set_skill(k, 70.0);
        }
        w.people[f as usize].recompute_might();
    }
    assert!(w.attack(&squad, foes[0]));
    let mut n = 0;
    while w.squad_battle().is_some() && n < 40_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_fit().is_empty(), "the squad lost");
    (w, band, at)
}

/// Round 3 (Jo): one member on half health walked into the camp in daylight
/// with five bandits there and took their things. Resting after its win, a
/// gang still sees who walks onto its own ground.
#[test]
fn a_resting_gang_still_guards_its_camp() {
    use gahturiyu_sim::sim::encounters::{DUMP_AT, GUARD_RING};
    let (mut w, band, camp) = beaten_by_a_camp();
    // Nobody is left lying in their camp.
    for &m in &w.squad.members {
        let k = w.squad.index(m).unwrap();
        assert!(w.member_pos(k).dist(camp) > GUARD_RING, "dragged out of the camp");
        assert!(w.member_pos(k).dist(camp) < DUMP_AT + 5.0, "but not far");
    }
    assert!(w.log.iter().any(|l| l.1.starts_with("They drag you out")), "{:?}", w.log);
    // Someone comes round and goes in for the pile.
    let ended = w.time;
    while w.squad_fit().is_empty() && w.time < ended + 3.0 * 3600.0 {
        w.step(30.0);
    }
    let up = w.squad_fit()[0];
    let ready = w.camps.iter().find(|c| c.group == band).unwrap().ready_at;
    assert!(ready > w.time, "the gang is resting");
    w.order_members(&[up], camp);
    let went = w.time;
    while w.squad_battle().is_none() && w.time < went + 600.0 {
        w.step(0.5);
    }
    assert!(w.squad_battle().is_some(), "walking into the camp starts a fight");
    assert!(w.log.iter().any(|l| l.1.starts_with("You're seen in their camp")), "{:?}", w.log);
}

/// Play-test item 13: "They'll have it back at their camp" is true. What the
/// robbers take (besides coin) lies in their camp's stash, and once they're
/// gone it can be taken back, no crime.
#[test]
fn what_they_take_is_in_their_camps_stash() {
    use gahturiyu_sim::sim::{containers::is_stash, loot::Source};
    let mut w = worldgen::generate(1);
    let squad = w.squad.members.clone();
    // Something worth taking on each, and no fight in them.
    for &m in &squad {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 1.0);
        }
        w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("gold_ring"), 1);
        w.people[m as usize].recompute_might();
    }
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 6, false);
    let stash = w.camps.iter().find(|c| c.group == band).unwrap().stash.expect("a stash");
    assert!(is_stash(stash));
    let had = w.container(stash).unwrap().items.clone();
    let foes: Vec<u32> = w.group(band).unwrap().members.clone();
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
        for k in SKILLS {
            w.people[f as usize].stats.set_skill(k, 70.0);
        }
        w.people[f as usize].recompute_might();
    }
    assert!(w.attack(&squad, foes[0]));
    let mut n = 0;
    while w.squad_battle().is_some() && n < 40_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_fit().is_empty(), "the squad lost");
    let now = w.container(stash).unwrap().items.clone();
    let rings = |v: &[gahturiyu_sim::sim::inventory::Entry]| v.iter().filter(|e| e.0 == items::id("gold_ring")).map(|e| e.1).sum::<u16>();
    assert!(rings(&now) > rings(&had), "the rings went to the stash: {now:?}");
    assert!(w.log.iter().any(|l| l.1.contains("back at their camp")));
    // The gang gone (moved on, say), someone comes round and takes it back.
    for &f in &foes {
        w.people[f as usize].dead = true;
    }
    let ended = w.time;
    while w.squad_fit().is_empty() && w.time < ended + 3.0 * 3600.0 {
        w.step(30.0);
    }
    let me = w.squad_fit()[0];
    let rings_before = w.count_of(me, "gold_ring");
    assert!(w.order_search(me, stash));
    let t0 = w.time;
    while w.searching_now(me).is_none() && w.time < t0 + 600.0 {
        w.step(0.5);
    }
    assert_eq!(w.searching_now(me), Some(stash));
    assert!(w.take_all_from(me, Source::Chest(stash)) > 0);
    assert!(w.count_of(me, "gold_ring") > rings_before, "got the rings back");
    assert!(!w.log.iter().any(|l| l.1.contains("seen stealing")), "{:?}", w.log);
}

/// A band of bandits coming into sight says it's bandits, rather than
/// "wandering, crosses your path", and nothing more.
#[test]
fn bandits_in_sight_say_so() {
    let mut w = worldgen::generate(3);
    let band = w.spawn_bandits(w.squad.pos.add(V2::new(400.0, 0.0)), 5, false);
    let line = w.describe_group(band);
    // (Laz: how hard a fight is, you find out by fighting.)
    assert!(line.ends_with(": bandits."), "{line}");
}
