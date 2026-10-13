//! The ferry between Salt Isle and the mainland (`docs/tutorial-island.md`:
//! the player leaves the island by ship, and the island stays on the map;
//! they can come back by ferry later).
//!
//! **The ship keeps a timetable** read off the clock, never stepped: it
//! lies at the island's dock through `ISLE_CALL` every day and at the
//! mainland port (`ferry_port`: the nearest coastal town's fishing dock)
//! through `PORT_CALL`; between, it's away. Where it's drawn is a pure
//! function of the time (`ship_pos`).
//!
//! **A voyage is the squad's own** (the player exception, like walking):
//! members ordered aboard walk to the dock and wait; when the ship is in and
//! everyone ordered is there, it sails with them, and until it arrives
//! their positions are read off the crossing (`do_boarding`: value at a
//! moment plus a rate, nothing integrated). Arrived, they stand on the far
//! dock; leaving the island ends the tutorial (`end_tutorial`).
//!
//! Whether the crew will take someone off the island is the tutorial's to
//! say (`can_board`): on the caught path they go with their buyer, on the
//! sneak path they stow away (both the second session's lessons; until
//! those are on, anyone may board).

use serde::{Deserialize, Serialize};

use super::condition::Activity;
use super::geo::V2;
use super::island::Spot;
use super::jobs::PlaceKind;
use super::person::PersonId;
use super::settlement::SettlementId;
use super::world::{World, HOUR};

/// The ship's pace, metres a second.
pub const SHIP_SPEED: f32 = 4.0;
/// Hours of the day the ship lies at the island's dock, and at the port.
pub const ISLE_CALL: (f64, f64) = (8.0, 11.0);
pub const PORT_CALL: (f64, f64) = (14.0, 17.0);
/// How near the dock someone must be to go aboard.
pub const BOARD_REACH: f32 = 8.0;
/// The ship lies this far off the dock's end.
pub const OFF_DOCK: f32 = 12.0;

/// Which dock.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Berth {
    Island,
    Port,
}

/// The squad's crossing, under way: along `path` (berth, round the
/// island's south, berth) at a steady pace.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Voyage {
    pub members: Vec<PersonId>,
    pub path: Vec<V2>,
    pub depart: f64,
    pub arrive: f64,
    pub to_berth: Berth,
}

impl Voyage {
    pub fn from(&self) -> V2 {
        self.path[0]
    }

    pub fn to(&self) -> V2 {
        self.path[self.path.len() - 1]
    }

    pub fn length(&self) -> f32 {
        self.path.windows(2).map(|w| w[0].dist(w[1])).sum()
    }

    /// Where the ship is at `t`, and the way it's heading.
    pub fn at(&self, t: f64) -> (V2, f32) {
        let f = ((t - self.depart) / (self.arrive - self.depart).max(1e-6)).clamp(0.0, 1.0) as f32;
        let mut left = f * self.length();
        for w in self.path.windows(2) {
            let d = w[1].sub(w[0]);
            let len = d.len();
            if left <= len || len <= 1e-3 {
                let k = if len <= 1e-3 { 0.0 } else { left / len };
                return (w[0].lerp(w[1], k), d.y.atan2(d.x));
            }
            left -= len;
        }
        let n = self.path.len();
        let d = self.path[n - 1].sub(self.path[n.saturating_sub(2)]);
        (self.path[n - 1], d.y.atan2(d.x))
    }
}

/// The ship's way round the island: it passes this far south of the
/// island's shore between its dock and the mainland.
pub const SEA_GATE_OFF: f32 = 140.0;

fn hour_of_day(t: f64) -> f64 {
    (t / HOUR).rem_euclid(24.0)
}

impl World {
    /// The mainland port: the nearest coastal town's fishing dock to the
    /// island, and where the ship lies off it.
    pub fn ferry_port(&self) -> Option<(SettlementId, V2)> {
        let isle = self.island()?;
        let mut best: Option<(SettlementId, V2, f32)> = None;
        for (t, tl) in self.society.towns.iter().enumerate() {
            if self.settlements[t].island {
                continue;
            }
            for wp in tl.places.iter().filter(|p| p.kind == PlaceKind::Dock) {
                let d = wp.pos.dist(isle.centre);
                if best.map(|b| d < b.2).unwrap_or(true) {
                    best = Some((t as SettlementId, wp.pos, d));
                }
            }
        }
        best.map(|(t, p, _)| (t, p))
    }

