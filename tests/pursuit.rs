//! A crime seen must be told, and the thief caught (Laz, 2026-10-09): most
//! witnesses tell the watch, some don't; a guard runs the thief down, or
//! loses them and it becomes a bounty.

use gahturiyu_sim::sim::{
    containers::Owner,
    geo::V2,
    jobs::Job,
    law::Wrong,
    person::PersonId,
    worldgen,
    world::HOUR,
    World,
};

/// World 1 at mid-morning, with the squad stood in a town that has a guard on
/// watch, member 0 beside a townsperson out of doors (who isn't a guard).
fn in_town() -> (World, u16, PersonId, Vec<PersonId>) {
    let mut w = worldgen::generate(1);
    while w.time < 10.0 * HOUR {
        w.step(60.0);
    }
    let t = w.time;
    let town = (0..w.settlements.len() as u16)
        .find(|&s| {
            let guards = w.settlements[s as usize].residents.iter().filter(|&&p| !w.people[p as usize].dead && w.life(p).job == Job::Guard && w.at_work(p, t)).count();
            guards > 0 && w.settlements[s as usize].residents.len() > 40
        })
        .expect("a town with its watch out");
    w.teleport_squad(w.settlements[town as usize].pos);
    w.step(0.5);
    let me = w.squad.members[0];
    let folk: Vec<PersonId> = w.residents_in_band1(town).into_iter().filter(|&p| w.building_at(w.person_pos(p)).is_none() && w.life(p).job != Job::Guard).collect();
    assert!(!folk.is_empty());
    let k = w.squad.index(me).unwrap();
    let at = w.person_pos(folk[0]).add(V2::new(1.5, 0.0));
    w.squad.at[k] = at;
    w.squad.goal[k] = at;
    w.squad.route[k].clear();
    (w, town, me, folk)
}

/// Report a theft from the witness's own household until one is told (each
/// witness gets their own roll).
fn told(w: &mut World, town: u16, me: PersonId, folk: &[PersonId]) -> bool {
    for &f in folk {
        let Some(h) = w.life(f).household else { continue };
        let at = w.person_pos(me);
        w.wrong_seen(me, town, Wrong::Theft, 30.0, "Seen stealing!".into(), Some(f), Some(Owner::Household(h)), at);
        if !w.pursuits.is_empty() {
            return true;
        }
    }
    false
}

#[test]
fn most_tell_the_watch_but_not_everyone() {
    let (mut w, _, me, folk) = in_town();
    let mut odds: Vec<f32> = folk.iter().map(|&f| w.report_chance(f, me, None)).collect();
    odds.sort_by(|a, b| a.total_cmp(b));
    let mean = odds.iter().sum::<f32>() / odds.len() as f32;
    eprintln!("{} witnesses; odds {:.2}..{:.2}, mean {mean:.2}", odds.len(), odds[0], odds[odds.len() - 1]);
    assert!(mean > 0.6, "most would tell");
    // Their own household's things: they almost always tell.
    let f = folk[0];
    let h = w.life(f).household.unwrap();
    assert!(w.report_chance(f, me, Some(Owner::Household(h))) > 0.95);
    // A friend of the thief is less likely to.
    let before = w.report_chance(f, me, None);
    w.regard.insert(f, 60.0);
    assert!(w.report_chance(f, me, None) < before - 0.1, "a friend looks away");
    w.regard.remove(&f);
    // So is one of the town's thieves.
    if let Some(r) = w.society.rings.iter().position(|r| !r.members.is_empty()) {
        let thief = w.society.rings[r].members[0];
        let honest = w.report_chance(thief, me, None);
        w.society.rings[r].members.clear();
        assert!(w.report_chance(thief, me, None) > honest + 0.3, "thieves don't turn in thieves");
    }
}

