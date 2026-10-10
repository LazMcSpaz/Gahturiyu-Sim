//! Making things, gathering, and using what you make.

use gahturiyu_sim::sim::{
    combat::{Battle, Fighter},
    crafting::{success_chance, Cannot, Station, RECIPES},
    geo::V2,
    items::{self, item, Kind},
    magic::spell,
    person::Person,
    race::Race,
    stats::{Calling, Skill},
    world::DAY,
    worldgen, World,
};

fn wait(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(1.0);
        t += 1.0;
    }
}

fn mage(w: &World) -> u32 {
    *w.squad.members.iter().find(|&&m| w.people[m as usize].stats.calling == Calling::Mage).unwrap()
}

fn recipe(output: &str) -> usize {
    RECIPES.iter().position(|r| r.output == output).unwrap()
}

#[test]
fn every_recipe_names_real_items() {
    for r in RECIPES {
        let _ = items::id(r.output);
        for (k, n) in r.inputs {
            let _ = items::id(k);
            assert!(*n > 0);
        }
    }
    // Every potion and scroll can be made, and so can every material that's
    // made rather than found.
    for key in ["healing_draught", "mana_tonic", "greater_healing", "scroll_heal", "scroll_fireball", "scroll_lightning", "scroll_paralyze", "iron_ingot", "leather"] {
        assert!(RECIPES.iter().any(|r| r.output == key), "no recipe for {key}");
    }
}

#[test]
fn a_mage_brews_a_potion_anywhere_with_a_mortar() {
    let mut w = worldgen::generate(1);
    let m = mage(&w);
    w.people[m as usize].stats.set_skill(Skill::Alchemy, 60.0);
    let before = w.count_of(m, "healing_draught");
    let skill = w.people[m as usize].stats.skill(Skill::Alchemy);
    w.start_craft(m, recipe("healing_draught")).expect("can brew");
    assert_eq!(w.count_of(m, "kelp_frond"), 2, "materials go in at the start");
    assert!(w.craft_progress(m).is_some());
    wait(&mut w, 45.0);
    assert!(w.craft_progress(m).is_none());
    assert_eq!(w.count_of(m, "healing_draught"), before + 1);
    assert!(w.people[m as usize].stats.skill(Skill::Alchemy) > skill);
}

#[test]
fn smithing_needs_a_forge() {
    let mut w = worldgen::generate(1);
    let q = *w.squad.members.iter().find(|&&m| w.people[m as usize].race == Race::Qotiro).unwrap();
    let k = w.squad.index(q).unwrap();
    // Away from town.
    w.teleport_squad(w.squad.pos.add(V2::new(400.0, 0.0)));
    assert_eq!(w.can_craft(q, recipe("knife")), Err(Cannot::NoStation(Station::Forge)));
    // At the forge.
    let forge = w.stations.iter().filter(|s| s.1 == Station::Forge).min_by(|a, b| a.0.dist(w.squad.at[k]).total_cmp(&b.0.dist(w.squad.at[k]))).unwrap().0;
    w.teleport_squad(forge.add(V2::new(1.0, 0.0)));
    let k = w.squad.index(q).unwrap();
    w.squad.at[k] = forge.add(V2::new(1.0, 0.0));
    w.squad.goal[k] = w.squad.at[k];
    w.people[q as usize].stats.set_skill(Skill::Smithing, 50.0);
    assert_eq!(w.can_craft(q, recipe("knife")), Ok(()));
    w.start_craft(q, recipe("knife")).unwrap();
    wait(&mut w, 100.0);
    // A knife of some grade, unless it was botched.
    let knives = w.people[q as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| items::info(e.0).form == items::id("knife")).count();
    assert!(knives == 1 || w.log.iter().any(|l| l.1.contains("botches")));
}

#[test]
fn a_botched_job_gives_half_back() {
    let mut w = worldgen::generate(1);
    let m = mage(&w);
    w.people[m as usize].stats.set_skill(Skill::Alchemy, 0.0);
    assert!(success_chance(0.0, 45.0) < 0.1);
    // Plenty of materials for several tries at something too hard.
    for (k, n) in [("kelp_frond", 20), ("ghostcap", 10), ("salt_crystal", 10)] {
        w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id(k), n);
    }
    let r = recipe("greater_healing");
    let mut botched = false;
    for _ in 0..4 {
        let kelp = w.count_of(m, "kelp_frond");
        let made = w.count_of(m, "greater_healing");
        w.start_craft(m, r).unwrap();
        wait(&mut w, 65.0);
        if w.count_of(m, "greater_healing") == made {
            assert_eq!(w.count_of(m, "kelp_frond"), kelp - 1, "half the kelp comes back");
            botched = true;
            break;
        }
    }
    assert!(botched);
}

