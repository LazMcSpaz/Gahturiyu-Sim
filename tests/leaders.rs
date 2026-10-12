//! Faction leaders as quest-givers (docs/leaders.md, F6). Leaders have no
//! seats until Laz's map, so each test seats someone by hand.

use gahturiyu_sim::sim::{
    dialogue::Topic,
    factions::{self, LEADERS},
    geo::V2,
    items,
    leaders::{self, Ask, TASKS, TRUSTED},
    person::PersonId,
    quests::{QuestKind, Stage},
    talk, worldgen, World,
};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

/// Townsfolk of the squad's starting town, awake and willing to talk.
fn locals(w: &World) -> Vec<PersonId> {
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap();
    town.residents.iter().copied().filter(|&p| !w.people[p as usize].dead && !w.is_indoors_asleep(p)).collect()
}

/// Stand next to someone and start talking.
fn talk_to(w: &mut World, npc: PersonId) -> PersonId {
    let lead = w.squad.members[0];
    let at = w.person_pos(npc);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    w.squad.at[0] = at.add(V2::new(1.0, 0.0));
    assert!(w.order_talk(lead, npc));
    walk(w, 2.0);
    assert!(w.talk.is_some(), "the conversation should open");
    lead
}

#[test]
fn every_task_names_a_real_leader_item_and_line() {
    for t in TASKS {
        let k = leaders::leader_index(t.leader).unwrap_or_else(|| panic!("no leader {}", t.leader));
        let _ = factions::id(LEADERS[k].faction);
        match t.ask {
            Ask::Fetch { item, .. } => {
                let _ = items::id(item);
            }
            Ask::Letter { to } => assert!(leaders::leader_index(to).is_some() && to != t.leader, "{}", t.key),
            Ask::BreakCamp { .. } => {}
        }
        let tag = format!("task={}", t.key);
        let has = |slot: &str| talk::pieces().iter().any(|p| p.topic == "leader" && p.slot == slot && p.must.iter().any(|m| *m == tag));
        assert!(has("offer"), "an offer line for {}", t.key);
        assert!(has("thanks"), "a thanks line for {}", t.key);
        assert!(t.needs <= 0.0 || has("not_yet"), "a not-yet line for {}", t.key);
        assert_eq!(TASKS.iter().filter(|x| x.key == t.key).count(), 1, "{} twice", t.key);
    }
}

#[test]
fn a_seated_leader_goes_by_their_name_and_belongs_to_their_faction() {
    let mut w = worldgen::generate(1);
    let who = locals(&w)[3];
    assert!(w.seat_leader("Hoqa", who));
    assert_eq!(w.people[who as usize].name(), Some("Hoqa"));
    assert_eq!(w.leader_style(who).as_deref(), Some("Hoqa of the Green Wall"));
    assert_eq!(w.factions_of(who)[0], factions::id("wardens"));
    assert_eq!(w.leader_seated("Hoqa"), Some(who));
    // Seated again elsewhere: one person to a seat.
    let other = locals(&w)[5];
    assert!(w.seat_leader("Hoqa", other));
    assert_eq!(w.leader_seated("Hoqa"), Some(other));
    assert!(w.leader_of(who).is_none());
    // Nobody of the squad can be seated.
    let m = w.squad.members[0];
    assert!(!w.seat_leader("Haʻo", m));
}

