//! Memory and grudges (Part 5, Stage 2).
//!
//! **Memories.** Each person keeps a few memories (`MEMORY_CAP`) of who wronged
//! or helped them, how, how much and when. A second deed by the same person
//! adds to what's remembered of them rather than taking a new line; when the
//! list is full the faintest memory goes. Memories fade with time — worked
//! out from the day they were made, never ticked — and the patient forget a
//! wrong more slowly.
//!
//! **Households** keep a short list (`FEELINGS_CAP`) of other households they
//! feel warm or cold toward, fed by what their members remember of that
//! household's people. Feuds live here.
//!
//! **Daily dealings.** At dawn, a few people have had something worth
//! remembering happen the day before with someone their routine actually
//! put them next to — at work, at the evening spot, or next door: a
//! kindness, a slight, a quarrel over work or a debt, a dispute over a sale.
//! Most days, for most people, nothing. Temper, needs and what they already
//! remember of each other weight it.
//!
//! **Escalation.** A household that has gone cold enough on another climbs a
//! ladder, one rung at a time and never faster than `ESCALATE_GAP`: it
//! avoids them; takes it to a public dispute; does them harm; turns to
//! violence; and at the top, a feud. Honour picks the branch at each rung —
//! the honourable act in the open (a hearing judged by custom; a claim, a
//! duel, a shunning or the record, whichever the wronged side's custom
//! favours; an open fight), the rest covertly (slander, sabotage or theft, a
//! beating in the dark) and may never be found out. Each act is remembered
//! by the other side, so a grudge can feed itself until it fades.
//!
//! Everything is settled at dawn with rolls keyed to who, whom and the day.

use serde::{Deserialize, Serialize};

use super::body;
use super::culture::Justice;
use super::history::Deed;
use super::lives::RESERVE_DAYS;
use super::person::PersonId;
use super::rng::{self, Rng};
use super::routine::{Doing, Spot};
use super::settlement::SettlementId;
use super::world::World;

// ---- Dials -----------------------------------------------------------------

/// Most memories a person keeps; most households a household feels strongly about.
pub const MEMORY_CAP: usize = 6;
pub const FEELINGS_CAP: usize = 4;
/// How fast wrongs fade a day (× 1.5 − patience) and good turns fade.
pub const WRONG_FADE: f32 = 0.03;
pub const HELP_FADE: f32 = 0.05;
/// The most a memory can hold, either way.
pub const MOST: f32 = 1.5;
/// Memories fainter than this are forgotten.
pub const FORGOTTEN: f32 = 0.04;
/// Share of a member's memory that their household feels.
pub const HOUSE_SHARE: f32 = 0.5;
/// How fast a household's feelings cool a day (× 1.5 − its patience); feuds
/// cool at `FEUD_COOL` of that.
pub const FEEL_FADE: f32 = 0.04;
pub const FEUD_COOL: f32 = 0.3;
/// Chance someone has a dealing worth remembering on a day (× 0.6 +
/// sociability × 0.8).
pub const DEALING_CHANCE: f32 = 0.05;
/// A household bigger than this is a village living as one: its members
/// keep score of each other.
pub const BIG_HOUSEHOLD: usize = 8;
/// How much each kind of dealing is remembered: (by the other, by the one
/// who started it).
pub const KINDNESS: (f32, f32) = (0.25, 0.1);
pub const SLIGHT: (f32, f32) = (-0.25, -0.05);
pub const QUARREL: (f32, f32) = (-0.3, -0.25);
pub const LOAN_THANKS: f32 = 0.15;
/// Warmth below which a household takes each next rung of the ladder:
/// avoid, dispute, harm, violence, feud.
pub const RUNGS: [f32; 5] = [-0.2, -0.35, -0.5, -0.65, -0.8];
/// Days at least between rungs (and between acts in a feud).
pub const ESCALATE_GAP: i32 = 3;
/// Chance a day of taking the next rung (× 0.5 + boldness/2 + (1 − patience)/2).
pub const ESCALATE_CHANCE: f32 = 0.5;
/// Honour at which the open and covert branches are even, and how wide the
/// band where either can happen.
pub const OPEN_AT: f32 = 0.5;
pub const OPEN_BAND: f32 = 0.3;
/// Chance a covert act is found out (who did it).
pub const FOUND_OUT: f32 = 0.35;
/// What each act takes from the other household: sabotage and theft in days
/// of its costs.
pub const SABOTAGE_DAYS: f32 = 1.5;
pub const THEFT_DAYS: f32 = 2.0;
/// A claim at the hall upheld costs the other household this many days.
pub const CLAIM_DAYS: f32 = 3.0;
/// How hard each rung is remembered by the one it's done to.
pub const HURT: [f32; 5] = [-0.05, -0.3, -0.4, -0.5, -0.3];
/// A beating leaves this share of each part's hit points.
pub const BEATEN_TO: f32 = 0.45;
/// Unrest a feud's outbreak adds.
pub const FEUD_UNREST: f32 = 4.0;

