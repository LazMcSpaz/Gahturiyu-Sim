//! Who rules a town, how wrongs are settled, and bondage (Society Part 3).
//!
//! **Government** is chosen from the town's blend like any custom. Each
//! people that leans toward a form of rule adds to it — elders, priestesses,
//! a rotating speaker — and every form strong enough in the blend gets a
//! **chamber** (a stilt village always has its speaker). Where no form holds
//! a clear majority, the chambers sit together as a **shared council**, seats
//! by head-count, with a Ṭaḍoro **arbiter** to break deadlocks. Each matter
//! (land, sea, rites, defence, records) belongs to whichever part leans to
//! it: land to the elders, the sea to the speaker, rites and defence to the
//! priestesses, records to the arbiter. So mixed towns come out with mixed
//! governments on their own: elders over the land and a speaker over the sea,
//! priestesses over rites with elders over land, and so on.
//!
//! The officeholders are real people: elders are the heads of the oldest
//! households (a seat passes at death); the high priestess is a Qotiro woman,
//! with Qotiro men as administrators below; the speaker changes each season
//! (a poor one is passed over); councillors are their people's eldest.
//! Everything is settled at dawn, on the world's timeline.
//!
//! **Rites.** Where priestesses rule, a rite is held every few days. How well
//! the town is doing weighs on it; a failed rite is a bad omen that can bring
//! the high priestess down — unless an arbiter rules the omen false.
//!
//! **Unrest** is tallied each dawn: hunger, an unpaid watch (and the crime an
//! unwatched town gets) and failed rites raise it; full stomachs and a paid
//! watch let it settle. At `REVOLT_AT` the town rises: its rulers are thrown
//! out and others take their places.
//!
//! **Wrongs** are settled by the wronged party's custom: elder judgement
//! (a fine; a temporary bond if it can't be paid), a duel to knockout fought
//! with the full combat rules, shunning (the village turns its back), or the
//! public record (it follows the offender wherever arbiters reach). Townsfolk
//! take their own disputes to a hearing when a grudge climbs that far
//! (`memory.rs`).
//!
//! **Bondage** runs for a term, from the clock; where slavery is allowed a
//! bond can be sold on into slavery — except that Roduro law (wherever the
//! elders sit) forbids selling a Roduro debtor into it. The bonded can run;
//! a stilt village that doesn't hold with slavery shelters them.
//!
//! **Standing** is what a squad member has earned in a town (favours, needed
//! goods sold, debts paid, the town defended), less their crimes; it opens
//! the hall, then buying land, then a voice in council. Leading also needs
//! the right birth (the canon rule): an elder is Roduro, a priestess a Qotiro
//! woman (Qotiro men administer), a speaker Horaro, an arbiter Ṭaḍoro.

use serde::{Deserialize, Serialize};

use super::combat::{Battle, Fighter};
use super::culture::{self, Blend, Justice, Rule, Slavery, RULES};
use super::jobs::Job;
use super::person::PersonId;
use super::race::Race;
use super::rng::{self, Rng};
use super::settlement::SettlementId;
use super::world::{World, DAY};

// ---- Dials -----------------------------------------------------------------

/// A form of rule this strong in the blend (times a keyed roll from 0.5 to
/// 1.5) gets a chamber; the strongest always does.
pub const CHAMBER_MIN: f32 = 0.2;
/// Under this, no form holds a clear majority: the chambers sit as a council.
pub const MAJORITY: f32 = 0.55;
/// Seats on a shared council.
pub const COUNCIL_SEATS: u32 = 7;
/// How many elders sit; how many priestesses (the first is high priestess);
/// how many administrators.
pub const ELDERS: usize = 3;
pub const PRIESTESSES: usize = 3;
pub const ADMINISTRATORS: usize = 2;
/// Re-form the government once this share of the people has changed.
pub const GOV_SHIFT: f32 = 0.15;
/// A season, in days (the speaker changes with it).
pub const SEASON_DAYS: i64 = 12;
/// Below this sociability a villager makes a poor speaker and is passed over.
pub const POOR_SPEAKER: f32 = 0.2;
/// Rites: how often (days), the chance when all's well, and what weighs on it.
pub const RITE_EVERY: i64 = 6;
pub const RITE_BASE: f32 = 0.9;
pub const RITE_HUNGER: f32 = 0.6;
pub const RITE_UNPAID: f32 = 0.15;
pub const RITE_UNREST: f32 = 0.35;
/// Chance an arbiter rules a bad omen false.
pub const ARBITER_STEADY: f32 = 0.5;
/// Unrest (0..100): what each dawn adds and takes away.
pub const U_HUNGER: f32 = 25.0;
pub const U_UNPAID: f32 = 12.0;
pub const U_CRIME: f32 = 2.0;
/// Extra wrongs a day in a town whose watch has gone unpaid.
pub const UNWATCHED_CRIME: f32 = 3.0;
pub const U_CALM: f32 = 4.0;
pub const U_FADE: f32 = 0.08;
pub const U_RITE: f32 = 10.0;
pub const U_SALE: f32 = 6.0;
pub const U_ESCAPE: f32 = 2.0;
/// Unrest at which the town rises, what's left of it after, and how long the
/// new rulers have before the town can rise again (days).
pub const REVOLT_AT: f32 = 80.0;
pub const REVOLT_LEFT: f32 = 0.5;
pub const REVOLT_GRACE: f64 = 6.0;
/// Chance a hearing finds for the one who brought it (where it isn't a duel).
pub const HEARING_UPHELD: f32 = 0.65;
/// Chance a fined townsperson can't pay and is bonded instead, and the term.
pub const CANT_PAY: f32 = 0.4;
pub const BOND_DAYS: (f32, f32) = (6.0, 24.0);
/// A bond's worth of work, coin a day (what a squad member's shortfall
/// becomes in days, and what buying one out costs).
pub const BOND_DAY_VALUE: f32 = 8.0;
pub const BOND_MAX_DAYS: f32 = 30.0;
/// Chance a day a bond is sold on (where slavery is allowed), and that the
/// bonded run.
pub const SALE_CHANCE: f32 = 0.1;
pub const ESCAPE_CHANCE: f32 = 0.04;
/// Days a shunning lasts.
pub const SHUN_DAYS: f64 = 6.0;
/// Standing steps: heard at the hall, buy land or a house, a voice in council.
pub const HEARD: f32 = 5.0;
pub const LAND: f32 = 20.0;
pub const COUNCIL: f32 = 50.0;
/// A public record weighs this much on standing wherever an arbiter sits.
pub const RECORD_WEIGHT: f32 = 5.0;
/// How far from town a bonded squad member can go before they've run.
pub const BOND_REACH: f32 = 150.0;
/// The longest a duel goes on, seconds.
pub const DUEL_SECS: f64 = 180.0;

