//! Weather in the rules, part 2 (`docs/weather-hooks.md`): a fire or a
//! standing torch out in the open is drowned by the first downpour, worked
//! out from the weather when it's lit; the dawn boats stay in when the sea
//! won't let them out.

use gahturiyu_sim::sim::{
    geo::{is_land, V2, WORLD_SIZE},
    items,
    torch::DROWNING_RAIN,
    world::HOUR,
    worldgen,
};

#[test]
fn a_fire_in_the_open_is_drowned_by_a_downpour() {
    let mut w = worldgen::generate(1);
    let start = (w.time / HOUR).ceil() * HOUR;
    // A spot on land and an hour of downpour in the next few days.
    let mut found = None;
    'search: for h in 2..(24 * 30) {
        for i in 1..10 {
            for j in 1..10 {
                let p = V2::new(i as f32 / 10.0 * WORLD_SIZE, j as f32 / 10.0 * WORLD_SIZE);
                if is_land(p) && !w.terrain.is_sea(p) && w.weather_at(p, start + h as f64 * HOUR).rain >= DROWNING_RAIN {
                    found = Some((p, start + h as f64 * HOUR));
                    break 'search;
                }
            }
        }
    }
    let (p, rain_at) = found.expect("a downpour somewhere");
    // A fire lit a little before it goes out when it comes.
    let lit = rain_at - 1.5 * HOUR;
    let out = w.drowned_by(p, lit, lit + 4.0 * HOUR);
    assert!(out <= rain_at && out > lit, "drowned at the downpour ({} h before it)", (rain_at - out) / HOUR);
    // In dry weather it burns its full time.
    let dry = (0..24 * 6).map(|h| start + h as f64 * HOUR).find(|&t| (0..5).all(|k| w.weather_at(p, t + k as f64 * HOUR).rain < 0.1)).expect("a dry spell");
    assert_eq!(w.drowned_by(p, dry, dry + 4.0 * HOUR), dry + 4.0 * HOUR);
    // And a campfire laid now gets its end the same way.
    let m = w.squad.members[0];
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("timber"), 2);
    let k = w.squad.index(m).unwrap();
    let at = w.squad.at[k].add(V2::new(2.0, 0.0));
    w.order_campfire(m, at).unwrap();
    for _ in 0..40 {
        w.step(30.0);
    }
    let f = w.campfires.last().expect("laid");
    assert_eq!(f.out_at, w.drowned_by(f.pos, f.lit_at, f.lit_at + gahturiyu_sim::sim::campfire::BURN_HOURS * HOUR));
}

#[test]
fn the_boats_stay_in_when_the_sea_wont_let_them_out() {
    let mut w = worldgen::generate(1);
    // A coastal town and an hour when the boats can't sail.
    let start = (w.time / HOUR).ceil() * HOUR;
    let mut found = None;
    'search: for h in 0..(24 * 10) {
        for s in &w.settlements {
            let Some(p) = s.stilts else { continue };
            if !w.weather_effects(p, start + h as f64 * HOUR).boats_can_sail && (6..18).contains(&(((start / HOUR) as i64 + h) % 24)) {
                found = Some((s.id, start + h as f64 * HOUR));
                break 'search;
            }
        }
    }
    let Some((town, t)) = found else {
        eprintln!("no rough sea in ten days here; nothing to check");
        return;
    };
    while w.time < t + 0.5 * HOUR {
        w.step(600.0f64.min(t + 0.5 * HOUR - w.time));
    }
    let tl = &w.society.towns[town as usize];
    let ci = tl.stilts.expect("a stilt village") as usize;
    assert_eq!(w.society.communities[ci].catch.rate, 0.0, "nobody out fishing in that sea");
}

#[test]
fn outdoor_work_stops_in_a_storm_and_indoor_work_goes_on() {
    use gahturiyu_sim::sim::{jobs::Job, routine::Doing};
    let w = worldgen::generate(1);
    let today = gahturiyu_sim::sim::World::day_of(w.time);
    // A town and a day with storm hours in working time, and someone of an
    // outdoor trade who works that day.
    let mut found = None;
    'search: for day in today..today + 30 {
        for s in &w.settlements {
            let mask = w.storm_hours(s.id, day);
            if mask & 0b0000_0000_0011_1111_1100_0000 == 0 {
                continue;
            }
            for &p in &s.residents {
                if w.life(p).job == Job::Farmer {
                    let plan = w.day_plan(p, day);
                    if let Some((ws, we)) = plan.work {
                        if (ws.ceil() as u32..we.floor() as u32).any(|h| mask & (1 << h) != 0) {
                            found = Some((p, day, mask, ws, we));
                            break 'search;
                        }
                    }
                }
            }
        }
    }
    let (p, day, mask, ws, we) = found.expect("a stormy working day somewhere in a month");
    let plan = w.day_plan(p, day);
    for h in ws.ceil() as u32..we.floor() as u32 {
        let hf = h as f32 + 0.5;
        let doing = plan.segs()[plan.index_at(hf)].doing;
        if mask & (1 << h) != 0 {
            assert_ne!(doing, Doing::Work, "out in the storm at {h}:30");
        }
    }
    assert!(plan.hours_of(Doing::Home, ws, we) > 0.0, "home through the storm");
}