// ---- State -----------------------------------------------------------------

/// Who a memory is about.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    Person(PersonId),
    Household(u32),
    /// A town's criminal ring.
    Ring(SettlementId),
    /// Whoever it was: a wrong with no one to blame.
    Someone,
}

/// Something someone remembers: who wronged or helped them, how, how much.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Memory {
    pub about: Who,
    /// The last thing they did.
    pub deed: Deed,
    /// How much, when made: below 0 a wrong, above a good turn.
    pub amount: f32,
    pub day: i32,
}

impl Memory {
    /// How strongly it's remembered on `day`, by someone this patient.
    pub fn strength(&self, day: i32, patience: f32) -> f32 {
        let rate = if self.amount < 0.0 { WRONG_FADE * (1.5 - patience) } else { HELP_FADE };
        self.amount * (-rate * (day - self.day).max(0) as f32).exp()
    }
}

/// How one household feels about another, and how far up the ladder it's gone.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Feeling {
    pub other: u32,
    pub warmth: f32,
    /// 0 nothing yet; 1 avoiding; 2 disputed; 3 harmed; 4 violence; 5 feud.
    pub stage: u8,
    /// The day of the last rung.
    pub since: i32,
}

pub const STAGES: [&str; 6] = ["", "avoiding them", "in dispute", "doing them harm", "come to blows", "in a feud"];

// ---- Remembering ------------------------------------------------------------

impl World {
    fn patience(&self, p: PersonId) -> f32 {
        self.people[p as usize].traits.patience
    }

    /// How strongly someone remembers a person now (0 if they don't).
    pub fn memory_of(&self, p: PersonId, about: Who, day: i32) -> f32 {
        let pat = self.patience(p);
        self.mind(p).memories.iter().filter(|m| m.about == about).map(|m| m.strength(day, pat)).sum()
    }

    /// How aggrieved someone is, 0..1, from the wrongs they remember.
    pub fn grievance_of(&self, p: PersonId) -> f32 {
        let day = World::day_of(self.time) as i32;
        let pat = self.patience(p);
        let bad: f32 = self.mind(p).memories.iter().map(|m| m.strength(day, pat)).filter(|&s| s < 0.0).map(|s| -s).sum();
        (bad / 1.5).min(1.0)
    }

    /// The wrong someone remembers most, if any.
    pub fn top_grudge(&self, p: PersonId) -> Option<Memory> {
        let day = World::day_of(self.time) as i32;
        let pat = self.patience(p);
        self.mind(p).memories.iter().copied().filter(|m| m.strength(day, pat) < -0.1).min_by(|a, b| a.strength(day, pat).total_cmp(&b.strength(day, pat)))
    }

