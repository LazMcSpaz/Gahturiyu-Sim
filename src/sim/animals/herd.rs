//! Herds and packs: wild animals in the groups they keep.
//!
//! A herd is a small record: what it is, where its home range is, and how
//! many it holds. Everything else about it is looked up, never stepped:
//!
//! - **Where it is** comes from the clock. Each hour of the day it is either
//!   resting at its den or out at a spot in its range, and which spot is
//!   drawn from its seed and that stretch of the day. So a herd costs
//!   nothing while nobody is looking, and is in the same place whoever asks.
//! - **How many it holds** is one growing number worked out for any moment
//!   (`region::logistic`), slowed and capped by the hunters living round it.
//!   Every change (a kill, the dawn stock-take) *settles* the number at that
//!   moment and starts again from there.
//! - **Its individuals** are built from its seed: each has a fixed size and
//!   age that never changes once seen. The only things stored per animal
//!   are wounds, and which ones have been killed and replaced.
//!
//! Regions' population numbers are the sums of their herds, so the far-away
//! view (numbers), the middle view (a herd on its rounds) and the close view
//! (individuals) are always the same animals.

use serde::{Deserialize, Serialize};

use super::super::body::{self, Part, HEAL_PER_HOUR, PARTS};
use super::super::combat::{Fighter, Side};
use super::super::effects::Summon;
use super::super::geo::{self, V2};
use super::super::group::GroupId;
use super::super::race::Race;
use super::super::rng::{self, Rng};
use super::super::stats::{Calling, Skill, Stats, N_SKILLS};
use super::super::world::{World, DAY, HOUR};
use super::region::{logistic, neighbours, RegionId};
use super::species::{Class, Sp, Species, Toward};

/// How long a herd stays at one spot in its range before moving on, hours.
pub const ROAM_HOURS: i64 = 2;
/// Walking pace between spots, as a share of flat-out speed.
pub const WALK_SHARE: f32 = 0.25;
/// How often a hunter with a road in its range goes to watch it.
pub const HAUNT_SHARE: f32 = 0.55;
/// An animal is young for this many days after it's born. The animals a
/// world starts with were born some time in the `LIFE_DAYS` before it
/// began, so about one in seven of them start out young.
pub const LIFE_DAYS: f64 = 42.0;
pub const YOUNG_DAYS: f64 = 6.0;
/// How big and strong the young are next to the grown.
pub const YOUNG_SIZE: f32 = 0.6;
/// How hard hunters press on what they hunt: each hunter per prey animal
/// in a region counts this much (see `Herd::settle`).
pub const PREDATION: f32 = 2.0;
/// With hunters pressing as hard as they can, prey grows this much slower
/// and fills this much less of its range.
pub const PREDATION_SLOWS: f32 = 0.6;
pub const PREDATION_THINS: f32 = 0.5;
/// How fast Briarbacks die back once the Overgrowth leaves them, per day.
pub const DIE_BACK: f64 = 0.5;
/// How far from her colony a Silk Mother goes to hunt, metres.
pub const MOTHER_RANGES: f32 = 160.0;

/// Wounds on one animal, as of a moment; they mend at the same steady rate
/// as people's.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct Hurt {
    pub lost: [f32; 6],
    pub at: f64,
}

impl Hurt {
    pub fn lost_at(&self, t: f64) -> [f32; 6] {
        let healed = HEAL_PER_HOUR * ((t - self.at).max(0.0) / HOUR) as f32;
        self.lost.map(|l| (l - healed).max(0.0))
    }

    pub fn healed(&self, t: f64) -> bool {
        self.lost_at(t).iter().all(|l| *l < 0.5)
    }
}

/// A herd out of its usual round: running from someone, or held somewhere
/// by a fight. It goes `from` → `to`, waits there until `until`, then walks
/// back into its round.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Away {
    pub from: V2,
    pub to: V2,
    pub depart: f64,
    pub arrive: f64,
    pub until: f64,
}

