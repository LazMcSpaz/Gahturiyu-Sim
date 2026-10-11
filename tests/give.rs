//! Handing things between squad members (playtest 1: "no way to give an
//! item to a squadmate"), and recruits who bring their own bread.

use gahturiyu_sim::sim::{geo::V2, items, worldgen};

#[test]
fn a_squadmate_can_hand_things_over_when_near() {
    let mut w = worldgen::generate(1);
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    let bread = items::id("flatbread");
    w.people[a as usize].detail.as_mut().unwrap().gear.add(bread, 4);
    let k = w.people[a as usize].detail.as_ref().unwrap().gear.bag.iter().position(|e| e.0 == bread).unwrap();
    let had = w.count_of(b, "flatbread");
    // Too far apart: told why.
    let kb = w.squad.index(b).unwrap();
    let far = w.squad.at[kb].add(V2::new(40.0, 0.0));
    w.squad.at[kb] = far;
    w.squad.goal[kb] = far;
    let why = w.give_entry(a, k, b).unwrap_err();
    assert!(why.contains("m away"), "{why}");
    // Side by side: it's theirs.
    let ka = w.squad.index(a).unwrap();
    let near = w.squad.at[ka].add(V2::new(1.0, 0.0));
    w.squad.at[kb] = near;
    w.squad.goal[kb] = near;
    let stack = w.count_of(a, "flatbread");
    assert!(stack >= 4);
    w.give_entry(a, k, b).unwrap();
    assert_eq!(w.count_of(b, "flatbread"), had + stack, "the whole stack");
    assert_eq!(w.count_of(a, "flatbread"), 0);
}

#[test]
fn recruits_bring_their_own_bread() {
    let mut w = worldgen::generate(1);
    // Run to the first dawn so who's out of work is sorted out.
    while w.time < 30.0 * 3600.0 {
        w.step(600.0);
    }
    let me = w.squad.members[0];
    let town = (0..w.settlements.len()).min_by(|&a, &b| w.settlements[a].pos.dist(w.squad.pos).total_cmp(&w.settlements[b].pos.dist(w.squad.pos))).unwrap();
    let willing = w.settlements[town].residents.iter().copied().find(|&p| w.join_terms(p) == Some(0));
    let p = willing.expect("someone free to join near the start");
    w.recruit(p, me).unwrap();
    assert!(w.count_of(p, "flatbread") >= 3, "they came with bread");
}

#[test]
fn a_squadmate_walks_over_and_gives_the_downed_a_draught() {
    use gahturiyu_sim::sim::body::Part;
    let mut w = worldgen::generate(1);
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    let t = w.time;
    {
        let p = &mut w.people[b as usize];
        p.wounds.lost[Part::Torso as usize] = p.stats.max_hp(Part::Torso) + 2.0;
        p.wounds.at = t;
    }
    w.step(0.5);
    assert!(w.is_down(b));
    // Trying to use it themselves: refused, and nothing used up.
    let draught = items::id("healing_draught");
    w.people[b as usize].detail.as_mut().unwrap().gear.add(draught, 1);
    let had = w.count_of(b, "healing_draught");
    assert!(!w.use_item(b, draught));
    assert!(w.why_cant_use(b, draught).unwrap().contains("out cold"));
    assert_eq!(w.count_of(b, "healing_draught"), had, "not used up");
    // A squadmate 30 m off has one: they walk over and give it.
    w.people[a as usize].detail.as_mut().unwrap().gear.add(draught, 1);
    let ka = w.squad.index(a).unwrap();
    let off = w.squad.at[ka].add(V2::new(30.0, 0.0));
    w.squad.at[ka] = off;
    w.squad.goal[ka] = off;
    let before = w.count_of(a, "healing_draught");
    w.order_dose(a, b).unwrap();
    for _ in 0..2000 {
        if w.dosing.is_empty() {
            break;
        }
        w.step(0.25);
    }
    assert_eq!(w.count_of(a, "healing_draught"), before - 1, "given");
    assert!(w.log.iter().any(|l| l.1.contains("gives") && l.1.contains("healing draught")), "{:?}", w.log);
}

