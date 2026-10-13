//! Diʻub and Ḍeshu on Salt Isle (docs/tutorial-island.md): who they are and
//! where, and Diʻub's four questions over the meal.

use gahturiyu_sim::sim::{
    dialogue::Topic,
    diub::{BOOST, DESHU, DIUB, QUESTIONS, RATHER_NOT},
    geo::V2,
    island::{Spot, HUT_BUILDING},
    person::PersonId,
    routine,
    stats::Skill,
    talk, worldgen, World,
};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

/// The opening's world, the squad's one member standing by Diʻub.
fn at_the_hut(seed: u64) -> (World, PersonId, PersonId) {
    let mut w = worldgen::generate_opening(seed, None, Default::default());
    let diub = w.strangers.diub.expect("Diʻub is there");
    let at = w.person_pos(diub);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    let me = w.squad.members[0];
    (w, me, diub)
}

fn talk_to(w: &mut World, me: PersonId, npc: PersonId) {
    w.squad.at[0] = w.person_pos(npc).add(V2::new(1.0, 0.0));
    assert!(w.order_talk(me, npc));
    walk(w, 2.0);
    assert!(w.talk.is_some(), "the talk opens");
}

#[test]
fn diub_keeps_to_the_hut_and_the_lord_to_the_hall() {
    let w = worldgen::generate(1);
    let (d, l) = (w.strangers.diub.unwrap(), w.strangers.lord.unwrap());
    assert_eq!(w.people[d as usize].name(), Some(DIUB));
    assert_eq!(w.people[l as usize].name(), Some(DESHU));
    let isle = w.island().unwrap();
    assert_eq!(w.people[d as usize].dwelling, Some(HUT_BUILDING as u16));
    assert!(w.person_pos(d).dist(isle.spot(Spot::Hut)) < 25.0, "Diʻub at the hut");
    assert!(w.person_pos(l).dist(isle.spot(Spot::Hall)) < 30.0, "the lord in the hall");
    for p in [d, l] {
        assert!(w.bond_of(p).is_none(), "neither is a debtor in the compound's books");
        let plan = w.day_plan(p, World::day_of(w.time));
        assert!(plan.segs[..plan.n].iter().all(|s| s.spot == routine::Spot::Home));
        assert_eq!(gahturiyu_sim::sim::names::gender(w.people[p as usize].seed), gahturiyu_sim::names::Gender::Male);
    }
}

#[test]
fn every_answer_has_its_words_and_his_reply() {
    let w = worldgen::generate(1);
    for q in 0..4u8 {
        for a in 0..4u8 {
            assert_ne!(w.answer_text(q, a), "...", "q{} a{}", q + 1, a + 1);
            let tags = [format!("q={}", q + 1), format!("a={}", a + 1)];
            assert!(talk::pieces().iter().any(|p| p.topic == "diub" && p.slot == "reply" && tags.iter().all(|t| p.must.iter().any(|m| m == t))), "a reply to q{} a{}", q + 1, a + 1);
        }
    }
    assert!(w.diub_line("offer").is_some());
    assert!(w.diub_line("night_market").is_some());
    // No background is ever named: none of his lines or the answers says one.
    for p in talk::pieces().iter().filter(|p| p.topic == "diub") {
        for word in ["background", "Background", "class", "origin"] {
            assert!(!p.text.contains(word), "{}", p.text);
        }
    }
}

#[test]
fn four_answers_tilt_who_you_were_and_the_lesson_is_done() {
    let (mut w, me, diub) = at_the_hut(1);
    let before: Vec<f32> = [Skill::Blade, Skill::Block, Skill::Alchemy, Skill::Sneak].iter().map(|&s| w.people[me as usize].stats.skill(s)).collect();
    talk_to(&mut w, me, diub);
    // He asks first, and the answers are the only things to say.
    let first = &w.talk.as_ref().unwrap().lines[0].1;
    assert!(first.contains("Washed up") && first.contains("ship"), "his own hello, then the first question: {first}");
    assert!(w.locked_topics().is_empty(), "nothing else to ask while he's asking");
    let topics = w.topics();
    assert_eq!(topics.iter().filter(|t| matches!(t, Topic::Answer(0, _))).count(), 5, "four answers and I'd rather not say");
    // Guarding the cargo; herbs; I'd rather not say; away.
    let said = w.ask(Topic::Answer(0, 1)).unwrap();
    assert!(said.contains("bottom of the sea") && said.contains("hands"), "his reply, then the next question: {said}");
    w.ask(Topic::Answer(1, 2));
    let shrug = w.ask(Topic::Answer(2, RATHER_NOT)).unwrap();
    assert!(shrug.contains("Nor would I"), "{shrug}");
    let last = w.ask(Topic::Answer(3, 3)).unwrap();
    assert!(last.contains("Binding"), "his offer after the fourth: {last}");
    let p = &w.people[me as usize];
    assert_eq!(p.stats.skill(Skill::Blade), before[0] + BOOST);
    assert_eq!(p.stats.skill(Skill::Block), before[1] + BOOST);
    assert_eq!(p.stats.skill(Skill::Alchemy), before[2] + BOOST);
    assert_eq!(p.stats.skill(Skill::Sneak), before[3] + BOOST, "away: Sneak and Dodge");
    assert_eq!(QUESTIONS[3][3], &[Skill::Sneak, Skill::Dodge]);
    assert!(w.tutorial().unwrap().is_done("questions"), "the lesson is done");
    // Asked out, he talks like anyone.
    assert!(!w.topics().iter().any(|t| matches!(t, Topic::Answer(..))));
    // His guide line for the next lesson is there.
    assert!(w.guide_line("night_run").is_some());
}
