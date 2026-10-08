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
