//! What the wet does to what the squad carries (the items design, 0.4):
//! water works in two stages, both on the clock (rule 8). First it soaks
//! through the outermost container (their pack, or their clothes if they
//! carry none): fast in a downpour with nothing to keep it out, slowly
//! through one that resists, never through a sealed one. Only then does it
//! reach what's inside, and that spoils over further time: the ink on papers
//! runs, then they're pulp. A roof stops the soaking and they dry slowly; a
//! fire beside them dries them fast; out of the rain they dry slower still.
//!
//! Each member's wetness is a value at a moment plus a rate. The rate is
//! worked out from the hour's rain at the top of each hour (`weather_at`,
//! read once per hour), and again whenever they go under a roof, come out,
//! or come by a fire or leave it; the moments a stage is crossed are solved
//! for. What's guarded (in a scroll case, an oilskin, a sealed strongbox) is
//! spared: `guarded`.

use serde::{Deserialize, Serialize};

use super::condition::Shelter;
use super::elements::is_paper;
use super::inventory::Gear;
use super::items::{container, item, Guards, ItemId, Kind, Proof, Slot};
use super::person::PersonId;
use super::torch::FlameAt;
use super::world::{World, HOUR};

/// One member's wetness: how far the wet has soaked through (0 dry .. 1
/// through) and how far what's inside has gone (0 .. 1 ruined), at `at`,
/// with the rates (per hour) since.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Soak {
    pub pid: PersonId,
    pub at: f64,
    pub through: f32,
    pub inside: f32,
    pub rate: f32,
    pub inside_rate: f32,
    /// Under a roof, and by a fire, when the rate was set.
    pub roofed: bool,
    pub fire: bool,
    /// Said the ink is running (once a soaking).
    #[serde(default)]
    pub warned: bool,
    /// How well their outer pack keeps water out when the rate was set
    /// (0 none, 1 resists, 2 sealed): a change of pack settles it.
    #[serde(default)]
    pub proof: u8,
    /// How fast their food goes off from `at`, days a day (`spoil.rs`).
    #[serde(default)]
    pub food_rate: f32,
}

/// Soaking through, per hour, in a downpour (rain 1): nothing in the way,
/// resisting, sealed. (Drizzle soaks by its share.)
pub const SOAK: [f32; 3] = [6.0, 0.8, 0.0];
/// Drying, per hour: out of the rain in the open, under a roof, by a fire.
pub const DRY_OPEN: f32 = 0.15;
pub const DRY_ROOF: f32 = 0.4;
pub const DRY_FIRE: f32 = 1.5;
/// Once soaked through, what's inside goes this far an hour (rain 1; a
/// soaked pack out of the rain still spoils at a third of it).
pub const SPOIL: f32 = 1.0;
pub const SPOIL_DRY: f32 = 0.33;
/// Inside this far, the ink runs; at 1, papers are pulp.
pub const INK_RUNS: f32 = 0.5;
/// A fire this near dries them, metres.
pub const FIRE_NEAR: f32 = 4.0;

fn proof_index(p: Proof) -> usize {
    match p {
        Proof::None => 0,
        Proof::Resists => 1,
        Proof::Sealed => 2,
    }
}

/// How well the outermost thing they carry keeps water out.
pub fn outer_proof(gear: &Gear) -> Proof {
    match gear.in_slot(Slot::Back).and_then(container) {
        Some(c) => c.water,
        None => Proof::None,
    }
}

/// Which of their things are guarded from the wet (or fire) by an inner
/// container they carry: each guards so many of its kind, papers first.
/// Returns how many of each item are guarded.
pub fn guarded(gear: &Gear, fire: bool) -> Vec<(ItemId, u16)> {
    let mut room: Vec<(Guards, u16)> = Vec::new();
    for e in &gear.bag {
        if let Some(c) = container(e.0) {
            let proof = if fire { c.fire } else { c.water };
            if let (Some((g, n)), Proof::Sealed) = (c.guards, proof) {
                room.push((g, n * e.1));
            }
        }
    }
    let mut out = Vec::new();
    // Papers first (that's what a wrap or a case is for), then food (it
    // keeps longer sealed), then the rest.
    let mut order: Vec<&super::inventory::Entry> = gear.bag.iter().collect();
    order.sort_by_key(|e| if is_paper(e.0) { 0 } else if super::spoil::is_food(e.0) { 1 } else { 2 });
    for e in order {
        let mut n = 0;
        for slot in room.iter_mut() {
            while slot.1 > 0 && n < e.1 && guards(slot.0, e.0) {
                slot.1 -= 1;
                n += 1;
            }
        }
        if n > 0 {
            out.push((e.0, n));
        }
    }
    out
}

