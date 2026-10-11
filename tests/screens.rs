//! The screens agent's sim side: what the squad knows of the world, for the
//! map (#154 UI-Map). A camp is on the map once seen or heard of.

use gahturiyu_sim::sim::{dialogue::Topic, geo::V2, known::Known, person::PersonId, worldgen, World};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

#[test]
fn no_camp_is_known_at_the_start() {
    let w = worldgen::generate(1);
    assert!(!w.camps.is_empty());
    let known: Vec<_> = w.camps.iter().filter(|c| c.known != Known::No).collect();
    assert!(known.is_empty(), "{} camps known before anyone looked", known.len());
}

#[test]
fn a_camp_is_seen_by_whoever_goes_near_it() {
    let mut w = worldgen::generate(1);
    let c = w.camps[0].clone();
    // The rest of the squad far off; one member walks within sight of it.
    w.teleport_squad(c.pos.add(V2::new(2400.0, 0.0)));
    w.step(0.5);
    assert_eq!(w.camp_known(c.group), Known::No);
    let k = 0;
    w.squad.at[k] = c.pos.add(V2::new(120.0, 0.0));
    w.squad.goal[k] = w.squad.at[k];
    w.squad.route[k].clear();
    w.step(0.5);
    assert_eq!(w.camp_known(c.group), Known::Seen);
}

#[test]
fn asking_about_bandits_puts_their_camp_on_the_map() {
    let mut w = worldgen::generate(1);
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().id;
    // Someone at home and awake, by day.
    while (w.time / 3600.0) % 24.0 < 10.0 {
        w.step(60.0);
    }
    let npc: PersonId = *w.settlements[town as usize].residents.iter().find(|&&p| !w.is_indoors_asleep(p) && !w.people[p as usize].dead).unwrap();
    let lead = w.squad.members[0];
    let at = w.person_pos(npc);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    w.squad.at[0] = at.add(V2::new(1.0, 0.0));
    assert!(w.order_talk(lead, npc));
    walk(&mut w, 2.0);
    assert!(w.talk.is_some(), "the conversation opens");
    let before = w.camps.iter().filter(|c| c.known != Known::No).count();
    let said = w.ask(Topic::Bandits).unwrap_or_default();
    let after = w.camps.iter().filter(|c| c.known != Known::No).count();
    if said.contains("camp") || said.contains("walk") {
        assert_eq!(after, before + 1, "told of a camp ({said}), but it isn't on the map");
        assert!(w.camps.iter().any(|c| c.known == Known::Heard));
    } else {
        assert_eq!(after, before, "told of no camp ({said})");
    }
}

#[test]
fn what_the_squad_knows_is_kept_in_a_save() {
    let mut w = worldgen::generate(1);
    let g = w.camps[1].group;
    let c = w.camps[1].pos;
    w.teleport_squad(c.add(V2::new(100.0, 0.0)));
    w.step(0.5);
    assert_eq!(w.camp_known(g), Known::Seen);
    let path = std::env::temp_dir().join(format!("screens_known_{}.sav", std::process::id()));
    w.save_to(&path).unwrap();
    let back = World::load_from(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(back.camp_known(g), Known::Seen);
}
