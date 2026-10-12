//! Laz's play notes of 11 Oct, P23 to P26: how far a chase goes, what the
//! player calls people, how fast wounds heal, and what's said where.

use gahturiyu_sim::sim::{
    ai,
    combat::SQUAD_SIDE,
    geo::V2,
    stats::SKILLS,
    worldgen, World,
};

fn harden(w: &mut World, who: &[u32], skill: f32) {
    for &m in who {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, skill);
        }
        w.people[m as usize].recompute_might();
    }
}

/// A band of four just east of the squad, out in the open, the squad at it.
fn into_a_fight(w: &mut World) -> u32 {
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 4, false);
    let foe = w.group(band).unwrap().members[0];
    let squad = w.squad.members.clone();
    assert!(w.attack(&squad, foe));
    band
}

/// The furthest anyone of the band ever got from where the fight started.
fn furthest(w: &World, far: &mut f32) {
    if let Some(b) = w.squad_battle() {
        let Some(g) = b.ground else { return };
        for f in b.fighters.iter().filter(|f| f.side != SQUAD_SIDE && f.active()) {
            *far = far.max(f.pos.dist(g));
        }
    }
}

/// P23: the whole squad turns and walks off. The band comes after it only
/// as far as their nerve takes them from where they fought, then lets it
/// go; the fight ends long before its time limit.
#[test]
fn p23_a_band_lets_a_squad_that_walks_off_go() {
    let mut w = worldgen::generate(1);
    let band = into_a_fight(&mut w);
    let start = w.time;
    w.step(2.0);
    let away = w.squad.pos.add(V2::new(-600.0, 0.0));
    w.order_squad(away);
    let mut far = 0.0f32;
    while w.squad_battle().is_some() && w.time - start < 900.0 {
        w.step(0.5);
        furthest(&w, &mut far);
    }
    let took = w.time - start;
    let most = (ai::CHASE_BASE + ai::CHASE_BOLD) * 1.3 * ai::CHASE_FREE;
    println!("over after {took:.0} s; the furthest any of them went: {far:.0} m (most {most:.0})");
    assert!(w.squad_battle().is_none() && took < 600.0, "the fight was still on after {took:.0} s");
    assert!(far <= most + 15.0, "one of them chased {far:.0} m from the fight");
    let _ = band;
}

/// P23: one of the squad walks off while the rest fight on. The band
/// doesn't all go after them: whoever follows turns back to the fight, and
/// the one walking off ends up clear of everyone, with nobody after them.
#[test]
fn p23_one_who_walks_off_is_let_go_while_the_rest_fight() {
    let mut w = worldgen::generate(1);
    let squad = w.squad.members.clone();
    harden(&mut w, &squad[..3], 60.0);
    let _band = into_a_fight(&mut w);
    let runner = squad[3];
    w.step(2.0);
    let away = w.squad.pos.add(V2::new(-500.0, 0.0));
    w.order_members(&[runner], away);
    let mut let_go = false;
    for _ in 0..240 {
        w.step(0.5);
        let Some(b) = w.squad_battle() else { break };
        let Some(r) = b.index_of(runner) else { break };
        let after = b.fighters.iter().filter(|f| f.side != SQUAD_SIDE && f.active() && f.target == Some(r)).count();
        let near = b.fighters.iter().filter(|f| f.side != SQUAD_SIDE && f.active() && !f.fleeing).map(|f| f.pos.dist(b.fighters[r].pos)).fold(f32::MAX, f32::min);
        if after == 0 && near > 45.0 {
            let_go = true;
            break;
        }
    }
    assert!(let_go, "the band kept after the one who walked off");
}