#[test]
fn a_told_theft_sends_a_guard_who_catches_the_thief() {
    let (mut w, town, me, folk) = in_town();
    assert!(told(&mut w, town, me, &folk), "somebody tells");
    let p = w.pursuits[0].clone();
    assert!(w.alerts.iter().any(|a| a.contains("fetch the watch")), "{:?}", w.alerts);
    assert!(w.bounty.is_empty(), "not a bounty while they're after you");
    let guard_was = w.person_pos(p.guard);
    // Stand still and wait.
    let mut n = 0;
    while !w.pursuits.is_empty() && n < 20_000 {
        w.step(0.25);
        n += 1;
        if w.chased_by(me).is_some() {
            assert_eq!(w.person_pos(p.guard), w.chase_pos(p.guard).unwrap(), "the guard is where the chase has them");
        }
    }
    eprintln!("caught after {:.0} s; the guard came {:.0} m", n as f64 * 0.25, guard_was.dist(w.person_pos(me)));
    assert!(w.pursuits.is_empty());
    assert!(w.log.iter().any(|l| l.1.contains("catches")), "{:?}", w.log);
    assert!(w.bounty.is_empty(), "caught, so no bounty");
}

/// P21: the guard runs round buildings and stalls and in by doors, never
/// through them, the whole way to the thief, here hiding behind a house.
#[test]
fn the_watch_runs_round_what_is_solid() {
    use gahturiyu_sim::sim::buildings::door_of;
    let (mut w, town, me, folk) = in_town();
    assert!(told(&mut w, town, me, &folk), "somebody tells");
    let guard = w.pursuits[0].guard;
    while w.chase_pos(guard).is_none() && !w.pursuits.is_empty() {
        w.step(0.25);
    }
    let g = w.chase_pos(guard).expect("the guard sets off");
    // The thief behind the house nearest the guard, last seen there.
    let s = w.settlements[town as usize].clone();
    let d = (0..s.buildings.len() as u16).filter_map(|i| door_of(&s, i)).min_by(|a, b| a.centre.dist(g).total_cmp(&b.centre.dist(g))).expect("a house");
    let away = d.centre.sub(g);
    let hide = w.free_spot(d.centre.add(away.scale((d.radius + 3.0) / away.len().max(0.01))));
    assert!(w.in_the_way(g, hide, 0.0).is_some(), "the house stands between the guard and the thief");
    let k = w.squad.index(me).unwrap();
    w.squad.at[k] = hide;
    w.squad.goal[k] = hide;
    w.squad.route[k].clear();
    w.pursuits[0].last_seen = hide;
    w.pursuits[0].seen_t = w.time;
    let (mut trail, mut n) = (vec![g], 0);
    while !w.pursuits.is_empty() && n < 20_000 {
        w.step(0.25);
        n += 1;
        if let Some(at) = w.chase_pos(guard) {
            trail.push(at);
        }
    }
    for s in trail.windows(2) {
        assert_eq!(w.in_the_way(s[0], s[1], 0.0), None, "the guard ran through something between {:?} and {:?}", s[0], s[1]);
    }
    assert!(w.log.iter().any(|l| l.1.contains("catches")), "and caught the thief: {:?}", w.log.iter().take(3).collect::<Vec<_>>());
}

#[test]
fn a_thief_who_gets_away_has_a_bounty_instead() {
    let (mut w, town, me, folk) = in_town();
    assert!(told(&mut w, town, me, &folk));
    // Off and away before the watch arrives.
    let far = w.settlements[town as usize].pos.add(V2::new(w.settlements[town as usize].reach + 400.0, 0.0));
    let k = w.squad.index(me).unwrap();
    w.squad.at[k] = far;
    w.squad.goal[k] = far;
    w.squad.route[k].clear();
    let mut n = 0;
    while !w.pursuits.is_empty() && n < 20_000 {
        w.step(0.25);
        n += 1;
    }
    assert!(w.pursuits.is_empty());
    assert!(w.bounty.get(&town).copied().unwrap_or(0.0) > 0.0, "a bounty: {:?}", w.log);
    assert!(w.log.iter().any(|l| l.1.contains("slipped the watch")));
}

