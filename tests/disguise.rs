//! Disguise, F3 (Laz, 12 Oct: New Vegas's standing with Kenshi's fit check).
//! A role garment puts a squad member in its group; the real ones who look
//! hard see through it by a keyed roll (how close they are, the kit, the
//! colours, Sneak, past doubts); what follows reads the squad's standing
//! with the group's faction; impersonating the watch is always a crime, a
//! priest only where the community forbids it.

use gahturiyu_sim::sim::{
    culture::Garb,
    disguise::{civic, Crime, CIVIC, MISFIT, WRONG_COLOURS},
    factions::{self, id},
    items::{self, Slot},
    jobs::Job,
    law::Wrong,
    materials::{Mark, Piece},
    person::PersonId,
    world::{DAY, HOUR},
    worldgen, World,
};

#[test]
fn the_groups_hold_together() {
    for c in CIVIC {
        if let Some(f) = c.faction {
            let _ = id(f);
        }
        if c.role.is_some() {
            assert!(!c.knows.is_empty(), "{}: someone knows the real ones", c.name);
        }
        if c.elite {
            assert_eq!(c.crime, Crime::Always, "{}: an elite uniform is always a crime", c.name);
        }
    }
    assert_eq!(CIVIC[civic("watch")].crime, Crime::Always);
    assert_eq!(CIVIC[civic("shrine_keepers")].crime, Crime::ByCustom);
}

/// Member 0 standing in the middle of the nearest town at midday, wearing
/// `garment` (made in `made_in`, if given) and nothing in their hands.
fn dressed(seed: u64, garment: &str, made_in: Option<u16>) -> (World, PersonId, u16) {
    let mut w = worldgen::generate(seed);
    while (w.time.rem_euclid(DAY) / HOUR - 12.0).abs() > 0.3 {
        w.step(HOUR / 4.0);
    }
    let here = w.squad.pos;
    let town = (0..w.settlements.len()).min_by(|&a, &b| w.settlements[a].pos.dist(here).total_cmp(&w.settlements[b].pos.dist(here))).unwrap() as u16;
    let at = w.settlements[town as usize].pos;
    w.teleport_squad(at);
    let m = w.squad.members[0];
    let id_ = items::id(garment);
    let d = w.people[m as usize].detail.as_mut().unwrap();
    for s in [Slot::MainHand, Slot::OffHand, Slot::Cloak] {
        d.gear.discard(s);
    }
    match made_in {
        Some(t) => {
            let mut p = Piece::new(items::max_durability(id_), w.time);
            p.mark = Some(Mark { maker: 0, town: Some(t), stamped: false });
            d.gear.add_piece(id_, p);
        }
        None => d.gear.add(id_, 1),
    }
    assert!(w.equip(m, id_), "put on the {garment}");
    (w, m, town)
}

fn a_guard(w: &World, town: u16) -> PersonId {
    w.settlements[town as usize].residents.iter().copied().find(|&p| w.life(p).job == Job::Guard).expect("a guard")
}

#[test]
fn a_tabard_makes_one_of_the_watch_and_the_fit_check_reads_the_kit() {
    let (w, m, town) = dressed(1, "guard_tabard", None);
    let watch = civic("watch");
    assert_eq!(w.disguise_of(m), Some(watch));
    let g = a_guard(&w, town);
    // Unarmed and in no town's colours: both count against them.
    assert_eq!(w.misfits(m, watch), 1, "one of the watch goes armed");
    let plain = w.seen_through_chance(g, m, watch, town);
    let (w2, m2, _) = dressed(1, "guard_tabard", Some(town));
    let fits = w2.seen_through_chance(g, m2, watch, town);
    assert!((plain - fits - WRONG_COLOURS).abs() < 1e-4, "this town's tabard: {plain} vs {fits}");
    // A townsperson who isn't of the watch looks less hard.
    let other = w.settlements[town as usize].residents.iter().copied().find(|&p| w.life(p).job == Job::Farmer).unwrap();
    assert!(w.seen_through_chance(other, m, watch, town) < plain);
    // The same roll every time it's asked.
    assert_eq!(w.seen_through(g, m, watch, town, w.time), w.seen_through(g, m, watch, town, w.time));
}