    /// Someone remembers a deed. A second deed by the same hand adds to what's
    /// remembered of them; when the list is full the faintest goes. Their
    /// household feels a share of it toward the other's.
    pub fn remember(&mut self, p: PersonId, about: Who, deed: Deed, amount: f32, day: i32) {
        if p as usize >= self.society.minds.len() || self.people[p as usize].dead {
            return;
        }
        let pat = self.patience(p);
        let m = &mut self.society.minds[p as usize];
        match m.memories.iter_mut().find(|x| x.about == about) {
            Some(x) => {
                x.amount = (x.strength(day, pat) + amount).clamp(-MOST, MOST);
                x.deed = deed;
                x.day = day;
            }
            None => {
                m.memories.retain(|x| x.strength(day, pat).abs() >= FORGOTTEN);
                m.memories.push(Memory { about, deed, amount, day });
                if m.memories.len() > MEMORY_CAP {
                    let k = (0..m.memories.len()).min_by(|&a, &b| m.memories[a].strength(day, pat).abs().total_cmp(&m.memories[b].strength(day, pat).abs())).unwrap();
                    m.memories.remove(k);
                }
            }
        }
        let other = match about {
            Who::Person(q) => self.society.lives.get(q as usize).and_then(|l| l.household),
            Who::Household(h) => Some(h),
            _ => None,
        };
        if let (Some(h), Some(o)) = (self.society.lives[p as usize].household, other) {
            if h != o {
                self.feel(h, o, amount * HOUSE_SHARE, day);
            }
        }
    }

    /// A household's feeling toward another moves by `delta`.
    pub fn feel(&mut self, h: u32, o: u32, delta: f32, day: i32) {
        let fs = &mut self.society.households[h as usize].feelings;
        match fs.iter_mut().find(|f| f.other == o) {
            Some(f) => f.warmth = (f.warmth + delta).clamp(-1.0, 1.0),
            None => {
                let new = Feeling { other: o, warmth: delta.clamp(-1.0, 1.0), stage: 0, since: day };
                if fs.len() < FEELINGS_CAP {
                    fs.push(new);
                } else if let Some(k) = (0..fs.len()).filter(|&k| fs[k].stage < 5).min_by(|&a, &b| fs[a].warmth.abs().total_cmp(&fs[b].warmth.abs())) {
                    if fs[k].warmth.abs() < delta.abs() {
                        fs[k] = new;
                    }
                }
            }
        }
    }

    /// How a household feels about another (0 if it doesn't much).
    pub fn feeling(&self, h: u32, o: u32) -> Option<Feeling> {
        self.society.households.get(h as usize)?.feelings.iter().find(|f| f.other == o).copied()
    }

    /// Is either household avoiding the other?
    fn avoiding(&self, h: u32, o: u32) -> bool {
        self.feeling(h, o).is_some_and(|f| f.stage >= 1) || self.feeling(o, h).is_some_and(|f| f.stage >= 1)
    }

    fn living(&self, h: u32) -> Vec<PersonId> {
        self.society.households[h as usize].members.iter().copied().filter(|&m| !self.people[m as usize].dead && !self.people[m as usize].in_squad).collect()
    }

    // ---- At dawn -----------------------------------------------------------------

    /// The day before's dealings, the households' feelings cooling, and any
    /// grudge climbing a rung.
    pub(super) fn dawn_ties(&mut self, town: SettlementId, t: f64) {
        self.dealings(town, t);
        self.cool_feelings(town);
        self.escalate(town, t);
    }

    fn town_households(&self, town: SettlementId) -> Vec<u32> {
        (0..self.society.households.len() as u32).filter(|&h| self.society.communities[self.society.households[h as usize].community as usize].town == town).collect()
    }

