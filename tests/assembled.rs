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
        assert!(["greeting", "topic", "feeling", "hook", "farewell", "bark", "origin", "work", "about", "folk", "ways", "line"].contains(&x.slot), "{x:?}");
    }
    // The theft set is there.
    assert!(p.iter().any(|x| x.topic == "theft" && x.text.contains("took {item}")));
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
    // The one robbed has nothing to sell about it (NM-57)...
    assert!(!w.options(npc, lead, &c.concerns, 0).contains(&Opt::Bribe));
    // ...but what someone else knows of a hidden theft can be bought, if
    // you've the coin.
    let other = w.people[npc as usize].home.map(|t| w.settlements[t as usize].residents[12]).unwrap();
    let coin = gahturiyu_sim::sim::items::id("coin");
    let has = w.squad_count(coin) >= 10;
    assert_eq!(w.options(other, lead, &c.concerns, 0).contains(&Opt::Bribe), has);
    // Threats need someone who looks able to carry them out.
    w.people[lead as usize].might = 10.0;
    assert!(!w.options(other, lead, &c.concerns, 0).contains(&Opt::Threaten));
    w.people[lead as usize].might = 90.0;
    assert!(w.options(other, lead, &c.concerns, 0).contains(&Opt::Threaten));
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

/// Every word asked for in the lines (`{w:root}`) is a real root, and a
/// native word comes through marked, with its meaning.
#[test]
fn native_words_in_the_lines_are_real() {
    use gahturiyu_sim::names::{self, Tongue};
    use gahturiyu_sim::sim::speech;
    for x in talk::pieces() {
        let mut rest = x.text;
        while let Some(a) = rest.find("{w:").or_else(|| rest.find("{W:")) {
            let b = rest[a..].find('}').unwrap();
            let root = &rest[a + 3..a + b];
            assert!(names::root(root).is_some(), "no root `{root}` in {x:?}");
            rest = &rest[a + b + 1..];
        }
    }
    let w = speech::native_word("friend", Tongue::Roduro, true);
    assert_eq!(speech::plain(&format!("{w}, come in.")), "Oqe (friend), come in.");
    assert_eq!(speech::bare(&format!("{w}, come in.")), "Oqe, come in.");
    let s = speech::spans(&format!("Hello, {w}."));
    assert_eq!(s.len(), 3);
    assert_eq!(s[1], ("Oqe".to_string(), Some("friend".to_string())));
}

/// NM-56: telling the watch is for the town it happened in; thanks come
/// from the one wronged, and are said as "you".
#[test]
fn thanks_for_telling_the_watch_come_from_the_wronged_and_say_you() {
    use gahturiyu_sim::sim::talk::Concern;
    let setup = || {
        let mut w = worldgen::generate(1);
        let here = w.squad.pos;
        let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().id;
        let res = w.settlements[town as usize].residents.clone();
        let (victim, thief) = (res[4], res[9]);
        let house = |w: &World, p: PersonId| w.society.lives[p as usize].household;
        let bystander = res.iter().copied().find(|&p| p != victim && p != thief && !w.people[p as usize].dead && house(&w, p) != house(&w, victim) && house(&w, p) != house(&w, thief)).unwrap();
        let t = w.time - 3600.0;
        let ev = w.note(Deed::Theft, Some(thief), Some(victim), town, t, false);
        (w, town, victim, bystander, ev)
    };
    let about = |ev: u32| Concern { subject: Subject::Theft, score: 1.0, event: Some(ev), opp: None, about: None };
    let thanks = |w: &World, npc: PersonId, lead: PersonId| w.on_mind(npc).into_iter().find(|c| c.subject == Subject::Kindness && c.about == Some(lead));

    // Someone who only heard of it: can send you to the watch, but owes you nothing.
    let (mut w, town, _, bystander, ev) = setup();
    let lead = w.squad.members[0];
    assert!(w.options(bystander, lead, &[about(ev)], 0).contains(&Opt::Report));
    // (A theft in another town isn't this town's watch's to hear.)
    let other = (town + 1) % w.settlements.len() as u16;
    let o = w.settlements[other as usize].residents.clone();
    let t = w.time - 3600.0;
    let there = w.note(Deed::Theft, Some(o[3]), Some(o[5]), other, t, false);
    assert!(!w.options(bystander, lead, &[about(there)], 0).contains(&Opt::Report), "a theft elsewhere isn't this watch's");
    assert_eq!(w.say_opt(bystander, lead, &[about(ev)], 0, Opt::Report), "Good. Let them answer for it.");
    assert!(thanks(&w, bystander, lead).is_none(), "no thanks from someone it never touched");

    // The one robbed: remembers it, and says so to your face as "you".
    let (mut w, _, victim, _, ev) = setup();
    let lead = w.squad.members[0];
    assert_eq!(w.say_opt(victim, lead, &[about(ev)], 0, Opt::Report), "Good. Let them answer for it.");
    let c = thanks(&w, victim, lead).expect("the one robbed remembers the help");
    let name = w.people[lead as usize].name().unwrap().to_string();
    let lines = vec![w.assemble_talk(victim, lead, Some(&c), false).text];
    assert!(lines.iter().all(|l| !l.contains(&name)), "named to their own face: {lines:?}");
    assert!(lines.iter().any(|l| l.contains("You ")), "{lines:?}");
}
