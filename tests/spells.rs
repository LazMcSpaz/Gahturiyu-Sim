//! The spell list, domain by domain: each spell does what it says.

use gahturiyu_sim::sim::{
    body::Part,
    combat::{Act, Battle, Fighter},
    effects::Does,
    geo::V2,
    items,
    magic::{spell, Spell},
    person::Person,
    race::Race,
    stats::{Attr, Calling, Skill},
    worldgen, World,
};

fn person(id: u32, race: Race, calling: Calling, skills: &[(Skill, f32)], budget: f32) -> Person {
    let mut p = Person::summary(id, 7000 + id as u64 * 11, race, None);
    p.specialize(calling, skills, budget);
    p.ensure_detail();
    p
}

/// A mage who knows every spell and rarely fails.
fn mage() -> Person {
    let mut p = person(1, Race::Tadoro, Calling::Mage, &[(Skill::Structured, 90.0), (Skill::Felt, 90.0), (Skill::Ritual, 90.0)], 60.0);
    p.stats.set_attr(Attr::Willpower, 90.0);
    p.detail.as_mut().unwrap().spells = gahturiyu_sim::sim::magic::all_spells().collect();
    p
}

fn brute(id: u32) -> Person {
    let mut p = person(id, Race::Qotiro, Calling::Warrior, &[(Skill::Blade, 50.0)], 400.0);
    p.stats.set_attr(Attr::Willpower, 1.0);
    p
}

/// Mage (0) and others; nobody thinks for themselves.
fn fight(others: &[(Person, u8, V2)]) -> Battle {
    let mut fs = vec![Fighter::from_person(&mage(), 0, V2::new(0.0, 0.0), 0.0)];
    let mut names = vec!["Mage".to_string()];
    for (k, (p, side, at)) in others.iter().enumerate() {
        fs.push(Fighter::from_person(p, *side, *at, 0.0));
        names.push(format!("F{}", k + 1));
    }
    for f in fs.iter_mut() {
        f.think_at = f64::INFINITY;
        f.mana = 1000.0;
        f.max_mana = 1000.0;
    }
    Battle::new(0, 31, 0.0, fs, names)
}

/// Cast (or release, for a ritual) until `check` holds. Panics if it never does.
fn cast_until(b: &mut Battle, s: Spell, target: Option<usize>, point: V2, check: impl Fn(&Battle) -> bool) {
    for _ in 0..30 {
        b.fighters[0].mana = 1000.0;
        if s.def().style == gahturiyu_sim::sim::magic::Style::Ritual {
            b.fighters[0].held = Some(s);
            assert!(b.release(0, target, point));
        } else {
            assert!(b.begin_cast(0, s, target, point), "{} wouldn't cast", s.def().name);
        }
        b.tick();
        while matches!(b.fighters[0].act, Act::Cast { .. }) {
            b.tick();
        }
        if check(b) {
            return;
        }
        while !matches!(b.fighters[0].act, Act::Idle) {
            b.tick();
        }
    }
    panic!("{} never took effect", s.def().name);
}

fn hp(b: &Battle, i: usize) -> f32 {
    b.fighters[i].hp.iter().sum()
}

// ---- Elemental ----------------------------------------------------------------

#[test]
fn spark_and_chill_sting_and_chill_slows() {
    let mut b = fight(&[(brute(2), 1, V2::new(8.0, 0.0))]);
    let full = hp(&b, 1);
    cast_until(&mut b, spell("spark"), Some(1), V2::new(8.0, 0.0), |b| hp(b, 1) < full);
    assert!(full - hp(&b, 1) < 15.0, "a spark is small");
    let speed = b.fighters[1].speed();
    cast_until(&mut b, spell("chill"), Some(1), V2::new(8.0, 0.0), |b| b.fighters[1].has(Does::Slow).is_some());
    assert!(b.fighters[1].speed() < speed * 0.8);
}

#[test]
fn stone_spikes_hit_only_the_legs() {
    let mut b = fight(&[(brute(2), 1, V2::new(8.0, 0.0))]);
    let before = b.fighters[1].hp;
    cast_until(&mut b, spell("stone_spikes"), None, V2::new(8.0, 0.0), |b| hp(b, 1) < before.iter().sum::<f32>());
    let f = &b.fighters[1];
    for p in [Part::Head, Part::Torso, Part::LeftArm, Part::RightArm] {
        assert_eq!(f.hp[p as usize], before[p as usize], "{p:?} shouldn't be touched");
    }
    assert!(f.hp[Part::LeftLeg as usize] < before[Part::LeftLeg as usize]);
}

