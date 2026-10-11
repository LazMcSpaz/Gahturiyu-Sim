//! Materials, grades, made pieces, wear, crafting and the trade in them.

use gahturiyu_sim::sim::{
    items::{self, catalogue, item, variant, Kind},
    materials::{Grade, Material},
};

#[test]
fn the_catalogue_holds_every_form_in_its_materials_and_grades() {
    let cat = catalogue();
    assert!(cat.defs.len() > 400 && cat.defs.len() < 60_000, "{} entries", cat.defs.len());
    let sword = items::id("longsword");
    // The usual, common version is the plain item itself.
    assert_eq!(variant(sword, Material::Forgeiron, Material::None, Grade::Common), Some(sword));
    let fine = variant(sword, Material::Forgeiron, Material::None, Grade::Fine).unwrap();
    let edge = variant(sword, Material::Edgeglass, Material::None, Grade::Common).unwrap();
    let both = variant(sword, Material::Edgeglass, Material::Forgeiron, Grade::Common).unwrap();
    let w = |id| *item(id).weapon().unwrap();
    assert!(w(fine).cut > w(sword).cut && item(fine).value > item(sword).value);
    assert!(w(edge).cut > w(sword).cut, "Edgeglass takes the keener edge");
    assert!(w(sword).pierce > w(edge).pierce, "Forgeiron gets through armour better");
    assert!(w(both).cut > w(sword).cut && w(both).pierce >= w(sword).pierce, "the two-tradition sword has both");
    assert!(!items::repairable(edge) && items::repairable(sword));
    println!("{} · {} · {}", item(fine).name, item(edge).name, item(both).name);
    // Nacre turns cuts but not blows.
    let coat = items::id("scale_hauberk");
    let nacre = variant(coat, Material::Nacre, Material::Slatewing, Grade::Common).unwrap();
    let (a, b) = (item(coat).armor().unwrap(), item(nacre).armor().unwrap());
    assert!(b.cut > a.cut && b.blunt < a.blunt);
    assert!(matches!(item(items::id("manual_smithing")).kind, Kind::Manual(_)));
}

use gahturiyu_sim::sim::{
    combat::Fighter,
    crafting::{Cannot, Station, RECIPES},
    geo::V2,
    jobs::{Good, Job, PlaceKind},
    materials::{Craft, Tradition},
    stats::{Calling, Skill},
    wear::fresh,
    world::{DAY, HOUR},
    worldgen, World,
};
use gahturiyu_sim::sim::items::Slot;

fn run(w: &mut World, secs: f64) {
    let n = (secs / (HOUR / 2.0)).round() as usize;
    for _ in 0..n {
        w.step(HOUR / 2.0);
    }
}

fn give(w: &mut World, who: u32, key: &str, n: u16) {
    w.people[who as usize].detail.as_mut().unwrap().gear.add(items::id(key), n);
}

fn member(w: &World, c: Calling) -> u32 {
    *w.squad.members.iter().find(|&&m| w.people[m as usize].stats.calling == c).unwrap()
}

/// Put a made piece in someone's hand.
fn wield(w: &mut World, who: u32, id: items::ItemId) {
    let t = w.time;
    let d = w.people[who as usize].detail.as_mut().unwrap();
    d.gear.add_piece(id, fresh(id, t));
    let k = d.gear.bag.len() - 1;
    assert!(w.equip_entry(who, k));
}

/// Wear from a fight: `blows` struck with whatever's in the hand.
fn strike(w: &mut World, who: u32, blows: f32) {
    let mut f = Fighter::from_person(&w.people[who as usize], 0, V2::new(0.0, 0.0), w.time);
    f.weapon_wear = blows;
    let t = w.time;
    w.wear_gear(&f, t);
}

#[test]
fn a_starting_squad_wears_the_shared_basics() {
    let w = worldgen::generate(1);
    for &m in &w.squad.members {
        let d = w.people[m as usize].detail.as_ref().unwrap();
        assert!(d.gear.equipped().count() >= 3);
        for id in d.gear.equipped() {
            assert_eq!(items::info(id).main.def().tradition, Tradition::Shared, "{} isn't a basic", item(id).name);
        }
        assert!(!d.crafts.is_empty(), "everyone starts with a craft of their own");
    }
}

