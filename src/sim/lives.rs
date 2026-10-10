//! Lives with stakes: each household's purse, each person's work and needs.
//!
//! **Money is per household** (one purse; a stilt village living as one
//! household shares one). Each dawn the day before is settled: what its
//! members earned (their hours at work, at their trade's pay, scaled by how
//! well the town is doing), less food at the town's prices, tax and upkeep;
//! a comfortable household spends some of what's over at the market. A purse
//! that runs dry borrows — from a household it's warm toward, a well-off
//! neighbour, or the merchants — and pays it back when it can.
//!
//! **Work.** Everyone is working, injured, away, jobless, bound or retired.
//! Work is lost to a long injury, a long time away, a trade with nothing to
//! work on for days (its inputs gone, or its shelf full), or a revolt. The
//! jobless look for posts at dawn; labouring is the fallback where a town has
//! room for more hands, but not always.
//!
//! **Needs**, five slow numbers per person (0..1), drift each dawn toward
//! where their situation puts them: money worry, hunger, safety, grievance
//! (from wrongs they remember) and ambition (from their temper).
//!
//! Everything here is settled at dawn on the world's timeline, from the
//! dawn's own time, so it comes out the same however the world is stepped.

use serde::{Deserialize, Serialize};

use super::jobs::{Good, Job};
use super::person::PersonId;
use super::routine::Doing;
use super::settlement::SettlementId;
use super::world::World;

// ---- Dials ---------------------------------------------------------------

/// Food a person eats in a day, at the town's grain price (coin).
pub const FOOD_COST: f32 = 2.0;
/// A household's upkeep a day (repairs, tools, fuel).
pub const UPKEEP: f32 = 1.0;
/// Days of costs a household likes to keep in hand; past that, it spends
/// this share of the rest at the market each day.
pub const RESERVE_DAYS: f32 = 6.0;
pub const SPEND_SHARE: f32 = 0.5;
/// Share of what's over the reserve that goes to paying debts each day.
pub const REPAY_SHARE: f32 = 0.6;
/// No one lends past this many days of a household's costs.
pub const DEBT_LIMIT_DAYS: f32 = 25.0;
/// A neighbour lends only from what's past this many days of their own costs.
pub const LEND_RESERVE_DAYS: f32 = 10.0;
/// A household owed more than this many days of the debtor's costs, and not
/// being paid back, sours on them by `DEBT_SOUR` a day.
pub const SOUR_DEBT_DAYS: f32 = 2.0;
pub const DEBT_SOUR: f32 = 0.04;
/// Hurt this badly (share of all hit points lost), you can't work.
pub const INJURED_SHARE: f32 = 0.4;
/// Laid up this many dawns, your post goes to someone else; likewise away.
pub const LONG_INJURY: u8 = 3;
pub const LONG_AWAY: u8 = 4;
/// A trade with nothing to do this many dawns running lets its people go.
pub const IDLE_DAYS: u8 = 3;
/// From this age, someone without a post has retired.
pub const RETIRE_AGE: u8 = 70;
/// The jobless labour only while labourers are fewer than this share of a
/// community.
pub const LABOUR_ROOM: f32 = 0.35;
/// How far each need moves toward its mark each dawn.
pub const NEED_RATE: f32 = 0.35;

/// The needs, by index into `Mind::needs`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Need {
    Money,
    Hunger,
    Safety,
    Grievance,
    Ambition,
}
pub const NEEDS: [Need; 5] = [Need::Money, Need::Hunger, Need::Safety, Need::Grievance, Need::Ambition];

impl Need {
    pub fn name(self) -> &'static str {
        match self {
            Need::Money => "money worries",
            Need::Hunger => "hunger",
            Need::Safety => "fear for their safety",
            Need::Grievance => "a grievance",
            Need::Ambition => "ambition",
        }
    }
}

/// Where someone stands with work.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Work {
    #[default]
    Working,
    /// Hurt too badly to work.
    Injured,
    /// On the road.
    Away,
    /// No post, and no labouring either.
    Jobless,
    /// Bound to work for someone else.
    Bonded,
    Retired,
}

