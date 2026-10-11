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

// ---- The making screen (#165 UI-Making): what a bench shows, and making together ----

/// The squad at the forge nearest it, nobody a smith yet.
fn at_the_forge() -> (World, V2) {
    let mut w = worldgen::generate(1);
    let here = w.squad.pos;
    let forge = w.stations.iter().filter(|s| s.1 == Station::Forge).min_by(|a, b| a.0.dist(here).total_cmp(&b.0.dist(here))).unwrap().0;
    w.teleport_squad(forge.add(V2::new(1.0, 0.0)));
    for (k, _) in w.squad.members.clone().iter().enumerate() {
        w.squad.at[k] = forge.add(V2::new(1.0, 0.3 * k as f32));
        w.squad.goal[k] = w.squad.at[k];
    }
    for m in w.squad.members.clone() {
        let p = &mut w.people[m as usize];
        p.detail.as_mut().unwrap().crafts.retain(|s| *s != Skill::Smithing && *s != Skill::Armoring);
        // Nothing of their own to make things from.
        for key in ["iron_ingot", "leather", "timber", "iron_ore", "charcoal"] {
            while p.detail.as_mut().unwrap().gear.take(items::id(key)) {}
        }
    }
    (w, forge)
}

fn take_up(w: &mut World, who: u32, skill: Skill, level: f32) {
    let p = &mut w.people[who as usize];
    let d = p.detail.as_mut().unwrap();
    if !d.crafts.contains(&skill) {
        d.crafts.push(skill);
    }
    p.stats.set_skill(skill, level);
}

fn give(w: &mut World, who: u32, key: &str, n: u16) {
    w.people[who as usize].detail.as_mut().unwrap().gear.add(items::id(key), n);
}

#[test]
fn a_bench_lists_what_it_makes_for_those_who_are_there() {
    use gahturiyu_sim::sim::crafting::{Hand, Standing};
    let (mut w, forge) = at_the_forge();
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    let bench = w.bench_for(a).expect("standing at a forge");
    assert_eq!(bench.station, Station::Forge);
    assert!(bench.at.dist(forge) < 0.1 && bench.carried_by.is_none());
    assert!(!bench.place.is_empty(), "it says where it is");
    assert!(w.bench_hands(&bench).contains(&a));
    // Nobody here has taken up smithing: nothing is listed.
    assert!(w.making_list(&bench).is_empty(), "a craft nobody near knows is left out");
    // One smith: the forge's smithing is listed, and nothing of other benches.
    take_up(&mut w, a, Skill::Smithing, 45.0);
    let rows = w.making_list(&bench);
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|r| RECIPES[r.recipe].station == Station::Forge && RECIPES[r.recipe].skill == Skill::Smithing));
    let knife = recipe("knife");
    let row = |w: &World| w.making_list(&bench).into_iter().find(|r| r.recipe == knife).expect("a knife is on the list");
    // Short of its parts: said part by part, have and need.
    let r = row(&w);
    assert_eq!(r.standing, Standing::Short);
    assert!(r.parts.iter().all(|p| p.1 == 0 && p.2 > 0), "{:?}", r.parts);
    // With the parts in hand, it can be made, by the one who knows how.
    for &(k, n) in RECIPES[knife].inputs {
        give(&mut w, a, k, n);
    }
    let r = row(&w);
    assert_eq!(r.standing, Standing::Ready);
    assert_eq!(r.best, Some(a));
    // Who could make it: in words. A second, clumsier hand comes after the first.
    take_up(&mut w, b, Skill::Smithing, 5.0);
    let makers = w.makers(&bench, knife);
    assert_eq!(makers.iter().map(|m| m.who).collect::<Vec<_>>(), vec![a, b]);
    assert!(makers[0].hand > makers[1].hand, "{:?}", makers);
    assert!(makers.iter().all(|m| m.at_bench && !m.busy));
    assert!(!Hand::Steady.words().chars().any(|c| c.is_ascii_digit()));
    // Walked off, they're named but can't work it.
    let k = w.squad.index(b).unwrap();
    w.squad.at[k] = forge.add(V2::new(30.0, 0.0));
    w.squad.goal[k] = w.squad.at[k];
    let far = w.makers(&bench, knife).into_iter().find(|m| m.who == b).unwrap();
    assert!(!far.at_bench);
    // Something far beyond anyone's skill is marked so.
    let hard = (0..RECIPES.len()).filter(|&i| RECIPES[i].station == Station::Forge && RECIPES[i].skill == Skill::Smithing).max_by(|&x, &y| RECIPES[x].difficulty.total_cmp(&RECIPES[y].difficulty)).unwrap();
    w.people[a as usize].stats.set_skill(Skill::Smithing, 0.0);
    if success_chance(0.0, RECIPES[hard].difficulty) < 0.2 {
        let r = w.making_list(&bench).into_iter().find(|r| r.recipe == hard).unwrap();
        assert_eq!(r.standing, Standing::Beyond);
    }
    // Away from any bench there's no screen to open (no mortar in that pack).
    w.teleport_squad(forge.add(V2::new(300.0, 0.0)));
    while w.people[a as usize].detail.as_mut().unwrap().gear.take(items::id("mortar_and_pestle")) {}
    assert!(w.bench_for(a).is_none());
}