/// P24: one rule for what the player reads as someone's name. Bandits are
/// "Bandit" (or "Bandit archer", "Bandit mage"), in the hover's name, a
/// fight's lines and a band coming into sight; the squad and townsfolk keep
/// their names; anyone talked with is known by name.
#[test]
fn p24_bandits_are_called_bandits() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let at = w.squad.pos.add(V2::new(12.0, 0.0));
    let band = w.spawn_bandits(at, 4, true);
    let members = w.group(band).unwrap().members.clone();
    let called: Vec<String> = members.iter().map(|&m| w.name_of(m)).collect();
    assert_eq!(called[0], "Bandit mage", "{called:?}");
    assert_eq!(called[1], "Bandit archer", "{called:?}");
    assert!(called.iter().all(|c| c.starts_with("Bandit")), "{called:?}");
    assert!(members.iter().all(|&m| w.people[m as usize].name().is_some_and(|n| !n.starts_with("Bandit"))), "they still have names of their own");
    let line = w.describe_group(band);
    assert!(line.ends_with(": bandits.") && !line.contains(w.people[members[0] as usize].name().unwrap()), "{line}");
    // The squad and townsfolk as before.
    let me = w.squad.members[0];
    assert_eq!(Some(w.name_of(me)), w.people[me as usize].name().map(str::to_string));
    let towner = (0..w.people.len() as u32).find(|&p| w.people[p as usize].home.is_some() && w.people[p as usize].name().is_some()).expect("a townsperson with a name");
    assert_eq!(Some(w.name_of(towner)), w.people[towner as usize].name().map(str::to_string));
    // A fight's lines name them the same way.
    assert!(w.attack(&w.squad.members.clone(), members[2]));
    let b = w.squad_battle().unwrap();
    for &m in &members {
        let i = b.index_of(m).unwrap();
        assert!(b.names[i].starts_with("Bandit"), "{:?}", b.names);
    }
    // Talked with, they're known by name.
    w.met.insert((members[3], me));
    assert_eq!(Some(w.name_of(members[3])), w.people[members[3] as usize].name().map(str::to_string));
}

/// P25: how long a typical bad wound takes to heal, by how the member
/// spends the time. The wound: a squad member's body down to two fifths, a
/// leg to half and an arm to two thirds (a fight lost, not quite out cold).
/// Prints the table for the report, before (the old numbers) and now.
#[test]
fn p25_how_long_a_bad_wound_takes() {
    use gahturiyu_sim::sim::body::{self, Part, Wounds, PARTS};
    let w = worldgen::generate(1);
    let p = &w.people[w.squad.members[0] as usize];
    let max = |k: Part| p.stats.max_hp(k);
    let mut lost = [0.0f32; 6];
    lost[Part::Torso as usize] = max(Part::Torso) * 0.6;
    lost[Part::LeftLeg as usize] = max(Part::LeftLeg) * 0.5;
    lost[Part::RightArm as usize] = max(Part::RightArm) * 0.35;
    let _ = PARTS;
    let hours = |base: f32, mult: f32, share: f32| -> f32 {
        let wd = Wounds { lost: lost.map(|l| l * share), rate: base * mult, ..Wounds::default() };
        wd.healed_in().unwrap()
    };
    // A day on the road: 16 hours walking, 8 asleep in the open.
    let road = |base: f32, share: f32| -> f32 {
        let worst = lost.iter().fold(0.0f32, |a, b| a.max(*b)) * share;
        let per_day = base * (16.0 * body::HEAL_WALKING + 8.0 * body::HEAL_SLEEP_OPEN);
        worst / per_day * 24.0
    };
    let rows: [(&str, f32); 6] = [
        ("walking all day", body::HEAL_WALKING),
        ("resting, awake", body::HEAL_RESTING),
        ("asleep in the open", body::HEAL_SLEEP_OPEN),
        ("asleep in a tent", body::HEAL_SLEEP_TENT),
        ("asleep under a roof", body::HEAL_SLEEP_INDOORS),
        ("asleep in a bed (inn)", body::HEAL_SLEEP_BED),
    ];
    let (old_base, new_base) = (6.0f32, body::HEAL_PER_HOUR);
    println!("A bad wound (torso {:.0} of {:.0} lost):", lost[Part::Torso as usize], max(Part::Torso));
    for (what, m) in rows {
        println!("  {what:24} before {:6.1} h   now {:6.1} h", hours(old_base, m, 1.0), hours(new_base, m, 1.0));
    }
    println!("  {:24} before {:6.1} h   now {:6.1} h", "on the road (16 + 8)", road(old_base, 1.0), road(new_base, 1.0));
    println!("  {:24} before {:6.1} h   now {:6.1} h", "healer, then resting", 0.0, hours(new_base, body::HEAL_RESTING, 1.0 - body::HEALER_MENDS));
    println!("  {:24} before {:6.1} h   now {:6.1} h", "healer, then a bed", 0.0, hours(new_base, body::HEAL_SLEEP_BED, 1.0 - body::HEALER_MENDS));
    // What was asked for: slower all round. A night in an inn bed no
    // longer mends it; a few days on the road do.
    assert!(hours(new_base, body::HEAL_SLEEP_BED, 1.0) > 9.0);
    assert!(road(new_base, 1.0) > 48.0);
}

