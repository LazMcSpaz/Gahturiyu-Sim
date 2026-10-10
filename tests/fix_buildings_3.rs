//! Fixes from the known-issues list, batch 3 (the buildings agent).

use gahturiyu_sim::sim::{
    buildings::{door_of, Door},
    geo::V2,
    world::{DAY, HOUR},
    worldgen, World,
};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

fn until_hour(w: &mut World, h: f64) {
    while (w.time.rem_euclid(DAY) / HOUR - h).abs() > 0.05 {
        w.step(60.0);
    }
}

/// Doors with a lock in the town nearest the squad.
fn locked_doors(w: &World) -> Vec<Door> {
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(w.squad.pos).total_cmp(&b.pos.dist(w.squad.pos))).unwrap().id;
    let s = &w.settlements[town as usize];
    (0..s.buildings.len() as u16).filter_map(|i| door_of(s, i)).filter(|d| d.lock > 0.0).collect()
}

/// A spot out in the street in front of a door.
fn in_front(d: &Door, m: f32) -> V2 {
    let out = d.outside.sub(d.centre);
    d.outside.add(out.scale(m / out.len().max(0.1)))
}

#[test]
fn nm_42_a_door_that_locks_on_the_way_stops_them_outside() {
    let mut w = worldgen::generate(1);
    let d = locked_doors(&w)[0];
    until_hour(&mut w, 19.9);
    while w.time.rem_euclid(DAY) / HOUR < 19.995 {
        w.step(1.0);
    }
    // Far enough that they reach the door after 20:00.
    w.teleport_squad(in_front(&d, 40.0));
    let m = w.squad.members[0];
    assert!(!w.is_locked(d.id), "still open as they set off");
    w.order_members(&[m], d.centre);
    walk(&mut w, 120.0);
    assert!(w.is_locked(d.id));
    let k = w.squad.index(m).unwrap();
    assert_ne!(w.squad.inside[k], Some(d.id), "walked into a locked house");
    assert!(w.building_at(w.squad.at[k]).map(|b| b.id) != Some(d.id));
}

#[test]
fn nm_43_a_chest_is_reached_from_its_room_and_only_while_there() {
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 10.0);
    let m = w.squad.members[0];
    // A house with an unlocked chest in it.
    let mut found = None;
    for d in locked_doors(&w).into_iter().take(12) {
        w.teleport_squad(in_front(&d, 6.0));
        w.order_members(&[m], d.centre);
        walk(&mut w, 60.0);
        if let Some(c) = w.containers_in(d.id).find(|c| c.lock <= 0.0).map(|c| (c.id, c.pos)) {
            found = Some((d, c));
            break;
        }
    }
    let (d, (cid, cpos)) = found.expect("a house with an open chest");
    assert!(w.order_search(m, cid));
    walk(&mut w, 60.0);
    assert!(w.source_now(m).is_some(), "opened from beside it");
    // The same reach, but from outside the wall: not through it.
    let k = w.squad.index(m).unwrap();
    let out = cpos.sub(d.centre);
    let mut p = cpos;
    while w.building_at(p).is_some() {
        p = p.add(out.scale(0.1 / out.len().max(0.1)));
    }
    w.squad.at[k] = p;
    w.squad.goal[k] = p;
    w.squad.inside[k] = None;
    assert!(w.source_now(m).is_none(), "reached through the wall from {:.1} m", p.dist(cpos));
    // Walked away: the going-through is over.
    w.squad.at[k] = d.centre;
    w.squad.inside[k] = Some(d.id);
    w.order_members(&[m], in_front(&d, 30.0));
    walk(&mut w, 60.0);
    assert!(w.looting.iter().all(|l| l.who != m), "still going through a chest from the street");
}

