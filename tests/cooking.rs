//! Cooking (the items design, 0.6): a Cooking skill and meal grades; a
//! campfire (any fire burning within reach) does basic food, a pot for
//! porridge and tea; a stove (a hearth indoors, a base's kitchen) does full
//! meals and everything a campfire does; cooked fills more than raw; spoiled
//! food turns the stomach.

use gahturiyu_sim::sim::{
    crafting::{Cannot, Station, RECIPES},
    geo::V2,
    items::{self, item, Kind},
    jobs,
    materials::Grade,
    solid::What,
    spoil::{self, Fresh},
    stats::Skill,
    world::{DAY, HOUR},
    worldgen, World,
};

fn recipe(output: &str) -> usize {
    RECIPES.iter().position(|r| r.output == output).unwrap()
}

fn wait(w: &mut World, secs: f64) {
    let end = w.time + secs;
    while w.time < end - 1e-6 {
        w.step(10.0f64.min(end - w.time));
    }
}

fn fills(key: &str) -> f32 {
    match item(items::id(key)).kind {
        Kind::Food(n) => n,
        _ => panic!("{key} isn't food"),
    }
}

/// A fire burning in some town, and a spot beside it.
fn by_a_fire(w: &World) -> V2 {
    for s in &w.settlements {
        for th in w.solids_near(s.pos, s.reach) {
            if th.what == What::Fire && w.fire_lit(th.id, w.time) {
                let c = th.shape.centre();
                for k in 0..16 {
                    let a = k as f32 * 0.4;
                    let p = c.add(V2::new(a.cos(), a.sin()).scale(2.6));
                    let nearer = w.stations.iter().all(|s| s.0.dist(p) > c.dist(p));
                    if nearer && w.fire_at(p).is_some() && w.building_at(p).is_none() && w.can_stand(p, 0.3) {
                        return p;
                    }
                }
            }
        }
    }
    panic!("no fire in any town");
}

/// Member 0 at a fire, nothing to eat but what's given, fed and watered.
fn cook_at_fire(seed: u64) -> (World, u32) {
    let mut w = worldgen::generate(seed);
    let at = by_a_fire(&w);
    w.teleport_squad(at);
    let m = w.squad.members[0];
    let d = w.people[m as usize].detail.as_mut().unwrap();
    d.gear.bag.retain(|e| !spoil::is_food(e.0));
    w.people[m as usize].cond.as_mut().unwrap().hunger = 0.0;
    // A steady hand, so a botch doesn't hide what's tested.
    w.people[m as usize].stats.set_skill(Skill::Cooking, 60.0);
    (w, m)
}

fn give(w: &mut World, m: u32, key: &str, n: u16) {
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id(key), n);
}

fn held(w: &World, m: u32, form: &str) -> Vec<(Grade, u16)> {
    let f = items::id(form);
    w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| items::info(e.0).form == f).map(|e| (items::info(e.0).grade, e.1)).collect()
}

#[test]
fn cooked_fills_more_than_raw() {
    assert!(fills("roast_meat") > fills("raw_meat") * 1.4);
    assert!(fills("spit_fish") > fills("fish") * 1.4);
    // And every dish fills more than what goes into it, eaten raw.
    for rc in RECIPES.iter().filter(|r| r.skill == Skill::Cooking) {
        let raw: f32 = rc.inputs.iter().map(|&(k, n)| if let Kind::Food(x) = item(items::id(k)).kind { x * n as f32 } else { 0.0 }).sum();
        assert!(fills(rc.output) * rc.makes as f32 > raw, "{} fills more than its raw food", rc.output);
    }
}

#[test]
fn any_fire_is_a_campfire_and_the_squad_can_cook_there() {
    let (mut w, m) = cook_at_fire(1);
    give(&mut w, m, "raw_meat", 2);
    assert!(w.station_near(w.person_pos(m), Station::Campfire).is_some());
    let b = w.bench_for(m).expect("a bench");
    assert_eq!(b.station, Station::Campfire);
    let list = w.making_list(&b);
    assert!(list.iter().any(|r| RECIPES[r.recipe].output == "roast_meat"));
    assert!(!list.iter().any(|r| RECIPES[r.recipe].station == Station::Stove), "no full meals over a campfire");
    w.make(m, recipe("roast_meat"), 2).unwrap();
    wait(&mut w, 40.0 * 60.0);
    let n: u16 = held(&w, m, "roast_meat").iter().map(|x| x.1).sum();
    assert!(n >= 1, "roast meat made");
    // Away from any fire, nothing to cook on.
    let away = w.person_pos(m).add(V2::new(400.0, 0.0));
    assert!(w.fire_at(away).is_none());
}