#[test]
fn edgeglass_wears_down_and_shatters_and_cant_be_mended() {
    let mut w = worldgen::generate(1);
    let m = member(&w, Calling::Warrior);
    let blade = variant(items::id("knife"), Material::Edgeglass, Material::None, Grade::Common).unwrap();
    wield(&mut w, m, blade);
    let (full, most) = w.wear_of(m, Slot::MainHand).unwrap();
    assert_eq!(full, most);
    strike(&mut w, m, 100.0);
    let (left, _) = w.wear_of(m, Slot::MainHand).unwrap();
    assert!(left < most && left > 0.0, "it wears ({left} of {most})");
    // No one can mend it: not a town's smith, not the squad.
    let smith = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).find(|&p| w.life(p).job == Job::Smith).unwrap();
    assert_eq!(w.pay_to_mend(smith, m, Slot::MainHand), Err("that can't be mended"));
    assert_eq!(w.mend_myself(m, m, Slot::MainHand), Err("that can't be mended"));
    // Worn to nothing, it's gone.
    strike(&mut w, m, 5000.0);
    assert_eq!(w.people[m as usize].detail.as_ref().unwrap().gear.in_slot(Slot::MainHand), None);
    assert!(w.log.iter().any(|l| l.1.contains("shatters")));
}

#[test]
fn anything_worn_to_nothing_is_gone() {
    let mut w = worldgen::generate(1);
    let m = member(&w, Calling::Warrior);
    let sword = items::id("longsword");
    wield(&mut w, m, sword);
    assert!(items::repairable(sword));
    let bag = w.people[m as usize].detail.as_ref().unwrap().gear.bag.len();
    strike(&mut w, m, 10_000.0);
    let d = w.people[m as usize].detail.as_ref().unwrap();
    assert_eq!(d.gear.in_slot(Slot::MainHand), None);
    assert_eq!(d.gear.bag.len(), bag, "not back in the pack either");
    assert!(w.log.iter().any(|l| l.1.contains("breaks")));
}

/// Hours of the day each step, until `f` holds (at most two days).
fn until(w: &mut World, f: impl Fn(&World) -> bool) {
    for _ in 0..96 {
        if f(w) {
            return;
        }
        w.step(HOUR / 2.0);
    }
    panic!("never happened");
}

#[test]
fn only_a_town_with_smiths_mends_forgeiron() {
    let mut w = worldgen::generate(1);
    let sword = items::id("longsword");
    // A Roduro-led town with no fire-metal workers (its smiths taken off to
    // other work), and a mixed town whose workyard has them.
    let roduro = (0..w.settlements.len()).max_by(|&a, &b| {
        let share = |t: usize| w.society.communities[w.society.towns[t].shore as usize].blend.share[0];
        share(a).total_cmp(&share(b))
    }).unwrap() as u16;
    for p in w.settlements[roduro as usize].residents.clone() {
        if matches!(w.life(p).job, Job::Smith | Job::Armourer) {
            w.society.lives[p as usize].job = Job::Farmer;
        }
    }
    let yard = (0..w.settlements.len() as u16)
        .find(|&t| t != roduro && w.society.towns[t as usize].places.iter().any(|p| p.kind == PlaceKind::Workyard) && w.settlements[t as usize].residents.iter().any(|&p| w.life(p).job == Job::Smith))
        .expect("a town with a workyard");
    let mut mended_in_yard = false;
    for _ in 0..20 {
        assert_eq!(w.mender_for(roduro, sword), None, "no one there works forgeiron");
        mended_in_yard |= w.mender_for(yard, sword).is_some();
        w.step(HOUR);
    }
    assert!(mended_in_yard, "the workyard's smiths mend it in working hours");
    // Basics, though, anyone with the handcraft can mend.
    assert!(items::craft_of(items::id("hide_coat")) == Craft::Handcraft);
}

