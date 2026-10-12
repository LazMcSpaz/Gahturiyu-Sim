//! The items design's new fight rules (`docs/items-design.md`, 0.2): each
//! one does what it says, from the same full rules wherever the fight is.
//! Piece A: bleeding and bandages, stagger and knockdown, blunt knocks out,
//! sneak multipliers.

use gahturiyu_sim::sim::{
    body::Part,
    combat::{self, Act, Battle, Fighter},
    effects::Does,
    geo::V2,
    items::{self, ArmorDef, WeaponDef},
    person::Person,
    race::Race,
    stats::{Calling, Skill},
    worldgen, World,
};

fn person(id: u32, skills: &[(Skill, f32)]) -> Person {
    let mut p = Person::summary(id, 1000 + id as u64 * 17, Race::Qotiro, None);
    p.specialize(Calling::Warrior, skills, 400.0);
    p.ensure_detail();
    p
}

fn weapon(key: &str) -> WeaponDef {
    *items::item(items::id(key)).weapon().unwrap()
}

/// Two fighters a step apart; nobody thinks for themselves unless told to.
fn pair(seed: u64) -> Battle {
    let (a, b) = (person(1, &[(Skill::Blade, 60.0), (Skill::Blunt, 60.0)]), person(2, &[(Skill::Dodge, 5.0)]));
    let mut fa = Fighter::from_person(&a, 0, V2::new(0.0, 0.0), 0.0);
    let mut fb = Fighter::from_person(&b, 1, V2::new(1.2, 0.0), 0.0);
    fa.think_at = f64::INFINITY;
    fb.think_at = f64::INFINITY;
    Battle::new(0, seed, 0.0, vec![fa, fb], vec!["A".into(), "B".into()])
}

/// A swings at B once with `w` and the blow is resolved. B's health is put
/// back first, so many blows can be counted without anyone going down.
fn one_blow(b: &mut Battle, w: WeaponDef) {
    b.fighters[0].weapon = w;
    let fresh = b.fighters[1].max_hp;
    b.fighters[1].hp = fresh;
    b.fighters[1].ko = false;
    let lands = b.time + 0.05;
    b.fighters[0].act = Act::Swing { target: 1, lands };
    for _ in 0..3 {
        b.tick();
        if !matches!(b.fighters[0].act, Act::Swing { .. }) {
            break;
        }
    }
    b.fighters[0].act = Act::Idle;
}

// ---- Rule 1: bleeding ---------------------------------------------------------

#[test]
fn a_bleed_never_stops_by_itself() {
    let mut b = pair(3);
    b.open_bleed(1, 0.3);
    let start = b.fighters[1].hp[Part::Torso as usize];
    for _ in 0..300 {
        b.tick();
    }
    let lost = start - b.fighters[1].hp[Part::Torso as usize];
    assert!((lost - 9.0).abs() < 0.2, "30 s at 0.3 a second takes 9 from the torso, took {lost}");
    assert!(b.fighters[1].bleeding() > 0.0, "still bleeding");
    // Left long enough, it puts them down, then kills them.
    while !b.fighters[1].ko {
        b.tick();
    }
    assert!(!b.fighters[1].dead);
    assert!(b.log.iter().any(|l| l.1.contains("collapses from loss of blood")), "down first");
    for _ in 0..20_000 {
        b.tick();
        if b.fighters[1].dead {
            break;
        }
    }
    assert!(b.fighters[1].dead, "bled to death");
}

#[test]
fn edges_open_bleeds_and_clubs_never_do() {
    let mut b = pair(5);
    let mut bled = 0;
    for _ in 0..80 {
        b.staunch(1);
        one_blow(&mut b, weapon("longsword"));
        bled += (b.fighters[1].bleeding() > 0.0) as u32;
    }
    assert!(bled >= 8, "a longsword opens bleeds ({bled} of 80)");
    let mut b = pair(5);
    for _ in 0..80 {
        one_blow(&mut b, weapon("club"));
        assert_eq!(b.fighters[1].bleeding(), 0.0, "a club opens no bleeds");
    }
}

#[test]
fn armour_that_stops_most_of_a_cut_stops_the_bleed() {
    let mut b = pair(5);
    b.fighters[1].armor = vec![ArmorDef { covers: &gahturiyu_sim::sim::body::PARTS, coverage: 1.0, cut: 0.7, blunt: 0.3, dodge_penalty: 0.0, conceal: 0.0 }];
    // (A blade that gets through none of it: Forgeiron's point would.)
    let mut sword = weapon("longsword");
    sword.pierce = 0.0;
    for _ in 0..80 {
        one_blow(&mut b, sword);
    }
    assert_eq!(b.fighters[1].bleeding(), 0.0, "a cut 70% stopped never bleeds");
}

#[test]
fn healing_closes_a_bleed() {
    let mut b = pair(3);
    b.open_bleed(1, 0.5);
    let draught = items::id("healing_draught");
    b.fighters[1].potions.push(draught);
    assert!(b.begin_drink(1, draught));
    for _ in 0..20 {
        b.tick();
    }
    assert_eq!(b.fighters[1].bleeding(), 0.0, "a healing draught stops the bleeding");
}

/// How many of `it` someone carries.
fn held(w: &World, m: u32, it: items::ItemId) -> u16 {
    w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == it).map(|e| e.1).sum()
}

