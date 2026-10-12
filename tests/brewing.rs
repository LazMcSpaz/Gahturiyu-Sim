//! Brewing at the alchemy table (Part 4; Laz's option (a): each bottle
//! carries its own mix). A brew carries what two or more of its ingredients
//! share; skill and the table's tools make it stronger and its harm briefer
//! (never gone); bottles stack only when their mixes are the same; brewing
//! teaches what the ingredients shared, tasting teaches one effect, and a
//! brewed mix is remembered.

use gahturiyu_sim::sim::{
    brewing::{NoBrew, BREW_SECS},
    crafting::Station,
    geo::V2,
    inventory::Gear,
    items::{
        self,
        herbs::{shared, Alch, Kit, Mix},
    },
    materials::{Grade, Piece},
    stats::Skill,
    worldgen, World,
};

fn id(k: &str) -> items::ItemId {
    items::id(k)
}

#[test]
fn a_brew_carries_what_two_or_more_ingredients_share() {
    // Two of one kind make nothing.
    assert!(shared(&[id("wild_berries"), id("ashberry")]).is_empty(), "two berries");
    // A leaf and a berry share Heal.
    assert_eq!(shared(&[id("bloodleaf"), id("wild_berries")]), vec![Alch::Heal]);
    // With a root: Heal and Restore stamina (helpful first).
    let s = shared(&[id("bloodleaf"), id("wild_berries"), id("sweetroot")]);
    assert!(s.contains(&Alch::Heal) && s.contains(&Alch::RestoreStamina) && s.len() == 2, "{s:?}");
    // Harm comes along too, listed after the help.
    let s = shared(&[id("nightberry"), id("mirebloom")]);
    assert_eq!(s, vec![Alch::Shadow, Alch::Poison]);
}

#[test]
fn skill_and_tools_make_it_stronger_and_the_harm_briefer_but_never_gone() {
    let fx = [Alch::Shadow, Alch::Poison];
    let bare = Mix::brew(&fx, 10.0, Kit::default());
    let c = Some(Grade::Masterwork);
    let best = Mix::brew(&fx, 100.0, Kit { mortar: c, retort: c, calcinator: c, alembic: c });
    assert!(bare.power < 1.0 && bare.bad_for > 1.2, "a bare table, a raw hand: weak, the harm long ({bare:?})");
    assert!(best.power > 1.8 && best.good_for > 1.8, "{best:?}");
    assert!(best.bad_for > 0.0 && best.bad_for <= 0.25, "the harm a flicker, not gone ({})", best.bad_for);
    assert!(best.grade() > bare.grade());
    // Each tool does its own part.
    let retort = Mix::brew(&fx, 50.0, Kit { retort: Some(Grade::Common), ..Kit::default() });
    let plain = Mix::brew(&fx, 50.0, Kit::default());
    assert!(retort.bad_for < plain.bad_for && retort.power == plain.power);
    let alembic = Mix::brew(&fx, 50.0, Kit { alembic: Some(Grade::Common), ..Kit::default() });
    assert!(alembic.power > plain.power && alembic.bad_for == plain.bad_for);
    // What it does scales with it.
    let poison = |m: &Mix| m.effects().iter().find(|e| e.does == gahturiyu_sim::sim::effects::Does::Poison).map(|e| match e.lasts {
        gahturiyu_sim::sim::effects::Lasts::Secs(s) => s,
        _ => 0.0,
    });
    assert!(poison(&best).unwrap() < poison(&bare).unwrap());
}

#[test]
fn bottles_stack_only_when_their_mixes_are_the_same() {
    let pot = id("brewed_potion");
    let a = Mix::brew(&[Alch::Heal], 40.0, Kit::default());
    let b = Mix::brew(&[Alch::Heal], 90.0, Kit::default());
    let mut g = Gear::default();
    g.add_piece(pot, Piece::bottle(a));
    g.add_piece(pot, Piece::bottle(a));
    g.add_piece(pot, Piece::bottle(b));
    assert_eq!(g.bag.len(), 2);
    assert_eq!(g.bag[0].1, 2, "the same mix stacks");
    assert_eq!(g.bottles().len(), 3);
    assert!(g.take_mix(&b));
    assert!(!g.take_mix(&b), "only the one");
    assert_eq!(g.bottles(), vec![a, a]);
}

