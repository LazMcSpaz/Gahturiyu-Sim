//! Faction voices, private lines and guilds (docs/factions.md, F4-F5).

use gahturiyu_sim::sim::{
    dialogue::Topic,
    factions::{self, HOSTILE},
    geo::V2,
    jobs::Job,
    person::PersonId,
    talk::{pieces, CONFIDES, TRUSTS_YOU},
    worldgen, World,
};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

/// The squad's home town and its residents.
fn home(w: &World) -> (usize, Vec<PersonId>) {
    let here = w.squad.pos;
    let (k, town) = w.settlements.iter().enumerate().min_by(|a, b| a.1.pos.dist(here).total_cmp(&b.1.pos.dist(here))).unwrap();
    (k, town.residents.clone())
}

fn with_job(w: &World, people: &[PersonId], job: Job) -> Option<PersonId> {
    people.iter().copied().find(|&p| w.life(p).job == job)
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

/// Lines of a slot tagged with this faction.
fn faction_lines(slot: &str, key: &str) -> Vec<&'static str> {
    let tag = format!("faction={key}");
    pieces().iter().filter(|p| p.slot == slot && p.must.iter().any(|m| *m == tag)).map(|p| p.text).collect()
}

#[test]
fn a_persons_factions_come_from_their_trade_and_their_towns_holder() {
    let mut w = worldgen::generate(1);
    for s in &mut w.settlements {
        s.holder = None;
    }
    let (k, folk) = home(&w);
    let scribe = folk.iter().copied().find(|&p| matches!(w.life(p).job, Job::Scribe | Job::Teacher | Job::Exchanger)).expect("someone of the Writers' trades");
    let writers = factions::id("writers");
    assert_eq!(w.factions_of(scribe), vec![writers], "no holder yet: their trade's guild only");
    let farmer = folk.iter().copied().find(|&p| w.life(p).job == Job::Farmer).unwrap();
    assert!(w.factions_of(farmer).is_empty());
    // Held by the banner lords: everyone there is theirs, and the Great House's.
    w.settlements[k].holder = Some(factions::id("banner_lords"));
    assert_eq!(w.factions_of(farmer), vec![factions::id("banner_lords"), factions::id("great_house")]);
    assert_eq!(w.factions_of(scribe), vec![writers, factions::id("banner_lords"), factions::id("great_house")]);
}

#[test]
fn a_faction_greets_in_its_own_attitude() {
    let mut w = worldgen::generate(1);
    for s in &mut w.settlements {
        s.holder = None;
    }
    let (k, folk) = home(&w);
    let lead = w.squad.members[0];
    let scribe = folk.iter().copied().find(|&p| matches!(w.life(p).job, Job::Scribe | Job::Teacher | Job::Exchanger)).unwrap();
    let said = w.assemble_talk(scribe, lead, None, true).text;
    assert!(faction_lines("greeting", "writers").iter().any(|l| said.contains(&l[..20])), "a Writer's greeting: {said}");
    // A Free Lord's man, in a town the Free Lords hold.
    w.settlements[k].holder = Some(factions::id("free_lords"));
    let farmer = with_job(&w, &folk, Job::Farmer).unwrap();
    let said = w.assemble_talk(farmer, lead, None, true).text;
    assert!(faction_lines("greeting", "free_lords").iter().any(|l| said.contains(&l[..20])), "a Free Lord's greeting: {said}");
    let bye = w.farewell(farmer, lead).text;
    assert!(faction_lines("farewell", "great_house").iter().any(|l| bye.contains(&l[..20])), "their parent's farewell: {bye}");
}