/// A world just after a fight in which a squad member was left bleeding,
/// with or without the bandages they set out with.
fn after_a_fight(bandages: bool) -> Option<(World, u32)> {
    let bandage = items::id("bandage");
    for seed in 1..12u64 {
        let mut w = worldgen::generate(seed);
        if !bandages {
            for &m in &w.squad.members.clone() {
                let g = &mut w.people[m as usize].detail.as_mut().unwrap().gear;
                while g.take(bandage) {}
            }
        }
        let before: u16 = w.squad.members.iter().map(|&m| held(&w, m, bandage)).sum();
        let at = w.squad.pos.add(V2::new(18.0, 4.0));
        w.spawn_bandits(at, 3, false);
        w.step(0.1);
        for _ in 0..40_000 {
            w.step(0.25);
            if w.battles.is_empty() && w.next_battle > 0 {
                break;
            }
        }
        if !bandages {
            // (On their feet: someone out cold can't use anything.)
            if let Some(&m) = w.squad.members.iter().find(|&&m| w.bleeding(m) > 0.0 && !w.people[m as usize].dead && !w.is_down(m)) {
                return Some((w, m));
            }
            continue;
        }
        // With bandages: anyone cut was bound when it was over.
        let after: u16 = w.squad.members.iter().filter(|&&m| !w.people[m as usize].dead).map(|&m| held(&w, m, bandage)).sum();
        if after < before {
            if let Some(&m) = w.squad.members.iter().find(|&&m| !w.people[m as usize].dead) {
                return Some((w, m));
            }
        }
    }
    None
}

#[test]
fn after_a_fight_the_squad_binds_its_cuts() {
    let (w, _) = after_a_fight(true).expect("a fight where someone was cut and bound");
    for &m in &w.squad.members {
        assert_eq!(w.bleeding(m), 0.0, "everyone bound");
    }
    assert!(w.log.iter().any(|l| l.1.contains("binds")), "and it says so");
}

#[test]
fn a_bandage_from_the_pack_stops_a_bleed() {
    let (mut w, m) = after_a_fight(false).expect("someone in the squad left bleeding");
    // (Only this one bleeding: anyone else cut in the fight is bound already.)
    for &o in &w.squad.members.clone() {
        if o != m {
            w.people[o as usize].wounds.bleed = 0.0;
        }
    }
    let bandage = items::id("bandage");
    assert!(w.why_cant_use(m, bandage).is_some(), "no bandage on them");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(bandage, 2);
    assert_eq!(w.why_cant_use(m, bandage), None);
    assert!(w.use_item(m, bandage));
    assert_eq!(w.bleeding(m), 0.0);
    assert_eq!(held(&w, m, bandage), 1, "one used");
    if w.squad.members.iter().all(|&o| w.bleeding(o) == 0.0) {
        assert!(w.why_cant_use(m, bandage).is_some(), "nobody near bleeding now");
    }
}

#[test]
fn a_squadmate_binds_someone_on_an_order() {
    let (mut w, m) = after_a_fight(false).expect("someone in the squad left bleeding");
    let bandage = items::id("bandage");
    let other = *w.squad.members.iter().find(|&&o| o != m && !w.people[o as usize].dead && !w.is_down(o)).expect("someone standing");
    w.people[other as usize].detail.as_mut().unwrap().gear.add(bandage, 1);
    let said = w.order_bind(other, m);
    assert!(said.is_ok(), "{said:?}");
    for _ in 0..600 {
        if w.bleeding(m) == 0.0 {
            break;
        }
        w.step(0.5);
    }
    assert_eq!(w.bleeding(m), 0.0, "bound");
    assert_eq!(held(&w, other, bandage), 0);
}

#[test]
fn in_a_fight_they_bind_a_bleed_once_nobody_is_on_them() {
    let mut b = pair(3);
    let bandage = items::id("bandage");
    assert!(combat::is_bandage(bandage), "found by what it does");
    assert!(!combat::is_bandage(items::id("healing_draught")), "a draught is drunk, not bound on");
    b.fighters[1].bandages = vec![bandage];
    b.open_bleed(1, 0.4);
    // Their foe far off: they bind it (a few seconds).
    b.fighters[0].pos = V2::new(30.0, 0.0);
    b.fighters[1].think_at = 0.0;
    for _ in 0..80 {
        b.tick();
    }
    assert_eq!(b.fighters[1].bleeding(), 0.0, "bound");
    assert_eq!(b.fighters[1].used, vec![bandage]);
    assert!(b.log.iter().any(|l| l.1 == "B binds their wounds."));
}

#[test]
fn bleeding_out_of_a_fight_is_solved_on_the_clock() {
    let (w, m) = after_a_fight(false).expect("someone in the squad left bleeding");
    let p = &w.people[m as usize];
    let h = p.wounds.torso_reaches(2.0 * p.stats.max_hp(Part::Torso)).expect("bleeding");
    let due = p.wounds.at + h as f64 * 3600.0;
    // However the world is stepped, they die at the same moment.
    let died = |step: f64| {
        let mut w = w.clone();
        while w.time < due + 120.0 && !w.people[m as usize].dead {
            w.step(step);
        }
        let line = w.log.iter().find(|l| l.1.contains("has bled to death")).cloned();
        (w.people[m as usize].dead, line.map(|l| l.0))
    };
    let (a, b) = (died(1.0), died(47.0));
    assert!(a.0 && b.0, "bled to death");
    assert!((a.1.unwrap() - b.1.unwrap()).abs() < 1e-3, "at the same moment however stepped ({:?} vs {:?})", a.1, b.1);
    assert!((a.1.unwrap() - due).abs() < 1.0, "when the torso runs out ({} vs {due})", a.1.unwrap());
}

#[test]
fn a_healing_draught_stops_a_bleed_out_of_a_fight() {
    let (mut w, m) = after_a_fight(false).expect("someone in the squad left bleeding");
    let draught = items::id("healing_draught");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(draught, 1);
    assert!(w.use_item(m, draught));
    assert_eq!(w.bleeding(m), 0.0);
    let t = w.time;
    let torso = |w: &World| w.people[m as usize].wounds.lost_at(w.time)[Part::Torso as usize];
    let before = torso(&w);
    while w.time < t + 600.0 {
        w.step(5.0);
    }
    assert!(torso(&w) <= before, "no more blood lost");
    assert!(!w.people[m as usize].dead);
}

