//! Fixes from the bug hunt, batch 4 (ruins-gangs): who a fight pulls in,
//! the night's mercy, getting at a stash, a lone member at a camp, a lair's
//! beast and its chest.

use gahturiyu_sim::sim::{
    geo::V2,
    loot::{LootRef, Source},
    ruins::RuinKind,
    stats::{Skill, SKILLS},
    worldgen,
    world::{DAY, HOUR},
    World,
};

fn weaken(w: &mut World, who: &[u32]) {
    for &m in who {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 1.0);
        }
        w.people[m as usize].recompute_might();
    }
}

fn harden_band(w: &mut World, band: u32) {
    let foes: Vec<u32> = w.group(band).unwrap().members.clone();
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
        for k in SKILLS {
            w.people[f as usize].stats.set_skill(k, 70.0);
        }
        w.people[f as usize].recompute_might();
    }
}

/// Move one member somewhere on their own, at once.
fn put(w: &mut World, m: u32, at: V2) {
    let k = w.squad.index(m).unwrap();
    w.squad.at[k] = at;
    w.squad.goal[k] = at;
    w.squad.route[k].clear();
}

/// Run on until the clock reads `hour` (0..24).
fn until_hour(w: &mut World, hour: f64) {
    let target = ((w.time / DAY).floor() * DAY + hour * HOUR).max(w.time);
    let target = if target <= w.time { target + DAY } else { target };
    while w.time < target {
        w.step(60.0_f64.min(target - w.time).max(0.01));
    }
}

#[test]
fn bl15_a_fight_takes_in_only_those_near_it() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let squad = w.squad.members.clone();
    let far = squad[3];
    let away = w.squad.pos.add(V2::new(-900.0, 0.0));
    put(&mut w, far, away);
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 3, false);
    let foe = w.group(band).unwrap().members[0];
    let near: Vec<u32> = squad[..1].to_vec();
    assert!(w.attack(&near, foe));
    assert!(w.fighter(squad[0]).is_some(), "the one sent is in it");
    assert!(w.fighter(squad[1]).is_some(), "one standing beside them is in it");
    assert!(w.fighter(far).is_none(), "one 900 m off is not");
    w.step(1.0);
    assert!(w.fighter(far).is_none(), "and stays out");
    // One who walks up to it joins in.
    put(&mut w, far, at.add(V2::new(-12.0, 0.0)));
    w.step(0.5);
    if w.squad_battle().is_some() {
        assert!(w.fighter(far).is_some(), "walked in: joined");
    }
}

#[test]
fn bl19_a_gang_that_beat_the_squad_leaves_it_alone_till_dawn() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let squad = w.squad.members.clone();
    weaken(&mut w, &squad);
    let at = w.squad.pos.add(V2::new(8.0, 0.0));
    let band = w.spawn_bandits(at, 5, false);
    harden_band(&mut w, band);
    let foe = w.group(band).unwrap().members[0];
    assert!(w.attack(&squad, foe));
    let mut n = 0;
    while w.squad_battle().is_some() && n < 40_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_battle().is_none(), "the fight ended");
    let ready = w.camps.iter().find(|c| c.group == band).map(|c| c.ready_at).expect("their camp");
    // Everyone comes round and stands about, near their camp, till dawn.
    let mut fights = 0;
    while w.time < ready - 60.0 {
        w.step(30.0);
        if w.squad_battle().is_some() {
            fights += 1;
            break;
        }
    }
    assert_eq!(fights, 0, "jumped again before dawn");
}