/// A pack shadowing travellers it means to attack.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Hunt {
    pub victim: GroupId,
    /// When the pack picked them up, and where it stood from them then
    /// (written down at that moment, so it never has to be worked out again
    /// from a schedule that has since moved on).
    pub seen: f64,
    pub off: V2,
    /// When it closes.
    pub strike: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Herd {
    /// Its place in `Animals::herds`.
    pub id: u32,
    pub sp: Sp,
    pub region: RegionId,
    pub seed: u64,
    /// The middle of its home range (its den, wallow or lair).
    pub home: V2,
    /// A stretch of road or path in its range that a hunter keeps watch on.
    pub haunt: Option<V2>,
    /// The most it ever holds.
    pub cap: u8,

    // --- Numbers: a count at one moment, growing from there ---------------
    pub n: f32,
    pub at: f64,
    /// Growth per second, and the size it grows toward, as last settled.
    pub rate: f64,
    pub limit: f32,
    /// An emptied range is taken up again at this time.
    pub restock_at: f64,

    // --- Individuals ---------------------------------------------------------
    /// Who stands in each place in the herd (empty: nobody has ever been
    /// replaced). An animal's looks come from the herd's seed and this
    /// number, so they never change.
    pub idents: Vec<u16>,
    pub next_ident: u16,
    /// Wounds, by place in the herd.
    pub hurt: Vec<(u8, Hurt)>,
    /// How many were alive at the last stock-take, and who has been born
    /// since the world began and when: (who, when). Anyone not listed was
    /// already there at the start.
    pub grown: u8,
    pub born: Vec<(u16, f64)>,

    // --- Out of its round ----------------------------------------------------
    pub away: Option<Away>,
    pub hunt: Option<Hunt>,
    /// It starts nothing with anyone before this time.
    pub ready_at: f64,
    /// Someone has seen it up close (nothing depends on this).
    pub met: bool,
    /// The silk colony it belongs to, if any.
    pub colony: Option<u32>,
}

/// One animal of a herd, as it is right now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Member {
    /// Its place in the herd.
    pub slot: usize,
    pub ident: u16,
    /// Size next to an ordinary grown one.
    pub size: f32,
    pub young: bool,
    /// Knocked out.
    pub down: bool,
    pub hurt: bool,
}

/// A wild herd someone could hunt, as the society side sees it.
#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub herd: u32,
    pub sp: Sp,
    pub count: usize,
    pub pos: V2,
    /// Not to be hunted (wild Turiyu).
    pub protected: bool,
    /// What one animal gives.
    pub yields: &'static [(&'static str, f32)],
}

