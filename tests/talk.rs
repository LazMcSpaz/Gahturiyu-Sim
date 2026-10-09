//! Conversations and jobs.

use gahturiyu_sim::sim::{
    dialogue::Topic,
    geo::V2,
    items,
    person::PersonId,
    quests::{QuestKind, Stage},
    stats::{Skill, SKILLS},
    worldgen, World,
};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

/// Townsfolk of the squad's starting town.
fn locals(w: &World) -> Vec<PersonId> {
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap();
    town.residents.clone()
}

/// Stand next to someone and start talking.
fn talk_to(w: &mut World, npc: PersonId) -> PersonId {
    let lead = w.squad.members[0];
    // Nobody talks in their sleep.
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

/// Someone offering that kind of job: jobs come from people's lives, so run
/// the world (an eventful one) until someone does.
fn find_offer(w: &mut World, want: fn(&QuestKind) -> bool) -> PersonId {
    for tl in &mut w.society.towns {
        tl.drama = 0.9;
    }
    for _ in 0..30 * 24 {
        for s in &w.settlements {
            for &p in &s.residents {
                if let Some((k, _, _)) = w.quest_offer(p) {
                    if want(&k) && w.busy_until[p as usize] <= w.time && !w.people[p as usize].dead {
                        return p;
                    }
                }
            }
        }
        w.step(3600.0);
    }
    panic!("nobody offers that");
}

#[test]
fn answers_are_steady_and_greetings_happen() {
    let mut w = worldgen::generate(1);
    let npc = locals(&w)[3];
    talk_to(&mut w, npc);
    w.ask(Topic::Advice);
    let first = w.talk.as_ref().unwrap().lines.last().unwrap().1.clone();
    w.ask(Topic::Advice);
    let again = w.talk.as_ref().unwrap().lines.last().unwrap().1.clone();
    assert_eq!(first, again);
    w.ask(Topic::ThisTown);
    assert!(w.talk.as_ref().unwrap().lines.last().unwrap().1.contains("founded"));
    w.ask(Topic::Goodbye);
    assert!(w.talk.is_none());
}

#[test]
fn a_bounty_sours_people() {
    let mut w = worldgen::generate(1);
    let npc = locals(&w)[5];
    let lead = w.squad.members[0];
    let before = w.disposition(npc, lead);
    let home = w.people[npc as usize].home.unwrap();
    w.bounty.insert(home, 400.0);
    assert!(w.disposition(npc, lead) < before - 50.0);
    talk_to(&mut w, npc);
    assert_eq!(w.topics(), vec![Topic::Goodbye], "they won't talk");
}

#[test]
fn a_fetch_job_from_start_to_finish() {
    let mut w = worldgen::generate(1);
    let npc = find_offer(&mut w, |k| matches!(k, QuestKind::Fetch { .. }));
    let lead = talk_to(&mut w, npc);
    assert!(w.topics().contains(&Topic::Work));
    w.ask(Topic::Work);
    assert!(w.topics().contains(&Topic::Accept));
    let regard = w.disposition(npc, lead);
    w.ask(Topic::Accept);
    assert_eq!(w.quests.len(), 1);
    let QuestKind::Fetch { item, count } = w.quests[0].kind.clone() else { panic!() };
    // Empty everyone's packs of it, so there's not enough yet.
    for m in w.squad.members.clone() {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        while d.gear.take(item) {}
    }
    w.ask(Topic::Report(0));
    assert_eq!(w.quests[0].stage, Stage::Active);
    w.people[lead as usize].detail.as_mut().unwrap().gear.add(item, count);
    w.ask(Topic::Report(0));
    assert_eq!(w.quests[0].stage, Stage::Done);
    assert_eq!(w.squad_count(item), 0, "handed over");
    assert_eq!(w.squad_count(items::id("coin")), w.quests[0].coin);
    assert!(w.disposition(npc, lead) > regard, "they like you better now");
    assert!(w.quest_offer(npc).is_none(), "one job each");
}

#[test]
fn a_letter_delivered() {
    let mut w = worldgen::generate(1);
    let npc = find_offer(&mut w, |k| matches!(k, QuestKind::Deliver { .. }));
    talk_to(&mut w, npc);
    w.ask(Topic::Work);
    w.ask(Topic::Accept);
    let QuestKind::Deliver { to } = w.quests[0].kind.clone() else { panic!() };
    assert_eq!(w.squad_count(items::id("sealed_letter")), 1);
    w.end_talk();
    // Off to the other town (the quick way).
    talk_to(&mut w, to);
    assert!(w.topics().contains(&Topic::Letter(0)));
    w.ask(Topic::Letter(0));
    assert_eq!(w.quests[0].stage, Stage::Done);
    assert_eq!(w.squad_count(items::id("sealed_letter")), 0);
    assert!(w.squad_count(items::id("coin")) > 0);
}

#[test]
fn breaking_a_camp_is_noticed_and_paid() {
    let mut w = worldgen::generate(1);
    let npc = find_offer(&mut w, |k| matches!(k, QuestKind::ClearCamp { .. }));
    talk_to(&mut w, npc);
    w.ask(Topic::Work);
    w.ask(Topic::Accept);
    w.end_talk();
    let QuestKind::ClearCamp { camp, at } = w.quests[0].kind.clone() else { panic!() };
    // A squad of veterans walks into the camp.
    for m in w.squad.members.clone() {
        for k in SKILLS {
            if w.people[m as usize].stats.skill(k) < 75.0 && matches!(k, Skill::Blunt | Skill::Spear | Skill::Blade | Skill::Dodge | Skill::Block | Skill::Structured) {
                w.people[m as usize].stats.set_skill(k, 75.0);
            }
        }
        w.people[m as usize].recompute_might();
    }
    w.teleport_squad(at.add(V2::new(20.0, 0.0)));
    walk(&mut w, 600.0);
    assert!(w.beaten_camps.contains(&camp) || !w.camps.iter().any(|c| c.group == camp), "the camp should be beaten");
    assert_eq!(w.quests[0].stage, Stage::Report);
    talk_to(&mut w, npc);
    w.ask(Topic::Report(0));
    assert_eq!(w.quests[0].stage, Stage::Done);
    // Paid what was promised (or a favour owed, from someone who can't pay).
    assert!(w.squad_count(items::id("coin")) >= w.quests[0].coin);
}