    /// Where the ship lies at a berth: off the dock, in the water
    /// (`OFF_DOCK` past where the land ends, seaward of the dock).
    pub fn berth_pos(&self, b: Berth) -> Option<V2> {
        let (dock, out) = match b {
            Berth::Island => {
                let isle = self.island()?;
                let dock = isle.spot(Spot::Dock);
                // Seaward: away from the island's centre.
                (dock, dock.sub(isle.centre))
            }
            // The mainland's docks run out toward the water (`atlas.rs`).
            Berth::Port => {
                let dock = self.ferry_port()?.1;
                let out = super::atlas::atlas().toward_sea(dock);
                (dock, if out.len() < 0.5 { V2::new(-1.0, 0.0) } else { out })
            }
        };
        let len = out.len().max(1e-3);
        let dir = out.scale(1.0 / len);
        let mut d = 0.0;
        while d < 120.0 && !self.terrain.is_sea(dock.add(dir.scale(d))) {
            d += 2.0;
        }
        Some(dock.add(dir.scale(d + OFF_DOCK)))
    }

    /// Where someone waits to board: the dock itself.
    pub fn dock_pos(&self, b: Berth) -> Option<V2> {
        match b {
            Berth::Island => Some(self.island()?.spot(Spot::Dock)),
            Berth::Port => self.ferry_port().map(|p| p.1),
        }
    }

    /// Is the ship lying at this berth at `t` (its timetable)?
    pub fn ship_in(&self, b: Berth, t: f64) -> bool {
        if self.voyage.is_some() {
            return false;
        }
        let h = hour_of_day(t);
        let (a, z) = match b {
            Berth::Island => ISLE_CALL,
            Berth::Port => PORT_CALL,
        };
        h >= a && h < z
    }

    /// When the ship next lies at this berth, from `t`.
    pub fn ship_next(&self, b: Berth, t: f64) -> f64 {
        let (a, _) = match b {
            Berth::Island => ISLE_CALL,
            Berth::Port => PORT_CALL,
        };
        let day = (t / HOUR / 24.0).floor();
        let today = (day * 24.0 + a) * HOUR;
        if today > t {
            today
        } else {
            today + 24.0 * HOUR
        }
    }

    /// The way between the berths: out from the island's dock, round the
    /// island's south side, and across to the port.
    fn sea_way(&self, from: Berth) -> Option<Vec<V2>> {
        let isle = self.island()?;
        let a = self.berth_pos(Berth::Island)?;
        let z = self.berth_pos(Berth::Port)?;
        let gate = isle.centre.add(V2::new(0.0, super::geo::ISLE_B + SEA_GATE_OFF));
        // From the gate, the way through open water to the port (`atlas.rs`:
        // the atlas doesn't know the island, so it's closed off by hand).
        let atlas = super::atlas::atlas();
        let course = atlas.sea_path(gate, z, super::atlas::PX * 0.9, Some((isle.centre, super::geo::ISLE_A + SEA_GATE_OFF * 0.5)));
        let mut path = vec![a, gate];
        path.extend(course);
        path.push(z);
        path.dedup_by(|p, q| p.dist(*q) < 1.0);
        if from == Berth::Port {
            path.reverse();
        }
        Some(path)
    }

    /// Where the ship is drawn at `t`, and the way it faces: at a berth, on
    /// the squad's crossing, or (away on its own) nowhere.
    pub fn ship_pos(&self, t: f64) -> Option<(V2, f32)> {
        if let Some(v) = &self.voyage {
            return Some(v.at(t));
        }
        for b in [Berth::Island, Berth::Port] {
            if self.ship_in(b, t) {
                let p = self.berth_pos(b)?;
                let dock = self.dock_pos(b)?;
                let d = p.sub(dock);
                return Some((p, d.y.atan2(d.x) + std::f32::consts::FRAC_PI_2));
            }
        }
        None
    }

    /// The berth nearest the squad.
    pub fn nearest_berth(&self) -> Option<Berth> {
        let here = self.squad.pos;
        let i = self.dock_pos(Berth::Island)?;
        let p = self.dock_pos(Berth::Port)?;
        Some(if here.dist(i) <= here.dist(p) { Berth::Island } else { Berth::Port })
    }