impl Herd {
    pub fn def(&self) -> &'static Species {
        self.sp.def()
    }

    // ---- Numbers ------------------------------------------------------------

    /// The count as a smooth number (a herd of 5.6 is five with a sixth on the way).
    pub fn count_at(&self, t: f64) -> f32 {
        if self.limit <= 0.0 {
            // Nothing to live on: dying back.
            return self.n * (-(DIE_BACK / DAY) * (t - self.at).max(0.0)).exp() as f32;
        }
        logistic(self.n, self.limit, self.rate, t - self.at)
    }

    /// How many are alive at time `t`.
    pub fn alive(&self, t: f64) -> usize {
        ((self.count_at(t) + 1e-3).floor().max(0.0) as usize).min(self.cap as usize)
    }

    /// Bring the count up to date at `t` and set how it grows from here,
    /// given how hard hunters are pressing on it (0..1).
    pub fn settle(&mut self, t: f64, pressure: f32) {
        let d = self.def();
        // (Never backwards: something the squad did later in this same step
        // may already have settled it past this moment.)
        let t = t.max(self.at);
        self.n = self.count_at(t);
        self.at = t;
        let p = pressure.clamp(0.0, 1.0);
        self.rate = d.growth as f64 * (1.0 - (PREDATION_SLOWS * p) as f64) / DAY;
        // (Half an animal over, so the count actually reaches the cap.)
        if self.limit > 0.0 {
            self.limit = ((self.cap as f32 + 0.5) * (1.0 - PREDATION_THINS * p)).max(1.5);
        }
        if self.alive(t) == 0 {
            if self.restock_at <= t && self.limit > 0.0 && d.restock_days > 0.0 {
                // Newcomers take up the empty range.
                self.n = (self.cap as f32).min(2.0);
                self.restock_at = f64::INFINITY;
                self.hurt.clear();
                self.born.clear();
                // (Grown animals from elsewhere, not newborns.)
                self.grown = self.alive(t) as u8;
            } else if self.restock_at.is_infinite() {
                self.n = 0.0;
                self.restock_at = t + d.restock_days as f64 * DAY;
            }
        }
        // Whoever has been added since the last stock-take was born.
        let alive = self.alive(t) as u8;
        for slot in self.grown..alive {
            let ident = self.ident(slot as usize);
            self.born.retain(|b| b.0 != ident);
            self.born.push((ident, t));
        }
        self.grown = alive;
        self.born.retain(|b| t - b.1 < YOUNG_DAYS * DAY);
    }

    // ---- Individuals ----------------------------------------------------------

    pub fn ident(&self, slot: usize) -> u16 {
        self.idents.get(slot).copied().unwrap_or(slot as u16)
    }

    pub fn slot_of(&self, ident: u16, t: f64) -> Option<usize> {
        (0..self.alive(t)).find(|&j| self.ident(j) == ident)
    }

    fn member_seed(&self, ident: u16) -> u64 {
        rng::key(&[self.seed, ident as u64, 0x4D45_4D42])
    }

    /// When the animal in this place was born. Those born since the world
    /// began are on record (noted at the stock-take that first counts
    /// them); the rest were born some time in the weeks before it, drawn
    /// from their seed — so a few of every herd start out young, and each
    /// grows up once and stays grown.
    pub fn born_at(&self, slot: usize) -> f64 {
        let ident = self.ident(slot);
        if let Some(b) = self.born.iter().find(|b| b.0 == ident) {
            return b.1;
        }
        -Rng::new(self.member_seed(ident)).f64() * LIFE_DAYS * DAY
    }

    /// Is the animal in this place young at time `t`?
    pub fn young(&self, slot: usize, t: f64) -> bool {
        self.cap > 1 && t - self.born_at(slot) < YOUNG_DAYS * DAY
    }

    /// Size next to an ordinary grown one of its kind.
    pub fn size(&self, slot: usize, t: f64) -> f32 {
        let mut r = Rng::new(self.member_seed(self.ident(slot)));
        let _ = r.f64();
        let grown = 0.88 + 0.26 * r.f32();
        if self.young(slot, t) {
            grown * YOUNG_SIZE
        } else {
            grown
        }
    }

    fn hurt_of(&self, slot: usize) -> Option<&Hurt> {
        self.hurt.iter().find(|h| h.0 as usize == slot).map(|h| &h.1)
    }

    /// Damage on each part of the animal in this place at time `t`.
    pub fn lost(&self, slot: usize, t: f64) -> [f32; 6] {
        self.hurt_of(slot).map(|h| h.lost_at(t)).unwrap_or([0.0; 6])
    }

    /// Health on each part, and the most it can be.
    pub fn health(&self, slot: usize, t: f64) -> ([f32; 6], [f32; 6]) {
        let max = max_hp(self.sp, self.size(slot, t));
        let lost = self.lost(slot, t);
        let mut hp = max;
        for k in 0..6 {
            hp[k] -= lost[k];
        }
        (hp, max)
    }

    /// Knocked out: head or body at nothing.
    pub fn down(&self, slot: usize, t: f64) -> bool {
        self.hurt_of(slot).is_some() && body::knocked_out(&self.health(slot, t).0)
    }

    /// When everyone knocked out will be able to stand again.
    pub fn all_up_at(&self, t: f64) -> f64 {
        let mut worst = 0.0f32;
        for j in 0..self.alive(t) {
            let (hp, _) = self.health(j, t);
            for p in [Part::Head, Part::Torso] {
                worst = worst.max(-hp[p as usize]);
            }
        }
        t + (worst.max(0.0) / HEAL_PER_HOUR) as f64 * HOUR
    }

    pub fn member(&self, slot: usize, t: f64) -> Member {
        let hurt = self.hurt_of(slot).map(|h| !h.healed(t)).unwrap_or(false);
        Member { slot, ident: self.ident(slot), size: self.size(slot, t), young: self.young(slot, t), down: hurt && self.down(slot, t), hurt }
    }

    pub fn members(&self, t: f64) -> Vec<Member> {
        (0..self.alive(t)).map(|j| self.member(j, t)).collect()
    }

    /// Write down what a fight left on the animal in this place.
    pub fn set_hurt(&mut self, slot: usize, lost: [f32; 6], t: f64) {
        self.hurt.retain(|h| h.0 as usize != slot);
        if lost.iter().any(|l| *l > 0.5) {
            self.hurt.push((slot as u8, Hurt { lost: lost.map(|l| l.max(0.0)), at: t }));
        }
    }

    /// The animal in this place is gone (killed, or taken away). The last
    /// one in the herd takes its place, and whoever is born into the freed
    /// place later is someone new.
    pub fn remove(&mut self, slot: usize, t: f64) {
        let alive = self.alive(t);
        if slot >= alive {
            return;
        }
        let count = self.count_at(t);
        if self.idents.is_empty() {
            self.idents = (0..self.cap as u16).collect();
            self.next_ident = self.cap as u16;
        }
        let last = alive - 1;
        self.idents[slot] = self.idents[last];
        self.idents[last] = self.next_ident;
        self.next_ident = self.next_ident.wrapping_add(1);
        self.hurt.retain(|h| h.0 as usize != slot && (h.0 as usize) < alive);
        for h in self.hurt.iter_mut() {
            if h.0 as usize == last {
                h.0 = slot as u8;
            }
        }
        self.n = (count.min(self.cap as f32 + 0.999) - 1.0).max(0.0);
        self.at = self.at.max(t);
        self.grown = self.grown.min(self.alive(t) as u8);
        if self.alive(t) == 0 {
            self.n = 0.0;
            self.restock_at = t + self.def().restock_days as f64 * DAY;
            self.hurt.clear();
            self.born.clear();
        }
    }

    /// How dangerous the herd is as it stands: the same kind of number as
    /// people's might, so the two can be weighed against each other.
    pub fn might(&self, t: f64) -> f32 {
        let m = self.def().body.might;
        (0..self.alive(t)).filter(|&j| !self.down(j, t)).map(|j| m * self.size(j, t)).sum()
    }

    // ---- Where it is, from the clock --------------------------------------------

    /// Up and about at time `t` (rather than lying up at home).
    pub fn active(&self, t: f64) -> bool {
        self.def().active.at(t)
    }

    /// A spot out in the home range for one stretch of the day.
    fn roam(&self, block: i64) -> V2 {
        let d = self.def();
        let mut r = Rng::from_keys(&[self.seed, block as u64, 0x524F_414D]);
        if self.sp == Sp::Tidepicker {
            // Out over the wet rocks along the water's edge.
            let y = self.home.y + r.range(-d.range, d.range);
            return V2::new(geo::coast_x(y) + r.range(3.0, 20.0), y);
        }
        if let Some(h) = self.haunt {
            if r.chance(HAUNT_SHARE) {
                let a = r.f32() * std::f32::consts::TAU;
                let p = h.add(V2::new(a.cos(), a.sin()).scale(r.range(15.0, 90.0)));
                return if geo::is_land(p) && geo::inland(p) > 8.0 { p } else { h };
            }
        }
        let a = r.f32() * std::f32::consts::TAU;
        let p = self.home.add(V2::new(a.cos(), a.sin()).scale(d.range * r.f32().sqrt()));
        if geo::is_land(p) && geo::inland(p) > 8.0 {
            p
        } else {
            self.home
        }
    }

    /// Where the herd spends hour number `slot` (hours since the world began).
    fn spot(&self, slot: i64) -> V2 {
        let mid = (slot as f64 + 0.5) * HOUR;
        if self.sp == Sp::SilkMother {
            // At the colony, except while she's off hunting.
            if !self.mother_out(mid) {
                return self.home;
            }
            let a = Rng::from_keys(&[self.seed, slot.div_euclid(24) as u64, 0x4F55_5421]).f32() * std::f32::consts::TAU;
            let p = self.home.add(V2::new(a.cos(), a.sin()).scale(MOTHER_RANGES));
            return if geo::is_land(p) { p } else { self.home };
        }
        if self.active(mid) {
            self.roam(slot.div_euclid(ROAM_HOURS))
        } else {
            self.home
        }
    }

    /// Where its round has it at time `t`: at this hour's spot, or on the
    /// way there from the last one.
    pub fn round_pos(&self, t: f64) -> V2 {
        let slot = (t / HOUR).floor() as i64;
        let (a, b) = (self.spot(slot - 1), self.spot(slot));
        let dist = a.dist(b);
        if dist < 0.5 {
            return b;
        }
        let pace = (self.def().speed * WALK_SHARE).max(0.3);
        let travel = ((dist / pace) as f64).min(0.8 * HOUR);
        a.lerp(b, (((t - slot as f64 * HOUR) / travel) as f32).clamp(0.0, 1.0))
    }

    /// Where it is at time `t`, away from its round or on it. (A pack
    /// shadowing travellers is placed by `World::herd_pos`, which knows
    /// where the travellers are.)
    pub fn pos_at(&self, t: f64) -> V2 {
        let on_round = self.round_pos(t);
        let Some(a) = self.away else { return on_round };
        if t < a.depart {
            return on_round;
        }
        if t < a.arrive {
            return a.from.lerp(a.to, (((t - a.depart) / (a.arrive - a.depart).max(1e-6)) as f32).clamp(0.0, 1.0));
        }
        if t < a.until {
            return a.to;
        }
        // Walking back into its round.
        let pace = (self.def().speed * WALK_SHARE * 2.0).max(0.5);
        let back = ((a.to.dist(self.round_pos(a.until)) / pace) as f64).max(30.0);
        a.to.lerp(on_round, (((t - a.until) / back) as f32).clamp(0.0, 1.0))
    }

    /// A word for what it's doing at time `t`.
    pub fn doing(&self, t: f64) -> &'static str {
        if let Some(h) = self.hunt {
            if t >= h.seen {
                return "stalking";
            }
        }
        if let Some(a) = self.away {
            if t >= a.depart && t < a.arrive {
                return if a.from.dist(a.to) > 1.0 { "running" } else { "standing its ground" };
            }
            if t < a.until {
                return if self.def().class == Class::Predator { "feeding" } else { "wary" };
            }
        }
        let d = self.def();
        if !self.active(t) {
            return match d.toward {
                Toward::Ambusher => "lying in wait",
                _ if self.sp == Sp::Wallowback && super::species::phase(t) == super::species::Phase::Day => "wallowing",
                _ if self.sp == Sp::Tidepicker => "hidden under the rocks",
                _ => "resting",
            };
        }
        match d.toward {
            Toward::PackHunter => "hunting",
            Toward::Ambusher => "lying in wait",
            Toward::Territorial => "guarding its ground",
            Toward::Hostile => "prowling",
            Toward::Harmless if self.sp == Sp::Silkcrawler => "spinning",
            Toward::Harmless => "foraging",
            _ => "grazing",
        }
    }

    /// How far the animals of a herd spread from its middle, metres.
    pub fn spread(&self, t: f64) -> f32 {
        let d = self.def();
        match self.sp {
            Sp::Tidepicker => 9.0,
            Sp::Silkcrawler => 6.0,
            _ => 1.0 + (self.alive(t) as f32).sqrt() * d.looks.len.max(0.6) * 1.3,
        }
    }

    /// Where the animal in this place stands, round the herd's middle `c`.
    pub fn member_at(&self, c: V2, slot: usize, t: f64) -> V2 {
        let mut r = Rng::new(self.member_seed(self.ident(slot)) ^ 0x5350_4F54);
        let a = r.f32() * std::f32::consts::TAU;
        let rad = self.spread(t) * r.f32().sqrt();
        // A slow drift, so a herd isn't a frozen picture. Drawing only in
        // effect: nothing is decided by it.
        let phase = r.f32() * 100.0;
        let tt = (t / 90.0) as f32 + phase;
        let drift = if self.down(slot, t) { 0.0 } else { self.def().looks.len.clamp(0.2, 1.5) * 0.6 };
        c.add(V2::new(a.cos(), a.sin()).scale(rad)).add(V2::new(tt.sin(), (tt * 0.7).cos()).scale(drift))
    }

    /// The animal in this place as a fighter. It fights under the same rules
    /// as anyone; only its numbers come from the species table.
    pub fn fighter(&self, slot: usize, t: f64, side: Side, pos: V2) -> Fighter {
        let size = self.size(slot, t);
        fighter(self.sp, size, &self.lost(slot, t), side, pos)
    }
}

