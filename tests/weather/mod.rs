//! Weather: looked up from the place and the time, never run.
//!
//! 1. The same seed, place and time give the same weather, whatever was
//!    asked before, and whether the world was stepped there or not.
//! 2. It changes smoothly, over time and across the map (lightning aside).
//! 3. The climate comes out as asked: the exposed coast mostly grey, storms
//!    a few days a month and more in winter, thick fog on several mornings a
//!    month and more in summer; snow in the mountains in winter, hardly ever
//!    on the shore.

use gahturiyu_sim::sim::geo::{V2, WORLD_SIZE};
use gahturiyu_sim::sim::terrain::Terrain;
use gahturiyu_sim::sim::weather::{self, report, Kind, Maker, Region, Sky, Weather};
use gahturiyu_sim::sim::world::{DAY, HOUR};
use gahturiyu_sim::sim::worldgen;

/// A spread of places over the whole map, sea included.
fn spots() -> Vec<V2> {
    let mut v = Vec::new();
    for j in 0..7 {
        for i in 0..7 {
            v.push(V2::new(700.0 + i as f32 * 3250.0, 900.0 + j as f32 * 3200.0));
        }
    }
    v
}

#[test]
fn weather_is_the_same_whatever_order_it_is_asked_in() {
    let terrain = Terrain::generate(5);
    let times = [0.0, 3.3 * DAY, 17.0 * DAY + 5.2 * HOUR, 100.7 * DAY, 4000.25 * DAY];
    let asks: Vec<(V2, f64)> = spots().into_iter().flat_map(|p| times.iter().map(move |&t| (p, t))).collect();
    let forward: Vec<Weather> = asks.iter().map(|&(p, t)| weather::weather_at(&terrain, 5, p, t)).collect();
    // Backwards, with other questions asked in between.
    let mut backward: Vec<Weather> = asks
        .iter()
        .rev()
        .map(|&(p, t)| {
            let _ = weather::weather_at(&terrain, 5, V2::new(p.y, p.x), t + 7.0 * DAY);
            let _ = weather::forecast(5, Region::Mountain, t - DAY, 5);
            weather::weather_at(&terrain, 5, p, t)
        })
        .collect();
    backward.reverse();
    assert_eq!(forward, backward, "asking in a different order changed the weather");
    // The same world built again has the same weather.
    let again = Terrain::generate(5);
    for (k, &(p, t)) in asks.iter().enumerate().step_by(7) {
        assert_eq!(forward[k], weather::weather_at(&again, 5, p, t));
    }
    // Asking about many places at once (the skies worked out first) gives the same answers.
    for &t in &times {
        let skies = weather::skies(5, t);
        for &p in &spots() {
            let quick = weather::weather_with(&skies, &weather::mix(&terrain, p), terrain.surface(p), weather::floor(&terrain, p));
            assert_eq!(quick, weather::weather_at(&terrain, 5, p, t));
        }
    }
    // Another world has its own.
    let other = Terrain::generate(6);
    let differs = asks.iter().enumerate().filter(|(k, &(p, t))| forward[*k] != weather::weather_at(&other, 6, p, t)).count();
    assert!(differs > asks.len() * 9 / 10, "another seed should have other weather");
    // Somewhere in all that there was actual weather, not one flat answer.
    let kinds: std::collections::HashSet<Kind> = forward.iter().map(|w| w.kind).collect();
    assert!(kinds.len() >= 4, "only {kinds:?}");
}

#[test]
fn stepping_the_world_does_not_change_the_weather() {
    let untouched = worldgen::generate(9);
    let mut hourly = worldgen::generate(9);
    let mut lumpy = worldgen::generate(9);
    let places = spots();
    let ahead: Vec<Vec<Weather>> = Region::ALL.iter().map(|&r| untouched.forecast(r, untouched.time, 31)).collect();
    for hour in 0..30 {
        for &p in places.iter().step_by(3) {
            let asked = untouched.weather_at(p, hourly.time);
            assert_eq!(hourly.weather(p), asked, "hour {hour}: the stepped world has different weather");
            assert_eq!(lumpy.weather(p), asked, "hour {hour}: the world stepped unevenly has different weather");
        }
        // Yesterday's forecast is today's weather.
        for (k, &r) in Region::ALL.iter().enumerate() {
            assert_eq!(ahead[k][hour], hourly.forecast(r, hourly.time, 1)[0], "the forecast for hour {hour} was wrong");
        }
        hourly.step(HOUR);
        for _ in 0..8 {
            lumpy.step(HOUR / 8.0);
        }
        assert!((hourly.time - lumpy.time).abs() < 1e-6);
    }
}