#[test]
fn a_craft_starts_with_a_teacher_or_a_manual_then_practice_raises_it() {
    let mut w = worldgen::generate(1);
    let mage = member(&w, Calling::Mage);
    assert!(!w.knows_craft(mage, Craft::Smithing));
    // At a forge, with the iron: still can't, without the craft.
    let k = w.squad.index(mage).unwrap();
    let forge = w.stations.iter().filter(|s| s.1 == Station::Forge).min_by(|a, b| a.0.dist(w.squad.at[k]).total_cmp(&b.0.dist(w.squad.at[k]))).unwrap().0;
    w.teleport_squad(forge.add(V2::new(1.0, 0.0)));
    let k = w.squad.index(mage).unwrap();
    w.squad.at[k] = forge.add(V2::new(1.0, 0.0));
    w.squad.goal[k] = w.squad.at[k];
    give(&mut w, mage, "iron_ingot", 3);
    give(&mut w, mage, "leather", 3);
    let knife = RECIPES.iter().position(|r| r.output == "knife").unwrap();
    assert_eq!(w.can_craft(mage, knife), Err(Cannot::Unknown(Craft::Smithing)));
    // Practice alone can't start it: there's no way to practise what you can't make.
    // A manual takes them in.
    give(&mut w, mage, "manual_smithing", 1);
    assert!(w.use_item(mage, items::id("manual_smithing")));
    assert!(!w.knows_craft(mage, Craft::Smithing), "not until they've studied it");
    for _ in 0..(9 * 60) {
        w.step(60.0);
    }
    assert!(w.knows_craft(mage, Craft::Smithing));
    assert_eq!(w.count_of(mage, "manual_smithing"), 1, "the manual is kept");
    // Now practice raises it.
    let before = w.people[mage as usize].stats.skill(Skill::Smithing);
    assert_eq!(w.can_craft(mage, knife), Ok(()));
    w.start_craft(mage, knife).unwrap();
    for _ in 0..100 {
        w.step(1.0);
    }
    assert!(w.people[mage as usize].stats.skill(Skill::Smithing) > before);
    let made = w.people[mage as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| items::info(e.0).form == items::id("knife")).count();
    let botched = w.log.iter().any(|l| l.1.contains("botches"));
    assert!(made == 1 || botched);
}

#[test]
fn a_crafter_at_work_teaches_their_trade_for_coin() {
    let mut w = worldgen::generate(1);
    let hunter = member(&w, Calling::Hunter);
    give(&mut w, hunter, "coin", 500);
    let weaver = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).find(|&p| w.life(p).job == Job::Weaver).unwrap();
    until(&mut w, |w| w.at_work(weaver, w.time));
    assert!(!w.knows_craft(hunter, Craft::Weaving));
    let (craft, price, style) = w.craft_lesson(weaver, hunter).expect("they'll teach");
    assert_eq!(craft, Craft::Weaving);
    let coin = w.squad_count(items::id("coin"));
    w.take_craft_lesson(weaver, hunter).unwrap();
    assert_eq!(w.squad_count(items::id("coin")), coin - price);
    let (gain, hours, _) = style.lesson();
    for _ in 0..((hours * 60.0) as usize + 2) {
        w.step(60.0);
    }
    assert!(w.knows_craft(hunter, Craft::Weaving));
    assert!(w.people[hunter as usize].stats.skill(Skill::Weaving) >= gain.min(10.0));
}

#[test]
fn a_town_without_ore_makes_no_forgeiron_until_ore_comes() {
    let mut w = worldgen::generate(1);
    // No caravans run in this world.
    for l in &mut w.society.lives {
        if l.job == Job::Caravaner {
            l.job = Job::Labourer;
        }
    }
    let n = w.settlements.len() as u16;
    let town = (0..n)
        .find(|&t| w.source(t, Good::Ore) == 0.0 && w.stock_now(t, Good::Ingots) < 0.5 && w.settlements[t as usize].residents.iter().any(|&p| w.life(p).job == Job::Smith))
        .expect("a town with smiths but no ore");
    let forgeiron = |w: &World| {
        let tl = &w.society.towns[town as usize];
        tl.shelf.iter().filter(|s| items::info(s.item).main == Material::Forgeiron || items::info(s.item).second == Material::Forgeiron).count() + w.stock_now(town, Good::Ingots).floor() as usize
    };
    run(&mut w, 3.0 * DAY);
    assert_eq!(forgeiron(&w), 0, "nothing to smelt");
    // Ore and charcoal come in (as a caravan would bring them).
    let st = &mut w.society.towns[town as usize].stock;
    st[Good::Ore.index()].base += 60.0;
    st[Good::Charcoal.index()].base += 60.0;
    run(&mut w, 3.0 * DAY);
    assert!(forgeiron(&w) > 0, "the smiths get to work");
}

#[test]
fn ore_is_cheaper_where_it_is_mined() {
    let mut w = worldgen::generate(1);
    run(&mut w, 2.0 * DAY);
    let n = w.settlements.len() as u16;
    let mine = (0..n).max_by(|&a, &b| w.source(a, Good::Ore).total_cmp(&w.source(b, Good::Ore))).unwrap();
    let dry: Vec<u16> = (0..n).filter(|&t| w.source(t, Good::Ore) == 0.0).collect();
    assert!(!dry.is_empty());
    let ore = items::id("iron_ore");
    for t in dry {
        assert!(w.price_factor(mine, Good::Ore) < w.price_factor(t, Good::Ore), "ore dearer where it's mined?");
        assert!(w.worth_in(mine, ore, None) < w.worth_in(t, ore, None));
    }
}