/// An animal's numbers as fight stats.
fn stats_of(sp: Sp) -> Stats {
    let b = &sp.def().body;
    let mut skills = [0.0; N_SKILLS];
    skills[Skill::Unarmed as usize] = b.skill;
    skills[Skill::Dodge as usize] = b.dodge;
    skills[Skill::Block as usize] = b.skill * 0.5;
    Stats { attrs: [b.strength, b.agility, b.toughness, 10.0, 60.0], skills, calling: Calling::Common }
}

/// The most health each part of an animal can have.
pub fn max_hp(sp: Sp, size: f32) -> [f32; 6] {
    let s = stats_of(sp);
    let mut out = [0.0; 6];
    for (i, p) in PARTS.iter().enumerate() {
        out[i] = s.max_hp(*p) * sp.def().body.hp * size;
    }
    out
}

/// An animal as a fighter, `size` times an ordinary grown one, carrying
/// `lost` damage already.
///
/// It is built on `Fighter::creature` (the constructor for anything in a
/// fight that isn't a person) and then given its own numbers, so that the
/// fight rules never look it up among the people. The creature kind it
/// borrows is only a marker; see ANIMALS.md for what a proper one would be.
pub fn fighter(sp: Sp, size: f32, lost: &[f32; 6], side: Side, pos: V2) -> Fighter {
    let d = sp.def();
    let b = &d.body;
    let mut f = Fighter::creature(Summon::Swarmling, side, pos, f64::INFINITY, 1.0, Race::Roduro);
    f.stats = stats_of(sp);
    f.max_hp = max_hp(sp, size);
    for k in 0..6 {
        f.hp[k] = f.max_hp[k] - lost[k];
    }
    f.weapon.cut = b.cut * size;
    f.weapon.blunt = b.blunt * size;
    f.weapon.reach = b.reach * size.max(0.7);
    f.weapon_name = b.attack;
    f.shield = 0.0;
    f.base_speed = d.speed;
    f.might = b.might * size;
    f.boldness = 1.0;
    f.ko = body::knocked_out(&f.hp);
    f
}