/// The biggest change allowed between two looks ten minutes (or 20 m) apart.
struct Limits {
    cloud: f32,
    falling: f32,
    wind: f32,
    fog: f32,
    temperature: f32,
    wetness: f32,
    storm: f32,
    sea: f32,
    /// As a ratio: sight may shrink or grow by this many times.
    sight: f32,
}

fn assert_close(a: &Weather, b: &Weather, lim: &Limits, what: &str) {
    assert!((a.cloud - b.cloud).abs() <= lim.cloud, "{what}: cloud jumped {} -> {}", a.cloud, b.cloud);
    assert!((a.rain + a.snow - b.rain - b.snow).abs() <= lim.falling, "{what}: rain or snow jumped {} -> {}", a.rain + a.snow, b.rain + b.snow);
    assert!((a.wind - b.wind).abs() <= lim.wind, "{what}: wind jumped {} -> {}", a.wind, b.wind);
    assert!((a.fog - b.fog).abs() <= lim.fog, "{what}: fog jumped {} -> {}", a.fog, b.fog);
    assert!((a.temperature - b.temperature).abs() <= lim.temperature, "{what}: temperature jumped {} -> {}", a.temperature, b.temperature);
    assert!((a.wetness - b.wetness).abs() <= lim.wetness, "{what}: wetness jumped {} -> {}", a.wetness, b.wetness);
    assert!((a.storm - b.storm).abs() <= lim.storm, "{what}: the storm jumped {} -> {}", a.storm, b.storm);
    assert!((a.sea - b.sea).abs() <= lim.sea, "{what}: the sea jumped {} -> {}", a.sea, b.sea);
    let ratio = (a.visibility / b.visibility).max(b.visibility / a.visibility);
    assert!(ratio <= lim.sight, "{what}: sight jumped {} -> {}", a.visibility, b.visibility);
}

#[test]
fn weather_changes_smoothly_over_time() {
    // Ten minutes apart, for two years, in every region. Fog rolling in is
    // the fastest thing there is: sight can go from miles to yards in an hour.
    let lim = Limits { cloud: 0.1, falling: 0.22, wind: 4.0, fog: 0.45, temperature: 0.6, wetness: 0.18, storm: 0.08, sea: 0.09, sight: 36.0 };
    for seed in [1u64, 2] {
        for r in Region::ALL {
            let mut prev = weather::weather_in(seed, r, 0.0);
            for k in 1..(96 * 144) {
                let w = weather::weather_in(seed, r, k as f64 * 600.0);
                assert_close(&prev, &w, &lim, &format!("{} at {:.2} days", r.name(), k as f64 / 144.0));
                prev = w;
            }
        }
    }
    // And at real places (where height and hollows come into it), across midnight too.
    let terrain = Terrain::generate(2);
    for &p in spots().iter().step_by(5) {
        let mut prev = weather::weather_at(&terrain, 2, p, 20.0 * DAY);
        for k in 1..(6 * 144) {
            let w = weather::weather_at(&terrain, 2, p, 20.0 * DAY + k as f64 * 600.0);
            assert_close(&prev, &w, &lim, &format!("({}, {}) at step {k}", p.x, p.y));
            prev = w;
        }
    }
}

#[test]
fn weather_changes_smoothly_across_region_borders() {
    // Twenty metres apart, right across the map. Temperature follows the
    // ground's height, so it is allowed what a steep slope gives it; and fog
    // has a top, so a steep slope can climb out of it in a few paces.
    let lim = Limits { cloud: 0.2, falling: 0.15, wind: 3.0, fog: 0.6, temperature: 1.6, wetness: 0.2, storm: 0.2, sea: 0.2, sight: 60.0 };
    let terrain = Terrain::generate(1);
    for t in [30.3 * DAY + 7.0 * HOUR, 41.0 * DAY + 13.0 * HOUR, 12.2 * DAY] {
        for y in [1500.0f32, 4000.0, 8000.0, 12_600.0, 15_000.0, 19_000.0] {
            let mut x = 500.0;
            let mut prev = weather::weather_at(&terrain, 1, V2::new(x, y), t);
            let mut prev_mix = weather::mix(&terrain, V2::new(x, y));
            while x < WORLD_SIZE - 500.0 {
                x += 20.0;
                let p = V2::new(x, y);
                let w = weather::weather_at(&terrain, 1, p, t);
                let m = weather::mix(&terrain, p);
                assert_close(&prev, &w, &lim, &format!("({x}, {y})"));
                for k in 0..weather::REGIONS {
                    assert!((m[k] - prev_mix[k]).abs() < 0.25, "({x}, {y}): a region border is a hard line");
                }
                assert!((m.iter().sum::<f32>() - 1.0).abs() < 1e-4, "region shares should add up to 1");
                prev = w;
                prev_mix = m;
            }
        }
    }
}