#[test]
fn porridge_over_a_campfire_wants_a_pot_and_water() {
    let (mut w, m) = cook_at_fire(2);
    give(&mut w, m, "grain", 2);
    let ri = recipe("camp_porridge");
    // In town there's water (its wells); no pot yet.
    let why = w.craft_blockers(m, ri);
    assert!(why.contains(&Cannot::NoPot), "{why:?}");
    assert!(!why.contains(&Cannot::NoWater));
    give(&mut w, m, "cooking_pot", 1);
    assert!(w.can_craft(m, ri).is_ok(), "{:?}", w.craft_blockers(m, ri));
    w.make(m, ri, 1).unwrap();
    wait(&mut w, 30.0 * 60.0);
    assert!(held(&w, m, "camp_porridge").iter().map(|x| x.1).sum::<u16>() >= 1);
    assert_eq!(w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == items::id("cooking_pot")).count(), 1, "the pot isn't used up");
}

#[test]
fn a_hearth_indoors_is_a_stove_for_full_meals_and_campfire_food_too() {
    let mut w = worldgen::generate(3);
    // A home with a hearth: its middle.
    let home = (0..w.settlements.len() as u16)
        .flat_map(|s| (0..w.settlements[s as usize].buildings.len() as u16).map(move |b| (s, b)))
        .filter_map(|(s, b)| gahturiyu_sim::sim::buildings::door_of(&w.settlements[s as usize], b))
        .find(|d| w.station_near(d.centre, Station::Stove).is_some())
        .expect("a building with a hearth");
    let inside = w.free_spot(home.centre);
    w.teleport_squad(inside);
    let m = w.squad.members[0];
    let at = w.person_pos(m);
    assert!(w.station_near(at, Station::Stove).is_some(), "standing by a stove");
    for (k, n) in [("raw_meat", 2), ("sweetroot", 1), ("hearthleaf", 1), ("grain", 1)] {
        give(&mut w, m, k, n);
    }
    let b = w.bench_for(m).expect("a bench");
    assert_eq!(b.station, Station::Stove);
    let list = w.making_list(&b);
    for dish in ["sweetroot_stew", "roast_meat", "camp_porridge"] {
        assert!(list.iter().any(|r| RECIPES[r.recipe].output == dish), "{dish} at a stove");
    }
    // A stove has its own pots.
    assert!(!w.craft_blockers(m, recipe("camp_porridge")).contains(&Cannot::NoPot));
    w.make(m, recipe("sweetroot_stew"), 1).unwrap();
    wait(&mut w, HOUR);
    assert!(held(&w, m, "sweetroot_stew").iter().map(|x| x.1).sum::<u16>() >= 1, "a stew made");
}

#[test]
fn out_in_the_wild_a_cook_draws_water_from_a_waterskin() {
    let mut w = worldgen::generate(4);
    // A camp's fire, out in the wild (its band can wait: nothing is stepped).
    let c = w.camps.iter().map(|c| c.pos).find(|&p| !w.water_at(p)).expect("a camp in the wild");
    let fire = w.fire_at(c).expect("its fire");
    let at = (0..16).map(|k| fire.add(V2::new((k as f32 * 0.4).cos(), (k as f32 * 0.4).sin()).scale(2.6))).find(|&p| w.can_stand(p, 0.3) && w.fire_at(p).is_some()).unwrap();
    w.teleport_squad(at);
    let m = w.squad.members[0];
    let ri = recipe("moorbell_tea");
    give(&mut w, m, "moorbell", 1);
    give(&mut w, m, "cooking_pot", 1);
    for &x in &w.squad.members.clone() {
        w.people[x as usize].cond.as_mut().unwrap().water = 0.0;
    }
    assert!(w.craft_blockers(m, ri).contains(&Cannot::NoWater), "no well, empty skins");
    w.people[m as usize].cond.as_mut().unwrap().water = 2.0;
    assert_eq!(w.cooking_water(m), Some(Some(m)));
    w.start_craft(m, ri).unwrap();
    assert_eq!(w.people[m as usize].cond.as_ref().unwrap().water, 1.0, "a drink's worth of water went in");
}

