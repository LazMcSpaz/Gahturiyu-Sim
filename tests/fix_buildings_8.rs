//! The unconfirmed job slips from the known-issues list (U-7 to U-11), shown
//! happening first and then fixed (the buildings agent, batch 8).

use gahturiyu_sim::sim::{
    chances::{Chance, JOB_STANDING},
    dialogue::Topic,
    geo::V2,
    items,
    person::PersonId,
    quests::Stage,
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

/// Open a talk between the first squad member and `npc`.
fn talk_to(w: &mut World, npc: PersonId) -> PersonId {
    let lead = w.squad.members[0];
    while w.is_indoors_asleep(npc) {
        w.step(60.0);
    }
    let at = w.person_pos(npc);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    w.squad.at[0] = at.add(V2::new(1.0, 0.0));
    assert!(w.order_talk(lead, npc));
    walk(w, 2.0);
    assert!(w.talk.is_some(), "the conversation should open");
    lead
}

/// A taken job for `asker` of kind `kind`, at the report-back stage.
fn job(w: &mut World, asker: PersonId, kind: Chance, reward: u16) -> usize {
    let town = w.people[asker as usize].home.unwrap();
    let mut o = w.opp(kind, asker, town);
    o.reward = reward;
    let id = w.post_opp(o, w.time).unwrap();
    let lead = w.squad.members[0];
    let qi = w.take_opportunity(id, lead).unwrap() as usize;
    w.quests[qi].stage = Stage::Report;
    qi
}

fn someone(w: &World) -> PersonId {
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().id;
    w.settlements[town as usize].residents.iter().copied().find(|&p| w.society.lives[p as usize].household.is_some() && !w.people[p as usize].dead).unwrap()
}

/// U-7: someone in a town post can't take guard work as well.
#[test]
fn u7_one_paid_work_at_a_time() {
    let mut w = worldgen::generate(1);
    let who = w.squad.members[0];
    let (job, place) = w.vacant_posts(0).into_iter().find(|(j, _)| j.craft().is_none()).unwrap();
    assert!(w.take_post_work(who, 0, job, place));
    let asker = someone(&w);
    let town = w.people[asker as usize].home.unwrap();
    let mut o = w.opp(Chance::Guard, asker, town);
    o.place = Some(place);
    o.amount = 5.0;
    let id = w.post_opp(o, w.time).unwrap();
    assert!(w.take_opportunity(id, who).is_none(), "took guard work on top of a post");
}

/// U-9, U-10: a job pays what the giver has (their purse never goes below
/// nothing), and a lawful one raises standing once, not twice.
#[test]
fn u9_u10_a_job_pays_what_the_giver_has_and_counts_once() {
    let mut w = worldgen::generate(1);
    let asker = someone(&w);
    let town = w.people[asker as usize].home.unwrap();
    let qi = job(&mut w, asker, Chance::FindOut, 40);
    let h = w.society.lives[asker as usize].household.unwrap();
    w.society.households[h as usize].purse.coin = 6.0;
    let lead = talk_to(&mut w, asker);
    let (coin, standing) = (w.count_of(lead, "coin"), w.standing(lead, town));
    w.ask(Topic::Report(qi));
    assert_eq!(w.count_of(lead, "coin") - coin, 6, "paid more than they had");
    assert!(w.society.households[h as usize].purse.coin >= 0.0);
    assert!((w.standing(lead, town) - standing - JOB_STANDING).abs() < 0.01, "standing counted twice: +{}", w.standing(lead, town) - standing);
}

/// U-8: something to get back is paid for only with it in hand.
#[test]
fn u8_no_pay_for_a_thing_not_brought_back() {
    let mut w = worldgen::generate(1);
    let asker = someone(&w);
    let town = w.people[asker as usize].home.unwrap();
    let mut o = w.opp(Chance::Recover, asker, town);
    o.reward = 20;
    o.item = Some(items::id("gold_ring"));
    let id = w.post_opp(o, w.time).unwrap();
    let lead = w.squad.members[0];
    let qi = w.take_opportunity(id, lead).unwrap() as usize;
    w.quests[qi].stage = Stage::Report;
    talk_to(&mut w, asker);
    w.ask(Topic::Report(qi));
    assert_eq!(w.quests[qi].stage, Stage::Report, "paid without the ring");
}

/// U-11: a job whose giver has died is called off, not left waiting.
#[test]
fn u11_a_job_with_nobody_to_answer_to_is_off() {
    let mut w = worldgen::generate(1);
    let asker = someone(&w);
    let qi = job(&mut w, asker, Chance::FindOut, 10);
    w.people[asker as usize].dead = true;
    let until = w.time + DAY + HOUR;
    while w.time < until {
        w.step(600.0);
    }
    assert_eq!(w.quests[qi].stage, Stage::Done);
}