#[test]
fn every_kind_of_country_is_on_the_map() {
    let terrain = Terrain::generate(1);
    let mut seen = [0u32; weather::REGIONS];
    let mut n = 0;
    for j in 0..70 {
        for i in 0..70 {
            let p = V2::new(150.0 + i as f32 * 300.0, 150.0 + j as f32 * 300.0);
            seen[weather::region_at(&terrain, p) as usize] += 1;
            n += 1;
        }
    }
    for r in Region::ALL {
        assert!(seen[r as usize] as f32 / n as f32 > 0.015, "hardly any {} on the map", r.name());
    }
    // Far out is open sea; the high north is mountain; the south-east is the plateau.
    assert_eq!(weather::region_at(&terrain, V2::new(300.0, 10_000.0)), Region::OpenSea);
    assert_eq!(weather::region_at(&terrain, V2::new(10_000.0, 800.0)), Region::Mountain);
    assert_eq!(weather::region_at(&terrain, V2::new(14_500.0, 14_500.0)), Region::Plateau);
}

#[test]
fn the_exposed_coast_is_mostly_grey() {
    // Laz: like Scotland. Overcast and drizzle are normal, clear days feel
    // special, storms a few times a month.
    for seed in [1u64, 2, 3] {
        let days = 30 * 48;
        let year = report::tally(seed, Region::ExposedCoast, 0, days, None);
        let summer = report::tally(seed, Region::ExposedCoast, 0, days, Some(1));
        let winter = report::tally(seed, Region::ExposedCoast, 0, days, Some(3));
        assert!((0.50..=0.60).contains(&year.grey), "world {seed}: grey or wet for {:.0}% of daylight", year.grey * 100.0);
        assert!((0.10..=0.20).contains(&year.clear), "world {seed}: clear for {:.0}% of daylight", year.clear * 100.0);
        assert!((2.0..=5.0).contains(&year.storm_days), "world {seed}: {:.1} storm days a month", year.storm_days);
        assert!(winter.storm_days > summer.storm_days * 1.5, "world {seed}: storms should be a winter thing ({:.1} vs {:.1})", winter.storm_days, summer.storm_days);
        assert!((4.0..=8.0).contains(&year.fog_mornings), "world {seed}: {:.1} thick-fog mornings a month", year.fog_mornings);
        assert!(summer.fog_mornings > winter.fog_mornings * 1.3, "world {seed}: sea fog should be a summer thing ({:.1} vs {:.1})", summer.fog_mornings, winter.fog_mornings);
        assert!(winter.grey > summer.grey + 0.1, "world {seed}: winter should be greyer than summer");
    }
}

#[test]
fn snow_belongs_to_the_mountains_in_winter() {
    for seed in [1u64, 2, 3] {
        let days = 30 * 48;
        let high_winter = report::tally(seed, Region::Mountain, 0, days, Some(3));
        let high_summer = report::tally(seed, Region::Mountain, 0, days, Some(1));
        let coast = report::tally(seed, Region::ExposedCoast, 0, days, None);
        let coast_summer = report::tally(seed, Region::ExposedCoast, 0, days, Some(1));
        assert!(high_winter.snow_days > 10.0, "world {seed}: only {:.1} snowy days a month in the winter mountains", high_winter.snow_days);
        assert!(high_winter.snow_days > 5.0 * high_summer.snow_days + 5.0, "world {seed}: mountain snow should be a winter thing");
        assert!(coast.snow_days < 1.0, "world {seed}: {:.1} snowy days a month on the coast is not rare", coast.snow_days);
        assert_eq!(coast_summer.snow_days, 0.0, "world {seed}: snow on the summer coast");
        // The plateau is the dry country.
        let plateau = report::tally(seed, Region::Plateau, 0, days, None);
        assert!(plateau.clear > 0.35 && plateau.raining < 0.1, "world {seed}: the plateau should be dry and clear");
    }
}

