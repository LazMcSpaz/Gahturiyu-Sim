//! The morning's features as Laz meets them (13 Oct QA): a weather forced
//! from the dev panel slows the squad as a real one would; a rule flipped
//! shows at once, paused or not; a save from an older version is told
//! apart without loading it.

use gahturiyu_sim::sim::{save, weather::Sky, worldgen};

fn blizzard() -> Sky {
    let mut k = Sky { cloud: 1.0, precip: 1.0, wind: 26.0, gust: 0.9, temp0: -6.0, wetness: 1.0, cloud_base: 400.0, storm: 1.0, ..Default::default() };
    k.wind_x = k.wind;
    k.snow_lying = -200.0;
    k
}

#[test]
fn a_forced_storm_slows_the_squad() {
    let mut w = worldgen::generate(1);
    // Off the road, where the weather counts in full.
    let m = w.squad.members[0];
    let off_road = (0..40).map(|k| w.squad.pos.add(gahturiyu_sim::sim::geo::V2::new(30.0 + k as f32 * 25.0, 40.0))).find(|&p| !w.terrain.on_road(p) && !w.terrain.is_sea(p)).expect("open ground");
    w.teleport_squad(off_road);
    w.set_forced_sky(Some(Sky { cloud: 0.05, wind: 2.0, temp0: 15.0, cloud_base: 1500.0, ..Default::default() }));
    let calm = w.member_speed(m);
    w.set_forced_sky(Some(blizzard()));
    let storm = w.member_speed(m);
    assert!(storm < calm * 0.95, "a forced blizzard slows them: {storm} vs {calm}");
    let at = w.person_pos(m);
    assert!(w.weather_pace(at, w.time) < 0.95);
    // Cleared, the real weather again (as the terrain reads it).
    w.set_forced_sky(None);
    assert_eq!(w.weather_pace(at, w.time), w.terrain.weather_pace(at, w.time));
}

#[test]
fn a_rule_flipped_shows_at_once_with_time_paused() {
    let mut w = worldgen::generate(1);
    let end = w.time + 12.0 * 3600.0;
    while w.time < end {
        w.step(600.0);
    }
    let m = w.squad.members[0];
    assert!(w.people[m as usize].cond.as_ref().unwrap().thirst_at(w.time) > 0.0);
    // The window's call when a switch is flipped, no step between.
    w.rules.thirst = false;
    w.rules_step();
    assert_eq!(w.people[m as usize].cond.as_ref().unwrap().thirst_at(w.time), 0.0);
}

#[test]
fn an_older_save_is_told_apart_by_its_header() {
    let w = worldgen::generate(5);
    let dir = std::env::temp_dir().join(format!("gaht-morning-qa-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (now, old, junk) = (dir.join("now.sav"), dir.join("old.sav"), dir.join("junk.sav"));
    w.save_to(&now).unwrap();
    let mut bytes = std::fs::read(&now).unwrap();
    bytes[4] = bytes[4].wrapping_sub(1);
    std::fs::write(&old, &bytes).unwrap();
    std::fs::write(&junk, b"hi").unwrap();
    assert!(save::loadable(&now));
    assert!(!save::loadable(&old), "an older format");
    assert!(!save::loadable(&junk), "not a save");
    assert!(!save::loadable(&dir.join("missing.sav")));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_forced_downpour_puts_out_what_burns_in_the_open() {
    use gahturiyu_sim::sim::campfire::Campfire;
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let at = w.person_pos(m);
    w.set_forced_sky(Some(Sky { cloud: 0.05, wind: 2.0, temp0: 15.0, cloud_base: 1500.0, ..Default::default() }));
    // A fire the squad laid (it took five minutes to catch), and one from the dev panel.
    let t = w.time;
    w.campfires.push(Campfire { id: 900, pos: at.add(gahturiyu_sim::sim::geo::V2::new(3.0, 0.0)), by: m, since: t - 300.0, lit_at: t, out_at: t + 4.0 * 3600.0 });
    let dev = w.dev_fire(at);
    assert!(w.dev_torch(m));
    w.step(1.0);
    let t = w.time;
    assert!(w.campfire(900).unwrap().burning(t), "the squad's fire burns");
    assert!(w.standing.iter().any(|s| s.burning(t)), "the torch burns");
    let rain = Sky { cloud: 1.0, precip: 1.0, wind: 8.0, temp0: 10.0, wetness: 1.0, storm: 0.3, cloud_base: 400.0, ..Default::default() };
    w.set_forced_sky(Some(rain));
    w.step(1.0);
    let t = w.time;
    assert!(!w.campfire(900).unwrap().burning(t), "the downpour puts the squad's fire out");
    assert!(!w.standing.iter().any(|s| s.burning(t)), "and the torch");
    assert!(w.campfire(dev).unwrap().burning(t), "the dev panel's fire burns whatever the weather");
}

#[test]
fn a_forced_blizzard_chills_from_the_moment_it_is_forced() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let off_road = (0..40).map(|k| w.squad.pos.add(gahturiyu_sim::sim::geo::V2::new(60.0 + k as f32 * 25.0, -50.0))).find(|&p| !w.terrain.on_road(p) && !w.terrain.is_sea(p)).expect("open ground");
    w.teleport_squad(off_road);
    w.set_forced_sky(Some(Sky { cloud: 0.05, wind: 2.0, temp0: 18.0, cloud_base: 1500.0, ..Default::default() }));
    w.step(1.0);
    let before = w.cold_of(m).unwrap();
    // Forced a few minutes into an hour, ten minutes before the next.
    let hour = 3600.0;
    let into = w.time.rem_euclid(hour);
    if into > hour - 900.0 {
        w.step(hour - into + 60.0);
    }
    w.set_forced_sky(Some(blizzard()));
    w.step(600.0);
    let after = w.cold_of(m).unwrap();
    assert!(after > before + 1.0, "colder within ten minutes: {before} -> {after}");
}