#[test]
fn gathering_takes_and_it_grows_back() {
    let mut w = worldgen::generate(1);
    let here = w.squad.pos;
    let n = *w.nodes.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap();
    let m = w.squad.members[0];
    w.teleport_squad(n.pos.add(V2::new(3.0, 0.0)));
    let before = w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == n.item).map(|e| e.1).sum::<u16>();
    assert!(w.order_gather(m, n.id));
    wait(&mut w, 20.0);
    let after = w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == n.item).map(|e| e.1).sum::<u16>();
    assert_eq!(after, before + n.amount);
    let node = w.nodes.iter().find(|x| x.id == n.id).unwrap();
    assert!(!node.ready(w.time));
    assert!(node.ready(w.time + DAY));
}

#[test]
fn potions_heal_and_restore_mana() {
    let mut w = worldgen::generate(1);
    let m = mage(&w);
    let t = w.time;
    let p = &mut w.people[m as usize];
    let base = p.stats.clone();
    let mut hp = p.wounds.hp_at(&base, t);
    hp[1] -= 30.0;
    hp[3] -= 20.0;
    p.wounds.set(&base, &hp, t);
    p.set_mana(0.0, t);
    let hurt: f32 = p.wounds.lost_at(t).iter().sum();
    assert!(w.use_item(m, items::id("healing_draught")));
    let p = &w.people[m as usize];
    let now: f32 = p.wounds.lost_at(t).iter().sum();
    assert!((hurt - now - 25.0).abs() < 0.5, "{hurt} -> {now}");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("mana_tonic"), 1);
    assert!(w.use_item(m, items::id("mana_tonic")));
    assert!(w.people[m as usize].mana_at(t) >= 39.9);
}

fn fighter(id: u32, side: u8, at: f32) -> Fighter {
    let mut p = Person::summary(id, 900 + id as u64, Race::Roduro, None);
    p.specialize(Calling::Warrior, &[(Skill::Blade, 40.0)], 200.0);
    p.ensure_detail();
    let mut f = Fighter::from_person(&p, side, V2::new(at, 0.0), 0.0);
    f.think_at = f64::INFINITY;
    f
}

#[test]
fn hurt_fighters_drink_their_potions() {
    let mut a = fighter(1, 0, 0.0);
    a.potions.push(items::id("healing_draught"));
    a.hp[1] = 10.0;
    a.think_at = 0.0;
    let b = fighter(2, 1, 30.0);
    let mut battle = Battle::new(0, 3, 0.0, vec![a, b], vec!["A".into(), "B".into()]);
    let before = battle.fighters[0].hp[1];
    for _ in 0..30 {
        battle.tick();
    }
    assert!(battle.fighters[0].potions.is_empty());
    assert_eq!(battle.fighters[0].used, vec![items::id("healing_draught")]);
    assert!(battle.fighters[0].hp[1] > before + 10.0);
}

#[test]
fn scrolls_cast_without_mana_and_never_fizzle() {
    let mut a = fighter(1, 0, 0.0);
    a.mana = 0.0;
    a.spells.clear();
    let scroll = items::id("scroll_lightning");
    assert!(matches!(item(scroll).kind, Kind::Scroll("lightning_bolt")));
    a.scrolls.push(scroll);
    let b = fighter(2, 1, 12.0);
    let mut battle = Battle::new(0, 3, 0.0, vec![a, b], vec!["A".into(), "B".into()]);
    let hp: f32 = battle.fighters[1].hp.iter().sum();
    assert!(battle.read_scroll(0, spell("lightning_bolt"), Some(1), V2::new(12.0, 0.0)));
    for _ in 0..12 {
        battle.tick();
    }
    assert!(battle.fighters[1].hp.iter().sum::<f32>() < hp, "the bolt should land");
    assert!(battle.fighters[0].scrolls.is_empty());
    assert!(battle.fighters[0].mana < 0.5, "no mana spent (only a trickle regained)");
}

/// Every building of this style in the world, by its door.
fn buildings_of(w: &World, key: &str) -> Vec<gahturiyu_sim::sim::buildings::Door> {
    let mut out = Vec::new();
    for (si, s) in w.settlements.iter().enumerate() {
        for i in 0..s.buildings.len() as u16 {
            if gahturiyu_sim::sim::layout::variant_in(s, i).is_some_and(|v| v.key == key) {
                if let Some(d) = w.door((si as u16, i)) {
                    out.push(d);
                }
            }
        }
    }
    out
}