/// The bottles (anything drunk from a bottle, brewed or not) in a pack that
/// nothing guards: a potion case, or a strongbox, takes as many as it holds,
/// the first in the pack first. Each one by itself, with its mix.
pub fn bottles_at_risk(gear: &Gear) -> Vec<(ItemId, Option<super::items::herbs::Mix>)> {
    let mut room: u16 = gear
        .bag
        .iter()
        .filter_map(|e| container(e.0).and_then(|c| c.guards).filter(|g| g.0 == Guards::Potions || g.0 == Guards::Small).map(|g| g.1 * e.1))
        .sum();
    let mut out = Vec::new();
    for e in gear.bag.iter().filter(|e| item(e.0).kind == Kind::Potion) {
        for _ in 0..e.1 {
            if room > 0 {
                room -= 1;
            } else {
                out.push((e.0, e.2.and_then(|p| p.mix)));
            }
        }
    }
    out
}

/// Does a guard of this kind take this?
pub fn guards(g: Guards, id: ItemId) -> bool {
    match g {
        Guards::Papers => is_paper(id),
        Guards::Potions => item(id).kind == Kind::Potion,
        Guards::Small => super::items::footprint(id) == (1, 1) || is_paper(id) || super::spoil::is_food(id),
    }
}

impl World {
    /// A member's wetness as it stands at `t`.
    pub fn soak_at(&self, pid: PersonId, t: f64) -> (f32, f32) {
        let Some(s) = self.soaks.iter().find(|s| s.pid == pid) else { return (0.0, 0.0) };
        let h = ((t - s.at).max(0.0) / HOUR) as f32;
        advance(s, h)
    }

    /// Settle every squad member's wetness up to now: at each top of the
    /// hour the rain is read again; a roof or a fire coming or going settles
    /// it there and then; a stage crossed in between is acted on at its
    /// moment.
    pub(super) fn update_soaking(&mut self) {
        let now = self.time;
        for pid in self.squad.members.clone() {
            if self.people[pid as usize].dead {
                continue;
            }
            if !self.soaks.iter().any(|s| s.pid == pid) {
                self.stamp_food(pid, now);
                let mut s = Soak { pid, at: now, through: 0.0, inside: 0.0, rate: 0.0, inside_rate: 0.0, roofed: false, fire: false, warned: false, proof: 0, food_rate: 1.0 };
                self.soak_rates(&mut s, now);
                self.soaks.push(s);
                let r = self.food_rate(pid, now);
                if let Some(x) = self.soaks.iter_mut().find(|x| x.pid == pid) {
                    x.food_rate = r;
                }
                continue;
            }
            // Food just come into their pack gets its freshness.
            self.stamp_food(pid, now);
            for _ in 0..64 {
                let k = self.soaks.iter().position(|s| s.pid == pid).unwrap();
                let s = self.soaks[k];
                let (roofed, fire) = self.cover(pid);
                // The next moment something changes: the top of the hour, or
                // a stage crossed.
                let hour = ((s.at / HOUR).floor() + 1.0) * HOUR;
                let mut next = hour;
                for (level, through) in [(1.0, true), (INK_RUNS, false), (1.0, false)] {
                    if let Some(h) = reaches(&s, level, through) {
                        let tt = s.at + h as f64 * HOUR;
                        if tt > s.at && tt < next {
                            next = tt;
                        }
                    }
                }
                let proof = self.people[pid as usize].detail.as_ref().map(|d| proof_index(outer_proof(&d.gear)) as u8).unwrap_or(0);
                // (Food turning stale or spoiled is a moment too.)
                if let Some(tt) = self.next_food_turn(pid, s.food_rate) {
                    if tt > s.at && tt < next {
                        next = tt;
                    }
                }
                let changed_cover = roofed != s.roofed || fire != s.fire || proof != s.proof;
                let t = if changed_cover { now.min(next) } else { next };
                if t > now {
                    break;
                }
                let (through, inside) = advance(&s, ((t - s.at) / HOUR) as f32);
                let mut s = Soak { at: t, through, inside, ..s };
                if s.inside >= INK_RUNS && !s.warned {
                    s.warned = true;
                    self.ink_runs(pid, t);
                }
                if s.inside >= 1.0 - 1e-4 {
                    self.pulp(pid, t);
                    s.inside = 0.0;
                    s.warned = false;
                }
                self.soak_rates(&mut s, t);
                let k = self.soaks.iter().position(|x| x.pid == pid).unwrap();
                self.soaks[k] = s;
                // Their food: settled at the old rate, then the new one.
                self.settle_food(pid, s.food_rate, t);
                let r = self.food_rate(pid, t);
                self.soaks[k].food_rate = r;
            }
        }
        self.soaks.retain(|s| self.squad.index(s.pid).is_some());
    }