// ---- State -----------------------------------------------------------------

/// What a part of government decides.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Matter {
    Land,
    Sea,
    Rites,
    Defence,
    Records,
}
pub const MATTERS: [Matter; 5] = [Matter::Land, Matter::Sea, Matter::Rites, Matter::Defence, Matter::Records];

impl Matter {
    pub fn name(self) -> &'static str {
        match self {
            Matter::Land => "land and inheritance",
            Matter::Sea => "the sea",
            Matter::Rites => "rites and the dead",
            Matter::Defence => "defence",
            Matter::Records => "records",
        }
    }
}

/// Who a matter belongs to.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Chamber(Rule),
    Council,
    Arbiter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Chamber {
    pub rule: Rule,
    /// Elders; priestesses (the first is high priestess); the season's speaker.
    pub holders: Vec<PersonId>,
    /// Under priestesses: the men who run day-to-day business.
    pub administrators: Vec<PersonId>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Government {
    pub chambers: Vec<Chamber>,
    /// The chambers sit together, seats by head-count.
    pub council: bool,
    /// Council seats per people (by `Race::index`).
    pub seats: [u8; 4],
    pub councillors: Vec<PersonId>,
    pub arbiter: Option<PersonId>,
    pub matters: Vec<(Matter, Owner)>,
    /// Head-counts the government was formed from.
    pub counted: [u32; 4],
    /// 0..100, as of the last dawn.
    pub unrest: f32,
    pub revolts: u32,
    /// Thrown out by a revolt or a bad omen: not chosen again.
    pub fallen: Vec<PersonId>,
    /// Wrongs since the last dawn.
    pub wrongs: f32,
    /// The last rite: (day, held well?).
    pub last_rite: Option<(i64, bool)>,
    /// Sales of Roduro debtors into slavery that Roduro law stopped.
    pub sales_blocked: u32,
    /// The stilt village has withdrawn from the town until then.
    pub shunned_until: f64,
    /// When the town last rose.
    #[serde(default)]
    pub last_revolt: Option<f64>,
}

impl Government {
    /// Add to (or take from) unrest, kept within 0..100.
    pub fn stir(&mut self, x: f32) {
        self.unrest = (self.unrest + x).clamp(0.0, 100.0);
    }
}

/// A bond: someone works for a holder until a set time (or for life).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Bond {
    pub who: PersonId,
    pub town: SettlementId,
    /// Who they work for (a household's head), if anyone in particular.
    pub holder: Option<PersonId>,
    pub slave: bool,
    /// Written down by an arbiter (term-limited however it's held).
    pub recorded: bool,
    pub since: f64,
    /// Free from then (infinite for a slave).
    pub until: f64,
}

/// A duel the law set, being fought now.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Duel {
    pub battle: u32,
    pub accused: PersonId,
    pub town: SettlementId,
    pub fine: f32,
}

/// Kinds of wrong.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrong {
    Theft,
    Trespass,
    Assault,
    Murder,
    /// Carrying stolen goods (traceable by maker's marks). Not yet detected:
    /// the hook for later.
    StolenGoods,
    /// Harming an arbiter: very serious everywhere.
    HarmArbiter,
    /// Raising the Roduro dead.
    Desecration,
    /// Running from a bond.
    Escape,
}

impl Wrong {
    pub fn name(self) -> &'static str {
        match self {
            Wrong::Theft => "theft",
            Wrong::Trespass => "trespass",
            Wrong::Assault => "assault",
            Wrong::Murder => "murder",
            Wrong::StolenGoods => "carrying stolen goods",
            Wrong::HarmArbiter => "harming an arbiter",
            Wrong::Desecration => "desecrating the dead",
            Wrong::Escape => "running from a bond",
        }
    }
    /// So grave it's written down wherever arbiters reach, however it's judged.
    pub fn always_recorded(self) -> bool {
        matches!(self, Wrong::HarmArbiter | Wrong::Murder | Wrong::Desecration)
    }

    /// Its weight on the public record.
    pub fn gravity(self) -> f32 {
        match self {
            Wrong::Trespass | Wrong::Escape => 1.0,
            Wrong::Theft | Wrong::StolenGoods => 2.0,
            Wrong::Assault => 3.0,
            Wrong::Desecration => 6.0,
            Wrong::Murder => 8.0,
            Wrong::HarmArbiter => 10.0,
        }
    }
}

/// A post a squad member might take.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Post {
    Elder,
    Priestess,
    Administrator,
    Speaker,
    Arbiter,
}

impl Post {
    pub fn name(self) -> &'static str {
        match self {
            Post::Elder => "elder",
            Post::Priestess => "priestess",
            Post::Administrator => "administrator",
            Post::Speaker => "speaker",
            Post::Arbiter => "arbiter",
        }
    }
}

/// Is this person a woman? (Rolled once from who they are.)
pub fn woman(seed: u64) -> bool {
    Rng::from_keys(&[seed, 0x5345_5821]).f32() < 0.5
}

/// How old a home is, years (the older, the louder its elder's voice).
pub fn home_age(seed: u64, town: SettlementId, building: u16) -> f32 {
    10.0 + 290.0 * Rng::from_keys(&[seed, town as u64, building as u64, 0x4147_4521]).f32().powf(1.5)
}

impl World {
    // ---- Who's who -------------------------------------------------------------

    fn living_here(&self, town: SettlementId) -> Vec<PersonId> {
        self.settlements[town as usize].residents.iter().copied().filter(|&p| !self.people[p as usize].dead && self.people[p as usize].home == Some(town)).collect()
    }

    /// Living residents of each people.
    pub fn town_counts(&self, town: SettlementId) -> [u32; 4] {
        let mut n = [0u32; 4];
        for p in self.living_here(town) {
            n[self.people[p as usize].race.index()] += 1;
        }
        n
    }

    pub fn government(&self, town: SettlementId) -> &Government {
        &self.society.towns[town as usize].gov
    }

    /// Who owns a matter in a town.
    pub fn owner_of(&self, town: SettlementId, m: Matter) -> Option<Owner> {
        self.government(town).matters.iter().find(|x| x.0 == m).map(|x| x.1)
    }

    /// Does Roduro law hold here (do elders sit)?
    pub fn roduro_law(&self, town: SettlementId) -> bool {
        self.government(town).chambers.iter().any(|c| c.rule == Rule::Elders)
    }

    /// Still fit to hold office here: alive, and living here (or one of your
    /// squad who took the post).
    fn seated(&self, town: SettlementId, p: PersonId) -> bool {
        let pp = &self.people[p as usize];
        !pp.dead && (pp.home == Some(town) || pp.in_squad)
    }