#[test]
fn a_released_firestorm_burns_everyone_in_it() {
    let mut b = fight(&[(brute(2), 1, V2::new(10.0, 0.0)), (brute(3), 1, V2::new(12.0, 2.0)), (brute(4), 0, V2::new(13.0, -1.0)), (brute(5), 1, V2::new(30.0, 0.0))]);
    let before: Vec<f32> = (0..5).map(|i| hp(&b, i)).collect();
    cast_until(&mut b, spell("firestorm"), None, V2::new(11.0, 0.0), |b| hp(b, 1) < before[1]);
    assert!(hp(&b, 2) < before[2] && hp(&b, 3) < before[3], "friend and foe alike");
    assert_eq!(hp(&b, 4), before[4], "but not anyone outside it");
    assert!(before[1] - hp(&b, 1) > 15.0, "a firestorm is big");
}

#[test]
fn tremor_throws_enemies_down_and_spares_friends() {
    let mut b = fight(&[(brute(2), 1, V2::new(4.0, 0.0)), (brute(3), 0, V2::new(-3.0, 0.0))]);
    cast_until(&mut b, spell("tremor"), None, V2::new(0.0, 0.0), |b| b.fighters[1].helpless());
    assert!(!b.fighters[2].helpless(), "friends keep their feet");
    assert_eq!(b.fighters[1].speed(), 0.0);
}

#[test]
fn rituals_cant_be_cast_only_released() {
    let mut b = fight(&[(brute(2), 1, V2::new(8.0, 0.0))]);
    assert!(!b.begin_cast(0, spell("firestorm"), None, V2::new(8.0, 0.0)));
    assert!(!b.release(0, None, V2::new(8.0, 0.0)), "nothing held");
}

/// A world with a bandit camp, the squad 20 m from it.
fn by_a_camp() -> World {
    let mut w = worldgen::generate(7);
    let camp = w.camps[0].pos;
    w.teleport_squad(camp.add(V2::new(9.0, 0.0)));
    w
}

fn squad_mage(w: &World) -> u32 {
    *w.squad.members.iter().find(|&&m| w.people[m as usize].stats.calling == Calling::Mage).unwrap()
}

fn knows_all(w: &mut World, m: u32) {
    let p = &mut w.people[m as usize];
    p.detail.as_mut().unwrap().spells = gahturiyu_sim::sim::magic::all_spells().collect();
    for k in [Skill::Felt, Skill::Structured, Skill::Ritual] {
        p.stats.set_skill(k, 90.0);
    }
    p.stats.set_attr(Attr::Willpower, 90.0);
    p.mana = 1000.0;
}

/// Cast out of a fight until it works (a felt spell almost always does).
fn cast_world(w: &mut World, who: u32, s: Spell, target: Option<u32>, point: Option<V2>, check: impl Fn(&World) -> bool) {
    for _ in 0..20 {
        w.people[who as usize].mana = 1000.0;
        w.people[who as usize].mana_at = w.time;
        w.cast(who, s, target, point).unwrap_or_else(|e| panic!("{}: {}", s.def().name, e.0));
        if check(w) {
            return;
        }
    }
    panic!("{} never took effect", s.def().name);
}

#[test]
fn douse_puts_out_a_campfire_and_kindle_lights_it_again() {
    let mut w = by_a_camp();
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let camp = w.camps[0].pos;
    let lit = |w: &World| w.fixed_lights().iter().any(|l| l.pos.dist(camp) < 0.1);
    assert!(lit(&w));
    cast_world(&mut w, m, spell("douse"), None, Some(camp), |w| !lit(w));
    // It stays out for hours by the clock, not by anyone watching.
    w.teleport_squad(camp.add(V2::new(2000.0, 0.0)));
    w.step(3600.0);
    assert!(!lit(&w));
    w.teleport_squad(camp.add(V2::new(9.0, 0.0)));
    cast_world(&mut w, m, spell("kindle"), None, Some(camp), lit);
    // And a squad member's torch.
    let other = w.squad.members[0];
    let at = w.person_pos(other);
    cast_world(&mut w, m, spell("kindle"), None, Some(at), |w| w.torch_lit(other));
    cast_world(&mut w, m, spell("douse"), None, Some(at), |w| !w.torch_lit(other));
    let _ = items::id("torch");
}

