//! Government, law and bondage.

use gahturiyu_sim::sim::{
    culture::{Justice, Rule, Slavery},
    law::{self, Post},
    person::PersonId,
    race::Race,
    settlement::SettlementId,
    world::{DAY, HOUR},
    worldgen, World,
};

fn run(w: &mut World, secs: f64) {
    let n = (secs / HOUR).round() as usize;
    for _ in 0..n {
        w.step(HOUR);
    }
}

fn towns(w: &World) -> Vec<SettlementId> {
    (0..w.settlements.len() as SettlementId).collect()
}

fn has(w: &World, t: SettlementId, r: Rule) -> bool {
    w.government(t).chambers.iter().any(|c| c.rule == r)
}

#[test]
fn every_town_is_governed_by_real_people() {
    let w = worldgen::generate(1);
    for t in towns(&w) {
        let g = w.government(t);
        assert!(!g.chambers.is_empty() || g.council, "{} has no government", w.settlements[t as usize].name);
        for c in &g.chambers {
            for &p in &c.holders {
                let pp = &w.people[p as usize];
                assert!(!pp.dead && pp.home == Some(t));
                match c.rule {
                    Rule::Elders => assert_eq!(pp.race, Race::Roduro),
                    Rule::Priestesses => assert!(pp.race == Race::Qotiro && law::woman(pp.seed)),
                    Rule::Speaker => assert_eq!(pp.race, Race::Horaro),
                }
            }
            for &p in &c.administrators {
                assert!(!law::woman(w.people[p as usize].seed), "administrators are men");
            }
        }
        // A stilt village always has its speaker; the sea is theirs.
        if w.society.towns[t as usize].stilts.is_some() {
            assert!(has(&w, t, Rule::Speaker));
            assert_eq!(w.owner_of(t, law::Matter::Sea), Some(law::Owner::Chamber(Rule::Speaker)));
        }
    }
    // Mixed towns come out with mixed governments.
    assert!(towns(&w).iter().any(|&t| w.government(t).chambers.len() >= 2));
    assert!(towns(&w).iter().any(|&t| w.government(t).chambers.len() == 1));
}

#[test]
fn the_form_of_government_follows_a_big_shift_in_who_lives_there() {
    let mut w = worldgen::generate(1);
    let t = towns(&w).into_iter().filter(|&t| has(&w, t, Rule::Elders)).max_by_key(|&t| w.town_counts(t)[0]).unwrap();
    // The Roduro of the town are gone.
    for p in w.settlements[t as usize].residents.clone() {
        if w.people[p as usize].race == Race::Roduro {
            w.people[p as usize].dead = true;
        }
    }
    run(&mut w, DAY + HOUR);
    assert!(!has(&w, t, Rule::Elders), "no Roduro left: no elder circle");
}

#[test]
fn an_unpaid_watch_breeds_unrest_until_the_town_rises() {
    let mut w = worldgen::generate(1);
    let t = 3;
    let before: Vec<u32> = w.government(t).chambers.iter().flat_map(|c| c.holders.clone()).collect();
    assert!(!before.is_empty());
    w.society.towns[t as usize].treasury = 0.0;
    w.society.towns[t as usize].owed = 1e6;
    let mut peak = 0.0f32;
    for _ in 0..14 {
        run(&mut w, DAY);
        peak = peak.max(w.government(t).unrest);
        if w.government(t).revolts > 0 {
            break;
        }
    }
    assert!(peak > 20.0, "unrest rose: {peak}");
    assert!(w.government(t).revolts > 0, "and the town rose");
    let after: Vec<u32> = w.government(t).chambers.iter().flat_map(|c| c.holders.clone()).collect();
    assert!(before.iter().all(|p| !after.contains(p)), "its rulers were thrown out");
    assert!(w.government(t).unrest < law::REVOLT_AT);
}