/// The squad at the nearest alchemy table, `m` (who's taken up alchemy)
/// beside it, nobody carrying anything to brew with.
fn at_a_table(seed: u64) -> (World, u32) {
    let mut w = worldgen::generate(seed);
    let here = w.squad.pos;
    let table = w.stations.iter().filter(|s| s.1 == Station::AlchemyTable).min_by(|a, b| a.0.dist(here).total_cmp(&b.0.dist(here))).expect("an alchemy table somewhere").0;
    w.teleport_squad(table.add(V2::new(1.0, 0.0)));
    for (k, _) in w.squad.members.clone().iter().enumerate() {
        w.squad.at[k] = table.add(V2::new(1.0, 0.3 * k as f32));
        w.squad.goal[k] = w.squad.at[k];
    }
    let m = w.squad.members[0];
    for p in w.squad.members.clone() {
        let d = w.people[p as usize].detail.as_mut().unwrap();
        d.gear.bag.retain(|e| items::ingredient(e.0).is_none());
    }
    let p = &mut w.people[m as usize];
    let d = p.detail.as_mut().unwrap();
    if !d.crafts.contains(&Skill::Alchemy) {
        d.crafts.push(Skill::Alchemy);
    }
    p.stats.set_skill(Skill::Alchemy, 40.0);
    (w, m)
}

fn give(w: &mut World, who: u32, key: &str, n: u16) {
    w.people[who as usize].detail.as_mut().unwrap().gear.add(id(key), n);
}

fn wait(w: &mut World, secs: f64) {
    let end = w.time + secs;
    while w.time < end - 1e-6 {
        w.step(5.0f64.min(end - w.time));
    }
}

#[test]
fn brewing_at_a_table_fills_a_bottle_teaches_and_remembers() {
    let (mut w, m) = at_a_table(1);
    assert!(w.table_for(m).is_some());
    give(&mut w, m, "bloodleaf", 2);
    give(&mut w, m, "wild_berries", 2);
    let ings = [id("bloodleaf"), id("wild_berries")];
    assert_eq!(w.known_effects(m, ings[0]), [None; 4], "nothing known yet");
    let skill = w.people[m as usize].stats.skill(Skill::Alchemy);
    w.brew(m, &ings, 2).expect("can brew");
    assert_eq!(w.count_of(m, "bloodleaf"), 1, "what goes in goes in at the start");
    assert_eq!(w.brew_blockers(m, &ings), vec![NoBrew::Busy]);
    wait(&mut w, 2.0 * BREW_SECS + 30.0);
    let bottles = w.people[m as usize].detail.as_ref().unwrap().gear.bottles();
    assert_eq!(bottles.len(), 2, "two bottles, one after the other");
    assert_eq!(bottles[0], bottles[1]);
    assert_eq!(bottles[0].carries().collect::<Vec<_>>(), vec![Alch::Heal]);
    assert_eq!(w.count_of(m, "bloodleaf"), 0);
    assert!(w.people[m as usize].stats.skill(Skill::Alchemy) > skill);
    // What they shared is known now, on both.
    assert!(w.known_effects(m, ings[0]).contains(&Some(Alch::Heal)));
    assert!(w.known_effects(m, ings[1]).contains(&Some(Alch::Heal)));
    assert_eq!(w.known_effects(m, ings[0]).iter().flatten().count(), 1, "and only that");
    // Remembered, by name.
    let lore = w.lore_of(m).unwrap();
    assert_eq!(lore.mixes.len(), 1);
    assert_eq!(lore.mixes[0].name, "Draught of heal");
    // Drunk, it heals and adds its toxicity.
    let t0 = w.toxicity_of(m).unwrap();
    let k = w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().position(|e| e.2.and_then(|p| p.mix).is_some()).unwrap();
    assert!(w.drink_entry(m, k));
    assert!(w.toxicity_of(m).unwrap() > t0);
    assert_eq!(w.people[m as usize].detail.as_ref().unwrap().gear.bottles().len(), 1);
}