// ---- Psychic ------------------------------------------------------------------

#[test]
fn daze_spoils_the_next_action() {
    let mut b = fight(&[(brute(2), 1, V2::new(6.0, 0.0))]);
    b.fighters[1].act = Act::Swing { target: 0, lands: 99.0 };
    cast_until(&mut b, spell("daze"), Some(1), V2::new(6.0, 0.0), |b| matches!(b.fighters[1].act, Act::Recover { .. }));
    assert!(b.fighters[1].think_at > b.time, "they lose a moment");
}

/// Let fighter `i` think for themselves from now.
fn wake_up(b: &mut Battle, i: usize) {
    b.fighters[i].think_at = 0.0;
}

#[test]
fn calm_holds_them_off_until_a_blow_lands() {
    let mut b = fight(&[(brute(2), 1, V2::new(6.0, 0.0))]);
    cast_until(&mut b, spell("calm"), Some(1), V2::new(6.0, 0.0), |b| b.fighters[1].has(Does::Calm).is_some());
    wake_up(&mut b, 1);
    let at = b.fighters[1].pos;
    for _ in 0..30 {
        b.tick();
    }
    assert!(b.fighters[1].pos.dist(at) < 0.5 && b.fighters[1].target.is_none(), "calm: they don't come on");
    b.wound(1, Part::Torso, 3.0);
    assert!(b.fighters[1].has(Does::Calm).is_none(), "a blow breaks it");
}

#[test]
fn fear_drives_them_off() {
    let mut b = fight(&[(brute(2), 1, V2::new(6.0, 0.0))]);
    cast_until(&mut b, spell("fear"), Some(1), V2::new(6.0, 0.0), |b| b.fighters[1].has(Does::Fear).is_some());
    wake_up(&mut b, 1);
    for _ in 0..40 {
        b.tick();
    }
    assert!(b.fighters[1].pos.dist(b.fighters[0].pos) > 12.0, "they ran");
    assert!(!b.fighters[1].fled, "but they'll be back");
}

#[test]
fn a_dominated_enemy_fights_for_you_and_then_turns_back() {
    let mut b = fight(&[(brute(2), 1, V2::new(6.0, 0.0)), (brute(3), 1, V2::new(8.0, 1.0))]);
    cast_until(&mut b, spell("dominate"), Some(1), V2::new(6.0, 0.0), |b| b.fighters[1].side == 0);
    assert!(b.hostile(1, 2), "now against their friend");
    assert!(!b.hostile(0, 1));
    // The fight isn't over while the turned one's own side still stands.
    b.tick();
    assert!(!b.over);
    while b.fighters[1].has(Does::Dominate).is_some() {
        b.tick();
    }
    b.tick();
    assert_eq!(b.fighters[1].side, 1, "back to their own side");
}

#[test]
fn spells_cast_on_the_road_last_and_come_into_a_fight() {
    let mut w = worldgen::generate(2);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    cast_world(&mut w, m, spell("nightsight"), None, None, |w| w.boon(m, Does::Nightsight) > 0.0);
    cast_world(&mut w, m, spell("sense_life"), None, None, |w| w.boon(m, Does::SenseLife) > 0.0);
    // They run out by the clock.
    w.step(200.0);
    assert_eq!(w.boon(m, Does::SenseLife), 0.0, "three minutes");
    assert!(w.boon(m, Does::Nightsight) > 0.0, "fifteen minutes");
    // A fight: the spell is on them there too, and still on them after.
    let at = w.squad.pos.add(V2::new(15.0, 0.0));
    w.spawn_bandits(at, 1, false);
    while w.battles.is_empty() {
        w.step(0.25);
    }
    assert!(w.battles[0].fighters.iter().find(|f| f.pid == m).unwrap().has(Does::Nightsight).is_some());
    while !w.battles.is_empty() {
        w.step(0.5);
    }
    assert!(w.boon(m, Does::Nightsight) > 0.0);
}