#[test]
fn a_bad_omen_can_bring_the_high_priestess_down() {
    let mut w = worldgen::generate(1);
    let t = towns(&w).into_iter().find(|&t| has(&w, t, Rule::Priestesses) && w.government(t).chambers.iter().any(|c| c.rule == Rule::Priestesses && c.holders.len() >= 2)).unwrap();
    let high = |w: &World| w.government(t).chambers.iter().find(|c| c.rule == Rule::Priestesses).unwrap().holders[0];
    let first = high(&w);
    // A town doing badly holds its rites badly.
    let well = w.rite_chance(t);
    w.society.towns[t as usize].gov.unrest = 90.0;
    assert!(w.rite_chance(t) < well);
    // Omens on several days: an arbiter may rule some of them false, not all.
    for d in 0..12 {
        w.bad_omen(t, w.time + d as f64 * DAY);
        if high(&w) != first {
            break;
        }
    }
    assert_ne!(high(&w), first, "she fell");
    assert!(w.government(t).fallen.contains(&first));
}

#[test]
fn a_term_of_bondage_ends_on_schedule() {
    let mut w = worldgen::generate(1);
    let t = 0;
    let p = w.settlements[t as usize].residents[5];
    let now = w.time;
    w.bond(p, t, None, now, now + 2.0 * DAY);
    assert!(w.is_bonded(p, now + DAY));
    run(&mut w, 2.0 * DAY + HOUR);
    assert!(!w.is_bonded(p, w.time), "free again");
}

#[test]
fn roduro_law_forbids_selling_a_roduro_debtor_into_slavery() {
    let mut w = worldgen::generate(1);
    // Where slavery is allowed: one town where the elders sit, one where they don't.
    let allow = |w: &mut World, t: SettlementId| {
        let ci = w.society.towns[t as usize].shore;
        w.society.communities[ci as usize].customs.slavery = Slavery::Allowed;
    };
    let with = towns(&w).into_iter().find(|&t| has(&w, t, Rule::Elders)).unwrap();
    let without = towns(&w).into_iter().find(|&t| !has(&w, t, Rule::Elders) && w.town_counts(t)[0] > 0).unwrap();
    for t in [with, without] {
        allow(&mut w, t);
        let debtor = *w.settlements[t as usize].residents.iter().find(|&&p| w.people[p as usize].race == Race::Roduro && !w.people[p as usize].dead).unwrap();
        let now = w.time;
        w.bond(debtor, t, None, now, now + 10.0 * DAY);
        let k = w.bonds.iter().position(|b| b.who == debtor).unwrap();
        let sold = w.sell_bond(k, now);
        if t == with {
            assert_eq!(sold, Err("Roduro law forbids it"));
            assert!(!w.bonds[k].slave);
        } else {
            assert_eq!(sold, Ok(()));
            assert!(w.bonds[k].slave && w.is_bonded(debtor, now + 1000.0 * DAY), "a slave for life");
        }
    }
}

#[test]
fn a_shunning_stops_the_dawn_boats() {
    let mut w = worldgen::generate(1);
    let t = towns(&w).into_iter().find(|&t| w.society.towns[t as usize].stilts.is_some()).unwrap();
    run(&mut w, DAY + 2.0 * HOUR);
    assert!(w.society.towns[t as usize].landed > 0.0, "the boats come in");
    let now = w.time;
    w.wrong_village(t, now);
    run(&mut w, DAY);
    assert_eq!(w.society.towns[t as usize].landed, 0.0, "the village has withdrawn");
    run(&mut w, law::SHUN_DAYS * DAY + DAY);
    assert!(w.society.towns[t as usize].landed > 0.0, "and comes back");
}

#[test]
fn no_standing_makes_someone_eligible_who_isnt() {
    let mut w = worldgen::generate(1);
    let posts = [Post::Elder, Post::Priestess, Post::Administrator, Post::Speaker, Post::Arbiter];
    let t = towns(&w).into_iter().max_by_key(|&t| w.government(t).chambers.len()).unwrap();
    let mut took = false;
    for m in w.squad.members.clone() {
        w.standing_in.insert((m, t), 1000.0);
        for post in posts {
            let res = w.take_post(m, t, post);
            if !w.eligible(m, post) {
                assert_eq!(res, Err("not eligible for that post"), "{post:?}");
            } else if res.is_ok() {
                took = true;
            }
        }
    }
    assert!(took, "someone eligible could take a post");
    // And standing still counts for the eligible.
    let m = w.squad.members[0];
    let other = (t + 1) % w.settlements.len() as SettlementId;
    let post = posts.into_iter().find(|&p| w.eligible(m, p)).unwrap();
    assert_eq!(w.take_post(m, other, post), Err("not enough standing here"));
}

