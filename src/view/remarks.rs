//! The squad speaks (drawing only): when one of them notices something (a
//! camp coming into sight, a town they've never seen, the first hungry pang,
//! dusk out in the open, a fight over) they say a short line over their head,
//! picked from `data/lines/squad.txt` by `World::squad_remark`. This replaces
//! pop-ups and tips that explained things.
//!
//! Like the floating words, it works by comparing the world with what the
//! window saw before; it changes nothing in the world. It is rare on
//! purpose: one remark at a time for the whole squad, `GAP` frames at least
//! between them, the same kind not again for `AGAIN` frames.

use std::collections::{HashMap, HashSet};

use gahturiyu_sim::sim::{
    condition::{stage_of, HungerStage, EXHAUSTED},
    geo::V2,
    group::GroupId,
    person::PersonId,
    ruins::RuinKind,
    stealth::{daylight, SIGHT},
    world::World,
};

use super::app::Game;

/// Frames between any two remarks (about 20 s at 30 frames a second).
const GAP: u32 = 600;
/// Frames before the same kind of remark again (about five minutes).
const AGAIN: u32 = 9000;
/// How long a remark stays up, frames.
const SHOWN: u32 = 200;
/// Checked every this many frames.
const EVERY: u32 = 10;
/// Daylight below this, out of town, and it's getting dark.
const DUSK: f32 = 0.45;

#[derive(Default)]
pub struct Remarks {
    loads: u32,
    started: bool,
    next_any: u32,
    next_kind: HashMap<&'static str, u32>,
    count: u64,
    camps: HashSet<GroupId>,
    robbers: HashSet<GroupId>,
    ruins: HashSet<u32>,
    towns: HashSet<u16>,
    hungry: HashMap<PersonId, HungerStage>,
    tired: HashSet<PersonId>,
    heavy: HashSet<PersonId>,
    dark: bool,
    fighting: bool,
    lost: bool,
    down: HashSet<PersonId>,
}