impl Work {
    pub fn name(self) -> &'static str {
        match self {
            Work::Working => "working",
            Work::Injured => "laid up",
            Work::Away => "away",
            Work::Jobless => "out of work",
            Work::Bonded => "bound to work",
            Work::Retired => "retired",
        }
    }
}

/// Everything about a person that changes with their life (kept small:
/// see the budget in the README).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Mind {
    pub work: Work,
    /// Dawns in a row laid up, away, or with nothing to work on.
    pub injured_days: u8,
    pub away_days: u8,
    pub idle_days: u8,
    /// Whether they had work in hand at their trade since the last dawn.
    pub had_work: bool,
    /// The post they lost, and why, if they've lost one.
    pub lost: Option<(Job, Loss)>,
    /// Money, hunger, safety, grievance, ambition: 0 (none) to 255
    /// (pressing); read them with `needs()`.
    pub need: [u8; 5],
    /// Who wronged or helped them (`memory.rs`).
    #[serde(default)]
    pub memories: super::few::Few<super::memory::Memory, { super::memory::MEMORY_CAP }>,
    /// World events they know of (`history.rs`).
    #[serde(default)]
    pub knows: super::few::Few<super::history::Known, { super::history::KNOWS_CAP }>,
}

impl Mind {
    /// Money, hunger, safety, grievance, ambition: 0 (none) to 1 (pressing).
    pub fn needs(&self) -> [f32; 5] {
        self.need.map(|x| x as f32 / 255.0)
    }

    pub fn set_need(&mut self, k: usize, v: f32) {
        self.need[k] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}

/// Why someone lost their post.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loss {
    Injury,
    Away,
    Idle,
    Revolt,
}

impl Loss {
    pub fn name(self) -> &'static str {
        match self {
            Loss::Injury => "while they were laid up",
            Loss::Away => "while they were away",
            Loss::Idle => "when there was no work to be had at it",
            Loss::Revolt => "when the town rose",
        }
    }
}

/// Who a debt is owed to.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Creditor {
    Household(u32),
    Merchants(SettlementId),
    Hall(SettlementId),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Debt {
    pub to: Creditor,
    pub amount: f32,
    /// They've stopped paying it.
    #[serde(default)]
    pub dodged: bool,
}

/// A household's money, as of the last dawn.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Purse {
    pub coin: f32,
    pub debts: Vec<Debt>,
    /// Yesterday's takings and costs (for the window).
    pub earned: f32,
    pub spent: f32,
}

impl Purse {
    pub fn debt(&self) -> f32 {
        self.debts.iter().map(|d| d.amount).sum()
    }
}

impl World {
    /// A person's work, needs, memories and knowledge. (People who came
    /// after the world was made — bandits — have an empty one.)
    pub fn mind(&self, p: PersonId) -> &super::lives::Mind {
        static EMPTY: Mind = Mind { work: Work::Working, injured_days: 0, away_days: 0, idle_days: 0, had_work: false, lost: None, need: [0; 5], memories: super::few::Few::new_with(super::memory::Memory::NONE), knows: super::few::Few::new_with(super::history::Known { id: 0 }) };
        self.society.minds.get(p as usize).unwrap_or(&EMPTY)
    }

    /// A household's costs for a day.
    pub fn daily_cost(&self, h: u32) -> f32 {
        let hh = &self.society.households[h as usize];
        let town = self.society.communities[hh.community as usize].town;
        let n = hh.members.iter().filter(|&&m| !self.people[m as usize].dead).count() as f32;
        let grain = self.price_factor_at(town, Good::Grain, self.society.rates_from).clamp(0.7, 1.6);
        let tl = &self.society.towns[town as usize];
        n * (FOOD_COST * grain + super::economy::TAX_PER_HEAD * (0.5 + 0.5 * tl.prosperity.min(1.0))) + UPKEEP
    }

