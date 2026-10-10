//! Torches, and light in general.
//!
//! Light comes from the sun (by hour), campfires, lit windows in town, and
//! torches. A torch is held in the off hand and lit or put out by order; a
//! standing torch is set in the ground and lights the area round it until it
//! burns down.
//!
//! A lit torch lights the ground round its holder, so they see (and shoot)
//! better in the dark. It also makes them a beacon: at night a torch can be
//! seen from far further than a person, and no amount of creeping hides it.
//!
//! Travellers on the road light torches after dark (`TRAVEL_TORCH_DARK`), so
//! night roads show moving lights, and bandit camps spot them from
//! `TORCH_SEEN` away rather than `encounters::CAMP_SIGHT`.
//!
//! Torches burn down by the clock. A lit torch knows the moment it will go
//! out; when that moment comes it is used up and, if there's another in the
//! pack, the next one is lit from it at exactly that moment. Put out early, a
//! torch keeps what's left of it for next time (while it stays in hand; a
//! part-burnt stub taken out of the hand is thrown away).

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::items::{self, item, Kind, Slot};
use super::person::PersonId;
use super::stealth::daylight;
use super::world::{World, HOUR};

/// Game hours a hand torch burns.
pub const TORCH_HOURS: f32 = 4.0;
/// Game hours a standing torch burns.
pub const STANDING_HOURS: f32 = 8.0;
/// How far a hand torch lights the ground, metres.
pub const TORCH_REACH: f32 = 12.0;
/// How strongly, at its holder's feet (light runs 0..1).
pub const TORCH_POWER: f32 = 0.8;
/// How far a standing torch lights the ground, metres.
pub const STANDING_REACH: f32 = 16.0;
pub const STANDING_POWER: f32 = 0.8;
/// How far a lit torch can be seen at full dark, metres. A person on their
/// own, even in good light, is seen from `stealth::SIGHT`.
pub const TORCH_SEEN: f32 = 220.0;
/// Travellers on the road light torches when daylight falls below this.
pub const TRAVEL_TORCH_DARK: f32 = 0.5;
/// How far magical light and darkness reach round whoever bears them, metres.
pub const GLOW_REACH: f32 = 12.0;
pub const GLOOM_REACH: f32 = 10.0;
/// Hours a campfire stays out once doused.
pub const DOUSE_HOURS: f64 = 3.0;
/// Sneaking with a lit torch: you're still this visible (1 = not hidden at all).
pub const TORCH_SNEAK: f32 = 0.9;

/// A source of light.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub pos: V2,
    /// How far it reaches, metres.
    pub reach: f32,
    /// How much it adds at its centre.
    pub power: f32,
    /// Flat (the same everywhere within reach, like a town's windows) or
    /// fading with distance (a fire).
    pub flat: bool,
}

impl Light {
    pub fn at(&self, p: V2) -> f32 {
        let d = self.pos.dist(p);
        if d >= self.reach {
            0.0
        } else if self.flat {
            self.power
        } else {
            self.power * (1.0 - d / self.reach)
        }
    }
}

/// Light at `p` from the sun at time `t` and the given sources, 0..1. Flat
/// sources don't stack with each other (one town at a time).
pub fn light_from(t: f64, lights: &[Light], p: V2) -> f32 {
    let mut l = daylight(t);
    if l >= 0.99 {
        return 1.0;
    }
    let mut flat = 0.0f32;
    for s in lights {
        let v = s.at(p);
        if s.flat {
            flat = flat.max(v);
        } else {
            l += v;
        }
    }
    (l + flat).clamp(0.0, 1.0)
}

/// Light from a spell: Glow lights the ground round its bearer (or a spot,
/// `radius` across), Gloom darkens it. None for any other effect.
pub fn spell_light(does: super::effects::Does, power: f32, pos: V2, radius: f32) -> Option<Light> {
    use super::effects::Does;
    match does {
        Does::Glow => Some(Light { pos, reach: radius.max(GLOW_REACH), power, flat: false }),
        Does::Gloom => Some(Light { pos, reach: GLOOM_REACH, power: -power, flat: false }),
        // Someone on fire lights the ground like a torch.
        Does::Burning => Some(Light { pos, reach: TORCH_REACH, power: TORCH_POWER, flat: false }),
        _ => None,
    }
}

/// A torch burning in someone's hand.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Flame {
    pub lit_at: f64,
    pub out_at: f64,
}

/// A torch set in the ground.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct StandingTorch {
    pub pos: V2,
    pub lit_at: f64,
    pub out_at: f64,
}

impl StandingTorch {
    pub fn burning(&self, t: f64) -> bool {
        self.lit_at <= t && t < self.out_at
    }
}