    /// Under a roof (indoors, a tent, a bed), and by a fire.
    fn cover(&self, pid: PersonId) -> (bool, bool) {
        let roofed = !matches!(self.shelter_of(pid), Shelter::Open);
        let at = self.person_pos(pid);
        let fire = self.flames_near(at, FIRE_NEAR).iter().any(|f| f.2 && matches!(f.0, FlameAt::Fire(_)));
        (roofed, fire)
    }

    /// The rates from `t` on: the hour's rain (unless roofed), the cover.
    fn soak_rates(&self, s: &mut Soak, t: f64) {
        let (roofed, fire) = self.cover(s.pid);
        s.roofed = roofed;
        s.fire = fire;
        let at = self.person_pos(s.pid);
        let hour = (t / HOUR).floor() * HOUR;
        let w = self.weather_at(at, hour);
        let rain = if roofed || !self.rules.weather_on_squad { 0.0 } else { (w.rain + w.snow * 0.3).clamp(0.0, 1.0) };
        let proof = self.people[s.pid as usize].detail.as_ref().map(|d| outer_proof(&d.gear)).unwrap_or(Proof::None);
        s.proof = proof_index(proof) as u8;
        s.rate = if fire {
            -DRY_FIRE
        } else if rain > 0.02 {
            rain * SOAK[proof_index(proof)]
        } else if roofed {
            -DRY_ROOF
        } else {
            -DRY_OPEN
        };
        s.inside_rate = if s.through >= 1.0 - 1e-4 && s.rate >= 0.0 { if rain > 0.02 { rain * SPOIL } else { SPOIL * SPOIL_DRY } } else { 0.0 };
        if s.rate < 0.0 {
            s.inside_rate = 0.0;
        }
    }

    /// The ink runs on what papers they carry unguarded.
    fn ink_runs(&mut self, pid: PersonId, t: f64) {
        let Some(d) = self.people[pid as usize].detail.as_ref() else { return };
        let safe = guarded(&d.gear, false);
        let wet = d.gear.bag.iter().any(|e| is_paper(e.0) && safe.iter().find(|g| g.0 == e.0).map(|g| g.1).unwrap_or(0) < e.1);
        if wet {
            let name = self.name_of(pid);
            self.log.push_front((t, format!("The ink is running on {name}'s papers: get them dry.")));
            self.log.truncate(14);
        }
    }

    /// Soaked through and left: the papers they carry unguarded are pulp.
    fn pulp(&mut self, pid: PersonId, t: f64) {
        let Some(d) = self.people[pid as usize].detail.as_mut() else { return };
        let safe = guarded(&d.gear, false);
        let mut lost: Vec<ItemId> = Vec::new();
        for e in d.gear.bag.iter_mut().filter(|e| is_paper(e.0)) {
            let keep = safe.iter().find(|g| g.0 == e.0).map(|g| g.1).unwrap_or(0).min(e.1);
            if e.1 > keep {
                lost.push(e.0);
                e.1 = keep;
            }
        }
        d.gear.bag.retain(|e| e.1 > 0);
        if lost.is_empty() {
            return;
        }
        let name = d.name.clone();
        self.people[pid as usize].recompute_might();
        self.log.push_front((t, format!("{name}'s {} soaked to pulp.", lost.iter().map(|&i| item(i).name.to_lowercase()).collect::<Vec<_>>().join(", "))));
        self.log.truncate(14);
    }
}

/// Where a wetness stands `h` hours on.
fn advance(s: &Soak, h: f32) -> (f32, f32) {
    let through = (s.through + s.rate * h).clamp(0.0, 1.0);
    // (What's inside starts going only once it's soaked through: that's a
    // moment settled for, so between settles its rate holds.)
    let inside = (s.inside + s.inside_rate * h).min(1.0);
    (through, inside)
}

/// Hours until `through` (or what's inside) reaches `level`, if it's going
/// that way.
fn reaches(s: &Soak, level: f32, through: bool) -> Option<f32> {
    let (v, r) = if through { (s.through, s.rate) } else { (s.inside, s.inside_rate) };
    (r > 0.0 && v < level).then(|| (level - v) / r)
}
