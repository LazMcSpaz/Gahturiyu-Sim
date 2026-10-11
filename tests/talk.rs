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
    // They name their town (in their own tongue, its English name alongside).
    let home = w.people[npc as usize].home.unwrap();
    assert!(w.talk.as_ref().unwrap().lines.last().unwrap().1.contains(&w.settlements[home as usize].name));
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
    // They go in (bandits don't always pick a fight with veterans).
    let first = w.groups.iter().find(|g| g.id == camp).map(|g| g.members[0]).expect("the camp's band");
    let all = w.squad.members.clone();
    assert!(w.attack(&all, first), "the squad goes in");
    walk(&mut w, 600.0);
    // (A long fight, or one others joined: let it finish.)
    let mut n = 0;
    while w.squad_battle().is_some() && n < 6000 {
        w.step(0.5);
        n += 1;
    }
    assert!(w.beaten_camps.contains(&camp) || !w.camps.iter().any(|c| c.group == camp), "the camp should be beaten");
    assert_eq!(w.quests[0].stage, Stage::Report);
    talk_to(&mut w, npc);
    w.ask(Topic::Report(0));
    assert_eq!(w.quests[0].stage, Stage::Done);
    // Paid what was promised (or a favour owed, from someone who can't pay).
    assert!(w.squad_count(items::id("coin")) >= w.quests[0].coin);
}

/// NM-54: a talk ends when the two part; nothing is said or sold at a distance.
#[test]
fn a_talk_ends_when_the_two_part() {
    let mut w = worldgen::generate(1);
    w.step(3.0 * 3600.0);
    let npc = locals(&w).into_iter().find(|&p| !w.is_indoors_asleep(p) && !w.people[p as usize].dead && w.building_at(w.person_pos(p)).is_none()).expect("someone out of doors");
    let lead = talk_to(&mut w, npc);
    // Standing with them, it stays open.
    walk(&mut w, 3.0);
    assert!(w.talk.is_some(), "still talking");
    assert!(w.topics().len() > 1);
    // Walk off: it's over, and nothing can be asked.
    let k = w.squad.index(lead).unwrap();
    let away = w.squad.at[k].add(V2::new(60.0, 0.0));
    w.order_members(&[lead], away);
    walk(&mut w, 60.0);
    assert!(w.squad.at[k].dist(w.person_pos(npc)) > 20.0, "they parted");
    assert!(w.talk.is_none(), "the talk is over once they've parted");
    assert!(w.topics().is_empty());
}

/// NM-62: the first-meeting greeting once; "you again" and warmth only for
/// someone they've talked with before.
#[test]
fn greetings_follow_whether_you_have_met() {
    use gahturiyu_sim::sim::talk;
    let mut w = worldgen::generate(1);
    w.step(3.0 * 3600.0);
    let lead = w.squad.members[0];
    // The local who thinks best of the lead on sight (sociable, the same people).
    let npc = locals(&w)
        .into_iter()
        .filter(|&p| !w.people[p as usize].dead && !w.is_indoors_asleep(p))
        .max_by(|&a, &b| w.regard_of(a, lead).total_cmp(&w.regard_of(b, lead)))
        .unwrap();
    w.regard.insert(npc, 40.0);
    assert!(w.regard_of(npc, lead) > talk::WARM, "well disposed from the start: {:.0}", w.regard_of(npc, lead));
    let musts = |w: &World| -> Vec<&'static str> {
        let said = &w.talk.as_ref().unwrap().pieces;
        talk::pieces().iter().filter(|p| said.contains(&p.id)).flat_map(|p| p.must.clone()).collect()
    };
    assert!(!w.met.contains(&(npc, lead)));
    talk_to(&mut w, npc);
    let first = musts(&w);
    assert!(!first.contains(&"warm") && !first.contains(&"known"), "a first meeting isn't greeted as a friend: {first:?}: {}", w.talk.as_ref().unwrap().lines[0].1);
    assert!(w.met.contains(&(npc, lead)));
    w.ask(Topic::Goodbye);
    talk_to(&mut w, npc);
    let second = musts(&w);
    assert!(!second.contains(&"stranger"), "met before: {second:?}: {}", w.talk.as_ref().unwrap().lines[0].1);
    // Someone else of the squad is still a stranger to them.
    assert!(!w.met.contains(&(npc, w.squad.members[1])));
}