    /// Who met whom yesterday: workmates, those at the same evening spot,
    /// neighbours. A few of them had something worth remembering.
    fn dealings(&mut self, town: SettlementId, t: f64) {
        let today = World::day_of(t);
        let yday = today - 1;
        let tl = &self.society.towns[town as usize];
        for ci in std::iter::once(tl.shore).chain(tl.stilts).collect::<Vec<_>>() {
            let folk: Vec<PersonId> = self.members_of(ci).filter(|&p| !self.people[p as usize].in_squad && self.society.lives[p as usize].household.is_some() && self.busy_until[p as usize] <= t).collect();
            // Where each was: (kind, which) — work, evening spot, neighbourhood.
            let mut venues: std::collections::BTreeMap<(u8, u32), Vec<PersonId>> = Default::default();
            let mut were: Vec<[Option<(u8, u32)>; 3]> = Vec::with_capacity(folk.len());
            for &p in &folk {
                let l = self.society.lives[p as usize];
                let plan = self.day_plan(p, yday);
                let work = match (plan.work, l.place) {
                    (Some(_), Some(pl)) => Some((0u8, pl as u32)),
                    _ => None,
                };
                let eve = plan.segs().iter().find(|s| matches!(s.doing, Doing::Evening | Doing::Market)).and_then(|s| match s.spot {
                    Spot::Place(x) => Some((1u8, x as u32)),
                    Spot::Hearth => Some((2, 0)),
                    Spot::Visit(b) => Some((3, b as u32)),
                    _ => None,
                });
                let home = self.society.households[l.household.unwrap() as usize].home.map(|b| (4u8, b as u32 / 3));
                let w = [work, eve, home];
                for v in w.iter().flatten() {
                    venues.entry(*v).or_default().push(p);
                }
                were.push(w);
            }
            for (i, &p) in folk.iter().enumerate() {
                let tr = self.people[p as usize].traits;
                let mut r = Rng::from_keys(&[self.seed, p as u64, today as u64, 0x4445_414C]);
                if !r.chance(DEALING_CHANCE * (0.6 + tr.sociability * 0.8)) {
                    continue;
                }
                let at: Vec<(u8, u32)> = were[i].iter().flatten().copied().collect();
                if at.is_empty() {
                    continue;
                }
                let v = at[r.below(at.len())];
                let there = &venues[&v];
                let q = there[r.below(there.len())];
                let (hp, hq) = (self.society.lives[p as usize].household.unwrap(), self.society.lives[q as usize].household.unwrap());
                // Close kin don't keep score; a big shared household (a whole
                // village) does, person to person.
                let big = self.society.households[hp as usize].members.len() > BIG_HOUSEHOLD;
                // Those avoiding each other still work side by side.
                if q == p || (hp == hq && !big) || (v.0 != 0 && self.avoiding(hp, hq)) {
                    continue;
                }
                self.deal(p, q, v.0, &mut r, today as i32);
            }
        }
    }

    /// What passed between `p` and `q` (met at a venue of kind `v`).
    fn deal(&mut self, p: PersonId, q: PersonId, v: u8, r: &mut Rng, day: i32) {
        let tr = self.people[p as usize].traits;
        let (hp, hq) = (self.society.lives[p as usize].household.unwrap(), self.society.lives[q as usize].household.unwrap());
        let warm = self.feeling(hp, hq).map(|f| f.warmth).unwrap_or(0.0);
        let mem = self.memory_of(p, Who::Person(q), day);
        let needs = self.mind(p).needs;
        let kind_w = 0.3 + tr.sociability * 0.5 + tr.patience * 0.3 + warm.max(0.0) * 0.8 + mem.max(0.0);
        let bad_w = 0.1 + (1.0 - tr.patience) * 0.35 + tr.boldness * 0.15 + needs[0] * 0.3 + needs[3] * 0.4 + (-warm).max(0.0) + (-mem).max(0.0);
        if r.f32() * (kind_w + bad_w) < kind_w {
            self.remember(q, Who::Person(p), Deed::Kindness, KINDNESS.0, day);
            self.remember(p, Who::Person(q), Deed::Kindness, KINDNESS.1, day);
            return;
        }
        let owes = |w: &World, a: u32, b: u32| w.society.households[a as usize].purse.debts.iter().any(|d| d.to == super::lives::Creditor::Household(b));
        let trade = |w: &World, x: PersonId| w.society.lives[x as usize].job.craft().is_some() || matches!(w.society.lives[x as usize].job, super::jobs::Job::Merchant);
        let (deed, hurt) = if owes(self, hp, hq) || owes(self, hq, hp) {
            (Deed::DebtQuarrel, QUARREL)
        } else if v == 0 {
            (Deed::WorkQuarrel, QUARREL)
        } else if v == 1 && (trade(self, p) || trade(self, q)) {
            (Deed::TradeDispute, QUARREL)
        } else {
            (Deed::Slight, SLIGHT)
        };
        self.remember(q, Who::Person(p), deed, hurt.0, day);
        self.remember(p, Who::Person(q), deed, hurt.1, day);
    }