#[test]
fn fog_pools_in_the_low_ground_and_thick_fog_hides_everything() {
    let sky = Sky { fog: 1.0, fog_depth: 30.0, cloud: 0.3, cloud_base: 900.0, temp0: 10.0, wind: 1.0, wind_x: 1.0, ..Default::default() };
    let low = weather::localise(&sky, Region::Lowland, 40.0, 40.0);
    let rise = weather::localise(&sky, Region::Lowland, 58.0, 40.0);
    let hill = weather::localise(&sky, Region::Lowland, 110.0, 40.0);
    assert!(low.fog > 0.95 && low.visibility < 40.0 && low.kind == Kind::ThickFog, "the valley floor should be in thick fog: {low:?}");
    assert!(rise.fog < low.fog && rise.fog > 0.0, "the fog should thin towards its top");
    assert!(hill.fog == 0.0 && hill.visibility > 10_000.0, "the hill should stand clear of it");
    // High ground under a low cloud base is in hill fog.
    let grey = Sky { cloud: 0.95, cloud_base: 300.0, temp0: 10.0, ..Default::default() };
    assert_eq!(weather::localise(&grey, Region::Mountain, 500.0, 200.0).kind, Kind::HillFog);
    assert_eq!(weather::localise(&grey, Region::Mountain, 150.0, 150.0).kind, Kind::Overcast);
    // And it does get that thick: sight down to about 30 m at the worst.
    let worst = (0..(96 * 72)).map(|k| weather::weather_in(1, Region::ExposedCoast, k as f64 * 1200.0).visibility).fold(f32::MAX, f32::min);
    assert!((25.0..45.0).contains(&worst), "the thickest fog in two years left {worst} m of sight");
    // Height makes it colder, and turns rain to snow.
    let wet = Sky { precip: 0.5, cloud: 1.0, cloud_base: 2000.0, temp0: 4.0, ..Default::default() };
    let shore = weather::localise(&wet, Region::Mountain, 0.0, 0.0);
    let top = weather::localise(&wet, Region::Mountain, 800.0, 700.0);
    assert!(shore.rain > 0.45 && shore.snow < 0.02, "rain at the shore");
    assert!(top.snow > 0.45 && top.rain < 0.02 && top.temperature < 0.0, "snow on the tops");
    assert!((shore.snowline - 461.5).abs() < 2.0, "the snowline should sit where the air reaches 1 degree: {}", shore.snowline);
}

#[test]
fn a_storm_brings_wind_first_then_rain_and_goes_dark() {
    // Find a strong storm with quiet days round it.
    let maker = Maker::new(4, Region::ExposedCoast);
    let (day, storm) = (2..2000)
        .find_map(|d| {
            let s = maker.storm_on(d)?;
            let alone = maker.storm_on(d - 1).is_none() && maker.storm_on(d + 1).is_none() && maker.storm_on(d - 2).is_none();
            (alone && s.strength > 0.85).then_some((d, s))
        })
        .expect("a strong storm somewhere in forty years");
    let at = |hours: f32| weather::weather_in(4, Region::ExposedCoast, storm.start + hours as f64 * HOUR);
    let before = at(-8.0);
    let worst = at(storm.build + storm.hold / 2.0);
    let after = at(storm.build + storm.hold + storm.clear + 8.0);
    assert!(worst.wind >= weather::GALE && worst.kind.is_storm(), "day {day}: no gale at the height of the storm: {worst:?}");
    assert!(worst.cloud > 0.9 && worst.rain + worst.snow > 0.6 && worst.storm > 0.8, "day {day}: the storm should be dark and lashing: {worst:?}");
    assert!(worst.visibility < 1500.0 && worst.sea > 0.8, "day {day}: {worst:?}");
    assert!(before.storm < 0.05 && after.storm < 0.05, "the storm should come and go");
    // The wind gets up before the rain comes.
    let first = |test: &dyn Fn(&Weather) -> bool| (-60..200).map(|k| k as f32 / 6.0).find(|&h| test(&at(h))).expect("it should happen");
    let windy = first(&|w| w.wind > before.wind.max(10.0) + 3.0);
    let raining = first(&|w| w.rain + w.snow > 0.3);
    assert!(windy < raining - 0.5, "day {day}: wind at {windy} h, rain at {raining} h");
    // The ground stays wet for hours after the rain has gone.
    let later = at(storm.build + storm.hold + storm.clear + 3.0);
    assert!(later.wetness > 0.3 || later.rain > 0.03, "the ground should still be wet three hours on: {later:?}");
}