impl World {
    /// Where a herd is at time `t`.
    pub fn herd_pos(&self, herd: u32, t: f64) -> V2 {
        let h = &self.animals.herds[herd as usize];
        if let Some(hu) = h.hunt {
            if t >= hu.seen {
                if let Some(g) = self.group(hu.victim) {
                    // Closing on them from where it picked them up.
                    let quarry = g.position_at(t);
                    let from = hu.off;
                    let far = from.len().max(1.0);
                    let f = (((t - hu.seen) / (hu.strike - hu.seen).max(1.0)) as f32).clamp(0.0, 1.0);
                    let keep = far + (STALK_CLOSE.min(far) - far) * f;
                    return quarry.add(from.scale(keep / far));
                }
            }
        }
        h.pos_at(t)
    }

    /// Where one animal of a herd stands at time `t`.
    pub fn animal_pos(&self, herd: u32, slot: usize, t: f64) -> V2 {
        let h = &self.animals.herds[herd as usize];
        h.member_at(self.herd_pos(herd, t), slot, t)
    }

    /// How hard hunters are pressing on one kind of prey in a region, 0..1:
    /// the hunters living in and round it, against the prey there.
    pub fn predation(&self, r: RegionId, prey: Sp, t: f64) -> f32 {
        let a = &self.animals;
        let mut hunters = 0.0;
        for nr in neighbours(r) {
            let share = if nr == r { 1.0 } else { 0.35 };
            for &h in &a.regions[nr as usize].herds {
                let h = &a.herds[h as usize];
                if h.def().prey.contains(&prey) {
                    hunters += share * h.alive(t) as f32;
                }
            }
        }
        if hunters <= 0.0 {
            return 0.0;
        }
        let here: f32 = a.regions[r as usize].herds.iter().map(|&h| &a.herds[h as usize]).filter(|h| h.sp == prey).map(|h| h.alive(t) as f32).sum();
        (PREDATION * hunters / here.max(1.0)).min(1.0)
    }