#[test]
fn what_the_maker_lacks_comes_from_whoever_stands_at_the_bench() {
    let (mut w, forge) = at_the_forge();
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    take_up(&mut w, a, Skill::Smithing, 60.0);
    let knife = recipe("knife");
    let inputs = RECIPES[knife].inputs;
    assert!(inputs.len() >= 2, "a knife takes two things");
    // The smith holds the first part, the other the rest.
    give(&mut w, a, inputs[0].0, inputs[0].1);
    for &(k, n) in &inputs[1..] {
        give(&mut w, b, k, n);
    }
    assert_eq!(w.can_craft(a, knife), Ok(()), "between them they have it");
    // Not if the other is across the yard.
    let k = w.squad.index(b).unwrap();
    w.squad.at[k] = forge.add(V2::new(40.0, 0.0));
    w.squad.goal[k] = w.squad.at[k];
    assert!(matches!(w.can_craft(a, knife), Err(Cannot::Missing(..))));
    w.squad.at[k] = forge.add(V2::new(1.0, 1.0));
    w.squad.goal[k] = w.squad.at[k];
    w.start_craft(a, knife).expect("made together");
    for &(key, _) in inputs {
        assert_eq!(w.count_of(a, key) + w.count_of(b, key), 0, "{key} went in");
    }
}

#[test]
fn several_in_a_row_end_when_they_should_however_time_is_stepped() {
    // (How many knives came out, how many are still queued, when each job
    // ended whether it came out or was botched, and whether it said it ran out.)
    let run = |step: f64, sets: u16| -> (u16, usize, Vec<f64>, bool) {
        let (mut w, _) = at_the_forge();
        let a = w.squad.members[0];
        // (Good enough that none is botched.)
        take_up(&mut w, a, Skill::Smithing, 100.0);
        let knife = recipe("knife");
        for &(k, n) in RECIPES[knife].inputs {
            give(&mut w, a, k, n * sets);
        }
        let start = w.time;
        w.make(a, knife, 3).expect("the first is begun");
        let end = start + RECIPES[knife].time * 3.0 + 120.0;
        let mut made_at: Vec<f64> = Vec::new();
        let mut steps = 0;
        // (Work that fell due during a step is seen to as the next one
        // begins, so one short step follows the last.)
        while w.time < end + 1.0 {
            w.step(if w.time < end { step.min(end - w.time) } else { 1.0 });
            steps += 1;
            for l in w.log.iter().filter(|l| l.0 >= start && (l.1.contains(" makes ") || l.1.contains(" botches "))) {
                if !made_at.contains(&(l.0 - start)) {
                    made_at.push(l.0 - start);
                }
            }
        }
        assert!(step < 2.0 || steps <= 4, "big steps really were big: {steps}");
        made_at.sort_by(|x, y| x.total_cmp(y));
        let knives = w.people[a as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| items::info(e.0).form == items::id("knife")).map(|e| e.1).sum();
        (knives, w.making_more.len(), made_at, w.log.iter().any(|l| l.1.contains("has no more")))
    };
    let t = RECIPES[recipe("knife")].time;
    let fine = run(1.0, 3);
    assert_eq!(fine.1, 0, "nothing left queued");
    assert_eq!(fine.2.len(), 3, "three asked for, three jobs done");
    assert!(fine.0 >= 2, "and nearly all of them came out: {}", fine.0);
    for (k, at) in fine.2.iter().enumerate() {
        assert!((at - t * (k + 1) as f64).abs() < 1e-6, "the {}th was done at {at}, not {}", k + 1, t * (k + 1) as f64);
    }
    let coarse = run(t * 2.5, 3);
    assert_eq!((coarse.0, coarse.1), (fine.0, 0), "the same knives in big steps as in small");
    assert_eq!(coarse.2, fine.2, "at the same moments");
    // Materials for two of the three: two are made, and it's said why it stopped.
    let short = run(1.0, 2);
    assert_eq!((short.2.len(), short.1), (2, 0), "two jobs, then it stops");
    assert!(short.3, "it says what ran out");
}

#[test]
fn how_long_a_job_takes_is_said_in_words() {
    use gahturiyu_sim::sim::crafting::time_words;
    assert_eq!(time_words(90.0), "a minute or two");
    assert_eq!(time_words(20.0 * 60.0), "about 20 minutes");
    assert_eq!(time_words(3600.0), "about an hour");
    assert_eq!(time_words(2.5 * 3600.0), "about 2½ hours");
    assert_eq!(time_words(4.0 * 3600.0), "about 4 hours");
    assert_eq!(time_words(3.0 * DAY), "about 3 days");
}