// ---- Rule 2: stagger and knockdown -------------------------------------------

#[test]
fn heavy_blunt_blows_stagger_and_floor_and_light_ones_dont() {
    let mut b = pair(9);
    let (mut staggered, mut floored) = (0, 0);
    for _ in 0..60 {
        // B is mid-swing each time: a stagger spoils it.
        b.fighters[1].statuses.retain(|s| s.does != Does::KnockDown);
        b.fighters[1].act = Act::Swing { target: 0, lands: b.time + 5.0 };
        one_blow(&mut b, weapon("stone_maul"));
        if b.fighters[1].has(Does::KnockDown).is_some() {
            floored += 1;
        } else if matches!(b.fighters[1].act, Act::Recover { .. }) {
            staggered += 1;
        }
    }
    assert!(floored >= 5, "a maul floors people ({floored} of 60)");
    assert!(staggered >= 5, "and staggers them ({staggered} of 60)");
    let mut b = pair(9);
    for _ in 0..60 {
        b.fighters[1].act = Act::Swing { target: 0, lands: b.time + 5.0 };
        one_blow(&mut b, weapon("knife"));
        assert!(matches!(b.fighters[1].act, Act::Swing { .. }), "a knife never staggers");
        assert!(b.fighters[1].has(Does::KnockDown).is_none());
    }
}

#[test]
fn floored_they_can_neither_dodge_nor_act_until_they_get_up() {
    let mut b = pair(9);
    b.knock_down(1, combat::KNOCKDOWN_SECS);
    assert!(b.fighters[1].helpless());
    for _ in 0..((combat::KNOCKDOWN_SECS / combat::DT) as usize + 2) {
        b.tick();
    }
    assert!(!b.fighters[1].helpless(), "up again after {} s", combat::KNOCKDOWN_SECS);
}

// ---- Rule 3: blunt knocks out ---------------------------------------------------

#[test]
fn blunt_puts_people_down_sooner_and_kills_them_later() {
    let (mut cut, mut club) = (pair(1), pair(1));
    let blows = |b: &mut Battle, c: f32, bl: f32| {
        let mut down_at = None;
        for n in 1..400 {
            b.wound_blow(1, Part::Torso, c, bl);
            if b.fighters[1].ko && down_at.is_none() {
                down_at = Some(n);
            }
            if b.fighters[1].dead {
                return (down_at.unwrap(), n);
            }
        }
        panic!("never died");
    };
    let (cut_down, cut_dead) = blows(&mut cut, 12.0, 0.0);
    let (club_down, club_dead) = blows(&mut club, 0.0, 12.0);
    assert!(club_down <= cut_down, "the club puts them down no later ({club_down} vs {cut_down})");
    assert!(club_dead - club_down > 2 * (cut_dead - cut_down), "and once down, takes far longer to kill ({} vs {} more blows)", club_dead - club_down, cut_dead - cut_down);
}

// ---- Rule 7: sneak multipliers --------------------------------------------------

#[test]
fn the_knife_has_the_worst_sneak_attack() {
    let knife = combat::sneak_mult(&weapon("knife"));
    for key in ["short_sword", "longsword", "club", "war_pick", "stone_maul", "spear", "glaive", "staff", "hatchet", "trident"] {
        assert!(knife > combat::sneak_mult(&weapon(key)) + 1.0, "the knife beats the {key} by far");
    }
    // And it shows in the wound.
    let hit = |key: &str, unaware: bool| {
        let mut b = pair(21);
        let mut total = 0.0;
        for _ in 0..30 {
            // (A blow wakes them: caught unawares afresh each time.)
            b.fighters[1].aware_at = if unaware { f64::INFINITY } else { 0.0 };
            one_blow(&mut b, weapon(key));
            total += b.fighters[1].damage_taken;
            b.fighters[1].damage_taken = 0.0;
        }
        total
    };
    let (open, sneak) = (hit("knife", false), hit("knife", true));
    assert!(sneak > open * 3.5, "a knife from behind does far more ({sneak:.0} vs {open:.0})");
}

#[test]
fn a_sap_from_behind_knocks_out_without_wounding() {
    // (The sap comes with the new forms; until then, one made here.)
    let sap = WeaponDef { skill: Skill::Blunt, cut: 0.0, blunt: 5.0, reach: 0.5, windup: 0.35, recover: 0.35, stuns: true, conceal: 0.8, ..items::FISTS };
    assert!(combat::stuns(&sap));
    for key in ["knife", "club", "short_sword"] {
        assert!(!combat::stuns(&weapon(key)), "the {key} doesn't stun");
    }
    let mut knocked = 0;
    for seed in 0..20 {
        let mut b = pair(100 + seed);
        b.fighters[1].aware_at = f64::INFINITY;
        b.fighters[0].weapon = sap;
        b.fighters[0].act = Act::Swing { target: 1, lands: b.time + 0.05 };
        for _ in 0..3 {
            b.tick();
        }
        let f = &b.fighters[1];
        if f.ko {
            knocked += 1;
            assert!(!f.dead);
            assert!(f.hp[Part::Torso as usize] >= f.max_hp[Part::Torso as usize] - 0.01, "the body isn't hurt");
            assert!(f.hp[Part::Head as usize] > -1.0, "only just out");
        }
    }
    assert!(knocked >= 5, "out cold most times it lands on the head or body ({knocked} of 20)");
}

// ---- Rule 4: keeping at bay ----------------------------------------------------

/// A spearman and a knifeman `gap` apart, both left to fight it out.
fn spear_and_knife(seed: u64, gap: f32) -> Battle {
    let (a, b) = (person(1, &[(Skill::Spear, 55.0)]), person(2, &[(Skill::Blade, 55.0)]));
    let mut fa = Fighter::from_person(&a, 0, V2::new(0.0, 0.0), 0.0);
    let mut fb = Fighter::from_person(&b, 1, V2::new(gap, 0.0), 0.0);
    fa.weapon = weapon("spear");
    fb.weapon = weapon("knife");
    fa.think_at = 0.0;
    fb.think_at = 0.0;
    Battle::new(0, seed, 0.0, vec![fa, fb], vec!["Spear".into(), "Knife".into()])
}