/// Stand by until the chase is over; true if it ended with the thief caught.
fn caught_in_the_end(w: &mut World) -> bool {
    let mut n = 0;
    while !w.pursuits.is_empty() && n < 20_000 {
        w.step(0.25);
        n += 1;
    }
    assert!(w.pursuits.is_empty(), "the chase ended");
    w.log.iter().any(|l| l.1.contains("catches"))
}

/// NM-20: stepping into the house next door doesn't shake the watch off:
/// a door close by where the thief was last seen is tried.
#[test]
fn nm20_the_watch_follows_through_the_next_door() {
    use gahturiyu_sim::sim::buildings::door_of;
    let (mut w, town, me, folk) = in_town();
    // Seen at it in the street, 25 m from a house that can be walked into.
    let s = w.settlements[town as usize].clone();
    let mut doors: Vec<_> = (0..s.buildings.len() as u16).filter_map(|i| door_of(&s, i)).filter(|d| !w.is_locked(d.id)).collect();
    doors.sort_by(|a, b| a.outside.dist(s.pos).total_cmp(&b.outside.dist(s.pos)));
    let d = doors.first().expect("an open door in town").clone();
    let out = d.outside.sub(d.inside);
    let from = d.outside.add(out.scale(25.0 / out.len().max(0.01)));
    let k = w.squad.index(me).unwrap();
    w.squad.at[k] = from;
    w.squad.goal[k] = from;
    w.squad.route[k].clear();
    assert!(told(&mut w, town, me, &folk), "somebody tells");
    // And in they go.
    w.order_members(&[me], d.centre);
    for _ in 0..600 {
        if w.squad.inside[k] == Some(d.id) || w.pursuits.is_empty() {
            break;
        }
        w.step(0.25);
    }
    // (Caught on the way in is the watch doing its job too.)
    assert!(w.squad.inside[k] == Some(d.id) || w.log.iter().any(|l| l.1.contains("catches")), "got indoors");
    assert!(caught_in_the_end(&mut w), "indoors next door, the thief slipped the watch: {:?}", w.log.iter().take(4).collect::<Vec<_>>());
    assert!(w.bounty.is_empty());
}

/// NM-20: nor does walking a hundred metres down the street in broad day,
/// with people about to point the way.
#[test]
fn nm20_the_watch_follows_down_the_street() {
    let (mut w, town, me, folk) = in_town();
    assert!(told(&mut w, town, me, &folk), "somebody tells");
    let from = w.person_pos(me);
    let s = w.settlements[town as usize].clone();
    // 110 m off, still in town, out of doors.
    let to = (0..16)
        .map(|i| {
            let a = i as f32 / 16.0 * std::f32::consts::TAU;
            from.add(V2::new(a.cos(), a.sin()).scale(110.0))
        })
        .filter(|&p| p.dist(s.pos) < s.reach && w.building_at(p).is_none() && !w.terrain.is_sea(p))
        .min_by(|a, b| a.dist(s.pos).total_cmp(&b.dist(s.pos)))
        .expect("a spot down the street");
    let k = w.squad.index(me).unwrap();
    w.squad.at[k] = to;
    w.squad.goal[k] = to;
    w.squad.route[k].clear();
    assert!(caught_in_the_end(&mut w), "110 m down the street, the thief slipped the watch: {:?}", w.log.iter().take(4).collect::<Vec<_>>());
}

