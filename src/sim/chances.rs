//! Opportunities: someone wants something done (Part 5, Stage 4).
//!
//! One structure for every job the world can offer: what kind, who's asking
//! (a person, a household, or a town's ring), against whom, where, for what
//! reward (coin from the giver's purse, or a favour owed), by when, whether
//! it's lawful and whether it's said openly. Opportunities are made at dawn by
//! storylines (`stories.rs`) whose people can't settle a thing themselves,
//! will ask an outsider, and can pay or owe. They're never on a board — you
//! hear of them by talking — except that lawful, open ones are also posted at
//! the hall in towns that keep written records.
//!
//! Taking one puts it in the squad's journal (`quests.rs`). Guard work is a
//! **contract**: the squad member assigned stands at the place in its hours
//! (sent there as the shift begins, or whenever idle), is paid each dawn for
//! a day they turned up, and deals with what happens there (a thief who tries
//! it is caught). A squad member can also take a town post that's going.
//!
//! Most talk-done jobs are finished by **pressing** someone: asking, paying
//! or threatening the one who holds the stolen thing, knows who did it, owes
//! the debt, needs scaring, or ran from their bond. Finishing, failing or
//! being found out writes events and memories like anything else.

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::group::GroupId;
use super::history::Deed;
use super::items::{self, ItemId};
use super::jobs::Job;
use super::memory::Who;
use super::person::PersonId;
use super::quests::{Quest, QuestKind, Stage};
use super::rng::Rng;
use super::settlement::SettlementId;
use super::world::{World, DAY, HOUR};

// ---- Dials -----------------------------------------------------------------

/// Days an opportunity stays open before it lapses, by kind (see `Chance::days`).
pub const OPEN_DAYS: f64 = 6.0;
/// Days to do a job once it's taken; and how long a job done but not
/// reported back stays open before it's let go.
pub const JOB_DAYS: f64 = 14.0;
pub const REPORT_DAYS: f64 = 10.0;
/// Guard work: days, and the day's pay as a share of the giver's daily costs.
pub const GUARD_DAYS: u8 = 4;
pub const GUARD_PAY: f32 = 0.6;
/// A day counts as worked if they were at the place this share of its hours.
pub const PRESENT_SHARE: f32 = 0.5;
/// Days a contract can be missed before it's called off.
pub const MISSED_LIMIT: u8 = 2;
/// How near the place counts as being there, metres.
pub const AT_POST: f32 = 14.0;
/// A post's hours (from, to).
pub const POST_HOURS: (f32, f32) = (8.0, 17.0);
/// A hand hired by the day is paid this many times a townsperson's own rate.
pub const HIRED_RATE: f32 = 3.0;
/// Rewards: share of what's at stake (a stolen thing's worth, a debt), and
/// the least.
pub const REWARD_SHARE: f32 = 0.4;
pub const REWARD_MIN: u16 = 15;
/// Payment for scaring or killing someone, in days of the giver's costs.
pub const INTIMIDATE_DAYS: f32 = 4.0;
pub const KILL_DAYS: f32 = 12.0;
/// Most opportunities open in a town at once.
pub const OPEN_CAP: usize = 6;
/// How much a job done or failed is remembered by the giver.
pub const THANKS: f32 = 0.5;
pub const LET_DOWN: f32 = -0.4;
/// Standing a job done earns.
pub const JOB_STANDING: f32 = 5.0;

// ---- State -----------------------------------------------------------------

/// What's wanted.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Chance {
    /// Stand guard at a workplace for some days.
    Guard,
    /// Get a stolen thing back.
    Recover,
    /// Find out who did something.
    FindOut,
    /// Get a debt paid.
    CollectDebt,
    /// See someone or a cargo safely to another town (not offered yet).
    Escort,
    /// Frighten or rough someone up.
    Intimidate,
    /// Kill someone, quietly.
    Kill,
    /// Fight someone's duel for them (not offered yet).
    Champion,
    /// Bring back someone who ran from their bond.
    Runaway,
    ClearCamp { camp: GroupId, at: V2 },
    Fetch { item: ItemId, count: u16 },
    Deliver { to: PersonId },
}