#[test]
fn a_spear_strikes_first_and_keeps_a_knife_at_bay() {
    let (mut first, mut pushed) = (0, 0);
    for seed in 0..20 {
        let mut b = spear_and_knife(300 + seed, 6.0);
        let mut who_first = None;
        for _ in 0..600 {
            b.tick();
            if who_first.is_none() {
                who_first = b.log.iter().find(|l| l.1.starts_with("Spear hits") || l.1.starts_with("Knife hits") || l.1.contains("dodges") || l.1.contains("blocks")).map(|l| l.1.starts_with("Spear") || l.1.starts_with("Knife dodges") || l.1.contains("blocks Spear"));
            }
            if b.over {
                break;
            }
        }
        first += who_first.unwrap_or(false) as u32;
        pushed += b.log.iter().any(|l| l.1.contains("keeps Knife at bay")) as u32;
    }
    assert!(first >= 15, "the spear gets the first blow in ({first} of 20)");
    assert!(pushed >= 8, "and drives the knife back ({pushed} of 20)");
}

#[test]
fn a_long_weapon_with_a_foe_inside_it_hits_soft() {
    let blows = |gap: f32| {
        let mut b = pair(41);
        b.fighters[1].pos = V2::new(gap, 0.0);
        let mut total = 0.0;
        for _ in 0..40 {
            one_blow(&mut b, weapon("spear"));
            total += b.fighters[1].damage_taken;
            b.fighters[1].damage_taken = 0.0;
        }
        total
    };
    let (inside, at_length) = (blows(0.7), blows(2.4));
    assert!(inside < at_length * 0.75, "crowded, the spear hits soft ({inside:.0} vs {at_length:.0})");
}

#[test]
fn a_crowded_spearman_steps_back() {
    let mut b = spear_and_knife(7, 0.7);
    b.fighters[1].think_at = f64::INFINITY;
    b.fighters[0].target = Some(1);
    b.fighters[0].think_at = f64::INFINITY;
    for _ in 0..10 {
        b.tick();
    }
    assert!(b.fighters[0].pos.dist(b.fighters[1].pos) > 0.9, "stepped back out ({:.2} m)", b.fighters[0].pos.dist(b.fighters[1].pos));
}

#[test]
fn a_spearman_who_cant_outpace_the_foe_stands_and_fights() {
    // Slowed (a heavy pack, say): stepping back would only be followed, the
    // whole fight long. They fight from where they stand instead.
    for seed in 0..6 {
        let mut b = spear_and_knife(40 + seed, 0.7);
        b.fighters[0].base_speed *= 0.6;
        let start = b.fighters[0].pos;
        let mut far = 0.0f32;
        for _ in 0..400 {
            b.tick();
            far = far.max(b.fighters[0].pos.dist(start));
            if b.over {
                break;
            }
        }
        assert!(far < 6.0, "seed {seed}: backed off {far:.1} m");
    }
}

// ---- Rule 5: tight spaces and sweeps ---------------------------------------------

#[test]
fn friends_pressing_close_make_it_tight() {
    let mut b = pair(5);
    assert_eq!(b.tightness(0), 0.0, "open ground");
    let mut friend = b.fighters[0].clone();
    friend.pos = V2::new(-0.9, 0.6);
    b.fighters.push(friend.clone());
    b.names.push("C".into());
    friend.pos = V2::new(-0.9, -0.6);
    b.fighters.push(friend);
    b.names.push("D".into());
    assert!(b.tightness(0) >= 0.6, "two friends at the elbows ({})", b.tightness(0));
}

#[test]
fn indoors_long_weapons_land_less_and_short_ones_dont_care() {
    // A real building, its walls round the fight (`World::around_fight`).
    let w = worldgen::generate(1);
    let d = (0..w.settlements[0].buildings.len() as u16).find_map(|i| gahturiyu_sim::sim::buildings::door_of(&w.settlements[0], i).filter(|d| d.half.x.min(d.half.y) >= 3.0)).expect("a big enough building");
    let landed = |key: &str, inside: bool| {
        let mut hits = 0;
        for seed in 0..12 {
            let mut b = pair(500 + seed);
            let at = if inside { w.free_spot(d.centre) } else { d.outside.add(d.outside.sub(d.centre).scale(3.0)) };
            b.fighters[0].pos = at;
            b.fighters[1].pos = at.add(V2::new(1.0, 0.0));
            b.around = w.around_fight(&b);
            if inside {
                assert!(b.tightness(0) >= 0.99, "indoors is tight");
            }
            for _ in 0..20 {
                one_blow(&mut b, weapon(key));
            }
            hits += b.log.iter().filter(|l| l.1.starts_with("A hits")).count();
        }
        hits
    };
    let (spear_in, spear_out) = (landed("spear", true), landed("spear", false));
    assert!((spear_in as f32) < spear_out as f32 * 0.9, "a spear lands less indoors ({spear_in} vs {spear_out})");
    let (knife_in, knife_out) = (landed("knife", true), landed("knife", false));
    assert!((knife_in as i32 - knife_out as i32).abs() <= (knife_out as i32) / 10 + 2, "a knife doesn't care ({knife_in} vs {knife_out})");
}