#[test]
fn a_squadmate_walks_over_to_hand_something_across() {
    // Playtest 2: three of five gives failed because the two had drifted
    // 47, 57 and 66 m apart. Now the giver walks over.
    let mut w = worldgen::generate(1);
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    let bread = items::id("flatbread");
    w.people[a as usize].detail.as_mut().unwrap().gear.add(bread, 4);
    let k = w.people[a as usize].detail.as_ref().unwrap().gear.bag.iter().position(|e| e.0 == bread).unwrap();
    let (had, stack) = (w.count_of(b, "flatbread"), w.count_of(a, "flatbread"));
    let kb = w.squad.index(b).unwrap();
    let far = w.squad.at[kb].add(V2::new(45.0, 0.0));
    w.squad.at[kb] = far;
    w.squad.goal[kb] = far;
    let said = w.order_give(a, k, b).unwrap();
    assert!(said.contains("walks over"), "{said}");
    assert_eq!(w.count_of(b, "flatbread"), had, "not yet");
    for _ in 0..4000 {
        if w.giving.is_empty() {
            break;
        }
        w.step(0.25);
    }
    assert!(w.giving.is_empty(), "they got there");
    assert_eq!(w.count_of(b, "flatbread"), had + stack, "the whole stack, on arrival");
    assert_eq!(w.count_of(a, "flatbread"), 0);
    assert!(w.person_pos(a).dist(w.person_pos(b)) < 10.0);
    // A new order for the giver calls a give off.
    w.people[a as usize].detail.as_mut().unwrap().gear.add(bread, 2);
    let k = w.people[a as usize].detail.as_ref().unwrap().gear.bag.iter().position(|e| e.0 == bread).unwrap();
    let ka = w.squad.index(a).unwrap();
    let off = w.squad.at[ka].add(V2::new(-50.0, 0.0));
    w.squad.at[ka] = off;
    w.squad.goal[ka] = off;
    w.order_give(a, k, b).unwrap();
    assert_eq!(w.giving.len(), 1);
    let here = w.person_pos(a);
    w.order_members(&[a], here);
    assert!(w.giving.is_empty(), "walking somewhere else instead");
}

#[test]
fn someone_who_has_strayed_is_flagged() {
    let mut w = worldgen::generate(1);
    assert!(w.squad.members.len() >= 3);
    for &m in &w.squad.members.clone() {
        assert_eq!(w.strayed(m), None, "the squad starts together");
    }
    let m = w.squad.members[1];
    let k = w.squad.index(m).unwrap();
    let off = w.squad.at[k].add(V2::new(60.0, 0.0));
    w.squad.at[k] = off;
    w.squad.goal[k] = off;
    let d = w.strayed(m).expect("60 m off is strayed");
    assert!(d > 50.0 && d < 70.0, "{d}");
    for &other in w.squad.members.iter().filter(|&&x| x != m) {
        assert_eq!(w.strayed(other), None, "the ones who stayed put haven't strayed");
    }
    // NM-13: one member a long way off is the stray, not everyone else.
    let far = w.squad.at[k].add(V2::new(400.0, 0.0));
    w.squad.at[k] = far;
    w.squad.goal[k] = far;
    let d = w.strayed(m).expect("460 m off is strayed");
    assert!(d > 440.0 && d < 470.0, "{d}");
    for &other in w.squad.members.iter().filter(|&&x| x != m) {
        assert_eq!(w.strayed(other), None, "one far member doesn't make strays of the rest");
    }
    // Two and two: the pair with the earliest member is "the others".
    if w.squad.members.len() == 4 {
        let j = w.squad.index(w.squad.members[3]).unwrap();
        w.squad.at[j] = far;
        w.squad.goal[j] = far;
        assert_eq!(w.strayed(w.squad.members[0]), None);
        assert!(w.strayed(w.squad.members[3]).is_some() && w.strayed(m).is_some());
    }
}

/// N1, item 4: a tent isn't "used". Asked to, the game says how it works
/// (it goes up when the squad beds down), and those who sleep near whoever
/// carries it sleep under it.
#[test]
fn n1_a_tent_needs_no_pitching() {
    use gahturiyu_sim::sim::condition::Shelter;
    let mut w = worldgen::generate(1);
    let tent = items::id("tent");
    let carrier = w.squad.members.iter().copied().find(|&m| w.count_of(m, "tent") > 0).expect("the hunter starts with a tent");
    let why = w.why_cant_use(carrier, tent).expect("a tent isn't used up");
    assert!(why.contains("beds down") && !why.contains("isn't something to use"), "{why}");
    // Out in the open, away from any roof, the squad beds down.
    let out = w.squad.pos.add(V2::new(600.0, 0.0));
    w.teleport_squad(out);
    let all = w.squad.members.clone();
    w.order_rest(&all);
    for _ in 0..60 {
        w.step(10.0);
    }
    for &m in &all {
        let c = w.people[m as usize].cond.as_ref().unwrap();
        assert!(w.is_asleep(m), "{} is asleep", w.name_of(m));
        assert_eq!(c.shelter, Shelter::Tent, "{} sleeps under the tent", w.name_of(m));
    }
}