fn is_torch(id: items::ItemId) -> bool {
    matches!(item(id).kind, Kind::Torch(_))
}

impl World {
    /// The torch in someone's off hand, if that's what's there.
    pub fn torch_in_hand(&self, pid: PersonId) -> Option<items::ItemId> {
        let d = self.people[pid as usize].detail.as_ref()?;
        d.gear.in_slot(Slot::OffHand).filter(|&i| is_torch(i))
    }

    /// Is this person's torch burning right now?
    pub fn torch_lit(&self, pid: PersonId) -> bool {
        self.torches.get(&pid).map(|f| f.lit_at <= self.time && self.time < f.out_at).unwrap_or(false)
    }

    /// Hours left on the torch in hand (burning or not).
    pub fn torch_hours_left(&self, pid: PersonId) -> Option<f32> {
        let it = self.torch_in_hand(pid)?;
        let secs = match self.torches.get(&pid) {
            Some(f) => f.out_at - self.time,
            None => self.torch_left.get(&pid).copied().unwrap_or(burn_secs(it)),
        };
        Some((secs / HOUR) as f32)
    }

    /// Light or put out a squad member's torch. With none in hand, one is
    /// taken from the pack (which puts away a shield or two-handed weapon).
    pub fn toggle_torch(&mut self, pid: PersonId) -> bool {
        if self.squad.index(pid).is_none() || self.fighting.contains_key(&pid) {
            return false;
        }
        let t = self.time;
        let name = self.people[pid as usize].name().unwrap_or("someone").to_string();
        if let Some(f) = self.torches.remove(&pid) {
            self.torch_left.insert(pid, f.out_at - t);
            self.say(t, format!("{name} puts out the torch."));
            return true;
        }
        // Nobody lights one in their sleep (and going to sleep puts it out),
        // so no torch burns, or is lit afresh, while its holder sleeps.
        if self.is_asleep(pid) {
            self.say(t, format!("{name} is asleep."));
            return false;
        }
        // Soaked through, nothing will light.
        if self.boon(pid, super::effects::Does::Wet) > 0.0 {
            self.say(t, format!("{name} is too wet to light a torch."));
            return false;
        }
        if self.torch_in_hand(pid).is_none() {
            let spare = self.people[pid as usize].detail.as_ref().and_then(|d| d.gear.bag.iter().map(|e| e.0).find(|&i| is_torch(i)));
            let Some(spare) = spare else {
                self.say(t, format!("{name} has no torch."));
                return false;
            };
            self.torch_left.remove(&pid);
            if !self.equip(pid, spare) {
                self.say(t, format!("{name} can't hold a torch."));
                return false;
            }
        }
        let it = self.torch_in_hand(pid).unwrap();
        let left = self.torch_left.remove(&pid).unwrap_or(burn_secs(it));
        self.torches.insert(pid, Flame { lit_at: t, out_at: t + left });
        self.say(t, format!("{name} lights a torch."));
        true
    }

    /// Set a standing torch in the ground beside this squad member, lit.
    pub fn place_torch(&mut self, pid: PersonId) -> bool {
        let it = items::id("standing_torch");
        let t = self.time;
        let pos = self.person_pos(pid).add(V2::new(1.2, 0.4));
        let p = &mut self.people[pid as usize];
        let Some(d) = p.detail.as_mut() else { return false };
        if self.fighting.contains_key(&pid) || !d.gear.take(it) {
            return false;
        }
        p.recompute_might();
        let hours = match item(it).kind {
            Kind::StandingTorch(h) => h,
            _ => STANDING_HOURS,
        };
        self.standing.push(StandingTorch { pos, lit_at: t, out_at: t + hours as f64 * HOUR });
        let name = self.people[pid as usize].name().unwrap_or("someone").to_string();
        self.say(t, format!("{name} sets a torch in the ground."));
        true
    }

    /// Something has left a torch-holder's hand: a lit or part-burnt torch is
    /// thrown away rather than packed (called by unequip and drop).
    pub(super) fn torch_left_hand(&mut self, pid: PersonId) {
        let burning = self.torches.remove(&pid).is_some();
        let stub = self.torch_left.remove(&pid).is_some();
        if burning || stub {
            let it = items::id("torch");
            if let Some(d) = self.people[pid as usize].detail.as_mut() {
                d.gear.take(it);
            }
            self.people[pid as usize].recompute_might();
        }
    }

    /// Nobody sleeps with a lit torch in hand: going to sleep at `t` puts it
    /// out, and what's left of it is kept for later.
    pub(super) fn torch_out_for_sleep(&mut self, pid: PersonId, t: f64) {
        if let Some(f) = self.torches.remove(&pid) {
            if f.out_at > t {
                self.torch_left.insert(pid, f.out_at - t);
            }
        }
    }

