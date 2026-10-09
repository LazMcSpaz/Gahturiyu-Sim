//! Conversation assembled from pieces in data files.

use gahturiyu_sim::sim::{
    dialogue::Topic,
    geo::V2,
    history::Deed,
    memory::Who,
    person::PersonId,
    talk::{self, Opt, Subject},
    worldgen, World,
};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

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

/// A local who's just been robbed (with a friendly temper toward strangers).
fn robbed(w: &mut World) -> PersonId {
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().id;
    let npc = w.settlements[town as usize].residents[4];
    let thief = w.settlements[town as usize].residents[9];
    w.people[npc as usize].traits.sociability = 0.9;
    let t = w.time - 3600.0 * 10.0;
    w.note(Deed::Theft, Some(thief), Some(npc), town, t, true);
    let day = World::day_of(w.time) as i32;
    w.remember(npc, Who::Someone, Deed::Theft, -0.6, day);
    npc
}

#[test]
fn the_pieces_parse() {
    let p = talk::pieces();
    assert!(p.len() > 60);
    for x in p {
        assert!(["greeting", "topic", "feeling", "hook", "farewell", "bark"].contains(&x.slot), "{x:?}");
    }
    // The theft set is there.
    assert!(p.iter().any(|x| x.topic == "theft" && x.text.contains("lifted {item}")));
}

#[test]
fn a_robbed_person_talks_about_it() {
    let mut w = worldgen::generate(1);
    let npc = robbed(&mut w);
    assert_eq!(w.on_mind(npc)[0].subject, Subject::Theft);
    talk_to(&mut w, npc);
    let line = w.talk.as_ref().unwrap().lines[0].1.clone();
    assert!(line.contains("robbed") || line.contains("lifted") || line.contains("dark"), "{line}");
}

#[test]
fn talking_twice_running_doesnt_repeat_lines() {
    let mut w = worldgen::generate(1);
    let npc = robbed(&mut w);
    talk_to(&mut w, npc);
    let first = w.talk.as_ref().unwrap().pieces.clone();
    let line1 = w.talk.as_ref().unwrap().lines[0].1.clone();
    w.ask(Topic::Goodbye);
    talk_to(&mut w, npc);
    let second = w.talk.as_ref().unwrap().pieces.clone();
    let line2 = w.talk.as_ref().unwrap().lines[0].1.clone();
    assert!(!first.is_empty() && !second.is_empty());
    // Pieces with something to say are never the same twice running.
    let said = |id: &u16| !talk::pieces()[*id as usize].text.is_empty();
    assert!(first.iter().filter(|x| said(x)).all(|x| !second.contains(x)), "{line1} / {line2}");
    assert_ne!(line1, line2);
}

#[test]
fn a_speaker_who_distrusts_you_wont_talk() {
    let mut w = worldgen::generate(1);
    let npc = robbed(&mut w);
    let lead = w.squad.members[0];
    let day = World::day_of(w.time) as i32;
    w.remember(npc, Who::Person(lead), Deed::Threat, -1.5, day);
    talk_to(&mut w, npc);
    let c = w.talk.as_ref().unwrap();
    assert!(c.refused, "they refuse: {}", c.lines[0].1);
    assert_eq!(w.topics(), vec![Topic::Goodbye]);
}

#[test]
fn what_you_can_say_depends_on_who_you_are() {
    let mut w = worldgen::generate(1);
    let npc = robbed(&mut w);
    let lead = talk_to(&mut w, npc);
    let c = w.talk.as_ref().unwrap().clone();
    // A hidden theft: what they know can be bought, if you've the coin.
    let coin = gahturiyu_sim::sim::items::id("coin");
    let has = w.squad_count(coin) >= 10;
    assert_eq!(w.options(npc, lead, &c.concerns, 0).contains(&Opt::Bribe), has);
    // Threats need someone who looks able to carry them out.
    w.people[lead as usize].might = 10.0;
    assert!(!w.options(npc, lead, &c.concerns, 0).contains(&Opt::Threaten));
    w.people[lead as usize].might = 90.0;
    assert!(w.options(npc, lead, &c.concerns, 0).contains(&Opt::Threaten));
}

#[test]
fn the_same_world_says_the_same_things() {
    let say = || {
        let mut w = worldgen::generate(1);
        let npc = robbed(&mut w);
        talk_to(&mut w, npc);
        w.talk.as_ref().unwrap().lines[0].1.clone()
    };
    assert_eq!(say(), say());
}