#[test]
fn a_judgement_that_cant_be_paid_is_worked_off() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let t = 0;
    let coin = gahturiyu_sim::sim::items::id("coin");
    let have = w.squad_count(coin);
    w.judge(m, t, law::Wrong::Theft, have as f32 + 80.0, Justice::Elders);
    assert_eq!(w.squad_count(coin), 0, "everything paid that could be");
    assert!(w.is_bonded(m, w.time), "the rest is worked off");
    let price = w.buy_out_price(m).unwrap();
    assert!(price >= 80);
    w.people[w.squad.members[1] as usize].detail.as_mut().unwrap().gear.add(coin, price);
    assert!(w.buy_out(m).is_ok());
    assert!(!w.is_bonded(m, w.time));
}

#[test]
fn a_duel_is_fought_out_and_the_loser_pays() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let t = w.settlements.iter().min_by(|a, b| a.pos.dist(w.squad.pos).total_cmp(&b.pos.dist(w.squad.pos))).unwrap().id;
    w.judge(m, t, law::Wrong::Theft, 30.0, Justice::Duel);
    assert_eq!(w.duels.len(), 1);
    let id = w.duels[0].battle;
    assert!(w.battle(id).is_some(), "the fight is on");
    for _ in 0..4000 {
        w.step(0.5);
        if w.duels.is_empty() {
            break;
        }
    }
    assert!(w.duels.is_empty(), "settled");
    assert!(!w.people[m as usize].dead, "no one dies of a duel");
}

// ---- The fixes after the bug hunt ----------------------------------------------------------

fn give_coin(w: &mut World, who: PersonId, n: u16) {
    w.people[who as usize].detail.as_mut().unwrap().gear.add(gahturiyu_sim::sim::items::id("coin"), n);
}

fn strip_coin(w: &mut World) {
    let coin = gahturiyu_sim::sim::items::id("coin");
    for m in w.squad.members.clone() {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        while d.gear.take(coin) {}
    }
    assert_eq!(w.squad_count(coin), 0);
}

/// NM-38: the sum named is the sum taken, and what's taken is said.
#[test]
fn nm38_a_fine_says_what_was_taken() {
    let coin = gahturiyu_sim::sim::items::id("coin");
    // Enough coin: "a fine of 12" takes 12, not 11.
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    strip_coin(&mut w);
    give_coin(&mut w, m, 100);
    w.judge(m, 0, law::Wrong::Theft, 11.6, Justice::Elders);
    assert_eq!(w.squad_count(coin), 88);
    assert!(w.log.iter().any(|l| l.1.contains("a fine of 12.")), "{:?}", w.log);
    assert!(w.log.iter().any(|l| l.1.ends_with("pays the fine: 12 coin.")), "{:?}", w.log);
    assert!(!w.is_bonded(m, w.time));
    // Part of it: said, with what's left.
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    strip_coin(&mut w);
    give_coin(&mut w, m, 18);
    w.judge(m, 0, law::Wrong::Theft, 75.0, Justice::Elders);
    assert_eq!(w.squad_count(coin), 0);
    assert!(w.log.iter().any(|l| l.1.contains("pays 18 of the 75 coin")), "{:?}", w.log);
    assert!(w.is_bonded(m, w.time));
    // None of it.
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    strip_coin(&mut w);
    w.judge(m, 0, law::Wrong::Theft, 40.0, Justice::Elders);
    assert!(w.log.iter().any(|l| l.1.contains("can't pay the 40 coin")), "{:?}", w.log);
    // By the record: half the fine, and the sum is named.
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    strip_coin(&mut w);
    give_coin(&mut w, m, 50);
    w.judge(m, 0, law::Wrong::Theft, 13.0, Justice::Record);
    let taken = 50 - w.squad_count(coin);
    assert!(taken == 6 || taken == 7, "{taken}");
    assert!(w.log.iter().any(|l| l.1.ends_with(&format!("pays the fine: {taken} coin."))), "{:?}", w.log);
}