#[test]
fn sway_warms_people_to_you() {
    use gahturiyu_sim::sim::dialogue::Topic;
    let mut w = worldgen::generate(1);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let npc = (0..w.people.len() as u32).find(|&p| !w.people[p as usize].in_squad && !w.people[p as usize].bandit && w.people[p as usize].home.is_some()).unwrap();
    let before = w.disposition(npc, m);
    cast_world(&mut w, m, spell("sway"), None, None, |w| w.boon(m, Does::Sway) > 0.0);
    assert!(w.disposition(npc, m) >= (before + 19.0).min(100.0));
    let _ = Topic::Goodbye;
}

// ---- Illusion -----------------------------------------------------------------

#[test]
fn silent_step_and_hide_and_glow_and_gloom() {
    let mut w = worldgen::generate(3);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let noise = w.noise_of(m);
    let seen = w.visibility_of(m);
    cast_world(&mut w, m, spell("silent_step"), None, None, |w| w.boon(m, Does::Silent) > 0.0);
    assert!(w.noise_of(m) < noise * 0.5);
    cast_world(&mut w, m, spell("hide"), None, None, |w| w.boon(m, Does::Hide) > 0.0);
    assert!(w.visibility_of(m) < seen * 0.5);
    // At night: glow lights the ground round them; gloom darkens it.
    while gahturiyu_sim::sim::stealth::daylight(w.time) > 0.2 {
        w.step(600.0);
    }
    let at = w.person_pos(m).add(V2::new(30.0, 30.0));
    w.teleport_squad(at);
    let dark = w.light_at(w.person_pos(m));
    cast_world(&mut w, m, spell("glow"), None, None, |w| w.boon(m, Does::Glow) > 0.0);
    assert!(w.light_at(w.person_pos(m)) > dark + 0.4);
    w.boons.retain(|b| b.does != Does::Glow);
    cast_world(&mut w, m, spell("gloom"), None, None, |w| w.boon(m, Does::Gloom) > 0.0);
    assert!(w.light_at(w.person_pos(m)) < dark + 0.01);
}

#[test]
fn a_decoy_draws_the_blows_then_fades() {
    let mut b = fight(&[(brute(2), 1, V2::new(6.0, 0.0))]);
    cast_until(&mut b, spell("decoy"), None, V2::new(3.0, 0.0), |b| b.fighters.iter().any(|f| f.is_decoy()));
    let d = b.fighters.iter().position(|f| f.is_decoy()).unwrap();
    assert_eq!(b.fighters[d].side, 0);
    b.fighters[1].think_at = 0.0;
    b.tick();
    assert_eq!(b.fighters[1].target, Some(d), "the enemy goes for the decoy");
    for _ in 0..250 {
        b.tick();
    }
    assert!(b.fighters[d].fled || b.fighters[d].ko, "gone after its time");
}

#[test]
fn in_disguise_your_crimes_and_bounty_go_unrecognised() {
    let mut w = worldgen::generate(1);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let home = w.people.iter().find(|p| !p.in_squad && !p.bandit && p.home.is_some()).unwrap();
    let (npc, town) = (home.id, home.home.unwrap());
    w.bounty.insert(town, 80.0);
    let cold = w.disposition(npc, m);
    cast_world(&mut w, m, spell("disguise"), None, None, |w| w.boon(m, Does::Disguise) > 0.0);
    assert!(w.disposition(npc, m) > cold + 15.0, "they don't know who you are");
}

#[test]
fn a_veil_hides_the_squad_from_lookouts() {
    let mut w = worldgen::generate(7);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let camp = w.camps[0].pos;
    w.teleport_squad(camp.add(V2::new(25.0, 0.0)));
    let g = w.camps[0].group;
    let (rate, _) = w.detect_rate(g, camp, m);
    assert!(rate > 0.0);
    w.held.insert(m, spell("veil"));
    w.release(m, None, None).unwrap();
    let (veiled, _) = w.detect_rate(g, camp, m);
    assert_eq!(veiled, 0.0, "inside the veil, unseen and unheard");
    // Hours later, by the clock, it lifts.
    w.teleport_squad(camp.add(V2::new(3000.0, 0.0)));
    w.step(7.0 * 3600.0);
    assert!(w.wards.is_empty());
}