/// What's noticed, and who noticed it.
struct Notice {
    about: &'static str,
    who: PersonId,
    values: Vec<(&'static str, String)>,
}

pub fn update(game: &mut Game) {
    let f = game.frame;
    if game.loads != game.remarks.loads {
        game.remarks = Remarks { loads: game.loads, ..Default::default() };
    }
    // Screenshots: `GAHT_REMARK=<about>` has the first member say one.
    if game.shot.is_some() {
        if f == 5 {
            if let (Ok(about), Some(&m)) = (std::env::var("GAHT_REMARK"), game.world.squad.members.first()) {
                let about: &'static str = Box::leak(about.into_boxed_str());
                now(game, m, about);
            }
        }
        return;
    }
    if f % EVERY != 0 || game.world.talk.is_some() {
        return;
    }
    let first = !game.remarks.started;
    let notice = look(&game.world, &mut game.remarks, f);
    game.remarks.started = true;
    // On the first look everything already in view counts as known.
    if first {
        return;
    }
    let Some(n) = notice else { return };
    let r = &mut game.remarks;
    r.count += 1;
    if let Some(line) = game.world.squad_remark(n.who, n.about, &n.values, r.count) {
        r.next_any = f + GAP;
        r.next_kind.insert(n.about, f + AGAIN);
        game.barks.retain(|b| b.0 != n.who);
        game.barks.push((n.who, gahturiyu_sim::sim::speech::plain(&line), f + SHOWN));
    }
}

/// Compare the world with what was seen before; note what's new, and return
/// the one thing worth saying now (if a remark is due).
fn look(w: &World, r: &mut Remarks, f: u32) -> Option<Notice> {
    let first = !r.started;
    let can = |r: &Remarks, about: &'static str| first || (f >= r.next_any && r.next_kind.get(about).is_none_or(|&t| f >= t));
    // Who can speak: awake, on their feet, not in a fight.
    let up: Vec<(PersonId, V2)> = w
        .squad
        .members
        .iter()
        .copied()
        .filter(|&m| !w.is_down(m) && !w.is_asleep(m) && w.fighter(m).is_none())
        .map(|m| (m, w.person_pos(m)))
        .collect();
    let nearest = |at: V2| up.iter().min_by(|a, b| a.1.dist(at).total_cmp(&b.1.dist(at))).copied();
    let mut out: Option<Notice> = None;
    let say = |out: &mut Option<Notice>, n: Notice| {
        if out.is_none() {
            *out = Some(n);
        }
    };

    // A fight just over.
    let fighting = w.squad_battle().is_some();
    if r.fighting && !fighting {
        let down = w.squad.members.iter().filter(|&&m| w.is_down(m)).count();
        r.lost = down == w.squad.members.len();
        let about = if down > 0 { "after_fight" } else { "won" };
        if !r.lost && can(r, about) {
            if let Some(&(m, _)) = up.first() {
                say(&mut out, Notice { about, who: m, values: Vec::new() });
            }
        }
    }
    r.fighting = fighting;
    // Coming round after losing.
    let down: HashSet<PersonId> = w.squad.members.iter().copied().filter(|&m| w.is_down(m)).collect();
    for &m in r.down.difference(&down) {
        if r.lost && up.iter().any(|u| u.0 == m) {
            r.lost = false;
            if can(r, "robbed") {
                say(&mut out, Notice { about: "robbed", who: m, values: Vec::new() });
            }
        }
    }
    r.down = down;

    // Camps coming into sight (by day further than by night).
    let reach = SIGHT * daylight(w.time).max(0.45);
    for c in &w.camps {
        let Some((m, at)) = nearest(c.pos) else { break };
        if at.dist(c.pos) > reach || w.is_warden(c.group) {
            continue;
        }
        let holds_ours = c.stash.and_then(|id| w.container(id)).is_some_and(|k| !k.ours.is_empty());
        if holds_ours && !r.robbers.contains(&c.group) {
            if can(r, "robbers") {
                r.robbers.insert(c.group);
                r.camps.insert(c.group);
                say(&mut out, Notice { about: "robbers", who: m, values: Vec::new() });
            }
        } else if !holds_ours && !r.camps.contains(&c.group) && can(r, "camp") {
            r.camps.insert(c.group);
            say(&mut out, Notice { about: "camp", who: m, values: Vec::new() });
        }
        if !holds_ours {
            r.robbers.remove(&c.group);
        }
    }
    // Ruins and lairs, once each.
    for ru in &w.ruins {
        let Some((m, at)) = nearest(ru.pos) else { break };
        if at.dist(ru.pos) > reach.max(60.0) || r.ruins.contains(&ru.id) {
            continue;
        }
        let about = match ru.kind {
            RuinKind::Ruin if w.ruin_held(ru.id) => "ruin",
            RuinKind::Lair(_) => "lair",
            _ => {
                r.ruins.insert(ru.id);
                continue;
            }
        };
        if can(r, about) {
            r.ruins.insert(ru.id);
            say(&mut out, Notice { about, who: m, values: Vec::new() });
        }
    }
    // A town none of them has been in.
    for s in &w.settlements {
        let Some((m, at)) = nearest(s.pos) else { break };
        if at.dist(s.pos) > s.radius() || r.towns.contains(&s.id) {
            continue;
        }
        if can(r, "new_town") {
            r.towns.insert(s.id);
            say(&mut out, Notice { about: "new_town", who: m, values: vec![("town", super::lexicon::town(w, s.id))] });
        }
    }
    // Their own bodies.
    for &(m, _) in &up {
        let stage = w.hunger_of(m).map(stage_of).unwrap_or(HungerStage::Fed);
        let was = r.hungry.get(&m).copied().unwrap_or(HungerStage::Fed);
        if stage > was {
            let about = if stage >= HungerStage::Starving { "starving" } else { "hungry" };
            if can(r, about) {
                r.hungry.insert(m, stage);
                say(&mut out, Notice { about, who: m, values: Vec::new() });
            }
        } else if stage < was {
            r.hungry.insert(m, stage);
        }
        let tired = w.tired_of(m).is_some_and(|t| t >= EXHAUSTED);
        if tired && !r.tired.contains(&m) {
            if can(r, "tired") {
                r.tired.insert(m);
                say(&mut out, Notice { about: "tired", who: m, values: Vec::new() });
            }
        } else if !tired {
            r.tired.remove(&m);
        }
        let heavy = w.load_of(m) > 1.0 && w.carrying(m).is_none();
        if heavy && !r.heavy.contains(&m) {
            if can(r, "heavy") {
                r.heavy.insert(m);
                say(&mut out, Notice { about: "heavy", who: m, values: Vec::new() });
            }
        } else if !heavy {
            r.heavy.remove(&m);
        }
    }
    // Night coming on, out in the open.
    let in_town = w.settlements.iter().any(|s| s.pos.dist(w.squad.pos) < s.radius());
    let dark = daylight(w.time) < DUSK;
    if dark && !r.dark && !in_town && !fighting {
        if can(r, "dark") {
            if let Some(&(m, _)) = up.first() {
                r.dark = true;
                say(&mut out, Notice { about: "dark", who: m, values: Vec::new() });
            }
        }
    } else if !dark {
        r.dark = false;
    }
    if first {
        r.dark = dark;
        return None;
    }
    out
}

/// A failed click, said by the member who was sent (`locked`, `blocked`).
pub fn now(game: &mut Game, who: PersonId, about: &'static str) {
    let f = game.frame;
    let r = &mut game.remarks;
    if r.next_kind.get(about).is_some_and(|&t| f < t) {
        return;
    }
    r.count += 1;
    if let Some(line) = game.world.squad_remark(who, about, &[], r.count) {
        // Answering a click is never held back for long.
        r.next_any = r.next_any.max(f + GAP / 3);
        r.next_kind.insert(about, f + 90);
        game.barks.retain(|b| b.0 != who);
        game.barks.push((who, gahturiyu_sim::sim::speech::plain(&line), f + SHOWN));
    }
}
