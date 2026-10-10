//! The text play tool, driven the way a play-tester drives it: one command a
//! call, the save loaded and written each time. These are the tool's own
//! faults from the bug hunt (odd input, wrong words), fixed.

use std::path::PathBuf;
use std::process::Command;

struct Game {
    save: PathBuf,
}

impl Game {
    fn new(name: &str, seed: u64) -> Game {
        let dir = std::env::temp_dir().join(format!("gaht-play-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let g = Game { save: dir.join("t.save") };
        let out = g.run(&["new", &seed.to_string()]);
        assert!(out.contains("A new game"), "{out}");
        g
    }

    /// Run one command; a crash fails the test with what was printed.
    fn run(&self, args: &[&str]) -> String {
        let out = Command::new(env!("CARGO_BIN_EXE_play")).arg(&self.save).args(args).output().expect("the play tool runs");
        let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        assert!(out.status.success() && !text.contains("panicked"), "`{}` crashed:\n{text}", args.join(" "));
        text
    }

    fn first_name(&self) -> String {
        let look = self.run(&["look"]);
        let line = look.lines().find(|l| l.contains("health")).expect("a squad line").trim_start_matches([' ', '*']);
        line.split_whitespace().next().unwrap().to_string()
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        if let Some(dir) = self.save.parent() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

/// RG-5 = BL-1 = NM-9: a person number that doesn't exist is refused.
#[test]
fn a_person_who_does_not_exist_is_refused_by_every_command() {
    let g = Game::new("ids", 3);
    let name = g.first_name();
    for cmd in [vec!["talk", "p999999"], vec!["loot", "p999999"], vec!["attack", "p999999"], vec!["carry", "p999999"], vec!["go", "p999999"], vec!["talk", "p4294967295"], vec!["cast", name.as_str(), "1", "p999999"], vec!["use", name.as_str(), "0", "p999999"], vec!["dose", name.as_str(), "p999999"]] {
        let out = g.run(&cmd);
        assert!(!out.contains("A conversation opens"), "{out}");
    }
    assert!(g.run(&["talk", "p999999"]).contains("Can't talk to them."));
}

/// NM-70: an id of the wrong kind is refused, not read as another thing's number.
#[test]
fn the_letter_of_an_id_counts() {
    let g = Game::new("letters", 3);
    assert!(g.run(&["pickup", "x6"]).contains("takes a g id"));
    assert!(g.run(&["pickup", "p6"]).contains("takes a g id"));
    assert!(g.run(&["gather", "p59"]).contains("takes a n id"));
    assert!(g.run(&["work", "h2"]).contains("takes a d id"));
    assert!(g.run(&["butcher", "g1"]).contains("takes a c id"));
    assert!(g.run(&["pickup", "17"]).contains("takes a g id"));
    assert!(g.run(&["enter", "0.2"]).contains("No such building."));
    assert!(g.run(&["hunt", "p3"]).contains("Can't hunt that."));
    assert!(g.run(&["search", "0.2.0"]).contains("Can't go through that"));
    // Nobody has gone anywhere.
    assert!(g.run(&["look"]).contains("Day 1, 06:0"));
}

/// NM-75 = BL-66: `wait` wants whole minutes, a week at most.
#[test]
fn wait_takes_minutes_and_nothing_else() {
    let g = Game::new("wait", 3);
    for bad in ["abc", "", "0", "-5", "1.5", "nan", "inf"] {
        let out = if bad.is_empty() { g.run(&["wait"]) } else { g.run(&["wait", bad]) };
        assert!(out.contains("Usage: wait M"), "wait {bad}: {out}");
    }
    assert!(g.run(&["wait", "99999999"]).contains("a week"));
    assert!(g.run(&["look"]).contains("Day 1, 06:00"), "no time passed");
    assert!(g.run(&["wait", "5"]).contains("(5 minutes pass.)"));
}

/// NM-17 = BL-65: the word beside the clock follows the light.
#[test]
fn the_clock_s_word_follows_the_light() {
    let g = Game::new("light", 3);
    assert!(g.run(&["look"]).contains("06:00 (dawn)"));
    assert!(g.run(&["wait", "60"]).contains("07:00 (day)"));
    let dusk = g.run(&["wait", "690"]);
    assert!(dusk.contains("18:30 (day)"), "light until about seven: {}", dusk.lines().find(|l| l.starts_with("==")).unwrap_or(""));
    let later = g.run(&["wait", "90"]);
    assert!(later.contains("20:00 (dusk)"), "{}", later.lines().find(|l| l.starts_with("==")).unwrap_or(""));
    let night = g.run(&["wait", "60"]);
    assert!(night.contains("21:00 (night)"), "{}", night.lines().find(|l| l.starts_with("==")).unwrap_or(""));
}

/// BL-64: `putdown` and `torch` say what happened.
#[test]
fn putdown_and_torch_say_what_happened() {
    let g = Game::new("torch", 3);
    assert!(g.run(&["putdown"]).contains("Nobody selected is carrying anyone."));
    let lit = g.run(&["torch"]);
    assert!(lit.contains("lit.") || lit.contains("Nobody selected has a torch"), "{lit}");
    if lit.contains("lit.") {
        assert!(g.run(&["torch"]).contains("put out."));
    }
    // BL-61: no "-0" in the load.
    assert!(!g.run(&["look"]).contains("load -0"));
}

// ---- The second round: arguments, who is ordered, talk, news -------------------------

use gahturiyu_sim::sim::{geo::V2, person::PersonId, world::DAY, World};

impl Game {
    /// Change the saved world directly (what no command can set up).
    fn edit(&self, f: impl FnOnce(&mut World)) {
        let mut w = World::load_from(&self.save).expect("the save loads");
        f(&mut w);
        w.save_to(&self.save).expect("the save is written");
    }

    fn world(&self) -> World {
        World::load_from(&self.save).expect("the save loads")
    }
}

fn first_name_of(w: &World, m: PersonId) -> String {
    w.people[m as usize].name().unwrap().split_whitespace().next().unwrap().to_string()
}

/// NM-69: a missing or zero argument gets a usage line; it doesn't quietly
/// act on the first member or the first line.
#[test]
fn a_missing_argument_is_a_usage_line() {
    let g = Game::new("args", 3);
    let name = g.first_name();
    let before = g.run(&["pack", &name]);
    assert!(g.run(&["dose"]).contains("Usage: dose"));
    assert!(g.run(&["dose", &name]).contains("Usage: dose"));
    assert!(g.run(&["unequip"]).contains("Usage: unequip"));
    assert!(g.run(&["unequip", &name]).contains("Usage: unequip"));
    assert!(g.run(&["give", &name, "0"]).contains("Usage: give"));
    assert!(g.run(&["cast", &name, "0"]).contains("Usage: cast"));
    assert!(g.run(&["pack", "Nobody"]).contains("Nobody in the squad is called Nobody."));
    assert!(g.run(&["spells", "Nobody"]).contains("Nobody in the squad is called Nobody."));
    assert_eq!(g.run(&["pack", &name]), before, "nothing was taken off, handed over or used up");
    // With a chest open, `take` and `take 0` take nothing.
    let mut chest = None;
    g.edit(|w| {
        let me = w.squad.members[0];
        'find: for town in 0..w.settlements.len() as u16 {
            for b in 0..w.settlements[town as usize].buildings.len() as u16 {
                let Some(d) = w.door((town, b)) else { continue };
                if w.is_locked(d.id) {
                    continue;
                }
                w.teleport_squad(d.centre);
                w.order_members(&[me], d.centre);
                for _ in 0..40 {
                    w.step(0.25);
                }
                if let Some(c) = w.containers_in(d.id).find(|c| c.lock == 0.0 && !c.items.is_empty()).map(|c| c.id) {
                    chest = Some(c);
                    break 'find;
                }
            }
        }
    });
    let c = chest.expect("an open chest with something in it");
    let open = g.run(&["search", &format!("k{}.{}.{}", c.0, c.1, c.2)]);
    assert!(open.contains("1."), "the chest is open: {open}");
    let had = g.world().container(c).unwrap().items.len();
    for bad in [vec!["take"], vec!["take", "0"], vec!["take", "-1"], vec!["take", "x"]] {
        let out = g.run(&bad);
        assert!(out.contains("Usage: take N"), "{bad:?}: {out}");
    }
    assert_eq!(g.world().container(c).unwrap().items.len(), had, "nothing was taken");
}

/// NM-16, NM-74: a direction is measured from whoever is ordered, and the
/// tool looks from them.
#[test]
fn a_selected_member_is_where_the_tool_looks_from() {
    let g = Game::new("focus", 3);
    let w = g.world();
    let m = w.squad.members[1];
    let name = first_name_of(&w, m);
    let start = w.person_pos(m);
    let map_all = g.run(&["map"]);
    g.run(&["select", &name]);
    g.run(&["go", "e", "300"]);
    let east = g.world().person_pos(m);
    assert!(east.x - start.x > 200.0, "they walked east: {:.0} m", east.x - start.x);
    g.run(&["go", "n", "60"]);
    let north = g.world().person_pos(m);
    assert!((north.x - east.x).abs() < 25.0 && (east.y - north.y - 60.0).abs() < 25.0, "60 m north of where they stood, not of the squad: moved {:.0} east, {:.0} north", north.x - east.x, east.y - north.y);
    // The map's distances are theirs now, and the squad's again with everyone selected.
    let map_one = g.run(&["map"]);
    assert_ne!(map_one, map_all, "distances are from the selected member");
    g.run(&["select", "all"]);
    let w = g.world();
    let nearest = w.settlements.iter().min_by(|x, y| x.pos.dist(w.squad.pos).total_cmp(&y.pos.dist(w.squad.pos))).unwrap();
    let first_town = g.run(&["map"]).lines().find(|l| l.trim_start().starts_with('t')).unwrap_or("").to_string();
    assert!(first_town.contains(&nearest.name), "with everyone selected, the nearest town is the squad's: {first_town}");
}

/// NM-72: an order to someone who can't act is refused at once, with why.
#[test]
fn the_bound_and_the_downed_are_refused_at_once() {
    let g = Game::new("unable", 3);
    let (mut bound, mut down, mut place) = (String::new(), String::new(), String::new());
    g.edit(|w| {
        let (a, b) = (w.squad.members[0], w.squad.members[1]);
        let town = w.settlements.iter().min_by(|x, y| x.pos.dist(w.squad.pos).total_cmp(&y.pos.dist(w.squad.pos))).unwrap().id;
        let t = w.time;
        w.bond(a, town, None, t, t + 5.0 * DAY);
        // Knocked out (torso to zero).
        let p = &mut w.people[b as usize];
        let max = p.stats.max_hp(gahturiyu_sim::sim::body::Part::Torso);
        p.wounds.lost = p.wounds.lost_at(t);
        p.wounds.lost[1] = max + 20.0;
        p.wounds.at = t;
        bound = first_name_of(w, a);
        down = first_name_of(w, b);
        place = w.settlements[town as usize].name.clone();
    });
    let clock = |g: &Game| g.run(&["look"]).lines().next().unwrap_or("").to_string();
    let before = clock(&g);
    g.run(&["select", &bound]);
    let look = g.run(&["look"]);
    assert!(look.contains(&format!("bound in {place}, 5 days left")), "{look}");
    for cmd in [vec!["go", "e", "20"], vec!["enter", "b0.0"], vec!["talk", "p1"], vec!["work", "d0"], vec!["pickup", "g0"], vec!["rest"], vec!["sneak"], vec!["torch"]] {
        let out = g.run(&cmd);
        assert!(out.starts_with(&format!("Nobody can: {bound} is bound in {place}.")), "{cmd:?}: {out}");
    }
    g.run(&["select", &down]);
    let out = g.run(&["go", "e", "20"]);
    assert!(out.starts_with(&format!("Nobody can: {down} is down.")), "{out}");
    let out = g.run(&["wake"]);
    assert!(out.contains(&format!("{down} is down, and can't be woken.")) && !out.contains("Up."), "{out}");
    assert_eq!(clock(&g), before, "none of that took any time");
    // The whole squad ordered: the others go, and nobody waits half an hour for the two who can't.
    g.run(&["select", "all"]);
    let out = g.run(&["go", "e", "20"]);
    assert!(!out.contains("(30 minutes pass.)"), "{out}");
}

/// NM-73: the nearest of those ordered talks; someone far off "isn't here";
/// talking again is as quick as the first time; a second talk says the
/// first was left.
#[test]
fn talk_sends_the_nearest_and_is_quick_the_second_time() {
    let g = Game::new("talk", 3);
    // Mid-morning, with people about.
    g.run(&["wait", "240"]);
    let mut ids: Vec<(PersonId, PersonId)> = Vec::new();
    let mut far = 0;
    g.edit(|w| {
        // Two townspeople up and out of doors near each other in the nearest town; the squad beside the first.
        let town = w.settlements.iter().min_by(|x, y| x.pos.dist(w.squad.pos).total_cmp(&y.pos.dist(w.squad.pos))).unwrap().id;
        let up: Vec<PersonId> = w.residents_in_band1(town).into_iter().filter(|&p| !w.is_indoors_asleep(p) && w.building_at(w.person_pos(p)).is_none() && !w.people[p as usize].dead).collect();
        let (a, b) = up
            .iter()
            .flat_map(|&a| up.iter().map(move |&b| (a, b)))
            .filter(|&(a, b)| a != b)
            .min_by(|x, y| w.person_pos(x.0).dist(w.person_pos(x.1)).total_cmp(&w.person_pos(y.0).dist(w.person_pos(y.1))))
            .expect("two people out of doors");
        assert!(w.person_pos(a).dist(w.person_pos(b)) < 100.0, "the nearest two are {:.0} m apart", w.person_pos(a).dist(w.person_pos(b)));
        let at = w.person_pos(a);
        w.teleport_squad(at.add(V2::new(3.0, 0.0)));
        // Member 0 (the first to be picked, before) is far off; member 1 is close.
        w.squad.at[0] = at.add(V2::new(120.0, 0.0));
        w.squad.goal[0] = w.squad.at[0];
        ids.push((a, b));
        // Someone in a town far away.
        let other = w.settlements.iter().max_by(|x, y| x.pos.dist(at).total_cmp(&y.pos.dist(at))).unwrap().id;
        far = w.settlements[other as usize].residents.iter().copied().find(|&p| !w.is_indoors_asleep(p) && !w.people[p as usize].dead).expect("someone awake there");
    });
    let (a, b) = ids[0];
    let lone = g.world().squad.at[0];
    assert!(g.run(&["talk", &format!("p{far}")]).contains("They aren't here."));
    let first = g.run(&["talk", &format!("p{a}")]);
    assert!(first.contains("TALKING with"), "{first}");
    assert!(g.world().squad.at[0].dist(lone) < 1.0, "the far member stayed put; the near one talked");
    // Again, with the talk still open, and again after leaving: no ten minutes.
    let again = g.run(&["talk", &format!("p{a}")]);
    assert!(again.contains("TALKING with") && !again.contains("minutes pass"), "{again}");
    g.run(&["bye"]);
    let third = g.run(&["talk", &format!("p{a}")]);
    assert!(third.contains("TALKING with") && !third.contains("(10 minutes pass.)"), "{third}");
    // On to someone else: the first is left, and it says so.
    let other = g.run(&["talk", &format!("p{b}")]);
    assert!(other.contains("You take your leave of"), "{other}");
}

/// NM-76: a news line shows once, whatever moment it is stamped with.
#[test]
fn a_news_line_shows_once() {
    let g = Game::new("news", 3);
    g.run(&["look"]);
    g.edit(|w| {
        let t = w.time;
        // Written first but stamped later (as a line about a moment still to come is), then one stamped now.
        w.log.push_front((t + 600.0, "A line stamped ten minutes on.".into()));
        w.log.push_front((t, "A line stamped now.".into()));
    });
    let look = g.run(&["look"]);
    assert!(look.contains("A line stamped ten minutes on.") && look.contains("A line stamped now."), "{look}");
    for _ in 0..3 {
        let again = g.run(&["look"]);
        assert!(!again.contains("A line stamped"), "shown twice: {again}");
    }
    // A later line stamped in the same minute as the last one read still shows.
    g.edit(|w| {
        let t = w.time;
        w.log.push_front((t, "Another, in the same minute.".into()));
    });
    assert!(g.run(&["look"]).contains("Another, in the same minute."));
}
