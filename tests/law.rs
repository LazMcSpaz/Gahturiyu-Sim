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