    /// Candidates for a chamber, best first.
    fn candidates(&self, town: SettlementId, rule: Rule, admins: bool, t: f64) -> Vec<PersonId> {
        let gov = self.government(town);
        let here: Vec<PersonId> = self.living_here(town).into_iter().filter(|p| !gov.fallen.contains(p) && !self.is_bonded(*p, t)).collect();
        let age = |p: PersonId| self.society.lives[p as usize].age;
        let race = |p: PersonId| self.people[p as usize].race;
        let mut out: Vec<(f32, PersonId)> = match rule {
            // The heads (eldest) of households, oldest home first.
            Rule::Elders => {
                let mut heads: Vec<(f32, PersonId)> = Vec::new();
                for h in &self.society.households {
                    let Some(&head) = h.members.iter().filter(|&&m| here.contains(&m)).max_by_key(|&&m| (age(m), std::cmp::Reverse(m))) else { continue };
                    if race(head) != Race::Roduro || self.society.communities[h.community as usize].town != town {
                        continue;
                    }
                    let home = h.home.map(|b| home_age(self.seed, town, b)).unwrap_or(5.0);
                    heads.push((home * 1000.0 + age(head) as f32, head));
                }
                heads
            }
            Rule::Priestesses => here
                .iter()
                .filter(|&&p| race(p) == Race::Qotiro && woman(self.people[p as usize].seed) != admins)
                .map(|&p| {
                    let calling = matches!(self.life(p).job, Job::Priest | Job::Official);
                    ((calling as u8 as f32) * 1000.0 + age(p) as f32, p)
                })
                .collect(),
            Rule::Speaker => {
                // The villagers in a fixed order; the season picks one.
                let mut v: Vec<(f32, PersonId)> = here.iter().filter(|&&p| race(p) == Race::Horaro).map(|&p| ((rng::key(&[self.seed, p as u64, 0x5350_4B52]) >> 40) as f32, p)).collect();
                v.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                return v.into_iter().map(|x| x.1).collect();
            }
        };
        out.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        out.into_iter().map(|x| x.1).collect()
    }

    /// This season's speaker: the next in turn, passing over poor speakers.
    fn speaker_for(&self, town: SettlementId, day: i64, t: f64) -> Option<PersonId> {
        let order = self.candidates(town, Rule::Speaker, false, t);
        if order.is_empty() {
            return None;
        }
        let season = day.div_euclid(SEASON_DAYS) as usize;
        (0..order.len()).map(|k| order[(season + k) % order.len()]).find(|&p| self.people[p as usize].traits.sociability >= POOR_SPEAKER).or(Some(order[season % order.len()]))
    }

    // ---- Forming a government ------------------------------------------------

    /// Choose a town's form of government from its blend, and seat it.
    pub(super) fn form_government(&mut self, town: SettlementId, t: f64) {
        let counts = self.town_counts(town);
        let key = culture::community_key(town, false) ^ 0x474F_5645;
        let blend = Blend::of(counts, self.seed, key);
        let w = blend.ruling;
        let top = (0..3).max_by(|&a, &b| w[a].total_cmp(&w[b]).then(b.cmp(&a))).unwrap_or(0);
        let has_stilts = self.society.towns[town as usize].stilts.is_some();
        let mut chambers = Vec::new();
        for (k, &rule) in RULES.iter().enumerate() {
            let roll = Rng::from_keys(&[self.seed, town as u64, k as u64, 0x4348_414D]).f32();
            let present = (k == top && w[k] > 0.0) || w[k] >= CHAMBER_MIN * (0.5 + roll) || (rule == Rule::Speaker && has_stilts);
            if present {
                chambers.push(Chamber { rule, holders: Vec::new(), administrators: Vec::new() });
            }
        }
        let council = chambers.is_empty() || (chambers.len() >= 2 && w.iter().all(|&x| x < MAJORITY));
        let tadoro = counts[Race::Tadoro.index()] > 0;
        let has = |r: Rule| chambers.iter().any(|c: &Chamber| c.rule == r);
        let biggest = chambers.iter().max_by(|a, b| w[a.rule as usize].total_cmp(&w[b.rule as usize])).map(|c| Owner::Chamber(c.rule));
        let fallback = if council { Some(Owner::Council) } else { biggest };
        let mut matters = Vec::new();
        for m in MATTERS {
            let own = match m {
                Matter::Land if has(Rule::Elders) => Some(Owner::Chamber(Rule::Elders)),
                Matter::Sea if has(Rule::Speaker) => Some(Owner::Chamber(Rule::Speaker)),
                Matter::Rites | Matter::Defence if has(Rule::Priestesses) => Some(Owner::Chamber(Rule::Priestesses)),
                Matter::Defence if has(Rule::Elders) => Some(Owner::Chamber(Rule::Elders)),
                Matter::Records if tadoro => Some(Owner::Arbiter),
                _ => fallback,
            };
            if let Some(o) = own {
                matters.push((m, o));
            }
        }
        // Council seats in proportion to head-count (largest remainder).
        let total: u32 = counts.iter().sum::<u32>().max(1);
        let mut seats = [0u8; 4];
        if council {
            let exact: Vec<f32> = counts.iter().map(|&c| c as f32 * COUNCIL_SEATS as f32 / total as f32).collect();
            let mut given = 0u32;
            for r in 0..4 {
                seats[r] = exact[r].floor() as u8;
                given += seats[r] as u32;
            }
            let mut order: Vec<usize> = (0..4).collect();
            order.sort_by(|&a, &b| (exact[b] - exact[b].floor()).total_cmp(&(exact[a] - exact[a].floor())).then(a.cmp(&b)));
            for &r in order.iter().take((COUNCIL_SEATS - given) as usize) {
                seats[r] += 1;
            }
        }
        // Whoever still sits in a chamber that survives keeps their seat.
        let old = self.government(town).chambers.clone();
        let mut chambers = chambers;
        for c in chambers.iter_mut() {
            if let Some(o) = old.iter().find(|o| o.rule == c.rule) {
                c.holders = o.holders.iter().copied().filter(|&p| self.seated(town, p)).collect();
                c.administrators = o.administrators.iter().copied().filter(|&p| self.seated(town, p)).collect();
            }
        }
        let gov = &mut self.society.towns[town as usize].gov;
        gov.chambers = chambers;
        gov.council = council;
        gov.seats = seats;
        gov.councillors.clear();
        gov.matters = matters;
        gov.counted = counts;
        self.fill_offices(town, t);
    }