// ---- Vital ---------------------------------------------------------------------

#[test]
fn second_wind_gives_back_stamina_in_a_fight_and_out() {
    let mut b = fight(&[(brute(2), 1, V2::new(8.0, 0.0))]);
    b.fighters[0].fatigue = 5.0;
    cast_until(&mut b, spell("second_wind"), Some(0), V2::new(0.0, 0.0), |b| b.fighters[0].fatigue > 40.0);
    let mut w = worldgen::generate(1);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let t = w.time;
    w.people[m as usize].cond.as_mut().unwrap().stamina = 10.0;
    w.people[m as usize].cond.as_mut().unwrap().at = t;
    cast_world(&mut w, m, spell("second_wind"), Some(m), None, |w| w.stamina_of(m).unwrap() > 0.4);
}

#[test]
fn might_makes_them_stronger_and_able_to_carry_more() {
    let mut w = worldgen::generate(1);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let other = w.squad.members[0];
    // Overloaded to start with.
    w.people[other as usize].detail.as_mut().unwrap().gear.add(items::id("iron_ore"), 40);
    let (load, pace) = (w.load_of(other), w.member_speed(other));
    cast_world(&mut w, m, spell("might"), Some(other), None, |w| w.boon(other, Does::Carry) > 0.0);
    assert!(w.load_of(other) < load - 0.1);
    assert!(w.member_speed(other) > pace);
    // In a fight it's strength.
    let mut b = fight(&[(brute(2), 0, V2::new(2.0, 0.0))]);
    let s = b.fighters[1].attr(Attr::Strength);
    cast_until(&mut b, spell("might"), Some(1), V2::new(2.0, 0.0), |b| b.fighters[1].attr(Attr::Strength) > s + 10.0);
}

#[test]
fn toughened_skin_takes_the_edge_off_blows() {
    let hits = |tough: bool| {
        let mut b = fight(&[(brute(2), 1, V2::new(1.5, 0.0))]);
        if tough {
            cast_until(&mut b, spell("toughen"), None, V2::new(0.0, 0.0), |b| b.fighters[0].has(Does::Toughen).is_some());
        }
        b.fighters[1].think_at = 0.0;
        let before = hp(&b, 0);
        for _ in 0..300 {
            b.tick();
        }
        before - hp(&b, 0)
    };
    assert!(hits(true) < hits(false) * 0.9);
}

#[test]
fn sustain_holds_off_hunger_and_tiredness_for_a_day() {
    let mut w = worldgen::generate(1);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    w.held.insert(m, spell("sustain"));
    w.release(m, None, None).unwrap();
    let all = w.squad.members.clone();
    let before: Vec<(f32, f32)> = all.iter().map(|&k| (w.hunger_of(k).unwrap(), w.tired_of(k).unwrap())).collect();
    for _ in 0..(10 * 60) {
        w.step(60.0);
    }
    for (k, &p) in all.iter().enumerate() {
        assert!((w.hunger_of(p).unwrap() - before[k].0).abs() < 0.5, "no hunger");
        assert!(w.tired_of(p).unwrap() <= before[k].1 + 0.5, "no tiredness");
    }
    // A day later it's over, at that moment.
    for _ in 0..(16 * 60) {
        w.step(60.0);
    }
    assert!(w.hunger_of(all[0]).unwrap() > before[0].0 + 1.0, "hungry again");
}

#[test]
fn regrow_restores_a_lost_limb() {
    let mut w = worldgen::generate(1);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let other = w.squad.members[0];
    w.people[other as usize].wounds.missing[Part::LeftArm as usize] = true;
    w.teleport_squad(w.squad.pos);
    w.held.insert(m, spell("regrow"));
    w.release(m, Some(other), None).unwrap();
    assert!(!w.people[other as usize].wounds.missing[Part::LeftArm as usize]);
    // It's the only way: Mend can't do it.
    w.people[other as usize].wounds.missing[Part::RightLeg as usize] = true;
    cast_world(&mut w, m, spell("mend"), Some(other), None, |_| true);
    assert!(w.people[other as usize].wounds.missing[Part::RightLeg as usize]);
}

// ---- Warding --------------------------------------------------------------------