#[test]
fn a_forge_indoors_is_a_forge() {
    // Playtests 2 and 3: standing inside the building called "Maker's
    // forge", with its forge in front of them, both testers were told there
    // was no forge. A station is where its furniture is: in a town's work
    // yard, or in the workshop someone keeps one in.
    let w = worldgen::generate(34);
    let forges: Vec<_> = ["roduro_forge", "qotiro_workyard"].iter().flat_map(|k| buildings_of(&w, k)).collect();
    assert!(!forges.is_empty(), "world 34 has a forge building (the tester stood in one)");
    for d in &forges {
        assert!(w.station_near(d.inside, Station::Forge).is_some(), "a forge, just inside the door of {:?}", d.id);
        assert!(w.station_near(d.centre, Station::Forge).is_some(), "and in the middle of the floor");
        assert!(w.station_near(d.centre, Station::Workbench).is_some(), "its workbench too");
        assert!(w.station_near(d.centre, Station::Loom).is_none(), "but it has no loom");
        // Outside the door it's just a house.
        let away = d.outside.add(d.outside.sub(d.centre).scale(2.0));
        if w.building_at(away).is_none() && w.stations.iter().all(|s| s.0.dist(away) > 6.0) {
            assert!(w.station_near(away, Station::Forge).is_none(), "not from the street");
        }
    }
    // A weaver's house has the loom and not the forge.
    for d in buildings_of(&w, "roduro_loomroom").iter().take(3) {
        assert!(w.station_near(d.centre, Station::Loom).is_some());
        assert!(w.station_near(d.centre, Station::Forge).is_none());
    }
    // A plain home is no workshop.
    for d in buildings_of(&w, "roduro_cottage").iter().take(3) {
        if w.stations.iter().all(|s| s.0.dist(d.centre) > 6.0) {
            for st in gahturiyu_sim::sim::crafting::STATIONS {
                assert!(w.station_near(d.centre, st).is_none(), "{st:?} in a cottage");
            }
        }
    }
}

#[test]
fn everything_in_the_way_of_a_recipe_is_said_at_once_and_in_plain_words() {
    // Playtest 3: "rock" was missing; with rock in hand, "ash" was. And the
    // reasons read `NoStation(Forge)` and `Missing("rock", 2)`.
    let mut w = worldgen::generate(1);
    let q = *w.squad.members.iter().find(|&&m| w.people[m as usize].race == Race::Qotiro).unwrap();
    w.people[q as usize].stats.set_skill(Skill::Smithing, 50.0);
    w.teleport_squad(w.squad.pos.add(V2::new(400.0, 0.0)));
    let ingot = recipe("iron_ingot");
    // Empty the pack of what it takes, so both materials are short.
    for &(k, _) in RECIPES[ingot].inputs {
        while w.count_of(q, k) > 0 {
            w.people[q as usize].detail.as_mut().unwrap().gear.take(items::id(k));
        }
    }
    let stops = w.craft_blockers(q, ingot);
    let short: Vec<_> = stops.iter().filter(|c| matches!(c, Cannot::Missing(..))).collect();
    assert_eq!(short.len(), RECIPES[ingot].inputs.len(), "every material they're short of: {stops:?}");
    assert!(stops.contains(&Cannot::NoStation(Station::Forge)), "and the forge they're not at: {stops:?}");
    // The first of them is what `can_craft` gives, as before.
    assert_eq!(w.can_craft(q, ingot), Err(stops[0]));
    // Plain words, no code.
    for c in &stops {
        let said = c.say();
        assert!(!said.contains('(') && !said.contains('"') && !said.contains('_'), "{said}");
    }
    assert_eq!(Cannot::NoStation(Station::Forge).say(), "has to be done at a forge");
    assert_eq!(Cannot::NoStation(Station::AlchemyTable).say(), "has to be done at an alchemy table");
    assert_eq!(Cannot::Missing("iron_ore", 2).say(), "needs 2 × iron ore");
    // With everything to hand and at a forge, nothing is in the way.
    for &(k, n) in RECIPES[ingot].inputs {
        w.people[q as usize].detail.as_mut().unwrap().gear.add(items::id(k), n);
    }
    let forge = w.stations.iter().find(|s| s.1 == Station::Forge).unwrap().0;
    w.teleport_squad(forge);
    let k = w.squad.index(q).unwrap();
    w.squad.at[k] = forge;
    w.squad.goal[k] = forge;
    assert!(w.craft_blockers(q, ingot).is_empty(), "{:?}", w.craft_blockers(q, ingot));
    assert_eq!(w.can_craft(q, ingot), Ok(()));
}