    /// Why this person isn't working, if they aren't.
    pub fn why_not_working(&self, p: PersonId) -> Option<String> {
        let m = self.mind(p);
        let lost = m.lost.map(|(j, why)| format!("; lost their post as {} {}", j.name().to_lowercase(), why.name())).unwrap_or_default();
        match m.work {
            Work::Working => None,
            Work::Injured => Some(format!("laid up for {} days{lost}", m.injured_days)),
            Work::Away => Some(format!("away from home{lost}")),
            Work::Jobless => Some(format!("out of work{}", if lost.is_empty() { String::new() } else { lost })),
            Work::Bonded => Some("bound to work for another household".into()),
            Work::Retired => Some("retired".into()),
        }
    }

    /// What a household needs most: its members' strongest shared need.
    pub fn household_need(&self, h: u32) -> (Need, f32) {
        let hh = &self.society.households[h as usize];
        let mut sum = [0.0f32; 5];
        let mut n: f32 = 0.0;
        for &m in hh.members.iter().filter(|&&m| !self.people[m as usize].dead) {
            for (k, x) in self.mind(m).needs().iter().enumerate() {
                sum[k] += x;
            }
            n += 1.0;
        }
        let (k, v) = sum.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0))).map(|(k, v)| (k, *v)).unwrap_or((0, 0.0));
        (NEEDS[k], v / n.max(1.0))
    }

    /// A person's most pressing need.
    pub fn top_need(&self, p: PersonId) -> (Need, f32) {
        let n = self.mind(p).needs();
        let k = (0..5).max_by(|&a, &b| n[a].total_cmp(&n[b]).then(b.cmp(&a))).unwrap_or(0);
        (NEEDS[k], n[k])
    }

    // ---- At dawn ---------------------------------------------------------------

    /// The day before, for a town's people: work status, then every
    /// household's purse, then everyone's needs.
    pub(super) fn dawn_lives(&mut self, town: SettlementId, t: f64) {
        self.work_status(town, t);
        self.settle_purses(town, t);
        self.dawn_ties(town, t);
        self.settle_needs(town, t);
    }

    fn residents_alive(&self, town: SettlementId) -> Vec<PersonId> {
        self.settlements[town as usize].residents.iter().copied().filter(|&p| !self.people[p as usize].dead && self.people[p as usize].home == Some(town) && !self.people[p as usize].in_squad).collect()
    }

    /// Who's working, laid up, away or out of work — and posts lost to it.
    fn work_status(&mut self, town: SettlementId, t: f64) {
        let mut lost_any = false;
        for p in self.residents_alive(town) {
            let pp = &self.people[p as usize];
            let lost_hp: f32 = pp.wounds.lost_at(t).iter().sum();
            let max: f32 = super::body::PARTS.iter().map(|&part| pp.stats.max_hp(part)).sum();
            let hurt = super::body::knocked_out(&pp.wounds.hp_at(&pp.stats, t)) || lost_hp >= INJURED_SHARE * max;
            let away = self.busy_until[p as usize] > t;
            let bonded = self.is_bonded(p, t);
            let l = self.society.lives[p as usize];
            // The day just ended was theirs to rest (an irregular rhythm can
            // give three in a row): no work then isn't idleness.
            let idle = super::making::has_town_work(l.job) && !self.society.minds[p as usize].had_work && !hurt && !away;
            let rested = idle && self.day_plan(p, World::day_of(t) - 1).rest;
            let m = &mut self.society.minds[p as usize];
            m.injured_days = if hurt { m.injured_days.saturating_add(1) } else { 0 };
            m.away_days = if away { m.away_days.saturating_add(1) } else { 0 };
            // Only trades with town recipes can be idle: carpenters and masons
            // keep their posts until town recipes exist for them.
            m.idle_days = if rested { m.idle_days } else if idle { m.idle_days.saturating_add(1) } else { 0 };
            m.had_work = false;
            let loss = if !l.job.is_post() {
                None
            } else if m.injured_days >= LONG_INJURY {
                Some(Loss::Injury)
            } else if m.away_days >= LONG_AWAY && l.job != Job::Caravaner {
                Some(Loss::Away)
            } else if m.idle_days >= IDLE_DAYS {
                Some(Loss::Idle)
            } else {
                None
            };
            if let Some(why) = loss {
                m.lost = Some((l.job, why));
                m.idle_days = 0;
                let lf = &mut self.society.lives[p as usize];
                lf.job = Job::None;
                lf.place = None;
                lost_any = true;
            }
            let l = self.society.lives[p as usize];
            let m = &mut self.society.minds[p as usize];
            m.work = if bonded {
                Work::Bonded
            } else if hurt {
                Work::Injured
            } else if away {
                Work::Away
            } else if matches!(l.job, Job::None | Job::Drifter) && l.age >= RETIRE_AGE {
                Work::Retired
            } else if matches!(l.job, Job::None | Job::Drifter) {
                Work::Jobless
            } else {
                Work::Working
            };
        }
        let _ = lost_any;
    }

    /// After posts are filled at dawn: anyone still without work labours if
    /// the town has room for more hands, else stays out of work.
    pub(super) fn place_jobless(&mut self, town: SettlementId) {
        let tl = &self.society.towns[town as usize];
        for ci in std::iter::once(tl.shore).chain(tl.stilts) {
            let members: Vec<PersonId> = self.members_of(ci).collect();
            let room = (members.len() as f32 * LABOUR_ROOM) as usize;
            let mut labourers = members.iter().filter(|&&p| self.society.lives[p as usize].job == Job::Labourer).count();
            let places = self.labour_places(ci);
            for &p in &members {
                let l = self.society.lives[p as usize];
                let m = &self.society.minds[p as usize];
                if l.job != Job::None || m.work == Work::Bonded || m.work == Work::Injured || l.age >= RETIRE_AGE || self.people[p as usize].in_squad {
                    continue;
                }
                if labourers < room && !places.is_empty() {
                    let lf = &mut self.society.lives[p as usize];
                    lf.job = Job::Labourer;
                    lf.place = Some(places[labourers % places.len()]);
                    labourers += 1;
                    self.society.minds[p as usize].work = Work::Working;
                }
            }
        }
    }

    /// Every household of the town: yesterday's earnings and costs, spending,
    /// borrowing and paying back.
    fn settle_purses(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t) - 1;
        let prosperity = self.society.towns[town as usize].prosperity.min(1.2);
        let scale = 0.5 + 0.5 * prosperity;
        // The watch's pay is only as good as the treasury.
        let watch_paid = if self.society.towns[town as usize].owed > 0.0 { 0.4 } else { 1.0 };
        let hhs: Vec<u32> = (0..self.society.households.len() as u32).filter(|&h| self.society.communities[self.society.households[h as usize].community as usize].town == town).collect();
        // Earnings, by household (a bound worker's go to their holder's).
        let mut earned: Vec<(u32, f32)> = Vec::new();
        for &h in &hhs {
            for &p in &self.society.households[h as usize].members.clone() {
                let pp = &self.people[p as usize];
                if pp.dead || pp.in_squad {
                    continue;
                }
                let m = self.mind(p);
                if !matches!(m.work, Work::Working | Work::Bonded) {
                    continue;
                }
                let job = self.society.lives[p as usize].job;
                let hours = self.day_plan(p, day).hours_of(Doing::Work, 0.0, 24.0) + self.day_plan(p, day).hours_of(Doing::Run, 0.0, 24.0);
                let pay = job.pay() * hours * scale * if job == Job::Guard { watch_paid } else { 1.0 };
                let to = match self.bond_at(p, t).and_then(|b| b.holder).and_then(|hd| self.society.lives[hd as usize].household) {
                    Some(hh) if m.work == Work::Bonded => hh,
                    _ => h,
                };
                earned.push((to, pay));
            }
        }
        // A bound worker's pay to a holder in another town goes straight there.
        for &(h, pay) in earned.iter().filter(|e| !hhs.contains(&e.0)) {
            self.society.households[h as usize].purse.coin += pay;
        }
        for &h in &hhs {
            let income: f32 = earned.iter().filter(|e| e.0 == h).map(|e| e.1).sum();
            let cost = self.daily_cost(h);
            let alive = self.society.households[h as usize].members.iter().any(|&m| !self.people[m as usize].dead);
            if !alive {
                continue;
            }
            let p = &mut self.society.households[h as usize].purse;
            p.earned = income;
            p.coin += income - cost;
            p.spent = cost;
        }
        // Spending and paying back what's over the reserve; borrowing what's short.
        let mut to_merchants = 0.0;
        for &h in &hhs {
            let cost = self.daily_cost(h);
            let reserve = cost * RESERVE_DAYS;
            let coin = self.society.households[h as usize].purse.coin;
            if coin > reserve {
                let over = coin - reserve;
                let owed: f32 = self.society.households[h as usize].purse.debts.iter().filter(|d| !d.dodged).map(|d| d.amount).sum();
                let repay = (over * REPAY_SHARE).min(owed);
                self.repay(h, repay, town, t);
                let left = self.society.households[h as usize].purse.coin - reserve;
                if left > 0.0 {
                    let spend = left * SPEND_SHARE;
                    self.society.households[h as usize].purse.coin -= spend;
                    self.society.households[h as usize].purse.spent += spend;
                    to_merchants += spend;
                }
            } else if coin < 0.0 {
                self.borrow(h, -coin, town, t);
            }
        }
        // A neighbour owed money and not being paid sours on the debtor.
        let day = World::day_of(t) as i32;
        for &h in &hhs {
            let cost = self.daily_cost(h);
            if self.society.households[h as usize].purse.coin > cost * RESERVE_DAYS {
                continue;
            }
            for d in self.society.households[h as usize].purse.debts.clone() {
                let Creditor::Household(o) = d.to else { continue };
                if d.amount < cost * SOUR_DEBT_DAYS {
                    continue;
                }
                let lender = self.society.households.get(o as usize).and_then(|x| x.members.first().copied());
                let debtor = self.society.households[h as usize].members.first().copied();
                if let (Some(a), Some(b)) = (lender, debtor) {
                    self.remember(a, super::memory::Who::Person(b), super::history::Deed::DebtQuarrel, -DEBT_SOUR, day);
                }
            }
        }
        // What's spent at market goes to the merchants.
        let tl = &mut self.society.towns[town as usize];
        let now = super::economy::purse_at(tl, t);
        tl.purse = (now + to_merchants).min(tl.purse_cap.max(now));
        tl.purse_at = t;
    }

    /// Borrow `amount`: from a household they're warm toward, the best-off
    /// neighbour, else the merchants. Past the limit, no one lends: they go
    /// without.
    fn borrow(&mut self, h: u32, amount: f32, town: SettlementId, t: f64) {
        let cost = self.daily_cost(h);
        let owed = self.society.households[h as usize].purse.debt();
        self.society.households[h as usize].purse.coin = 0.0;
        if owed + amount > cost * DEBT_LIMIT_DAYS {
            return;
        }
        let community = self.society.households[h as usize].community;
        let spare = |w: &World, o: u32| w.society.households[o as usize].purse.coin - w.daily_cost(o) * LEND_RESERVE_DAYS;
        let warm: Vec<u32> = self.society.households[h as usize].feelings.iter().filter(|f| f.warmth > 0.3).map(|f| f.other).collect();
        let lender = warm
            .into_iter()
            .filter(|&o| (o as usize) < self.society.households.len() && spare(self, o) >= amount)
            .next()
            .or_else(|| {
                (0..self.society.households.len() as u32)
                    .filter(|&o| o != h && self.society.households[o as usize].community == community && spare(self, o) >= amount)
                    .max_by(|&a, &b| spare(self, a).total_cmp(&spare(self, b)).then(b.cmp(&a)))
            });
        let to = match lender {
            Some(o) => {
                self.society.households[o as usize].purse.coin -= amount;
                // Remembered kindly, the first time.
                if !self.society.households[h as usize].purse.debts.iter().any(|d| d.to == Creditor::Household(o)) {
                    let day = World::day_of(t) as i32;
                    if let (Some(a), Some(b)) = (self.society.households[h as usize].members.first().copied(), self.society.households[o as usize].members.first().copied()) {
                        self.remember(a, super::memory::Who::Person(b), super::history::Deed::Loan, super::memory::LOAN_THANKS, day);
                    }
                }
                Creditor::Household(o)
            }
            None => {
                let tl = &mut self.society.towns[town as usize];
                let now = super::economy::purse_at(tl, t);
                tl.purse = (now - amount).max(0.0);
                tl.purse_at = t;
                Creditor::Merchants(town)
            }
        };
        let debts = &mut self.society.households[h as usize].purse.debts;
        match debts.iter_mut().find(|d| d.to == to) {
            Some(d) => d.amount += amount,
            None => debts.push(Debt { to, amount, dodged: false }),
        }
    }

    /// Pay back `amount` of a household's debts, oldest first.
    fn repay(&mut self, h: u32, mut amount: f32, town: SettlementId, t: f64) {
        while amount > 0.01 {
            // (A debt they've stopped paying isn't paid.)
            let Some(k) = self.society.households[h as usize].purse.debts.iter().position(|d| !d.dodged) else { break };
            let d = self.society.households[h as usize].purse.debts[k];
            let pay = amount.min(d.amount);
            amount -= pay;
            self.society.households[h as usize].purse.coin -= pay;
            match d.to {
                Creditor::Household(o) => {
                    if let Some(oh) = self.society.households.get_mut(o as usize) {
                        oh.purse.coin += pay;
                    }
                }
                Creditor::Merchants(tn) | Creditor::Hall(tn) => {
                    let tl = &mut self.society.towns[tn as usize];
                    if matches!(d.to, Creditor::Hall(_)) {
                        tl.treasury += pay;
                    } else {
                        let now = super::economy::purse_at(tl, t);
                        tl.purse = (now + pay).min(tl.purse_cap.max(now));
                        tl.purse_at = t;
                    }
                }
            }
            let debts = &mut self.society.households[h as usize].purse.debts;
            debts[k].amount -= pay;
            if debts[k].amount <= 0.01 {
                debts.remove(k);
            }
        }
        let _ = town;
    }

    /// Each person's needs drift toward where their life puts them.
    fn settle_needs(&mut self, town: SettlementId, t: f64) {
        let gov = self.government(town);
        let town_fear = (gov.unrest / 100.0 + gov.wrongs * 0.1).min(1.0);
        let day = World::day_of(t) as i32;
        for p in self.residents_alive(town) {
            let l = self.society.lives[p as usize];
            let fed = l.community.map(|c| self.society.communities[c as usize].food.overall).unwrap_or(1.0);
            let (money, poor) = match l.household {
                Some(h) => {
                    let cost = self.daily_cost(h).max(0.1);
                    let pu = &self.society.households[h as usize].purse;
                    let short = (1.0 - pu.coin / (cost * RESERVE_DAYS)).max(0.0);
                    let owing = pu.debt() / (cost * DEBT_LIMIT_DAYS);
                    ((short * 0.5 + owing).min(1.0), pu.coin <= 0.0 && pu.debt() > 0.0)
                }
                None => (0.5, true),
            };
            // Hungry households eat last.
            let hunger = ((1.0 - fed) * if poor { 2.0 } else { 0.6 } + if poor { 0.3 } else { 0.0 }).min(1.0);
            let safety = (town_fear + self.known_danger(p, day)).min(1.0);
            let grievance = self.grievance_of(p, day);
            let tr = self.people[p as usize].traits;
            let placed = l.job.pay() >= 1.8 || self.holds_office(p);
            let ambition = (tr.boldness * 0.6 + (1.0 - tr.patience) * 0.2 + tr.sociability * 0.15 - if placed { 0.3 } else { 0.0 }).clamp(0.0, 1.0);
            let target = [money, hunger, safety, grievance, ambition];
            let m = &mut self.society.minds[p as usize];
            let now = m.needs();
            for k in 0..5 {
                m.set_need(k, now[k] + (target[k] - now[k]) * NEED_RATE);
            }
        }
    }

    /// Does this person hold any office in their town?
    pub fn holds_office(&self, p: PersonId) -> bool {
        let Some(town) = self.people[p as usize].home else { return false };
        let g = self.government(town);
        g.arbiter == Some(p) || g.councillors.contains(&p) || g.chambers.iter().any(|c| c.holders.contains(&p) || c.administrators.contains(&p))
    }
}
