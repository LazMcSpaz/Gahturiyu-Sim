//! #171: townsfolk walk round buildings and through doors, as the squad does
//! (Laz: "Do it now"). The buildings agent, batch 9.

use gahturiyu_sim::sim::{routine::Doing, world::DAY, worldgen};

/// Walking between the parts of their day, nobody passes through a house
/// that isn't where they're coming from or going to.
#[test]
fn townsfolk_walk_round_buildings() {
    let mut w = worldgen::generate(1);
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().id;
    let folk = w.settlements[town as usize].residents.clone();
    let (mut walking, mut through) = (0, 0);
    let t0 = std::time::Instant::now();
    let mut asked = 0;
    for _ in 0..(DAY / 120.0) as usize {
        w.step(120.0);
        for &p in &folk {
            if !matches!(w.doing_now(p), Some((Doing::Walking, _))) {
                continue;
            }
            walking += 1;
            asked += 1;
            let at = w.person_pos(p);
            if let Some(b) = w.building_at(at) {
                // Their own home, or where they're headed, is fine; anyone
                // else's walls aren't.
                let own = w.people[p as usize].dwelling.is_some_and(|d| (town, d) == b.id);
                if !own {
                    through += 1;
                }
            }
        }
    }
    let per = t0.elapsed().as_secs_f64() / asked.max(1) as f64;
    eprintln!("{walking} walking samples, {through} inside someone else's walls; {:.1} µs a look-up", per * 1e6);
    assert!(walking > 100, "too few walkers seen: {walking}");
    assert!((through as f64) < walking as f64 * 0.02, "{through} of {walking} walking samples were inside someone else's house");
}