impl Chance {
    pub fn name(self) -> &'static str {
        match self {
            Chance::Guard => "guard work",
            Chance::Recover => "get back what was stolen",
            Chance::FindOut => "find out who did it",
            Chance::CollectDebt => "collect a debt",
            Chance::Escort => "escort",
            Chance::Intimidate => "put a scare into someone",
            Chance::Kill => "a quiet killing",
            Chance::Champion => "stand as champion",
            Chance::Runaway => "bring back a runaway",
            Chance::ClearCamp { .. } => "clear a bandit camp",
            Chance::Fetch { .. } => "fetch goods",
            Chance::Deliver { .. } => "deliver a letter",
        }
    }

    pub fn tag(self) -> u8 {
        match self {
            Chance::Guard => 0,
            Chance::Recover => 1,
            Chance::FindOut => 2,
            Chance::CollectDebt => 3,
            Chance::Escort => 4,
            Chance::Intimidate => 5,
            Chance::Kill => 6,
            Chance::Champion => 7,
            Chance::Runaway => 8,
            Chance::ClearCamp { .. } => 9,
            Chance::Fetch { .. } => 10,
            Chance::Deliver { .. } => 11,
        }
    }

    /// Lawful work?
    pub fn legal(self) -> bool {
        !matches!(self, Chance::Intimidate | Chance::Kill)
    }
}

/// Who wants it done.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giver {
    Person(PersonId),
    Household(u32),
    Ring(SettlementId),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OppState {
    Open,
    Taken,
    Done,
    Failed,
    Lapsed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Opportunity {
    pub id: u32,
    pub kind: Chance,
    pub giver: Giver,
    /// Who to talk to about it.
    pub asker: PersonId,
    pub town: SettlementId,
    /// Who it's against or about.
    pub target: Option<PersonId>,
    /// Where (a workplace of the town).
    pub place: Option<u16>,
    /// Coin; 0 with `favour` means a favour owed instead.
    pub reward: u16,
    pub favour: bool,
    pub deadline: f64,
    pub legal: bool,
    /// Said openly (else only to those they trust, and never posted).
    pub open: bool,
    pub state: OppState,
    pub taken_by: Option<PersonId>,
    /// The event it's about, the thing at stake, and how much.
    pub event: Option<u32>,
    pub item: Option<ItemId>,
    pub amount: f32,
    /// The squad has heard of it.
    pub known: bool,
    /// The job is done; tell the asker.
    pub done: bool,
}

/// A squad member's paid work in town: guarding a place, or a post.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Contract {
    pub member: PersonId,
    pub town: SettlementId,
    pub place: u16,
    /// The opportunity it's for (guard work), or none for a post.
    pub opp: Option<u32>,
    /// A post taken up: the job.
    pub post: Option<Job>,
    /// Hours of the day (from, to).
    pub hours: (f32, f32),
    pub pay: f32,
    pub until: f64,
    /// Seconds at the place today, and days missed so far.
    pub present: f64,
    pub missed: u8,
    pub days_paid: u8,
    /// Whether the member was sent there at this shift's start.
    pub sent_day: i64,
    /// The first day of work.
    pub first_day: i64,
}

/// Something stolen, and where it is now.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Stolen {
    pub item: ItemId,
    pub piece: Option<super::materials::Piece>,
    pub from: Who,
    pub thief: PersonId,
    pub town: SettlementId,
    pub event: u32,
    pub day: i32,
    /// Sold on through the ring (the ring knows whose it was).
    pub fenced: bool,
    /// Back with its owner or in the squad's hands.
    pub recovered: bool,
}

/// How a squad member presses someone.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    Ask,
    Pay,
    Threaten,
}

impl World {
    // ---- Making them ---------------------------------------------------------

    /// Make an opportunity (none if the town has too many open). Returns its id.
    pub fn post_opp(&mut self, mut o: Opportunity, t: f64) -> Option<u32> {
        let open = self.society.opps.iter().filter(|x| x.town == o.town && x.state == OppState::Open).count();
        if open >= OPEN_CAP {
            return None;
        }
        o.id = self.society.next_opp;
        self.society.next_opp += 1;
        if o.deadline <= t {
            o.deadline = t + OPEN_DAYS * DAY;
        }
        o.legal = o.kind.legal();
        let id = o.id;
        self.society.opps.push(o);
        Some(id)
    }

    /// A blank opportunity from `asker` in `town`.
    pub fn opp(&self, kind: Chance, asker: PersonId, town: SettlementId) -> Opportunity {
        Opportunity { id: 0, kind, giver: Giver::Person(asker), asker, town, target: None, place: None, reward: 0, favour: false, deadline: 0.0, legal: true, open: true, state: OppState::Open, taken_by: None, event: None, item: None, amount: 0.0, known: false, done: false }
    }