/// NM-30: an arrest wakes the accused, and nobody asleep is sent to fight.
#[test]
fn nm30_nobody_sleeps_through_an_arrest_or_fights_a_duel_asleep() {
    let mut w = worldgen::generate(1);
    let all = w.squad.members.clone();
    w.order_rest(&all);
    for _ in 0..20 {
        w.step(30.0);
    }
    assert!(all.iter().all(|&m| w.is_asleep(m)), "everyone is asleep");
    let m = all[2];
    let t = w.settlements.iter().min_by(|a, b| a.pos.dist(w.squad.pos).total_cmp(&b.pos.dist(w.squad.pos))).unwrap().id;
    w.judge(m, t, law::Wrong::Theft, 30.0, Justice::Duel);
    assert!(!w.is_asleep(m), "the accused is up");
    assert_eq!(w.duels.len(), 1);
    let b = w.battle(w.duels[0].battle).expect("the duel is on");
    let ours: Vec<PersonId> = b.fighters.iter().filter(|f| f.side == gahturiyu_sim::sim::combat::SQUAD_SIDE).map(|f| f.pid).collect();
    assert_eq!(ours, vec![m], "with the others asleep, the accused fights their own duel");
}

fn nearest_town(w: &World) -> SettlementId {
    w.settlements.iter().min_by(|a, b| a.pos.dist(w.squad.pos).total_cmp(&b.pos.dist(w.squad.pos))).unwrap().id
}

fn said(w: &World, what: &str) -> bool {
    w.log.iter().any(|l| l.1.contains(what))
}

/// NM-24: what a town saw stolen goes back when the thief is caught there;
/// what can't be found is added to the fine; put down, it is still its
/// owner's.
#[test]
fn nm24_what_was_seen_stolen_goes_back_on_arrest() {
    use gahturiyu_sim::sim::{inventory::Entry, items};
    let (coin, helm, cap) = (items::id("coin"), items::id("iron_helm"), items::id("leather_cap"));
    let mut w = worldgen::generate(1);
    let (m, mate) = (w.squad.members[0], w.squad.members[1]);
    let town = nearest_town(&w);
    strip_coin(&mut w);
    give_coin(&mut w, m, 200);
    // A helm the thief still has, a cap handed to a squadmate standing by,
    // and a second helm nobody has any more.
    let count = |w: &World, it| w.squad.members.iter().map(|&p| w.people[p as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == it).map(|e| e.1).sum::<u16>()).sum::<u16>();
    assert_eq!((count(&w, helm), count(&w, cap)), (0, 0), "the squad starts with neither in its packs");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(helm, 1);
    w.people[mate as usize].detail.as_mut().unwrap().gear.add(cap, 1);
    w.mark_hot(town, &[Entry(helm, 2, None), Entry(cap, 1, None)], None);
    assert!(w.is_hot(town, helm) && w.is_hot(town, cap));
    w.judge(m, town, law::Wrong::Theft, 30.0, Justice::Elders);
    assert_eq!(count(&w, helm), 0, "the helm is taken back");
    assert_eq!(count(&w, cap), 0, "and the cap from the squadmate standing by");
    assert!(said(&w, "takes back what was stolen: iron helm, leather cap."), "{:?}", w.log);
    assert!(w.hot.is_empty(), "the town has no more claim");
    // The missing helm's worth (120) is on top of the fine of 30.
    assert!(said(&w, "a fine of 150."), "{:?}", w.log);
    assert_eq!(w.squad_count(coin), 200 - 150, "{:?}", w.log);

    // Put down in the town that knows it, it lies there as the town's.
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let town = nearest_town(&w);
    let at = w.settlements[town as usize].pos;
    w.teleport_squad(at);
    w.people[m as usize].detail.as_mut().unwrap().gear.add(helm, 1);
    w.mark_hot(town, &[Entry(helm, 1, None)], None);
    let before = w.ground.len();
    assert!(w.drop_item(m, helm));
    assert_eq!(w.ground.len(), before + 1);
    assert_eq!(w.ground.last().unwrap().owner, Some(town), "still the town's");
    assert!(!w.is_hot(town, helm), "and back with it");
}