fn talk_to(w: &mut World, npc: u32) -> u32 {
    let lead = w.squad.members[0];
    while w.is_indoors_asleep(npc) {
        w.step(60.0);
    }
    let at = w.person_pos(npc);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    w.squad.at[0] = at.add(V2::new(1.0, 0.0));
    assert!(w.order_talk(lead, npc));
    w.step(0.5);
    assert!(w.talk.is_some(), "the conversation should open");
    lead
}

/// P26: what's asked depends on where the talk is and who with. In their
/// own town, someone is asked what it's like; on the road with others,
/// where they're headed and what's out that way, and never "this town".
/// The kind of place comes from one function, `place_at`.
#[test]
fn p26_what_is_asked_follows_the_situation() {
    use gahturiyu_sim::sim::{dialogue::Topic, place::Place};
    let mut w = worldgen::generate(1);
    // The kind of place.
    let town = &w.settlements[0];
    assert_eq!(w.place_at(town.pos), Place::Town(town.id));
    let camp = w.camps.first().map(|c| (c.pos, c.group));
    if let Some((at, g)) = camp {
        assert_eq!(w.place_at(at), Place::Camp(g));
    }
    // In town, with someone who lives there.
    let local = w.settlements[0].residents.iter().copied().find(|&p| !w.people[p as usize].dead && w.place_at(w.person_pos(p)) == Place::Town(w.settlements[0].id)).expect("someone at home in town");
    talk_to(&mut w, local);
    let t = w.topics();
    assert!(t.contains(&Topic::ThisTown) && !t.contains(&Topic::Around) && !t.contains(&Topic::Headed), "{t:?}");
    w.end_talk();
    // Out on the road, with travellers.
    let mut found = None;
    for _ in 0..48 {
        w.step(1800.0);
        found = (0..w.people.len() as u32).find(|&p| {
            let pp = &w.people[p as usize];
            !pp.dead && !pp.bandit && !pp.in_squad && w.group_of[p as usize].and_then(|g| w.group(g)).is_some_and(|g| !g.hostile) && w.place_at(w.person_pos(p)).town().is_none()
        });
        if found.is_some() {
            break;
        }
    }
    let traveller = found.expect("someone on the road");
    talk_to(&mut w, traveller);
    let t = w.topics();
    println!("on the road ({:?}): {t:?}", w.place_at(w.person_pos(traveller)));
    assert!(t.contains(&Topic::Headed) && t.contains(&Topic::Around) && !t.contains(&Topic::ThisTown), "{t:?}");
    let where_to = w.ask(Topic::Headed).unwrap();
    let round = w.ask(Topic::Around).unwrap();
    println!("headed: {where_to}\naround: {round}");
    assert!(!where_to.is_empty() && !where_to.starts_with("Hm.") && !round.starts_with("Hm."));
}