    /// Fill any empty seat (the dead, those who left, the season's new speaker).
    pub(super) fn fill_offices(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t);
        let gov = self.government(town).clone();
        let mut chambers = gov.chambers.clone();
        for c in chambers.iter_mut() {
            c.holders.retain(|&p| self.seated(town, p) && !gov.fallen.contains(&p));
            c.administrators.retain(|&p| self.seated(town, p) && !gov.fallen.contains(&p));
            match c.rule {
                Rule::Speaker => {
                    // Your own speaker keeps the voice; otherwise the season's turn.
                    if !c.holders.iter().any(|&p| self.people[p as usize].in_squad) {
                        c.holders = self.speaker_for(town, day, t).into_iter().collect();
                    }
                }
                rule => {
                    let want = if rule == Rule::Elders { ELDERS } else { PRIESTESSES };
                    for p in self.candidates(town, rule, false, t) {
                        if c.holders.len() >= want {
                            break;
                        }
                        if !c.holders.contains(&p) {
                            c.holders.push(p);
                        }
                    }
                    if rule == Rule::Priestesses {
                        for p in self.candidates(town, rule, true, t) {
                            if c.administrators.len() >= ADMINISTRATORS {
                                break;
                            }
                            if !c.administrators.contains(&p) {
                                c.administrators.push(p);
                            }
                        }
                    }
                }
            }
        }
        // Councillors: each people's eldest, as many as their seats.
        let mut councillors: Vec<PersonId> = gov.councillors.iter().copied().filter(|&p| self.seated(town, p) && !gov.fallen.contains(&p)).collect();
        if gov.council {
            for race in super::race::ALL_RACES {
                let seats = gov.seats[race.index()] as usize;
                let have = councillors.iter().filter(|&&p| self.people[p as usize].race == race).count();
                let mut pool: Vec<PersonId> = self.living_here(town).into_iter().filter(|&p| self.people[p as usize].race == race && !councillors.contains(&p) && !gov.fallen.contains(&p)).collect();
                pool.sort_by_key(|&p| (std::cmp::Reverse(self.society.lives[p as usize].age), p));
                councillors.extend(pool.into_iter().take(seats.saturating_sub(have)));
            }
        }
        // The arbiter: a Ṭaḍoro arbiter by trade first, else the eldest Ṭaḍoro.
        let mut arbiter = gov.arbiter.filter(|&p| self.seated(town, p));
        if arbiter.is_none() {
            let mut pool: Vec<PersonId> = self.living_here(town).into_iter().filter(|&p| self.people[p as usize].race == Race::Tadoro).collect();
            pool.sort_by_key(|&p| (self.life(p).job != Job::Arbiter, std::cmp::Reverse(self.society.lives[p as usize].age), p));
            arbiter = pool.first().copied();
        }
        let g = &mut self.society.towns[town as usize].gov;
        g.chambers = chambers;
        g.councillors = councillors;
        g.arbiter = arbiter;
    }

    // ---- The dawn tally ------------------------------------------------------

    /// Each dawn: re-form the government if the people have shifted, fill
    /// seats, hold any rite, settle disputes, bonds and unrest.
    pub(super) fn dawn_law(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t);
        if culture::shift(self.government(town).counted, self.town_counts(town)) >= GOV_SHIFT {
            let before: Vec<Rule> = self.government(town).chambers.iter().map(|c| c.rule).collect();
            self.form_government(town, t);
            let after: Vec<Rule> = self.government(town).chambers.iter().map(|c| c.rule).collect();
            if before != after {
                let name = self.settlements[town as usize].name.clone();
                self.say(t, format!("{name} is governed anew: {}.", self.gov_words(town)));
            }
        } else {
            self.fill_offices(town, t);
        }
        // The stilt village comes back once a shunning has run its course.
        if let Some(si) = self.society.towns[town as usize].stilts {
            let until = self.government(town).shunned_until;
            if until > 0.0 && t >= until {
                self.society.communities[si as usize].withdrawn = false;
                self.society.towns[town as usize].gov.shunned_until = 0.0;
            }
        }
        if self.government(town).chambers.iter().any(|c| c.rule == Rule::Priestesses) && day.rem_euclid(RITE_EVERY) == (town as i64).rem_euclid(RITE_EVERY) {
            self.hold_rite(town, t);
        }
        self.settle_bonds(town, t);
        self.tally_unrest(town, t);
    }

    /// How likely a rite is to go well, by how the town is doing.
    pub fn rite_chance(&self, town: SettlementId) -> f32 {
        let tl = &self.society.towns[town as usize];
        let food = self.town_food(town);
        (RITE_BASE - RITE_HUNGER * (1.0 - food).max(0.0) - if tl.owed > 0.0 { RITE_UNPAID } else { 0.0 } - RITE_UNREST * tl.gov.unrest / 100.0).clamp(0.05, 0.98)
    }

    fn hold_rite(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t);
        let ok = Rng::from_keys(&[self.seed, town as u64, day as u64, 0x5249_5445]).f32() < self.rite_chance(town);
        self.society.towns[town as usize].gov.last_rite = Some((day, ok));
        if !ok {
            self.bad_omen(town, t);
        }
    }

    /// A bad omen — a failed rite, or (later) one reported or faked by
    /// others. An arbiter may rule it false; otherwise the high priestess
    /// falls and the next takes her place.
    pub fn bad_omen(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t);
        let name = self.settlements[town as usize].name.clone();
        let gov = self.government(town);
        let Some(ci) = gov.chambers.iter().position(|c| c.rule == Rule::Priestesses) else { return };
        if gov.arbiter.is_some() && Rng::from_keys(&[self.seed, town as u64, day as u64, 0x5354_4459]).f32() < ARBITER_STEADY {
            self.society.towns[town as usize].gov.stir(U_RITE * 0.25);
            self.say(t, format!("A bad omen in {name}; the arbiter rules it was no true sign."));
            return;
        }
        let gov = &mut self.society.towns[town as usize].gov;
        gov.stir(U_RITE);
        if gov.chambers[ci].holders.is_empty() {
            return;
        }
        let fallen = gov.chambers[ci].holders.remove(0);
        gov.fallen.push(fallen);
        let who = self.name_of(fallen);
        self.fill_offices(town, t);
        self.say(t, format!("The rite fails in {name}. The high priestess {who} is brought down."));
    }

    fn tally_unrest(&mut self, town: SettlementId, t: f64) {
        let food = self.town_food(town);
        let tl = &self.society.towns[town as usize];
        let guards = self.living_here(town).iter().filter(|&&p| self.life(p).job == Job::Guard).count() as f32;
        // The share of the watch's pay that was met.
        let paid = if tl.owed <= 0.0 { 1.0 } else { (1.0 - tl.owed / (guards * super::economy::GUARD_WAGE).max(1.0)).clamp(0.0, 1.0) };
        let wrongs = tl.gov.wrongs + UNWATCHED_CRIME * (1.0 - paid);
        let fed = food >= 0.95 && paid >= 0.99;
        let gov = &mut self.society.towns[town as usize].gov;
        let du = U_HUNGER * (1.0 - food).max(0.0) + U_UNPAID * (1.0 - paid) + U_CRIME * wrongs - if fed { U_CALM } else { 0.0 };
        gov.unrest = (gov.unrest * (1.0 - U_FADE) + du).clamp(0.0, 100.0);
        gov.wrongs = 0.0;
        let settled_in = gov.last_revolt.map(|r| t - r >= REVOLT_GRACE * DAY).unwrap_or(true);
        if gov.unrest >= REVOLT_AT && settled_in {
            self.revolt(town, t);
        }
    }

    /// The town rises: its rulers are thrown out, others take their seats.
    pub fn revolt(&mut self, town: SettlementId, t: f64) {
        let gov = &mut self.society.towns[town as usize].gov;
        let out: Vec<PersonId> = gov.chambers.iter().flat_map(|c| c.holders.iter().chain(&c.administrators)).chain(&gov.councillors).copied().collect();
        gov.fallen.extend(out.iter().copied());
        // Officials and priests thrown out lose their posts.
        for &p in &out {
            if matches!(self.society.lives[p as usize].job, Job::Official | Job::Priest) {
                let job = self.society.lives[p as usize].job;
                self.society.lives[p as usize].job = Job::None;
                self.society.lives[p as usize].place = None;
                if let Some(m) = self.society.minds.get_mut(p as usize) {
                    m.lost = Some((job, super::lives::Loss::Revolt));
                }
            }
        }
        let gov = &mut self.society.towns[town as usize].gov;
        let n = gov.fallen.len();
        if n > 60 {
            gov.fallen.drain(..n - 60);
        }
        for c in gov.chambers.iter_mut() {
            c.holders.clear();
            c.administrators.clear();
        }
        gov.councillors.clear();
        gov.unrest *= REVOLT_LEFT;
        gov.revolts += 1;
        gov.last_revolt = Some(t);
        self.fill_offices(town, t);
        let name = self.settlements[town as usize].name.clone();
        self.say(t, format!("{name} rises! Its rulers are thrown out: {} now.", self.rulers_words(town)));
    }

    // ---- Townsfolk's disputes ------------------------------------------------

    /// A dispute brought to a hearing by `wronged` against `offender` (a
    /// grudge's second rung, `memory.rs`), settled by the wronged party's
    /// custom. True if the offender lost.
    pub(super) fn public_dispute(&mut self, offender: PersonId, wronged: PersonId, town: SettlementId, t: f64, r: &mut Rng) -> bool {
        let day = World::day_of(t);
        self.society.towns[town as usize].gov.wrongs += 1.0;
        let custom = self.justice_for(town, wronged, r);
        let lost = match custom {
            Justice::Duel => self.npc_duel(offender, wronged, t, rng::key(&[self.seed, offender as u64, wronged as u64, day as u64, 0x4455_454C])),
            _ => r.chance(HEARING_UPHELD),
        };
        if !lost {
            return false;
        }
        match custom {
            Justice::Elders | Justice::Duel => {
                if r.chance(CANT_PAY) && !self.is_bonded(offender, t) {
                    let days = r.range(BOND_DAYS.0, BOND_DAYS.1) as f64;
                    let holder = self.head_of(wronged);
                    self.bond(offender, town, holder, t, t + days * DAY);
                }
            }
            Justice::Shunning => {}
            Justice::Record => *self.records.entry(offender).or_insert(0.0) += 1.0,
        }
        true
    }

    /// Whose law: the wronged party's community custom; where that's unclear
    /// (two ways nearly even) and an arbiter sits, the arbiter rules — which
    /// means the record.
    pub(super) fn justice_for(&self, town: SettlementId, wronged: PersonId, r: &mut Rng) -> Justice {
        let ci = self.society.lives[wronged as usize].community.unwrap_or(self.society.towns[town as usize].shore);
        let c = &self.society.communities[ci as usize];
        let mut w = c.blend.justice;
        w.sort_by(|a, b| b.total_cmp(a));
        if w[0] - w[1] < 0.15 && self.government(town).arbiter.is_some() && r.chance(0.5) {
            return Justice::Record;
        }
        c.customs.justice
    }

    /// A duel between townsfolk, fought out at once with the full combat
    /// rules, to knockout. True if the offender lost.
    pub(super) fn npc_duel(&mut self, offender: PersonId, wronged: PersonId, t: f64, seed: u64) -> bool {
        let a = Fighter::from_person(&self.people[offender as usize], 0, super::geo::V2::new(0.0, 0.0), t);
        let b = Fighter::from_person(&self.people[wronged as usize], 1, super::geo::V2::new(3.0, 0.0), t);
        let mut battle = Battle::new(u32::MAX, seed, t, vec![a, b], vec![self.name_of(offender), self.name_of(wronged)]);
        battle.advance_to(t + DUEL_SECS);
        // To knockout: wounds stay, but no one dies of a duel.
        for f in &mut battle.fighters {
            f.dead = false;
        }
        let lost = !battle.fighters[0].active() || (battle.fighters[1].active() && battle.fighters[0].hp.iter().sum::<f32>() < battle.fighters[1].hp.iter().sum::<f32>());
        // (Spirits called up in it aren't people.)
        for f in battle.fighters.iter().filter(|f| f.is_person()) {
            let p = &mut self.people[f.pid as usize];
            let base = p.stats.clone();
            let mut hp = f.hp;
            for x in hp.iter_mut() {
                *x = x.max(1.0);
            }
            p.wounds.set(&base, &hp, battle.time);
        }
        lost
    }

    /// The head of someone's household (its eldest).
    fn head_of(&self, p: PersonId) -> Option<PersonId> {
        let h = self.society.lives[p as usize].household?;
        self.society.households.get(h as usize)?.members.iter().copied().filter(|&m| !self.people[m as usize].dead).max_by_key(|&m| (self.society.lives[m as usize].age, std::cmp::Reverse(m)))
    }

    // ---- Bonds -----------------------------------------------------------------

    /// Is someone bound right now?
    pub fn is_bonded(&self, p: PersonId, t: f64) -> bool {
        self.bonds.iter().any(|b| b.who == p && b.until > t && b.since <= t)
    }

    pub fn bond_of(&self, p: PersonId) -> Option<&Bond> {
        self.bonds.iter().find(|b| b.who == p && b.until > self.time)
    }

    /// Bind someone for a term. Where bonds are recorded, the record says so.
    pub fn bond(&mut self, who: PersonId, town: SettlementId, holder: Option<PersonId>, t: f64, until: f64) {
        let ci = self.society.towns[town as usize].shore;
        let recorded = self.society.communities[ci as usize].customs.slavery == Slavery::Recorded;
        self.bonds.retain(|b| b.who != who || b.until <= t);
        self.bonds.push(Bond { who, town, holder, slave: false, recorded, since: t, until });
        if self.people[who as usize].in_squad {
            let name = self.name_of(who);
            let place = self.settlements[town as usize].name.clone();
            self.say(t, format!("{name} is bound to work in {place} for {:.0} days.", (until - t) / DAY));
        }
    }

    /// Try to sell bond `k` on into slavery, to a household where slavery is
    /// allowed. Roduro law (wherever elders sit) forbids it for a Roduro
    /// debtor; a recorded bond stays term-limited.
    pub fn sell_bond(&mut self, k: usize, t: f64) -> Result<(), &'static str> {
        let b = *self.bonds.get(k).ok_or("no such bond")?;
        if b.slave || b.until <= t {
            return Err("not a bond in force");
        }
        if b.recorded {
            return Err("the record keeps it term-limited");
        }
        let town = b.town;
        // A buyer: the head of a household whose people allow slavery.
        let buyer = self
            .society
            .households
            .iter()
            .filter(|h| self.society.communities[h.community as usize].town == town && self.society.communities[h.community as usize].customs.slavery == Slavery::Allowed)
            .filter_map(|h| h.members.iter().copied().filter(|&m| !self.people[m as usize].dead && m != b.who).max_by_key(|&m| (self.society.lives[m as usize].age, std::cmp::Reverse(m))))
            .next()
            .ok_or("no one here keeps slaves")?;
        let roduro_debtor = self.people[b.who as usize].race == Race::Roduro;
        if roduro_debtor && self.roduro_law(town) {
            self.society.towns[town as usize].gov.sales_blocked += 1;
            return Err("Roduro law forbids it");
        }
        let name = self.settlements[town as usize].name.clone();
        let bonds = &mut self.bonds[k];
        bonds.slave = true;
        bonds.until = f64::INFINITY;
        bonds.holder = Some(buyer);
        // In a mixed town it's a sore point; a villager sold off wrongs the village.
        let mixed = self.government(town).chambers.len() >= 2;
        if roduro_debtor || mixed {
            self.society.towns[town as usize].gov.stir(U_SALE);
        }
        let from_stilts = self.society.lives[b.who as usize].community.map(|c| self.society.communities[c as usize].stilts).unwrap_or(false);
        if from_stilts {
            self.wrong_village(town, t);
        }
        let _ = name;
        Ok(())
    }

    /// Bonds that end, are sold on, or are run from, at dawn.
    fn settle_bonds(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t);
        for k in 0..self.bonds.len() {
            let b = self.bonds[k];
            if b.town != town || b.until <= t || self.people[b.who as usize].in_squad {
                continue;
            }
            let mut r = Rng::from_keys(&[self.seed, b.who as u64, day as u64, 0x424F_4E44]);
            if !b.slave && r.chance(SALE_CHANCE) {
                let _ = self.sell_bond(k, t);
            }
            // Running: a stilt village that won't hold slaves takes them in.
            let b = self.bonds[k];
            if r.chance(ESCAPE_CHANCE) {
                if let Some(si) = self.society.towns[town as usize].stilts {
                    if self.society.communities[si as usize].customs.slavery != Slavery::Allowed {
                        self.bonds[k].until = t;
                        let holder_allows = b.holder.and_then(|h| self.society.lives[h as usize].community).map(|c| self.society.communities[c as usize].customs.slavery == Slavery::Allowed).unwrap_or(false);
                        if holder_allows {
                            self.society.towns[town as usize].gov.stir(U_ESCAPE);
                        }
                    }
                }
            }
        }
        self.bonds.retain(|b| b.until > t || self.people[b.who as usize].in_squad && b.until > t - DAY);
    }

    // ---- Shunning ----------------------------------------------------------------

    /// The town has wronged its stilt village: the village withdraws (no
    /// dawn boats, no catch) for a while.
    pub fn wrong_village(&mut self, town: SettlementId, t: f64) {
        let Some(si) = self.society.towns[town as usize].stilts else { return };
        self.society.communities[si as usize].withdrawn = true;
        let g = &mut self.society.towns[town as usize].gov;
        g.shunned_until = g.shunned_until.max(t + SHUN_DAYS * DAY);
        let name = self.settlements[town as usize].name.clone();
        self.say(t, format!("The stilt village turns its back on {name}: no boats will come ashore."));
    }

    /// Is a squad member shunned by this community?
    pub fn shunned_by(&self, p: PersonId, community: u32) -> bool {
        self.shunned.iter().any(|s| s.0 == p && s.1 == community && s.2 > self.time)
    }

    // ---- Standing and posts ------------------------------------------------------

    /// What someone has earned in a town, less what's on the public record
    /// where an arbiter sits.
    pub fn standing(&self, p: PersonId, town: SettlementId) -> f32 {
        let own = self.standing_in.get(&(p, town)).copied().unwrap_or(0.0);
        let record = if self.government(town).arbiter.is_some() { self.records.get(&p).copied().unwrap_or(0.0) * RECORD_WEIGHT } else { 0.0 };
        own - record
    }

    pub(super) fn add_standing(&mut self, p: PersonId, town: SettlementId, x: f32) {
        *self.standing_in.entry((p, town)).or_insert(0.0) += x;
    }

    /// What standing opens, in words.
    pub fn standing_word(s: f32) -> &'static str {
        if s >= COUNCIL {
            "a voice in council"
        } else if s >= LAND {
            "may buy land or a house"
        } else if s >= HEARD {
            "heard at the hall"
        } else if s < 0.0 {
            "distrusted"
        } else {
            "a stranger"
        }
    }

    /// Could this person hold this post at all (birth, the canon rule)?
    pub fn eligible(&self, p: PersonId, post: Post) -> bool {
        let pp = &self.people[p as usize];
        match post {
            Post::Elder => pp.race == Race::Roduro,
            Post::Priestess => pp.race == Race::Qotiro && woman(pp.seed),
            Post::Administrator => pp.race == Race::Qotiro && !woman(pp.seed),
            Post::Speaker => pp.race == Race::Horaro,
            Post::Arbiter => pp.race == Race::Tadoro,
        }
    }

    /// A squad member takes a post in a town: they must be eligible, have a
    /// voice in council, and the town must have that post.
    pub fn take_post(&mut self, p: PersonId, town: SettlementId, post: Post) -> Result<(), &'static str> {
        if !self.eligible(p, post) {
            return Err("not eligible for that post");
        }
        if self.standing(p, town) < COUNCIL {
            return Err("not enough standing here");
        }
        let rule = match post {
            Post::Elder => Some(Rule::Elders),
            Post::Priestess | Post::Administrator => Some(Rule::Priestesses),
            Post::Speaker => Some(Rule::Speaker),
            Post::Arbiter => None,
        };
        let g = &mut self.society.towns[town as usize].gov;
        match rule {
            None => g.arbiter = Some(p),
            Some(rule) => {
                let c = g.chambers.iter_mut().find(|c| c.rule == rule).ok_or("this town has no such post")?;
                let list = if post == Post::Administrator { &mut c.administrators } else { &mut c.holders };
                if list.contains(&p) {
                    return Err("already holds it");
                }
                if post == Post::Speaker {
                    list.clear();
                } else if !list.is_empty() {
                    list.pop();
                }
                list.push(p);
            }
        }
        let name = self.name_of(p);
        let place = self.settlements[town as usize].name.clone();
        self.say(self.time, format!("{name} takes up the post of {} in {place}.", post.name()));
        Ok(())
    }

    // ---- The squad before the law ----------------------------------------------

    /// A squad member's wrong is known in `town`. Guards on shift (if the
    /// watch is paid) arrest and judge at once by the town's custom;
    /// otherwise it stands as a bounty, and the news travels.
    pub(super) fn wrong_done(&mut self, who: PersonId, town: SettlementId, wrong: Wrong, fine: f32) -> bool {
        let t = self.time;
        self.add_standing(who, town, -fine / 4.0);
        self.society.towns[town as usize].gov.wrongs += 1.0;
        if wrong.always_recorded() {
            *self.records.entry(who).or_insert(0.0) += wrong.gravity();
        }
        let tl = &self.society.towns[town as usize];
        let paid = tl.owed <= 0.0;
        let on_watch = self.living_here(town).into_iter().any(|p| self.life(p).job == Job::Guard && self.at_work(p, t));
        let mut r = Rng::from_keys(&[self.seed, who as u64, (t / 60.0) as u64, 0x4152_5354]);
        let in_duel = self.duels.iter().any(|d| d.accused == who) || self.fighting.contains_key(&who);
        if !(on_watch && (paid || r.chance(0.3))) || self.is_bonded(who, t) || in_duel {
            return false;
        }
        // Arrested: judged by the custom of those wronged (the town on land,
        // where it happened).
        let custom = {
            let ci = self.society.towns[town as usize].shore;
            self.society.communities[ci as usize].customs.justice
        };
        self.judge(who, town, wrong, fine, custom);
        true
    }

    /// Judgement on a squad member.
    pub fn judge(&mut self, who: PersonId, town: SettlementId, wrong: Wrong, fine: f32, custom: Justice) {
        let t = self.time;
        let name = self.name_of(who);
        let place = self.settlements[town as usize].name.clone();
        match custom {
            Justice::Elders => {
                self.say(t, format!("{name} is judged by the elders of {place} for {}: a fine of {fine:.0}.", wrong.name()));
                self.pay_or_bond(who, town, fine);
            }
            Justice::Duel => {
                self.say(t, format!("{name} must answer for {} by duel in {place}.", wrong.name()));
                self.start_duel(who, town, fine);
            }
            Justice::Shunning => {
                // Whichever community holds to shunning turns its back.
                let tl = &self.society.towns[town as usize];
                let ci = std::iter::once(tl.shore).chain(tl.stilts).find(|&c| self.society.communities[c as usize].customs.justice == Justice::Shunning).unwrap_or(tl.shore);
                self.shunned.push((who, ci, t + SHUN_DAYS * DAY));
                self.say(t, format!("{place} turns its back on {name} for {}: no one will trade with them.", wrong.name()));
            }
            Justice::Record => {
                // (The gravest are on the record already.)
                if !wrong.always_recorded() {
                    *self.records.entry(who).or_insert(0.0) += wrong.gravity();
                }
                self.say(t, format!("{name}'s {} in {place} is written into the record; it will follow them.", wrong.name()));
                self.pay_or_bond(who, town, fine * 0.5);
            }
        }
    }

    /// Pay a fine from the squad's coin; what can't be paid is worked off.
    fn pay_or_bond(&mut self, who: PersonId, town: SettlementId, fine: f32) {
        let coin = super::items::id("coin");
        let have = self.squad_count(coin) as f32;
        let pay = have.min(fine).floor();
        self.take_from_squad(coin, pay as u16);
        let short = fine - pay;
        if short >= 1.0 {
            let days = (short / BOND_DAY_VALUE).clamp(1.0, BOND_MAX_DAYS) as f64;
            let t = self.time;
            self.bond(who, town, None, t, t + days * DAY);
        }
    }

    /// What it costs to buy a squad member's bond out now.
    pub fn buy_out_price(&self, who: PersonId) -> Option<u16> {
        let b = self.bond_of(who)?;
        if b.slave {
            return Some(400);
        }
        Some((((b.until - self.time) / DAY) as f32 * BOND_DAY_VALUE).ceil().max(1.0) as u16)
    }

    /// Buy a squad member out of their bond.
    pub fn buy_out(&mut self, who: PersonId) -> Result<(), &'static str> {
        let price = self.buy_out_price(who).ok_or("they aren't bound")?;
        let coin = super::items::id("coin");
        if self.squad_count(coin) < price {
            return Err("not enough coin");
        }
        self.take_from_squad(coin, price);
        let t = self.time;
        for b in self.bonds.iter_mut().filter(|b| b.who == who && b.until > t) {
            b.until = t;
        }
        let name = self.name_of(who);
        self.say(t, format!("{name} is bought out of their bond."));
        Ok(())
    }

    /// A bound squad member who strays too far from the town has run.
    pub(super) fn check_runaways(&mut self) {
        let t = self.time;
        for m in self.squad.members.clone() {
            let Some(b) = self.bond_of(m).copied() else { continue };
            let s = &self.settlements[b.town as usize];
            if self.person_pos(m).dist(s.pos) > s.reach + BOND_REACH {
                for x in self.bonds.iter_mut().filter(|x| x.who == m && x.until > t) {
                    x.until = t;
                }
                let name = self.name_of(m);
                self.say(t, format!("{name} has run from their bond in {}.", s.name));
                *self.bounty.entry(b.town).or_insert(0.0) += 60.0;
                self.crime_known(b.town);
                self.add_standing(m, b.town, -15.0);
                *self.records.entry(m).or_insert(0.0) += Wrong::Escape.gravity();
            }
        }
    }

    /// The town's champion and yours meet: the duel is fought where the
    /// accused stands, with the full combat rules, to knockout.
    fn start_duel(&mut self, accused: PersonId, town: SettlementId, fine: f32) {
        let t = self.time;
        // Champions are allowed: the strongest of the squad standing near.
        let at = self.person_pos(accused);
        let ours = self.squad_fit().into_iter().filter(|&m| self.person_pos(m).dist(at) < 30.0).max_by(|&a, &b| self.people[a as usize].might.total_cmp(&self.people[b as usize].might).then(b.cmp(&a))).unwrap_or(accused);
        let theirs = self
            .living_here(town)
            .into_iter()
            .filter(|&p| {
                let pp = &self.people[p as usize];
                !self.fighting.contains_key(&p) && !self.is_bonded(p, t) && self.busy_until[p as usize] <= t && self.group_of[p as usize].is_none() && !super::body::knocked_out(&pp.wounds.hp_at(&pp.stats, t))
            })
            .max_by(|&a, &b| {
                let guard = |p: PersonId| (self.life(p).job == Job::Guard) as u8;
                guard(a).cmp(&guard(b)).then(self.people[a as usize].might.total_cmp(&self.people[b as usize].might)).then(b.cmp(&a))
            });
        let Some(theirs) = theirs else {
            self.pay_or_bond(accused, town, fine);
            return;
        };
        let id = self.begin_duel(ours, theirs, at);
        self.duels.push(Duel { battle: id, accused, town, fine });
    }

    /// A duel has ended: the loser pays.
    pub(super) fn duel_over(&mut self, b: &Battle) {
        let Some(k) = self.duels.iter().position(|d| d.battle == b.id) else { return };
        let d = self.duels.remove(k);
        let won = b.winner() == Some(super::combat::SQUAD_SIDE);
        let name = self.name_of(d.accused);
        let t = b.time;
        if won {
            self.say(t, format!("{name}'s side wins the duel; the matter is closed."));
            self.add_standing(d.accused, d.town, 2.0);
        } else {
            self.say(t, format!("{name}'s side loses the duel, and must pay {:.0}.", d.fine * 1.5));
            self.pay_or_bond(d.accused, d.town, d.fine * 1.5);
        }
    }

    /// After a fight the squad was in: townsfolk it hurt or killed (outside
    /// the law's own duels) are wrongs where they live; Roduro dead raised in
    /// a town where the elders sit are desecrated; a town defended from
    /// bandits remembers it.
    pub(super) fn fight_wrongs(&mut self, b: &Battle) {
        use super::combat::SQUAD_SIDE;
        let squad: Vec<PersonId> = b.fighters.iter().filter(|f| f.home == SQUAD_SIDE && f.is_person() && self.people[f.pid as usize].in_squad).map(|f| f.pid).collect();
        let Some(&doer) = squad.first() else { return };
        let mut seen: Vec<(PersonId, SettlementId, Wrong)> = Vec::new();
        for f in b.fighters.iter().filter(|f| f.is_person() && f.home != SQUAD_SIDE && f.home != super::combat::GRAVE_SIDE) {
            let p = &self.people[f.pid as usize];
            if p.bandit || p.in_squad {
                continue;
            }
            let Some(town) = p.home else { continue };
            if f.damage_taken <= 0.0 && !f.dead {
                continue;
            }
            let wrong = if self.life(f.pid).job == Job::Arbiter {
                Wrong::HarmArbiter
            } else if p.dead {
                Wrong::Murder
            } else {
                Wrong::Assault
            };
            seen.push((f.pid, town, wrong));
        }
        // The dead raised by your side: Roduro dead lie beneath their homes.
        for f in b.fighters.iter().filter(|f| f.raised && f.is_person() && f.raised_by == Some(SQUAD_SIDE)) {
            if self.people[f.pid as usize].race != Race::Roduro {
                continue;
            }
            if let Some(town) = self.town_at(f.pos).filter(|&t| self.roduro_law(t)) {
                seen.push((f.pid, town, Wrong::Desecration));
            }
        }
        // One charge per town: the gravest wrong, every fine together.
        let mut charges: Vec<(SettlementId, Wrong, f32)> = Vec::new();
        for (_, town, wrong) in seen {
            let fine = match wrong {
                Wrong::Assault => 60.0,
                Wrong::Murder => 300.0,
                Wrong::HarmArbiter => 400.0,
                Wrong::Desecration => 200.0,
                _ => 40.0,
            };
            match charges.iter_mut().find(|c| c.0 == town) {
                Some(c) => {
                    c.2 += fine;
                    if wrong.gravity() > c.1.gravity() {
                        c.1 = wrong;
                    }
                }
                None => charges.push((town, wrong, fine)),
            }
        }
        for (town, wrong, fine) in charges {
            let name = self.name_of(doer);
            self.crime_of(doer, town, wrong, fine, format!("{name}'s side is seen at {}!", wrong.name()));
        }
        // Bandits beaten by a town: it's grateful.
        if b.winner() == Some(SQUAD_SIDE) && b.fighters.iter().any(|f| f.is_person() && self.people[f.pid as usize].bandit) {
            let at = b.fighters.first().map(|f| f.pos).unwrap_or_default();
            if let Some(town) = self.settlements.iter().filter(|s| s.pos.dist(at) <= s.reach + 600.0).min_by(|a, c| a.pos.dist(at).total_cmp(&c.pos.dist(at))).map(|s| s.id) {
                for &m in &squad {
                    self.add_standing(m, town, 3.0);
                }
            }
        }
    }

    // ---- Words, for the window -------------------------------------------------

    /// "Elder circle and rotating speaker, in council".
    pub fn gov_words(&self, town: SettlementId) -> String {
        let g = self.government(town);
        let parts: Vec<&str> = g.chambers.iter().map(|c| c.rule.name()).collect();
        let mut s = if parts.is_empty() { "a shared council".to_string() } else { parts.join(" and ") };
        if g.council && !parts.is_empty() {
            s += ", in shared council";
        }
        if g.arbiter.is_some() {
            s += ", with a Ṭaḍoro arbiter";
        }
        s
    }

    /// The leading officeholders by name.
    pub fn rulers_words(&self, town: SettlementId) -> String {
        let g = self.government(town);
        let mut out = Vec::new();
        for c in &g.chambers {
            let names: Vec<String> = c.holders.iter().map(|&p| self.name_of(p)).collect();
            if !names.is_empty() {
                let what = match c.rule {
                    Rule::Elders => "elders",
                    Rule::Priestesses => "high priestess",
                    Rule::Speaker => "speaker",
                };
                let names = if c.rule == Rule::Priestesses { names[..1].to_vec() } else { names };
                out.push(format!("{what} {}", names.join(", ")));
            }
        }
        if out.is_empty() {
            "no one".into()
        } else {
            out.join("; ")
        }
    }
}