/// NM-29: an arrest calls in what the town already held against the squad.
#[test]
fn nm29_an_arrest_calls_in_what_is_already_owed() {
    let coin = gahturiyu_sim::sim::items::id("coin");
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let town = nearest_town(&w);
    strip_coin(&mut w);
    give_coin(&mut w, m, 300);
    w.bounty.insert(town, 130.0);
    w.judge(m, town, law::Wrong::Theft, 30.0, Justice::Elders);
    assert_eq!(w.squad_count(coin), 300 - 160);
    assert!(!w.bounty.contains_key(&town), "nothing is still owed: {:?}", w.bounty);
    assert!(said(&w, "a fine of 30, and the 130 already owed: 160 in all."), "{:?}", w.log);
    // A bounty somewhere else is that town's business.
    let other = (town + 1) % w.settlements.len() as SettlementId;
    w.bounty.insert(other, 50.0);
    w.judge(m, town, law::Wrong::Theft, 10.0, Justice::Elders);
    assert_eq!(w.bounty.get(&other), Some(&50.0));
}

/// NM-31: the accused fights their own duel, the town sends someone fit,
/// and the two start apart.
#[test]
fn nm31_the_accused_fights_and_the_champion_is_fit() {
    use gahturiyu_sim::sim::combat::SQUAD_SIDE;
    let mut w = worldgen::generate(1);
    let town = nearest_town(&w);
    // The weakest of the squad is accused, with the others awake beside them.
    let m = w.squad.members.iter().copied().min_by(|&a, &b| w.people[a as usize].might.total_cmp(&w.people[b as usize].might)).unwrap();
    let mut champions = Vec::new();
    for round in 0..3 {
        let t = w.time;
        w.judge(m, town, law::Wrong::Theft, 20.0, Justice::Duel);
        if w.duels.is_empty() {
            // (Nobody fit left to send, or the accused can't stand: a plain fine.)
            break;
        }
        let b = w.battle(w.duels[0].battle).expect("the duel is on");
        let ours: Vec<&_> = b.fighters.iter().filter(|f| f.side == SQUAD_SIDE).collect();
        let theirs: Vec<&_> = b.fighters.iter().filter(|f| f.side != SQUAD_SIDE).collect();
        assert_eq!(ours.iter().map(|f| f.pid).collect::<Vec<_>>(), vec![m], "round {round}: the accused, not a champion");
        assert_eq!(theirs.len(), 1);
        assert!(w.health_share(theirs[0].pid, t) >= law::CHAMPION_FIT, "round {round}: the town's champion is at {:.0}%", w.health_share(theirs[0].pid, t) * 100.0);
        let gap = ours[0].pos.dist(theirs[0].pos);
        assert!((gap - law::DUEL_GAP).abs() < 0.5, "they start {gap:.1} m apart");
        champions.push(theirs[0].pid);
        for _ in 0..4000 {
            w.step(0.5);
            if w.duels.is_empty() {
                break;
            }
        }
        assert!(w.duels.is_empty(), "settled");
        if w.is_down(m) || w.is_bonded(m, w.time) {
            break;
        }
    }
    assert!(!champions.is_empty(), "at least one duel was fought");
}