#[test]
fn nm_50_a_trade_post_wants_the_trade_and_the_town_counts_the_hire() {
    use gahturiyu_sim::sim::jobs::Job;
    let w0 = worldgen::generate(1);
    // A crafter who gives up their trade: the post is going.
    let (town, smith) = (0..w0.settlements.len() as u16)
        .find_map(|t| w0.settlements[t as usize].residents.iter().copied().find(|&p| w0.life(p).job.craft().is_some() && w0.life(p).place.is_some()).map(|p| (t, p)))
        .expect("a crafter");
    let job = w0.life(smith).job;
    let craft = job.craft().unwrap();
    let who = w0.squad.members[0];
    let fresh = || {
        let mut w = worldgen::generate(1);
        w.society.lives[smith as usize].job = Job::Labourer;
        w
    };
    let locals = |w: &World| w.settlements[town as usize].residents.iter().filter(|&&p| w.life(p).job == job).count();
    let mut w = fresh();
    let place = w.vacant_posts(town).into_iter().find(|&(j, _)| j == job).map(|(_, pl)| pl).expect("the post is going");
    // Without the trade: not offered, not taken.
    w.people[who as usize].stats.set_skill(craft.skill(), 0.0);
    assert!(!w.posts_for_member(town, who).iter().any(|&(j, _)| j == job));
    assert!(!w.take_post_work(who, town, job, place));
    // Knowing it: taken, and the town doesn't hire a local for it as well.
    w.people[who as usize].stats.set_skill(craft.skill(), 60.0);
    assert!(w.posts_for_member(town, who).iter().any(|&(j, _)| j == job));
    assert!(w.take_post_work(who, town, job, place));
    let mut control = fresh();
    let before = locals(&w);
    for _ in 0..(3.0 * DAY / 600.0) as usize {
        w.step(600.0);
        control.step(600.0);
    }
    let (hired, without) = (locals(&w), locals(&control));
    eprintln!("{job:?} in {town}: locals before {before}, with the squad hired {hired}, without {without}");
    assert!(hired <= without);
    if without > before {
        assert!(hired < without, "the town filled the post the squad holds");
    }
}

#[test]
fn nm_57_a_secret_is_bought_from_those_who_know_and_only_paid_for_when_told() {
    use gahturiyu_sim::sim::{
        chances::Chance,
        history::Deed,
        items,
        memory::Who,
        talk::{Opt, Subject},
    };
    let mut w = worldgen::generate(1);
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().id;
    let res = w.settlements[town as usize].residents.clone();
    let (victim, thief) = (res[4], res[9]);
    let ev = w.note(Deed::Theft, Some(thief), Some(victim), town, w.time - 10.0 * HOUR, true);
    let day = World::day_of(w.time) as i32;
    w.remember(victim, Who::Someone, Deed::Theft, -0.6, day);
    let lead = w.squad.members[0];
    let coin = items::id("coin");
    w.people[lead as usize].detail.as_mut().unwrap().gear.add(coin, 20);
    // The one robbed: no "something for your trouble".
    let c = w.on_mind(victim);
    let k = c.iter().position(|x| x.event == Some(ev)).expect("the theft is on their mind");
    assert!(!w.options(victim, lead, &c, k).contains(&Opt::Bribe));
    // With the job taken, it weighs on the thief's own household.
    let mut o = w.opp(Chance::FindOut, victim, town);
    o.event = Some(ev);
    let id = w.post_opp(o, w.time).unwrap();
    w.take_opportunity(id, lead).unwrap();
    let h = w.society.lives[thief as usize].household.unwrap();
    let kin = w.society.households[h as usize].members.iter().copied().find(|&p| p != thief).unwrap_or(thief);
    let c = w.on_mind(kin);
    let k = c.iter().position(|x| x.subject == Subject::Theft && x.event == Some(ev)).expect("on the mind of those who know");
    assert!(w.options(kin, lead, &c, k).contains(&Opt::Bribe));
    // Someone who doesn't know takes nothing for saying so.
    let stranger = res.iter().copied().find(|&p| p != victim && !w.knows_culprit(p, thief) && w.society.lives[p as usize].household.is_some()).unwrap();
    let had = w.squad_count(coin);
    let said = w.say_opt(stranger, lead, &c, k, Opt::Bribe);
    assert_eq!(w.squad_count(coin), had, "paid for: {said}");
}

#[test]
fn nm_60_a_late_worker_still_goes_home_to_bed() {
    use gahturiyu_sim::sim::routine::Doing;
    let w = worldgen::generate(1);
    let day = World::day_of(w.time);
    let mut late = 0;
    for p in 0..w.people.len() as u32 {
        if w.people[p as usize].in_squad || w.people[p as usize].dead || w.people[p as usize].home.is_none() {
            continue;
        }
        for d in day..day + 7 {
            let plan = w.day_plan(p, d);
            let last = plan.segs().last().unwrap();
            // Night workers end the day at work.
            if matches!(last.doing, Doing::Work | Doing::Rounds | Doing::Ferry) {
                continue;
            }
            late += plan.work.is_some_and(|(_, we)| we >= 21.0) as usize;
            assert_eq!(last.doing, Doing::Asleep, "{} on day {d} (work {:?}) ends the day {:?} at {:.1}", w.name_of(p), plan.work, last.doing, last.from);
        }
    }
    assert!(late > 0, "someone works late");
}