    /// Can and will this person pay `want`? Coin if the household can spare
    /// it; a favour if they're honourable; else they can't ask.
    pub(super) fn can_reward(&self, p: PersonId, want: f32) -> Option<(u16, bool)> {
        let h = self.society.lives.get(p as usize)?.household?;
        let spare = self.society.households[h as usize].purse.coin - self.daily_cost(h) * 2.0;
        if spare >= want {
            Some((want.max(REWARD_MIN as f32) as u16, false))
        } else if self.society.lives[p as usize].habits.honour > 0.6 {
            Some((0, true))
        } else {
            None
        }
    }

    // ---- Looking them up -----------------------------------------------------

    pub fn opportunity(&self, id: u32) -> Option<&Opportunity> {
        self.society.opps.iter().find(|o| o.id == id)
    }

    fn opp_mut(&mut self, id: u32) -> Option<&mut Opportunity> {
        self.society.opps.iter_mut().find(|o| o.id == id)
    }

    /// The open opportunity this person would tell the squad about, if any.
    pub fn open_offer(&self, npc: PersonId) -> Option<&Opportunity> {
        self.society.opps.iter().find(|o| o.asker == npc && o.state == OppState::Open && o.deadline > self.time)
    }

    /// Lawful, open work posted at the hall, where the town keeps records.
    pub fn hall_board(&self, town: SettlementId) -> Vec<u32> {
        if !self.keeps_records(town) {
            return Vec::new();
        }
        self.society.opps.iter().filter(|o| o.town == town && o.state == OppState::Open && o.legal && o.open && o.deadline > self.time).map(|o| o.id).collect()
    }

    /// Does this town keep written records (a record custom, recorded bonds,
    /// or an arbiter)?
    pub fn keeps_records(&self, town: SettlementId) -> bool {
        let tl = &self.society.towns[town as usize];
        let c = &self.society.communities[tl.shore as usize].customs;
        c.justice == super::culture::Justice::Record || c.slavery == super::culture::Slavery::Recorded || tl.gov.arbiter.is_some()
    }

    /// Opportunities in a town the squad has heard of.
    pub fn known_opps(&self, town: SettlementId) -> Vec<&Opportunity> {
        self.society.opps.iter().filter(|o| o.town == town && o.known && matches!(o.state, OppState::Open | OppState::Taken)).collect()
    }

    /// One line on an opportunity.
    pub fn opp_line(&self, o: &Opportunity) -> String {
        let name = |p: Option<PersonId>| p.map(|p| self.name_of(p)).unwrap_or_else(|| "someone".into());
        let pay = if o.favour { "a favour owed".to_string() } else { format!("{} coin", o.reward) };
        let what = match o.kind {
            Chance::Guard => format!("guard {}'s place for {} days, {:.0} coin a day", name(Some(o.asker)), GUARD_DAYS, o.amount),
            Chance::Recover => format!("get back {}'s {}", name(Some(o.asker)), o.item.map(|i| items::item(i).name.to_lowercase()).unwrap_or_default()),
            Chance::FindOut => format!("find out who wronged {}", name(Some(o.asker))),
            Chance::CollectDebt => format!("collect {:.0} coin owed by {}", o.amount, name(o.target)),
            Chance::Intimidate => format!("put a scare into {}", name(o.target)),
            Chance::Kill => format!("see that {} doesn't wake up", name(o.target)),
            Chance::Runaway => format!("bring back {}, who ran from their bond", name(o.target)),
            Chance::ClearCamp { .. } => "break the bandit camp nearby".into(),
            Chance::Fetch { item, count } => format!("bring {count} × {}", items::item(item).name.to_lowercase()),
            Chance::Deliver { to } => format!("take a letter to {}", name(Some(to))),
            k => k.name().into(),
        };
        format!("{what} — {pay}{}", if o.legal { "" } else { " (unlawful)" })
    }

    // ---- Taking them ---------------------------------------------------------