    /// Bring every herd's numbers up to date and set how each grows from
    /// here: the dawn stock-take. Also when emptied ranges are taken up.
    pub(super) fn settle_herds(&mut self, t: f64) {
        let pressure: Vec<f32> = self.animals.herds.iter().map(|h| if h.def().class == Class::Prey { self.predation(h.region, h.sp, t) } else { 0.0 }).collect();
        for (h, p) in self.animals.herds.iter_mut().zip(pressure) {
            h.settle(t, p);
            // Old wounds, and running-off long over, are forgotten.
            h.hurt.retain(|x| !x.1.healed(t));
            if h.away.map(|a| t > a.until + HOUR).unwrap_or(false) {
                h.away = None;
            }
        }
    }

    /// Wild herds of things worth hunting within `radius` of a point: what
    /// they are, how many, where, and what one animal gives. (For hunters:
    /// "what game is near this point".)
    pub fn game_near(&self, p: V2, radius: f32) -> Vec<Game> {
        let t = self.time;
        self.animals
            .herds
            .iter()
            .filter(|h| h.def().class == Class::Prey && h.home.dist(p) <= radius + h.def().range)
            .filter_map(|h| {
                let count = h.alive(t);
                let pos = self.herd_pos(h.id, t);
                (count > 0 && pos.dist(p) <= radius).then(|| Game { herd: h.id, sp: h.sp, count, pos, protected: h.def().protected, yields: h.def().yields })
            })
            .collect()
    }