    /// Torches that have burnt down by now: used up, and the next lit from
    /// them at that same moment if there's one in the pack.
    pub(super) fn update_torches(&mut self) {
        let t = self.time;
        let mut done: Vec<PersonId> = self.torches.iter().filter(|(_, f)| f.out_at <= t).map(|(&p, _)| p).collect();
        done.sort_unstable();
        for pid in done {
            let mut f = self.torches[&pid];
            let name = self.people[pid as usize].name().unwrap_or("someone").to_string();
            loop {
                // This one's gone.
                let p = &mut self.people[pid as usize];
                let next = p.detail.as_mut().and_then(|d| {
                    d.gear.discard(Slot::OffHand);
                    let spare = d.gear.bag.iter().map(|e| e.0).find(|&i| is_torch(i))?;
                    d.gear.equip(spare).ok().map(|_| spare)
                });
                p.recompute_might();
                match next {
                    Some(it) => {
                        let at = f.out_at;
                        f = Flame { lit_at: at, out_at: at + burn_secs(it) };
                        self.say(at, format!("{name}'s torch burns down; they light another."));
                        if f.out_at > t {
                            self.torches.insert(pid, f);
                            break;
                        }
                    }
                    None => {
                        self.say(f.out_at, format!("{name}'s torch burns out."));
                        self.torches.remove(&pid);
                        break;
                    }
                }
            }
        }
        self.standing.retain(|s| s.out_at > t);
    }

    /// Every light that stays put: campfires, towns' windows at night, and
    /// standing torches.
    pub fn fixed_lights(&self) -> Vec<Light> {
        self.fixed_lights_at(self.time)
    }

    /// The same, as they were (or will be) at time `t`.
    pub fn fixed_lights_at(&self, t: f64) -> Vec<Light> {
        let mut out: Vec<Light> = self.camps.iter().filter(|c| t >= c.doused_until).map(|c| Light { pos: c.pos, reach: 18.0, power: 0.7, flat: false }).collect();
        out.extend(self.settlements.iter().map(|s| Light { pos: s.pos, reach: s.reach + 10.0, power: 0.25, flat: true }));
        out.extend(self.standing.iter().filter(|s| s.burning(t)).map(|s| Light { pos: s.pos, reach: STANDING_REACH, power: STANDING_POWER, flat: false }));
        // Lights set on the ground by spells (a wisp).
        out.extend(self.wards.iter().filter(|w| t < w.until).filter_map(|w| spell_light(w.does, w.power, w.pos, w.radius)));
        out
    }

    /// Is this travelling group carrying a lit torch at time `t`? Travellers
    /// (not bandits) light one while they're on the move after dark.
    pub fn group_torch_lit(&self, g: &super::group::Group, t: f64) -> bool {
        !g.hostile && daylight(t) < TRAVEL_TORCH_DARK && g.is_moving(t) && !self.fighting_groups.contains(&g.id)
    }

    /// Every light, including torches being carried.
    pub fn lights(&self) -> Vec<Light> {
        let mut out = self.fixed_lights();
        // Travellers on the road at night.
        let t = self.time;
        if daylight(t) < TRAVEL_TORCH_DARK {
            out.extend(self.groups.iter().filter(|g| self.group_torch_lit(g, t)).map(|g| Light { pos: g.position_at(t), reach: TORCH_REACH, power: TORCH_POWER, flat: false }));
        }
        let mut held: Vec<PersonId> = self.torches.keys().copied().filter(|&p| self.torch_lit(p)).collect();
        held.sort_unstable();
        out.extend(held.into_iter().map(|p| Light { pos: self.person_pos(p), reach: TORCH_REACH, power: TORCH_POWER, flat: false }));
        // Glow and gloom on people, and lights set on the ground by spells.
        for b in self.boons.iter().filter(|b| t < b.until) {
            if let Some(l) = spell_light(b.does, b.power, self.person_pos(b.pid), 0.0) {
                out.push(l);
            }
        }
        // Anyone on fire in a fight going on here.
        for bt in &self.battles {
            for f in bt.fighters.iter().filter(|f| !f.dead && !f.fled) {
                if let Some(s) = f.has(super::effects::Does::Burning) {
                    out.extend(spell_light(s.does, s.power, f.pos, 0.0));
                }
            }
        }
        out
    }

    pub(super) fn say(&mut self, t: f64, line: String) {
        self.log.push_front((t, line));
        self.log.truncate(14);
    }
}

/// Seconds a torch item burns.
fn burn_secs(it: items::ItemId) -> f64 {
    match item(it).kind {
        Kind::Torch(h) => h as f64 * HOUR,
        _ => TORCH_HOURS as f64 * HOUR,
    }
}