#[test]
fn a_better_cook_makes_better_meals_and_better_meals_fill_more() {
    let grades = |skill: f32| {
        let (mut w, m) = cook_at_fire(5);
        w.people[m as usize].stats.set_skill(Skill::Cooking, skill);
        give(&mut w, m, "raw_meat", 10);
        w.make(m, recipe("roast_meat"), 10).unwrap();
        wait(&mut w, 3.0 * HOUR);
        held(&w, m, "roast_meat")
    };
    let score = |v: &[(Grade, u16)]| v.iter().map(|&(g, n)| g as u32 as f32 * n as f32).sum::<f32>() / v.iter().map(|x| x.1 as f32).sum::<f32>().max(1.0);
    let (poor, good) = (grades(5.0), grades(95.0));
    assert!(!poor.is_empty() && !good.is_empty(), "{poor:?} {good:?}");
    assert!(score(&good) > score(&poor) + 0.8, "skill raises the grade ({poor:?} vs {good:?})");
    let at = |g: Grade| fills(items::variant(items::id("roast_meat"), gahturiyu_sim::sim::materials::Material::None, gahturiyu_sim::sim::materials::Material::None, g).map(|i| item(i).key).unwrap());
    assert!(at(Grade::Masterwork) > at(Grade::Common) && at(Grade::Common) > at(Grade::Crude));
    // A graded dish keeps like its plain one.
    let fine = items::variant(items::id("trail_cake"), gahturiyu_sim::sim::materials::Material::None, gahturiyu_sim::sim::materials::Material::None, Grade::Fine).unwrap();
    assert_eq!(spoil::shelf_of(fine), spoil::shelf_of(items::id("trail_cake")));
    assert!(item(fine).name.starts_with("Fine "));
}

#[test]
fn spoiled_food_turns_the_stomach() {
    let mut w = worldgen::generate(6);
    let m = w.squad.members[0];
    let d = w.people[m as usize].detail.as_mut().unwrap();
    d.gear.bag.retain(|e| !spoil::is_food(e.0));
    d.gear.add(items::id("fish"), 3);
    w.step(1.0);
    // Three days on, the fish has spoiled (two days' shelf life).
    w.people[m as usize].cond.as_mut().unwrap().hunger = 0.0;
    let t0 = w.time;
    for l in w.larder.iter_mut().filter(|l| l.pid == m) {
        l.at = t0 - 3.0 * DAY;
    }
    assert_eq!(w.freshness(m, items::id("fish"), w.time), Fresh::Spoiled);
    let before = w.toxicity_of(m).unwrap_or(0.0);
    assert!(w.eat(m, items::id("fish"), w.time));
    let after = w.toxicity_of(m).unwrap_or(0.0);
    assert!(after > before + 1.0, "a little toxicity ({before:.1} -> {after:.1})");
    assert!(w.log.iter().any(|l| l.1.contains("spoiled")));
}

#[test]
fn every_dish_can_be_made_from_what_can_be_had() {
    // Everything a dish wants is sold in towns, gathered, or hunted.
    let hunted = ["raw_meat", "egg", "animal_oil", "fish", "milk"];
    let gathered = ["ashberry", "puffcap", "sweetroot", "wild_berries", "bloodleaf", "mussels", "salt_crystal"];
    for rc in RECIPES.iter().filter(|r| r.skill == Skill::Cooking) {
        for &(k, _) in rc.inputs {
            assert!(jobs::good_of(k).is_some() || hunted.contains(&k) || gathered.contains(&k), "{} wants {k}, which can't be had", rc.output);
        }
    }
    assert_eq!(RECIPES.iter().filter(|r| r.skill == Skill::Cooking).count(), 15, "the fifteen dishes");
}

// ---- Meal buffs (Part 5, rule 3), teas and broths (rule 6) -------------------

use gahturiyu_sim::sim::effects::Does;

fn graded(key: &str, g: Grade) -> items::ItemId {
    items::variant(items::id(key), gahturiyu_sim::sim::materials::Material::None, gahturiyu_sim::sim::materials::Material::None, g).unwrap()
}