    /// Take an opportunity for the squad, `who` doing it. Guard work starts a
    /// contract. Returns the journal entry.
    pub fn take_opportunity(&mut self, id: u32, who: PersonId) -> Option<u32> {
        let o = self.opportunity(id)?.clone();
        if o.state != OppState::Open || !self.people[who as usize].in_squad {
            return None;
        }
        let kind = match o.kind {
            Chance::ClearCamp { camp, at } => QuestKind::ClearCamp { camp, at },
            Chance::Fetch { item, count } => QuestKind::Fetch { item, count },
            Chance::Deliver { to } => {
                self.people[to as usize].ensure_detail();
                if let Some(d) = self.people[who as usize].detail.as_mut() {
                    d.gear.add(items::id("sealed_letter"), 1);
                }
                QuestKind::Deliver { to }
            }
            _ => QuestKind::Job { opp: id },
        };
        if o.kind == Chance::Guard {
            let place = o.place?;
            let day = World::day_of(self.time);
            let hours = self.day_plan(o.asker, day).work.unwrap_or((8.0, 18.0));
            // From tomorrow (paid at each dawn after); the last pay is the
            // dawn after the last day.
            let first_day = day + 1;
            let until = (first_day + GUARD_DAYS as i64) as f64 * DAY + super::society::DAWN as f64 * HOUR;
            self.society.contracts.push(Contract { member: who, town: o.town, place, opp: Some(id), post: None, hours, pay: o.amount, until, present: 0.0, missed: 0, days_paid: 0, sent_day: -1, first_day });
        }
        let qid = self.quests.len() as u32;
        self.quests.push(Quest { id: qid, giver: o.asker, kind, stage: Stage::Active, coin: o.reward, bonus: None, opp: Some(id) });
        let t = self.time;
        let o = self.opp_mut(id)?;
        o.state = OppState::Taken;
        o.taken_by = Some(who);
        o.known = true;
        // The clock for the work starts now.
        o.deadline = t + JOB_DAYS * DAY;
        Some(qid)
    }

    /// Posts going in a town a squad member could fill: (job, workplace).
    pub fn vacant_posts(&self, town: SettlementId) -> Vec<(Job, u16)> {
        let ci = self.society.towns[town as usize].shore;
        let n = self.members_of(ci).count();
        let mut out = Vec::new();
        for job in Self::POST_ORDER {
            let places = self.places_for(ci, job);
            if places.is_empty() || self.society.contracts.iter().any(|c| c.town == town && c.post == Some(job)) {
                continue;
            }
            let have = self.members_of(ci).filter(|&p| self.society.lives[p as usize].job == job).count();
            if have < self.posts_for(ci, job, n) {
                out.push((job, places[0]));
            }
        }
        if out.is_empty() {
            if let Some(&pl) = self.labour_places(ci).first() {
                out.push((Job::Labourer, pl));
            }
        }
        out
    }

    /// A squad member takes up a post going in town, paid by the day from the
    /// treasury (or the merchants, for labour).
    pub fn take_post_work(&mut self, who: PersonId, town: SettlementId, job: Job, place: u16) -> bool {
        if !self.people[who as usize].in_squad || self.society.contracts.iter().any(|c| c.member == who) {
            return false;
        }
        let day = World::day_of(self.time);
        let hours = POST_HOURS;
        let pay = self.post_wage(job) as f32;
        self.society.contracts.push(Contract { member: who, town, place, opp: None, post: Some(job), hours, pay, until: f64::INFINITY, present: 0.0, missed: 0, days_paid: 0, sent_day: day - 1, first_day: day + 1 });
        true
    }

    /// What a squad member is paid for a day at a post (named before it's
    /// taken). A hand hired by the day from outside is paid for the work, not
    /// as kin sharing the household pot: a few times a townsperson's own
    /// hourly rate, so a day's work stands beside a few hours at a woodlot.
    pub fn post_wage(&self, job: Job) -> u16 {
        (job.pay().max(0.7) * (POST_HOURS.1 - POST_HOURS.0) * HIRED_RATE).round() as u16
    }

    /// The squad's town work, a line each, naming the worker: where, the
    /// hours, the wage and how it's going.
    pub fn work_lines(&self) -> Vec<String> {
        let day = World::day_of(self.time);
        let h = (self.time.rem_euclid(DAY) / HOUR) as f32;
        self.society
            .contracts
            .iter()
            .filter(|c| self.squad.index(c.member).is_some())
            .map(|c| {
                let name = self.name_of(c.member);
                let what = c.post.map(|j| format!("works as {}", j.name().to_lowercase())).unwrap_or_else(|| "stands guard".into());
                let place = self.society.towns[c.town as usize].places.get(c.place as usize).map(|p| p.kind.name().to_lowercase()).unwrap_or_else(|| "town".into());
                let now = match self.on_shift(c.member) {
                    Some(true) => "at work now".to_string(),
                    Some(false) => "should be there now".to_string(),
                    None if day < c.first_day => "starts tomorrow".to_string(),
                    None if h < c.hours.0 => format!("shift today from {:02.0}:00", c.hours.0),
                    None => "shift over for today".to_string(),
                };
                format!("{name} {what} at the {place} in {}, {:02.0}:00–{:02.0}:00, {:.0} coin a day ({} paid; {now}).", self.settlements[c.town as usize].name, c.hours.0, c.hours.1, c.pay, if c.days_paid == 1 { "a day".to_string() } else { format!("{} days", c.days_paid) })
            })
            .collect()
    }