    /// Take up to `n` animals out of a wild herd without a fight (a hunter's
    /// day's work, say). Returns what they give, as names and amounts, and
    /// leaves the carcasses to nobody. The herd's numbers drop and grow back
    /// on their own.
    pub fn take_animals(&mut self, herd: u32, n: usize) -> Vec<(&'static str, f32)> {
        self.take_animals_at(herd, n, self.time)
    }

    /// The same, as of a given moment (for a caller on the world's timeline).
    pub fn take_animals_at(&mut self, herd: u32, n: usize, t: f64) -> Vec<(&'static str, f32)> {
        let Some(h) = self.animals.herds.get_mut(herd as usize) else { return Vec::new() };
        let t = t.max(h.at);
        let took = n.min(h.alive(t));
        for _ in 0..took {
            let last = h.alive(t) - 1;
            h.remove(last, t);
        }
        self.animals.stats.animals_killed += took as u32;
        h_yields(self.animals.herds[herd as usize].sp, took as f32)
    }

    /// A word for what a herd is doing right now.
    pub fn herd_doing(&self, herd: u32) -> &'static str {
        self.animals.herds[herd as usize].doing(self.time)
    }
}

/// How close a shadowing pack gets before it strikes, metres.
pub const STALK_CLOSE: f32 = 22.0;