#[test]
fn a_maul_sweeps_foes_in_front_and_friends_only_when_tight() {
    let mut b = pair(61);
    b.fighters[0].weapon = weapon("stone_maul");
    // Two more foes in the arc, one behind.
    for (k, at) in [V2::new(1.1, 0.7), V2::new(1.1, -0.7), V2::new(-1.2, 0.0)].into_iter().enumerate() {
        let mut f = b.fighters[1].clone();
        f.pos = at;
        b.fighters.push(f);
        b.names.push(format!("F{k}"));
    }
    let mut struck = [0; 5];
    for _ in 0..30 {
        for j in 1..5 {
            b.fighters[j].hp = b.fighters[j].max_hp;
            b.fighters[j].ko = false;
            b.fighters[j].statuses.clear();
            b.fighters[j].damage_taken = 0.0;
        }
        b.fighters[0].act = Act::Swing { target: 1, lands: b.time + 0.05 };
        for _ in 0..2 {
            b.tick();
        }
        for j in 1..5 {
            struck[j] += (b.fighters[j].damage_taken > 0.0) as u32;
        }
    }
    // (A maul sweeps two in all: the one aimed at and the nearer beside.)
    assert!(struck[2] + struck[3] > 10, "one beside the target is caught too ({struck:?})");
    assert_eq!(struck[4], 0, "nobody behind");
    // A greatsword's sweep (three in all) catches both.
    b.fighters[0].weapon = WeaponDef { sweep: 3, ..weapon("stone_maul") };
    let mut both = 0;
    for _ in 0..30 {
        for j in 1..5 {
            b.fighters[j].hp = b.fighters[j].max_hp;
            b.fighters[j].ko = false;
            b.fighters[j].statuses.clear();
            b.fighters[j].damage_taken = 0.0;
        }
        b.fighters[0].act = Act::Swing { target: 1, lands: b.time + 0.05 };
        for _ in 0..2 {
            b.tick();
        }
        both += (b.fighters[2].damage_taken > 0.0 && b.fighters[3].damage_taken > 0.0) as u32;
    }
    assert!(both > 5, "a three-way sweep catches both beside ({both} of 30)");
    // A friend at the maul-man's elbow, in the arc: spared in the open...
    let mut b = pair(62);
    b.fighters[0].weapon = weapon("stone_maul");
    let mut friend = b.fighters[0].clone();
    friend.pos = V2::new(0.9, 0.9);
    b.fighters.push(friend);
    b.names.push("Friend".into());
    let swing = |b: &mut Battle| {
        let mut hurt = 0;
        for _ in 0..30 {
            b.fighters[2].damage_taken = 0.0;
            b.fighters[2].hp = b.fighters[2].max_hp;
            b.fighters[1].hp = b.fighters[1].max_hp;
            b.fighters[1].ko = false;
            b.fighters[0].act = Act::Swing { target: 1, lands: b.time + 0.05 };
            for _ in 0..2 {
                b.tick();
            }
            hurt += (b.fighters[2].damage_taken > 0.0) as u32;
        }
        hurt
    };
    assert_eq!(swing(&mut b), 0, "friends aren't swept in the open");
    // ...but caught when it's tight (two more friends pressing in).
    for at in [V2::new(-0.8, 0.5), V2::new(-0.8, -0.5)] {
        let mut f = b.fighters[2].clone();
        f.pos = at;
        b.fighters.push(f);
        b.names.push("Press".into());
    }
    assert!(swing(&mut b) > 0, "a friend in the arc is caught when it's tight");
}

// ---- Rule 6: the off-hand parry -------------------------------------------------

#[test]
fn an_off_hand_blade_parries_and_strikes_back() {
    // B holds a short sword and nothing else (their own kit might be a
    // spear or a shield, which takes the off hand).
    let count = |off: Option<WeaponDef>| {
        let (mut taken, mut back, mut parries) = (0.0, 0, 0);
        for seed in 0..10 {
            let mut b = pair(700 + seed);
            b.fighters[1].weapon = weapon("short_sword");
            b.fighters[1].shield = 0.0;
            b.fighters[1].offhand = off;
            b.fighters[1].stats.set_skill(Skill::Block, 60.0);
            for _ in 0..30 {
                let before = b.fighters[0].damage_taken;
                one_blow(&mut b, weapon("longsword"));
                taken += b.fighters[1].damage_taken;
                b.fighters[1].damage_taken = 0.0;
                back += (b.fighters[0].damage_taken > before) as u32;
            }
            parries += b.fx.iter().filter(|x| matches!(x.kind, combat::FxKind::Parry { .. })).count();
        }
        (taken, back, parries)
    };
    let (bare, bare_back, bare_parries) = count(None);
    let (with_knife, knife_back, parries) = count(Some(weapon("knife")));
    assert!(with_knife < bare * 0.92, "the knife in the off hand turns blows ({with_knife:.0} taken vs {bare:.0})");
    assert_eq!((bare_back, bare_parries), (0, 0));
    assert!(parries > 10, "parried with it ({parries})");
    assert!(knife_back > 0, "and now and then strikes back");
    // Not with a two-handed weapon in the other hand, nor a shield.
    let mut b = pair(9);
    b.fighters[1].offhand = Some(weapon("knife"));
    b.fighters[1].weapon = weapon("short_sword");
    b.fighters[1].shield = 0.0;
    assert!(b.fighters[1].offhand_up().is_some());
    b.fighters[1].weapon = weapon("spear");
    assert!(b.fighters[1].offhand_up().is_none(), "a two-handed weapon takes both hands");
    b.fighters[1].weapon = weapon("short_sword");
    b.fighters[1].shield = 0.4;
    assert!(b.fighters[1].offhand_up().is_none(), "a shield takes the off hand");
    for key in ["knife", "short_sword"] {
        assert!(combat::offhand_ok(&weapon(key)), "the {key} goes in the off hand");
    }
    for key in ["longsword", "hatchet", "club", "spear"] {
        assert!(!combat::offhand_ok(&weapon(key)), "the {key} doesn't");
    }
}

