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
    b.fighters[1].armor = vec![ArmorDef { covers: &gahturiyu_sim::sim::body::PARTS, coverage: 1.0, cut: 0.7, blunt: 0.3, dodge_penalty: 0.0 }];
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

/// A world just after a fight in which a squad member was left bleeding
/// (they carry no bandages yet).
fn bleeding_after_a_fight() -> Option<(World, u32)> {
    for seed in 1..12u64 {
        let mut w = worldgen::generate(seed);
        let at = w.squad.pos.add(V2::new(18.0, 4.0));
        w.spawn_bandits(at, 3, false);
        w.step(0.1);
        for _ in 0..40_000 {
            w.step(0.25);
            if w.battles.is_empty() && w.next_battle > 0 {
                break;
            }
        }
        if let Some(&m) = w.squad.members.iter().find(|&&m| w.bleeding(m) > 0.0 && !w.people[m as usize].dead) {
            return Some((w, m));
        }
    }
    None
}

#[test]
fn bleeding_out_of_a_fight_is_solved_on_the_clock() {
    let (w, m) = bleeding_after_a_fight().expect("someone in the squad left bleeding");
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
    let (mut w, m) = bleeding_after_a_fight().expect("someone in the squad left bleeding");
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
    let sap = WeaponDef { skill: Skill::Blunt, cut: 0.0, blunt: 4.0, reach: 0.8, windup: 0.35, recover: 0.4, two_handed: false, parry: 0.0, range: 0.0, ammo: None, pierce: 0.0, entangle: 0.0 };
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