/// What `n` animals of a species give.
pub fn h_yields(sp: Sp, n: f32) -> Vec<(&'static str, f32)> {
    sp.def().yields.iter().map(|(k, a)| (*k, a * n)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn herd(sp: Sp, cap: u8, n: f32) -> Herd {
        Herd {
            id: 0,
            sp,
            region: 0,
            seed: 77,
            home: V2::new(9_000.0, 9_000.0),
            haunt: None,
            cap,
            n,
            at: 0.0,
            rate: sp.def().growth as f64 / DAY,
            limit: cap as f32 + 0.5,
            restock_at: f64::INFINITY,
            idents: Vec::new(),
            next_ident: 0,
            hurt: Vec::new(),
            grown: n.floor() as u8,
            born: Vec::new(),
            away: None,
            hunt: None,
            ready_at: 0.0,
            met: false,
            colony: None,
        }
    }

    #[test]
    fn everyone_grows_up_once_and_newcomers_are_born_young() {
        let mut h = herd(Sp::Ridgehound, 6, 4.2);
        // Whoever starts out young is grown within the week and stays grown.
        for j in 0..4 {
            assert!(!h.young(j, YOUNG_DAYS * DAY + 1.0));
            assert!(!h.young(j, 300.0 * DAY));
        }
        // The pack grows; the stock-take that first counts the newcomer
        // notes its birth, and it is young for its first days only.
        let later = 200.0 * DAY;
        assert!(h.alive(later) > 4);
        h.settle(later, 0.0);
        assert!(h.young(4, later + DAY), "a newborn should be young");
        assert!(h.size(4, later + DAY) < 0.8);
        assert!(!h.young(4, later + 10.0 * DAY) && h.size(4, later + 10.0 * DAY) > 0.8, "and grown ten days on");
    }

    #[test]
    fn a_herd_stays_in_its_range_and_is_where_the_clock_says() {
        let h = herd(Sp::Brushleaper, 6, 5.0);
        for k in 0..400 {
            let t = k as f64 * 613.0;
            let p = h.round_pos(t);
            assert!(p.dist(h.home) <= h.def().range + 1.0, "strayed {} m at {t}", p.dist(h.home));
            assert_eq!(p, h.round_pos(t), "asked twice, two answers");
        }
        // It moves by walking, not by jumping: no faster than it can run.
        let mut last = h.round_pos(0.0);
        for k in 1..5000 {
            let p = h.round_pos(k as f64 * 10.0);
            assert!(last.dist(p) <= h.def().speed * 10.0 + 0.1, "jumped {} m in 10 s", last.dist(p));
            last = p;
        }
    }

    #[test]
    fn killing_one_keeps_the_others_who_they_were() {
        let mut h = herd(Sp::Ridgehound, 6, 6.2);
        let t = 100.0;
        let before: Vec<(u16, f32)> = (0..6).map(|j| (h.ident(j), h.size(j, t))).collect();
        h.set_hurt(5, [0.0, 30.0, 0.0, 0.0, 0.0, 0.0], t);
        h.remove(2, t);
        assert_eq!(h.alive(t), 5);
        // The last one moved into the gap, wounds and all.
        assert_eq!(h.ident(2), before[5].0);
        assert_eq!(h.size(2, t), before[5].1);
        assert!(h.lost(2, t)[1] > 29.0);
        for j in [0, 1, 3, 4] {
            assert_eq!((h.ident(j), h.size(j, t)), before[j]);
        }
        // Whoever grows into the sixth place later is someone new.
        assert!(!before.iter().any(|b| b.0 == h.ident(5)));
    }

    #[test]
    fn a_herd_grows_back_and_an_emptied_range_is_taken_up() {
        let mut h = herd(Sp::Brushleaper, 6, 6.4);
        for _ in 0..4 {
            h.remove(0, 0.0);
        }
        assert_eq!(h.alive(0.0), 2);
        assert!(h.alive(60.0 * DAY) > 2, "left alone it recovers");
        assert_eq!(h.alive(400.0 * DAY), 6);
        h.remove(0, 0.0);
        h.remove(0, 0.0);
        assert_eq!(h.alive(50.0 * DAY), 0, "none left to breed");
        h.settle(h.restock_at + 1.0, 0.0);
        assert_eq!(h.alive(h.at), 2);
    }

    #[test]
    fn hunters_hold_prey_down() {
        let mut free = herd(Sp::Brushleaper, 6, 3.0);
        let mut pressed = free.clone();
        free.settle(0.0, 0.0);
        pressed.settle(0.0, 1.0);
        let later = 200.0 * DAY;
        assert!(pressed.count_at(later) < free.count_at(later) - 1.0);
    }
}