#[test]
fn nm_49_missed_work_is_said_and_pay_follows_the_hours() {
    use gahturiyu_sim::sim::society::DAWN;
    let mut w = worldgen::generate(1);
    let (job, place) = w.vacant_posts(0).into_iter().find(|(j, _)| j.craft().is_none()).unwrap();
    let who = w.squad.members[0];
    let at = w.society.towns[0].places[place as usize].pos;
    // Too far to go: no work done.
    w.teleport_squad(at.add(V2::new(4000.0, 0.0)));
    assert!(w.take_post_work(who, 0, job, place));
    let first = w.contract_of(who).unwrap().first_day;
    let dawn = |d: i64| d as f64 * DAY + DAWN as f64 * HOUR;
    // The day after the first shift: a line for the missed day.
    while w.time < dawn(first + 1) + 60.0 {
        w.step(60.0);
    }
    assert!(w.log.iter().any(|(_, l)| l.contains("didn't come to work")), "{:?}", w.log);
    assert!(w.contract_of(who).is_some());
    // A part day: paid for the part.
    while w.time < dawn(first + 2) - 120.0 {
        w.step(60.0);
    }
    let shift = (w.contract_of(who).unwrap().hours.1 - w.contract_of(who).unwrap().hours.0) as f64 * HOUR;
    let pay = w.contract_of(who).unwrap().pay;
    w.society.contracts.iter_mut().find(|c| c.member == who).unwrap().present = shift * 0.3;
    let coin = w.count_of(who, "coin");
    while w.time < dawn(first + 2) + 60.0 {
        w.step(60.0);
    }
    let got = w.count_of(who, "coin") - coin;
    assert!(got > 0 && (got as f32) < pay * 0.5, "paid {got} of {pay} for a third of a shift");
    // Then three days missed in a row: the post goes, and it's said.
    let mut lost = false;
    for d in first + 3..first + 6 {
        while w.time < dawn(d) + 60.0 {
            w.step(60.0);
        }
        if w.contract_of(who).is_none() {
            lost = w.log.iter().any(|(_, l)| l.contains("lost the work"));
            break;
        }
    }
    assert!(w.contract_of(who).is_none(), "{:?}", w.contract_of(who));
    assert!(lost, "{:?}", w.log);
}

#[test]
fn bl_37_a_lock_is_told_in_words() {
    let w = worldgen::generate(1);
    for lock in [5.0, 30.0, 50.0, 90.0] {
        let words = w.lock_outlook(lock);
        assert!(!words.chars().any(|c| c.is_ascii_digit() || c == '%'), "{words}");
    }
}

#[test]
fn bl_49_a_day_standing_in_town_isnt_a_march() {
    use gahturiyu_sim::sim::condition::EXHAUSTED;
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 6.5);
    until_hour(&mut w, 21.0);
    for &m in &w.squad.members.clone() {
        let t = w.tired_of(m).unwrap();
        assert!(t < EXHAUSTED, "worn out ({t:.0}) from standing about");
    }
}

#[test]
fn nm_66_travellers_in_town_find_beds_at_night() {
    let mut w = worldgen::generate(1);
    let mut lodged = 0;
    for _ in 0..(3.0 * DAY / 600.0) as usize {
        w.step(600.0);
        let h = w.time.rem_euclid(DAY) / HOUR;
        for g in w.groups.iter().filter(|g| !g.hostile && !g.is_moving(w.time)) {
            let at = g.position_at(w.time);
            let in_town = w.settlements.iter().any(|s| s.pos.dist(at) < s.radius());
            if in_town && (h >= 22.0 || h < 5.5) {
                lodged += 1;
                assert!(g.members.iter().all(|&m| w.is_indoors_asleep(m)), "a party stands in the street at {h:.1}h");
            }
            if h > 8.0 && h < 20.0 {
                assert!(g.members.iter().all(|&m| !w.is_indoors_asleep(m)));
            }
        }
    }
    assert!(lodged > 0, "some party stopped in a town overnight");
}

