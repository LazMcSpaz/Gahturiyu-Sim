//! Weather: looked up from the place and the time, never run.
//!
//! 1. The same seed, place and time give the same weather, whatever was
//!    asked before, and whether the world was stepped there or not.
//! 2. It changes smoothly, over time and across the map (lightning aside).
//! 3. The climate comes out as asked: the exposed coast mostly grey, storms
//!    a few days a month and more in winter, thick fog on several mornings a
//!    month and more in summer; snow in the mountains in winter, hardly ever
//!    on the shore.
//! 4. The country shares its big weather, and it comes in off the sea: a
//!    storm on the coast is a storm at sea, and reaches the east later.
//! 5. Lightning is looked up too: the same strikes however they are asked for.
//! 6. Omens are real weather, the same however asked, and each comes as
//!    often as its rarity says.
//! 7. What the weather does (`weather_effects`): nothing on a fine day, the
//!    right things in fog, storm and frost; always within bounds.

use gahturiyu_sim::sim::geo::{V2, WORLD_SIZE};
use gahturiyu_sim::sim::terrain::Terrain;
use gahturiyu_sim::sim::weather::{self, report, Effects, Kind, Maker, OmenKind, Region, Sky, Weather};
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
        for &p in &spots() {
            let skies = weather::skies(5, t, weather::lag(p));
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

#[test]
fn the_country_shares_its_big_weather() {
    // Hour by hour for ten years, as the weather comes in off the sea.
    let seed = 3;
    let hours = 48 * 24 * 10;
    let (mut coast_storm, mut sea_too, mut low_too) = (0, 0, 0);
    let (mut dry_storm, mut sea_with_dry) = (0, 0);
    let (mut a, mut b) = (Vec::with_capacity(hours), Vec::with_capacity(hours));
    for h in 0..hours {
        let t = h as f64 * HOUR;
        let at = |r: Region| weather::sky(seed, r, t);
        let (sea, coast, low, dry) = (at(Region::OpenSea), at(Region::ExposedCoast), at(Region::Lowland), at(Region::Plateau));
        if coast.storm >= 0.5 {
            coast_storm += 1;
            sea_too += (sea.storm >= 0.3) as u32;
            low_too += (low.storm >= 0.3) as u32;
        }
        if dry.storm >= 0.5 {
            dry_storm += 1;
            sea_with_dry += (sea.storm >= 0.3) as u32;
        }
        a.push(coast.cloud);
        b.push(low.cloud);
    }
    assert!(coast_storm > 200, "there should be storms to compare");
    // A storm on the coast is a storm at sea, and usually inland too.
    assert!(sea_too as f32 > 0.85 * coast_storm as f32, "the sea shared {sea_too} of {coast_storm} coast storm hours");
    assert!(low_too as f32 > 0.5 * coast_storm as f32, "the lowland shared {low_too} of {coast_storm} coast storm hours");
    // The plateau gets few storms, and none that the sea did not have first.
    assert!(dry_storm > 20 && (dry_storm as f32) < 0.6 * coast_storm as f32, "{dry_storm} plateau storm hours against {coast_storm} on the coast");
    assert!(sea_with_dry as f32 > 0.85 * dry_storm as f32);
    // Grey days are mostly grey for the coast and the country behind it alike.
    let n = a.len() as f32;
    let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
    let cov: f32 = a.iter().zip(&b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let (va, vb): (f32, f32) = (a.iter().map(|x| (x - ma).powi(2)).sum(), b.iter().map(|y| (y - mb).powi(2)).sum());
    let together = cov / (va * vb).sqrt();
    assert!(together > 0.6, "coast and lowland cloud move together only {together}");
}

#[test]
fn weather_comes_in_off_the_sea() {
    let hours = weather::climate().crossing_hours as f64;
    // It reaches the east later than the west, everywhere, and takes about
    // `crossing_hours` over the whole map.
    for row in 0..20 {
        let y = 500.0 + row as f32 * 1000.0;
        let (west, mid, east) = (weather::lag(V2::new(500.0, y)), weather::lag(V2::new(10_000.0, y)), weather::lag(V2::new(20_500.0, y)));
        assert!(west < mid && mid < east && west >= 0.0 && east <= hours * HOUR, "row {y}: {west} {mid} {east}");
        assert!(east - west > 0.6 * hours * HOUR);
    }
    // What a place further east gets is what the sea's edge had that much earlier.
    let behind = 1.3 * HOUR;
    for k in 0..400 {
        let t = k as f64 * 5.3 * HOUR;
        for r in [Region::Lowland, Region::Mountain] {
            let (here, there) = (weather::sky(4, r, t), weather::sky_at(4, r, t + behind, behind));
            // (Bar the hair's breadth the season has moved on in the meantime.)
            assert!((here.cloud - there.cloud).abs() < 0.02 && (here.storm - there.storm).abs() < 0.005 && (here.precip - there.precip).abs() < 0.02, "{}: the weather changed on its way in: {here:?} / {there:?}", r.name());
        }
    }
    // So a storm peaks later inland: find a strong one and time it at two
    // lowland places far apart, west and east.
    let terrain = Terrain::generate(4);
    let lowland: Vec<V2> = (0..40).map(|i| V2::new(3200.0 + i as f32 * 300.0, 10_400.0)).filter(|&p| weather::mix(&terrain, p)[Region::Lowland as usize] > 0.98).collect();
    let (west, east) = (lowland[0], *lowland.last().unwrap());
    assert!(east.x - west.x > 3000.0, "need two lowland places well apart");
    let maker = Maker::new(4, Region::Lowland);
    let storm = (2..3000).find_map(|d| maker.storm_on(d).filter(|s| s.strength > 0.8 && maker.storm_on(d - 1).is_none() && maker.storm_on(d + 1).is_none())).expect("a strong lowland storm");
    let half_up = |p: V2| (0..600).map(|k| storm.start - 4.0 * HOUR + k as f64 * 60.0).find(|&t| weather::weather_at(&terrain, 4, p, t).storm > 0.4).expect("the storm should reach it");
    let apart = half_up(east) - half_up(west);
    let due = weather::lag(east) - weather::lag(west);
    assert!(due > 600.0 && (apart - due).abs() < 180.0, "the storm reached the east {apart} s after the west; due {due} s");
}

#[test]
fn lightning_strikes_are_the_same_however_they_are_asked_for() {
    let terrain = Terrain::generate(6);
    let maker = Maker::new(6, Region::OpenSea);
    let storm = (2..3000).find_map(|d| maker.storm_on(d).filter(|s| s.thunder && s.strength > 0.8)).expect("a thunderstorm");
    let (from, to) = (storm.start + storm.build as f64 * HOUR, storm.start + (storm.build + 1.5) as f64 * HOUR);
    let all = weather::strikes(&terrain, 6, from, to);
    assert!(all.len() > 10 && all.len() < 400, "{} strikes in an hour and a half of thunderstorm", all.len());
    // In pieces, and asked for again: the same strikes.
    let cut = from + 1234.5;
    let mut pieces = weather::strikes(&terrain, 6, from, cut);
    pieces.extend(weather::strikes(&terrain, 6, cut, to));
    assert_eq!(all, pieces, "cutting the question in two changed the lightning");
    assert_eq!(all, weather::strikes(&terrain, 6, from, to));
    // Every strike lands in a thunderstorm, in time order, on the map.
    for w in all.windows(2) {
        assert!(w[0].t <= w[1].t);
    }
    for s in &all {
        assert!(weather::weather_at(&terrain, 6, s.pos, s.t).lightning > 0.0, "lightning out of a sky with no thunder in it");
        assert!((from..to).contains(&s.t) && s.pos.x >= 0.0 && s.pos.x <= WORLD_SIZE);
    }
    // No storm, no lightning: the day before a quiet spell.
    let quiet = (5..3000).find(|&d| (d - 2..=d + 1).all(|k| Region::ALL.iter().all(|&r| Maker::new(6, r).storm_on(k).is_none()))).unwrap();
    assert!(weather::strikes(&terrain, 6, quiet as f64 * DAY, (quiet + 1) as f64 * DAY).is_empty());
}

#[test]
fn snow_lies_on_the_winter_hills() {
    // Midwinter is three quarters of the way through the 48-day year.
    let (mut winter, mut summer, mut shore) = (0, 0, 0);
    for year in 0..20 {
        for day in 0..6 {
            let w = weather::weather_in(2, Region::Mountain, (year * 48 + 33 + day) as f64 * DAY + 12.0 * HOUR);
            let s = weather::weather_in(2, Region::Mountain, (year * 48 + 9 + day) as f64 * DAY + 12.0 * HOUR);
            let c = weather::weather_in(2, Region::ExposedCoast, (year * 48 + 33 + day) as f64 * DAY + 12.0 * HOUR);
            winter += (weather::localise(&weather::sky(2, Region::Mountain, (year * 48 + 33 + day) as f64 * DAY), Region::Mountain, 700.0, 600.0).snow_cover > 0.5) as u32;
            summer += (s.snow_lies_above < 900.0) as u32;
            shore += (c.snow_cover > 0.5) as u32;
            assert!(w.snow_lies_above < s.snow_lies_above, "snow should lie lower in winter");
        }
    }
    assert!(winter > 90, "the high ground was white on only {winter} of 120 midwinter days");
    assert_eq!(summer, 0, "snow on the summer tops");
    assert!(shore <= 2, "the shore was white on {shore} of 120 midwinter days");
}

#[test]
fn each_omen_comes_as_often_as_its_rarity_says() {
    // The bands are in the climate file: times a year, for a typical region
    // the omen can happen in (lightning on a landmark: the whole country).
    let years = 150;
    for seed in [1u64, 3] {
        let world = worldgen::generate(seed);
        let landmarks = world.landmarks();
        assert!(landmarks.len() >= 5, "the towns are the landmarks");
        for (kind, by_region) in weather::omen_rates(seed, 0, years) {
            let rate = if kind == OmenKind::LandmarkStrike { weather::strike_rate(&world.terrain, seed, &landmarks, 0, years) } else { weather::typical_rate(kind, &by_region) };
            let (least, most) = kind.band();
            assert!(rate >= least && rate <= most, "world {seed}: {} came {rate:.2} times a year; it is {}, which means {least} to {most}", kind.name(), kind.rarity().name());
            for r in Region::ALL {
                assert!(kind.happens_in(r) || by_region[r as usize] == 0.0, "{} in the {}", kind.name(), r.name());
            }
        }
    }
    // The rarer the name, the rarer the band.
    let o = &weather::climate().omens;
    assert!(o.very_rare.1 <= o.rare.0 && o.rare.1 <= o.uncommon.0);
}

#[test]
fn omens_are_real_weather_and_the_same_however_asked() {
    let seed = 7;
    let o = &weather::climate().omens;
    let mut seen = std::collections::HashMap::new();
    let days = 48 * 120;
    let mut all = Vec::new();
    for day in 0..days {
        for r in Region::ALL {
            let found = weather::omens(seed, r, day);
            for w in found.windows(2) {
                assert!(w[0].at <= w[1].at, "a day's omens come in time order");
            }
            for om in &found {
                assert_eq!(om.rarity, om.kind.rarity());
                assert_eq!(om.region, r);
                assert!(om.at >= day as f64 * DAY && om.at < (day + 1) as f64 * DAY, "{} on day {day} is timed outside it", om.kind.name());
                assert!(!om.text.is_empty());
                let n = seen.entry(om.kind).or_insert(0usize);
                *n += 1;
                // Look hard at the first few of each kind.
                if *n > 6 {
                    continue;
                }
                let at = |t: f64| weather::weather_in(seed, r, t);
                match om.kind {
                    OmenKind::DeadCalm => {
                        for k in 0..=(o.calm_hours * 2.0) as usize {
                            let w = at(om.at + k as f64 * 0.5 * HOUR);
                            assert!(w.wind < o.calm_wind, "a dead calm with a {} m/s wind", w.wind);
                        }
                    }
                    OmenKind::NoonFog => {
                        assert!(weather::sky(seed, r, om.at).fog >= o.noon_fog_thick);
                        assert!(((om.at / HOUR).rem_euclid(24.0) - 12.0).abs() < 1e-6);
                        assert_eq!(report::season_of(day), 1, "fog at noon in high summer, out of summer");
                    }
                    OmenKind::WinterThunder => {
                        assert_eq!(report::season_of(day), 3, "thunder in midwinter, out of winter");
                        let heard = (0..40).any(|k| at(om.at + k as f64 * 0.25 * HOUR).lightning > 0.05);
                        assert!(heard, "thunder in midwinter with no lightning after it");
                    }
                    OmenKind::ShoreSnow => {
                        let w = weather::localise(&weather::sky(seed, r, om.at), r, 0.0, 0.0);
                        assert!(w.snow >= o.shore_snow_falls && w.temperature < 2.5, "snow on the shore at {} deg C", w.temperature);
                    }
                    OmenKind::StormRun => {
                        for back in 0..o.storm_run_days as i64 {
                            let stormy = (0..48).any(|k| {
                                let w = at((day - back) as f64 * DAY + k as f64 * 0.5 * HOUR);
                                w.storm >= 0.5 || w.wind >= weather::GALE * 0.95
                            });
                            assert!(stormy, "day {} of a run of storm days was quiet", day - back);
                        }
                    }
                    OmenKind::LandmarkStrike => unreachable!("lightning on a landmark needs the land"),
                }
            }
            all.push(found);
        }
    }
    for kind in OmenKind::ALL {
        if kind != OmenKind::LandmarkStrike {
            assert!(seen.get(&kind).copied().unwrap_or(0) > 0, "{} never came in 120 years", kind.name());
        }
    }
    // Asked again, backwards and in bits: the same omens.
    let mut k = all.len();
    for day in (0..days).rev().step_by(7) {
        for r in Region::ALL.iter().rev() {
            let _ = weather::omens(seed, Region::Plateau, day + 3);
            k = (day as usize) * 6 + *r as usize;
            assert_eq!(all[k], weather::omens(seed, *r, day));
        }
    }
    assert!(k < all.len());
    // The weather reaches the east later, and so do its omens.
    let east = 1.5 * HOUR;
    let later: usize = (0..days).map(|d| weather::omens_lagging(seed, Region::Mountain, d, east).len()).sum();
    let sooner: usize = (0..days).map(|d| weather::omens(seed, Region::Mountain, d).len()).sum();
    assert!((later as f32 - sooner as f32).abs() <= 0.2 * sooner as f32 + 3.0, "{sooner} omens at the sea's edge, {later} an hour and a half inland");
}

#[test]
fn lightning_on_a_landmark_is_a_real_strike_on_a_real_town() {
    let world = worldgen::generate(3);
    let landmarks = world.landmarks();
    let o = &weather::climate().omens;
    let mut found = 0;
    for day in 0..48 * 12 {
        let omens = weather::strike_omens(&world.terrain, world.seed, &landmarks, day);
        for om in &omens {
            found += 1;
            let (pos, k) = (om.pos.expect("a strike has a place"), om.landmark.expect("and a landmark"));
            assert!(pos.dist(landmarks[k].pos) <= o.strike_reach);
            assert!(om.text.contains(landmarks[k].name));
            assert!(world.strikes(om.at - 1.0, om.at + 1.0).iter().any(|s| s.pos == pos && s.power >= o.strike_power), "no such strike");
            assert!(world.weather_at(pos, om.at).lightning > 0.0);
            // The town's own region hears of it; the others don't.
            let home = world.climate_region(landmarks[k].pos);
            assert_eq!(om.region, home);
            assert!(world.omens(home, day).iter().any(|x| x.landmark == Some(k)));
            assert!(world.omens_at(landmarks[k].pos, day).iter().any(|x| x.landmark == Some(k)) || weather::lag(landmarks[k].pos) > 0.0);
            let other = Region::ALL.iter().copied().find(|r| *r != home).unwrap();
            assert!(world.omens(other, day).iter().all(|x| x.landmark != Some(k)));
        }
        assert_eq!(omens, weather::strike_omens(&world.terrain, world.seed, &landmarks, day));
    }
    assert!(found >= 5, "only {found} towns struck in twelve years");
}

/// A sky built by hand, brought down to a spot at sea level.
fn made(region: Region, sky: Sky) -> Weather {
    weather::localise(&sky, region, 2.0, 0.0)
}

#[test]
fn a_fine_day_changes_nothing_and_bad_weather_does() {
    // Noon at midsummer (day 12 of the 48-day year) and at midwinter (day 36).
    let (summer, winter) = (12.5 * DAY, 36.5 * DAY);
    let fine = Sky { cloud: 0.1, wind: 3.0, wind_x: 3.0, temp0: 17.0, cloud_base: 2000.0, snow_lying: 2500.0, ..Sky::default() };
    let e = weather::effects_of(&made(Region::Lowland, fine), summer);
    assert!(e.sight_mult == 1.0 && e.hearing_mult == 1.0 && e.ranged_accuracy_mult == 1.0 && e.travel_speed_mult == 1.0, "a fine day changed something: {e:?}");
    assert!(e.slip_risk == 0.0 && e.exposure == 0.0 && e.shelter_seeking == 0.0 && e.boats_can_sail && e.outdoor_work_ok && e.sea_danger < 0.2);
    assert!(e.fire_spread_mult > 1.0 && e.fire_spread_mult < 1.3, "a light breeze on dry ground fans a fire a little: {}", e.fire_spread_mult);
    assert!(e.crop_growth_mult > 0.6 && e.crop_growth_mult < 1.2);
    let none = Effects::NONE;
    assert!(none.sight_mult == 1.0 && none.boats_can_sail && none.outdoor_work_ok);

    // Thick fog: you can't see, the boats stay in, but you hear as well as ever.
    let fog = weather::effects_of(&made(Region::ExposedCoast, Sky { fog: 1.0, fog_depth: 60.0, ..fine }), summer);
    assert!(fog.sight_mult < 0.25 && fog.hearing_mult == 1.0 && !fog.boats_can_sail && fog.sea_danger >= 0.5 && !fog.outdoor_work_ok, "{fog:?}");
    // Mist is nothing like as bad.
    let mist = weather::effects_of(&made(Region::ExposedCoast, Sky { fog: 0.4, fog_depth: 60.0, ..fine }), summer);
    assert!(mist.sight_mult > fog.sight_mult && mist.boats_can_sail && mist.outdoor_work_ok);

    // A thunderstorm on the coast: everything is worse.
    let storm_sky = Sky { cloud: 1.0, precip: 0.9, wind: 26.0, wind_x: 26.0, gust: 0.8, temp0: 9.0, storm: 1.0, lightning: 3.0, sea: 1.0, wetness: 1.0, cloud_base: 300.0, snow_lying: 1200.0, ..Sky::default() };
    let storm = weather::effects_of(&made(Region::ExposedCoast, storm_sky), summer);
    assert!(storm.sight_mult < 0.9 && storm.hearing_mult < 0.4 && storm.ranged_accuracy_mult < 0.45 && storm.travel_speed_mult < 0.65, "{storm:?}");
    assert!(storm.slip_risk > 0.3 && storm.exposure > 0.5 && storm.fire_spread_mult < 0.1 && !storm.boats_can_sail && storm.sea_danger > 0.9);
    assert!(!storm.outdoor_work_ok && storm.shelter_seeking > 0.95);
    // The same gale is easier going in the sheltered low country.
    let sheltered = weather::effects_of(&made(Region::Lowland, storm_sky), summer);
    assert!(sheltered.travel_speed_mult > storm.travel_speed_mult);

    // Drizzle: people shrug it off and carry on.
    let drizzle = weather::effects_of(&made(Region::ExposedCoast, Sky { cloud: 0.9, precip: 0.12, wetness: 0.4, wind: 6.0, wind_x: 6.0, temp0: 11.0, ..fine }), summer);
    assert!(drizzle.outdoor_work_ok && drizzle.boats_can_sail && drizzle.shelter_seeking < 0.05 && drizzle.fire_spread_mult < 0.6, "{drizzle:?}");

    // A dry gale fans fire; hard frost and lying snow stop the crops and the feet.
    let dry_gale = weather::effects_of(&made(Region::Plateau, Sky { wind: 20.0, wind_x: 20.0, ..fine }), summer);
    assert!(dry_gale.fire_spread_mult > 1.5 && dry_gale.ranged_accuracy_mult < 0.7);
    let frost = weather::effects_of(&made(Region::Upland, Sky { cloud: 0.2, wind: 8.0, wind_x: 8.0, temp0: -6.0, wetness: 0.5, cloud_base: 2000.0, snow_lying: -300.0, ..Sky::default() }), winter);
    assert!(frost.crop_growth_mult == 0.0 && frost.exposure > 0.7 && frost.slip_risk >= 0.4 && frost.travel_speed_mult < 0.7 && frost.feels_like < -6.0, "{frost:?}");
    // Winter alone slows the crops, even on a mild damp day.
    let mild = Sky { cloud: 0.6, wind: 4.0, wind_x: 4.0, temp0: 12.0, wetness: 0.4, cloud_base: 900.0, snow_lying: 2000.0, ..Sky::default() };
    let (in_summer, in_winter) = (weather::effects_of(&made(Region::Lowland, mild), summer), weather::effects_of(&made(Region::Lowland, mild), winter));
    assert!(in_winter.crop_growth_mult < 0.25 * in_summer.crop_growth_mult && in_summer.crop_growth_mult > 0.9);
    // A clouded night is darker than a clear one; by day the cloud costs nothing.
    let grey = Sky { cloud: 1.0, ..fine };
    let midnight = 12.0 * DAY;
    assert!(weather::effects_of(&made(Region::Lowland, grey), midnight).sight_mult < weather::effects_of(&made(Region::Lowland, fine), midnight).sight_mult);
    assert_eq!(weather::effects_of(&made(Region::Lowland, grey), summer).sight_mult, 1.0);
}

#[test]
fn what_the_weather_does_is_looked_up_and_stays_within_bounds() {
    let terrain = Terrain::generate(2);
    let (mut sailing, mut ashore, mut working, mut stopped) = (0, 0, 0, 0);
    for &p in spots().iter() {
        for k in 0..400 {
            let t = k as f64 * 7.3 * HOUR;
            let e = weather::weather_effects(&terrain, 2, p, t);
            assert_eq!(e, weather::effects_of(&weather::weather_at(&terrain, 2, p, t), t).with_travel(e.travel_speed_mult), "worked out two ways");
            for (name, v, most) in [
                ("sight", e.sight_mult, 1.0),
                ("hearing", e.hearing_mult, 1.0),
                ("aim", e.ranged_accuracy_mult, 1.0),
                ("travel", e.travel_speed_mult, 1.0),
                ("slip", e.slip_risk, 1.0),
                ("exposure", e.exposure, 1.0),
                ("fire", e.fire_spread_mult, 2.0),
                ("sea danger", e.sea_danger, 1.0),
                ("crops", e.crop_growth_mult, 1.3),
                ("shelter", e.shelter_seeking, 1.0),
            ] {
                assert!(v.is_finite() && (0.0..=most).contains(&v), "{name} is {v} at ({}, {}), hour {}", p.x, p.y, t / HOUR);
            }
            assert!(e.sight_mult > 0.0 && e.hearing_mult > 0.0 && e.ranged_accuracy_mult > 0.0 && e.travel_speed_mult > 0.3 && e.fire_spread_mult > 0.0);
            assert_eq!(e.boats_can_sail, e.sea_danger < 0.5);
            sailing += e.boats_can_sail as u32;
            ashore += !e.boats_can_sail as u32;
            working += e.outdoor_work_ok as u32;
            stopped += !e.outdoor_work_ok as u32;
        }
    }
    // Most days the boats go out and work goes on; some days not.
    assert!(sailing > 3 * ashore && ashore > 0, "boats out {sailing} times, kept in {ashore}");
    assert!(working > 4 * stopped && stopped > 0, "work went on {working} times, stopped {stopped}");
    // A day's growth is the average of its hours.
    let p = spots()[10];
    let day = weather::crop_growth_over(&terrain, 2, p, 12.0 * DAY, 13.0 * DAY);
    let hours: f32 = (0..24).map(|h| weather::weather_effects(&terrain, 2, p, 12.0 * DAY + (h as f64 + 0.5) * HOUR).crop_growth_mult).sum::<f32>() / 24.0;
    assert!((day - hours).abs() < 1e-5);
}