    /// In a squad member's working hours: are they at the place (true) or
    /// not there yet (false)? None outside their hours, or with no work.
    pub fn on_shift(&self, who: PersonId) -> Option<bool> {
        let c = self.contract_of(who)?;
        let h = (self.time.rem_euclid(DAY) / HOUR) as f32;
        if World::day_of(self.time) < c.first_day || h < c.hours.0 || h >= c.hours.1 || self.time >= c.until {
            return None;
        }
        let k = self.squad.index(who)?;
        Some(self.member_pos(k).dist(self.contract_pos(c)) <= AT_POST)
    }

    /// Give up a post or contract.
    pub fn quit_work(&mut self, who: PersonId) {
        let t = self.time;
        let ended: Vec<Contract> = self.society.contracts.iter().filter(|c| c.member == who).cloned().collect();
        self.society.contracts.retain(|c| c.member != who);
        for c in ended {
            if let Some(id) = c.opp {
                self.fail_opp(id, t);
            }
        }
    }

    /// The contract a squad member is working, if any.
    pub fn contract_of(&self, who: PersonId) -> Option<&Contract> {
        self.society.contracts.iter().find(|c| c.member == who)
    }

    /// Where a contract's place is.
    pub fn contract_pos(&self, c: &Contract) -> V2 {
        // (A town laid out anew may have lost the place: then the town's middle.)
        self.society.towns[c.town as usize].places.get(c.place as usize).map(|p| p.pos).unwrap_or(self.settlements[c.town as usize].pos)
    }

    /// Is a squad member on a contract at its place right now?
    pub fn on_watch(&self, town: SettlementId, place: u16, t: f64) -> Option<PersonId> {
        let h = (t.rem_euclid(DAY) / HOUR) as f32;
        self.society.contracts.iter().find(|c| c.town == town && c.place == place && h >= c.hours.0 && h < c.hours.1 && self.squad.index(c.member).is_some_and(|k| self.member_pos(k).dist(self.contract_pos(c)) <= AT_POST)).map(|c| c.member)
    }

    /// Each step (the squad's own time): members on contract go to their place
    /// as the shift begins, or whenever they're standing idle in its hours;
    /// time there is counted.
    pub(super) fn work_contracts(&mut self, dt: f64) {
        if self.society.contracts.is_empty() {
            return;
        }
        let day = World::day_of(self.time);
        let h = (self.time.rem_euclid(DAY) / HOUR) as f32;
        for i in 0..self.society.contracts.len() {
            let c = self.society.contracts[i].clone();
            let Some(k) = self.squad.index(c.member) else { continue };
            if day < c.first_day || h < c.hours.0 || h >= c.hours.1 || self.time >= c.until || self.want_talk.is_some_and(|w| w.0 == c.member) {
                continue;
            }
            let at = self.contract_pos(&c);
            let here = self.member_pos(k);
            if here.dist(at) <= AT_POST {
                self.society.contracts[i].present += dt;
            } else if c.sent_day != day || (self.squad.route[k].is_empty() && self.squad.goal[k] == self.squad.at[k]) {
                // (Up from their bedroll, if they'd lain down.)
                self.send(c.member, at);
                self.society.contracts[i].sent_day = day;
            }
        }
    }

    /// At dawn: pay for yesterday's work, or count it missed; contracts end.
    pub(super) fn dawn_contracts(&mut self, t: f64) {
        let coin = items::id("coin");
        let mut i = 0;
        while i < self.society.contracts.len() {
            let c = self.society.contracts[i].clone();
            if self.society.towns[c.town as usize].places.get(c.place as usize).is_none() {
                // The town was laid out anew and the place is gone.
                self.society.contracts.remove(i);
                if let Some(id) = c.opp {
                    self.fail_opp(id, t);
                }
                continue;
            }
            let need = (c.hours.1 - c.hours.0) as f64 * HOUR * PRESENT_SHARE as f64;
            // The day being settled is yesterday's.
            let began = World::day_of(t) - 1 >= c.first_day;
            let mut ended = false;
            if !began {
                // Not started yet.
            } else if c.present >= need {
                // Paid from the giver's purse (guard work) or the hall (a post).
                let pay = match c.opp.and_then(|id| self.opportunity(id)).and_then(|o| self.society.lives.get(o.asker as usize)).and_then(|l| l.household) {
                    Some(hh) => {
                        let p = c.pay.min(self.society.households[hh as usize].purse.coin.max(0.0));
                        self.society.households[hh as usize].purse.coin -= p;
                        p
                    }
                    None => {
                        let tl = &mut self.society.towns[c.town as usize];
                        let p = c.pay.min(tl.treasury.max(0.0));
                        tl.treasury -= p;
                        p
                    }
                };
                if let Some(d) = self.people[c.member as usize].detail.as_mut() {
                    d.gear.add(coin, pay.round() as u16);
                }
                self.people[c.member as usize].recompute_might();
                self.society.contracts[i].days_paid = self.society.contracts[i].days_paid.saturating_add(1);
                let name = self.name_of(c.member);
                let short = if pay + 0.5 < c.pay { format!(" (of the {:.0} owed: the purse ran short)", c.pay) } else { String::new() };
                self.say(t, format!("{name} is paid {:.0} coin{short}.", pay.round()));
            } else {
                self.society.contracts[i].missed = self.society.contracts[i].missed.saturating_add(1);
                if self.society.contracts[i].missed > MISSED_LIMIT {
                    ended = true;
                    if let Some(id) = c.opp {
                        self.fail_opp(id, t);
                    }
                }
            }
            self.society.contracts[i].present = 0.0;
            if t >= c.until {
                ended = true;
                if let Some(id) = c.opp {
                    self.finish_opp(id, c.member, t);
                    if let Some(q) = self.quests.iter_mut().find(|q| q.opp == Some(id)) {
                        q.stage = Stage::Done;
                    }
                }
            }
            if ended {
                self.society.contracts.remove(i);
            } else {
                i += 1;
            }
        }
    }