#[test]
fn no_table_no_brew_and_a_mortar_in_the_pack_wont_do() {
    let (mut w, m) = at_a_table(1);
    give(&mut w, m, "bloodleaf", 1);
    give(&mut w, m, "wild_berries", 1);
    give(&mut w, m, "mortar_and_pestle", 1);
    let far = w.squad.pos.add(V2::new(400.0, 0.0));
    w.teleport_squad(far);
    let ings = [id("bloodleaf"), id("wild_berries")];
    assert_eq!(w.brew(m, &ings, 1), Err(NoBrew::NoTable));
    assert!(w.bench_for(m).is_none(), "no bench in the pack");
    // Two of one kind, or one alone, won't brew either.
    let (mut w, m) = at_a_table(1);
    give(&mut w, m, "wild_berries", 1);
    give(&mut w, m, "ashberry", 1);
    assert!(w.brew_blockers(m, &[id("wild_berries"), id("ashberry")]).contains(&NoBrew::OneKind));
    assert!(w.brew_blockers(m, &[id("wild_berries")]).contains(&NoBrew::Count));
}

#[test]
fn a_brew_that_shares_nothing_teaches_that_much_and_fills_nothing() {
    let (mut w, m) = at_a_table(1);
    give(&mut w, m, "frostleaf", 1);
    give(&mut w, m, "puffcap", 1);
    let ings = [id("frostleaf"), id("puffcap")];
    assert!(shared(&ings).is_empty());
    w.brew(m, &ings, 1).unwrap();
    wait(&mut w, BREW_SECS + 30.0);
    assert!(w.people[m as usize].detail.as_ref().unwrap().gear.bottles().is_empty());
    assert_eq!(w.count_of(m, "frostleaf"), 0, "used up all the same");
}

#[test]
fn tasting_teaches_one_effect_and_sickens_a_little() {
    let (mut w, m) = at_a_table(1);
    give(&mut w, m, "nightberry", 5);
    let ing = id("nightberry");
    let t0 = w.toxicity_of(m).unwrap();
    for n in 1..=4 {
        assert!(w.taste(m, ing));
        assert_eq!(w.known_effects(m, ing).iter().flatten().count(), n, "one more each taste");
    }
    assert!(w.toxicity_of(m).unwrap() > t0);
    // All known: a fifth taste teaches nothing.
    assert!(w.taste(m, ing));
    assert_eq!(w.known_effects(m, ing).iter().flatten().count(), 4);
    assert_eq!(w.count_of(m, "nightberry"), 0);
}

#[test]
fn a_town_alchemists_table_has_every_tool_and_a_base_has_what_its_store_holds() {
    let (w, m) = at_a_table(1);
    let at = w.table_for(m).unwrap();
    let kit = w.table_kit(at);
    assert!(kit.mortar.is_some(), "every table has a mortar at least: {kit:?}");
}

#[test]
fn brewing_comes_out_the_same_however_the_world_is_stepped() {
    let (mut w0, m) = at_a_table(2);
    give(&mut w0, m, "bloodleaf", 3);
    give(&mut w0, m, "wild_berries", 3);
    w0.brew(m, &[id("bloodleaf"), id("wild_berries")], 3).unwrap();
    let run = |step: f64| {
        let mut w = w0.clone();
        let end = w.time + 3.0 * BREW_SECS + 60.0;
        while w.time < end - 1e-6 {
            w.step(step.min(end - w.time));
        }
        (w.people[m as usize].detail.as_ref().unwrap().gear.bottles(), w.lore_of(m).cloned())
    };
    assert_eq!(run(5.0), run(200.0));
}