#[test]
fn brace_turns_one_blow() {
    let mut b = fight(&[(brute(2), 1, V2::new(1.5, 0.0))]);
    cast_until(&mut b, spell("brace"), None, V2::new(0.0, 0.0), |b| b.fighters[0].has(Does::Brace).is_some());
    b.fighters[1].think_at = 0.0;
    let full = hp(&b, 0);
    let mut turned = false;
    for _ in 0..200 {
        b.tick();
        if b.log.iter().any(|l| l.1.contains("brace turns")) {
            turned = true;
            break;
        }
    }
    assert!(turned);
    assert_eq!(hp(&b, 0), full, "that blow did nothing");
    assert!(b.fighters[0].has(Does::Brace).is_none(), "and the brace is spent");
}

#[test]
fn resist_halves_elemental_harm() {
    let burn = |resist: bool| {
        let mut b = fight(&[(brute(2), 0, V2::new(5.0, 0.0))]);
        if resist {
            cast_until(&mut b, spell("resist"), Some(1), V2::new(5.0, 0.0), |b| b.fighters[1].has(Does::ResistElements).is_some());
        }
        let before = hp(&b, 1);
        cast_until(&mut b, spell("lightning_bolt"), Some(1), V2::new(5.0, 0.0), |b| hp(b, 1) < before);
        before - hp(&b, 1)
    };
    let (bare, warded) = (burn(false), burn(true));
    assert!((warded / bare - 0.5).abs() < 0.15, "{warded} vs {bare}");
}

#[test]
fn dispel_unravels_spells_and_sends_creatures_back() {
    let mut b = fight(&[(brute(2), 1, V2::new(6.0, 0.0))]);
    b.fighters[1].statuses.push(gahturiyu_sim::sim::magic::Status { does: Does::Barrier, power: 0.5, until: 1e9 });
    cast_until(&mut b, spell("dispel"), Some(1), V2::new(6.0, 0.0), |b| b.fighters[1].statuses.is_empty());
    b.fighters[1].held = None;
    b.call_up(1, gahturiyu_sim::sim::effects::Summon::SpiritBeast, V2::new(8.0, 0.0), 1e9, 1.0);
    let beast = b.fighters.len() - 1;
    cast_until(&mut b, spell("dispel"), Some(beast), V2::new(8.0, 0.0), |b| b.fighters[beast].fled);
}

#[test]
fn enemies_cant_step_into_a_sanctuary() {
    let mut b = fight(&[(brute(2), 1, V2::new(20.0, 0.0))]);
    cast_until(&mut b, spell("sanctuary"), None, V2::new(0.0, 0.0), |b| !b.zones.is_empty());
    b.fighters[1].think_at = 0.0;
    for _ in 0..200 {
        b.tick();
    }
    assert!(b.fighters[1].pos.dist(V2::new(0.0, 0.0)) >= 11.9, "kept out: {:?}", b.fighters[1].pos);
    assert!(hp(&b, 0) >= b.fighters[0].max_hp.iter().sum::<f32>() - 0.01);
}

#[test]
fn a_tripwire_wakes_sleepers_and_a_sanctuary_keeps_lookouts_off() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    cast_world(&mut w, m, spell("tripwire"), None, None, |w| !w.wards.is_empty());
    let who = w.squad.members.clone();
    w.order_rest(&who);
    for _ in 0..200 {
        w.step(1.0);
    }
    assert!(who.iter().all(|&p| w.is_asleep(p)));
    w.spawn_bandits(w.squad.pos.add(V2::new(3.0, 0.0)), 2, false);
    w.step(0.25);
    w.step(0.25);
    let b = w.squad_battle().expect("attacked");
    assert!(b.fighters.iter().filter(|f| f.side == 0).all(|f| f.aware_at <= b.start), "everyone's up");

    // Sheltering in a sanctuary, lookouts that notice them hold off.
    let mut w = worldgen::generate(7);
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let camp = w.camps[0].pos;
    w.teleport_squad(camp.add(V2::new(15.0, 0.0)));
    w.held.insert(m, spell("sanctuary"));
    w.release(m, None, None).unwrap();
    for _ in 0..200 {
        w.step(0.25);
    }
    assert!(w.battles.iter().all(|b| b.fighters.iter().all(|f| f.side != 0)), "no fight with the squad");
}