#[test]
fn private_lines_only_to_someone_they_trust() {
    let mut w = worldgen::generate(2);
    let (k, folk) = home(&w);
    w.settlements[k].holder = Some(factions::id("tide_houses"));
    // Someone who doesn't open up to strangers at once.
    let lead0 = w.squad.members[0];
    let npc = folk.iter().copied().find(|&p| w.regard_of(p, lead0) < CONFIDES - 15.0 && w.life(p).job != Job::None).expect("someone reserved");
    let lead = talk_to(&mut w, npc);
    assert!(w.topics().contains(&Topic::Views), "what they make of things");
    assert!(!w.topics().contains(&Topic::Confide), "not yet");
    assert!(w.locked_topics().iter().any(|(t, why)| *t == Topic::Confide && why.contains("well enough")));
    let views = w.ask(Topic::Views).unwrap();
    assert!(faction_lines("line", "first_peoples").iter().chain(faction_lines("line", "tide_houses").iter()).any(|l| views.contains(&l[..15])), "{views}");
    // Good turns remembered: now they open up.
    w.regard.insert(npc, 60.0);
    while w.regard_of(npc, lead) < CONFIDES {
        *w.regard.get_mut(&npc).unwrap() += 10.0;
    }
    assert!(w.regard_of(npc, lead) >= CONFIDES);
    assert!(w.topics().contains(&Topic::Confide));
    let deep = w.ask(Topic::Confide).unwrap();
    assert!(faction_lines("private", "first_peoples").iter().chain(faction_lines("private", "tide_houses").iter()).any(|l| deep.contains(&l[..15])), "{deep}");
    let _ = TRUSTS_YOU;
}

#[test]
fn a_syndicate_shows_only_to_the_trusted() {
    let w = worldgen::generate(3);
    let lead = w.squad.members[0];
    let (_, folk) = home(&w);
    let crook = folk.iter().copied().find(|&p| w.syndicate_of(p).is_some()).expect("someone in a syndicate");
    let s = w.syndicate_of(crook).unwrap();
    assert!(matches!(factions::def(s).key, "old_hands" | "night_market"));
    assert!(w.guild_of(crook, false) != Some(s), "kept quiet from strangers");
    assert_eq!(w.guild_of(crook, true).filter(|&g| g == s), Some(s), "but not from friends");
    let stranger = w.assemble_talk(crook, lead, None, true).text;
    assert!(!faction_lines("greeting", factions::def(s).key).iter().any(|l| stranger.contains(*l)));
}

#[test]
fn joining_a_guild_in_talk() {
    let mut w = worldgen::generate(1);
    let (_, folk) = home(&w);
    let scribe = folk.iter().copied().find(|&p| matches!(w.life(p).job, Job::Scribe | Job::Teacher | Job::Exchanger)).unwrap();
    let writers = factions::id("writers");
    let before = w.standing_with(writers);
    talk_to(&mut w, scribe);
    assert!(w.topics().contains(&Topic::JoinGuild(writers)));
    let said = w.ask(Topic::JoinGuild(writers)).unwrap();
    assert!(w.in_guild(writers), "{said}");
    assert!(w.standing_with(writers) > before);
    assert!(!w.topics().contains(&Topic::JoinGuild(writers)), "not twice");
    // Their greetings know you now.
    let lead = w.squad.members[0];
    w.talk = None;
    let said = w.assemble_talk(scribe, lead, None, true).text;
    assert!(!said.is_empty());
}

#[test]
fn the_two_syndicates_are_rivals() {
    let mut w = worldgen::generate(4);
    let (old, night) = (factions::id("old_hands"), factions::id("night_market"));
    w.join_guild(night).unwrap();
    assert!(w.standing_with(old) <= HOSTILE, "the Old Hands are against you now ({})", w.standing_with(old));
    assert_eq!(w.can_join(old), Err("you're with the other lot"));
    assert!(w.leave_guild(night));
    assert!(w.can_join(old).is_err(), "and they won't have you");
}

#[test]
fn the_rooted_are_a_one_way_door() {
    let mut w = worldgen::generate(5);
    let (writers, rooted) = (factions::id("writers"), factions::id("rooted"));
    w.join_guild(writers).unwrap();
    w.join_guild(rooted).unwrap();
    assert_eq!(w.guilds, vec![rooted], "everything else is gone");
    assert!(w.can_join(writers).is_err());
    assert!(!w.leave_guild(rooted), "and there's no leaving");
    // Not a guild or the Rooted: nothing to join.
    let mut w = worldgen::generate(5);
    assert!(w.join_guild(factions::id("great_house")).is_err());
}