#[test]
fn a_knife_put_on_beside_a_sword_goes_in_the_off_hand() {
    use gahturiyu_sim::sim::{inventory::Gear, items::Slot};
    let (sword, knife, spear, shield) = (items::id("longsword"), items::id("knife"), items::id("spear"), items::id("round_shield"));
    let mut g = Gear::default();
    for it in [sword, knife, spear, shield] {
        g.add(it, 1);
    }
    g.equip(knife).unwrap();
    assert_eq!(g.in_slot(Slot::MainHand), Some(knife), "alone, a knife goes in the main hand");
    g.equip(sword).unwrap();
    assert_eq!(g.in_slot(Slot::MainHand), Some(sword));
    g.equip(knife).unwrap();
    assert_eq!(g.in_slot(Slot::OffHand), Some(knife), "beside a sword, in the off hand");
    let p = person(9, &[(Skill::Blade, 40.0)]);
    let mut p = p;
    p.detail.as_mut().unwrap().gear = g.clone();
    let f = Fighter::from_person(&p, 0, V2::new(0.0, 0.0), 0.0);
    assert!(f.offhand_up().is_some(), "and it parries in a fight");
    // A shield takes its place; a two-handed weapon takes both hands.
    g.equip(shield).unwrap();
    assert_eq!(g.in_slot(Slot::OffHand), Some(shield));
    g.equip(knife).unwrap();
    g.equip(spear).unwrap();
    assert_eq!(g.in_slot(Slot::OffHand), None, "a spear takes both hands");
}

// ---- Rule 8: thrown weapons ----------------------------------------------------

/// A hatchet-thrower (with a club to draw after) and a foe `gap` away.
fn thrower(seed: u64, gap: f32, spares: usize) -> Battle {
    let mut b = pair(seed);
    let hatchet = items::id("hatchet");
    b.fighters[0].weapon = weapon("hatchet");
    b.fighters[0].weapon_item = Some(hatchet);
    b.fighters[0].throws = vec![hatchet; spares];
    b.fighters[0].backup = Some((weapon("club"), "Club"));
    b.fighters[0].stats.set_skill(Skill::Marksman, 60.0);
    b.fighters[1].pos = V2::new(gap, 0.0);
    b.fighters[1].weapon = weapon("knife");
    b.fighters[0].think_at = 0.0;
    b
}

#[test]
fn out_of_reach_they_throw_and_it_lands_where_it_stops() {
    let mut b = thrower(11, 9.0, 2);
    b.fighters[1].think_at = f64::INFINITY;
    for _ in 0..60 {
        b.tick();
    }
    assert!(b.thrown.len() >= 2, "both spares thrown ({})", b.thrown.len());
    assert!(b.fighters[0].throws.is_empty());
    for th in &b.thrown {
        assert!(th.at.dist(b.fighters[1].pos) < 9.0, "lands near the target or past it ({:?})", th.at);
    }
    assert!(b.thrown[..2].iter().all(|th| !th.from_hand), "the spares went first");
    assert!(b.fx.iter().any(|x| matches!(x.kind, combat::FxKind::Throw { .. })), "seen flying");
    assert!(b.fighters[0].trained[Skill::Marksman as usize] > 0.0, "throwing is Marksman");
    // Then the one in the hand, and the club comes out.
    for _ in 0..40 {
        b.tick();
    }
    assert!(b.thrown.iter().any(|t| t.from_hand), "the hand's hatchet thrown too");
    assert_eq!(b.fighters[0].weapon_item, None);
    assert_eq!(b.fighters[0].weapon_name, "Club", "the club drawn");
}

#[test]
fn with_nothing_else_to_draw_they_keep_the_one_in_hand() {
    let mut b = thrower(12, 9.0, 0);
    b.fighters[0].backup = None;
    b.fighters[1].think_at = f64::INFINITY;
    for _ in 0..60 {
        b.tick();
    }
    assert!(b.thrown.is_empty(), "not thrown away");
}

#[test]
fn thrown_weapons_break_by_chance_on_what_they_hit() {
    let count = |breaks: f32| {
        let mut broke = 0;
        let mut thrown = 0;
        for seed in 0..15 {
            let mut b = thrower(400 + seed, 8.0, 3);
            b.fighters[1].think_at = f64::INFINITY;
            let mut w = weapon("hatchet");
            w.breaks = breaks;
            b.fighters[0].weapon = w;
            // (The spares are hatchets too: their numbers come from the item.)
            for _ in 0..90 {
                b.tick();
            }
            thrown += b.thrown.len();
            broke += b.thrown.iter().filter(|t| t.broke).count();
        }
        (broke, thrown)
    };
    let (broke, thrown) = count(0.08);
    assert!(thrown > 30);
    assert!(broke > 0 && broke < thrown / 2, "a hatchet breaks now and then ({broke} of {thrown})");
    assert!(b_never_breaks(), "a thing that never breaks doesn't");
}

fn b_never_breaks() -> bool {
    let mut b = thrower(77, 8.0, 3);
    b.fighters[1].think_at = f64::INFINITY;
    b.fighters[0].throws.clear();
    let net = items::id("net");
    b.fighters[0].throws = vec![net; 3];
    for _ in 0..90 {
        b.tick();
    }
    !b.thrown.is_empty() && b.thrown.iter().all(|t| !t.broke || item_breaks(t.item))
}

fn item_breaks(it: items::ItemId) -> bool {
    items::item(it).weapon().is_some_and(|w| w.breaks > 0.0)
}

#[test]
fn a_thrown_net_tangles() {
    let mut tangled = 0;
    for seed in 0..10 {
        let mut b = thrower(900 + seed, 6.0, 0);
        b.fighters[1].think_at = f64::INFINITY;
        b.fighters[0].throws = vec![items::id("net")];
        for _ in 0..40 {
            b.tick();
        }
        tangled += b.log.iter().any(|l| l.1.contains("net tangles")) as u32;
    }
    assert!(tangled > 0, "a net thrown tangles ({tangled} of 10)");
}