#[test]
fn keeping_quiet_means_nothing_follows() {
    let (mut w, town, me, folk) = in_town();
    // Someone who'd never tell: a close friend with no scruples.
    let mut quiet = 0;
    for &f in &folk {
        w.regard.insert(f, 100.0);
        let at = w.person_pos(me);
        let said = w.log.len() + w.alerts.len();
        let last = w.log.front().cloned();
        w.wrong_seen(me, town, Wrong::Theft, 30.0, "Seen stealing!".into(), Some(f), None, at);
        if w.pursuits.is_empty() && w.bounty.is_empty() {
            quiet += 1;
            // Nothing comes of it, and nothing is said (BL-37).
            assert!(w.log.front().cloned() == last && w.log.len() + w.alerts.len() == said, "{:?}", w.log.front());
        }
        w.pursuits.clear();
        w.bounty.clear();
    }
    eprintln!("{quiet} of {} friends kept quiet", folk.len());
    assert!(quiet > 0, "friends look away sometimes");
}

/// NM-28: a crime seen with nobody to come running still says something
/// follows, and what's owed can be looked up afterwards.
#[test]
fn nm28_a_bounty_posted_says_so_and_can_be_looked_up() {
    let mut w = worldgen::generate(1);
    assert!(w.wanted_lines().is_empty(), "nothing owed at the start");
    // A town with no guard on watch just now (some hour of the night).
    let mut found = None;
    'hours: for _ in 0..48 {
        w.step(HOUR / 2.0);
        let t = w.time;
        for s in 0..w.settlements.len() as u16 {
            let guards = w.settlements[s as usize].residents.iter().filter(|&&p| !w.people[p as usize].dead && w.life(p).job == Job::Guard && w.at_work(p, t)).count();
            if guards == 0 && w.settlements[s as usize].residents.len() > 40 {
                found = Some(s);
                break 'hours;
            }
        }
    }
    let town = found.expect("a town with nobody on watch at some hour");
    w.teleport_squad(w.settlements[town as usize].pos);
    let me = w.squad.members[0];
    let at = w.person_pos(me);
    // (No witness named: the town saw it for itself, so it is told.)
    w.wrong_seen(me, town, Wrong::Theft, 40.0, "Seen stealing!".into(), None, None, at);
    assert!(w.pursuits.is_empty(), "nobody on watch to send");
    let owed = w.bounty.get(&town).copied().unwrap_or(0.0);
    assert!(owed >= 40.0, "a bounty instead: {owed}");
    let alert = w.alerts.last().cloned().unwrap_or_default();
    assert!(alert.contains("Seen stealing!") && alert.contains("watch will hear"), "the line the player sees says it isn't over: {alert:?}");
    let name = w.settlements[town as usize].name.clone();
    assert_eq!(w.wanted_lines(), vec![format!("Wanted in {name}: {owed:.0} coin owed.")]);
    // Paid off, it's gone from the list.
    w.bounty.insert(town, 0.0);
    assert!(w.wanted_lines().is_empty());
}

fn guards_of(w: &World, town: usize) -> Vec<PersonId> {
    w.settlements[town].residents.iter().copied().filter(|&p| !w.people[p as usize].dead && w.life(p).job == Job::Guard).collect()
}