/// NM-64, NM-67: hearsay has no count; a thing taken is named as it's said.
#[test]
fn rumours_have_no_count_and_things_taken_read_right() {
    use gahturiyu_sim::sim::talk;
    for p in talk::pieces() {
        assert!(p.topic != "rumours" || !p.text.contains("{count}"), "{}", p.text);
        assert!(p.topic != "money" || !p.text.contains("We owe {debt} coin"), "{}", p.text);
    }
    assert_eq!(talk::a_thing(items::id("spear")), "a spear");
    assert_eq!(talk::a_thing(items::id("iron_helm")), "an iron helm");
    assert_eq!(talk::a_thing(items::id("flatbread")), "flatbread");
    assert_eq!(talk::a_thing(items::id("arrows")), "arrows");
}

/// NM-63 = BL-36: two days standing in town, and the news holds nothing the
/// squad couldn't see or wouldn't miss.
#[test]
fn the_news_isnt_crowded_with_passers_by() {
    let mut w = worldgen::generate(1);
    let mut all: Vec<String> = Vec::new();
    for _ in 0..2 * 24 * 6 {
        w.step(600.0);
        for l in w.log.iter() {
            if !all.contains(&l.1) {
                all.push(l.1.clone());
            }
        }
    }
    for l in &all {
        for no in ["bound for", "heading home", "crosses your path", "comes into view", "takes up work as", " m away", "for a day's work"] {
            assert!(!l.contains(no), "pushed at the squad: {l}");
        }
    }
}

/// RG-6: people don't all say hello the same way. The greeting follows the
/// speaker's people, job, temper, mood and the hour; every tag a greeting
/// asks for is one the game sets, and every word of their own is a real one.
#[test]
fn rg6_greetings_vary_by_who_is_speaking() {
    use gahturiyu_sim::names;
    use gahturiyu_sim::sim::{jobs, talk};
    use std::collections::BTreeSet;
    let job_tags: Vec<String> = jobs::ALL_JOBS.iter().map(|j| format!("job={}", j.name().to_lowercase())).collect();
    let peoples = ["roduro", "qotiro", "horaro", "tadoro"];
    let mut greetings = 0;
    for p in talk::pieces().iter().filter(|p| p.slot == "greeting") {
        greetings += 1;
        for c in p.must.iter().chain(p.not.iter()) {
            if c.starts_with("job=") {
                assert!(job_tags.contains(&c.to_string()), "no such job: {c}");
            }
            if let Some(v) = c.strip_prefix("voice=").or(c.strip_prefix("you=")) {
                assert!(peoples.contains(&v), "no such people: {c}");
            }
        }
        let mut rest = p.text;
        while let Some(i) = rest.find("{w:").into_iter().chain(rest.find("{W:")).min() {
            let tail = &rest[i + 3..];
            let end = tail.find('}').expect("a closed brace");
            assert!(names::root(&tail[..end]).is_some(), "no such root `{}` in: {}", &tail[..end], p.text);
            rest = &tail[end..];
        }
    }
    // One town, first meeting, one moment: many different hellos, and
    // different trades among them.
    let w = worldgen::generate(1);
    let lead = w.squad.members[0];
    let mut said: BTreeSet<u16> = BTreeSet::new();
    let mut folk = 0;
    for p in locals(&w) {
        if w.people[p as usize].dead {
            continue;
        }
        let s = w.assemble_talk(p, lead, None, true);
        if s.refused || s.pieces.is_empty() {
            continue;
        }
        folk += 1;
        said.insert(s.pieces[0]);
    }
    let by_job = said.iter().filter(|&&id| talk::pieces()[id as usize].must.iter().any(|c| c.starts_with("job="))).count();
    println!("{folk} townsfolk, {} different greetings, {by_job} of them by trade", said.len());
    assert!(said.len() >= 15, "{folk} townsfolk greet in only {} ways", said.len());
    assert!(by_job >= 3, "only {by_job} greetings by trade");
    assert!(greetings >= 100, "{greetings} greetings");
}