#[test]
fn a_grown_order_is_ready_on_time_and_dies_with_its_tender() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    give(&mut w, m, "coin", 2000);
    // Three weeks' wait: enough to eat (starving kills now, B2).
    for &k in &w.squad.members.clone() {
        give(&mut w, k, "salted_meat", 40);
    }
    let tenders: Vec<u32> = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).filter(|&p| w.life(p).job == Job::StoneTender).collect();
    let (a, b) = (tenders[0], *tenders.iter().find(|&&p| w.people[p as usize].home != w.people[tenders[0] as usize].home).unwrap());
    for t in [a, b] {
        let town = w.people[t as usize].home.unwrap();
        let st = &mut w.society.towns[town as usize].stock;
        for g in [Good::Rock, Good::Ash, Good::Leather] {
            st[g.index()].base += 40.0;
        }
    }
    until(&mut w, |w| w.at_work(a, w.time) && w.at_work(b, w.time));
    let helm = RECIPES.iter().position(|r| r.output == "iron_helm" && r.main == Material::Slatewing).unwrap();
    assert!(w.order_options(a).iter().any(|o| o.0 == helm));
    let coin = w.squad_count(items::id("coin"));
    let t0 = w.time;
    let ka = w.place_order(a, helm).unwrap();
    let price = w.order_options(a).iter().find(|o| o.0 == helm).map(|o| o.1).unwrap_or(0);
    let _ = price;
    let deposit = w.orders[ka].deposit;
    assert_eq!(w.squad_count(items::id("coin")), coin - deposit);
    assert!((w.orders[ka].ready_at - (t0 + 21.0 * DAY)).abs() < 1.0, "Slatewing takes three weeks");
    let kb = w.place_order(b, helm).unwrap();
    let paid = deposit + w.orders[kb].deposit;
    // Tender b dies: their order and its deposit are lost at the next dawn.
    w.people[b as usize].dead = true;
    run(&mut w, DAY);
    assert!(w.orders_with(b).is_empty());
    assert_eq!(w.squad_count(items::id("coin")), coin - paid, "no deposit back");
    // Tender a's comes due: not before its time (a day later for each day they were away).
    let k = w.orders_with(a)[0];
    let due = w.orders[k].ready_at;
    assert!(due >= t0 + 21.0 * DAY && ((due - t0 - 21.0 * DAY) / DAY).fract().abs() < 1e-6);
    let left = due - w.time - HOUR;
    w.step(left);
    let k = w.orders_with(a)[0];
    assert!(!w.order_ready(k));
    let left = w.orders[k].ready_at - w.time + 1.0;
    w.step(left);
    let k = w.orders_with(a)[0];
    assert!(w.order_ready(k));
    until(&mut w, |w| w.at_work(a, w.time));
    let k = w.orders_with(a)[0];
    let id = w.collect_order(k).unwrap();
    assert_eq!(items::info(id).main, Material::Slatewing);
    let d = w.people[m as usize].detail.as_ref().unwrap();
    let e = d.gear.bag.iter().find(|e| e.0 == id).expect("in the pack");
    assert_eq!(e.2.unwrap().mark.unwrap().maker, a, "it carries the Tender's mark");
}

#[test]
fn towns_make_marked_pieces_and_a_stamp_is_worth_more() {
    let mut w = worldgen::generate(1);
    run(&mut w, 2.0 * DAY);
    let shelves: Vec<_> = w.society.towns.iter().flat_map(|t| t.shelf.iter().copied()).collect();
    assert!(shelves.len() > 50, "the towns' crafters are making things: {}", shelves.len());
    assert!(shelves.iter().any(|s| s.piece.and_then(|p| p.mark).map(|m| m.stamped).unwrap_or(false)), "some of it is stamped");
    assert!(shelves.iter().any(|s| items::info(s.item).grade == Grade::Fine));
    // Every made piece records its maker and town.
    for s in &shelves {
        if let Some(p) = s.piece {
            let m = p.mark.expect("a mark");
            assert!(m.town.is_some() && w.life(m.maker).job.craft().is_some() || w.people[m.maker as usize].dead);
        }
    }
    let s = shelves.iter().find(|s| s.piece.and_then(|p| p.mark).map(|m| m.stamped).unwrap_or(false)).unwrap();
    let mut plain = s.piece.unwrap();
    plain.mark.as_mut().unwrap().stamped = false;
    let town = s.piece.unwrap().mark.unwrap().town.unwrap();
    assert!(w.worth_in(town, s.item, s.piece.as_ref()) > w.worth_in(town, s.item, Some(&plain)));
}