/// NM-40: a town with two guards or more keeps a watch through the night
/// (it used to take three, and the whole watch sat down to eat at 02:00);
/// and the night watch walks the town instead of standing at the post.
#[test]
fn nm40_someone_keeps_watch_at_night_and_walks_the_town() {
    let mut w = worldgen::generate(3);
    let towns: Vec<usize> = (0..w.settlements.len()).filter(|&s| guards_of(&w, s).len() >= 2).collect();
    assert!(towns.len() > 10);
    // The second night, at one, two and half past three in the morning.
    let mut dark = Vec::new();
    for h in [25.0, 26.25, 27.5] {
        while w.time < 24.0 * HOUR + h * HOUR {
            w.step(300.0);
        }
        let t = w.time;
        for &s in &towns {
            if !guards_of(&w, s).iter().any(|&p| w.at_work(p, t)) {
                dark.push((w.settlements[s].name.clone(), h - 24.0));
            }
        }
    }
    // (A town whose one night guard has the night off is dark that night;
    // that's a rest day, not a town without a watch.)
    eprintln!("{} towns with two guards or more; unwatched: {dark:?}", towns.len());
    for h in [1.0, 2.25, 3.5] {
        let n = dark.iter().filter(|d| d.1 == h).count();
        // (A third or so: each town's night guard has their night off now
        // and then, and some are away on the roads.)
        assert!(n * 100 <= towns.len() * 35, "at {h} in the morning {n} of {} towns have nobody on watch: {dark:?}", towns.len());
    }
    // Someone on the night watch is out walking: they move, they stay in
    // town, and where they are is the same whenever it's asked.
    let t = w.time;
    let (town, walker) = towns.iter().find_map(|&s| guards_of(&w, s).into_iter().find(|&p| w.life(p).shift == 1 && w.at_work(p, t)).map(|p| (s, p))).expect("a night guard on watch");
    let here = w.person_pos(walker);
    assert_eq!(here, w.person_pos(walker));
    let (centre, reach) = (w.settlements[town].pos, w.settlements[town].reach);
    let mut far = 0.0f32;
    for _ in 0..8 {
        w.step(300.0);
        let at = w.person_pos(walker);
        if w.at_work(walker, w.time) {
            far = far.max(at.dist(here));
            assert!(at.dist(centre) <= reach + 60.0, "the watch keeps to the town: {:.0} m from its middle", at.dist(centre));
        }
    }
    assert!(far > 15.0, "in forty minutes the night watch moved {far:.0} m");
}

/// NM-40: with nobody at the post, a witness fetches a guard who is up
/// and close by, though off duty.
#[test]
fn nm40_a_guard_near_by_comes_though_off_duty() {
    let mut w = worldgen::generate(3);
    // Mid-afternoon: the night watch is up and at home, off duty.
    while w.time < 15.5 * HOUR {
        w.step(300.0);
    }
    let t = w.time;
    let found = (0..w.settlements.len()).find_map(|s| guards_of(&w, s).into_iter().find(|&p| w.life(p).shift == 1 && !w.at_work(p, t) && !w.is_indoors_asleep(p) && w.is_about(p, t)).map(|p| (s, p)));
    let (town, off) = found.expect("a night guard up and off duty in the afternoon");
    // The day watch is laid out cold: nobody at the post to fetch.
    for g in guards_of(&w, town) {
        if g != off {
            let p = &mut w.people[g as usize];
            let max = p.stats.max_hp(gahturiyu_sim::sim::body::Part::Torso);
            p.wounds.lost = p.wounds.lost_at(t);
            p.wounds.lost[1] = max + 20.0;
            p.wounds.at = t;
        }
    }
    assert!(!guards_of(&w, town).iter().any(|&p| w.at_work(p, t)), "nobody on watch");
    let near = w.person_pos(off).add(V2::new(20.0, 0.0));
    w.teleport_squad(near);
    let me = w.squad.members[0];
    w.wrong_seen(me, town as u16, Wrong::Theft, 30.0, "Seen stealing!".into(), None, None, near);
    assert_eq!(w.pursuits.iter().map(|p| p.guard).collect::<Vec<_>>(), vec![off], "the guard 20 m away is fetched: {:?}", w.log.iter().take(3).collect::<Vec<_>>());
    assert!(w.bounty.is_empty());
}

// ---- N8 (#144, NM-22): the watch and a wanted squad ---------------------------

fn put(w: &mut World, who: PersonId, at: V2) {
    let k = w.squad.index(who).unwrap();
    w.squad.at[k] = at;
    w.squad.goal[k] = at;
    w.squad.route[k].clear();
}