/// NM-32: a town's champion beaten in a duel lies where they fell.
#[test]
fn nm32_a_beaten_champion_lies_where_they_fell() {
    use gahturiyu_sim::sim::combat::SQUAD_SIDE;
    let mut seen = 0;
    for seed in 1..6u64 {
        let mut w = worldgen::generate(seed);
        let town = nearest_town(&w);
        let m = w.squad.members.iter().copied().max_by(|&a, &b| w.people[a as usize].might.total_cmp(&w.people[b as usize].might)).unwrap();
        w.judge(m, town, law::Wrong::Theft, 20.0, Justice::Duel);
        let Some(d) = w.duels.first().copied() else { continue };
        let champion = w.battle(d.battle).unwrap().fighters.iter().find(|f| f.side != SQUAD_SIDE).unwrap().pid;
        let mut fell = None;
        for _ in 0..4000 {
            if let Some(f) = w.battle(d.battle).and_then(|b| b.fighters.iter().find(|f| f.pid == champion)) {
                fell = Some(f.pos);
            }
            w.step(0.5);
            if w.duels.is_empty() {
                break;
            }
        }
        if !w.is_down(champion) {
            continue;
        }
        seen += 1;
        let at = w.person_pos(champion);
        assert!(at.dist(fell.unwrap()) < 3.0, "seed {seed}: the beaten champion is {:.0} m from where they fell", at.dist(fell.unwrap()));
        // And still there a few minutes on, while still down.
        for _ in 0..10 {
            w.step(30.0);
        }
        if w.is_down(champion) {
            assert!(w.person_pos(champion).dist(at) < 0.5);
        }
    }
    assert!(seen > 0, "no duel in five worlds ended with the champion down");
}

/// NM-34: a town feeds those it holds to work.
#[test]
fn nm34_a_town_feeds_those_it_holds() {
    use gahturiyu_sim::sim::items::{item, Kind};
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let town = nearest_town(&w);
    w.people[m as usize].detail.as_mut().unwrap().gear.bag.retain(|e| !matches!(item(e.0).kind, Kind::Food(_)));
    let now = w.time;
    w.bond(m, town, None, now, now + 6.0 * DAY);
    run(&mut w, 5.0 * DAY);
    assert!(w.is_bonded(m, w.time));
    assert!(!w.is_down(m), "not starved at their work");
    let hunger = w.hunger_of(m).unwrap();
    assert!(hunger < gahturiyu_sim::sim::condition::STARVING, "fed by the town: hunger {hunger:.0} after five days, {:?}", w.log);
    // A free member with no food isn't fed by anyone.
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    for p in w.squad.members.clone() {
        w.people[p as usize].detail.as_mut().unwrap().gear.bag.retain(|e| !matches!(item(e.0).kind, Kind::Food(_)));
    }
    run(&mut w, 5.0 * DAY);
    assert!(w.hunger_of(m).unwrap() >= gahturiyu_sim::sim::condition::STARVING, "free and without food for five days, they starve: {:?}", w.hunger_of(m));
}

/// NM-35: someone bound can't be carried off, and running costs more than
/// what was owed.
#[test]
fn nm35_the_bound_arent_carried_off_and_running_costs_more() {
    use gahturiyu_sim::sim::{body, geo::V2};
    let mut w = worldgen::generate(1);
    let (m, mate) = (w.squad.members[0], w.squad.members[1]);
    let town = nearest_town(&w);
    // Down, and not bound: a squadmate can pick them up.
    let t = w.time;
    {
        let p = &mut w.people[m as usize];
        let max = p.stats.max_hp(body::Part::Torso);
        p.wounds.lost = p.wounds.lost_at(t);
        p.wounds.lost[1] = max + 20.0;
        p.wounds.at = t;
    }
    assert!(w.is_down(m));
    assert!(w.can_carry(mate, m), "down and free: can be carried");
    w.bond(m, town, None, t, t + 10.0 * DAY);
    assert!(!w.can_carry(mate, m), "down and bound: stays where the town has them");
    // Run (taken far off): the bounty is the days still owed and more.
    let left = w.buy_out_price(m).unwrap() as f32;
    let far = w.settlements[town as usize].pos.add(V2::new(w.settlements[town as usize].reach + 2000.0, 0.0));
    w.teleport_squad(far);
    w.step(0.5);
    assert!(!w.is_bonded(m, w.time), "run");
    let owed = w.bounty.get(&town).copied().unwrap_or(0.0);
    assert!(owed >= left + law::RUN_PRICE - 1.0, "running costs {owed:.0}, the bond was {left:.0}");
}