#[test]
fn a_fetch_job_from_a_leader_pays_and_raises_standing() {
    let mut w = worldgen::generate(1);
    let who = locals(&w)[2];
    w.seat_leader("Hoqa", who);
    let (kind, coin, _) = w.quest_offer(who).expect("Hoqa has work");
    let QuestKind::Fetch { item, count } = kind else { panic!("a fetch: {kind:?}") };
    assert_eq!(item, items::id("bandage"));
    let lead = talk_to(&mut w, who);
    let said = w.ask(Topic::Work).unwrap();
    assert!(said.contains("bandages"), "{said}");
    assert!(w.topics().contains(&Topic::Accept));
    w.ask(Topic::Accept).unwrap();
    let qi = w.quests.iter().position(|q| q.task == Some("bandages_for_the_wall")).expect("in the journal");
    assert_eq!(w.quests[qi].giver, who);
    // Taken: it isn't offered again.
    assert!(w.leader_offer(who).is_none());
    // Hand them over.
    let before = w.standing_with(factions::id("wardens"));
    let rooted = w.standing_with(factions::id("rooted"));
    let coin_before = w.count_of(lead, "coin");
    w.people[lead as usize].detail.as_mut().unwrap().gear.add(item, count);
    let said = w.ask(Topic::Report(qi)).unwrap();
    assert_eq!(w.quests[qi].stage, Stage::Done, "{said}");
    assert!(said.contains(&coin.to_string()), "their own thanks, with the pay: {said}");
    assert_eq!(w.count_of(lead, "coin"), coin_before + coin);
    assert!(w.standing_with(factions::id("wardens")) > before);
    // The Wardens' enemies think the less of it.
    assert!(w.standing_with(factions::id("rooted")) < rooted);
}

#[test]
fn a_letter_goes_to_another_seated_leader() {
    let mut w = worldgen::generate(2);
    let l = locals(&w);
    let (niralea, nemu) = (l[1], l[6]);
    w.seat_leader("Niralea", niralea);
    // With nobody seated as Nemu there's nothing to carry.
    assert!(w.leader_offer(niralea).is_none());
    w.seat_leader("Nemu", nemu);
    let lead = w.squad.members[0];
    let qid = w.accept_quest(niralea, lead).expect("a letter to carry");
    let q = w.quests[qid as usize].clone();
    assert_eq!(q.kind, QuestKind::Deliver { to: nemu });
    assert!(w.count_of(lead, "sealed_letter") >= 1);
    talk_to(&mut w, nemu);
    let i = qid as usize;
    assert!(w.topics().contains(&Topic::Letter(i)));
    let tide = w.standing_with(factions::id("tide_houses"));
    w.ask(Topic::Letter(i));
    assert_eq!(w.quests[i].stage, Stage::Done);
    assert!(w.standing_with(factions::id("tide_houses")) > tide);
}

#[test]
fn breaking_the_binders_camp_for_the_council_points_at_slavers() {
    let mut w = worldgen::generate(1);
    let who = locals(&w)[4];
    w.seat_leader("Waarua", who);
    let lead = w.squad.members[0];
    let qid = w.accept_quest(who, lead).expect("a camp to break");
    let QuestKind::ClearCamp { camp, .. } = w.quests[qid as usize].kind.clone() else { panic!() };
    assert!(w.is_slaver_camp(camp));
    assert!(w.quest_line(&w.quests[qid as usize].clone()).contains("Binders'"));
    // Broken: report back.
    w.beaten_camps.insert(camp);
    if let Some(c) = w.camps.iter_mut().find(|c| c.group == camp) {
        c.ready_at = w.time + 86_400.0;
    }
    walk(&mut w, 1.0);
    assert_eq!(w.quests[qid as usize].stage, Stage::Report);
}

#[test]
fn the_high_priestess_wants_the_war_chiefs_word_first() {
    let mut w = worldgen::generate(1);
    let l = locals(&w);
    let (textrim, totkit) = (l[2], l[7]);
    w.seat_leader("Textrim", textrim);
    w.seat_leader("Totkit", totkit);
    assert!(w.leader_offer(textrim).is_none());
    assert!(w.leader_holds_back(textrim).is_some());
    talk_to(&mut w, textrim);
    assert!(w.topics().contains(&Topic::Work));
    let said = w.ask(Topic::Work).unwrap();
    assert!(said.contains("Totkit"), "{said}");
    assert!(!w.topics().contains(&Topic::Accept));
    w.ask(Topic::Goodbye);
    // Totkit's test passed: now she'll ask.
    let lead = w.squad.members[0];
    let qid = w.accept_quest(totkit, lead).expect("Totkit's test") as usize;
    w.quests[qid].stage = Stage::Report;
    talk_to(&mut w, totkit);
    w.ask(Topic::Report(qid));
    assert!(w.standing_with(factions::id("last_wave")) >= TRUSTED);
    assert!(w.leader_offer(textrim).is_some());
}