/// A spot out of doors this far from `from`, and at least `clear` from
/// every guard of the town but the one stood at `from`.
fn spot_by(w: &World, town: u16, from: V2, dist: f32, clear: f32) -> V2 {
    let guards = guards_of(w, town as usize);
    (0..16)
        .map(|k| from.add(V2::new((k as f32 * 0.3927).cos(), (k as f32 * 0.3927).sin()).scale(dist)))
        .find(|&p| w.building_at(p).is_none() && guards.iter().all(|&g| w.person_pos(g).dist(from) < 0.5 || w.person_pos(g).dist(p) >= clear))
        .expect("a clear spot by the guard")
}

/// A town where something is owed and member 0's face is known for it (a
/// theft told of, and the thief away before the watch came); member 0 left
/// far outside. With it, a guard of the town on watch out of doors, and
/// what's owed.
fn wanted() -> (World, u16, PersonId, PersonId, f32) {
    let (mut w, town, me, folk) = in_town();
    assert!(told(&mut w, town, me, &folk));
    let s = &w.settlements[town as usize];
    let far = s.pos.add(V2::new(s.reach + 400.0, 0.0));
    put(&mut w, me, far);
    let mut n = 0;
    while !w.pursuits.is_empty() && n < 20_000 {
        w.step(0.25);
        n += 1;
    }
    let owed = w.bounty.get(&town).copied().unwrap_or(0.0);
    assert!(owed > 0.0 && w.wanted.contains(&(town, me)), "owed {owed}, known {:?}", w.wanted);
    let t = w.time;
    let guard = guards_of(&w, town as usize).into_iter().find(|&g| w.at_work(g, t) && w.building_at(w.person_pos(g)).is_none()).expect("a guard on watch out of doors");
    (w, town, me, guard, owed)
}

fn wait(w: &mut World, secs: f32) {
    for _ in 0..(secs / 0.25) as usize {
        w.step(0.25);
    }
}

/// A guard of the town who sees a wanted face close by knows it and comes;
/// caught, what was owed is called in and the matter is closed.
#[test]
fn n8_a_guard_who_sees_a_wanted_face_close_by_comes_after_them() {
    let (mut w, town, me, guard, owed) = wanted();
    let gp = w.person_pos(guard);
    let coin = gahturiyu_sim::sim::items::id("coin");
    let purse = w.squad_count(coin);
    let at = spot_by(&w, town, gp, 8.0, 0.0);
    put(&mut w, me, at);
    w.step(0.25);
    assert!(w.pursuits.iter().any(|p| p.culprit == me && p.town == town && p.owed), "known and gone after: {:?}", w.pursuits);
    assert!(w.alerts.iter().any(|a| a.contains("of the watch knows")), "{:?}", w.alerts.last());
    // Nothing more is owed for being known: the debt is what it was.
    assert_eq!(w.charge_on(me), owed);
    assert!(caught_in_the_end(&mut w), "{:?}", w.log.iter().rev().take(4).collect::<Vec<_>>());
    assert!(w.log.iter().any(|l| l.1.contains("is held in")), "called in");
    assert!(w.bounty.get(&town).is_none() && !w.wanted.iter().any(|x| x.0 == town), "the matter is closed: {:?} {:?}", w.bounty, w.wanted);
    // Paid, or to be worked off, or riding on a duel, by the town's custom.
    let paid = purse - w.squad_count(coin);
    assert!(paid > 0 || w.is_bonded(me, w.time) || !w.duels.is_empty(), "something came of it");
    // And with nothing owed, nobody comes again.
    wait(&mut w, 30.0);
    assert!(w.pursuits.is_empty());
}