    /// Feelings cool with time; the patient hold on longer.
    fn cool_feelings(&mut self, town: SettlementId) {
        for h in self.town_households(town) {
            let folk = self.living(h);
            if folk.is_empty() {
                continue;
            }
            let pat = folk.iter().map(|&m| self.patience(m)).sum::<f32>() / folk.len() as f32;
            let fs = &mut self.society.households[h as usize].feelings;
            for f in fs.iter_mut() {
                let rate = FEEL_FADE * (1.5 - pat) * if f.stage >= 5 { FEUD_COOL } else { 1.0 };
                f.warmth *= 1.0 - rate;
                // A grudge that's cooled past the rung it reached steps back down.
                while f.stage > 0 && f.warmth > RUNGS[f.stage as usize - 1] * 0.5 {
                    f.stage -= 1;
                }
            }
            fs.retain(|f| f.warmth.abs() >= 0.03 || f.stage > 0);
        }
    }

    /// Grudges past their next rung may climb it.
    fn escalate(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t) as i32;
        for h in self.town_households(town) {
            for f in self.society.households[h as usize].feelings.clone() {
                if (f.other as usize) >= self.society.households.len() || day - f.since < ESCALATE_GAP {
                    continue;
                }
                let next = (f.stage as usize).min(4);
                if f.stage < 5 && f.warmth >= RUNGS[next] {
                    continue;
                }
                self.grudge_step(h, f.other, town, t);
            }
        }
    }

    /// The most aggrieved member of `h` against household `o`, and whom they blame.
    fn grudge_pair(&self, h: u32, o: u32, day: i32) -> Option<(PersonId, PersonId)> {
        let ours = self.living(h);
        let theirs = self.living(o);
        if ours.is_empty() || theirs.is_empty() {
            return None;
        }
        let mut best: Option<(f32, PersonId, PersonId)> = None;
        for &a in &ours {
            for &b in &theirs {
                let s = self.memory_of(a, Who::Person(b), day);
                if best.is_none_or(|x| s < x.0) {
                    best = Some((s, a, b));
                }
            }
        }
        best.map(|x| (x.1, x.2))
    }

    /// Household `h` may take the next rung against `o` (or act again in a feud).
    pub fn grudge_step(&mut self, h: u32, o: u32, town: SettlementId, t: f64) {
        let day = World::day_of(t) as i32;
        let Some((a, b)) = self.grudge_pair(h, o, day) else { return };
        let tr = self.people[a as usize].traits;
        let mut r = Rng::from_keys(&[self.seed, h as u64, o as u64, day as u64, 0x434C_494D]);
        if !r.chance(ESCALATE_CHANCE * (0.5 + tr.boldness * 0.5 + (1.0 - tr.patience) * 0.5)) {
            return;
        }
        let stage = self.feeling(h, o).map(|f| f.stage).unwrap_or(0);
        // In a feud, acts of harm and violence go on.
        let rung = if stage >= 5 { 3 + r.below(2) as u8 } else { stage + 1 };
        let honour = self.society.lives[a as usize].habits.honour;
        let open = r.chance(((honour - OPEN_AT) / OPEN_BAND + 0.5).clamp(0.0, 1.0));
        let deed = self.act(a, b, rung, open, town, t, &mut r);
        if let Some(f) = self.society.households[h as usize].feelings.iter_mut().find(|f| f.other == o) {
            f.stage = f.stage.max(rung);
            f.since = day;
        }
        if rung == 5 {
            // A feud is both households'.
            self.feel(o, h, 0.0, day);
            if let Some(f) = self.society.households[o as usize].feelings.iter_mut().find(|f| f.other == h) {
                f.stage = 5;
                f.warmth = f.warmth.min(RUNGS[4]);
                f.since = day;
            }
            self.society.towns[town as usize].gov.unrest += FEUD_UNREST;
        }
        let _ = deed;
    }

    /// `a` acts against `b` at a rung, openly or not. Returns what was done.
    fn act(&mut self, a: PersonId, b: PersonId, rung: u8, open: bool, town: SettlementId, t: f64, r: &mut Rng) -> Deed {
        let day = World::day_of(t) as i32;
        let (ha, hb) = (self.society.lives[a as usize].household.unwrap(), self.society.lives[b as usize].household.unwrap());
        let found = open || r.chance(FOUND_OUT);
        let deed = match (rung, open) {
            (1, _) => Deed::Avoid,
            (2, true) => {
                // A hearing, judged by the wronged side's custom.
                let mut jr = Rng::from_keys(&[self.seed, a as u64, b as u64, day as u64, 0x4845_4152]);
                let _lost = self.public_dispute(b, a, town, t, &mut jr);
                Deed::PublicDispute
            }
            (2, false) => Deed::Slander,
            (3, true) => {
                let mut jr = Rng::from_keys(&[self.seed, a as u64, b as u64, day as u64, 0x434C_4149]);
                match self.justice_for(town, a, &mut jr) {
                    Justice::Elders => {
                        // A claim on their labour: the hall makes them pay in work's worth.
                        if jr.chance(0.6) {
                            let take = (self.daily_cost(hb) * CLAIM_DAYS).min(self.society.households[hb as usize].purse.coin.max(0.0));
                            self.society.households[hb as usize].purse.coin -= take;
                            self.society.households[ha as usize].purse.coin += take;
                        }
                        Deed::Claim
                    }
                    Justice::Duel => {
                        self.npc_duel(b, a, t, rng::key(&[self.seed, a as u64, b as u64, day as u64, 0x4455_454C]));
                        Deed::Duel
                    }
                    Justice::Shunning => Deed::Shun,
                    Justice::Record => {
                        *self.records.entry(b).or_insert(0.0) += 1.0;
                        Deed::Record
                    }
                }
            }
            (3, false) => {
                let cost = self.daily_cost(hb);
                if self.mind(a).needs[0] > 0.3 || self.society.households[ha as usize].purse.coin < self.daily_cost(ha) * RESERVE_DAYS * 0.5 {
                    let take = (cost * THEFT_DAYS).min(self.society.households[hb as usize].purse.coin.max(0.0));
                    self.society.households[hb as usize].purse.coin -= take;
                    self.society.households[ha as usize].purse.coin += take;
                    Deed::Theft
                } else {
                    let lose = (cost * SABOTAGE_DAYS).min(self.society.households[hb as usize].purse.coin.max(0.0));
                    self.society.households[hb as usize].purse.coin -= lose;
                    Deed::Sabotage
                }
            }
            (4, true) => {
                self.npc_duel(b, a, t, rng::key(&[self.seed, a as u64, b as u64, day as u64, 0x4252_574C]));
                Deed::Brawl
            }
            (4, false) => {
                let pp = &mut self.people[b as usize];
                let base = pp.stats.clone();
                let mut hp = pp.wounds.hp_at(&base, t);
                for (k, x) in hp.iter_mut().enumerate() {
                    *x = x.min(base.max_hp(body::PARTS[k]) * BEATEN_TO);
                }
                pp.wounds.set(&base, &hp, t);
                Deed::Beating
            }
            _ => Deed::Feud,
        };
        if rung >= 2 && found {
            self.society.towns[town as usize].gov.wrongs += 1.0;
        }
        // The one it's done to remembers it — against whoever did it, if they know.
        let hurt = HURT[(rung as usize - 1).min(4)];
        if rung >= 2 {
            let about = if found { Who::Person(a) } else { Who::Someone };
            self.remember(b, about, deed, hurt, day);
            if !found {
                // The household still blames whoever they already suspect.
                self.feel(hb, ha, hurt * HOUSE_SHARE * 0.5, day);
            }
        }
        self.note(deed, Some(a), Some(b), town, t, !found);
        deed
    }
}
