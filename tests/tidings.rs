//! News you can follow (playtest 3: "The rite fails in Dìruhoshi" was the
//! best hook of the evening, and there was no way to chase it). What
//! befalls a town is written into its history, so it travels the roads like
//! any other news; people who have heard say where it happened and how far
//! off that is; and the squad's own news no longer announces what it wasn't
//! there to see (Laz, 10 Oct: less telling, more finding out).

use gahturiyu_sim::sim::{
    culture::Rule,
    dialogue::Topic,
    history::{Deed, Known},
    person::PersonId,
    speech,
    world::{DAY, HOUR},
    worldgen, World,
};

/// A town where a bad omen today brings the high priestess down (some
/// have no priestesses; in some an arbiter rules the omen false).
fn failed_rite(w: &mut World, far_from_squad: bool) -> (u16, u32, PersonId) {
    let t = w.time;
    for town in 0..w.settlements.len() as u16 {
        if w.squad_is_at(town) == far_from_squad {
            continue;
        }
        let gov = w.government(town);
        let Some(c) = gov.chambers.iter().find(|c| c.rule == Rule::Priestesses) else { continue };
        let Some(&high) = c.holders.first() else { continue };
        let before = w.events(town).len();
        w.bad_omen(town, t);
        if w.government(town).fallen.contains(&high) {
            let e = w.events(town)[before..].iter().find(|e| e.deed == Deed::RiteFailed).expect("the failed rite is in the town's history");
            assert_eq!((e.victim, e.actor), (Some(high), None));
            return (town, e.id, high);
        }
    }
    panic!("no town's rite could be made to fail");
}

/// Stand squad member 0 by `npc` and open a talk.
fn talk_to(w: &mut World, npc: PersonId) {
    let me = w.squad.members[0];
    let at = w.person_pos(npc);
    for k in 0..w.squad.members.len() {
        w.squad.at[k] = at;
        w.squad.goal[k] = at;
        w.squad.route[k].clear();
    }
    w.squad.pos = at;
    assert!(w.order_talk(me, npc));
    for _ in 0..40 {
        if w.talk.is_some() {
            break;
        }
        w.step(0.25);
    }
    assert!(w.talk.is_some(), "the talk opened");
}

#[test]
fn a_far_town_s_troubles_are_not_announced_to_the_squad() {
    let mut w = worldgen::generate(21);
    for _ in 0..30 {
        w.step(HOUR);
    }
    let lines = w.log.len();
    let (town, _, _) = failed_rite(&mut w, true);
    let name = w.settlements[town as usize].name.clone();
    assert!(!w.squad_is_at(town));
    assert!(!w.log.iter().any(|l| l.1.contains("rite fails")), "nothing in the squad's news about {name}: {:?}", w.log);
    assert!(w.log.len() <= lines + 1);
}

#[test]
fn the_town_the_squad_is_in_says_so() {
    let mut w = worldgen::generate(21);
    for _ in 0..30 {
        w.step(HOUR);
    }
    // Bring the squad to each town in turn until one's rite fails under them.
    let t = w.time;
    let mut said = false;
    for town in 0..w.settlements.len() as u16 {
        let gov = w.government(town);
        let Some(c) = gov.chambers.iter().find(|c| c.rule == Rule::Priestesses) else { continue };
        let Some(&high) = c.holders.first() else { continue };
        w.teleport_squad(w.settlements[town as usize].pos);
        assert!(w.squad_is_at(town));
        w.bad_omen(town, t);
        if w.government(town).fallen.contains(&high) {
            let name = w.settlements[town as usize].name.clone();
            assert!(w.log.iter().any(|l| l.1.contains(&format!("The rite fails in {name}"))), "{:?}", w.log);
            said = true;
            break;
        }
    }
    assert!(said, "some town's rite failed with the squad there");
}

#[test]
fn someone_who_has_heard_says_where_it_happened_and_how_far_off() {
    let mut w = worldgen::generate(21);
    for _ in 0..30 {
        w.step(HOUR);
    }
    let (there, id, high) = failed_rite(&mut w, true);
    let day = World::day_of(w.time) as i32;
    // Someone at home in the town the squad is in hears of it (as a
    // traveller's tiding would tell them: `hear_tidings`).
    let here = (0..w.settlements.len() as u16).find(|&s| w.squad_is_at(s)).expect("the squad starts in a town");
    assert_ne!(here, there);
    let hearer = w.settlements[here as usize]
        .residents
        .iter()
        .copied()
        .find(|&p| !w.people[p as usize].in_squad && !w.people[p as usize].dead && w.people[p as usize].home == Some(here) && w.is_about(p, w.time) && !w.is_indoors_asleep(p))
        .expect("someone about in town");
    assert!(w.learn(hearer, Known { id }, day), "news to them");
    talk_to(&mut w, hearer);
    assert!(w.topics().contains(&Topic::Rumours), "{:?}", w.topics());
    w.ask(Topic::Rumours);
    let said = speech::plain(&w.talk.as_ref().unwrap().lines.last().unwrap().1);
    let (there_name, priestess) = (w.settlements[there as usize].name.clone(), w.name_of(high));
    assert!(said.contains(&there_name), "names the town: {said}");
    assert!(said.contains(&priestess), "and who fell: {said}");
    assert!(said.contains("walk"), "and how far: {said}");
    let from = w.settlements[here as usize].pos;
    let dir = gahturiyu_sim::sim::quests::compass(w.settlements[there as usize].pos.sub(from));
    assert!(said.contains(dir), "and which way ({dir}): {said}");
    assert!(!said.contains("someone") && !said.contains('{'), "no gaps in it: {said}");
}

#[test]
fn the_news_takes_the_road() {
    // No help from the test: the rite fails, travellers set out, and after
    // some days people in other towns know of it.
    let mut w = worldgen::generate(21);
    for _ in 0..30 {
        w.step(HOUR);
    }
    let (there, id, _) = failed_rite(&mut w, true);
    let knew: Vec<PersonId> = w.knowers(id);
    assert!(knew.iter().all(|&p| w.people[p as usize].home == Some(there)), "at first only that town knows");
    for _ in 0..(8.0 * DAY / HOUR) as usize {
        w.step(HOUR);
    }
    let elsewhere = w.knowers(id).into_iter().filter(|&p| w.people[p as usize].home != Some(there)).count();
    assert!(elsewhere > 0, "after eight days, word has reached another town");
}