#[test]
fn after_a_fight_the_squads_throws_lie_about_to_be_gathered() {
    let hatchet = items::id("hatchet");
    for seed in 1..10u64 {
        let mut w = worldgen::generate(seed);
        // Everyone with three hatchets to throw.
        for &m in &w.squad.members.clone() {
            let d = w.people[m as usize].detail.as_mut().unwrap();
            d.gear.add(hatchet, 3);
        }
        let at = w.squad.pos.add(V2::new(16.0, 4.0));
        w.spawn_bandits(at, 3, false);
        w.step(0.1);
        for _ in 0..40_000 {
            w.step(0.25);
            if w.battles.is_empty() && w.next_battle > 0 {
                break;
            }
        }
        if w.thrown_lying.is_empty() {
            continue;
        }
        let lying = w.thrown_lying.clone();
        for id in &lying {
            assert!(w.ground.iter().any(|g| g.id == *id && g.item == hatchet), "a hatchet on the ground");
        }
        let carried = |w: &World| w.squad.members.iter().map(|&m| held(w, m, hatchet)).sum::<u16>();
        let before = carried(&w);
        let fit: Vec<u32> = w.squad.members.iter().copied().filter(|&m| !w.is_down(m) && !w.people[m as usize].dead).collect();
        // (Room for them all: a fight can leave one standing, and a pack has
        // only so many cells; one with no room stops, `gather_next`.)
        for &m in &fit {
            let d = w.people[m as usize].detail.as_mut().unwrap();
            d.gear.add(items::id("large_pack"), 1);
            d.gear.equip(items::id("large_pack")).unwrap();
        }
        assert!(w.gather_thrown(&fit) >= lying.len());
        for _ in 0..4000 {
            w.step(0.25);
            if w.thrown_lying.is_empty() {
                break;
            }
        }
        assert!(w.thrown_lying.is_empty(), "all gathered");
        assert_eq!(carried(&w), before + lying.len() as u16, "back in the packs");
        return;
    }
    panic!("no fight left a hatchet lying");
}

#[test]
fn a_fight_while_gathering_ends_it_and_it_can_be_ordered_again() {
    // Heavy packs on before the fight, so nobody can outpace anyone (the
    // squad mustn't be led off: the spearmen stand, `can_step_back`). On
    // seed 2 the bandits come back while the squad gathers.
    let hatchet = items::id("hatchet");
    let mut w = worldgen::generate(2);
    let start = w.squad.pos;
    for &m in &w.squad.members.clone() {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        d.gear.add(hatchet, 3);
        d.gear.add(items::id("large_pack"), 1);
        d.gear.equip(items::id("large_pack")).unwrap();
    }
    w.spawn_bandits(start.add(V2::new(16.0, 4.0)), 3, false);
    w.step(0.1);
    let mut far = 0.0f32;
    for _ in 0..40_000 {
        w.step(0.25);
        for &m in &w.squad.members {
            far = far.max(w.person_pos(m).dist(start));
        }
        if w.battles.is_empty() && w.next_battle > 0 {
            break;
        }
    }
    assert!(far < 60.0, "the fight stays where it was ({far:.0} m)");
    assert!(!w.thrown_lying.is_empty());
    let fit = |w: &World| w.squad.members.iter().copied().filter(|&m| !w.is_down(m) && !w.people[m as usize].dead).collect::<Vec<u32>>();
    w.gather_thrown(&fit(&w));
    let (mut fought, mut orders) = (false, 1);
    for _ in 0..20_000 {
        w.step(0.25);
        if !w.battles.is_empty() {
            fought = true;
            continue;
        }
        if w.thrown_lying.is_empty() {
            break;
        }
        // Over: nobody stands waiting on an order they aren't walking to.
        for p in &w.pickups {
            let k = w.squad.index(p.who).unwrap();
            let g = w.ground.iter().find(|g| g.id == p.thing).unwrap();
            assert!(w.squad.goal[k].dist(g.pos) < 3.0 || w.squad.at[k].dist(g.pos) < 3.0, "a pickup left hanging");
        }
        if w.gatherers.is_empty() && w.pickups.is_empty() {
            w.gather_thrown(&fit(&w));
            orders += 1;
            assert!(orders < 10);
        }
    }
    assert!(w.thrown_lying.is_empty(), "all gathered in the end");
    let _ = fought;
}

// ---- Rule 9: concealment ---------------------------------------------------------

#[test]
fn hidden_things_are_hard_to_notice_and_never_certain() {
    use gahturiyu_sim::sim::conceal::{notice_chance, Look};
    for h in [0.0, 0.3, 0.6, 0.9, 1.0] {
        for sharp in [0.0, 0.5, 1.0] {
            let (g, s, f) = (notice_chance(h, sharp, Look::Glance), notice_chance(h, sharp, Look::SizeUp), notice_chance(h, sharp, Look::Search));
            assert!(g <= s && s <= f, "looking harder notices more");
            assert!(g > 0.0 && f < 1.0, "never certain either way");
        }
    }
    assert!(notice_chance(0.9, 0.5, Look::Glance) < 0.2, "a well-hidden knife slips past a glance");
    assert!(notice_chance(0.9, 0.5, Look::Search) > 0.5, "a search mostly finds it");
}

