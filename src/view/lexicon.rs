//! Which names the window shows (Laz): the common English ones ("Mudstrand",
//! "Smith"), or things as the people around you name them, in their own
//! tongue. A setting (O, "Names"); drawing only. Whichever is shown, the
//! other is offered alongside where there's room (a tooltip's second line).
//!
//! "The people around you" are the founders of the nearest town; out in the
//! wilds, the lead squad member's own people.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use gahturiyu_sim::names::{self, PlaceName, Tongue};
use gahturiyu_sim::sim::{settlement::SettlementId, World};

static NATIVE: AtomicBool = AtomicBool::new(false);
/// The towns' whole names, worked out once per world (`Game::loads`).
static PLACES: Mutex<(u32, Vec<PlaceName>)> = Mutex::new((u32::MAX, Vec::new()));

pub fn set_native(on: bool) {
    NATIVE.store(on, Ordering::Relaxed);
}

pub fn native() -> bool {
    NATIVE.load(Ordering::Relaxed)
}

/// Work the towns' names out again when the world has changed.
pub fn refresh(w: &World, loads: u32) {
    let Ok(mut p) = PLACES.lock() else { return };
    if p.0 != loads || p.1.len() != w.settlements.len() {
        *p = (loads, gahturiyu_sim::sim::names::towns(w));
    }
}

fn place(id: SettlementId) -> Option<PlaceName> {
    PLACES.lock().ok()?.1.get(id as usize).cloned()
}

/// The tongue of the people round the squad.
pub fn tongue_here(w: &World) -> Tongue {
    let here = w.squad.pos;
    let near = w.settlements.iter().filter(|s| s.pos.dist(here) <= s.radius() + 300.0).min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here)));
    match near {
        Some(s) => s.founders.into(),
        None => w.squad.members.first().map(|&m| w.people[m as usize].race.into()).unwrap_or(Tongue::Roduro),
    }
}

/// A town's name, as set.
pub fn town(w: &World, id: SettlementId) -> String {
    if native() {
        if let Some(p) = place(id) {
            return p.name().to_string();
        }
    }
    w.settlements[id as usize].name.clone()
}

/// The other name for a town, with what it means: for a tooltip.
pub fn town_other(w: &World, id: SettlementId) -> Option<String> {
    let p = place(id)?;
    if native() {
        Some(format!("{} in English (\u{201c}{}\u{201d})", w.settlements[id as usize].name, p.meaning))
    } else {
        Some(format!("Its own name: {} (\u{201c}{}\u{201d}), said {}", p.name(), p.meaning, p.say))
    }
}

/// A thing's name (a job, an item, a building, a creature), as set.
pub fn thing(w: &World, english: &str) -> String {
    if native() {
        if let Some(n) = names::name_in_speech(english, tongue_here(w)) {
            return capital(&n);
        }
    }
    english.to_string()
}

/// The other name for a thing: for a tooltip.
pub fn thing_other(w: &World, english: &str) -> Option<String> {
    let t = tongue_here(w);
    let n = names::name_in_speech(english, t)?;
    if native() {
        Some(format!("{english} in English"))
    } else {
        Some(format!("{} here: {}", t.name(), capital(&n)))
    }
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