#[test]
fn nm_53_nobody_joins_those_who_wronged_them_and_the_watch_stays() {
    use gahturiyu_sim::sim::{history::Deed, jobs::Job, memory::Who};
    let w0 = worldgen::generate(1);
    let (town, npc) = (0..w0.settlements.len() as u16).find_map(|t| w0.willing_in(t).first().map(|&(p, _)| (t, p))).expect("someone willing");
    let mut w = worldgen::generate(1);
    assert!(w.join_terms(npc).is_some());
    let day = World::day_of(w.time) as i32;
    w.remember(npc, Who::Person(w.squad.members[1]), Deed::Theft, -1.0, day);
    assert!(w.join_terms(npc).is_none(), "joins those who robbed them");
    assert!(w.why_not_join(npc).is_some());
    // Nobody of the watch or the shrine goes, anywhere.
    let w = worldgen::generate(1);
    for &p in &w.settlements[town as usize].residents {
        if matches!(w.life(p).job, Job::Guard | Job::Priest) {
            assert!(w.join_terms(p).is_none());
        }
    }
}

#[test]
fn nm_53_a_member_can_be_sent_away_in_town() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[2];
    let n = w.squad.members.len();
    // Out in the wild: not here.
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().clone();
    w.teleport_squad(town.pos.add(V2::new(town.radius() + 600.0, 0.0)));
    assert!(w.send_away(m).is_err());
    w.teleport_squad(town.pos);
    let line = w.send_away(m).unwrap();
    assert!(line.contains(&town.name), "{line}");
    assert_eq!(w.squad.members.len(), n - 1);
    assert!(!w.people[m as usize].in_squad);
    assert_eq!(w.people[m as usize].home, Some(town.id));
    assert!(w.settlements[town.id as usize].residents.contains(&m));
    // The town goes on with them in it.
    for _ in 0..(DAY / 600.0) as usize {
        w.step(600.0);
    }
    assert!(!w.people[m as usize].dead);
}

/// NM-33: one number for health. Read between settles it is the same as
/// what a fight starts from (a jump after that is a squadmate's mend, which
/// the fight's lines name).
#[test]
fn nm_33_health_read_now_is_what_a_fight_starts_from() {
    use gahturiyu_sim::sim::body::{self, Part};
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let t = w.time;
    {
        let p = &mut w.people[m as usize];
        let max = p.stats.max_hp(Part::Torso);
        p.wounds.lost = p.wounds.lost_at(t);
        p.wounds.lost[1] = max * 0.7;
        p.wounds.at = t;
    }
    for _ in 0..120 {
        w.step(30.0);
    }
    let read = |w: &World| {
        let p = &w.people[m as usize];
        let hp = p.wounds.hp_at(&p.stats, w.time);
        (hp[0] / p.stats.max_hp(Part::Head)).min(hp[1] / p.stats.max_hp(Part::Torso))
    };
    let before = read(&w);
    let here = w.squad.pos;
    let foe = (0..w.people.len() as u32).find(|&p| w.people[p as usize].bandit && !w.people[p as usize].dead && w.person_pos(p).dist(here) < 20000.0).unwrap();
    let at = w.person_pos(foe);
    w.teleport_squad(at.add(V2::new(3.0, 0.0)));
    assert!(w.attack(&[m], foe));
    let start = w.fighter(m).unwrap().vitality();
    assert!((start - before).abs() < 0.01, "{before:.3} before, {start:.3} as the fight starts");
    let _ = body::Part::Head;
}

/// The text tool, a command a call (as `tests/play_tool.rs` drives it).
fn play(save: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_play")).arg(save).args(args).output().expect("the play tool runs");
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success() && !text.contains("panicked"), "`{}` crashed:\n{text}", args.join(" "));
    text
}

fn play_save(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("gaht-fb3-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("t.save")
}

/// NM-14 (play): `search` on a locked chest sends whoever has picks, opens
/// it once picked, and says what came of it. NM-46: the hall, kitchens and
/// yards in town are listed.
#[test]
fn nm_14_nm_46_play_search_sends_the_picker_and_look_lists_the_town() {
    let save = play_save("search");
    assert!(play(&save, &["new", "1"]).contains("A new game"));
    let look = play(&save, &["look", "places"]);
    assert!(look.lines().any(|l| l.trim_start().starts_with('w') && l.contains("  the ")), "{look}");
    play(&save, &["enter", "b2.8"]);
    let out = play(&save, &["search", "k2.8.1"]);
    assert!(out.contains("Wahiwea picks the lock") || out.contains("Sausuyu picks the lock"), "{out}");
    assert!(out.contains("opens a chest"), "{out}");
    let _ = std::fs::remove_dir_all(save.parent().unwrap());
}