#[test]
fn a_hidden_knife_isnt_counted_until_its_noticed() {
    use gahturiyu_sim::sim::conceal::{self, Look};
    use gahturiyu_sim::sim::items::Slot;
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let looker = w.settlements[0].residents[0];
    // Only a knife: what they look like with it seen, and unseen.
    {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        d.gear.unequip(Slot::MainHand);
        d.gear.unequip(Slot::OffHand);
        d.gear.add(items::id("knife"), 1);
        d.gear.equip(items::id("knife")).unwrap();
    }
    w.people[m as usize].recompute_might();
    let full = w.people[m as usize].might;
    let gear = w.people[m as usize].kit();
    assert!(!conceal::shows(&gear, Slot::MainHand), "a knife doesn't show on the figure");
    let (mut lower, mut same) = (0, 0);
    for h in 0..200 {
        let t = w.time + h as f64 * 3600.0;
        let seen = w.seen_might(looker, m, Look::Glance, t);
        assert!(seen <= full + 1e-3);
        if seen < full - 1e-3 {
            lower += 1;
        } else {
            same += 1;
        }
        // The same look in the same hour comes out the same.
        assert_eq!(seen, w.seen_might(looker, m, Look::Glance, t));
    }
    assert!(lower > same, "a glance mostly misses it ({lower} missed, {same} seen)");
    // A longsword in the hand is plain to see.
    let d = w.people[m as usize].detail.as_mut().unwrap();
    d.gear.add(items::id("longsword"), 1);
    d.gear.unequip(Slot::MainHand);
    d.gear.equip(items::id("longsword")).unwrap();
    assert!(conceal::shows(&w.people[m as usize].kit(), Slot::MainHand));
}

// ---- Heft (Part 1, B1): weight, strength and grade ------------------------------

#[test]
fn lighter_material_swings_quicker_and_heavier_wants_more_strength() {
    use gahturiyu_sim::sim::materials::{Grade, Material};
    let def = |key: &str, m: Material, g: Grade| *items::item(items::variant(items::id(key), m, Material::None, g).expect("a variant")).weapon().unwrap();
    let iron = def("longsword", Material::Forgeiron, Grade::Common);
    let glass = def("longsword", Material::Edgeglass, Grade::Common);
    assert!(glass.windup < iron.windup * 0.9, "Edgeglass swings quicker ({} vs {})", glass.windup, iron.windup);
    assert!(glass.strength < iron.strength && glass.kg < iron.kg);
    let master = def("longsword", Material::Forgeiron, Grade::Masterwork);
    let crude = def("longsword", Material::Forgeiron, Grade::Crude);
    assert!(master.balance > 0.0 && crude.balance < 0.0, "grade balance");
    // In the hand: an average fighter wields a Common longsword as it is;
    // a weak one is slower and less sure, up to the caps.
    let b = pair(1);
    let mut f = b.fighters[0].clone();
    f.stats.set_attr(gahturiyu_sim::sim::stats::Attr::Strength, 35.0);
    assert_eq!(f.wield(&iron), (1.0, 1.0, 1.0));
    let maul = weapon("stone_maul");
    f.stats.set_attr(gahturiyu_sim::sim::stats::Attr::Strength, 25.0);
    let (time, hit, tire) = f.wield(&maul);
    assert!((time - 1.4).abs() < 1e-4 && (hit - 0.8).abs() < 1e-4 && tire > 1.0, "40 short: +40% time, -20% to hit ({time}, {hit})");
    f.stats.set_attr(gahturiyu_sim::sim::stats::Attr::Strength, 1.0);
    let (time, hit, _) = f.wield(&maul);
    assert!((time - 1.5).abs() < 1e-4 && (hit - 0.75).abs() < 1e-4, "capped");
    // Finely made, quicker and surer.
    f.stats.set_attr(gahturiyu_sim::sim::stats::Attr::Strength, 60.0);
    let (time, hit, _) = f.wield(&master);
    assert!(time < 1.0 && hit > 1.0);
}

#[test]
fn heavier_weapons_tire_more_per_blow() {
    let tired = |key: &str| {
        let mut b = pair(3);
        b.fighters[0].stats.set_attr(gahturiyu_sim::sim::stats::Attr::Strength, 90.0);
        let before = b.fighters[0].fatigue;
        one_blow(&mut b, weapon(key));
        before - b.fighters[0].fatigue
    };
    assert!(tired("stone_maul") > tired("longsword") && tired("longsword") > tired("knife"));
}

// ---- Armour on the body (0.3) ---------------------------------------------------

/// B wearing these.
fn wearing(b: &mut Battle, keys: &[&str]) {
    let p = {
        let mut p = person(2, &[(Skill::Dodge, 5.0)]);
        let d = p.detail.as_mut().unwrap();
        d.gear = gahturiyu_sim::sim::inventory::Gear::default();
        for k in keys {
            d.gear.add(items::id(k), 1);
            d.gear.equip(items::id(k)).unwrap();
        }
        p
    };
    let mut f = Fighter::from_person(&p, 1, b.fighters[1].pos, 0.0);
    f.think_at = f64::INFINITY;
    b.fighters[1] = f;
}

#[test]
fn the_outer_layer_meets_the_blow_first_and_glass_slips_past_light_armour() {
    use gahturiyu_sim::sim::materials::{Grade, Material};
    let glass = items::variant(items::id("longsword"), Material::Edgeglass, Material::None, Grade::Common).expect("an Edgeglass longsword");
    let iron = items::variant(items::id("longsword"), Material::Forgeiron, Material::None, Grade::Common).unwrap();
    // Against a light hide coat, Edgeglass gets more through than its edge
    // alone would; against plate it doesn't.
    let through = |it: items::ItemId, keys: &[&str]| {
        let mut total = 0.0;
        for seed in 0..15 {
            let mut b = pair(1000 + seed);
            wearing(&mut b, keys);
            b.fighters[0].weapon_item = Some(it);
            let w = *items::item(it).weapon().unwrap();
            for _ in 0..20 {
                one_blow(&mut b, w);
                total += b.fighters[1].damage_taken;
                b.fighters[1].damage_taken = 0.0;
            }
        }
        total
    };
    let (g_bare, i_bare) = (through(glass, &[]), through(iron, &[]));
    let (g_hide, i_hide) = (through(glass, &["hide_coat"]), through(iron, &["hide_coat"]));
    assert!(g_hide / g_bare > i_hide / i_bare, "glass keeps more of its bite through a hide coat");
}

