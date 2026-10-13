//! The rules toggles (`rules.rs`): all on by default, saved with the world,
//! and each system answers "no" at its decision point when its rule is off.

use gahturiyu_sim::sim::condition::CHILLED;
use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::island::Spot;
use gahturiyu_sim::sim::rules::{Rules, RULES};
use gahturiyu_sim::sim::world::HOUR;
use gahturiyu_sim::sim::worldgen;

#[test]
fn all_on_by_default_and_each_can_be_switched() {
    let mut r = Rules::default();
    assert!(r.all_on());
    for rule in RULES {
        assert_eq!(r.get(rule.key), Some(true), "{}", rule.key);
        r.toggle(rule.key);
        assert_eq!(r.get(rule.key), Some(false));
        r.toggle(rule.key);
    }
    assert!(r.all_on());
    r.apply("potions_break=off, thirst=0,bleeding=no,nonsense=off,toxicity=maybe");
    assert_eq!((r.potions_break, r.thirst, r.bleeding, r.toxicity), (false, false, false, true));
    assert!(!r.set("nonsense", true));
    // The world carries them, and a save keeps them.
    let mut w = worldgen::generate(5);
    assert!(w.rules.all_on());
    w.rules.set("gear_wears", false);
    let back = gahturiyu_sim::sim::World::load_bytes(&w.save_bytes()).unwrap();
    assert_eq!(back.rules, w.rules);
    assert!(!back.rules.gear_wears);
}

#[test]
fn weather_off_the_squad_leaves_them_warm_and_quick() {
    // The opening: wrecked in a storm, chilled.
    let mut w = worldgen::generate_opening(5, None, Default::default());
    let m = w.squad.members[0];
    let shore = w.island().unwrap().spot(Spot::Shore);
    assert!(!w.weather_effects(shore, w.time).outdoor_work_ok || w.time == 6.0 * HOUR);
    let storm = !w.weather_effects(shore, w.time).outdoor_work_ok;
    // With the rule on, standing in the storm they head for the cold; the walking pace suffers.
    w.step(0.5);
    let aiming_cold = w.people[m as usize].cond.as_ref().unwrap().cold_to;
    let pace_on = w.member_speed(m);
    // Rule off: nothing of the weather reaches them.
    w.rules.weather_on_squad = false;
    w.teleport_squad(shore.add(V2::new(1.0, 0.0)));
    w.step(0.5);
    let c = w.people[m as usize].cond.as_ref().unwrap();
    assert_eq!(c.cold_to, 0.0, "no cold to head for with the rule off (was heading for {aiming_cold})");
    let pace_off = w.member_speed(m);
    if storm {
        assert!(pace_off >= pace_on, "no weather on the pace: {pace_off} vs {pace_on}");
    }
    // They were chilled from the sea; that drains away now rather than deepening.
    for _ in 0..240 {
        w.step(30.0);
    }
    let c = w.people[m as usize].cond.as_ref().unwrap();
    assert!(c.cold_at(w.time) < CHILLED, "warming up out of the weather: {}", c.cold_at(w.time));
}

#[test]
fn weather_off_fires_leaves_a_torch_to_burn_its_hours() {
    let mut w = worldgen::generate(5);
    let pos = w.squad.pos;
    // Some hour with a downpour, if the seed has one in the next days.
    let mut wet = None;
    let mut h = (w.time / HOUR).floor() * HOUR;
    while h < w.time + 240.0 * HOUR {
        if w.weather_at(pos, h).rain >= gahturiyu_sim::sim::torch::DROWNING_RAIN {
            wet = Some(h);
            break;
        }
        h += HOUR;
    }
    let Some(wet) = wet else { return };
    let from = wet - HOUR;
    assert!(w.drowned_by(pos, from, from + 6.0 * HOUR) < from + 6.0 * HOUR, "drowned with the rule on");
    w.rules.weather_on_things = false;
    assert_eq!(w.drowned_by(pos, from, from + 6.0 * HOUR), from + 6.0 * HOUR, "burns its hours with the rule off");
}

#[test]
fn a_forced_sky_reaches_every_rule_and_a_dev_fire_burns() {
    use gahturiyu_sim::sim::weather::{sky, Region};
    let mut w = worldgen::generate(5);
    let pos = w.squad.pos;
    // A downpour forced: the weather at any spot says so, and the effects follow.
    let mut storm = sky(w.seed, Region::Lowland, 0.0);
    storm.precip = 1.0;
    storm.cloud = 1.0;
    storm.temp0 = 12.0;
    storm.storm = 1.0;
    w.set_forced_sky(Some(storm));
    for p in [pos, pos.add(V2::new(5000.0, 3000.0)), pos.add(V2::new(-1500.0, 8000.0))] {
        assert!(w.weather_at(p, w.time).rain >= 0.8, "forced rain at {p:?}: {}", w.weather_at(p, w.time).rain);
        assert!(!w.weather_effects(p, w.time).outdoor_work_ok);
    }
    // A real fire lit in the forced downpour goes out at once; the dev fire burns its hours anyway.
    let at = pos.add(V2::new(3.0, 0.0));
    assert!(w.drowned_by(at, w.time, w.time + 4.0 * HOUR) <= w.time + 1.0, "a real fire is drowned by the forced rain");
    let id = w.dev_fire(at);
    let f = *w.campfire(id).unwrap();
    assert!(f.burning(w.time) && f.burning(w.time + 3.5 * HOUR) && w.fire_at(at.add(V2::new(1.5, 0.0))).is_some());
    // Cleared: the seed's own weather again.
    w.set_forced_sky(None);
    assert_eq!(w.weather_at(pos, w.time).rain, worldgen::generate(5).weather_at(pos, w.time).rain);
    // Not saved: a loaded world has its own weather.
    w.set_forced_sky(Some(storm));
    let back = gahturiyu_sim::sim::World::load_bytes(&w.save_bytes()).unwrap();
    assert!(back.forced_sky.is_none());
}