#[test]
fn a_priest_with_a_sword_stands_out() {
    let (mut w, m, town) = dressed(1, "vestment", None);
    let keepers = civic("shrine_keepers");
    assert_eq!(w.disguise_of(m), Some(keepers));
    let priest = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).find(|&p| w.life(p).job == Job::Priest).expect("a priest");
    let bare = w.seen_through_chance(priest, m, keepers, town);
    let sword = items::id("knife");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(sword, 1);
    assert!(w.equip(m, sword));
    assert!((w.seen_through_chance(priest, m, keepers, town) - bare - MISFIT).abs() < 1e-4);
    // Impersonating a priest is a crime only where it's forbidden.
    let forbids = w.community_of(priest).unwrap().customs.garb == Garb::Forbidden;
    assert_eq!(w.impersonation_is_crime(keepers, priest), forbids);
    assert!(w.impersonation_is_crime(civic("watch"), priest), "the watch, always");
}

/// Run the world on until the watch has looked hard at member `m` (doubted
/// or come after them), at most `hours`.
fn until_seen(w: &mut World, m: PersonId, hours: f64) -> bool {
    let end = w.time + hours * HOUR;
    let k = w.squad.index(m).unwrap();
    while w.time < end {
        // Beside a guard who's up (the watch must be there to look).
        let here = w.squad.at[k];
        let guard = w.people.iter().map(|p| p.id).filter(|&p| w.life(p).job == Job::Guard && !w.people[p as usize].dead && w.doing_now(p).is_some_and(|d| d.0 != gahturiyu_sim::sim::routine::Doing::Asleep)).min_by(|&a, &b| w.person_pos(a).dist(here).total_cmp(&w.person_pos(b).dist(here)));
        if let Some(g) = guard {
            let at = w.person_pos(g).add(gahturiyu_sim::sim::geo::V2::new(1.5, 0.0));
            if at.dist(here) > 3.0 {
                w.squad.at[k] = at;
                w.squad.goal[k] = at;
                w.squad.route[k].clear();
            }
        }
        w.step(10.0);
        if w.doubted_times(m, civic("watch")) > 0 || w.pursuits.iter().any(|p| p.culprit == m) {
            return true;
        }
    }
    false
}

#[test]
fn hostile_holders_come_after_a_false_watchman_and_friendly_ones_let_it_pass() {
    // Hostile: they come for them, for impersonation.
    let (mut w, m, town) = dressed(1, "guard_tabard", None);
    w.settlements[town as usize].holder = Some(id("banner_lords"));
    w.deed(id("banner_lords"), -90.0);
    assert!(w.standing_with(id("banner_lords")) <= factions::HOSTILE);
    assert!(until_seen(&mut w, m, 4.0), "the watch looked");
    // (The guard beside them catches them at once: the line says so.)
    assert!(w.log.iter().any(|l| l.1.contains("comes for them")), "came after them: {:?}", w.log);
    let _ = Wrong::Impersonation;
    // Friendly: a long look, and nothing more.
    let (mut w, m, town) = dressed(1, "guard_tabard", None);
    w.settlements[town as usize].holder = Some(id("banner_lords"));
    w.deed(id("banner_lords"), 60.0);
    assert!(until_seen(&mut w, m, 4.0), "the watch looked");
    assert!(w.pursuits.iter().all(|p| p.culprit != m), "let it pass");
    assert!(w.doubted_times(m, civic("watch")) > 0, "but they'll look harder next time");
}

#[test]
fn every_group_has_a_garment_that_passes_for_it() {
    for (g, c) in CIVIC.iter().enumerate() {
        let role = c.role.unwrap_or_else(|| panic!("{} has a garment", c.name));
        let garment = items::ITEMS.iter().position(|i| items::look(i_id(i.key)).role == Some(role)).unwrap_or_else(|| panic!("{}: something to wear", c.name));
        let _ = (g, garment);
    }
}

fn i_id(key: &str) -> items::ItemId {
    items::id(key)
}