    // ---- Doing them ------------------------------------------------------------

    /// The open, taken jobs a conversation with `npc` could move on: (opp, what pressing does).
    pub fn pressable(&self, npc: PersonId) -> Vec<u32> {
        self.society
            .opps
            .iter()
            .filter(|o| o.state == OppState::Taken && !o.done && o.taken_by.is_some_and(|m| self.people[m as usize].in_squad))
            .filter(|o| match o.kind {
                Chance::Recover => self.society.stolen.iter().any(|s| Some(s.item) == o.item && s.event == o.event.unwrap_or(u32::MAX) && !s.recovered && s.thief == npc),
                Chance::FindOut => o.event.and_then(|e| self.event(e)).is_some_and(|e| e.hidden && e.actor.is_some_and(|a| self.knows_culprit(npc, a))),
                Chance::CollectDebt | Chance::Intimidate | Chance::Runaway => o.target == Some(npc),
                _ => false,
            })
            .map(|o| o.id)
            .collect()
    }

    /// Would this person know who did a thing done by `actor`? The doer and
    /// their household, and the ring's members if the doer is one of theirs.
    pub fn knows_culprit(&self, npc: PersonId, actor: PersonId) -> bool {
        let kin = |x: PersonId| self.society.lives.get(x as usize).and_then(|l| l.household);
        npc == actor || (kin(npc).is_some() && kin(npc) == kin(actor)) || self.society.rings.iter().any(|r| r.members.contains(&npc) && r.members.contains(&actor))
    }