/// Only if spotted and recognised (Laz): not a squadmate nobody saw at it,
/// not from across the square, not through a disguise, not creeping past
/// out of plain sight.
#[test]
fn n8_only_if_spotted_and_recognised() {
    use gahturiyu_sim::sim::effects::{lasting, Does, Reach};
    use gahturiyu_sim::sim::pursuit::KNOW_FACE;
    let (mut w, town, me, guard, owed) = wanted();
    let gp = w.person_pos(guard);
    // A squadmate who wasn't seen at it stands right by the guard.
    let other = w.squad.members[1];
    let at = spot_by(&w, town, gp, 5.0, 0.0);
    put(&mut w, other, at);
    wait(&mut w, 20.0);
    assert!(w.pursuits.is_empty(), "nobody knows this one's face");
    // The one who was, in sight but too far off to be known.
    let at = spot_by(&w, town, gp, 55.0, KNOW_FACE + 15.0);
    put(&mut w, me, at);
    wait(&mut w, 20.0);
    assert!(w.pursuits.is_empty(), "too far to know a face");
    // Close by, in disguise.
    assert!(w.apply_effect(me, &lasting(Does::Disguise, 1.0, 1800.0, Reach::Caster), 1.0));
    let at = spot_by(&w, town, gp, 6.0, 0.0);
    put(&mut w, me, at);
    wait(&mut w, 20.0);
    assert!(w.pursuits.is_empty(), "not through a disguise");
    w.boons.clear();
    let at = spot_by(&w, town, gp, 55.0, KNOW_FACE + 15.0);
    put(&mut w, me, at);
    // Creeping: known only from much nearer.
    w.set_sneaking(me, true);
    w.step(0.25);
    let near = KNOW_FACE * w.visibility_of(me);
    assert!(near < KNOW_FACE * 0.6, "creeping is harder to make out: {near}");
    let at = spot_by(&w, town, gp, (near * 1.6).max(near + 2.0), near * 1.5);
    put(&mut w, me, at);
    wait(&mut w, 20.0);
    assert!(w.pursuits.is_empty(), "crept past at {:.0} m (known within {near:.0})", (near * 1.6).max(near + 2.0));
    assert_eq!(w.bounty.get(&town).copied(), Some(owed));
    // Right under the guard's nose, creeping or not.
    let at = spot_by(&w, town, gp, (near * 0.5).max(0.8), 0.0);
    put(&mut w, me, at);
    w.step(0.25);
    // (Or caught on the spot, if another of the watch stood within reach.)
    assert!(w.pursuits.iter().any(|p| p.culprit == me && p.owed) || w.log.iter().any(|l| l.1.contains("is held in")), "known close to");
}

/// Known and lost again, it stays owed as it was (no more for running, and
/// the face is still known); a new wrong seen on the way is judged as one.
#[test]
fn n8_lost_again_it_stays_owed_and_a_new_wrong_is_judged() {
    let (mut w, town, me, guard, owed) = wanted();
    let gp = w.person_pos(guard);
    let at = spot_by(&w, town, gp, 8.0, 0.0);
    put(&mut w, me, at);
    w.step(0.25);
    assert!(w.pursuits.iter().any(|p| p.culprit == me && p.owed));
    let s = &w.settlements[town as usize];
    let far = s.pos.add(V2::new(s.reach + 400.0, 0.0));
    put(&mut w, me, far);
    wait(&mut w, 5.0);
    assert!(w.pursuits.is_empty(), "out of town, lost");
    assert_eq!(w.bounty.get(&town).copied(), Some(owed), "the same debt, no more");
    assert!(w.wanted.contains(&(town, me)));
    assert_eq!(w.log.iter().filter(|l| l.1.contains("slipped the watch")).count(), 2);
    // Back, known again; and seen at something new on the way.
    let at = spot_by(&w, town, gp, 8.0, 0.0);
    put(&mut w, me, at);
    w.step(0.25);
    assert!(w.pursuits.iter().any(|p| p.culprit == me && p.owed));
    let at = w.person_pos(me);
    w.wrong_seen(me, town, Wrong::Trespass, 20.0, "Seen where they shouldn't be!".into(), None, None, at);
    let p = w.pursuits.iter().find(|p| p.culprit == me).expect("still chased");
    assert!(!p.owed && p.wrong == Wrong::Trespass && p.fine == 20.0, "{p:?}");
}