/// NM-36: "for 3 days" is three days; the days left can be read; and the
/// end of the bond is said, once, at the moment it ends.
#[test]
fn nm36_a_bond_says_how_long_and_when_it_ends() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let town = nearest_town(&w);
    strip_coin(&mut w);
    let start = w.time;
    // 20 coin short at 8 a day: three whole days.
    w.judge(m, town, law::Wrong::Theft, 20.0, Justice::Elders);
    let place = w.settlements[town as usize].name.clone();
    assert!(said(&w, &format!("is bound to work in {place} for 3 days.")), "{:?}", w.log);
    assert_eq!(w.bond_days_left(m), Some(3));
    assert_eq!(w.bond_of(m).unwrap().until, start + 3.0 * DAY);
    run(&mut w, 2.5 * DAY);
    assert_eq!(w.bond_days_left(m), Some(1));
    assert!(w.is_bonded(m, w.time));
    // (To an hour past its end: the news only keeps its last few lines.)
    run(&mut w, 0.5 * DAY + HOUR);
    assert!(!w.is_bonded(m, w.time));
    let ended: Vec<&(f64, String)> = w.log.iter().filter(|l| l.1.contains("is worked off")).collect();
    assert_eq!(ended.len(), 1, "{:?}", w.log);
    assert_eq!(ended[0].0, start + 3.0 * DAY, "dated when it ended, however time was stepped");
    assert!(w.bonds.iter().all(|b| b.who != m), "and the bond is gone");
    // One day's worth says "a day".
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    strip_coin(&mut w);
    w.judge(m, town, law::Wrong::Theft, 5.0, Justice::Elders);
    assert!(said(&w, "for a day."), "{:?}", w.log);
}

/// NM-37: a guard of the town, wherever met, can sell the bond.
#[test]
fn nm37_any_guard_or_official_of_the_town_sells_the_bond() {
    use gahturiyu_sim::sim::{dialogue::Topic, geo::V2, jobs::Job};
    let mut w = worldgen::generate(1);
    let (me, bound) = (w.squad.members[0], w.squad.members[1]);
    let town = nearest_town(&w);
    let guard = w.settlements[town as usize].residents.iter().copied().find(|&p| !w.people[p as usize].dead && w.life(p).job == Job::Guard && !w.is_indoors_asleep(p)).expect("a guard who is up");
    let at = w.person_pos(guard);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    w.squad.at[0] = at.add(V2::new(1.0, 0.0));
    let t = w.time;
    w.bond(bound, town, None, t, t + 4.0 * DAY);
    assert!(w.order_talk(me, guard));
    for _ in 0..20 {
        w.step(0.1);
    }
    assert!(w.talk.is_some(), "talking");
    let price = w.buy_out_price(bound).unwrap();
    assert!(w.topics().iter().any(|t| matches!(t, Topic::BuyOut(p, _) if *p == bound)), "the guard offers the bond ({price} coin): {:?}", w.topics());
}

/// NM-27: a note pays as fifty coin, with change.
#[test]
fn nm27_notes_pay_with_change() {
    let (coin, note) = (gahturiyu_sim::sim::items::id("coin"), gahturiyu_sim::sim::items::id("note"));
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let town = nearest_town(&w);
    strip_coin(&mut w);
    for p in w.squad.members.clone() {
        let d = w.people[p as usize].detail.as_mut().unwrap();
        while d.gear.take(note) {}
    }
    give_coin(&mut w, m, 6);
    w.people[w.squad.members[1] as usize].detail.as_mut().unwrap().gear.add(note, 1);
    assert_eq!(w.squad_count(coin), 56, "six coin and a note is 56");
    assert_eq!(w.squad_has(coin), 6);
    w.judge(m, town, law::Wrong::Theft, 40.0, Justice::Elders);
    assert!(!w.is_bonded(m, w.time), "the note paid: {:?}", w.log);
    assert_eq!((w.squad_has(coin), w.squad_has(note)), (16, 0), "40 paid out of 56, the note broken for change");
    assert!(said(&w, "pays the fine: 40 coin."), "{:?}", w.log);
    // Coin enough: no note is touched.
    w.people[m as usize].detail.as_mut().unwrap().gear.add(note, 2);
    w.judge(m, town, law::Wrong::Theft, 10.0, Justice::Elders);
    assert_eq!((w.squad_has(coin), w.squad_has(note)), (6, 2));
}