    /// Will the crew take this member off the island? The tutorial's
    /// endings say (`tutorial.rs`): on the caught path, once they're bought
    /// out (when that lesson is built); on the sneak path they stow away,
    /// which means sneaking aboard; with no path chosen yet, anyone.
    pub fn can_board(&self, who: PersonId, from: Berth) -> Result<(), &'static str> {
        if from != Berth::Island {
            return Ok(());
        }
        let Some(tu) = self.tutorial().filter(|t| !t.over) else { return Ok(()) };
        match tu.path {
            Some(super::tutorial::Path::Caught) => {
                let gated = super::tutorial::lesson("bought").is_some_and(|x| x.on);
                if gated && !tu.is_done("bought") {
                    return Err("the crew won't take a debtor");
                }
            }
            Some(super::tutorial::Path::Sneak) => {
                let sneaking = self.squad.index(who).is_some_and(|k| self.squad.sneaking[k]);
                if !sneaking {
                    return Err("a runaway has to sneak aboard");
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Send some of the squad to board the ship at the nearest dock. They
    /// walk there and wait for it; it sails when all of them are aboard.
    pub fn order_board(&mut self, who: &[PersonId]) -> Result<(), &'static str> {
        if self.voyage.is_some() {
            return Err("they're at sea");
        }
        let b = self.nearest_berth().ok_or("there's no ship here")?;
        let dock = self.dock_pos(b).ok_or("there's no ship here")?;
        let mut going = Vec::new();
        for &m in who {
            if self.squad.index(m).is_none() || !self.free_to_order(m) || self.fighting.contains_key(&m) {
                continue;
            }
            self.can_board(m, b)?;
            going.push(m);
        }
        if going.is_empty() {
            return Err("nobody can go");
        }
        self.order_members(&going, dock);
        self.boarding.retain(|m| !going.contains(m));
        self.boarding.extend(going);
        Ok(())
    }

    /// The squad's step: those waiting at the dock go aboard when the ship
    /// is in, and a crossing under way carries its passengers.
    pub(super) fn do_boarding(&mut self) {
        let t = self.time;
        if let Some(v) = self.voyage.clone() {
            if t >= v.arrive {
                // Put in: everyone stands on the dock.
                let dock = self.dock_pos(v.to_berth).unwrap_or(v.to());
                // (Landward of the dock: the water is a step away.)
                let out = super::atlas::atlas().toward_sea(dock);
                let land = if out.len() < 0.5 { V2::new(0.0, 0.0) } else { out.scale(-3.0) };
                for (n, &m) in v.members.iter().enumerate() {
                    let Some(k) = self.squad.index(m) else { continue };
                    let a = 0.7 + n as f32 * 1.9;
                    let at = self.free_spot(dock.add(land).add(V2::new(a.cos(), a.sin()).scale(1.5)));
                    self.squad.at[k] = at;
                    self.squad.goal[k] = at;
                    self.squad.route[k].clear();
                }
                let where_ = match v.to_berth {
                    Berth::Island => super::island::NAME.to_string(),
                    Berth::Port => self.ferry_port().map(|(t, _)| self.settlements[t as usize].name.clone()).unwrap_or_default(),
                };
                self.say(t, format!("The ship puts in at {where_}."));
                if v.to_berth == Berth::Port {
                    self.end_tutorial();
                }
                self.voyage = None;
                self.recentre_squad();
            } else {
                let (p, _) = v.at(t);
                for (n, &m) in v.members.iter().enumerate() {
                    let Some(k) = self.squad.index(m) else { continue };
                    let at = p.add(super::squad::formation(n).scale(0.6));
                    self.squad.at[k] = at;
                    self.squad.goal[k] = at;
                    self.squad.route[k].clear();
                    self.set_activity(m, Activity::Resting);
                }
            }
            return;
        }
        if self.boarding.is_empty() {
            return;
        }
        let Some(b) = self.nearest_berth() else { return };
        let (Some(dock), Some(path)) = (self.dock_pos(b), self.sea_way(b)) else { return };
        if !self.ship_in(b, t) {
            return;
        }
        let waiting: Vec<PersonId> = self.boarding.iter().copied().filter(|&m| self.squad.index(m).is_some() && !self.is_down(m) && !self.fighting.contains_key(&m)).collect();
        if waiting.is_empty() {
            self.boarding.clear();
            return;
        }
        if !waiting.iter().all(|&m| self.person_pos(m).dist(dock) <= BOARD_REACH) {
            return;
        }
        let length: f32 = path.windows(2).map(|w| w[0].dist(w[1])).sum();
        let arrive = t + (length / SHIP_SPEED) as f64;
        let to_berth = match b {
            Berth::Island => Berth::Port,
            Berth::Port => Berth::Island,
        };
        self.voyage = Some(Voyage { members: waiting.clone(), path, depart: t, arrive, to_berth });
        self.boarding.clear();
        if b == Berth::Island {
            if self.tutorial().and_then(|t| t.path) == Some(super::tutorial::Path::Sneak) {
                self.lesson_done("stow");
                // Diʻub takes them into the Night Market (second's `caught.rs`).
                self.stowed_away();
            }
            self.lesson_done("sail");
        }
        let names: Vec<String> = waiting.iter().map(|&m| self.name_of(m)).collect();
        self.say(t, format!("{} board the ship.", names.join(", ")));
        self.do_boarding();
    }

    /// Is this member at sea on the ship?
    pub fn aboard(&self, m: PersonId) -> bool {
        self.voyage.as_ref().is_some_and(|v| v.members.contains(&m))
    }
}