    /// A squad member presses `npc` about a job. Returns what they say back.
    pub fn press(&mut self, id: u32, who: PersonId, npc: PersonId, how: Press) -> String {
        let Some(o) = self.opportunity(id).cloned() else { return "What job?".into() };
        let t = self.time;
        let day = World::day_of(t) as i32;
        let coin = items::id("coin");
        let mut r = Rng::from_keys(&[self.seed, id as u64, npc as u64, who as u64, how as u64, (t / HOUR) as u64, 0x5052_4553]);
        // Will they give way? Asking works on the honest and the friendly;
        // paying on the poor; threats on the timid (against the one threatening).
        let them = self.people[npc as usize].traits;
        let honour = self.society.lives.get(npc as usize).map(|l| l.habits.honour).unwrap_or(0.5);
        let you = self.people[who as usize].might / 100.0;
        let price = (o.amount * 0.5).max(10.0) as u16;
        let yields = match how {
            Press::Ask => r.chance((honour - 0.3).max(0.0) + self.disposition(npc, who) / 200.0 - 0.2),
            Press::Pay => {
                if self.squad_count(coin) < price {
                    return format!("You haven't {price} coin.");
                }
                r.chance(0.5 + self.mind(npc).needs()[0] * 0.5)
            }
            Press::Threaten => r.chance((0.5 + you.min(1.5) * 0.3 - them.boldness * 0.5).clamp(0.05, 0.95)),
        };
        if how == Press::Threaten {
            // Threatened is remembered, whatever comes of it.
            self.remember(npc, Who::Person(who), Deed::Threat, -0.4, day);
            self.note(Deed::Threat, Some(who), Some(npc), o.town, t, false);
        }
        if !yields {
            return match how {
                Press::Ask => "I don't know what you're talking about.".into(),
                Press::Pay => "Keep your money.".into(),
                Press::Threaten => "Do your worst. I'm not afraid of you.".into(),
            };
        }
        if how == Press::Pay {
            self.take_from_squad(coin, price);
            if let Some(h) = self.society.lives.get(npc as usize).and_then(|l| l.household) {
                self.society.households[h as usize].purse.coin += price as f32;
            }
        }
        match o.kind {
            Chance::Recover => {
                if let Some(s) = self.society.stolen.iter_mut().find(|s| Some(s.item) == o.item && Some(s.event) == o.event && !s.recovered) {
                    s.recovered = true;
                }
                if let (Some(it), Some(d)) = (o.item, self.people[who as usize].detail.as_mut()) {
                    d.gear.add(it, 1);
                }
                self.people[who as usize].recompute_might();
                self.mark_done(id);
                "Fine. Take it, and go.".into()
            }
            Chance::FindOut => {
                let ev = o.event.unwrap();
                let actor = self.event(ev).and_then(|e| e.actor);
                self.expose(ev, t);
                // One of the ring's: now the squad knows who leads it.
                if let Some(a) = actor {
                    if let Some(r) = self.society.rings.iter_mut().find(|r| r.members.contains(&a)) {
                        r.found = true;
                    }
                }
                self.mark_done(id);
                format!("It was {}. You didn't hear it from me.", actor.map(|a| self.name_of(a)).unwrap_or_default())
            }
            Chance::CollectDebt => {
                let Some(h) = self.society.lives[npc as usize].household else { return "Hm.".into() };
                let paid = o.amount.min(self.society.households[h as usize].purse.coin.max(0.0));
                if paid < 1.0 {
                    return "I've nothing. Look for yourself.".into();
                }
                self.society.households[h as usize].purse.coin -= paid;
                if let Some(ch) = self.society.lives.get(o.asker as usize).and_then(|l| l.household) {
                    self.society.households[ch as usize].purse.coin += paid;
                    let debts = &mut self.society.households[h as usize].purse.debts;
                    if let Some(d) = debts.iter_mut().find(|d| d.to == super::lives::Creditor::Household(ch)) {
                        d.amount -= paid;
                        d.dodged = false;
                    }
                    debts.retain(|d| d.amount > 0.01);
                }
                if let Some(x) = self.opp_mut(id) {
                    x.amount -= paid;
                }
                if o.amount - paid > 0.5 {
                    return format!("Here — {paid:.0}. It's all there is. The rest when I have it.");
                }
                self.mark_done(id);
                format!("Here — {paid:.0}. Tell them we're square.")
            }
            Chance::Intimidate => {
                self.remember(npc, Who::Person(who), Deed::Threat, -0.3, day);
                self.mark_done(id);
                "All right! All right. I'll keep my head down.".into()
            }
            Chance::Runaway => {
                let holder = o.asker;
                self.bond(npc, o.town, Some(holder), t, t + o.amount.max(1.0) as f64 * DAY);
                self.society.runaways.retain(|x| x.0 != npc);
                self.mark_done(id);
                "I'll go back. Don't hurt me.".into()
            }
            _ => "Hm.".into(),
        }
    }

    /// A job's work is done: go back to the asker.
    fn mark_done(&mut self, id: u32) {
        if let Some(o) = self.opp_mut(id) {
            o.done = true;
        }
        if let Some(q) = self.quests.iter_mut().find(|q| q.opp == Some(id)) {
            q.stage = Stage::Report;
        }
    }

    /// Notice jobs done out in the world: a target dead.
    pub(super) fn update_opps(&mut self) {
        for i in 0..self.society.opps.len() {
            let o = &self.society.opps[i];
            if o.state == OppState::Taken && !o.done && o.kind == Chance::Kill && o.target.is_some_and(|p| self.people[p as usize].dead) {
                let id = o.id;
                self.mark_done(id);
            }
        }
    }