#[test]
fn bl20_a_stash_and_a_ruins_cache_sit_clear_of_everyone_round_the_fire() {
    let w = worldgen::generate(1);
    for c in &w.camps {
        let Some(id) = c.stash else { continue };
        let at = w.container(id).unwrap().pos;
        let g = w.group(c.group).unwrap();
        for &m in &g.members {
            let d = w.person_pos(m).dist(at);
            assert!(d > 4.5, "a bandit sits {d:.1} m from their stash");
        }
    }
    for ru in w.ruins.iter().filter(|r| r.kind == RuinKind::Ruin) {
        let g = w.group(ru.guards.unwrap()).unwrap();
        for c in w.ruin_containers(ru.id) {
            for &m in &g.members {
                let d = w.person_pos(m).dist(c.pos);
                assert!(d > 4.5, "a warden sits {d:.1} m from the cache");
            }
        }
    }
}

#[test]
fn bl20_at_night_a_good_sneak_at_the_stash_has_a_while_before_being_seen() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    until_hour(&mut w, 1.0);
    // Somewhere quiet: the squad far off, one member at a camp's stash.
    let c = w.camps[0].clone();
    let stash = w.container(c.stash.unwrap()).unwrap().pos;
    let squad = w.squad.members.clone();
    w.teleport_squad(c.pos.add(V2::new(600.0, 0.0)));
    let m = squad[0];
    w.people[m as usize].stats.set_skill(Skill::Sneak, 80.0);
    w.set_sneaking(m, true);
    put(&mut w, m, stash.add(stash.sub(c.pos).scale(1.0 / stash.dist(c.pos)).scale(1.0)));
    let lit = w.watch_rate(c.group, m);
    // With the fire doused (the thief's way in).
    let t = w.time;
    w.camps[0].doused_until = t + 3600.0;
    let rate = w.watch_rate(c.group, m);
    eprintln!("night: fire lit {lit:.2}/s, doused {rate:.2}/s");
    assert!(rate < 0.15, "seen at {rate:.2} a second: under seven seconds at the stash");
    // By day the same spot is hopeless.
    until_hour(&mut w, 12.0);
    put(&mut w, m, stash.add(stash.sub(c.pos).scale(1.0 / stash.dist(c.pos)).scale(1.0)));
    let day = w.watch_rate(c.group, m);
    assert!(day > rate * 3.0, "day {day:.2} is much worse than night {rate:.2}");
}

#[test]
fn bl21_one_member_alone_in_a_camp_is_seen() {
    let mut w = worldgen::generate(1);
    let c = w.camps[0].clone();
    // The rest far off; one walks into the camp in daylight.
    w.teleport_squad(c.pos.add(V2::new(2400.0, 0.0)));
    until_hour(&mut w, 11.0);
    let m = w.squad.members[0];
    put(&mut w, m, c.pos.add(V2::new(20.0, 0.0)));
    let mut seen = false;
    for _ in 0..120 {
        w.step(0.5);
        if w.suspicion_of(m) >= 1.0 || w.fighter(m).is_some() {
            seen = true;
            break;
        }
    }
    assert!(seen, "walked into a camp alone and nobody looked");
}

#[test]
fn bl55_a_beast_at_home_defends_its_chest() {
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 12.0);
    let mut tried = 0;
    for ru in w.ruins.clone() {
        let RuinKind::Lair(h) = ru.kind else { continue };
        if !w.beast_at_home(h) {
            continue;
        }
        let id = w.ruin_containers(ru.id).next().map(|c| c.id).unwrap();
        let spot = w.container(id).unwrap().pos;
        w.teleport_squad(spot.add(V2::new(2.0, 0.0)));
        let m = w.squad.members[0];
        assert!(w.order_search(m, id));
        let mut k = 0;
        while w.searching_now(m) != Some(id) && k < 600 && w.squad_battle().is_none() {
            w.step(0.25);
            k += 1;
        }
        if w.squad_battle().is_some() {
            // Set on before even reaching it: the beast is home all right.
            tried += 1;
            break;
        }
        assert!(w.take_from(m, Source::Chest(id), LootRef::Pack(0)));
        assert!(w.squad_battle().is_some(), "took from its chest under its nose");
        tried += 1;
        break;
    }
    assert!(tried > 0, "no lair with its beast at home at noon");
}