/// Member 0, hungry enough to eat anything, given `it`.
fn fed(seed: u64, it: items::ItemId) -> (World, u32) {
    let mut w = worldgen::generate(seed);
    let m = w.squad.members[0];
    w.people[m as usize].detail.as_mut().unwrap().gear.add(it, 1);
    w.step(1.0);
    // (Hungry only now, so they haven't eaten it by themselves.)
    w.people[m as usize].cond.as_mut().unwrap().hunger = 60.0;
    (w, m)
}

fn boon(w: &World, m: u32, d: Does) -> f32 {
    w.boons_on(m).iter().filter(|b| b.does == d).map(|b| b.power).sum()
}

#[test]
fn a_full_meal_gives_its_buff_for_some_hours_and_raw_food_none() {
    let (mut w, m) = fed(11, items::id("sweetroot_stew"));
    let heal_before = w.people[m as usize].cond.as_ref().unwrap().heal_rate();
    assert!(w.eat(m, items::id("sweetroot_stew"), w.time));
    assert!(boon(&w, m, Does::Mending) > 0.2, "the stew mends");
    let c = w.people[m as usize].cond.as_ref().unwrap();
    assert!(c.heal_rate() > heal_before * 1.2, "wounds mend faster ({:.2} vs {heal_before:.2})", c.heal_rate());
    // Six hours on, it's worn off.
    let end = w.time + 6.5 * HOUR;
    while w.time < end {
        w.step(60.0);
    }
    assert_eq!(boon(&w, m, Does::Mending), 0.0);
    assert_eq!(w.people[m as usize].cond.as_ref().unwrap().mending, 0.0, "and the condition knows");
    // Roast meat fills, nothing more.
    let (mut w, m) = fed(12, items::id("roast_meat"));
    assert!(w.eat(m, items::id("roast_meat"), w.time));
    assert!(w.boons_on(m).is_empty());
}

#[test]
fn one_meal_buff_at_a_time_the_newest_replacing_it() {
    let (mut w, m) = fed(13, items::id("sweetroot_stew"));
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("mussel_pot"), 1);
    assert!(w.eat(m, items::id("sweetroot_stew"), w.time));
    assert!(boon(&w, m, Does::Mending) > 0.0);
    w.people[m as usize].cond.as_mut().unwrap().hunger = 60.0;
    assert!(w.eat(m, items::id("mussel_pot"), w.time));
    assert_eq!(boon(&w, m, Does::Mending), 0.0, "the stew's buff gone");
    assert!(boon(&w, m, Does::Breath) > 0.2, "the mussel pot's in");
    // Breath: resting, stamina comes back faster.
    let c = w.people[m as usize].cond.as_ref().unwrap();
    assert!(c.breath > 0.2);
}

#[test]
fn a_better_cooked_meal_gives_a_stronger_buff() {
    let power = |g: Grade| {
        let it = graded("firepod_stew", g);
        let (mut w, m) = fed(14, it);
        assert!(w.eat(m, it, w.time));
        boon(&w, m, Does::Warmth)
    };
    let (crude, common, fine) = (power(Grade::Crude), power(Grade::Common), power(Grade::Fine));
    assert!(crude < common && common < fine, "{crude} {common} {fine}");
}

#[test]
fn tea_and_broth_quench_thirst() {
    let (mut w, m) = fed(15, items::id("moorbell_tea"));
    w.people[m as usize].cond.as_mut().unwrap().thirst = 60.0;
    let before = w.people[m as usize].cond.as_ref().unwrap().thirst;
    assert!(w.eat(m, items::id("moorbell_tea"), w.time));
    let after = w.people[m as usize].cond.as_ref().unwrap().thirst;
    assert!(after < before - 20.0, "{before} -> {after}");
    assert!(boon(&w, m, Does::Warmth) > 0.0, "and it warms");
}

#[test]
fn a_spoiled_meal_gives_no_buff() {
    let (mut w, m) = fed(16, items::id("sweetroot_stew"));
    for l in w.larder.iter_mut().filter(|l| l.pid == m) {
        l.at -= 10.0 * DAY;
    }
    assert_eq!(w.freshness(m, items::id("sweetroot_stew"), w.time), Fresh::Spoiled);
    assert!(w.eat(m, items::id("sweetroot_stew"), w.time));
    assert_eq!(boon(&w, m, Does::Mending), 0.0);
}