    /// The squad hands in a finished job (on reporting back): the asker
    /// remembers it, the town too.
    pub(super) fn finish_opp(&mut self, id: u32, who: PersonId, t: f64) {
        let Some(o) = self.opportunity(id).cloned() else { return };
        if matches!(o.state, OppState::Done | OppState::Failed) {
            return;
        }
        let day = World::day_of(t) as i32;
        self.opp_mut(id).unwrap().state = OppState::Done;
        self.remember(o.asker, Who::Person(who), Deed::JobDone, THANKS, day);
        if o.legal {
            self.add_standing(who, o.town, JOB_STANDING);
        }
        // Coin from the giver's purse (a favour owed is a good turn remembered).
        if !o.favour && o.reward > 0 && !matches!(o.kind, Chance::Guard) {
            match o.giver {
                Giver::Ring(town) => {
                    if let Some(r) = self.society.rings.iter_mut().find(|r| r.town == town) {
                        r.purse -= o.reward as f32;
                    }
                }
                _ => {
                    if let Some(h) = self.society.lives.get(o.asker as usize).and_then(|l| l.household) {
                        self.society.households[h as usize].purse.coin -= o.reward as f32;
                    }
                }
            }
        } else if o.favour {
            *self.regard.entry(o.asker).or_insert(0.0) += 10.0;
        }
        // Deeds against someone are remembered by them — and, if it comes out,
        // against whoever asked for it.
        if let (Some(target), Chance::Kill) = (o.target, o.kind) {
            let ev = self.note(Deed::Killing, Some(who), Some(target), o.town, t, true);
            let _ = ev;
        }
        // A stolen thing goes back to its owner.
        if o.kind == Chance::Recover {
            if let (Some(it), true) = (o.item, self.squad_count(o.item.unwrap_or(0)) > 0) {
                self.take_from_squad(it, 1);
            }
        }
    }

    /// A job taken and not done.
    pub(super) fn fail_opp(&mut self, id: u32, t: f64) {
        let Some(o) = self.opportunity(id).cloned() else { return };
        if matches!(o.state, OppState::Done | OppState::Failed) {
            return;
        }
        self.opp_mut(id).unwrap().state = OppState::Failed;
        if let Some(who) = o.taken_by {
            self.remember(o.asker, Who::Person(who), Deed::JobFailed, LET_DOWN, World::day_of(t) as i32);
            let name = self.name_of(who);
            self.say(t, format!("{name} let {} down.", self.name_of(o.asker)));
        }
        if let Some(q) = self.quests.iter_mut().find(|q| q.opp == Some(id)) {
            q.stage = Stage::Done;
        }
    }

    /// An event comes out: who did it is known, and those who knew of it
    /// learn who.
    pub fn expose(&mut self, ev: u32, t: f64) {
        let Some(e) = self.event(ev).cloned() else { return };
        if !e.hidden {
            return;
        }
        if let Some(evs) = self.society.history.towns.get_mut(e.town as usize) {
            if let Some(x) = evs.iter_mut().find(|x| x.id == ev) {
                x.hidden = false;
            }
        }
        let day = World::day_of(t) as i32;
        if let (Some(a), Some(v)) = (e.actor, e.victim) {
            self.remember(v, Who::Person(a), e.deed, -e.severity, day);
        }
        // A hired deed that comes out: the wronged household turns on whoever
        // asked for it too.
        if let Some(o) = self.society.opps.iter().find(|o| o.target == e.victim && o.kind == Chance::Kill && o.state == OppState::Done).cloned() {
            if let (Some(v), Some(gh)) = (e.victim.and_then(|v| self.society.lives.get(v as usize).and_then(|l| l.household)), self.society.lives.get(o.asker as usize).and_then(|l| l.household)) {
                self.feel(v, gh, -0.8, day);
            }
        }
    }

    /// At dawn: lapsed opportunities close; jobs past their time fail.
    pub(super) fn dawn_opps(&mut self, t: f64) {
        for i in 0..self.society.opps.len() {
            let o = &self.society.opps[i];
            // A killing whose target has died is done, whenever it was noticed.
            if o.state == OppState::Taken && o.kind == Chance::Kill && o.target.is_some_and(|p| self.people[p as usize].dead) && !o.done {
                let id = o.id;
                self.mark_done(id);
            }
            let o = &self.society.opps[i];
            if o.deadline > t {
                continue;
            }
            match o.state {
                OppState::Open => self.society.opps[i].state = OppState::Lapsed,
                OppState::Taken if !o.done && o.kind != Chance::Guard => {
                    let id = o.id;
                    self.fail_opp(id, t);
                }
                // Done but never reported back: let it go.
                OppState::Taken if o.done && o.deadline + REPORT_DAYS * DAY <= t => self.society.opps[i].state = OppState::Lapsed,
                _ => {}
            }
        }
        // Long-closed ones are forgotten.
        self.society.opps.retain(|o| matches!(o.state, OppState::Open | OppState::Taken) || o.deadline > t - 10.0 * DAY);
    }

    // ---- Who'd join ------------------------------------------------------------

    /// People in a town who'd join the squad if asked (`join_terms`).
    pub fn would_join(&self, town: SettlementId) -> Vec<PersonId> {
        self.settlements[town as usize].residents.iter().copied().filter(|&p| self.join_terms(p).is_some()).collect()
    }
}
